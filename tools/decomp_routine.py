#!/usr/bin/env python3
"""Fresh prepare/build/compare/verdict job for the bounded RAM decompression pilot."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def command(argv, run, timeout):
    with (run / 'commands.jsonl').open('a') as log:
        log.write(json.dumps(argv) + '\n')
    result = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
    with (run / 'build-execution.log').open('a') as log:
        log.write(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f'command failed ({result.returncode}): {argv[0]}: {result.stdout + result.stderr}')
    return result.stdout


def verify_source(manifest, source):
    pin = next(p for p in json.loads((ROOT / 'tools/dependencies.json').read_text()) if p['name'] == 'snesrev-sm')
    archive_path = ROOT / 'deps/downloads' / pin['archive']
    if pin['revision'] != manifest['source_revision'] or pin['sha256'] != manifest['source_archive_sha256'] or digest(archive_path) != pin['sha256']:
        raise ValueError('source archive identity differs from routine manifest')
    identities = {}
    with tarfile.open(archive_path) as archive:
        for member in archive.getmembers():
            if member.isfile():
                relative = Path(*Path(member.name).parts[1:])
                if relative == Path('LICENSE.txt') or relative == Path('src/sm_80.c') or relative.suffix == '.h':
                    path = source / relative
                    if path.read_bytes() != archive.extractfile(member).read():
                        raise ValueError(f'source differs from pinned archive: {relative}')
                    identities[str(relative)] = digest(path)
    bank = (source / 'src/sm_80.c').read_bytes()
    start = bank.index(b'static uint32 decompress_src;')
    end = bank.index(b'static uint8 ReadPpuByte', start)
    identities['routine-and-helper'] = hashlib.sha256(bank[start:end]).hexdigest()
    return identities


def prepare_fixture(fixture, directory):
    # Validate the initial literal-only contract before letting unsafe C see it.
    if not fixture['id'].replace('-', '').isalnum():
        raise ValueError('invalid fixture id')
    destination = fixture['destination']
    if not 0 <= destination < 0x20000:
        raise ValueError('fixture destination outside WRAM')
    if not 0 <= fixture['initial_fill'] <= 255:
        raise ValueError('invalid initial fill')
    stream = bytes(fixture['stream'])
    cursor = 0
    stores = 0
    while cursor < len(stream):
        header = stream[cursor]
        cursor += 1
        if header == 255:
            break
        if header & 0xe0 == 0xe0:
            if cursor == len(stream):
                raise ValueError('truncated extended header')
            command = (header * 8) & 0xe0
            length = ((header & 3) << 8 | stream[cursor]) + 1
            cursor += 1
        else:
            command = header & 0xe0
            length = (header & 31) + 1
        if command != 0:
            raise ValueError('fixture command outside initial literal-only C contract')
        stores += length
        cursor += length
        if stores + destination > 0x20000:
            raise ValueError('fixture destination writes outside WRAM')
    else:
        raise ValueError('fixture lacks bounded terminator')
    if cursor != len(stream):
        raise ValueError('fixture stream truncated or trailing bytes')
    if len(fixture['expected_bytes']) != stores or fixture['expected_written'] != stores:
        raise ValueError('fixture expected extent differs from declared write extent')
    directory.mkdir()
    # Fixture bytes and expected bytes are separately specified, never derived by decompression.
    rom = bytearray(0x8000)
    source = fixture['source']
    if not 0 <= source <= 0xffffff or source & 0x8000 == 0:
        raise ValueError('invalid fixture source')
    offset = (((source >> 16) << 15) | (source & 0x7fff)) & 0x3fffff
    if offset + len(stream) > len(rom) or (source & 0xffff) + len(stream) > 0x10000:
        raise ValueError('initial fixture contract excludes bank-crossing streams')
    rom[offset:offset + len(fixture['stream'])] = bytes(fixture['stream'])
    initial = bytes([fixture['initial_fill']]) * 0x20000
    expected = bytearray(initial)
    dst = fixture['destination']
    expected[dst:dst + len(fixture['expected_bytes'])] = bytes(fixture['expected_bytes'])
    (directory / 'rom.bin').write_bytes(rom)
    (directory / 'before.wram').write_bytes(initial)
    (directory / 'expected.wram').write_bytes(expected)
    (directory / 'fixture.json').write_text(json.dumps(fixture, indent=2) + '\n')


def run_job(args):
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    verdict = {'contract': 'decomp-offline-v1', 'status': 'error', 'routine_accepted': False,
               'stages': [], 'fixtures': [], 'observed_cases': [], 'run': str(run),
               'original_rom_parity': 'not-tested', 'live_rust_ownership': 'not-tested'}
    try:
        manifest = json.loads(args.manifest.read_text())
        shutil.copyfile(args.manifest, run / 'job.json')
        verdict['manifest_sha256'] = digest(args.manifest)
        verdict['source_revision'] = manifest['source_revision']
        source = ROOT / 'deps/snesrev-sm'
        verdict['source_identities'] = verify_source(manifest, source)
        shutil.copyfile(source / 'LICENSE.txt', run / 'LICENSE.txt')
        inputs = [ROOT / 'tools/decomp/routine_oracle.c', Path(__file__),
                  ROOT / 'crates/decomp/src/lib.rs', ROOT / 'crates/decomp/src/main.rs']
        verdict['implementation_identities'] = {str(p.relative_to(ROOT)): digest(p) for p in inputs}
        for p in inputs:
            shutil.copyfile(p, run / p.name)
        for fixture in manifest['fixtures']:
            prepare_fixture(fixture, run / fixture['id'])
        verdict['stages'].append('prepare')
        verdict['compiler'] = command(['cc', '--version'], run, args.timeout).splitlines()[0]
        verdict['rustc'] = command(['rustc', '--version'], run, args.timeout).strip()
        command(['cc', '-std=c11', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections',
                 '-iquote', str(source / 'src'), str(source / 'src/sm_80.c'), str(run / 'routine_oracle.c'),
                 '-Wl,--gc-sections', '-o', str(run / 'c-oracle')], run, args.timeout)
        command(['rustc', '--edition=2024', '--crate-type=rlib', '--crate-name=decomp', '-O',
                 str(run / 'lib.rs'), '-o', str(run / 'libdecomp.rlib')], run, args.timeout)
        command(['rustc', '--edition=2024', '-O', str(run / 'main.rs'), '--extern',
                 f'decomp={run / "libdecomp.rlib"}', '-o', str(run / 'rust-candidate')], run, args.timeout)
        verdict['binaries'] = {name: digest(run / name) for name in ['c-oracle', 'rust-candidate']}
        verdict['stages'].append('build')
        observed = set()
        for fixture in manifest['fixtures']:
            directory = run / fixture['id']
            record = {'id': fixture['id'], 'status': 'error',
                      'fixture_sha256': digest(directory / 'fixture.json'),
                      'rom_sha256': digest(directory / 'rom.bin'),
                      'initial_sha256': digest(directory / 'before.wram'),
                      'expected_sha256': digest(directory / 'expected.wram')}
            verdict['fixtures'].append(record)
            for subject, executable in [('c', 'c-oracle'), ('rust', 'rust-candidate')]:
                stdout = command([str(run / executable), str(directory / 'rom.bin'),
                                  str(directory / 'before.wram'), str(fixture['source']),
                                  str(fixture['destination']), str(directory / f'{subject}.wram')], run, args.timeout)
                (directory / f'{subject}.json').write_text(stdout)
                record[subject + '_sha256'] = digest(directory / f'{subject}.wram')
            candidate = json.loads((directory / 'rust.json').read_text())
            if candidate['status'] != 'ok' or candidate['written'] != fixture['expected_written'] or candidate['final_source'] != fixture['expected_final_source'] or candidate['cases'] != fixture['expected_cases']:
                raise ValueError(f"candidate diagnostics differ: {fixture['id']}")
            observed.update(candidate['cases'])
            expected = (directory / 'expected.wram').read_bytes()
            for subject in ['c', 'rust']:
                actual = (directory / f'{subject}.wram').read_bytes()
                if actual != expected:
                    record['mismatch_offsets'] = [i for i, (a, b) in enumerate(zip(actual, expected)) if a != b][:32]
                    raise ValueError(f"whole WRAM mismatch: {fixture['id']}/{subject}")
            record['status'] = 'matched'
        verdict['observed_cases'] = sorted(observed)
        verdict['missing_cases'] = sorted(set(manifest['required_cases']) - observed)
        verdict['stages'].append('compare')
        # Later tickets establish full coverage and negative controls before any acceptance.
        verdict['status'] = 'partial'
        verdict['stages'].append('verdict')
    except Exception as error:
        verdict['error'] = f'{type(error).__name__}: {error}'
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    print(json.dumps(verdict))
    return 0 if verdict['status'] == 'partial' else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=ROOT / 'tools/decomp/decompress-job.json')
    parser.add_argument('--output', type=Path, default=ROOT / 'out/decomp/routine')
    parser.add_argument('--timeout', type=int, default=60)
    args = parser.parse_args()
    if args.timeout < 1: parser.error('timeout must be positive')
    return run_job(args)

if __name__ == '__main__':
    raise SystemExit(main())
