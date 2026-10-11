"""Thin adapters to the Rust sm-pipeline worker/cache module and legacy builders."""
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
_local = threading.local()
_binary = None


def within(path, root):
    return Path(path) == Path(root) or Path(root) in Path(path).parents


def shared_root():
    if os.environ.get('SM_PIPELINE_ROOT'):
        return Path(os.environ['SM_PIPELINE_ROOT']).resolve()
    common = subprocess.check_output(['git', 'rev-parse', '--git-common-dir'], cwd=ROOT, text=True).strip()
    return (ROOT / common).resolve().parent / 'out/pipeline'


def binary():
    global _binary
    if os.environ.get('SM_PIPELINE_BINARY'):
        return Path(os.environ['SM_PIPELINE_BINARY']).resolve()
    root = shared_root()
    if _binary and _binary[0] == root:
        return _binary[1]
    root.mkdir(parents=True, exist_ok=True)
    inputs = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'crates/pipeline/Cargo.toml', ROOT / 'rust-toolchain.toml']
    inputs += sorted((ROOT / 'crates/pipeline/src').rglob('*.rs'))
    identity = ''.join(sha(p) for p in inputs) + subprocess.check_output(['rustc', '-vV'], cwd=ROOT, text=True)
    fingerprint = hashlib.sha256(identity.encode()).hexdigest()
    # Different tool revisions in simultaneous worktrees retain different
    # executables. A later bootstrap must not replace a worker's selected CLI.
    location = root / 'bootstrap' / fingerprint
    location.mkdir(parents=True, exist_ok=True)
    destination = location / 'sm-pipeline'
    with (root / 'bootstrap.lock').open('a') as held:
        fcntl.flock(held, fcntl.LOCK_EX)
        seal = location / 'bootstrap.json'
        if seal.exists() and destination.exists():
            saved = json.loads(seal.read_text())
            if saved['source'] == fingerprint and saved['binary'] == sha(destination):
                _binary = (root, destination)
                return destination
        target = root / 'bootstrap-target'
        subprocess.run(['cargo', 'build', '-p', 'pipeline', '--locked', '--offline', '--release',
                        '--jobs', '1', '--target-dir', str(target)], cwd=ROOT, check=True)
        shutil.copy2(target / 'release/sm-pipeline', destination)
        seal.write_text(json.dumps({'source':fingerprint,'binary':sha(destination)})+'\n')
        shutil.rmtree(target)
    _binary = (root, destination)
    return destination


@contextlib.contextmanager
def lease():
    if getattr(_local, 'leased', False):
        yield 0.0
        return
    requested = str(os.environ.get('SM_PIPELINE_WORKERS', min(4, os.cpu_count() or 1)))
    process = subprocess.Popen([str(binary()), 'lease', str(shared_root()), requested],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    try:
        result = json.loads(process.stdout.readline())
        _local.leased = True
        yield result['worker_wait_seconds']
    finally:
        _local.leased = False
        process.stdin.close()
        process.stdout.close()
        if process.wait() != 0:
            raise ValueError('Rust worker lease failed')


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as source:
        for chunk in iter(lambda: source.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def cache_artifact(command, inputs, destination, build):
    started = time.monotonic()
    request = {'root':str(shared_root()),'checkout':str(ROOT),'command':command,
               'inputs':[str(Path(p).resolve()) for p in inputs], 'destination':str(Path(destination).resolve())}
    process = subprocess.Popen([str(binary()), 'cache'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    try:
        process.stdin.write(json.dumps(request)+'\n');process.stdin.flush()
        reply = json.loads(process.stdout.readline())
        if not reply['hit']:
            build(Path(reply['artifact']))
            process.stdin.write('commit\n');process.stdin.flush()
        completed = json.loads(process.stdout.readline())
        if process.wait() != 0:
            raise ValueError('Rust compiler cache failed')
        return dict(completed,seconds=time.monotonic()-started)
    finally:
        process.stdin.close()
        process.stdout.close()
        process.wait()


def cargo_artifact(package, member, destination, execute):
    if package not in ('evidence', 'decomp'):
        raise ValueError('Cargo cache input closure is defined only for evidence and decomp')
    inputs = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'rust-toolchain.toml', ROOT / 'crates' / package / 'Cargo.toml']
    inputs += list((ROOT / 'crates' / package / 'src').rglob('*.rs'))
    inputs += list((ROOT / '.cargo').glob('*'))
    inputs += list((ROOT / 'crates').glob('*/Cargo.toml'))
    inputs += list((ROOT / 'deps/rustysnes').glob('crates/rustysnes-core/Cargo.toml'))
    inputs = [p for p in inputs if p.is_file()]
    command = ['cargo', 'build', '--locked', '--offline', '--release', '-p', package]
    def build(artifact):
        target = artifact.parent / 'target'
        execute(command + ['--jobs', '1', '--target-dir', str(target)], ROOT, package + '-build')
        shutil.copy2(target / 'release' / member, artifact)
        shutil.rmtree(target)
    return cache_artifact(command, inputs, destination, build)


def invoke(operation, request):
    result = subprocess.run([str(binary()), operation], input=json.dumps(request)+'\n',
                            stdout=subprocess.PIPE, text=True, check=True)
    return json.loads(result.stdout)


def cached_c_build(command, run, execute):
    return invoke('c-build', {'command': command, 'run': str(Path(run).resolve()),
                            'checkout': str(ROOT), 'root': str(shared_root()),
                            'workers': int(os.environ.get('SM_PIPELINE_WORKERS', min(4, os.cpu_count() or 1)))})
