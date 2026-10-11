#!/usr/bin/env python3
"""Exercise offline-verified Rust families in independent C/mixed replay."""
import argparse
import datetime
import json
import hashlib
import os
import uuid
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DISPATCHER_LEAVES = {
    'MoveEprojWithVelocity': 'uint16',
    'MoveEprojWithVelocityX': 'uint16',
    'MoveEprojWithVelocityY': 'uint16',
    'Eproj_FuncE73E_MoveXY': 'void',
    'Eproj_FuncE722': 'uint16',
}
DISPATCHER_SYNTHETIC = {'EprojInstruction', 'EprojPreinstruction'}


def load_dispatcher_contract(path, source_identity):
    """Authenticate dispatcher seals and admit only the supported live leaves."""
    from port_plm_instructions import verify_plm_instruction_evidence
    evidence = verify_plm_instruction_evidence(path)
    manifest = json.loads((path.parent / 'manifest.json').read_text())
    module = Path(evidence['implementation']['module']).resolve()
    if evidence['source'] != source_identity:
        raise ValueError('dispatcher source identity differs')
    if module != ROOT / 'crates/port/src/enemy_projectile_instructions.rs' or manifest.get('bank') != 'sm_86.c':
        raise ValueError('dispatcher evidence does not seal the installed bank86 module')
    routines = []
    for routine in manifest['routines']:
        routine = dict(routine)
        symbol = routine['symbol']
        if symbol in DISPATCHER_SYNTHETIC:
            routine['live_eligible'] = False
        elif routine.get('live_eligible', True):
            if symbol not in DISPATCHER_LEAVES:
                raise ValueError(f'dispatcher routine requires support selection before live admission: {symbol}')
            if routine.get('arg_types', ['uint16'] * routine.get('args', 0)) != ['uint16'] or routine['return'] != DISPATCHER_LEAVES[symbol]:
                raise ValueError(f'dispatcher leaf signature differs: {symbol}')
        routines.append(routine)
    if not any(r.get('live_eligible', True) for r in routines):
        raise ValueError('no live dispatcher leaf contracts')
    manifest = dict(manifest, routines=routines)
    manifest['rust_imports'] = sorted(set(manifest.get('rust_imports', [])) | {
        'audio', 'enemy', 'enemy_projectiles', 'enemy_projectile_collision', 'system',
    })
    return manifest, module


def stream_digest(path):
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def share_finished_outputs(directory, inventory, anchor_verdict):
    """Share identical sealed outputs only after the producing process exits."""
    anchor = json.loads(anchor_verdict.read_text())
    # Historical source seals may be stale: this authenticates immutable bytes,
    # not admission of the historical implementation against current source.
    if stream_digest(anchor_verdict.parent / 'live') != anchor.get('binary_sha256'):
        raise ValueError('dedup anchor binary seal differs')
    if anchor.get('status') != 'mixed-C-Rust-integration':
        raise ValueError('dedup anchor is not an accepted live run')
    reclaimed = 0
    for relative in ('hardware.log', 'rom-view.bin'):
        expected = anchor['artifacts'].get(relative)
        previous = anchor_verdict.parent / 'c-0' / relative
        current = directory / relative
        if not expected or previous.is_symlink() or current.is_symlink():
            raise ValueError('invalid dedup anchor output')
        if stream_digest(previous) != expected:
            raise ValueError('dedup anchor output seal differs')
        if inventory.get(relative) != expected:
            continue
        if stream_digest(current) != expected:
            raise ValueError('finished output seal differs before dedup')
        old = current.stat()
        prior = previous.stat()
        if old.st_dev != prior.st_dev or old.st_ino == prior.st_ino:
            continue
        if old.st_size != prior.st_size:
            raise ValueError('sealed dedup output sizes differ')
        with previous.open('rb') as left, current.open('rb') as right:
            while True:
                chunk = left.read(1024 * 1024)
                if chunk != right.read(1024 * 1024):
                    raise ValueError('sealed dedup output bytes differ')
                if not chunk:
                    break
        temporary = current.with_name(current.name + '.dedup-' + uuid.uuid4().hex)
        try:
            os.link(previous, temporary)
            os.replace(temporary, current)
        finally:
            temporary.unlink(missing_ok=True)
        if stream_digest(current) != expected:
            raise ValueError('finished output seal differs after dedup')
        if old.st_nlink == 1:
            reclaimed += old.st_size
    return reclaimed


def definition_pattern(symbol):
    """Match a definition, including const pointer returns, without call sites."""
    return re.compile(r'(?m)^((?:[A-Za-z_]\w*[ \t]+)+(?:\*[ \t]*)?)' + re.escape(symbol)
                      + r'(\s*\([^\n]*\)\s*\{)')


