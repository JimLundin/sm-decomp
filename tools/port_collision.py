#!/usr/bin/env python3
"""Full-WRAM collision comparison with synchronous actual upstream PLM callbacks.

Remaining C gameplay callbacks are measured, never counted as Rust ownership.
"""
import argparse
import datetime
import json
from pathlib import Path
import shutil
import subprocess

from port_family import (ROOT, WRAM_SIZE, IDENTIFIER, digest, verify_source,
                         execute, load_asset, prepare_case)


def generate_collision_abi(manifest,module):
    """Reusable guarded synchronous bridge; caller imports Wram/root gameplay modules."""
    context = manifest.get('plm_amount_context', 'collision')
    if context not in ('collision', 'none'):
        raise ValueError('invalid PLM amount context')
    bridge = 'sm_rust_collision_spawn_plm' if context == 'collision' else 'sm_rust_collision_spawn_plm_without_amount'
    ffi_amount = ',amount:*mut i32' if context == 'collision' else ''
    method_amount = ',amount:&mut i32' if context == 'collision' else ''
    call_amount = ',amount' if context == 'collision' else ''
    return f'''#[path={json.dumps(str(module))}] mod family;
unsafe extern "C" {{fn {bridge}(ram:*mut u8,len:usize,header:u16{ffi_amount})->i32;}}
struct CCallback;
impl family::Host for CCallback {{
 fn spawn_plm(&mut self,ram:&mut Wram<'_>,header:u16{method_amount})->Result<u8,&'static str>{{
  // Isolated owned buffer: C never aliases the live Wram reference.
  let mut transaction:Vec<u8>=(0..WRAM_SIZE).map(|i|ram.byte(i)).collect();
  let result=unsafe{{{bridge}(transaction.as_mut_ptr(),transaction.len(),header{call_amount})}};
  if !(0..=255).contains(&result){{return Err("C collision callback failed");}}
  for (i,value) in transaction.into_iter().enumerate(){{ram.set_byte(i,value);}}
  Ok(result as u8)
 }}
}}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn port_collision_call(ram:*mut u8,len:usize,rom:*const u8,rom_len:usize,id:u32,a:i32,b:i32)->i64{{
 if ram.is_null()||len!=WRAM_SIZE||rom.is_null()||rom_len==0||rom_len>0x400000{{return -1;}}
 let Some(ram_end)=(ram as usize).checked_add(len) else{{return -1;}};
 let Some(rom_end)=(rom as usize).checked_add(rom_len) else{{return -1;}};
 if (ram as usize)<rom_end&&(rom as usize)<ram_end{{return -1;}}
 let names={json.dumps([r['symbol'] for r in manifest['routines']])};
 let Some(name)=names.get(id as usize) else{{return -1;}};
 // Copy canonical C memory before acquiring Rust state; callbacks only see copies.
 let mut state=unsafe{{std::slice::from_raw_parts(ram,len)}}.to_vec();
 let rom=unsafe{{std::slice::from_raw_parts(rom,rom_len)}};
 let value=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||family::dispatch_host(&mut Wram::new(&mut state).unwrap(),rom,&mut CCallback,name,a,b)));
 match value{{Ok(Ok(value))=>{{unsafe{{std::ptr::copy_nonoverlapping(state.as_ptr(),ram,len)}};i64::from(value)}},_=>-1}}
}}
'''


