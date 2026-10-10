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
    parser.add_argument('--frames', type=int, default=9000)
    parser.add_argument('--accepted', type=Path, default=ROOT / 'out/decomp/review/offline/decomp-acceptance.json')
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
        if not families:
            raise ValueError('no family contracts')
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
        rust = [f'#[path={json.dumps(str(ROOT / "crates/port/src/lib.rs"))}] mod memory;', 'use memory::Wram;']
        wrappers, symbols, units = [], [], sorted((source / 'src').glob('sm_*.c'))
        bank_texts = {}
        for family_index, (manifest, module) in enumerate(families):
            rust.append(generate_family_abi(manifest, module, family_index))
            wrappers.append(f'extern int64_t port_family_{family_index}(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);')
            bank = source / 'src' / manifest['bank']
            text = bank_texts.get(bank, bank.read_text())
            for routine_index, routine in enumerate(manifest['routines']):
                symbol = routine['symbol']
                types = routine.get('arg_types', ['uint16'] * routine.get('args', 0))
                count = len(types)
                if symbol in symbols or not re.fullmatch('[A-Za-z_][A-Za-z_0-9]*', symbol):
                    raise ValueError('duplicate or invalid live symbol')
                index = len(symbols)
                symbols.append(symbol)
                signature = re.compile(r'(?m)^(\w+\s+)' + re.escape(symbol) + r'(\s*\([^\n]*\)\s*\{)')
                text, replacements = signature.subn(r'\1C_' + symbol + r'\2', text)
                if replacements != 1:
                    raise ValueError(f'definition rename count differs: {symbol}')
                c_type = {'void': 'void', 'uint8': 'uint8_t', 'uint16': 'uint16_t', 'uint32': 'uint32_t', 'int32': 'int32_t'}[routine['return']]
                declaration = ','.join(kind + '_t ' + name for kind, name in zip(types, ['a', 'b'])) or 'void'
                arguments = ','.join(['a', 'b'][:count])
                actual = ','.join(['a' if count > 0 else '0', 'b' if count > 1 else '0'])
                c_call = f'C_{symbol}({arguments})'
                reference = f'{c_call};' if c_type == 'void' else f'result={c_call};'
                returned = '' if c_type == 'void' else f'return ({c_type})result;'
                wrappers.append(f'''extern {c_type} C_{symbol}({declaration});
{c_type} {symbol}({declaration}) {{
 int64_t result=0;
 int outer=port_call_depth++==0;
 unsigned long capture=outer?++port_outer_hits[{index}]:0;
 if(outer && capture<=8)PortCapture({index},capture,"before");
 if(port_mixed) {{result=port_family_{family_index}(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,{routine_index},{actual}); if(result<0) {{fprintf(stderr,"Rust family error {symbol}\\n");exit(4);}} port_rust_hits[{index}]++;}}
 else {{{reference} port_c_hits[{index}]++;}}
 port_call_depth--;
 if(outer) {{
  if(capture<=8)PortCapture({index},capture,"after");
  fprintf(port_calls,"{index} %lu %lld %lld %lld\\n",capture,(long long)({actual.split(',')[0]}),(long long)({actual.split(',')[1]}),(long long)result);
 }}
 {returned}
}}''')
            bank_texts[bank] = text
        library_source = run / 'families.rs'
        library_source.write_text('\n'.join(rust))
        execute(['rustc', '--edition=2024', '-O', '--crate-type=staticlib', str(library_source), '-o', str(run / 'libfamilies.a')], run, 'rust-build')
        execute(['cargo', 'build', '-p', 'decomp', '--lib', '--release', '--locked', '--offline', '--target-dir', str(run / 'target')], ROOT, 'decompressor-build')
        verify_accepted(args.accepted)
        execute(['rustc', '--edition=2024', '-O', '--crate-type=staticlib', str(ROOT / 'tools/decomp/next_random.rs'), '-o', str(run / 'librng.a')], run, 'rng-build')
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
 return 0;
}
''')
        units += [source / 'src' / name for name in ('spc_player.c', 'tracing.c', 'util.c')]
        units += sorted((source / 'src/snes').glob('*.c'))
        build = ['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source / 'src'), *map(str, units), str(host), str(run / 'libfamilies.a'), str(run / 'target/release/libdecomp.a'), str(run / 'librng.a'), '-Wl,--gc-sections', '-Wl,--wrap=Call', '-Wl,--wrap=cpu_runOpcode', '-Wl,--wrap=DecompressToMem', '-Wl,--wrap=ppu_write', '-Wl,--wrap=RtlApuWrite', '-Wl,--wrap=RtlRunFrame', '-lm', '-ldl', '-lpthread', '-o', str(run / 'live')]
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
                if any(row['c' if mode == 'mixed' else 'rust'] for row in hits[name].values()):
                    raise ValueError('replacement execution was not exclusive')
        if any(inventory != inventories['c-0'] for inventory in inventories.values()):
            raise ValueError('independent C/mixed state or effects diverged')
        if not sum(row['rust'] for row in hits['mixed-0'].values()):
            raise ValueError('no Rust gameplay family executed')
        if any(progress != verdict['progress']['c-0'] for progress in verdict['progress'].values()):
            raise ValueError('gameplay progress or measured fallback diverged')
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