def generate_plm_abi(positioned=False):
    """Choose supported setup before mutation; errors never trigger a C retry."""
    abi = '''#[unsafe(no_mangle)]
pub unsafe extern "C" fn port_native_plm(ram:*mut u8,len:usize,rom:*const u8,rom_len:usize,header:u16,amount:*mut i32)->i32 {
 if ram.is_null() || len!=WRAM_SIZE || rom.is_null() || rom_len==0 || rom_len>0x400000 || amount.is_null() || (amount as usize)%4!=0 {return -1;}
 let Some(rend)=(ram as usize).checked_add(len) else {return -1;};
 let Some(oend)=(rom as usize).checked_add(rom_len) else {return -1;};
 let Some(aend)=(amount as usize).checked_add(4) else {return -1;};
 if (ram as usize)<oend && (rom as usize)<rend || (amount as usize)<rend && (ram as usize)<aend || (amount as usize)<oend && (rom as usize)<aend {return -1;}
 let rom=unsafe {std::slice::from_raw_parts(rom,rom_len)};
 let supported=std::panic::catch_unwind(||memory::plm::supports_header(rom,header));
 match supported {Ok(Ok(false))=>return -2,Ok(Ok(true))=>{},_=>return -1}
 let mut owned=unsafe {std::slice::from_raw_parts(ram,len)}.to_vec();
 let mut displacement=unsafe {*amount};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||memory::plm::spawn(&mut Wram::new(&mut owned).unwrap(),rom,header,&mut displacement)));
 match result {Ok(Ok(flags))=>{unsafe {std::ptr::copy_nonoverlapping(owned.as_ptr(),ram,len);*amount=displacement;}flags.into()},_=>-1}
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn port_native_positioned_plm(ram:*mut u8,len:usize,rom:*const u8,rom_len:usize,kind:u16,a:u16,b:u16)->i32 {
 if ram.is_null()||len!=WRAM_SIZE||rom.is_null()||rom_len==0||rom_len>0x400000{return -1;}
 let Some(rend)=(ram as usize).checked_add(len)else{return -1;};let Some(oend)=(rom as usize).checked_add(rom_len)else{return -1;};
 if (ram as usize)<oend&&(rom as usize)<rend{return -1;}
 let rom=unsafe{std::slice::from_raw_parts(rom,rom_len)};
 let mut owned=unsafe{std::slice::from_raw_parts(ram,len)}.to_vec();
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  let mut r=Wram::new(&mut owned).unwrap();
  if !(0..40).any(|i|r.word(0x1c37+i*2)==0){return Ok(0);}
  let header=match kind{1=>memory::plm::room_entry(&r,rom,a)?.0,2=>a,_=>return Err("allocation kind")};
  if !memory::plm::supports_positioned_header(rom,header)?{return Ok(-2);}
  match kind{1=>memory::plm::spawn_room(&mut r,rom,a)?,2=>memory::plm::spawn_hardcoded(&mut r,rom,b as u8,(b>>8) as u8,a)?,_=>unreachable!()};Ok(1)
 }));
 match result{Ok(Ok(value))=>{if value>=0{unsafe{std::ptr::copy_nonoverlapping(owned.as_ptr(),ram,len);}}value},_=>-1}
}'''
    return abi if positioned else abi[:abi.index('#[unsafe(no_mangle)]', 1)]


def generate_positioned_plm_host():
    """Intercept both allocation entries, including original same-bank callers."""
    return r"""void PortC_SpawnRoomPLM(uint16);
void PortC_SpawnHardcodedPlm(SpawnHardcodedPlmArgs);
extern int32_t port_native_positioned_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,uint16_t,uint16_t);
static void PortPositionedPlm(unsigned kind,uint16 a,uint16 b) {
 int32 native=-2;
 if(port_mixed)native=port_native_positioned_plm(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,kind,a,b);
 port_room_total[kind-1]++;
 if(native==-2){port_room_c[kind-1]++;if(kind==1)PortC_SpawnRoomPLM(a);else PortC_SpawnHardcodedPlm((SpawnHardcodedPlmArgs){b,b>>8,a});}
 else {if(native<0){fprintf(stderr,"room PLM error %u %u\n",kind,a);exit(4);}port_room_rust[kind-1]++;port_room_setups[kind-1]+=native;}
 fprintf(port_effects,"room-plm %u %u %u\n",kind,a,b);
}
void SpawnRoomPLM(uint16 entry){PortPositionedPlm(1,entry,0);}
void SpawnHardcodedPlm(SpawnHardcodedPlmArgs p){PortPositionedPlm(2,p.field_2,p.field_0|p.field_1<<8);}
"""