def build(manifest, source, run):
    routines = manifest['routines']
    probe = manifest.get('infrastructure_probe', False)
    branches = []
    for routine in routines:
        symbol = routine['symbol']
        if not IDENTIFIER.fullmatch(symbol):
            raise ValueError('invalid routine symbol')
        kinds = routine.get('arg_types', [])
        if len(kinds) > 2 or any(k not in ('int32', 'uint8', 'uint16', 'uint32') for k in kinds):
            raise ValueError('unsupported collision argument signature')
        if routine['return'] not in ('int32', 'uint32', 'uint8', 'uint16', 'void'):
            raise ValueError('unsupported collision return signature')
        args = ','.join('('+kind+')'+name for kind,name in zip(kinds,['a','b']))
        call = f'{symbol}({args})'
        statement = f'{call};result=0;' if routine['return']=='void' else f'result=(uint32_t){call};'
        if probe:
            if symbol != 'CollisionCallbackProbe':
                raise ValueError('infrastructure probe cannot claim actual routine')
            statement = 'int32_t amount=a;cur_coll_amt32=&amount;__wrap_SpawnPLM((uint16_t)b);cur_coll_amt32=NULL;result=(uint32_t)amount;'
        branches.append(f'if(!strcmp(argv[2],"{symbol}")){{{statement}}}')
    module = Path(manifest['rust_module'])
    if not module.is_absolute(): module = ROOT / module
    wrapper = run / 'wrapper.rs'
    wrapper.write_text(f'#[path={json.dumps(str(ROOT/"crates/port/src/lib.rs"))}] mod memory;\nuse memory::*;\n'+generate_collision_abi(manifest,module))
    host = run / 'oracle.c'
    host.write_text('''#define main retained_headless_main
''' + f'#include {json.dumps(str(ROOT/"tools/decomp/headless.c"))}\n' + '''#undef main
#include "funcs.h"
#include <inttypes.h>
extern int32 *cur_coll_amt32;
extern uint8 __real_SpawnPLM(uint16);
extern int64_t port_collision_call(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);
static FILE *callback_log;
static const char *callback_directory;
static unsigned long callbacks;
static void CallbackSnapshot(const char *phase) {
 char path[4096];snprintf(path,sizeof(path),"%s/callback-%04lu-%s.wram",callback_directory,callbacks,phase);
 FILE *f=fopen(path,"wb");if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))exit(2);
}
uint8 __wrap_SpawnPLM(uint16 header) {
 callbacks++;int32 before=cur_coll_amt32?*cur_coll_amt32:0;
 CallbackSnapshot("before");
 uint8 value=__real_SpawnPLM(header);
 CallbackSnapshot("after");
 fprintf(callback_log,"%u %" PRId32 " %" PRId32 " %u\\n",header,before,cur_coll_amt32?*cur_coll_amt32:0,value);
 return value;
}
int32_t sm_rust_collision_spawn_plm(uint8_t *ram,size_t len,uint16_t header,int32_t *amount) {
 if(!ram||len!=sizeof(g_ram)||!amount||ram==g_ram)return -1;
 memcpy(g_ram,ram,len);int32 *previous=cur_coll_amt32;cur_coll_amt32=amount;
 uint8 result=__wrap_SpawnPLM(header);cur_coll_amt32=previous;
 memcpy(ram,g_ram,len);return result;
}
int32_t sm_rust_collision_spawn_plm_without_amount(uint8_t *ram,size_t len,uint16_t header) {
 if(!ram||len!=sizeof(g_ram)||ram==g_ram)return -1;
 memcpy(g_ram,ram,len);int32 *previous=cur_coll_amt32;cur_coll_amt32=NULL;
 uint8 result=__wrap_SpawnPLM(header);cur_coll_amt32=previous;
 memcpy(ram,g_ram,len);return result;
}
int main(int argc,char **argv){
 if(argc!=10)return 2;
 if(!SnesInit(argv[8]))return 2;
 g_spc_player=SpcPlayer_Create();SpcPlayer_Initialize(g_spc_player);
 int32_t a=(int32_t)strtoll(argv[3],NULL,10),b=(int32_t)strtoll(argv[4],NULL,10);uint32_t result=0;
 FILE *f=fopen(argv[5],"rb");if(!f||fread(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fgetc(f)!=EOF||fclose(f))return 2;
 callback_directory=argv[9];char log_path[4096];snprintf(log_path,sizeof(log_path),"%s/callbacks.txt",callback_directory);
 callback_log=fopen(log_path,"w");if(!callback_log)return 2;
 if(!strcmp(argv[1],"rust")){
 int64_t value=port_collision_call(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,(uint32_t)strtoul(argv[7],NULL,10),a,b);
 if(value<0)return 4;result=(uint32_t)value;
 }else{
''' + 'else '.join(branches) + '''else return 2;
 }
 if(fclose(callback_log))return 2;
 f=fopen(argv[6],"wb");if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))return 2;
 printf("{\\"return\\":%u,\\"c_gameplay_callbacks\\":%lu,\\"cpu_fallback_calls\\":%lu,\\"cpu_fallback_instructions\\":%lu}\\n",result,callbacks,fallback_calls,fallback_instructions);return 0;
}
''')
    execute(['rustc','--edition=2024','-O','--crate-type=staticlib',str(wrapper),'-o',str(run/'libcollision.a')],run,'rust-build')
    units = sorted((source/'src').glob('sm_*.c'))
    units += [source/'src'/name for name in ('spc_player.c','tracing.c','util.c')]
    units += sorted((source/'src/snes').glob('*.c'))
    execute(['cc','-O2','-fno-strict-aliasing','-ffunction-sections','-fdata-sections','-iquote',str(source/'src'),*map(str,units),str(host),str(run/'libcollision.a'),'-Wl,--gc-sections','-Wl,--wrap=SpawnPLM','-Wl,--wrap=Call','-Wl,--wrap=cpu_runOpcode','-Wl,--wrap=DecompressToMem','-lm','-ldl','-lpthread','-o',str(run/'collision')],run,'c-build')
    dependencies = {str(path):digest(path) for path in sorted((ROOT/'crates/port/src').glob('*.rs'))}
    dependencies[str(module)] = digest(module)
    for i,path in enumerate(dependencies):shutil.copyfile(path,run/f'dependency-{i:03d}.rs')
    return {'rust_dependencies':dependencies,'module':str(module),'module_sha256':digest(module),
            'harness_sha256':digest(__file__),'binary_sha256':digest(run/'collision')}


