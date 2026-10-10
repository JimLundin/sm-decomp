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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--family-verdict', type=Path, action='append', required=True)
    parser.add_argument('--frames', type=int, default=9000)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    verdict = {'status': 'rejected', 'live_exercised': False,
               'original_rom_parity': 'not-tested', 'commands': []}
    try:
        from port_family import digest, verify_source
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
            previous = json.loads(path.read_text())
            if previous['status'] != 'offline-verified':
                raise ValueError('family evidence is not offline verified')
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
        def execute(command, directory, label):
            verdict['commands'].append({'argv': command, 'cwd': str(directory)})
            with (directory / (label + '.stdout')).open('wb') as stdout, (directory / (label + '.stderr')).open('wb') as stderr:
                result = subprocess.run(command, cwd=directory, stdout=stdout, stderr=stderr, timeout=240)
            if result.returncode:
                raise ValueError(f'{label} failed with exit {result.returncode}')
        rust = [f'#[path={json.dumps(str(ROOT / "crates/port/src/lib.rs"))}] mod memory;', 'use memory::Wram;']
        wrappers, symbols, units = [], [], sorted((source / 'src').glob('sm_*.c'))
        bank_texts = {}
        for family_index, (manifest, module) in enumerate(families):
            rust.append(f'#[path={json.dumps(str(module))}] mod family{family_index};')
            rust.append(f'''#[unsafe(no_mangle)]
pub unsafe extern "C" fn port_family_{family_index}(ram:*mut u8,len:usize,id:u32,a:u16,b:u16)->i64 {{
 if ram.is_null() || len!=0x20000 || (ram as usize).checked_add(len).is_none() {{return -1;}}
 let names={json.dumps([r['symbol'] for r in manifest['routines']])};
 let Some(name)=names.get(id as usize) else {{return -1;}};
 let bytes=unsafe {{std::slice::from_raw_parts_mut(ram,len)}};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||family{family_index}::dispatch(&mut Wram::new(bytes).unwrap(),name,a,b)));
 match result {{Ok(Ok(value))=>i64::from(value),_=>-1}}
}}''')
            wrappers.append(f'extern int64_t port_family_{family_index}(uint8_t*,size_t,uint32_t,uint16_t,uint16_t);')
            bank = source / 'src' / manifest['bank']
            text = bank_texts.get(bank, bank.read_text())
            for routine_index, routine in enumerate(manifest['routines']):
                symbol, count = routine['symbol'], routine['args']
                if symbol in symbols or not re.fullmatch('[A-Za-z_][A-Za-z_0-9]*', symbol):
                    raise ValueError('duplicate or invalid live symbol')
                index = len(symbols)
                symbols.append(symbol)
                signature = re.compile(r'(?m)^(\w+\s+)' + re.escape(symbol) + r'(\s*\([^\n]*\)\s*\{)')
                text, replacements = signature.subn(r'\1C_' + symbol + r'\2', text)
                if replacements != 1:
                    raise ValueError(f'definition rename count differs: {symbol}')
                c_type = {'void': 'void', 'uint8': 'uint8_t', 'uint16': 'uint16_t', 'uint32': 'uint32_t'}[routine['return']]
                declaration = ','.join(['uint16_t a', 'uint16_t b'][:count]) or 'void'
                arguments = ','.join(['a', 'b'][:count])
                actual = ','.join(['a' if count > 0 else '0', 'b' if count > 1 else '0'])
                c_call = f'C_{symbol}({arguments})'
                reference = f'{c_call};' if c_type == 'void' else f'result={c_call};'
                returned = '' if c_type == 'void' else f'return ({c_type})result;'
                wrappers.append(f'''extern {c_type} C_{symbol}({declaration});
{c_type} {symbol}({declaration}) {{
 int64_t result=0;
 if(port_mixed) {{result=port_family_{family_index}(g_ram,sizeof(g_ram),{routine_index},{actual}); if(result<0) {{fprintf(stderr,"Rust family error {symbol}\\n");exit(4);}} port_rust_hits[{index}]++;}}
 else {{{reference} port_c_hits[{index}]++;}}
 {returned}
}}''')
            bank_texts[bank] = text
        library_source = run / 'families.rs'
        library_source.write_text('\n'.join(rust))
        execute(['rustc', '--edition=2024', '-O', '--crate-type=staticlib', str(library_source), '-o', str(run / 'libfamilies.a')], run, 'rust-build')
        transformations = {}
        for bank, text in bank_texts.items():
            transformed = run / bank.name
            transformed.write_text(text)
            units[units.index(bank)] = transformed
            transformations[bank.name] = {'original_sha256': digest(bank), 'transformed_sha256': digest(transformed)}
        host = run / 'host.c'
        host.write_text('''#define main retained_main
''' + f'#include {json.dumps(str(ROOT / "tools/decomp/headless.c"))}\n' + '''#undef main
#include "snes/ppu.h"
static FILE *port_effects;
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
static int port_mixed;
''' + f'static unsigned long port_c_hits[{len(symbols)}],port_rust_hits[{len(symbols)}];\n' + '\n'.join(wrappers) + '''
int main(int argc,char **argv) {
 if(argc!=6) return 2;
 port_mixed=!strcmp(argv[1],"mixed");
 port_effects=fopen("hardware.log","wb");if(!port_effects)return 2;
 char *original[]={argv[0],argv[2],argv[3],argv[4],argv[5],NULL};
 int result=retained_main(5,original);
 if(fclose(port_effects))return 2;
 if(result) return result;
 for(unsigned i=0;i<sizeof(port_c_hits)/sizeof(*port_c_hits);i++) printf("port-hit %u %lu %lu\\n",i,port_c_hits[i],port_rust_hits[i]);
 return 0;
}
''')
        units += [source / 'src' / name for name in ('spc_player.c', 'tracing.c', 'util.c')]
        units += sorted((source / 'src/snes').glob('*.c'))
        build = ['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source / 'src'), *map(str, units), str(host), str(run / 'libfamilies.a'), '-Wl,--gc-sections', '-Wl,--wrap=Call', '-Wl,--wrap=cpu_runOpcode', '-Wl,--wrap=DecompressToMem', '-Wl,--wrap=ppu_write', '-Wl,--wrap=RtlApuWrite', '-Wl,--wrap=RtlRunFrame', '-lm', '-ldl', '-lpthread', '-o', str(run / 'live')]
        execute(build, run, 'host-build')
        inventories, hits = {}, {}
        for mode in ('c', 'mixed'):
            for repeat in range(2):
                name = f'{mode}-{repeat}'
                directory = run / name
                (directory / 'fixtures').mkdir(parents=True)
                (directory / 'saves').mkdir()
                execute([str(run / 'live'), mode, str(rom), str(inputs), str(args.frames), str(directory / 'terminal.bin')], directory, 'replay')
                inventories[name] = {str(p.relative_to(directory)): digest(p) for p in directory.rglob('*') if p.is_file() and p.name not in ('replay.stdout', 'replay.stderr')}
                rows = [line.split() for line in (directory / 'replay.stdout').read_text().splitlines() if line.startswith('port-hit ')]
                if len(rows) != len(symbols):
                    raise ValueError('missing per-routine execution counters')
                hits[name] = {symbols[int(row[1])]: {'c': int(row[2]), 'rust': int(row[3])} for row in rows}
                progress = [json.loads(line) for line in (directory / 'replay.stdout').read_text().splitlines() if line.startswith('{')]
                verdict.setdefault('progress', {})[name] = progress
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
