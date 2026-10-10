#!/usr/bin/env python3
"""Compile pinned actual bank routines and compare full canonical WRAM with Rust.

This establishes offline translation equivalence for declared valid cases, never
original-ROM parity or live execution. Every run retains its source identities,
commands, binaries, before/after states and mismatch verdict.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
WRAM_SIZE = 0x20000
IDENTIFIER = re.compile(r'^[A-Za-z_][A-Za-z_0-9]*$')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify_source(source):
    pin = next(p for p in json.loads((ROOT / 'tools/dependencies.json').read_text()) if p['name'] == 'snesrev-sm')
    archive_path = source.parent / 'downloads' / pin['archive']
    if digest(archive_path) != pin['sha256']:
        raise ValueError('pinned archive identity changed')
    identities = {}
    with tarfile.open(archive_path) as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            relative = Path(*Path(member.name).parts[1:])
            path = source / relative
            if path.read_bytes() != archive.extractfile(member).read():
                raise ValueError(f'pinned source changed: {relative}')
            identities[str(relative)] = digest(path)
    return {'revision': pin['revision'], 'archive_sha256': pin['sha256'], 'files': identities,
            'freshness': 'exact pinned archive; no latest-upstream assertion'}


def parse_result(stdout):
    # Original C diagnostics remain in retained stdout; final line is ABI result.
    lines = stdout.splitlines()
    if not lines or not re.fullmatch(rb'[0-9]+', lines[-1]):
        raise ValueError('missing final numeric routine return')
    return int(lines[-1])


def execute(command, run, label):
    with (run / 'commands.jsonl').open('a') as log:
        log.write(json.dumps(command) + '\n')
    result = subprocess.run(command, capture_output=True, timeout=180)
    (run / (label + '.stdout')).write_bytes(result.stdout)
    (run / (label + '.stderr')).write_bytes(result.stderr)
    if result.returncode:
        raise ValueError(f'{label} failed ({result.returncode}): {result.stderr.decode(errors="replace")}')
    return result.stdout


def build(manifest, source, run):
    prototypes, branches = [], []
    for routine in manifest['routines']:
        symbol, return_type = routine['symbol'], routine['return']
        types = routine.get('arg_types', ['uint16'] * routine.get('args', 0))
        count = len(types)
        if not IDENTIFIER.fullmatch(symbol) or count not in (0, 1, 2) or return_type not in ('void', 'uint8', 'uint16', 'uint32', 'int32') or any(t not in ('uint8', 'uint16', 'uint32', 'int32') for t in types):
            raise ValueError('invalid routine signature')
        c_type = {'void': 'void', 'uint8': 'uint8_t', 'uint16': 'uint16_t', 'uint32': 'uint32_t', 'int32': 'int32_t'}[return_type]
        prototypes.append(f'extern {c_type} {symbol}({", ".join(t + "_t" for t in types) or "void"});')
        call = f'{symbol}({", ".join("(" + t + "_t)" + arg for t, arg in zip(types, ["a", "b"]))})'
        result = f'{call}; result=0;' if return_type == 'void' else f'result={call};'
        branches.append(f'if (!strcmp(argv[1], "{symbol}")) {{ {result} }}')
    support = manifest.get('c_support', [])
    uses_rtl = 'sm_rtl.c' in support or manifest['bank'] == 'sm_rtl.c'
    globals_c = 'extern uint8_t g_ram[0x20000]; extern const uint8_t *g_rom;' if uses_rtl else 'uint8_t g_ram[0x20000]; const uint8_t *g_rom;'
    rom_setup = '''FILE *rf=fopen(argv[6],"rb"); if(!rf)return 2; if(fseek(rf,0,SEEK_END))return 2; long n=ftell(rf); rewind(rf); uint8_t *rom=malloc(n); if(!rom||fread(rom,1,n,rf)!=(size_t)n||fclose(rf))return 2; g_rom=rom;''' if 'rom' in manifest else ''
    oracle = run / 'oracle.c'
    oracle.write_text('''#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "sm_rtl.h"
#include "funcs.h"
''' + globals_c + '\n' + '\n'.join(prototypes) + '''
int main(int argc, char **argv) {
 if(argc!=6 && argc!=7) return 2;
 int64_t a=strtoll(argv[2],0,10), b=strtoll(argv[3],0,10); uint32_t result=0;
 ''' + rom_setup + '''
 FILE *f=fopen(argv[4],"rb");
 if(!f || fread(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fgetc(f)!=EOF || fclose(f)) return 2;
 ''' + ' else '.join(branches) + ''' else return 2;
 f=fopen(argv[5],"wb");
 if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f)) return 2;
 printf("%u\\n",result); return 0;
}
''')
    module = Path(manifest['rust_module'])
    if not module.is_absolute():
        module = ROOT / module
    signed = any('arg_types' in r for r in manifest['routines'])
    if 'rom' in manifest:
        rust_setup = 'let rom=std::fs::read(&args[6]).unwrap();'
        rust_call = 'family::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(),&rom,&args[1],args[2].parse::<i64>().unwrap() as i32,args[3].parse::<i64>().unwrap() as i32)'
    elif signed:
        rust_setup = ''
        rust_call = 'family::dispatch_i32(&mut Wram::new(&mut bytes).unwrap(),&args[1],args[2].parse::<i64>().unwrap() as i32,args[3].parse::<i64>().unwrap() as i32)'
    else:
        rust_setup = ''
        rust_call = 'family::dispatch(&mut Wram::new(&mut bytes).unwrap(),&args[1],args[2].parse().unwrap(),args[3].parse().unwrap())'
    imports = manifest.get('rust_imports', [])
    if any(not IDENTIFIER.fullmatch(name) for name in imports):
        raise ValueError('invalid Rust module import')
    root_imports = ''.join(f'use memory::{name};\n' for name in imports)
    wrapper = run / 'wrapper.rs'
    wrapper.write_text(f'''#[path = {json.dumps(str(ROOT / 'crates/port/src/lib.rs'))}] mod memory;
use memory::Wram;
{root_imports}#[path = {json.dumps(str(module))}] mod family;
fn main() {{
 let args:Vec<String>=std::env::args().collect(); assert!(args.len()==6 || args.len()==7);
 let mut bytes=std::fs::read(&args[4]).unwrap();
 {rust_setup}
 let result={rust_call}.unwrap();
 std::fs::write(&args[5],bytes).unwrap(); println!("{{}}",result);
}}
''')
    bank = source / 'src' / manifest['bank']
    if bank.parent != source / 'src' or not bank.name.startswith('sm_') or bank.suffix != '.c':
        raise ValueError('invalid source bank')
    extra = []
    for name in support:
        path = source / 'src' / name
        if path.parent != source / 'src' or path.suffix != '.c':
            raise ValueError('invalid C support source')
        if path != bank: extra.append(str(path))
    execute(['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source / 'src'), str(bank), *extra, str(oracle), '-Wl,--gc-sections', '-o', str(run / 'c')], run, 'c-build')
    execute(['rustc', '--edition=2024', '-O', str(wrapper), '-o', str(run / 'rust')], run, 'rust-build')
    shutil.copyfile(module, run / 'candidate.rs')
    shutil.copyfile(ROOT / 'crates/port/src/lib.rs', run / 'memory.rs')
    dependency_paths = {p.resolve() for p in (ROOT / 'crates/port/src').glob('*.rs')}
    for value in manifest.get('rust_dependencies', []):
        path = Path(value)
        dependency_paths.add((path if path.is_absolute() else ROOT / path).resolve())
    dependencies = {str(path): digest(path) for path in sorted(dependency_paths)}
    for index, path in enumerate(sorted(dependency_paths)):
        shutil.copyfile(path, run / f'dependency-{index:03d}.rs')
    return {'rust_dependencies': dependencies, 'module': str(module), 'module_sha256': digest(module), 'memory_sha256': digest(ROOT / 'crates/port/src/lib.rs'), 'harness_sha256': digest(__file__), 'binaries': {name: digest(run / name) for name in ('c', 'rust')}}


def load_asset(path, expected_digest, extent=None):
    path = Path(path)
    data = path.read_bytes()
    if not expected_digest or digest(path) != expected_digest:
        raise ValueError(f'asset identity mismatch: {path}')
    if extent is not None and len(data) != extent:
        raise ValueError(f'asset extent mismatch: {path}')
    return data


def prepare_case(case, directory, arg_types):
    name = case['name']
    if not re.fullmatch('[A-Za-z_0-9-]+', name):
        raise ValueError('case name must be a safe filename')
    fill = case.get('fill', 0)
    if not isinstance(fill, int) or not 0 <= fill <= 255:
        raise ValueError('invalid initial fill')
    if 'before_file' in case:
        if 'writes' in case or 'fill' in case:
            raise ValueError('captured before_file cannot be patched')
        ram = bytearray(load_asset(case['before_file'], case.get('before_sha256'), WRAM_SIZE))
    else:
        ram = bytearray([fill]) * WRAM_SIZE
    for offset, data in case.get('writes', []):
        if not isinstance(offset, int) or offset < 0 or offset + len(data) > WRAM_SIZE:
            raise ValueError('case write outside canonical WRAM')
        ram[offset:offset + len(data)] = bytes(data)
    for index, key in enumerate(('a', 'b')):
        kind = arg_types[index] if index < len(arg_types) else 'uint16'
        low, high = {'uint8': (0, 255), 'uint16': (0, 65535), 'int32': (-2147483648, 2147483647), 'uint32': (0, 4294967295)}[kind]
        if not isinstance(case.get(key, 0), int) or not low <= case.get(key, 0) <= high:
            raise ValueError(f'argument outside {kind}')
    directory.mkdir()
    (directory / 'before.wram').write_bytes(ram)
    if 'expected_after_file' in case:
        (directory / 'expected-after.wram').write_bytes(load_asset(case['expected_after_file'], case.get('expected_after_sha256'), WRAM_SIZE))
    (directory / 'case.json').write_text(json.dumps(case, indent=2) + '\n')


def verify(manifest_path, source, output):
    run = output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    verdict = {'status': 'rejected', 'run': str(run), 'live_exercised': False, 'original_rom_parity': 'not-tested', 'cases': []}
    try:
        manifest = json.loads(manifest_path.read_text())
        shutil.copyfile(manifest_path, run / 'manifest.json')
        verdict['family'] = manifest['family']
        verdict['source'] = verify_source(source)
        shutil.copyfile(source / 'LICENSE.txt', run / 'LICENSE.txt')
        verdict['compilers'] = {'c': subprocess.check_output(['cc', '--version'], text=True).splitlines()[0], 'rust': subprocess.check_output(['rustc', '--version'], text=True).strip()}
        if 'rom' in manifest:
            rom = load_asset(manifest['rom']['file'], manifest['rom'].get('sha256'))
            if not 0 < len(rom) <= 0x400000:
                raise ValueError('ROM extent outside LoROM view')
            (run / 'rom.bin').write_bytes(rom)
            verdict['rom'] = {'sha256': digest(run / 'rom.bin'), 'bytes': len(rom)}
        verdict['implementation'] = build(manifest, source, run)
        count = 0
        for routine in manifest['routines']:
            if not routine['cases']:
                raise ValueError(f'no declared valid cases for {routine["symbol"]}')
            for case in routine['cases']:
                directory = run / case['name']
                prepare_case(case, directory, routine.get('arg_types', ['uint16'] * routine.get('args', 0)))
                results = {}
                for subject in ('rust', 'c'):
                    # Bounded candidate admission precedes any unchecked C execution.
                    result = execute([str(run / subject), routine['symbol'], str(case.get('a', 0)), str(case.get('b', 0)), str(directory / 'before.wram'), str(directory / (subject + '-after.wram'))] + ([str(run / 'rom.bin')] if 'rom' in manifest else []), directory, subject)
                    results[subject] = parse_result(result)
                record = {'routine': routine['symbol'], 'case': case['name'], 'return': results, 'before_sha256': digest(directory / 'before.wram'), 'after_sha256': {subject: digest(directory / (subject + '-after.wram')) for subject in ('c', 'rust')}}
                verdict['cases'].append(record)
                if results['c'] != results['rust']:
                    raise ValueError(f'return mismatch: {routine["symbol"]}/{case["name"]}')
                if 'expected_return' in case and results['c'] != case['expected_return']:
                    raise ValueError(f'independent expected return mismatch: {case["name"]}')
                c, rust = [(directory / (subject + '-after.wram')).read_bytes() for subject in ('c', 'rust')]
                if len(c) != WRAM_SIZE or len(rust) != WRAM_SIZE:
                    raise ValueError('subject did not retain full WRAM extent')
                if c != rust:
                    offset = next(i for i in range(WRAM_SIZE) if c[i] != rust[i])
                    record['first_mismatch'] = {'offset': offset, 'c': c[offset], 'rust': rust[offset]}
                    raise ValueError(f'WRAM mismatch at {offset:#x}: {routine["symbol"]}/{case["name"]}')
                if 'expected_after_file' in case:
                    expected = (directory / 'expected-after.wram').read_bytes()
                    record['capture_after_sha256'] = digest(directory / 'expected-after.wram')
                    if c != expected:
                        offset = next(i for i in range(WRAM_SIZE) if c[i] != expected[i])
                        raise ValueError(f'captured expected WRAM mismatch at {offset:#x}: {case["name"]}')
                count += 1
        if not count:
            raise ValueError('empty family cannot be verified')
        verdict.update(status='offline-verified', verified_cases=count, verified_routines=len(manifest['routines']), scope='declared valid fixture domain only; full canonical WRAM and return values')
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        verdict['error'] = str(error)
    verdict['artifacts'] = {str(path.relative_to(run)): digest(path) for path in sorted(run.rglob('*')) if path.is_file() and path.name != 'verdict.json'}
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    return verdict




def verify_family_evidence(verdict_path):
    """Authenticate retained offline evidence before using it as a live input.

    Integrity is relative to this local verdict and current pinned implementation,
    not an external cryptographic signer. Rechecks actual output files and coverage.
    """
    verdict_path = Path(verdict_path)
    verdict = json.loads(verdict_path.read_text())
    run = verdict_path.parent
    if verdict.get('status') != 'offline-verified' or not verdict.get('cases'):
        raise ValueError('family evidence is not nonempty offline verification')
    artifacts = verdict.get('artifacts')
    if not artifacts:
        raise ValueError('family evidence missing artifact inventory; regenerate')
    for name, expected in artifacts.items():
        path = run / name
        if run.resolve() not in path.resolve().parents or digest(path) != expected:
            raise ValueError(f'family artifact changed: {name}')
    required = {'manifest.json', 'c', 'rust', 'candidate.rs', 'memory.rs', 'oracle.c', 'wrapper.rs'}
    if not required <= artifacts.keys():
        raise ValueError('family artifacts incomplete')
    manifest = json.loads((run / 'manifest.json').read_text())
    pin = next(p for p in json.loads((ROOT / 'tools/dependencies.json').read_text()) if p['name'] == 'snesrev-sm')
    if verdict['source']['revision'] != pin['revision'] or verdict['source']['archive_sha256'] != pin['sha256']:
        raise ValueError('family pinned source identity changed')
    implementation = verdict['implementation']
    for path, expected in [(Path(implementation['module']), implementation['module_sha256']),
                           (ROOT / 'crates/port/src/lib.rs', implementation['memory_sha256']),
                           (Path(__file__), implementation['harness_sha256'])]:
        if digest(path) != expected:
            raise ValueError(f'family implementation changed: {path}')
    for dependency, expected in implementation.get('rust_dependencies', {}).items():
        if digest(dependency) != expected:
            raise ValueError(f'family Rust dependency changed: {dependency}')
    for binary in ('c', 'rust'):
        if digest(run / binary) != implementation['binaries'][binary]:
            raise ValueError(f'family binary changed: {binary}')
    expected_cases = {(r['symbol'], c['name']): c for r in manifest['routines'] for c in r['cases']}
    if not expected_cases or len(expected_cases) != sum(len(r['cases']) for r in manifest['routines']):
        raise ValueError('family manifest has empty or duplicate cases')
    observed = set()
    for case in verdict['cases']:
        key = (case['routine'], case['case'])
        if key not in expected_cases or key in observed:
            raise ValueError('family coverage changed')
        observed.add(key)
        directory = run / case['case']
        paths = [directory / 'before.wram', directory / 'c-after.wram', directory / 'rust-after.wram']
        if any(str(path.relative_to(run)) not in artifacts for path in paths):
            raise ValueError('family case artifacts incomplete')
        before, c, rust = [path.read_bytes() for path in paths]
        if any(len(data) != WRAM_SIZE for data in (before, c, rust)) or c != rust:
            raise ValueError('family actual full WRAM outputs differ')
        results = {subject: parse_result((directory / (subject + '.stdout')).read_bytes()) for subject in ('c', 'rust')}
        if results != case['return'] or results['c'] != results['rust']:
            raise ValueError('family actual return outputs differ')
        config = expected_cases[key]
        if 'expected_return' in config and results['c'] != config['expected_return']:
            raise ValueError('family expected return changed')
        if 'expected_after_file' in config and str((directory / 'expected-after.wram').relative_to(run)) not in artifacts:
            raise ValueError('family capture artifact missing')
        if 'expected_after_file' in config and c != (directory / 'expected-after.wram').read_bytes():
            raise ValueError('family actual captured output differs')
    if observed != expected_cases.keys() or verdict.get('verified_cases') != len(observed) or verdict.get('verified_routines') != len(manifest['routines']):
        raise ValueError('family coverage incomplete')
    return verdict


def inventory(source):
    """Lexical definition/call index, including unresolved and indirect calls.

    Comments and literals are blanked before brace matching so nested bodies
    stay intact. This is a dependency discovery aid, not a C call-graph proof.
    """
    identity = verify_source(source)
    routines = []
    literal = re.compile(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'', re.S)
    definition = re.compile(r'^\s*((?:static\s+)?[A-Za-z_][A-Za-z_0-9\s*]*?)\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(([^;{}]*?)\)\s*\{', re.M)
    for path in sorted((source / 'src').glob('sm_*.c')):
        text = path.read_text()
        clean = literal.sub(lambda m: ''.join('\n' if c == '\n' else ' ' for c in m.group()), text)
        for match in definition.finditer(clean):
            start, depth, cursor = match.end() - 1, 1, match.end()
            while cursor < len(clean) and depth:
                depth += (clean[cursor] == '{') - (clean[cursor] == '}')
                cursor += 1
            if depth:
                raise ValueError(f'unbalanced body: {path.name}:{match.group(2)}')
            body = clean[start:cursor]
            suffix = text[start:min(cursor, start + 160)].split('\n', 1)[0]
            address = re.search(r'//\s*(0x[0-9a-fA-F]{6})\b', suffix)
            calls = sorted(set(re.findall(r'\b([A-Za-z_][A-Za-z_0-9]*)\s*\(', body)) - {'if', 'for', 'while', 'switch', 'sizeof'})
            routines.append({'symbol': match.group(2), 'source': str(path.relative_to(source)),
                             'line': text.count('\n', 0, match.start(2)) + 1,
                             'original_address': address.group(1) if address else None,
                             'signature': ' '.join(clean[match.start():start].split()),
                             'body_lines': body.count('\n') + 1,
                             'definition_sha256': hashlib.sha256(text[match.start():cursor].encode()).hexdigest(),
                             'calls': calls, 'indirect_indexed_calls': sorted(set(re.findall(r'\b([A-Za-z_][A-Za-z_0-9]*)\s*\[[^;{}]*?\]\s*\(', body))), 'translation_status': 'unverified'})
    symbols = {r['symbol'] for r in routines}
    for routine in routines:
        routine['defined_dependencies'] = [call for call in routine['calls'] if call in symbols]
        routine['external_or_macro_calls'] = [call for call in routine['calls'] if call not in symbols]
    banks = {}
    for routine in routines:
        bank = banks.setdefault(routine['source'], {'definitions': 0, 'body_lines': 0})
        bank['definitions'] += 1
        bank['body_lines'] += routine['body_lines']
    return {'source_identity': identity, 'scope': 'actual sm_*.c definitions; lexical direct calls, unresolved macro/external/indirect calls require review; no acceptance inferred',
            'definition_count': len(routines), 'banks': banks, 'routines': routines}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    family = sub.add_parser('verify')
    family.add_argument('family', nargs='?')
    family.add_argument('--manifest', type=Path)
    family.add_argument('--source', type=Path, default=ROOT / 'deps/snesrev-sm')
    family.add_argument('--output', type=Path, required=True)
    index = sub.add_parser('inventory')
    index.add_argument('--source', type=Path, default=ROOT / 'deps/snesrev-sm')
    index.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'inventory':
        result = inventory(args.source.resolve())
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps({'definitions': result['definition_count'], 'output': str(args.output)}))
        return 0
    manifest = args.manifest or ROOT / 'tools/port' / (str(args.family) + '.json')
    verdict = verify(manifest, args.source.resolve(), args.output)
    print(json.dumps({key: verdict[key] for key in ('status', 'run', 'error') if key in verdict}))
    return 0 if verdict['status'] == 'offline-verified' else 1

if __name__ == '__main__':
    raise SystemExit(main())