def generate_family_abi(manifest, module, index):
    """Generate the bounded synchronous ABI used by the live host."""
    if 'rom' in manifest:
        call = f'family{index}::dispatch_rom(&mut Wram::new(bytes).unwrap(),rom,name,a,b)'
    elif any('arg_types' in r for r in manifest['routines']):
        call = f'family{index}::dispatch_i32(&mut Wram::new(bytes).unwrap(),name,a,b)'
    else:
        call = f'family{index}::dispatch(&mut Wram::new(bytes).unwrap(),name,a as u16,b as u16)'
    return f'''#[path={json.dumps(str(module))}] mod family{index};
#[unsafe(no_mangle)]
pub unsafe extern "C" fn port_family_{index}(ram:*mut u8,len:usize,rom:*const u8,rom_len:usize,id:u32,a:i32,b:i32)->i64 {{
 if ram.is_null() || len!=0x20000 || (ram as usize).checked_add(len).is_none() {{return -1;}}
 if rom.is_null() || rom_len==0 || rom_len>0x400000 || (rom as usize).checked_add(rom_len).is_none() {{return -1;}}
 if (ram as usize)<(rom as usize)+rom_len && (rom as usize)<(ram as usize)+len {{return -1;}}
 let names={json.dumps([r['symbol'] for r in manifest['routines']])};
 let Some(name)=names.get(id as usize) else {{return -1;}};
 let bytes=unsafe {{std::slice::from_raw_parts_mut(ram,len)}};
 let rom=unsafe {{std::slice::from_raw_parts(rom,rom_len)}};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{call}));
 match result {{Ok(Ok(value))=>i64::from(value),_=>-1}}
}}'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--family-verdict', type=Path, action='append', default=[])
    parser.add_argument('--collision-verdict', type=Path, action='append', default=[])
    parser.add_argument('--dispatcher-verdict', type=Path, action='append', default=[],
                        help='Authenticated bank86 dispatcher evidence for safe movement and extent leaves')
    parser.add_argument('--dedup-anchor-verdict', type=Path, help='Share byte-identical finished logs with a sealed prior live run')
    parser.add_argument('--plm-verdict', type=Path)
    parser.add_argument('--room-plm', action='store_true', help='Require native room/hardcoded setup hits')
    parser.add_argument('--frames', type=int, default=9000)
    parser.add_argument('--accepted', type=Path, default=ROOT / 'out/port/decompressor-offline/decomp-acceptance.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    verdict = {'status': 'rejected', 'live_exercised': False,
               'original_rom_parity': 'not-tested', 'commands': []}
    try:
        from port_family import digest, verify_source, verify_family_evidence
        from decomp_baseline import ROM_SHA256
        source = ROOT / 'deps/snesrev-sm'
        verdict['source'] = verify_source(source)
        if not 1 <= args.frames <= 129712:
            raise ValueError('frames outside supplied movie extent')
        rom = ROOT / 'rom/sm.sfc'
        inputs = ROOT / 'movies/super-metroid-any-3653.inputs'
        if digest(rom) != ROM_SHA256:
            raise ValueError('original ROM identity differs')
        verdict.update(rom_sha256=digest(rom), inputs_sha256=digest(inputs), frames=args.frames)
        families = []
        dispatcher_indexes = set()
        for path in args.family_verdict:
            previous = verify_family_evidence(path)
            manifest = json.loads((path.parent / 'manifest.json').read_text())
            module = Path(previous['implementation']['module'])
            if digest(module) != previous['implementation']['module_sha256']:
                raise ValueError('family source changed since verification')
            if digest(ROOT / 'crates/port/src/lib.rs') != previous['implementation']['memory_sha256']:
                raise ValueError('shared memory boundary changed since verification')
            if previous['source'] != verdict['source']:
                raise ValueError('family source identity differs')
            families.append((manifest, module))
        collision_indexes = set()
        for path in args.dispatcher_verdict:
            dispatcher_indexes.add(len(families))
            families.append(load_dispatcher_contract(path, verdict['source']))
            verdict.setdefault('dispatcher_predecessors', []).append({'path': str(path.resolve()), 'sha256': digest(path)})
        for collision_path in args.collision_verdict:
            from port_collision import verify_collision_evidence
            evidence = verify_collision_evidence(collision_path)
            collision_manifest = json.loads((collision_path.parent / 'manifest.json').read_text())
            if collision_manifest.get('infrastructure_probe') or not evidence.get('gameplay_translation', True):
                raise ValueError('infrastructure callback probe cannot replace gameplay')
            collision_module = Path(collision_manifest['rust_module'])
            if not collision_module.is_absolute():
                collision_module = ROOT / collision_module
            collision_indexes.add(len(families))
            families.append((collision_manifest, collision_module))
        if not families:
            raise ValueError('no family contracts')
        if args.room_plm and not args.plm_verdict:
            raise ValueError('room PLM requires allocation evidence')
        if args.plm_verdict:
            from port_plm import verify_plm_evidence
            plm_evidence = verify_plm_evidence(args.plm_verdict)
            if plm_evidence['source'] != verdict['source']:
                raise ValueError('native PLM source identity differs')
            if Path(plm_evidence['implementation']['module']).resolve() != ROOT / 'crates/port/src/plm.rs':
                raise ValueError('native PLM evidence does not seal the installed module')
            if args.room_plm:
                if not {'room','hardcoded'} <= plm_evidence.get('allocator_cases',{}).keys():raise ValueError('positioned allocation coverage missing')
            verdict['plm_predecessor_sha256'] = digest(args.plm_verdict)
        from decomp_routine import verify_accepted
        verify_accepted(args.accepted)
        verdict['decompressor_predecessor_sha256'] = digest(args.accepted)
        def execute(command, directory, label):
            verdict['commands'].append({'argv': command, 'cwd': str(directory)})
            logs = run if directory == ROOT else directory
            with (logs / (label + '.stdout')).open('wb') as stdout, (logs / (label + '.stderr')).open('wb') as stderr:
                timeout = max(240, args.frames / 40) if label == 'replay' else 240
                result = subprocess.run(command, cwd=directory, stdout=stdout, stderr=stderr, timeout=timeout)
            if result.returncode:
                raise ValueError(f'{label} failed with exit {result.returncode}')
        rust = [f'#[path={json.dumps(str(ROOT / "crates/port/src/lib.rs"))}] mod memory;', 'use memory::*;']
        if args.plm_verdict:
            rust.append(generate_plm_abi(args.room_plm))
        imports = sorted({name for manifest, _ in families for name in manifest.get('rust_imports', [])})
        for name in imports:
            if not re.fullmatch('[A-Za-z_][A-Za-z_0-9]*', name):
                raise ValueError('invalid imported gameplay module')
            rust.append(f'use memory::{name};')
        wrappers, symbols, units = [], [], sorted((source / 'src').glob('sm_*.c'))
        bank_texts = {}
        for family_index, (manifest, module) in enumerate(families):
            if family_index in collision_indexes:
                from port_collision import generate_collision_abi
                abi_symbol = f'port_collision_{family_index}'
                generated = generate_collision_abi(manifest, module).replace('port_collision_call', abi_symbol)
                rust.append(f'mod collision{family_index} {{use super::*;\n{generated}\n}}')
            else:
                rust.append(generate_family_abi(manifest, module, family_index))
                abi_symbol = f'port_family_{family_index}'
            wrappers.append(f'extern int64_t {abi_symbol}(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);')
            for routine_index, routine in enumerate(manifest['routines']):
                if routine.get('live_eligible', True) is not True:
                    continue
                symbol = routine['symbol']
                types = routine.get('arg_types', ['uint16'] * routine.get('args', 0))
                count = len(types)
                if symbol in symbols or not re.fullmatch('[A-Za-z_][A-Za-z_0-9]*', symbol):
                    raise ValueError('duplicate or invalid live symbol')
                index = len(symbols)
                symbols.append(symbol)
                signature = definition_pattern(symbol)
                owners = [bank for bank in sorted((source / 'src').glob('sm_*.c')) if signature.search(bank.read_text())]
                if len(owners) != 1:
                    raise ValueError(f'expected one owning source unit for {symbol}')
                bank = owners[0]
                text = bank_texts.get(bank, bank.read_text())
                text, replacements = signature.subn(r'\1C_' + symbol + r'\2', text)
                if replacements != 1:
                    raise ValueError(f'definition rename count differs: {symbol}')
                # Some bank-local functions have no funcs.h declaration. Keep a
                # declaration for callers after renaming their original definition.
                definition = signature.search(bank.read_text())
                prototype = definition.group(0).rsplit('{', 1)[0].rstrip() + ';\n'
                include_end = text.find('#include "funcs.h"')
                if include_end < 0:
                    raise ValueError(f'owning unit lacks declarations include: {symbol}')
                include_end = text.index('\n', include_end) + 1
                text = text[:include_end] + prototype + text[include_end:]
                bank_texts[bank] = text
                c_type = {'void': 'void', 'uint8': 'uint8_t', 'uint16': 'uint16_t', 'uint32': 'uint32_t', 'int32': 'int32_t', 'rom_ptr': 'const uint16_t *'}[routine['return']]
                declaration = ','.join(('const uint16_t *' if kind == 'rom_ptr' else kind + '_t ') + name for kind, name in zip(types, ['a', 'b'])) or 'void'
                arguments = ','.join(['a', 'b'][:count])
                actual_args = ['a' if count > 0 else '0', 'b' if count > 1 else '0']
                if 'rom_ptr' in types:
                    actual_args = [f'PortPointerResult({name},a)' if kind == 'rom_ptr' else name for kind, name in zip(types, ['a', 'b'])]
                actual = ','.join(actual_args)
                c_call = f'C_{symbol}({arguments})'
                reference = f'{c_call};' if c_type == 'void' else f'result=(uint32_t){c_call};'
                returned = '' if c_type == 'void' else f'return ({c_type})result;'
                if routine['return'] == 'rom_ptr':
                    reference = f'const uint16_t *reference={c_call};result=PortPointerResult(reference,a);'
                    returned = 'return PortPointerFromResult((uint32_t)result);'
                wrappers.append(f'''extern {c_type} C_{symbol}({declaration});
{c_type} {symbol}({declaration}) {{
 int64_t result=0;
 int outer=port_call_depth++==0;
 unsigned long capture=outer?++port_outer_hits[{index}]:0;
 if(outer && capture<=8)PortCapture({index},capture,"before");
 if(port_mixed) {{result={abi_symbol}(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,{routine_index},{actual}); if(result<0) {{fprintf(stderr,"Rust family error {symbol}\\n");exit(4);}} port_rust_hits[{index}]++;}}
 else {{{reference} port_c_hits[{index}]++;}}
 port_call_depth--;
 if(outer) {{
  if(capture<=8)PortCapture({index},capture,"after");
  fprintf(port_calls,"{index} %lu %lld %lld %lld\\n",capture,(long long)({actual_args[0]}),(long long)({actual_args[1]}),(long long)result);
 }}
 {returned}
}}''')
        library_source = run / 'families.rs'
        library_source.write_text('\n'.join(rust))
        execute(['rustc', '--edition=2024', '-O', '--crate-type=staticlib', str(library_source), '-o', str(run / 'libfamilies.a')], run, 'rust-build')
        execute(['cargo', 'build', '-p', 'decomp', '--lib', '--release', '--locked', '--offline', '--target-dir', str(run / 'target')], ROOT, 'decompressor-build')
        verify_accepted(args.accepted)
        execute(['rustc', '--edition=2024', '-O', '--crate-type=staticlib', str(ROOT / 'tools/decomp/next_random.rs'), '-o', str(run / 'librng.a')], run, 'rng-build')
        verdict['rng_source_sha256'] = digest(ROOT / 'tools/decomp/next_random.rs')
        rng_cli = run / 'rng-cli'
        execute(['rustc', '--edition=2024', '-O', '--cfg', 'cli', str(ROOT / 'tools/decomp/next_random.rs'), '-o', str(rng_cli)], run, 'rng-cli-build')
        execute([str(rng_cli), 'exhaustive'], run, 'rng-exhaustive')
        rng_predecessor = ROOT / 'out/decomp/review/jobs/20261010T185638.603810Z'
        rng_verified = json.loads((rng_predecessor / 'verdict.json').read_text())
        if rng_verified.get('status') != 'accepted' or rng_verified['implementation']['tools/decomp/next_random.rs'] != verdict['rng_source_sha256']:
            raise ValueError('RNG source differs from accepted exhaustive predecessor')
        if digest(run / 'rng-exhaustive.stdout') != rng_verified['synthetic']['table_sha256']:
            raise ValueError('RNG exhaustive behavior differs from predecessor')
        verdict['rng_predecessor_sha256'] = digest(rng_predecessor / 'verdict.json')
        if args.room_plm:
            bank=source/'src/sm_84.c'
            text=bank_texts.get(bank,bank.read_text())
            for symbol in ('SpawnRoomPLM','SpawnHardcodedPlm'):
                pattern=definition_pattern(symbol)
                if len(pattern.findall(text))!=1:raise ValueError('allocator definition count differs')
                text=pattern.sub(lambda m:m.group(1)+'PortC_'+symbol+m.group(2),text)
            bank_texts[bank]=text
        rng_bank = source / 'src/sm_80.c'
        rng_text = bank_texts.get(rng_bank, rng_bank.read_text())
        if rng_text.count('uint16 NextRandom(void) {') != 1:
            raise ValueError('RNG definition rename count differs')
        bank_texts[rng_bank] = rng_text.replace('uint16 NextRandom(void) {', 'uint16 C_NextRandom(void) {')
        transformations = {}
        for bank, text in bank_texts.items():
            transformed = run / bank.name
            transformed.write_text(text)
            units[units.index(bank)] = transformed
            transformations[bank.name] = {'original_sha256': digest(bank), 'transformed_sha256': digest(transformed)}
        host = run / 'host.c'
        host_dependencies = [ROOT / 'tools/decomp/headless.c', ROOT / 'tools/decomp/headless_frame.h']
        verdict['host_dependencies'] = {}
        for dependency in host_dependencies:
            shutil.copyfile(dependency, run / dependency.name)
            verdict['host_dependencies'][dependency.name] = digest(dependency)
        host.write_text('''#define main retained_main
