#!/usr/bin/env python3
"""Exercise offline-verified Rust families in independent C/mixed replay."""
import argparse
import datetime
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def definition_pattern(symbol):
    """Match a definition, including const pointer returns, without call sites."""
    return re.compile(r'(?m)^((?:[A-Za-z_]\w*[ \t]+)+(?:\*[ \t]*)?)' + re.escape(symbol)
                      + r'(\s*\([^\n]*\)\s*\{)')


def generate_plm_abi():
    """Choose supported setup before mutation; errors never trigger a C retry."""
    return '''#[unsafe(no_mangle)]
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
}'''


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
    parser.add_argument('--family-verdict', type=Path, action='append', required=True)
    parser.add_argument('--collision-verdict', type=Path)
    parser.add_argument('--plm-verdict', type=Path)
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
        collision_index = None
        if args.collision_verdict:
            from port_collision import verify_collision_evidence
            evidence = verify_collision_evidence(args.collision_verdict)
            collision_manifest = json.loads((args.collision_verdict.parent / 'manifest.json').read_text())
            if collision_manifest.get('infrastructure_probe') or not evidence.get('gameplay_translation', True):
                raise ValueError('infrastructure callback probe cannot replace gameplay')
            collision_module = Path(collision_manifest['rust_module'])
            if not collision_module.is_absolute():
                collision_module = ROOT / collision_module
            collision_index = len(families)
            families.append((collision_manifest, collision_module))
        if not families:
            raise ValueError('no family contracts')
        if args.plm_verdict:
            from port_plm import verify_plm_evidence
            plm_evidence = verify_plm_evidence(args.plm_verdict)
            if plm_evidence['source'] != verdict['source']:
                raise ValueError('native PLM source identity differs')
            if Path(plm_evidence['implementation']['module']).resolve() != ROOT / 'crates/port/src/plm.rs':
                raise ValueError('native PLM evidence does not seal the installed module')
            verdict['plm_predecessor_sha256'] = digest(args.plm_verdict)
        from decomp_routine import verify_accepted
        verify_accepted(args.accepted)
        verdict['decompressor_predecessor_sha256'] = digest(args.accepted)
        def execute(command, directory, label):
            verdict['commands'].append({'argv': command, 'cwd': str(directory)})
            logs = run if directory == ROOT else directory
            with (logs / (label + '.stdout')).open('wb') as stdout, (logs / (label + '.stderr')).open('wb') as stderr:
                result = subprocess.run(command, cwd=directory, stdout=stdout, stderr=stderr, timeout=240)
            if result.returncode:
                raise ValueError(f'{label} failed with exit {result.returncode}')
        rust = [f'#[path={json.dumps(str(ROOT / "crates/port/src/lib.rs"))}] mod memory;', 'use memory::*;']
        if args.plm_verdict:
            rust.append(generate_plm_abi())
        imports = sorted({name for manifest, _ in families for name in manifest.get('rust_imports', [])})
        for name in imports:
            if not re.fullmatch('[A-Za-z_][A-Za-z_0-9]*', name):
                raise ValueError('invalid imported gameplay module')
            rust.append(f'use memory::{name};')
        wrappers, symbols, units = [], [], sorted((source / 'src').glob('sm_*.c'))
        bank_texts = {}
        for family_index, (manifest, module) in enumerate(families):
            if family_index == collision_index:
                from port_collision import generate_collision_abi
                rust.append(generate_collision_abi(manifest, module))
                abi_symbol = 'port_collision_call'
            else:
                rust.append(generate_family_abi(manifest, module, family_index))
                abi_symbol = f'port_family_{family_index}'
            wrappers.append(f'extern int64_t {abi_symbol}(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);')
            for routine_index, routine in enumerate(manifest['routines']):
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
        host.write_text('''#define main retained_main
#define __wrap_DecompressToMem retained_decompression_observer
''' + f'#include {json.dumps(str(ROOT / "tools/decomp/headless.c"))}\n' + '''#undef main
#undef __wrap_DecompressToMem
#include "snes/ppu.h"
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
static unsigned long port_plm_callbacks,port_plm_c,port_plm_rust;
''' + ('#define PORT_NATIVE_PLM\nextern int32_t port_native_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);\n' if args.plm_verdict else '') + '''
extern uint8 __real_SpawnPLM(uint16);
uint8 __wrap_SpawnPLM(uint16 header) {
 int32 before=cur_coll_amt32?*cur_coll_amt32:0;
 uint8 result;int32 displacement=before;int32 native=-2;
#ifdef PORT_NATIVE_PLM
 // Native acceptance covers synchronous collision context with a real amount.
 if(port_mixed && cur_coll_amt32)native=port_native_plm(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,header,&displacement);
#endif
 if(native==-2){result=__real_SpawnPLM(header);port_plm_c++;}
 else {if(native<0){fprintf(stderr,"native PLM error %u\\n",header);exit(4);}result=native;if(cur_coll_amt32)*cur_coll_amt32=displacement;port_plm_rust++;}
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
 printf("plm-callback-hit %lu %lu %lu\\n",port_plm_callbacks,port_plm_c,port_plm_rust);
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
                if len(callbacks) != 1 or len(callbacks[0]) != 3:
                    raise ValueError('missing PLM callback execution counter')
                total, c_calls, rust_calls = map(int, callbacks[0])
                if total != c_calls + rust_calls or (mode == 'c' and rust_calls):
                    raise ValueError('invalid PLM callback ownership counts')
                verdict.setdefault('plm_callback_hits', {})[name] = {'total': total, 'c': c_calls, 'rust': rust_calls}
                if any(row['c' if mode == 'mixed' else 'rust'] for row in hits[name].values()):
                    raise ValueError('replacement execution was not exclusive')
        if any(inventory != inventories['c-0'] for inventory in inventories.values()):
            mismatches = [key for key in inventories['c-0'] if any(inventory.get(key) != inventories['c-0'][key] for inventory in inventories.values())]
            raise ValueError(f'independent C/mixed state or effects diverged: {mismatches[:5]}')
        if not sum(row['rust'] for row in hits['mixed-0'].values()):
            raise ValueError('no Rust gameplay family executed')
        if any(progress != verdict['progress']['c-0'] for progress in verdict['progress'].values()):
            raise ValueError('gameplay progress or measured fallback diverged')
        if len({counts['total'] for counts in verdict['plm_callback_hits'].values()}) != 1:
            raise ValueError('PLM callback execution counts diverged')
        if verdict['plm_callback_hits']['mixed-0'] != verdict['plm_callback_hits']['mixed-1']:
            raise ValueError('native PLM ownership differs across independent replays')
        # Preserve successful comparison evidence even if an additional installed
        # subsystem was unexercised and therefore cannot receive admission.
        verdict.update(state_effects_match=True, hits=hits, artifacts=inventories['c-0'],
                       transformations=transformations, binary_sha256=digest(run / 'live'))
        if args.plm_verdict and not verdict['plm_callback_hits']['mixed-0']['rust']:
            raise ValueError('no native Rust PLM setup executed')
        verdict.update(status='mixed-C-Rust-integration', live_exercised=True, hits=hits,
                       transformations=transformations, artifacts=inventories['c-0'],
                       binary_sha256=digest(run / 'live'),
                       remaining_CPU_fallback=verdict['progress']['mixed-0'][-1]['fallback_calls'],
                       scope='four fresh processes; serialized periodic hardware/game state, full terminal WRAM/SRAM, ordered PPU/APU effects and captures; no original-ROM parity or same-process reset claim')
    except (OSError, ValueError, KeyError, TypeError, ImportError, subprocess.SubprocessError) as error:
        verdict['error'] = str(error)
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    print(json.dumps({key: verdict[key] for key in ('status', 'run', 'error') if key in verdict}))
    return 0 if verdict['live_exercised'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
