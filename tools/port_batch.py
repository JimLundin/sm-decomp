#!/usr/bin/env python3
"""Repeatable, resumable port verification and cumulative replay orchestration."""
import argparse
import fcntl
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import sys
import os
import time
import hashlib
import signal
from contextlib import ExitStack
from port_evidence import atomic_json

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / 'tools/port/batch.json'
LATEST = ROOT / 'out/port/batches/latest.json'


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    atomic_json(path, value)


def read_batch(path, create=False, profile='gameplay'):
    if path is None and LATEST.exists():
        path = Path(json.loads(LATEST.read_text())['batch'])
    if path is None:
        if not create:
            raise ValueError('No batch yet; run tools/port_batch.py verify first.')
        stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
        path = ROOT / 'out/port/batches' / stamp / 'batch.json'
    path = path.resolve()
    if path.exists():
        return path, json.loads(path.read_text())
    if not create:
        raise ValueError(f'Batch does not exist: {path}')
    batch = {'profile': profile, 'families': {}, 'replays': []}
    save(path, batch)
    save(LATEST, {'batch': str(path)})
    return path, batch


def execute(argv, directory, label):
    started = time.monotonic()
    directory.mkdir(parents=True, exist_ok=True)
    print(f'{label}: starting', flush=True)
    with (directory / 'stdout.log').open('wb') as stdout, (directory / 'stderr.log').open('wb') as stderr:
        process = subprocess.Popen(argv, cwd=ROOT, stdout=stdout, stderr=stderr, start_new_session=True)
        try:
            while True:
                try:
                    code = process.wait(timeout=30)
                    break
                except subprocess.TimeoutExpired:
                    print(f'{label}: still running (logs: {directory})', flush=True)
        except BaseException:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            raise
    verdicts = sorted(directory.glob('evidence/*/verdict.json'))
    record = {'command': argv, 'exit_code': code, 'logs': str(directory), 'seconds': time.monotonic()-started}
    if verdicts:
        record['verdict'] = str(verdicts[-1])
        record['status'] = json.loads(verdicts[-1].read_text())['status']
    else:
        record['status'] = 'failed' if code else 'complete'
    print(f'{label}: {record["status"]} (logs: {directory})', flush=True)
    return record


def validate(kind, verdict):
    if kind in ('family', 'offline-family'):
        from port_family import verify_family_evidence
        return verify_family_evidence(verdict)
    if kind == 'callback':
        from port_collision import verify_collision_evidence
        return verify_collision_evidence(verdict)
    if kind == 'power-bomb':
        from port_power_bomb import verify_power_bomb_evidence
        return verify_power_bomb_evidence(verdict)
    if kind == 'plm':
        from port_plm import verify_plm_evidence
        return verify_plm_evidence(verdict)
    from port_plm_instructions import verify_plm_instruction_evidence
    return verify_plm_instruction_evidence(verdict)


def verification_command(kind, manifest, output):
    script = {'family': 'port_family.py', 'offline-family': 'port_family.py',
              'callback': 'port_collision.py', 'plm': 'port_plm.py',
              'power-bomb': 'port_power_bomb.py',
              'plm-instructions': 'port_plm_instructions.py', 'dispatcher': 'port_plm_instructions.py'}[kind]
    command = [sys.executable, str(ROOT / 'tools' / script)]
    if kind in ('family', 'offline-family'):
        command.append('verify')
    return command + ['--manifest', str(manifest), '--source', str(ROOT / 'deps/snesrev-sm'), '--output', str(output)]


def catalog_entry(config, name):
    entry = config['families'][name]
    if isinstance(entry, str):
        return {'kind': entry, 'manifest': 'tools/port/' + name + '.json',
                'live': {'family': 'family', 'callback': 'callback', 'dispatcher': 'dispatcher'}.get(entry)}
    return entry