def verify(manifest_path,source,output):
    run=output.resolve()/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    verdict={'status':'rejected','run':str(run),'live_exercised':False,'original_rom_parity':'not-tested','cases':[]}
    try:
        manifest=json.loads(manifest_path.read_text());shutil.copyfile(manifest_path,run/'manifest.json')
        verdict['family']=manifest['family'];verdict['gameplay_translation']=not manifest.get('infrastructure_probe',False)
        verdict['source']=verify_source(source);shutil.copyfile(source/'LICENSE.txt',run/'LICENSE.txt')
        rom=manifest['rom'];data=load_asset(rom['file'],rom['sha256'])
        if not 0<len(data)<=0x400000:raise ValueError('invalid ROM extent')
        (run/'rom.bin').write_bytes(data)
        verdict['implementation']=build(manifest,source,run)
        seen=set()
        for index,routine in enumerate(manifest['routines']):
            if not routine['cases']:raise ValueError('empty routine coverage')
            for case in routine['cases']:
                if case['name'] in seen:raise ValueError('duplicate case name')
                seen.add(case['name']);directory=run/case['name']
                prepare_case(case,directory,routine.get('arg_types',[]))
                results={}
                for subject in ('rust','c'):
                    callbacks=directory/subject;callbacks.mkdir()
                    stdout=execute([str(run/'collision'),subject,routine['symbol'],str(case.get('a',0)),str(case.get('b',0)),str(directory/'before.wram'),str(directory/(subject+'-after.wram')),str(index),str(run/'rom.bin'),str(callbacks)],directory,subject)
                    results[subject]=json.loads(stdout.splitlines()[-1])
                record={'routine':routine['symbol'],'case':case['name'],'results':results,'callbacks':results['rust']['c_gameplay_callbacks']}
                verdict['cases'].append(record)
                if results['c']!=results['rust']:raise ValueError('return/callback/fallback metrics mismatch')
                if results['rust']['cpu_fallback_calls'] or results['rust']['cpu_fallback_instructions']:raise ValueError('CPU fallback during offline collision comparison')
                if results['rust']['return']!=case.get('expected_return',results['rust']['return']):raise ValueError('independent expected return mismatch')
                if record['callbacks']<case.get('minimum_callbacks',0):raise ValueError('callback coverage missing')
                before,c,rust=[(directory/name).read_bytes() for name in ('before.wram','c-after.wram','rust-after.wram')]
                if any(len(s)!=WRAM_SIZE for s in (before,c,rust)):raise ValueError('full WRAM extent missing')
                if c!=rust:
                    offset=next(i for i in range(WRAM_SIZE) if c[i]!=rust[i]);record['first_mismatch']={'offset':offset,'c':c[offset],'rust':rust[offset]}
                    raise ValueError(f'WRAM mismatch at {offset:#x}')
                if 'expected_after_file' in case and c!=(directory/'expected-after.wram').read_bytes():raise ValueError('captured full WRAM mismatch')
                c_files={p.name:p.read_bytes() for p in (directory/'c').iterdir()}
                rust_files={p.name:p.read_bytes() for p in (directory/'rust').iterdir()}
                if c_files!=rust_files:raise ValueError('ordered actual C callback state/argument sequence mismatch')
                if len(c_files)!=1+2*record['callbacks'] or len(c_files['callbacks.txt'].splitlines())!=record['callbacks']:
                    raise ValueError('callback record extent missing')
        if not seen:raise ValueError('empty collision family')
        verdict.update(status='offline-verified',verified_cases=len(seen),verified_routines=len(manifest['routines']),scope='full WRAM, return, ordered callback arguments and complete callback state; declared valid inputs')
    except (OSError,ValueError,KeyError,TypeError,subprocess.SubprocessError) as error:verdict['error']=str(error)
    verdict['artifacts']={str(p.relative_to(run)):digest(p) for p in sorted(run.rglob('*')) if p.is_file() and p.name!='verdict.json'}
    (run/'verdict.json').write_text(json.dumps(verdict,indent=2)+'\n');return verdict



