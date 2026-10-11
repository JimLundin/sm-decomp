#!/usr/bin/env python3
"""Passive actual-C enemy instruction diagnostic; never Rust admission."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = next(p for p in Path(__file__).resolve().parents if (p/'tools/port_family.py').is_file())
sys.path.insert(0, str(ROOT / 'tools'))
import port_family

SUPPORT = r'''
#include <stdio.h>
extern void InventoryProjectileHostSave(uint16*);
static unsigned long enemy_frames[256],enemy_frame_first[256],enemy_frame_last[256];
static unsigned long enemy_opcodes[65536],enemy_opcode_first[65536],enemy_opcode_last[65536];
static unsigned frame_captures[3][65536],opcode_captures[3][65536];
static unsigned long enemy_inventory_sequence;
static unsigned enemy_instruction_depth;
static FILE *enemy_inventory_events;
static unsigned EnemyInventoryWindow(void){return snes_frame_counter<36000?0:snes_frame_counter<50000?1:2;}
static uint32 EnemyInventoryPointer(const uint16 *p,uint8 bank) {
 if(!p)return 0;
 uintptr_t value=(uintptr_t)p;
 if(value<0x10000)return ((uint32)bank<<16)|(uint16)value;
 uintptr_t base=(uintptr_t)RomBankBase(bank);
 if(value<base+0x8000 || value>=base+0x10000){fprintf(stderr,"inventory pointer extent\n");exit(2);}
 return ((uint32)bank<<16)|(uint16)(value-base);
}
static void EnemyInventoryCapture(const char *kind,uint16 pc,unsigned window,unsigned sequence,const char *phase) {
 char name[160];uint16 words[8];uint8 bytes[16];InventoryProjectileHostSave(words);
 snprintf(name,sizeof(name),"fixtures/enemy-%s-%04x-window%u-%02u-%s.wram",kind,pc,window,sequence,phase);
 FILE *f=fopen(name,"wb");if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f))exit(2);
 for(unsigned i=0;i<8;i++){bytes[2*i]=words[i];bytes[2*i+1]=words[i]>>8;}
 snprintf(name,sizeof(name),"fixtures/enemy-%s-%04x-window%u-%02u-%s.host",kind,pc,window,sequence,phase);
 f=fopen(name,"wb");if(!f || fwrite(bytes,1,16,f)!=16 || fclose(f))exit(2);
}
static void EnemyInventoryEvent(const char *kind,const char *phase,uint16 slot,uint32 opcode,uint32 argument,uint32 returned,unsigned capture) {
 uint16 words[8];InventoryProjectileHostSave(words);EnemyData *E=gEnemyData(slot);
 if(!enemy_inventory_events){enemy_inventory_events=fopen("enemy-events.jsonl","w");if(!enemy_inventory_events)exit(2);}
 fprintf(enemy_inventory_events,"{\"sequence\":%lu,\"frame\":%u,\"kind\":\"%s\",\"phase\":\"%s\",\"depth\":%u,\"slot\":%u,\"header\":%u,\"bank\":%u,\"pc\":%u,\"timer\":%u,\"ai_bits\":%u,\"extra\":%u,\"opcode\":%u,\"argument\":%u,\"returned\":%u,\"capture\":%u,\"orientation\":%u,\"f_source\":%u,\"host\":[",
 ++enemy_inventory_sequence,snes_frame_counter,kind,phase,enemy_instruction_depth,slot,E->enemy_ptr,E->bank,E->current_instruction,E->instruction_timer,E->ai_handler_bits,E->extra_properties,opcode,argument,returned,capture,GET_WORD(g_ram+0x7820),eproj_unk1995);
 for(unsigned i=0;i<8;i++)fprintf(enemy_inventory_events,"%s%u",i?",":"",words[i]);
 fprintf(enemy_inventory_events,"]}\n");
}
void CInventory_ProcessEnemyInstructions(void);
const uint16 *CInventory_CallEnemyInstr(uint32,uint16,const uint16*);
void ProcessEnemyInstructions(void) {
 uint16 slot=cur_enemy_index;EnemyData *E=gEnemyData(slot);uint8 bank=E->bank;uint16 pc=E->current_instruction;
 if(!enemy_frames[bank])enemy_frame_first[bank]=snes_frame_counter;enemy_frames[bank]++;enemy_frame_last[bank]=snes_frame_counter;
 unsigned window=EnemyInventoryWindow();unsigned capture=bank==0xa6 && frame_captures[window][pc]<2?++frame_captures[window][pc]:0;
 if(bank==0xa6)EnemyInventoryEvent("frame","before",slot,0,0,0,capture);
 if(capture)EnemyInventoryCapture("frame",pc,window,capture,"before");
 enemy_instruction_depth++;CInventory_ProcessEnemyInstructions();enemy_instruction_depth--;
 if(capture)EnemyInventoryCapture("frame",pc,window,capture,"after");
 if(bank==0xa6)EnemyInventoryEvent("frame","after",slot,0,0,0,capture);
}
const uint16 *CallEnemyInstr(uint32 ea,uint16 slot,const uint16 *argument) {
 uint8 bank=ea>>16;uint16 opcode=ea;unsigned window=EnemyInventoryWindow(),capture=0;
 if(bank==0xa6){
  if(!enemy_opcodes[opcode])enemy_opcode_first[opcode]=snes_frame_counter;enemy_opcodes[opcode]++;enemy_opcode_last[opcode]=snes_frame_counter;
  capture=opcode_captures[window][opcode]<2?++opcode_captures[window][opcode]:0;
  EnemyInventoryEvent("opcode","before",slot,ea,EnemyInventoryPointer(argument,bank),0,capture);
  if(capture)EnemyInventoryCapture("opcode",opcode,window,capture,"before");
 }
 const uint16 *result=CInventory_CallEnemyInstr(ea,slot,argument);
 if(bank==0xa6){
  if(capture)EnemyInventoryCapture("opcode",opcode,window,capture,"after");
  EnemyInventoryEvent("opcode","after",slot,ea,EnemyInventoryPointer(argument,bank),EnemyInventoryPointer(result,bank),capture);
 }
 return result;
}
void EnemyInventorySave(void) {
 FILE *f=fopen("enemy-instructions.tsv","w");if(!f)exit(2);
 fprintf(f,"kind\taddress\thits\tfirst_frame\tlast_frame\n");
 for(unsigned i=0;i<256;i++)if(enemy_frames[i])fprintf(f,"frame-bank\t%02x\t%lu\t%lu\t%lu\n",i,enemy_frames[i],enemy_frame_first[i],enemy_frame_last[i]);
 for(unsigned i=0;i<65536;i++)if(enemy_opcodes[i])fprintf(f,"opcode\ta6%04x\t%lu\t%lu\t%lu\n",i,enemy_opcodes[i],enemy_opcode_first[i],enemy_opcode_last[i]);
 if(fclose(f) || (enemy_inventory_events && fclose(enemy_inventory_events)))exit(2);
}
'''

HOST_SAVE = r'''
void InventoryProjectileHostSave(uint16 *v) {
 v[0]=eproj_spawn_pt.x;v[1]=eproj_spawn_pt.y;v[2]=eproj_spawn_rect.x;v[3]=eproj_spawn_rect.y;
 v[4]=eproj_spawn_rect.w;v[5]=eproj_spawn_rect.h;v[6]=eproj_spawn_r22;v[7]=eproj_spawn_varE24;
}
'''


def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
    return h.hexdigest()


def transform_a0(original):
    # Rename definitions only; native call sites retain the observed names.
    for anchor in ('void ProcessEnemyInstructions(void) {',
                   'const uint16 *CallEnemyInstr(uint32 ea, uint16 k, const uint16 *j) {'):
        if original.count(anchor)!=1:raise ValueError('native inventory anchor differs')
        original=original.replace(anchor,anchor.replace('ProcessEnemyInstructions','CInventory_ProcessEnemyInstructions').replace('CallEnemyInstr','CInventory_CallEnemyInstr'))
    anchor='#include "enemy_types.h"\n'
    if original.count(anchor)!=1:raise ValueError('enemy include differs')
    return original.replace(anchor,anchor+SUPPORT)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames',type=int,default=129712)
    parser.add_argument('--output',type=Path)
    parser.add_argument('--authenticate',type=Path)
    args=parser.parse_args()
    if args.authenticate:
        verdict=json.loads(args.authenticate.read_text());run=args.authenticate.resolve().parent
        for relative,expected in verdict['artifacts'].items():
            path=run/relative
            if path.is_symlink() or run not in path.resolve().parents or digest(path)!=expected:
                raise ValueError('diagnostic artifact changed: '+relative)
        print(json.dumps({'status':'authenticated','verdict_sha256':digest(args.authenticate),
                          'artifacts':len(verdict['artifacts']),'scope':'retained diagnostic bytes only'}))
        return 0
    if not args.output:parser.error('--output required for capture')
    if not 1<=args.frames<=129712:parser.error('movie extent required')
    run=args.output.resolve()/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ');run.mkdir(parents=True);(run/'fixtures').mkdir()
    source=ROOT/'deps/snesrev-sm/src';identity=port_family.verify_source(source.parent)
    rom=ROOT/'rom/sm.sfc';inputs=ROOT/'movies/super-metroid-any-3653.inputs'
    from decomp_baseline import ROM_SHA256
    if digest(rom)!=ROM_SHA256:raise ValueError('ROM identity differs')
    originals=run/'original-source';shutil.copytree(source,originals)
    shutil.copyfile(Path(__file__),run/'tool-used.py')
    (run/'sm_a0.c').write_text(transform_a0((originals/'sm_a0.c').read_text()))
    (run/'sm_86.c').write_text((originals/'sm_86.c').read_text()+HOST_SAVE)
    host=(ROOT/'tools/decomp/headless.c').read_text()
    start=host.index('static void SaveRoutineState');end=host.index('// Linker wrapping',start);host=host[:start]+host[end:]
    start=host.index('void __wrap_DecompressToMem');end=host.index('// The host has no audio thread',start)
    host=host[:start]+'void __wrap_DecompressToMem(uint32 src,uint8 *dst){decompression_calls++;__real_DecompressToMem(src,dst);}\nextern void EnemyInventorySave(void);\n'+host[end:]
    if host.count('  fclose(inputs);\n  return 0;')!=1:raise ValueError('host completion differs')
    host=host.replace('  fclose(inputs);\n  return 0;','  fclose(inputs);\n  EnemyInventorySave();\n  return 0;')
    (run/'headless.c').write_text(host);shutil.copyfile(ROOT/'tools/decomp/headless_frame.h',run/'headless_frame.h')
    units=[run/p.name if p.name in ('sm_a0.c','sm_86.c') else p for p in sorted(originals.glob('sm_*.c'))]
    units += [originals/n for n in ('spc_player.c','tracing.c','util.c')]+sorted((originals/'snes').glob('*.c'))
    command=['cc','-O2','-fno-strict-aliasing','-ffunction-sections','-fdata-sections','-iquote',str(originals),*map(str,units),str(run/'headless.c'),'-Wl,--gc-sections','-Wl,--wrap=Call','-Wl,--wrap=cpu_runOpcode','-Wl,--wrap=DecompressToMem','-lm','-o',str(run/'inventory')]
    status='diagnostic-failed';error=None
    try:
        port_family.execute(command,run,'build')
        replay=[str(run/'inventory'),str(rom),str(inputs),str(args.frames),str(run/'terminal.bin')]
        with (run/'commands.jsonl').open('a') as f:f.write(json.dumps(replay)+'\n')
        with (run/'replay.stdout').open('wb') as out,(run/'replay.stderr').open('wb') as err:
            subprocess.run(replay,cwd=run,stdout=out,stderr=err,timeout=max(300,args.frames/40),check=True)
        status='diagnostic-complete'
    except (subprocess.SubprocessError,OSError,ValueError) as exception:error=str(exception)
    # Seal failed/interrupted observations too; no Rust admission claim.
    verdict={'status':status,'error':error,'frames':args.frames,'source':identity,'live_exercised':False,
             'scope':'passive actual-C interpreter/opcode observations; no Rust or original-ROM parity claim',
             'rom_sha256':digest(rom),'inputs_sha256':digest(inputs),'tool_sha256':digest(Path(__file__)),
             'capture_windows':'prefix<36000; first-combat36000..49999; later-combat>=50000',
             'artifacts':{str(p.relative_to(run)):digest(p) for p in sorted(run.rglob('*')) if p.is_file()}}
    (run/'verdict.json').write_text(json.dumps(verdict,indent=2)+'\n')
    print(json.dumps({'status':status,'run':str(run),'error':error}))
    return 0 if status=='diagnostic-complete' else 1


if __name__=='__main__':raise SystemExit(main())