def generator_digest(entry):
    command = entry.get('prepare')
    if not command:
        return None
    sources = {}
    for arg in command + entry.get('prepare_inputs', []):
        path = Path(arg.replace('{root}', str(ROOT)))
        if not path.is_absolute():
            path = ROOT / path
        if path.is_file():
            sources[str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    return hashlib.sha256(json.dumps({'command': command, 'sources': sources}, sort_keys=True).encode()).hexdigest()


def catalog(config):
    """Discover offline manifests; only explicit catalog adapters grant live use."""
    config = json.loads(json.dumps(config))
    for path in sorted((ROOT / 'tools/port').glob('*.json')):
        if path.stem in config['families']:
            continue
        manifest = json.loads(path.read_text())
        if not manifest.get('rust_module') or not manifest.get('routines'):
            continue
        # Dispatcher signatures and host state use the existing bounded harness.
        symbols = {r['symbol'] for r in manifest['routines']}
        kind = 'dispatcher' if symbols & {'EprojInstruction', 'EprojPreinstruction', 'PlmInstruction', 'PlmFrame', 'PlmControlFrame'} else 'family'
        config['families'][path.stem] = {'kind': kind, 'manifest': str(path.relative_to(ROOT)), 'live': None}
    config['profiles']['offline-all'] = list(config['families'])
    return config


def verify_batch(path, batch, config, names, jobs=3, fresh=False):
    from port_family import digest
    def verify_one(name):
        entry = catalog_entry(config, name)
        kind = entry['kind']
        manifest = ROOT / entry['manifest']
        generator_sha256 = (digest(ROOT / 'tools/port_plm_fixtures.py') if kind == 'plm-instructions' else
                            generator_digest(entry))
        previous = batch['families'].get(name)
        if (previous and not fresh and previous.get('manifest_sha256') == digest(manifest)
                and previous.get('generator_sha256') == generator_sha256):
            try:
                if previous.get('exit_code') != 0:
                    raise ValueError('previous worker did not complete successfully')
                validate(kind, Path(previous['verdict']))
                print(f'{name}: reused authenticated evidence', flush=True)
                return name, previous
            except (OSError, ValueError, KeyError, TypeError):
                print(f'{name}: stale evidence; rebuilding', flush=True)
        stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
        directory = path.parent / name / stamp
        selected_manifest = manifest
        if entry.get('prepare'):
            command = [arg.replace('{root}', str(ROOT)).replace('{output}', str(directory / 'fixtures')).replace('{manifest}', str(directory / 'fixtures/manifest.json')) for arg in entry['prepare']]
            prepared = execute(command, directory / 'prepare', name + ' fixtures')
            if prepared['exit_code']:
                return name, prepared
            selected_manifest = directory / 'fixtures/manifest.json'
        if kind == 'plm-instructions':
            selected_manifest = directory / 'fixtures/manifest.json'
            generated = execute([sys.executable, str(ROOT / 'tools/port_plm_fixtures.py'),
                                 '--output-dir', str(directory / 'fixtures'), '--manifest', str(selected_manifest)],
                                directory / 'prepare', name + ' fixtures')
            if generated['exit_code']:
                return name, generated
        record = execute(verification_command(kind, selected_manifest, directory / 'evidence'), directory, name)
        record.update(kind=kind, manifest_sha256=digest(manifest), generator_sha256=generator_sha256)
        if not record['exit_code']:
            try:
                validate(kind, Path(record['verdict']))
            except (OSError, ValueError, KeyError, TypeError) as error:
                record.update(status='failed', exit_code=1, error=str(error))
        return name, record
    failures = False
    with ThreadPoolExecutor(max_workers=jobs) as executor:
        pending = {executor.submit(verify_one, name): name for name in names}
        for future in as_completed(pending):
            name = pending[future]
            try:
                name, record = future.result()
            except Exception as error:
                record = {'status': 'failed', 'exit_code': 1, 'error': str(error)}
                print(f'{name}: failed: {error}', flush=True)
            batch['families'][name] = record
            failures |= bool(record['exit_code'])
            save(path, batch)
    return 1 if failures else 0


def replay_command(batch, config, names, frames, output, native_plm=False, room_plm=False):
    from port_family import digest

    def require_current_manifest(name, record):
        manifest = ROOT / catalog_entry(config, name)['manifest']
        if record.get('manifest_sha256') != digest(manifest):
            raise ValueError(f'{name}: manifest changed since verification; run verify first')

    command = [sys.executable, str(ROOT / 'tools/port_live.py'), '--frames', str(frames), '--output', str(output)]
    contracts = 0
    for name in names:
        entry = catalog_entry(config, name)
        kind = entry.get('live')
        if kind not in ('family', 'callback', 'dispatcher'):
            continue
        record = batch['families'].get(name, {})
        if record.get('exit_code') != 0 or not record.get('verdict'):
            raise ValueError(f'{name}: missing passing evidence; run verify first')
        require_current_manifest(name, record)
        # The live consumer authenticates every retained artifact before building.
        command += [{'callback': '--collision-verdict', 'family': '--family-verdict', 'dispatcher': '--dispatcher-verdict'}[kind], record['verdict']]
        contracts += 1
    if not contracts:
        raise ValueError('No live-eligible families selected')
    if native_plm or room_plm:
        record = batch['families'].get('plm-spawn', {})
        if record.get('exit_code') != 0 or not record.get('verdict'):
            raise ValueError('Native PLM requires passing plm-spawn evidence')
        require_current_manifest('plm-spawn', record)
        command += ['--plm-verdict', record['verdict']]
    if room_plm:
        command.append('--room-plm')
    return command


def run_main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('verify', 'replay', 'run', 'status', 'check'))
    parser.add_argument('--batch', type=Path, help='Batch record; defaults to latest, creating the first on verify/run')
    parser.add_argument('--profile', default='gameplay')
    parser.add_argument('--families', nargs='+', help='Verify/replay a subset from the catalog')
    parser.add_argument('--jobs', type=int, default=3)
    parser.add_argument('--fresh', action='store_true', help='Rebuild instead of reusing authenticated evidence')
    parser.add_argument('--frames', type=int, default=18000)
    parser.add_argument('--room-plm', action='store_true', help='Require native room/hardcoded PLM setup execution')
    parser.add_argument('--native-plm', action='store_true', help='Require exercised native PLM setup in replay')
    parser.add_argument('--live-jobs', type=int, default=4)
    parser.add_argument('--case-jobs', type=int, default=1)
    parser.add_argument('--changed-since', help='Select catalog families touched since this Git revision')
    args = parser.parse_args()
    try:
        if args.command == 'check':
            stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
            directory = ROOT / 'out/port/checks' / stamp
            commands = [['cargo', 'fmt', '-p', 'port', '-p', 'pipeline', '-p', 'evidence', '--', '--check'],
                        ['cargo', 'test', '-p', 'port', '-p', 'pipeline', '-p', 'evidence', '--locked', '--offline'],
                        ['cargo', 'clippy', '-p', 'port', '-p', 'pipeline', '-p', 'evidence', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
                        [sys.executable, '-m', 'unittest', 'discover', '-s', 'tools', '-p', 'test_port*.py']]
            for label, command in zip(('format', 'rust-tests', 'clippy', 'harness-tests'), commands):
                result = execute(command, directory / label, label)
                if result['exit_code']:
                    return result['exit_code']
            return 0
        config = catalog(json.loads(CONFIG.read_text()))
        names = list(dict.fromkeys(args.families or config['profiles'][args.profile]))
        if args.changed_since:
            changed = set(subprocess.check_output(['git', 'diff', '--name-only', args.changed_since], cwd=ROOT, text=True).splitlines())
            selected = []
            for name in names:
                entry = catalog_entry(config, name)
                manifest = json.loads((ROOT / entry['manifest']).read_text())
                module = Path(manifest['rust_module'])
                from port_inputs import wrapper_memory
                import tempfile
                with tempfile.TemporaryDirectory() as temporary:
                    generated = wrapper_memory(Path(temporary), [module if module.is_absolute() else ROOT / module], manifest.get('rust_imports', []))
                    closure = generated.read_text()
                dependencies = {str(module if module.is_absolute() else ROOT / module)}
                dependencies.update(str(ROOT / p) if not Path(p).is_absolute() else p for p in manifest.get('rust_dependencies', []))
                global_change = any(p.startswith(('tools/', 'crates/pipeline/', '.cargo/')) or p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'crates/port/src/memory.rs') for p in changed)
                if global_change or entry['manifest'] in changed or any(str(ROOT / p) in closure or str(ROOT / p) in dependencies for p in changed):
                    selected.append(name)
            names = selected
        if not 1 <= args.case_jobs <= 64 or not 1 <= args.live_jobs <= 4:
            raise ValueError('case jobs must be 1..64; live jobs must be 1..4')
        os.environ['SM_PORT_CASE_JOBS'] = str(args.case_jobs)
        if (args.native_plm or args.room_plm) and 'plm-spawn' not in names:
            names.append('plm-spawn')
        if not 1 <= args.jobs <= 8 or not 1 <= args.frames <= 129712:
            raise ValueError('jobs must be 1..8; frames must be 1..129712')
        if any(name not in config['families'] for name in names):
            raise ValueError('Unknown family; see tools/port/batch.json')
        selection = ROOT / 'out/port/batches/selection.lock'
        selection.parent.mkdir(parents=True, exist_ok=True)
        with ExitStack() as ownership:
            with selection.open('a') as selecting:
                fcntl.flock(selecting, fcntl.LOCK_EX)
                path, batch = read_batch(args.batch, args.command in ('verify', 'run'), args.profile)
                lock = ownership.enter_context(path.with_suffix('.lock').open('a'))
                try:
                    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError:
                    raise ValueError('another command is writing this batch; retry after it finishes')
            return operate(args, path, batch, config, names)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'port batch: {error}', file=sys.stderr)
        return 1


