#!/usr/bin/env python3
"""Compare independently replayed C and accepted mixed C/Rust subjects."""
import argparse
import datetime
import json
from pathlib import Path
import subprocess
import sys
import shutil
import tarfile
from decomp_routine import verify_accepted, digest
ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--next-random-library', type=Path)
    parser.add_argument('--accepted', type=Path, default=ROOT / 'out/decomp/routine/decomp-acceptance.json')
    parser.add_argument('--frames', type=int, default=9000)
    parser.add_argument('--rom', type=Path, default=ROOT / 'rom/sm.sfc')
    parser.add_argument('--inputs', type=Path, default=ROOT / 'movies/super-metroid-any-3653.inputs')
    parser.add_argument('--timeout', type=int, default=180)
    parser.add_argument('--output', type=Path, default=ROOT / 'out/decomp/live')
    args = parser.parse_args()
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    verdict = {'status': 'error', 'stages': [], 'mixed_integration': False}
    try:
        seal = verify_accepted(args.accepted.resolve())
        verdict['offline_acceptance_sha256'] = digest(args.accepted.resolve())
        verdict['stages'].append('offline-verified')
        if args.frames != 9000 or args.timeout < 1:
            raise ValueError('live acceptance requires frozen 9000-frame prefix')
        from decomp_baseline import ROM_SHA256, REVISION
        if digest(args.rom) != ROM_SHA256:
            raise ValueError('original ROM identity differs')
        canonical_inputs = ROOT / 'movies/super-metroid-any-3653.inputs'
        if digest(args.inputs) != digest(canonical_inputs):
            raise ValueError('movie input identity differs')
        source = ROOT / 'deps/snesrev-sm'
        pin = next(p for p in json.loads((ROOT / 'tools/dependencies.json').read_text()) if p['name'] == 'snesrev-sm')
        archive_path = ROOT / 'deps/downloads' / pin['archive']
        if pin['revision'] != REVISION or digest(archive_path) != pin['sha256']:
            raise ValueError('upstream archive changed')
        with tarfile.open(archive_path) as archive:
            for member in archive.getmembers():
                if member.isfile():
                    relative = Path(*Path(member.name).parts[1:])
                    if (source / relative).read_bytes() != archive.extractfile(member).read():
                        raise ValueError(f'upstream source differs: {relative}')
        verdict.update(contract='decomp-live-v1', original_rom_parity='not-tested',
            input_adaptation='bit-reversal-only; no lag compensation',
            source_revision=REVISION, source_archive_sha256=pin['sha256'],
            inputs_sha256=digest(args.inputs), rom_sha256=digest(args.rom), commands=[])
        def execute(command, cwd, name, success=True):
            verdict['commands'].append({'argv': command, 'cwd': str(cwd)})
            with ((cwd if cwd != ROOT else run) / (name + '.stdout.log')).open('w') as stdout, ((cwd if cwd != ROOT else run) / (name + '.stderr.log')).open('w') as stderr:
                result = subprocess.run(command, cwd=cwd, stdout=stdout, stderr=stderr, timeout=args.timeout)
            if success and result.returncode:
                raise ValueError(f'{name} failed with exit {result.returncode}')
            return result
        execute(['cargo', 'build', '-p', 'decomp', '--lib', '--release', '--locked', '--offline',
                 '--target-dir', str(run / 'target')], ROOT, 'live-rust-build')
        # Verify the same identities again before linking the accepted implementation.
        verify_accepted(args.accepted.resolve())
        library = run / 'target/release/libdecomp.a'
        units = sorted((source / 'src').glob('sm_*.c'))
        if args.next_random_library:
            bank = source / 'src/sm_80.c'
            text = bank.read_text()
            signature = 'uint16 NextRandom(void) {'
            if text.count(signature) != 1: raise ValueError('NextRandom definition identity/count differs')
            transformed = run / 'sm_80_rng.c'
            transformed.write_text(text.replace(signature, 'uint16 C_NextRandom(void) {'))
            units[units.index(bank)] = transformed
            verdict['rng_bank_transformation'] = {'original_sha256': digest(bank), 'transformed_sha256': digest(transformed), 'definition_renames': 1}
        units += [source / 'src' / n for n in ('spc_player.c', 'tracing.c', 'util.c')]
        units += sorted((source / 'src/snes').glob('*.c'))
        host = ROOT / 'tools/decomp/live.c'
        copied = [host, ROOT / 'tools/decomp/headless.c', Path(__file__).resolve(), source / 'LICENSE.txt']
        for path in copied: shutil.copyfile(path, run / path.name)
        verdict['sources'] = {str(p): digest(p) for p in [*units, *copied]}
        verdict['rust_library_sha256'] = digest(library)
        verdict['compiler'] = subprocess.check_output(['cc', '--version'], text=True).splitlines()[0]
        verdict['rust_compiler'] = subprocess.check_output(['rustc', '--version'], text=True).strip()
        build = ['cc', '-std=gnu11', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections',
            '-iquote', str(source / 'src'), *map(str, units), str(host), str(library),
            '-Wl,--gc-sections', '-Wl,--wrap=Call', '-Wl,--wrap=cpu_runOpcode',
            '-Wl,--wrap=DecompressToMem', '-Wl,--wrap=ppu_write', '-Wl,--wrap=RtlApuWrite',
            '-lm', '-ldl', '-lpthread', '-lz', '-o', str(run / 'live')]
        if args.next_random_library:
            build[1:1] = ['-DNEXT_RANDOM_JOB', '-iquote', str(ROOT / 'tools/decomp')]
            build.insert(build.index('-lm'), str(args.next_random_library.resolve()))
            verdict['rng_library_sha256'] = digest(args.next_random_library)
            verdict['rng_host_sha256'] = digest(ROOT / 'tools/decomp/next_random_live.c')
        execute(build, run, 'host-build')
        verdict['binary_sha256'] = digest(run / 'live')
        verdict['stages'].append('built')
        execute([str(run / 'live'), '--abi-control'], run, 'abi-control')
        verdict['abi_controls'] = json.loads((run / 'abi-control.stdout.log').read_text())
        inventories = {}
        verdict['subjects'] = {}
        for mode in ('c', 'mixed'):
            for repeat in range(2):
                name = f'{mode}-{repeat}'
                subject = run / name
                subject.mkdir()
                command = [str(run / 'live'), mode, str(args.rom.resolve()), str(args.inputs.resolve()), str(args.frames), 'unused']
                execute(command, subject, 'replay')
                all_rows = [json.loads(line) for line in (subject / 'replay.stdout.log').read_text().splitlines() if line.startswith('{')]
                rows = [r for r in all_rows if 'frames' in r]
                if args.next_random_library:
                    rng_rows = [r for r in all_rows if 'rng_c_hits' in r]
                    if len(rng_rows) != 2 or any(r['epoch'] != i for i, r in enumerate(rng_rows)):
                        raise ValueError('missing RNG epochs')
                    for row in rng_rows:
                        if row['rng_rust_hits' if mode == 'mixed' else 'rng_c_hits'] <= 0 or row['rng_c_hits' if mode == 'mixed' else 'rng_rust_hits'] != 0:
                            raise ValueError('NextRandom replacement not exclusively exercised')
                    verdict.setdefault('rng_subjects', {})[name] = rng_rows
                if len(rows) != 2 or any(r['frames'] != 9000 or r['epoch'] != i for i, r in enumerate(rows)):
                    raise ValueError('incomplete same-process replay')
                for row in rows:
                    if mode == 'mixed' and (row['rust_hits'] != 23 or row['c_hits'] != 0 or row['modes'] == 0):
                        raise ValueError('accepted Rust routine was not exercised exclusively')
                    if mode == 'c' and (row['c_hits'] != 23 or row['rust_hits'] != 0):
                        raise ValueError('C routine oracle not exercised')
                verdict['subjects'][name] = rows
                epoch_sets = []
                for epoch in range(2):
                    directory = subject / f'epoch-{epoch}'
                    inventory = {str(p.relative_to(directory)): digest(p) for p in sorted(directory.rglob('*')) if p.is_file()}
                    if len(list(directory.glob('frame-*.bin'))) != 10 or len(list((directory / 'fixtures').glob('decompress-*.wram'))) != 46:
                        raise ValueError('missing checkpoints or call states')
                    epoch_sets.append(inventory)
                if epoch_sets[0] != epoch_sets[1]:
                    mismatches = [k for k in epoch_sets[0] if epoch_sets[0][k] != epoch_sets[1].get(k)]
                    raise ValueError(f'same-process reset diverged: {name}: {mismatches[:3]}')
                inventories[name] = epoch_sets[0]
        reference = inventories['c-0']
        for name, inventory in inventories.items():
            if reference != inventory:
                mismatches = [k for k in reference if reference[k] != inventory.get(k)]
                raise ValueError(f'independent replay diverged: {name}: {mismatches[:3]}')
        # Invoke an invalid source at the first actual gameplay ABI call.
        control = run / 'error-control'; control.mkdir()
        result = execute([str(run / 'live'), 'error', str(args.rom.resolve()), str(args.inputs.resolve()), '9000', 'unused'], control, 'replay', False)
        errors = (control / 'replay.stderr.log').read_text()
        if result.returncode != 4 or 'ABI error status=2 rust_hits=0 c_hits=0 calls=1' not in errors:
            raise ValueError('ABI error control did not stop at the exercised replacement')
        if list(control.rglob('frame-*.bin')) or list(control.rglob('*after.wram')):
            raise ValueError('ABI error control emitted a successful state')
        verdict.update(status='mixed-C-Rust-integration', mixed_integration=True,
            comparison='4 independent processes, each reset and replayed twice; full checkpoint states, call states and ordered PPU/APU effects match',
            checkpoint_identities=reference, observed_modes=[name for i, name in enumerate(['literal', 'byte-fill', 'word-fill', 'increment-fill', 'absolute-copy', 'absolute-xor', 'relative-copy', 'relative-xor']) if verdict['subjects']['mixed-0'][0]['modes'] & (1 << i)],
            unexercised_modes=[name for i, name in enumerate(['literal', 'byte-fill', 'word-fill', 'increment-fill', 'absolute-copy', 'absolute-xor', 'relative-copy', 'relative-xor']) if not verdict['subjects']['mixed-0'][0]['modes'] & (1 << i)], error_control={'exit': result.returncode, 'rust_hits': 0, 'c_hits': 0, 'calls': 1},
            remaining_gameplay_cpu_fallback=verdict['subjects']['mixed-0'][0]['fallback_calls'])
        verdict['stages'].append('compared')
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        verdict['error'] = str(error)
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    print(json.dumps({'status': verdict['status'], 'evidence': str(run), 'error': verdict.get('error')}))
    return 0 if verdict['mixed_integration'] else 1
if __name__ == '__main__':
    sys.exit(main())