def verify_collision_evidence(verdict_path):
    """Recheck retained comparisons and current dependencies before live admission."""
    verdict_path=Path(verdict_path);run=verdict_path.parent
    verdict=json.loads(verdict_path.read_text())
    if verdict.get('status')!='offline-verified' or not verdict.get('cases'):
        raise ValueError('collision evidence is not verified')
    artifacts=verdict.get('artifacts',{})
    for name,expected in artifacts.items():
        path=run/name
        if run.resolve() not in path.resolve().parents or digest(path)!=expected:
            raise ValueError(f'collision artifact changed: {name}')
    if not {'manifest.json','collision','oracle.c','wrapper.rs','libcollision.a','rom.bin'}<=artifacts.keys():
        raise ValueError('collision evidence artifacts incomplete')
    implementation=verdict['implementation']
    if digest(__file__)!=implementation['harness_sha256']:
        raise ValueError('collision harness changed')
    for path,expected in implementation['rust_dependencies'].items():
        if digest(path)!=expected:raise ValueError(f'collision dependency changed: {path}')
    manifest=json.loads((run/'manifest.json').read_text())
    pin=next(p for p in json.loads((ROOT/'tools/dependencies.json').read_text()) if p['name']=='snesrev-sm')
    if verdict['source']['revision']!=pin['revision'] or verdict['source']['archive_sha256']!=pin['sha256']:
        raise ValueError('collision pinned source identity changed')
    expected={(r['symbol'],c['name']):c for r in manifest['routines'] for c in r['cases']}
    if len(expected)!=sum(len(r['cases']) for r in manifest['routines']):raise ValueError('duplicate collision cases')
    observed=set()
    for record in verdict['cases']:
        key=(record['routine'],record['case'])
        if key not in expected or key in observed:raise ValueError('collision coverage changed')
        observed.add(key);directory=run/record['case']
        states=[(directory/name).read_bytes() for name in ('before.wram','c-after.wram','rust-after.wram')]
        if any(len(state)!=WRAM_SIZE for state in states) or states[1]!=states[2]:raise ValueError('actual collision WRAM mismatch')
        results={subject:json.loads((directory/(subject+'.stdout')).read_bytes().splitlines()[-1]) for subject in ('c','rust')}
        if results!=record['results'] or results['c']!=results['rust']:raise ValueError('actual collision metrics mismatch')
        if results['rust']['cpu_fallback_calls'] or results['rust']['cpu_fallback_instructions']:raise ValueError('collision CPU fallback')
        callbacks={subject:{p.name:p.read_bytes() for p in (directory/subject).iterdir()} for subject in ('c','rust')}
        if callbacks['c']!=callbacks['rust']:raise ValueError('actual callback sequence mismatch')
        for subject in ('c','rust'):
            for name in callbacks[subject]:
                if str((directory/subject/name).relative_to(run)) not in artifacts:
                    raise ValueError('callback artifact inventory incomplete')
        count=results['rust']['c_gameplay_callbacks']
        if count!=record['callbacks'] or count<expected[key].get('minimum_callbacks',0):raise ValueError('callback coverage changed')
        if 'expected_return' in expected[key] and results['rust']['return']!=expected[key]['expected_return']:raise ValueError('expected collision return changed')
    if observed!=expected.keys() or verdict['verified_cases']!=len(observed) or verdict['verified_routines']!=len(manifest['routines']):
        raise ValueError('collision coverage incomplete')
    return verdict


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,required=True)
    parser.add_argument('--source',type=Path,default=ROOT/'deps/snesrev-sm')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();verdict=verify(args.manifest,args.source.resolve(),args.output)
    print(json.dumps({k:verdict[k] for k in ('status','run','error') if k in verdict}))
    return 0 if verdict['status']=='offline-verified' else 1

if __name__=='__main__':raise SystemExit(main())