def operate(args, path, batch, config, names):
    try:
        if args.command == 'status':
            print(f'Batch: {path}')
            for name, record in batch['families'].items():
                print(f'{name}: {record["status"]} {record.get("verdict", record.get("error", ""))}')
            for record in batch['replays']:
                print(f'replay: {record["status"]} {record.get("verdict", "")}')
            return 0
        if args.command in ('verify', 'run') and verify_batch(path, batch, config, names, args.jobs, args.fresh):
            return 1
        if args.command in ('replay', 'run'):
            stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
            directory = path.parent / 'replays' / stamp
            command = replay_command(batch, config, names, args.frames, directory / 'evidence', args.native_plm, args.room_plm)
            command += ['--live-jobs', str(args.live_jobs)]
            record = execute(command, directory, 'replay')
            batch['replays'].append(record)
            save(path, batch)
            from port_cleanup import deduplicate
            print(f"Reclaimed {deduplicate(path.parent, True)} snapshot bytes")
            return record['exit_code']
        if args.command == 'verify':
            from port_cleanup import deduplicate
            print(f"Reclaimed {deduplicate(path.parent, True)} snapshot bytes")
        return 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'port batch: {error}', file=sys.stderr)
        return 1


def main():
    return run_main()


if __name__ == '__main__':
    raise SystemExit(main())
