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
    # Admission proves every C access is bounded, without predicting output values.
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
    def take(count):
        nonlocal cursor
        if cursor + count > len(stream):
            raise ValueError('truncated fixture operand or data')
        result = stream[cursor:cursor + count]
        cursor += count
        return result
    while cursor < len(stream):
        header = take(1)[0]
        if header == 255:
            break
        if header & 0xe0 == 0xe0:
            mode = (header * 8) & 0xe0
            length = ((header & 3) << 8 | take(1)[0]) + 1
        else:
            mode = header & 0xe0
            length = (header & 31) + 1
        if destination + stores + length > 0x20000:
            raise ValueError('fixture destination writes outside WRAM')
        if mode & 0x80:
            if mode >= 0xc0:
                reference = destination + stores - take(1)[0]
            else:
                reference = destination + int.from_bytes(take(2), 'little')
            if reference < 0 or reference + length > 0x20000:
                raise ValueError('fixture backreference reads outside WRAM')
        else:
            take(length if mode == 0 else 2 if mode == 0x40 else 1)
        stores += length
    else:
        raise ValueError('fixture lacks bounded terminator')
    if cursor != len(stream):
        raise ValueError('fixture stream has trailing bytes')
    if len(fixture['expected_bytes']) != stores or fixture['expected_written'] != stores:
        raise ValueError('fixture expected extent differs from declared write extent')
    directory.mkdir()
    # Fixture bytes and expected bytes are separately specified, never derived by decompression.
    source = fixture['source']
    if not 0 <= source <= 0xffffff or source & 0x8000 == 0:
        raise ValueError('invalid fixture source')
    placements = []
    address = source
    for byte in stream:
        if address > 0xffffff:
            raise ValueError('fixture source exceeds 24-bit ROM')
        offset = (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff
        placements.append((offset, byte))
        old = address
        address += 1
        if old & 0xffff == 0xffff:
            address += 0x8000
    if len({offset for offset, _ in placements}) != len(placements):
        raise ValueError('fixture source aliases earlier stream bytes')
    rom = bytearray(max(offset for offset, _ in placements) + 1)
    for offset, byte in placements:
        rom[offset] = byte
    initial = bytes([fixture['initial_fill']]) * 0x20000
    expected = bytearray(initial)
    dst = fixture['destination']
    expected[dst:dst + len(fixture['expected_bytes'])] = bytes(fixture['expected_bytes'])
    (directory / 'rom.bin').write_bytes(rom)
    (directory / 'before.wram').write_bytes(initial)
    (directory / 'expected.wram').write_bytes(expected)
    (directory / 'fixture.json').write_text(json.dumps(fixture, indent=2) + '\n')



def prepare_captures(path, manifest, run):
    captures = json.loads(path.read_text())
    if captures.get('source_revision') != manifest['source_revision']:
        raise ValueError('capture source identity differs from routine manifest')
    if (captures.get('contract'), captures.get('routine'), captures.get('routine_address')) != ('decomp-c-captures-v1', 'DecompressToMem', 0x80b119):
        raise ValueError('capture routine contract differs')
    if captures.get('rom_view_size') != 0x400000 or not captures.get('calls'):
        raise ValueError('captures require full ROM views and at least one call')
    initial_view = path.parent / captures['rom_view_file']
    if initial_view.stat().st_size != 0x400000 or digest(initial_view) != captures['rom_view_sha256']:
        raise ValueError('capture initial ROM view identity differs')
    shutil.copyfile(path, run / 'captures.json')
    fixtures = []
    ids = set()
    for sequence, call in enumerate(captures['calls'], 1):
        if (call['sequence'] != sequence or call['fixture_id'] in ids
                or not call['fixture_id'].replace('-', '').isalnum()):
            raise ValueError('capture sequence/identity differs')
        ids.add(call['fixture_id'])
        def verified(name, identity, size):
            artifact = path.parent / name
            if artifact.stat().st_size != size or digest(artifact) != identity:
                raise ValueError(f'capture artifact identity differs: {name}')
            return artifact.read_bytes()
        if call['before']['size'] != 0x20000 or call['after']['size'] != 0x20000:
            raise ValueError('capture WRAM declared extent differs')
        rom = verified(call['rom_view_file'], call['rom_view_sha256'], 0x400000)
        before = verified(call['before']['file'], call['before']['sha256'], 0x20000)
        after = verified(call['after']['file'], call['after']['sha256'], 0x20000)
        source, destination = call['source_address'], call['destination_offset']
        if not 0 <= destination < 0x20000:
            raise ValueError('capture destination outside WRAM')
        address = source
        stream, offsets = bytearray(), []
        def take(count):
            nonlocal address
            values = bytearray()
            for _ in range(count):
                if not 0 <= address <= 0xffffff or not address & 0x8000 or len(stream) >= 0x400000:
                    raise ValueError('capture source outside bounded LoROM')
                offset = (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff
                values.append(rom[offset]); stream.append(rom[offset]); offsets.append(offset)
                old = address; address += 1
                if old & 0xffff == 0xffff: address += 0x8000
            return values
        stores = 0
        reads = []
        while True:
            header = take(1)[0]
            if header == 255: break
            if header & 0xe0 == 0xe0:
                mode = (header * 8) & 0xe0
                length = ((header & 3) << 8 | take(1)[0]) + 1
            else:
                mode = header & 0xe0; length = (header & 31) + 1
            if destination + stores + length > 0x20000:
                raise ValueError('capture destination writes outside WRAM')
            if mode & 0x80:
                reference = (destination + stores - take(1)[0] if mode >= 0xc0
                             else destination + int.from_bytes(take(2), 'little'))
                if reference < 0 or reference + length > 0x20000:
                    raise ValueError('capture backreference reads outside WRAM')
                reads.append([reference, reference + length])
            else:
                take(length if mode == 0 else 2 if mode == 0x40 else 1)
            stores += length
        directory = run / call['fixture_id']; directory.mkdir()
        fixture = {'id': call['fixture_id'], 'captured': True, 'source': source,
                   'destination': destination, 'expected_written': stores,
                   'expected_final_source': address, 'source_offsets': offsets,
                   'stream_sha256': hashlib.sha256(stream).hexdigest(), 'read_extents': reads, 'capture': call}
        (directory / 'rom.bin').write_bytes(rom)
        (directory / 'before.wram').write_bytes(before)
        (directory / 'expected.wram').write_bytes(after)
        (directory / 'fixture.json').write_text(json.dumps(fixture, indent=2) + '\n')
        fixtures.append(fixture)
    return fixtures, captures


def compare_original(args, captures, fixtures, run, source):
    original = args.original_rom.resolve()
    if digest(original) != captures['original_rom_sha256'] or original.stat().st_size != 0x300000:
        raise ValueError('original ROM identity differs from capture manifest')
    original_bytes = original.read_bytes()
    runner = ROOT / 'tools/decomp/original_oracle.c'
    shutil.copyfile(runner, run / runner.name)
    # Verify the CPU itself against the same frozen archive, independently of game code.
    with tarfile.open(ROOT / 'deps/downloads' / next(p['archive'] for p in json.loads((ROOT / 'tools/dependencies.json').read_text()) if p['name'] == 'snesrev-sm')) as archive:
        cpu_member = next(m for m in archive.getmembers() if m.name.endswith('/src/snes/cpu.c'))
        cpu = source / 'src/snes/cpu.c'
        if cpu.read_bytes() != archive.extractfile(cpu_member).read():
            raise ValueError('original instruction runner CPU source differs from archive')
    command(['cc', '-std=c11', '-O2', '-ffunction-sections', '-fdata-sections', '-iquote',
             str(source / 'src'), str(cpu), str(run / runner.name), '-Wl,--gc-sections',
             '-o', str(run / 'original-oracle')], run, args.timeout)
    evidence = {'status': 'bounded-routine-matched', 'rom_sha256': digest(original),
                'cpu_sha256': digest(cpu), 'shim_sha256': digest(runner),
                'binary_sha256': digest(run / 'original-oracle'),
                'oracle': 'unchanged original ROM executed by pinned upstream 65816 instruction CPU',
                'limit': 'three captured calls; output below 64 KiB; no timing or whole-game claim',
                'fixtures': [], 'ineligible': [], 'source_stream_checks': []}
    (run / 'original-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    for fixture in fixtures:
        directory = run / fixture['id']
        offsets = fixture['source_offsets']
        captured_rom = (directory / 'rom.bin').read_bytes()
        stream_matches = all(offset < len(original_bytes) and original_bytes[offset] == captured_rom[offset] for offset in offsets)
        evidence['source_stream_checks'].append({'id': fixture['id'], 'stream_sha256': fixture['stream_sha256'], 'bytes': len(offsets), 'matches_unmodified_rom': stream_matches})
        if (fixture['expected_written'] >= 0x10000 or fixture['destination'] < 0x2000 or
            any(start < 0x2000 for start, end in fixture['read_extents']) or
            not stream_matches):
            evidence['ineligible'].append(fixture['id']); continue
        if len(evidence['fixtures']) == 3: continue
        record = {'id': fixture['id'], 'source': fixture['source'], 'destination': fixture['destination'],
                  'stream_sha256': fixture['stream_sha256'], 'stream_matches_unmodified_rom': True,
                  'status': 'error'}
        evidence['fixtures'].append(record)
        try:
            diagnostics = []
            for name in ['original', 'original_repeat']:
                stdout = command([str(run / 'original-oracle'), str(original), str(directory / 'before.wram'),
                                  str(fixture['source']), str(fixture['destination']), str(directory / f'{name}.wram')], run, args.timeout)
                (directory / f'{name}.json').write_text(stdout)
                diagnostics.append(json.loads(stdout))
            actual = (directory / 'original.wram').read_bytes()
            if actual != (directory / 'original_repeat.wram').read_bytes() or diagnostics[0] != diagnostics[1]:
                raise ValueError(f"original repeat differs: {fixture['id']}")
            diagnostic = diagnostics[0]
            # Input staging, decoded routine scratch and exact observed stack-write interval.
            excluded = set(range(0x47, 0x51)) | set(range(0x1fd, 0x200))
            excluded.update(range(diagnostic['stack_written_min'], diagnostic['stack_written_max'] + 1))
            expected = (directory / 'expected.wram').read_bytes()
            mismatch = [i for i in range(0x20000) if actual[i] != expected[i]]
            record.update(diagnostic)
            record['excluded_offsets'] = sorted(excluded)
            record['raw_mismatch_offsets'] = mismatch
            record['unexcluded_mismatch_offsets'] = [i for i in mismatch if i not in excluded]
            record['sha256'] = digest(directory / 'original.wram')
            if record['unexcluded_mismatch_offsets']:
                raise ValueError(f"original ROM routine mismatch: {fixture['id']}")
            record['status'] = 'matched'
        finally:
            (run / 'original-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    (run / 'original-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    if len(evidence['fixtures']) != 3:
        raise ValueError('fewer than three eligible original ROM calls')
    return evidence

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
        inputs = [ROOT / 'tools/decomp/routine_oracle.c', Path(__file__).resolve(),
                  ROOT / 'crates/decomp/src/lib.rs', ROOT / 'crates/decomp/src/main.rs']
        verdict['implementation_identities'] = {str(p.relative_to(ROOT)): digest(p) for p in inputs}
        for p in inputs:
            shutil.copyfile(p, run / p.name)
        for fixture in manifest['fixtures']:
            prepare_fixture(fixture, run / fixture['id'])
        captures = None
        fixtures = list(manifest['fixtures'])
        if args.captures:
            captured, captures = prepare_captures(args.captures.resolve(), manifest, run)
            fixtures += captured
            verdict['captures_manifest_sha256'] = digest(args.captures)
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
        for fixture in fixtures:
            directory = run / fixture['id']
            record = {'id': fixture['id'], 'status': 'error',
                      'fixture_sha256': digest(directory / 'fixture.json'),
                      'rom_sha256': digest(directory / 'rom.bin'),
                      'initial_sha256': digest(directory / 'before.wram'),
                      'expected_sha256': digest(directory / 'expected.wram')}
            verdict['fixtures'].append(record)
            for subject, executable in [('c', 'c-oracle'), ('rust', 'rust-candidate'),
                                        ('c_repeat', 'c-oracle'), ('rust_repeat', 'rust-candidate')]:
                stdout = command([str(run / executable), str(directory / 'rom.bin'),
                                  str(directory / 'before.wram'), str(fixture['source']),
                                  str(fixture['destination']), str(directory / f'{subject}.wram')], run, args.timeout)
                (directory / f'{subject}.json').write_text(stdout)
                record[subject + '_sha256'] = digest(directory / f'{subject}.wram')
            candidate = json.loads((directory / 'rust.json').read_text())
            if candidate['status'] != 'ok' or candidate['written'] != fixture['expected_written'] or candidate['final_source'] != fixture['expected_final_source'] or (not fixture.get('captured') and candidate['cases'] != fixture['expected_cases']):
                raise ValueError(f"candidate diagnostics differ: {fixture['id']}")
            record['observed_cases'] = candidate['cases']
            observed.update(candidate['cases'])
            expected = (directory / 'expected.wram').read_bytes()
            for subject in ['c', 'rust', 'c_repeat', 'rust_repeat']:
                actual = (directory / f'{subject}.wram').read_bytes()
                if actual != expected:
                    record['actual_size'] = len(actual)
                    record['mismatch_offsets'] = [i for i in range(max(len(actual), len(expected))) if actual[i:i+1] != expected[i:i+1]][:32]
                    if record['mismatch_offsets']:
                        offset = record['mismatch_offsets'][0]
                        record['first_mismatch'] = {'routine': 'DecompressToMem', 'fixture': fixture['id'], 'subject': subject, 'offset': offset, 'actual': list(actual[offset:offset+1]), 'expected': list(expected[offset:offset+1])}
                    raise ValueError(f"whole WRAM mismatch: {fixture['id']}/{subject}")
            if (directory / 'rust.json').read_bytes() != (directory / 'rust_repeat.json').read_bytes():
                raise ValueError(f"repeat diagnostics differ: {fixture['id']}")
            record['status'] = 'matched'
        verdict['observed_cases'] = sorted(observed)
        verdict['missing_cases'] = sorted(set(manifest['required_cases']) - observed)
        if verdict['missing_cases']:
            raise ValueError('required synthetic cases were not exercised')
        verdict['synthetic_verified'] = True
        if captures:
            verdict['captured_calls_verified'] = len(captures['calls'])
            verdict['captured_missing_cases'] = sorted(set(manifest['required_cases']) - set().union(*(set(r['observed_cases']) for r in verdict['fixtures'] if r['id'] in {f['id'] for f in captured})))
            if args.original_rom:
                verdict['original_rom_parity'] = compare_original(args, captures, captured, run, source)
        elif args.original_rom:
            raise ValueError('original ROM checks require captured calls')
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
    parser.add_argument('--captures', type=Path, help='local decomp-c-captures-v1 manifest')
    parser.add_argument('--original-rom', type=Path, help='unmodified ROM for bounded independent checks')
    parser.add_argument('--timeout', type=int, default=60)
    args = parser.parse_args()
    if args.timeout < 1: parser.error('timeout must be positive')
    return run_job(args)

if __name__ == '__main__':
    raise SystemExit(main())
