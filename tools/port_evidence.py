#!/usr/bin/env python3
"""Durable packed live observations, with legacy evidence reading support."""
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
SUBJECTS = ('c-0', 'c-1', 'mixed-0', 'mixed-1')


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def atomic_json(path, document):
    temporary = path.with_name(path.name + '.pending-' + uuid.uuid4().hex)
    try:
        with temporary.open('x') as out:
            json.dump(document, out, indent=2)
            out.write('\n')
            out.flush()
            os.fsync(out.fileno())
        os.replace(temporary, path)
        fd = os.open(str(path.parent), os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(fd)
        finally:
            os.close(fd)
    finally:
        temporary.unlink(missing_ok=True)


def build(run, execute):
    """Retain the exact reader binary used by this evidence, not Cargo outputs."""
    execute(['cargo', 'build', '--locked', '--release', '-p', 'evidence',
             '--target-dir', str(run / 'evidence-build')], ROOT, 'evidence-build')
    binary = run / 'sm-evidence'
    shutil.copy2(run / 'evidence-build/release/sm-evidence', binary)
    shutil.rmtree(run / 'evidence-build')
    return binary


@contextlib.contextmanager
def store_session(store):
    """Serialize pipeline writers without stealing the crate's crash lock."""
    store.mkdir(parents=True, exist_ok=True)
    if store.is_symlink():
        raise ValueError('evidence store is a symlink')
    with (store / 'pipeline.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def packed_replay(command, directory, binary, store, provenance, timeout):
    """Stream C events to Rust, then seal all staged observations before release."""
    stage = directory / '.evidence-stage'
    stage.mkdir()
    (stage / "fixtures").mkdir()
    (stage / "saves").mkdir()
    fifo = directory / '.effects.pipe'
    os.mkfifo(fifo, 0o600)
    provenance_path = directory / 'evidence-provenance.json'
    atomic_json(provenance_path, provenance)
    candidate = directory / '.evidence-candidate.json'
    trace_receipt = directory / 'trace-receipt.json'
    pointer_path = directory / 'evidence.json'
    started = time.monotonic()
    metrics = {}
    # O_RDWR prevents an early child exit from hanging FIFO open. The sentinel is
    # closed after C exits, so the Rust reader then sees EOF and seals its trace.
    with store_session(store):
        before_bytes = sum(p.stat().st_size for p in store.glob("*.pack"))
        sentinel = os.open(str(fifo), os.O_RDWR | os.O_NONBLOCK)
        source = fifo.open('rb', buffering=0)
        collector = producer = None
        try:
            with (directory / 'evidence.stderr').open('wb') as err:
                collector = subprocess.Popen([str(binary), 'capture', str(store), str(trace_receipt),
                                              str(stage), str(provenance_path), str(candidate)],
                                             stdin=source, stdout=subprocess.PIPE, stderr=err)
                # A successful store open is required before starting a producer.
                import select
                if not select.select([collector.stdout], [], [], 60)[0] or collector.stdout.readline() != b'ready\n':
                    raise ValueError('evidence collector failed to open the store')
                env = dict(os.environ, SM_EVIDENCE_PIPE=str(fifo), SM_EVIDENCE_STAGING=str(stage))
                with (directory / 'replay.stdout').open('wb') as out, (directory / 'replay.stderr').open('wb') as err:
                    producer = subprocess.Popen(command, cwd=directory, stdout=out, stderr=err, env=env)
                    deadline = time.monotonic() + timeout
                    while producer.poll() is None:
                        if collector.poll() is not None:
                            raise ValueError('evidence collector exited during replay')
                        if time.monotonic() >= deadline:
                            raise subprocess.TimeoutExpired(command, timeout)
                        time.sleep(0.05)
                    code = producer.returncode
                    metrics["producer_seconds"] = time.monotonic() - started
                    sealing_started = time.monotonic()
                os.close(sentinel)
                sentinel = None
                code_collector = collector.wait(timeout=timeout)
            if code or code_collector:
                raise ValueError(f'replay/collector failed with exits {code}/{code_collector}')
            pointer = json.loads(candidate.read_text())
            pointer.update(store=os.path.relpath(store, directory), provenance=provenance)
            # Compare restored members to every staged file before discarding bytes.
            staged = {str(p.relative_to(stage)): digest(p) for p in stage.rglob('*') if p.is_file()}
            if staged != {k: v for k, v in pointer['artifacts'].items() if k != 'hardware.log'}:
                raise ValueError('packed restoration differs from staged observations')
            pointer['pipeline_seconds'] = time.monotonic() - started
            pointer["producer_seconds"] = metrics["producer_seconds"]
            pointer["seal_seconds"] = time.monotonic() - sealing_started
            pointer["stored_added_bytes"] = sum(p.stat().st_size for p in store.glob("*.pack")) - before_bytes
            atomic_json(pointer_path, pointer)
            shutil.rmtree(stage)
            candidate.unlink()
            trace_receipt.unlink()
            provenance_path.unlink()
            return pointer
        finally:
            for process in (producer, collector):
                if process is not None and process.poll() is None:
                    process.kill()
                    process.wait()
            if collector is not None and collector.stdout is not None:
                collector.stdout.close()
            source.close()
            if sentinel is not None:
                os.close(sentinel)
            fifo.unlink(missing_ok=True)


def read_pointer(directory):
    pointer_path = directory / 'evidence.json'
    if pointer_path.is_symlink():
        raise ValueError('evidence pointer is a symlink')
    pointer = json.loads(pointer_path.read_text())
    if pointer.get('format') != 'sm-evidence-v1':
        raise ValueError('unknown evidence format')
    return pointer


def authenticate_subject(directory, expected=None, pointer_sha256=None):
    if pointer_sha256 and digest(directory / 'evidence.json') != pointer_sha256:
        raise ValueError('subject pointer changed')
    pointer = read_pointer(directory)
    store = (directory / pointer['store']).resolve()
    binary = directory.parent / 'sm-evidence'
    if pointer.get('provenance', {}).get('reader_sha256') and digest(binary) != pointer['provenance']['reader_sha256']:
        raise ValueError('retained evidence reader changed')
    result = subprocess.run([str(binary), 'inventory', str(store), pointer['run']],
                            capture_output=True, text=True, check=True)
    actual = json.loads(result.stdout)
    for key in ('observations', 'files', 'bytes', 'artifacts', 'provenance'):
        if actual[key] != pointer[key]:
            raise ValueError(f'packed evidence {key} differs')
    if expected is not None and actual['artifacts'] != expected:
        raise ValueError('packed observation inventory differs')
    return pointer


def materialize(directory, destination, member=None, frame=None):
    pointer = read_pointer(directory)
    binary = directory.parent / 'sm-evidence'
    store = (directory / pointer['store']).resolve()
    if pointer.get('provenance', {}).get('reader_sha256') and digest(binary) != pointer['provenance']['reader_sha256']:
        raise ValueError('retained evidence reader changed')
    if member is None:
        subprocess.run([str(binary), 'extract', str(store), pointer['run'], str(destination)], check=True)
    else:
        command = [str(binary), 'frame' if frame is not None else 'cat', str(store), pointer['run'], member]
        if frame is not None:
            command.append(str(frame))
        with destination.open('xb') as out:
            subprocess.run(command, stdout=out, check=True)


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('subject', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--member')
    parser.add_argument('--frame', type=int)
    args = parser.parse_args()
    if args.frame is not None and args.member is None:
        parser.error('--frame requires --member')
    materialize(args.subject.resolve(), args.destination.resolve(), args.member, args.frame)


if __name__ == '__main__':
    main()
