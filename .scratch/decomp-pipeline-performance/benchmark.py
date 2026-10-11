#!/usr/bin/env python3
"""Study harness: actual frozen replays; exact inventories, never new admission.

Run from any directory. Outputs contain derived observations and stay under out/.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import contextlib
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
from port_evidence import SUBJECTS, packed_replay
from port_authenticate_live import authenticate


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_counters(directory, frozen, name):
    lines = (directory / 'replay.stdout').read_text().splitlines()
    rows = [line.split() for line in lines if line.startswith('port-hit ')]
    assert len(rows) == len(frozen['hits'][name])
    actual = [{'c': int(row[2]), 'rust': int(row[3])} for row in rows]
    assert actual == list(frozen['hits'][name].values()), f'{name}: ownership changed'
    progress = [json.loads(line) for line in lines if line.startswith('{')]
    assert progress == frozen['progress'][name], f'{name}: progress changed'


def live(args):
    import port_evidence
    original_session = port_evidence.store_session
    worker_metrics = threading.local()
    @contextlib.contextmanager
    def measured_session(store):
        started = time.monotonic()
        with original_session(store):
            worker_metrics.lock_wait_seconds = time.monotonic() - started
            yield
    port_evidence.store_session = measured_session
    baseline = args.baseline.resolve()
    authenticated = authenticate(baseline)
    frozen = json.loads(baseline.read_text())
    assert digest(args.rom.resolve()) == frozen['rom_sha256']
    assert digest(args.inputs.resolve()) == frozen['inputs_sha256']
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    samples = []
    for repeat in range(args.repeats):
        for label, workers, shared in (('serial-shared', 1, True),
                                       ('parallel-shared', 4, True),
                                       ('parallel-private', 4, False)):
            run = output / f'{repeat}-{label}'
            run.mkdir()
            for name in ('live', 'sm-evidence'):
                shutil.copy2(baseline.parent / name, run / name)
            def subject(name):
                directory = run / name
                directory.mkdir()
                old = json.loads((baseline.parent / name / 'evidence.json').read_text())
                store = run / ('store' if shared else f'store-{name}')
                command = [str(run / 'live'), name.split('-')[0], str(args.rom.resolve()),
                           str(args.inputs.resolve()), str(frozen['frames']),
                           str(directory / '.evidence-stage/terminal.bin')]
                pointer = packed_replay(command, directory, run / 'sm-evidence', store,
                                        old['provenance'], 240)
                assert pointer['artifacts'] == old['artifacts'], f'{name}: inventory changed'
                assert pointer['observations'] == old['observations'], f'{name}: root changed'
                assert pointer['run'] == old['run'], f'{name}: provenance root changed'
                check_counters(directory, frozen, name)
                pointer['measured_lock_wait_seconds'] = worker_metrics.lock_wait_seconds
                return name, pointer
            started = time.monotonic()
            load_before = Path('/proc/loadavg').read_text().strip()
            with ThreadPoolExecutor(max_workers=workers) as executor:
                pointers = dict(executor.map(subject, SUBJECTS))
            elapsed = time.monotonic() - started
        # Publish a study verdict readable by the normal historical authenticator.
            verdict = dict(frozen)
            verdict['commands'] = []
            verdict['study_only'] = 'frozen binaries; no current-source admission'
            verdict['evidence_storage']['subjects'] = {
                name: {'run': p['run'], 'observations': p['observations'],
                       'pointer_sha256': digest(run / name / 'evidence.json')}
                for name, p in pointers.items()}
            path = run / 'verdict.json'
            path.write_text(json.dumps(verdict, indent=2) + '\n')
            auth_started = time.monotonic()
            check = authenticate(path)
            auth_seconds = time.monotonic() - auth_started
            stored = sum(p.stat().st_size for p in run.glob('store*/*.pack'))
            row = dict(label=label, repeat=repeat, workers=workers, shared_store=shared,
                       wall_seconds=elapsed, authentication_seconds=auth_seconds,
                       stored_bytes=stored, verdict=str(path), verdict_sha256=digest(path),
                       exact_inventory=True, exact_roots=True,
                       exact_progress_and_ownership=True, authentication=check,
                       host_load_before=load_before,
                       host_load_after=Path('/proc/loadavg').read_text().strip(),
                       subjects={name: {key: p[key] for key in ('pipeline_seconds',
                           'producer_seconds', 'seal_seconds', 'stored_added_bytes',
                           'measured_lock_wait_seconds')}
                           for name, p in pointers.items()})
            samples.append(row)
            print(json.dumps({k: row[k] for k in ('label', 'repeat', 'wall_seconds',
                'authentication_seconds', 'stored_bytes', 'exact_inventory')}), flush=True)
            (output / 'live-results.json').write_text(json.dumps(dict(baseline=str(baseline),
                baseline_authentication=authenticated, samples=samples), indent=2) + '\n')


def offline(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    rows = []
    for repeat in range(args.repeats):
        for workers in (1, 2):
            run = output / f'{repeat}-offline-{workers}'
            run.mkdir()
            def family(name):
                command = [sys.executable, str(Path(__file__).resolve()), 'offline-family',
                           '--manifest', str(ROOT / 'tools/port' / f'{name}.json'),
                           '--source', str(args.source.resolve()), '--output', str(run / name)]
                started = time.monotonic()
                with (run / f'{name}.stdout').open('wb') as out, (run / f'{name}.stderr').open('wb') as err:
                    subprocess.run(command, stdout=out, stderr=err, check=True)
                return name, dict(seconds=time.monotonic() - started, command=command,
                    stages=json.loads((run / name / 'timings.json').read_text()))
            started = time.monotonic()
            with ThreadPoolExecutor(max_workers=workers) as executor:
                families = dict(executor.map(family, ('system', 'audio')))
            row = dict(workers=workers, repeat=repeat,
                       wall_seconds=time.monotonic() - started, families=families)
            rows.append(row)
            print(json.dumps(row), flush=True)
            (output / 'offline-results.json').write_text(json.dumps(rows, indent=2) + '\n')


def offline_family(args):
    import port_family
    stages = {}
    def instrument(name, label):
        original = getattr(port_family, name)
        def measured(*positional, **keywords):
            started = time.monotonic()
            try:
                return original(*positional, **keywords)
            finally:
                key = label(*positional, **keywords) if callable(label) else label
                stages[key] = stages.get(key, 0) + time.monotonic() - started
        setattr(port_family, name, measured)
    instrument('execute', lambda command, run, label: 'execute-' + label)
    instrument('verify_source', 'source-authentication')
    instrument('build', 'build-total')
    instrument('prepare_case', 'fixture-preparation')
    instrument('digest', 'hashing-total-overlapping')
    verdict = port_family.verify(args.manifest.resolve(), args.source.resolve(), args.output.resolve())
    path = next(args.output.glob('*/verdict.json'))
    started = time.monotonic()
    port_family.verify_family_evidence(path)
    stages['evidence-authentication'] = time.monotonic() - started
    args.output.joinpath('timings.json').write_text(json.dumps(stages, indent=2) + '\n')
    print(json.dumps(dict(status=verdict['status'], cases=verdict.get('verified_cases'), stages=stages)))
    assert verdict['status'] == 'offline-verified'


def closure(args):
    from port_family import parse_result, verify_family_evidence
    path = args.baseline.resolve()
    baseline = verify_family_evidence(path)
    run = path.parent
    output = args.output.resolve()
    output.mkdir(parents=True)
    memory = ROOT / 'crates/port/src/lib.rs'
    # Study-only extraction. A production change must establish a shared module.
    (output / 'memory.rs').write_text(re.sub(r'^pub mod \w+;\n', '', memory.read_text(), flags=re.M))
    wrapper = (run / 'wrapper.rs').read_text()
    wrapper = wrapper.replace(json.dumps(str(memory)), json.dumps(str(output / 'memory.rs')))
    (output / 'wrapper.rs').write_text(wrapper)
    timings = {}
    for label, source in (('broad', run / 'wrapper.rs'), ('narrow', output / 'wrapper.rs')):
        command = ['rustc', '--edition=2024', '-O', '--emit=link,dep-info', str(source),
                   '-o', str(output / label)]
        started = time.monotonic()
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL,
                       stderr=(output / (label + '.stderr')).open('wb'))
        timings[label] = time.monotonic() - started
    started = time.monotonic()
    for index, case in enumerate(baseline['cases']):
        directory = run / case['case']
        config = json.loads((directory / 'case.json').read_text())
        result = subprocess.run([str(output / 'narrow'), case['routine'],
            str(config.get('a', 0)), str(config.get('b', 0)), str(directory / 'before.wram'),
            str(output / 'after.wram')], capture_output=True, check=True)
        assert parse_result(result.stdout) == case['return']['c']
        assert digest(output / 'after.wram') == case['after_sha256']['c']
    timings['narrow-case-replay'] = time.monotonic() - started
    (output / 'after.wram').unlink()
    row = dict(compile_seconds=timings, cases=len(baseline['cases']),
               all_full_wram_and_returns_match=True, baseline=str(path),
               dependencies={label: (output / (label + '.d')).read_text().splitlines()[0]
                             for label in ('broad', 'narrow')})
    (output / 'closure-results.json').write_text(json.dumps(row, indent=2) + '\n')
    print(json.dumps({k: v for k, v in row.items() if k != 'dependencies'}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('live', 'offline', 'offline-family', 'closure'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repeats', type=int, default=2)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--rom', type=Path)
    parser.add_argument('--inputs', type=Path)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--manifest', type=Path)
    args = parser.parse_args()
    {'live': live, 'offline': offline, 'offline-family': offline_family,
     'closure': closure}[args.mode](args)


if __name__ == '__main__':
    main()