#define __wrap_DecompressToMem retained_decompression_observer
''' + f'#include {json.dumps(str(run / "headless.c"))}\n' + '''#undef main
#undef __wrap_DecompressToMem
#include "snes/ppu.h"
#include "funcs.h"
static int port_mixed;
static unsigned long port_decompress_c,port_decompress_rust,port_rng_c,port_rng_rust;
typedef struct PortDecompressReport {size_t written;uint32_t final_source,modes;} PortDecompressReport;
extern int sm_decompress_v1(const uint8_t*,size_t,uint8_t*,size_t,uint32_t,size_t,PortDecompressReport*);
extern int32_t sm_next_random_v1(uint8_t*,size_t);
extern uint16 C_NextRandom(void);
uint16 NextRandom(void) {
 if(!port_mixed){port_rng_c++;return C_NextRandom();}
 int32_t result=sm_next_random_v1(g_ram,sizeof(g_ram));
 if(result<0)exit(4);port_rng_rust++;return result;
}
void __wrap_DecompressToMem(uint32 src,uint8 *dst) {
 if(!port_mixed){retained_decompression_observer(src,dst);port_decompress_c++;return;}
 uintptr_t base=(uintptr_t)g_ram,address=(uintptr_t)dst;
 if(address<base || address-base>=sizeof(g_ram))exit(4);
 decompression_calls++;SaveRoutineState("before");
 PortDecompressReport report={0};
 if(sm_decompress_v1(g_rom,g_snes->cart->romSize,g_ram,sizeof(g_ram),src,address-base,&report))exit(4);
 port_decompress_rust++;SaveRoutineState("after");
}
static FILE *port_effects;
static FILE *port_calls;
static unsigned port_call_depth;
extern int32 *cur_coll_amt32;
static unsigned long port_plm_callbacks,port_plm_c,port_plm_rust,port_plm_rust_setups;
''' + ('#define PORT_NATIVE_PLM\nextern int32_t port_native_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);\n' if args.plm_verdict else '') + '''
extern uint8 __real_SpawnPLM(uint16);
uint8 __wrap_SpawnPLM(uint16 header) {
 int32 before=cur_coll_amt32?*cur_coll_amt32:0;
 uint8 result;int32 displacement=before;int32 native=-2;
 int free_slot=0;for(unsigned j=0;j<80;j+=2)if(GET_WORD(g_ram+0x1c37+j)==0){free_slot=1;break;}
#ifdef PORT_NATIVE_PLM
 // Native acceptance covers synchronous collision context with a real amount.
 if(port_mixed && cur_coll_amt32)native=port_native_plm(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,header,&displacement);
#endif
 if(native==-2){result=__real_SpawnPLM(header);port_plm_c++;}
 else {if(native<0){fprintf(stderr,"native PLM error %u\\n",header);exit(4);}result=native;if(cur_coll_amt32)*cur_coll_amt32=displacement;port_plm_rust++;if(free_slot)port_plm_rust_setups++;}
 port_plm_callbacks++;
 fprintf(port_effects,"plm %u %d %d %u\\n",header,before,cur_coll_amt32?*cur_coll_amt32:0,result);
 return result;
}
int32_t sm_rust_collision_spawn_plm(uint8_t *transaction,size_t len,uint16_t header,int32_t *amount) {
 if(!transaction || len!=sizeof(g_ram) || !amount || transaction==g_ram)return -1;
 // Owned Rust transaction has no alias with canonical C state or retained refs.
 memcpy(g_ram,transaction,len);
 int32 *previous=cur_coll_amt32;cur_coll_amt32=amount;
 uint8 result=__wrap_SpawnPLM(header);cur_coll_amt32=previous;
 memcpy(transaction,g_ram,len);
 return result;
}
int32_t sm_rust_collision_spawn_plm_without_amount(uint8_t *transaction,size_t len,uint16_t header) {
 if(!transaction || len!=sizeof(g_ram) || transaction==g_ram)return -1;
 memcpy(g_ram,transaction,len);
 int32 *previous=cur_coll_amt32;cur_coll_amt32=NULL;
 uint8 result=__wrap_SpawnPLM(header);cur_coll_amt32=previous;
 memcpy(transaction,g_ram,len);return result;
}
static uint32_t PortPointerResult(const uint16_t *pointer,uint16_t enemy) {
 if(!pointer)return 0;
 if(enemy>=0x800 || (enemy&0x3f))exit(4);
 uintptr_t p=(uintptr_t)pointer;unsigned bank=g_ram[0xf78+enemy+0x2e];
 if(p<0x10000){if(p<0x8000)exit(4);return (bank<<16)|p;}
 uintptr_t base=(uintptr_t)g_rom+((bank&0x7f)*0x8000);
 if(p<base || p>=base+0x8000)exit(4);
 return (bank<<16)|(0x8000+p-base);
}
static const uint16_t *PortPointerFromResult(uint32_t address) {
 if(!address)return NULL;
 if((address&0xffff)<0x8000)exit(4);
 size_t offset=(((address>>16)<<15)|(address&0x7fff))&0x3fffff;
 if(offset>=g_snes->cart->romSize)exit(4);
 return (const uint16_t*)(g_rom+offset);
}
static void PortCapture(unsigned routine,unsigned long sequence,const char *phase) {
 char name[120];snprintf(name,sizeof(name),"fixtures/port-%03u-%03lu-%s.wram",routine,sequence,phase);
 FILE *f=fopen(name,"wb");if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f))exit(2);
}
static unsigned long port_room_total[2],port_room_c[2],port_room_rust[2],port_room_setups[2];
''' + (generate_positioned_plm_host() if args.room_plm else '') + '''
static unsigned port_frame;
void __real_ppu_write(Ppu *,uint8_t,uint8_t);
void __wrap_ppu_write(Ppu *ppu,uint8_t adr,uint8_t val) {
 fprintf(port_effects,"ppu %u %d %u %u\\n",port_frame,ppu==g_snes->my_ppu,adr,val);
 __real_ppu_write(ppu,adr,val);
}
void __real_RtlApuWrite(uint32 adr,uint8 val);
void __wrap_RtlApuWrite(uint32 adr,uint8 val) {
 fprintf(port_effects,"apu %u %u %u\\n",port_frame,adr,val);
 __real_RtlApuWrite(adr,val);
}
static void PortSave(void *ctx,void *data,size_t size) {
 if(fwrite(data,1,size,ctx)!=size) exit(2);
}
bool __real_RtlRunFrame(int);
bool __wrap_RtlRunFrame(int pad) {
 bool result=__real_RtlRunFrame(pad); port_frame++;
 if(port_frame==1 || port_frame%1000==0) {
  char name[80];snprintf(name,sizeof(name),"frame-%06u.bin",port_frame);
  FILE *f=fopen(name,"wb");if(!f)exit(2);
  uint8 version=g_snes->runningWhichVersion;
  snes_saveload(g_snes,PortSave,f);g_snes->runningWhichVersion=version;
  ppu_saveload(g_snes->my_ppu,PortSave,f);ppu_saveload(g_snes->snes_ppu,PortSave,f);
  PortSave(f,g_sram,0x2000);PortSave(f,pixels,sizeof(pixels));
  if(fclose(f))exit(2);
 }
 return result;
}
''' + f'static unsigned long port_c_hits[{len(symbols)}],port_rust_hits[{len(symbols)}],port_outer_hits[{len(symbols)}];\n' + '\n'.join(wrappers) + '''
int main(int argc,char **argv) {
 if(argc!=6) return 2;
 port_mixed=!strcmp(argv[1],"mixed");
 port_effects=fopen("hardware.log","wb");if(!port_effects)return 2;
 port_calls=fopen("routine-calls.log","wb");if(!port_calls)return 2;
 char *original[]={argv[0],argv[2],argv[3],argv[4],argv[5],NULL};
 int result=retained_main(5,original);
 if(fclose(port_effects))return 2;
 if(fclose(port_calls))return 2;
 if(result) return result;
 for(unsigned i=0;i<sizeof(port_c_hits)/sizeof(*port_c_hits);i++) printf("port-hit %u %lu %lu\\n",i,port_c_hits[i],port_rust_hits[i]);
 printf("predecessor-hit %lu %lu %lu %lu\\n",port_decompress_c,port_decompress_rust,port_rng_c,port_rng_rust);
 for(unsigned i=0;i<2;i++)printf("room-plm-hit %u %lu %lu %lu %lu\\n",i,port_room_total[i],port_room_c[i],port_room_rust[i],port_room_setups[i]);
 printf("plm-callback-hit %lu %lu %lu %lu\\n",port_plm_callbacks,port_plm_c,port_plm_rust,port_plm_rust_setups);
 return 0;
}
''')
        units += [source / 'src' / name for name in ('spc_player.c', 'tracing.c', 'util.c')]
        units += sorted((source / 'src/snes').glob('*.c'))
        build = ['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source / 'src'), *map(str, units), str(host), str(run / 'libfamilies.a'), str(run / 'target/release/libdecomp.a'), str(run / 'librng.a'), '-Wl,--gc-sections', '-Wl,--wrap=Call', '-Wl,--wrap=cpu_runOpcode', '-Wl,--wrap=DecompressToMem', '-Wl,--wrap=ppu_write', '-Wl,--wrap=RtlApuWrite', '-Wl,--wrap=RtlRunFrame', '-Wl,--wrap=SpawnPLM', '-lm', '-ldl', '-lpthread', '-o', str(run / 'live')]
        execute(build, run, 'host-build')
        inventories, hits = {}, {}
        for mode in ('c', 'mixed'):
            for repeat in range(2):
                name = f'{mode}-{repeat}'
                directory = run / name
                (directory / 'fixtures').mkdir(parents=True)
                (directory / 'saves').mkdir()
                execute([str(run / 'live'), mode, str(rom), str(inputs), str(args.frames), str(directory / 'terminal.bin')], directory, 'replay')
                inventories[name] = {str(p.relative_to(directory)): digest(p) for p in directory.rglob('*') if p.is_file() and p.name not in ('replay.stdout', 'replay.stderr', 'calls.jsonl') and p.suffix != '.rom'}
                if args.dedup_anchor_verdict:
                    reclaimed = share_finished_outputs(directory, inventories[name], args.dedup_anchor_verdict)
                    verdict.setdefault('finished_output_dedup', {})[name] = reclaimed
                    if name != 'mixed-1' and shutil.disk_usage(run).free < (directory / 'hardware.log').stat().st_size + 512 * 1024 * 1024:
                        raise ValueError('insufficient space for another independent replay log')
                    verdict['dedup_anchor'] = {'path': str(args.dedup_anchor_verdict.resolve()),
                                              'sha256': stream_digest(args.dedup_anchor_verdict),
                                              'outputs': {key: inventories[name][key] for key in ('hardware.log', 'rom-view.bin')},
                                              'scope': 'historical immutable bytes only; no current-source admission'}
                rows = [line.split() for line in (directory / 'replay.stdout').read_text().splitlines() if line.startswith('port-hit ')]
                if len(rows) != len(symbols):
                    raise ValueError('missing per-routine execution counters')
                hits[name] = {symbols[int(row[1])]: {'c': int(row[2]), 'rust': int(row[3])} for row in rows}
                progress = [json.loads(line) for line in (directory / 'replay.stdout').read_text().splitlines() if line.startswith('{')]
                verdict.setdefault('progress', {})[name] = progress
                predecessor = [line.split()[1:] for line in (directory / 'replay.stdout').read_text().splitlines() if line.startswith('predecessor-hit ')]
                if len(predecessor) != 1:
                    raise ValueError('missing predecessor execution counters')
                dc, dr, rc, rr = map(int, predecessor[0])
                if (mode == 'mixed' and (dc or rc or not dr or not rr)) or (mode == 'c' and (dr or rr or not dc or not rc)):
                    raise ValueError('prior Rust replacements were not exclusively exercised')
                verdict.setdefault('predecessor_hits', {})[name] = {'decompressor_c': dc, 'decompressor_rust': dr, 'rng_c': rc, 'rng_rust': rr}
                callbacks = [line.split()[1:] for line in (directory / 'replay.stdout').read_text().splitlines() if line.startswith('plm-callback-hit ')]
                if len(callbacks) != 1 or len(callbacks[0]) != 4:
                    raise ValueError('missing PLM callback execution counter')
                total, c_calls, rust_calls, rust_setups = map(int, callbacks[0])
                if total != c_calls + rust_calls or rust_setups > rust_calls or (mode == 'c' and rust_calls):
                    raise ValueError('invalid PLM callback ownership counts')
                positioned=[line.split()[1:] for line in (directory/'replay.stdout').read_text().splitlines() if line.startswith('room-plm-hit ')]
                if len(positioned)!=2:raise ValueError('missing positioned PLM counters')
                counts={}
                for row in positioned:
                    kind,positioned_total,pc,pr,setups=map(int,row)
                    if kind not in (0,1) or kind in counts or positioned_total!=pc+pr or setups>pr or (mode=='c' and pr):raise ValueError('invalid positioned counters')
                    counts[kind]={'total':positioned_total,'c':pc,'rust':pr,'rust_setups':setups}
                verdict.setdefault('room_plm_hits',{})[name]=counts
                verdict.setdefault('plm_callback_hits', {})[name] = {'total': total, 'c': c_calls, 'rust': rust_calls, 'rust_setups': rust_setups}
                if any(row['c' if mode == 'mixed' else 'rust'] for row in hits[name].values()):
                    raise ValueError('replacement execution was not exclusive')
        if any(inventory != inventories['c-0'] for inventory in inventories.values()):
            mismatches = [key for key in inventories['c-0'] if any(inventory.get(key) != inventories['c-0'][key] for inventory in inventories.values())]
            raise ValueError(f'independent C/mixed state or effects diverged: {mismatches[:5]}')
        if any(progress != verdict['progress']['c-0'] for progress in verdict['progress'].values()):
            raise ValueError('gameplay progress or measured fallback diverged')
        if len({counts['total'] for counts in verdict['plm_callback_hits'].values()}) != 1:
            raise ValueError('PLM callback execution counts diverged')
        if verdict['plm_callback_hits']['mixed-0'] != verdict['plm_callback_hits']['mixed-1']:
            raise ValueError('native PLM ownership differs across independent replays')
        # Preserve successful comparison evidence even if an additional installed
        # subsystem was unexercised and therefore cannot receive admission.
        verdict["room_plm_installed"] = args.room_plm
        verdict.update(state_effects_match=True, hits=hits, artifacts=inventories['c-0'],
                       transformations=transformations, binary_sha256=digest(run / 'live'))
        dispatcher_symbols = {r['symbol'] for i in dispatcher_indexes for r in families[i][0]['routines']
                              if r.get('live_eligible', True)}
        verdict['dispatcher_symbols'] = sorted(dispatcher_symbols)
        verdict['dispatcher_hits'] = {name: {symbol: counts[symbol] for symbol in dispatcher_symbols}
                                      for name, counts in hits.items()}
        if dispatcher_symbols and not sum(hits['mixed-0'][symbol]['rust'] for symbol in dispatcher_symbols):
            raise ValueError('no Rust bank86 dispatcher leaf executed')
        if not sum(row['rust'] for row in hits['mixed-0'].values()):
            raise ValueError('no Rust gameplay family executed')
        if args.room_plm:
            if verdict['room_plm_hits']['mixed-0']!=verdict['room_plm_hits']['mixed-1']:raise ValueError('positioned ownership differs')
            if not sum(row['rust_setups'] for row in verdict['room_plm_hits']['mixed-0'].values()):raise ValueError('no native positioned setup executed')
        if args.plm_verdict and not args.room_plm and not verdict['plm_callback_hits']['mixed-0']['rust_setups']:
            raise ValueError('no native Rust PLM setup executed')
        verdict.update(status='mixed-C-Rust-integration', live_exercised=True, hits=hits,
                       transformations=transformations, artifacts=inventories['c-0'],
                       binary_sha256=digest(run / 'live'),
                       remaining_CPU_fallback=verdict['progress']['mixed-0'][-1]['fallback_calls'],
                       scope='four fresh processes; serialized periodic hardware/game state, full terminal WRAM/SRAM, ordered PPU/APU effects and captures; no original-ROM parity or same-process reset claim')
    except (OSError, ValueError, KeyError, TypeError, ImportError, subprocess.SubprocessError) as error:
        verdict['error'] = str(error)
    # Cargo intermediates are not part of the replay artifact inventory. The
    # linked live binary and serialized observations remain retained.
    shutil.rmtree(run / 'target', ignore_errors=True)
    if verdict.get('status') == 'mixed-C-Rust-integration':
        for archive in ('libfamilies.a', 'librng.a'):
            (run / archive).unlink(missing_ok=True)
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    print(json.dumps({key: verdict[key] for key in ('status', 'run', 'error') if key in verdict}))
    return 0 if verdict['live_exercised'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
