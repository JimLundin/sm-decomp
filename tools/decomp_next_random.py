"""One disjoint, serialized translation job; predecessor seals remain immutable."""
import datetime
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
from decomp_routine import digest, verify_accepted
ROOT = Path(__file__).resolve().parents[1]

def run_job(args):
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir()
    verdict = {'status': 'error', 'routine_accepted': False, 'run': str(run), 'commands': [], 'original_rom_parity': 'not-tested'}
    def execute(command, name):
        verdict['commands'].append(command)
        with (run / (name + '.stdout')).open('wb') as out, (run / (name + '.stderr')).open('wb') as err:
            result = subprocess.run(command, cwd=ROOT, stdout=out, stderr=err, timeout=180)
        if result.returncode: raise ValueError(f'{name} failed: {result.returncode}')
        return (run / (name + '.stdout')).read_bytes()
    try:
        verify_accepted(args.accepted)
        predecessor = digest(args.accepted)
        verdict['predecessor'] = {'pointer': str(args.accepted.resolve()), 'sha256': predecessor}
        pin = next(x for x in json.loads((ROOT / 'tools/dependencies.json').read_text()) if x['name'] == 'snesrev-sm')
        source = ROOT / 'deps/snesrev-sm'
        archive_path = ROOT / 'deps/downloads' / pin['archive']
        if digest(archive_path) != pin['sha256']: raise ValueError('archive identity changed')
        with tarfile.open(archive_path) as archive:
            for member in archive.getmembers():
                if member.isfile():
                    relative = Path(*Path(member.name).parts[1:])
                    if (source / relative).read_bytes() != archive.extractfile(member).read(): raise ValueError(f'pinned source changed: {relative}')
        manifest = json.loads((ROOT / 'tools/decomp/next-random-job.json').read_text())
        if manifest['source_revision'] != pin['revision']: raise ValueError('job source identity differs')
        shutil.copyfile(source / 'LICENSE.txt', run / 'LICENSE.txt')
        shutil.copyfile(ROOT / 'tools/decomp/next-random-job.json', run / 'job.json')
        bank = source / 'src/sm_80.c'
        rust = ROOT / 'tools/decomp/next_random.rs'
        oracle = ROOT / 'tools/decomp/next_random_oracle.c'
        inputs = [rust, oracle, ROOT / 'tools/decomp/next_random_live.c', ROOT / 'tools/decomp_next_random.py', ROOT / 'tools/decomp_jobs.py', ROOT / 'tools/decomp/live.c', ROOT / 'tools/decomp_live.py']
        verdict['implementation'] = {str(p.relative_to(ROOT)): digest(p) for p in inputs}
        for path in inputs: shutil.copyfile(path, run / path.name)
        verdict['source'] = {'revision': pin['revision'], 'archive_sha256': pin['sha256'], 'bank_sha256': digest(bank), 'freshness': 'exact pinned archive verified; no broader latest upstream claim'}
        verdict['compiler'] = subprocess.check_output(['cc', '--version'], text=True).splitlines()[0]
        verdict['rust_compiler'] = subprocess.check_output(['rustc', '--version'], text=True).strip()
        execute(['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source / 'src'), str(bank), str(oracle), '-Wl,--gc-sections', '-o', str(run / 'c')], 'c-build')
        execute(['rustc', '--edition=2024', '-O', '--cfg', 'cli', str(rust), '-o', str(run / 'rust')], 'rust-build')
        execute(['rustc', '--edition=2024', '-O', '--crate-type', 'staticlib', '-C', 'panic=abort', str(rust), '-o', str(run / 'libnext_random.a')], 'rust-library')
        c_results = execute([str(run / 'c'), 'exhaustive'], 'c-exhaustive')
        rust_results = execute([str(run / 'rust'), 'exhaustive'], 'rust-exhaustive')
        if len(c_results) != 131072 or c_results != rust_results:
            mismatch = next((i // 2 for i, (a, b) in enumerate(zip(c_results, rust_results)) if a != b), None)
            raise ValueError(f'first exhaustive mismatch seed={mismatch}')
        for case in manifest['independent_examples']:
            observed = int.from_bytes(c_results[case['seed'] * 2:case['seed'] * 2 + 2], 'little')
            if observed != case['expected']: raise ValueError(f'independent example mismatch seed={case["seed"]}')
        verdict['synthetic'] = {'all_65536_seeds': True, 'full_WRAM_unchanged_outside_two_bytes': True, 'independent_examples': manifest['independent_examples'], 'table_sha256': hashlib.sha256(c_results).hexdigest()}
        # Same public candidate ABI rejects bad extents and null pointers without mutation.
        control = run / 'abi-control.c'
        control.write_text('#include <stdint.h>\n#include <stdlib.h>\nextern int32_t sm_next_random_v1(uint8_t*,size_t);\nint main(void){uint8_t *p=calloc(1,131072); if(!p)return 2; if(sm_next_random_v1(0,131072)!=-1||sm_next_random_v1(p,1)!=-1||p[1509]!=0||sm_next_random_v1(p,131072)!=273)return 3; free(p);return 0;}\n')
        execute(['cc', str(control), str(run / 'libnext_random.a'), '-lm', '-ldl', '-lpthread', '-o', str(run / 'abi-control')], 'abi-build')
        execute([str(run / 'abi-control')], 'abi-control')
        live_output = run / 'live'
        execute(['python3', str(ROOT / 'tools/decomp_live.py'), '--accepted', str(args.accepted.resolve()), '--next-random-library', str(run / 'libnext_random.a'), '--output', str(live_output)], 'live-run')
        live_run = next(p for p in live_output.iterdir() if p.is_dir())
        live = json.loads((live_run / 'verdict.json').read_text())
        if not live['mixed_integration']: raise ValueError('live integration failed')
        fixtures = live_run / 'c-0/epoch-0/fixtures'
        captured = []
        for before in sorted(fixtures.glob('rng-*-before.wram')):
            after = before.with_name(before.name.replace('before', 'after'))
            seed = int.from_bytes(before.read_bytes()[1509:1511], 'little')
            expected = after.read_bytes()
            for subject in ('c', 'rust'):
                output = run / (before.stem + '-' + subject + '.wram')
                result = int(execute([str(run / subject), str(before), str(output)], before.stem + '-' + subject))
                if output.read_bytes() != expected or result != int.from_bytes(expected[1509:1511], 'little'):
                    raise ValueError(f'first captured mismatch {before.name} {subject}')
            captured.append({'before': str(before), 'before_sha256': digest(before), 'after': str(after), 'after_sha256': digest(after), 'seed': seed})
        if len(captured) != 16: raise ValueError('missing observed fixtures')
        # Every actual call is checked against the exhaustive oracle table, including ordered semantic effects.
        calls = 0
        with gzip.open(live_run / 'c-0/epoch-0/hardware.log.gz', 'rt') as effects:
            for line in effects:
                parts = line.split()
                if len(parts) > 1 and parts[1] == 'rng':
                    seed, result = int(parts[3]), int(parts[4])
                    expected = int.from_bytes(c_results[seed * 2:seed * 2 + 2], 'little')
                    if result != expected or parts[6:] != [f'1509:{expected & 255}', f'1510:{expected >> 8}']:
                        raise ValueError(f'first ordered semantic effect mismatch call={calls + 1}')
                    calls += 1
        if calls != live['rng_subjects']['c-0'][0]['rng_c_hits']: raise ValueError('unlogged routine calls')
        verify_accepted(args.accepted)
        if digest(args.accepted) != predecessor: raise ValueError('predecessor changed')
        verdict.update(status='accepted', routine_accepted=True, captures=captured, observed_calls=calls, live_run=str(live_run), live_verdict_sha256=digest(live_run / 'verdict.json'), live_hits=live['rng_subjects'], binaries={p.name: digest(p) for p in (run / 'c', run / 'rust', run / 'libnext_random.a')}, remaining_CPU_fallback=live['remaining_gameplay_cpu_fallback'], scope='NextRandom all input words offline; observed 9000-frame mixed C/Rust prefix only; semantic LE store order, not machine instruction timing')
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        verdict['error'] = str(error)
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    if not verdict['routine_accepted']: raise ValueError(f'{verdict.get("error")}; evidence={run}')
    return {'status': 'accepted', 'accepted_jobs': [{'job': 'next-random', 'exercised': True, 'calls': verdict['observed_calls'], 'evidence': str(run), 'verdict_sha256': digest(run / 'verdict.json')}], 'remaining_CPU_fallback': verdict['remaining_CPU_fallback']}
