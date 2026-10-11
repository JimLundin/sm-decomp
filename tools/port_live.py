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
    'EprojPreInstr_CeresElevatorPlatform': 'void',
    'EprojPreInstr_CeresElevatorPad': 'void',
    'EprojInstr_Delete': 'rom_ptr',
    'EprojInstr_SetPreInstr_': 'rom_ptr',
    'EprojInstr_ClearPreInstr': 'rom_ptr',
    'EprojInstr_Goto': 'rom_ptr',
}
DISPATCHER_SYNTHETIC = {'EprojInstruction', 'EprojPreinstruction'}
PROJECTILE_ENTRIES = {
    'SpawnEprojWithRoomGfx': ['uint16', 'uint16'],
    'EprojRunAll': [],
    'SpawnEprojWithGfx': ['uint16', 'uint32'],
}
OBSERVED_PROJECTILE_IDS = (0x9642, 0x9650, 0x965e, 0x966c, 0x967a, 0x9688, 0x9696, 0xa387, 0xa395)


def load_projectile_contract(path, source_identity):
    """Keep host-aware allocator ownership separate from ordinary family ABI."""
    from port_plm_instructions import verify_plm_instruction_evidence
    evidence = verify_plm_instruction_evidence(path)
    manifest = json.loads((path.parent / 'manifest.json').read_text())
    module = Path(evidence['implementation']['module']).resolve()
    if evidence['source'] != source_identity:
        raise ValueError('projectile source identity differs')
    if (module != ROOT / 'crates/port/src/observed_projectile_ownership.rs'
            or manifest.get('bank') != 'sm_86.c' or not manifest.get('projectile_host')
            or 'rom' not in manifest):
        raise ValueError('projectile evidence requires installed host-aware ownership module and bank86 ROM')
    entries = set()
    for routine in manifest['routines']:
        symbol = routine['symbol']
        if symbol == 'EprojInstruction':
            if routine.get('live_eligible', True):
                raise ValueError('projectile synthetic instruction must be offline-only')
            continue
        if routine.get('live_eligible', True):
            if (symbol not in PROJECTILE_ENTRIES or symbol in entries
                    or routine.get('arg_types', ['uint16'] * routine.get('args', 0)) != PROJECTILE_ENTRIES[symbol]
                    or routine['return'] != 'void'):
                raise ValueError(f'projectile native signature differs: {symbol}')
            entries.add(symbol)
            if symbol == 'SpawnEprojWithGfx' and routine.get('packed_args') != 'eproj-with-gfx-v1':
                raise ValueError('projectile gfx native contract requires packed enemy/definition marker')
            if symbol != 'SpawnEprojWithGfx' and 'packed_args' in routine:
                raise ValueError('projectile packed marker only applies to native WithGfx')
    if entries != PROJECTILE_ENTRIES.keys():
        raise ValueError('projectile allocator and scheduler contracts both required')
    return manifest, module


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
            expected_args = ['uint16', 'rom_ptr'] if DISPATCHER_LEAVES[symbol] == 'rom_ptr' else ['uint16']
            if routine.get('arg_types', ['uint16'] * routine.get('args', 0)) != expected_args or routine['return'] != DISPATCHER_LEAVES[symbol]:
                raise ValueError(f'dispatcher leaf signature differs: {symbol}')
            if DISPATCHER_LEAVES[symbol] == 'rom_ptr' and routine.get('pointer_bank') != 0x86:
                raise ValueError(f'dispatcher leaf pointer bank differs: {symbol}')
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


def hardware_output(inventory):
    names = [name for name in ('hardware.log', 'hardware.log.gz') if name in inventory]
    if len(names) != 1:
        raise ValueError('expected one sealed hardware event stream')
    return names[0]


def generate_effect_writer(compressed=False, packed=False):
    """Keep the observer's formatted bytes and order; only change storage."""
    if packed:
        return ('#include <stdarg.h>\n#include <unistd.h>\nstatic FILE *port_effects;\n'
                'static void PortEffect(FILE *stream,const char *format,...) {\n'
                ' va_list args;va_start(args,format);int n=vfprintf(stream,format,args);va_end(args);if(n<0)exit(2);\n}\n'
                '#define PortEffectsOpen() fopen(getenv("SM_EVIDENCE_PIPE"),"wb")\n'
                '#define PortEffectsClose() fclose(port_effects)\n')
    if not compressed:
        return ('static FILE *port_effects;\n'
                '#define PortEffect fprintf\n'
                '#define PortEffectsOpen() fopen("hardware.log","wb")\n'
                '#define PortEffectsClose() fclose(port_effects)\n')
    return r"""#include <zlib.h>
#include <stdarg.h>
static gzFile port_effects;
static void PortEffect(gzFile stream,const char *format,...) {
 char buffer[256];va_list args;va_start(args,format);
 int count=vsnprintf(buffer,sizeof(buffer),format,args);va_end(args);
 if(count<0 || (size_t)count>=sizeof(buffer) || gzwrite(stream,buffer,count)!=count)exit(2);
}
#define PortEffectsOpen() gzopen("hardware.log.gz","wb1")
#define PortEffectsClose() (gzclose(port_effects)!=Z_OK)
"""


def reclaim_duplicate_rom_fixtures(directory, inventory):
    """Share excluded copies only after all match the retained sealed ROM view."""
    view = directory / 'rom-view.bin'
    expected = inventory.get('rom-view.bin')
    fixtures = sorted((directory / 'fixtures').glob('*.rom'))
    if not expected or view.is_symlink() or stream_digest(view) != expected:
        raise ValueError('retained ROM view seal differs')
    exemplars = {expected: view}
    plans = []
    groups = {}
    for fixture in fixtures:
        if fixture.is_symlink():
            raise ValueError('excluded ROM fixture is a symlink')
        seal = stream_digest(fixture)
        exemplar = exemplars.get(seal)
        if exemplar is None:
            exemplars[seal] = fixture
            exemplar = fixture
        elif fixture.stat().st_size != exemplar.stat().st_size or fixture.read_bytes() != exemplar.read_bytes():
            raise ValueError('ROM fixture bytes differ despite matching seal')
        plans.append((fixture, exemplar))
        groups.setdefault(seal, {'retained': str(exemplar.relative_to(directory)), 'copies': 0})['copies'] += 1
    reclaimed = 0
    shared = 0
    for fixture, exemplar in plans:
        old = fixture.stat()
        if old.st_ino == exemplar.stat().st_ino and old.st_dev == exemplar.stat().st_dev:
            continue
        temporary = fixture.with_name(fixture.name + '.dedup-' + uuid.uuid4().hex)
        try:
            os.link(exemplar, temporary)
            os.replace(temporary, fixture)
        finally:
            temporary.unlink(missing_ok=True)
        if old.st_nlink == 1:
            reclaimed += old.st_size
        shared += 1
    return {'copies_shared': shared, 'snapshot_count': len(fixtures), 'bytes_reclaimed': reclaimed,
            'retained_views': groups}


def share_finished_baseline(directory, inventory, baseline, baseline_inventory):
    """Share finished observations without admitting the baseline gameplay."""
    reclaimed = 0
    for relative in (hardware_output(inventory), 'rom-view.bin'):
        current = directory / relative
        previous = baseline / relative
        expected = baseline_inventory.get(relative)
        if not expected or current.is_symlink() or previous.is_symlink():
            raise ValueError('invalid finished baseline observation')
        if stream_digest(previous) != expected or stream_digest(current) != inventory[relative]:
            raise ValueError('finished baseline observation seal differs')
        if inventory[relative] != expected:
            continue
        old = current.stat()
        prior = previous.stat()
        if old.st_dev != prior.st_dev or old.st_ino == prior.st_ino:
            continue
        with current.open('rb') as left, previous.open('rb') as right:
            while True:
                chunk = left.read(1024 * 1024)
                if chunk != right.read(1024 * 1024):
                    raise ValueError('finished baseline observation bytes differ')
                if not chunk:
                    break
        temporary = current.with_name(current.name + '.dedup-' + uuid.uuid4().hex)
        try:
            os.link(previous, temporary)
            os.replace(temporary, current)
        finally:
            temporary.unlink(missing_ok=True)
        if stream_digest(current) != expected:
            raise ValueError('finished baseline observation seal differs after sharing')
        if old.st_nlink == 1:
            reclaimed += old.st_size
    return reclaimed


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
    for relative in (hardware_output(inventory), 'rom-view.bin'):
        expected = anchor['artifacts'].get(relative)
        # Historical plain and compressed representations remain separate seals.
        if relative.startswith('hardware.log') and relative not in anchor['artifacts']:
            continue
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
 PortEffect(port_effects,"room-plm %u %u %u\n",kind,a,b);
}
void SpawnRoomPLM(uint16 entry){PortPositionedPlm(1,entry,0);}
void SpawnHardcodedPlm(SpawnHardcodedPlmArgs p){PortPositionedPlm(2,p.field_2,p.field_0|p.field_1<<8);}
"""


def generate_projectile_abi():
    """An eight-word synchronous transaction; -2 alone permits native C."""
    return r'''#[unsafe(no_mangle)]
pub unsafe extern "C" fn port_native_projectile(ram:*mut u8,len:usize,rom:*const u8,rom_len:usize,host:*mut u16,host_len:usize,kind:u16,a:u16,b:u32)->i32 {
 if ram.is_null() || len!=WRAM_SIZE || rom_len>0x400000 || (rom.is_null() && rom_len!=0) {return -1;}
 let Some(rend)=(ram as usize).checked_add(len)else{return -1;};
 let Some(oend)=(rom as usize).checked_add(rom_len)else{return -1;};
 if (ram as usize)<oend && (rom as usize)<rend {return -1;}
 let name=match kind{1=>"SpawnEprojWithRoomGfx",2=>"EprojRunAll",3=>"SpawnEprojWithGfx",_=>return -1};
 let rom=if rom_len==0{&[]}else{unsafe{std::slice::from_raw_parts(rom,rom_len)}};
 let mut owned=unsafe{std::slice::from_raw_parts(ram,len)}.to_vec();
 let no_host=if kind!=2{(0..18).all(|i|owned[0x1997+i*2]!=0 || owned[0x1998+i*2]!=0)}else{owned[0x198e]&0x80==0};
 if !no_host {
  if host.is_null() || host_len!=8 || (host as usize)%2!=0{return -1;}
  let Some(hend)=(host as usize).checked_add(16)else{return -1;};
  if (host as usize)<rend && (ram as usize)<hend || (host as usize)<oend && (rom.as_ptr() as usize)<hend{return -1;}
 }
 let supported=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||memory::observed_projectile_ownership::supports_call(&Wram::new(&mut owned).unwrap(),rom,name,a.into(),b as i32)));
 match supported{Ok(Ok(false))=>return -2,Ok(Ok(true))=>{},_=>return -1}
 let mut context=if no_host{memory::enemy_projectiles::ProjectileHost::default()}else{memory::enemy_projectiles::ProjectileHost::from_words(unsafe{std::ptr::read(host.cast::<[u16;8]>())})};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||memory::observed_projectile_ownership::dispatch_host(&mut Wram::new(&mut owned).unwrap(),rom,&mut context,name,a.into(),b as i32)));
 match result{Ok(Ok(value))=>{let Ok(value)=i32::try_from(value)else{return -1;};unsafe{std::ptr::copy_nonoverlapping(owned.as_ptr(),ram,len);if !no_host{std::ptr::write(host.cast::<[u16;8]>(),context.words());}}value},_=>-1}
}'''


def generate_projectile_bank_adapters():
    """Access the actual bank86 globals, including its static r22 scratch."""
    return r'''
void PortProjectileHostLoad(const uint16 *v) {
 eproj_spawn_pt=(Point16U){v[0],v[1]};eproj_spawn_rect=(Rect16U){v[2],v[3],v[4],v[5]};
 eproj_spawn_r22=v[6];eproj_spawn_varE24=v[7];
}
void PortProjectileHostSave(uint16 *v) {
 v[0]=eproj_spawn_pt.x;v[1]=eproj_spawn_pt.y;
 v[2]=eproj_spawn_rect.x;v[3]=eproj_spawn_rect.y;v[4]=eproj_spawn_rect.w;v[5]=eproj_spawn_rect.h;
 v[6]=eproj_spawn_r22;v[7]=eproj_spawn_varE24;
}
'''


def generate_projectile_host():
    """Bound native ownership and retain material combat separately from Ceres."""
    return r'''void PortC_SpawnEprojWithRoomGfx(uint16,uint16);
void PortC_SpawnEprojWithGfx(uint16,uint16,uint16);
void PortC_EprojRunAll(void);
void PortProjectileHostLoad(const uint16*);
void PortProjectileHostSave(uint16*);
extern int32_t port_native_projectile(uint8_t*,size_t,const uint8_t*,size_t,uint16_t*,size_t,uint16_t,uint16_t,uint32_t);
static unsigned long port_projectile_total[3],port_projectile_c[3],port_projectile_rust[3],port_projectile_refused[3];
static unsigned long port_projectile_active_c[3],port_projectile_active_rust[3],port_projectile_9642_c[3],port_projectile_9642_rust[3];
static unsigned port_projectile_capture[3],port_projectile_combat_capture[3];
static unsigned port_projectile_depth;
static const uint16 port_projectile_ids[9]={0x9642,0x9650,0x965e,0x966c,0x967a,0x9688,0x9696,0xa387,0xa395};
static unsigned long port_projectile_id_c[9],port_projectile_id_rust[9],port_projectile_id_refused[9];
static unsigned port_projectile_id_capture[9];
static unsigned long port_projectile_alloc_total[9],port_projectile_alloc_c[9],port_projectile_alloc_rust[9],port_projectile_alloc_refused[9];
static unsigned port_projectile_alloc_capture[9];
static int PortProjectileActive(void) {
 for(unsigned k=0;k<36;k+=2)if(GET_WORD(g_ram+0x1997+k))return 1;
 return 0;
}
static int PortProjectileHas9642(void) {
 for(unsigned k=0;k<36;k+=2)if(GET_WORD(g_ram+0x1997+k)==0x9642)return 1;
 return 0;
}
static int PortProjectileFree(void) {
 for(unsigned k=0;k<36;k+=2)if(!GET_WORD(g_ram+0x1997+k))return 1;
 return 0;
}
static void PortProjectileCapture(unsigned kind,unsigned sequence,int combat,const char *phase) {
 char name[120];uint16 words[8];uint8 bytes[16];PortProjectileHostSave(words);
 snprintf(name,sizeof(name),"fixtures/projectile-%s-%u-%03u-%s.wram",combat?"9642":"active",kind,sequence,phase);
 FILE *f=fopen(name,"wb");if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f))exit(2);
 for(unsigned i=0;i<8;i++){bytes[i*2]=words[i];bytes[i*2+1]=words[i]>>8;}
 snprintf(name,sizeof(name),"fixtures/projectile-%s-%u-%03u-%s.host",combat?"9642":"active",kind,sequence,phase);
 f=fopen(name,"wb");if(!f || fwrite(bytes,1,sizeof(bytes),f)!=sizeof(bytes) || fclose(f))exit(2);
}
static void PortProjectileIdCapture(unsigned id,unsigned sequence,const char *phase) {
 char name[120];uint16 words[8];uint8 bytes[16];PortProjectileHostSave(words);
 snprintf(name,sizeof(name),"fixtures/projectile-id-%04x-%03u-%s.wram",port_projectile_ids[id],sequence,phase);
 FILE *f=fopen(name,"wb");if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f))exit(2);
 for(unsigned i=0;i<8;i++){bytes[i*2]=words[i];bytes[i*2+1]=words[i]>>8;}
 snprintf(name,sizeof(name),"fixtures/projectile-id-%04x-%03u-%s.host",port_projectile_ids[id],sequence,phase);
 f=fopen(name,"wb");if(!f || fwrite(bytes,1,sizeof(bytes),f)!=sizeof(bytes) || fclose(f))exit(2);
}
static void PortProjectileAllocCapture(unsigned id,unsigned sequence,const char *phase) {
 char name[120];uint16 words[8];uint8 bytes[16];PortProjectileHostSave(words);
 snprintf(name,sizeof(name),"fixtures/projectile-allocation-%04x-%03u-%s.wram",port_projectile_ids[id],sequence,phase);
 FILE *f=fopen(name,"wb");if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f))exit(2);
 for(unsigned i=0;i<8;i++){bytes[i*2]=words[i];bytes[i*2+1]=words[i]>>8;}
 snprintf(name,sizeof(name),"fixtures/projectile-allocation-%04x-%03u-%s.host",port_projectile_ids[id],sequence,phase);
 f=fopen(name,"wb");if(!f || fwrite(bytes,1,sizeof(bytes),f)!=sizeof(bytes) || fclose(f))exit(2);
}
static void PortProjectileC(unsigned kind,uint16 a,uint32_t b) {
 if(kind==1)PortC_SpawnEprojWithRoomGfx(a,(uint16)b);
 else if(kind==3)PortC_SpawnEprojWithGfx(a,(uint16)b,(uint16)(b>>16));
 else PortC_EprojRunAll();
}
static void PortProjectile(unsigned kind,uint16 a,uint32_t b) {
 if(port_projectile_depth){PortProjectileC(kind,a,b);return;}
 unsigned i=kind-1;uint16 words[8];int32 native=-2;
 uint16 definition=kind==1?a:(uint16)(b>>16);
 int allocating=kind!=2;
 int executing=allocating?PortProjectileFree():(GET_WORD(g_ram+0x198d)&0x8000)!=0;
 int active=executing && (PortProjectileActive() || (allocating && definition==0x9642));
 int combat=executing && (allocating?definition==0x9642:PortProjectileHas9642());
 unsigned allocation_id=9,allocation_capture=0;
 if(allocating && executing)for(unsigned j=0;j<9;j++)if(definition==port_projectile_ids[j]){allocation_id=j;break;}
 if(allocation_id<9){port_projectile_alloc_total[allocation_id]++;if(port_projectile_alloc_capture[allocation_id]<2){allocation_capture=++port_projectile_alloc_capture[allocation_id];PortProjectileAllocCapture(allocation_id,allocation_capture,"before");}}
 unsigned capture=active && port_projectile_capture[i]<8?++port_projectile_capture[i]:0;
 unsigned material=combat && port_projectile_combat_capture[i]<8?++port_projectile_combat_capture[i]:0;
 unsigned ids[9]={0},id_capture[9]={0};
 if(kind==2 && executing)for(unsigned k=0;k<36;k+=2)for(unsigned j=0;j<9;j++)if(GET_WORD(g_ram+0x1997+k)==port_projectile_ids[j])ids[j]++;
 for(unsigned j=0;j<9;j++)if(ids[j] && port_projectile_id_capture[j]<2){id_capture[j]=++port_projectile_id_capture[j];PortProjectileIdCapture(j,id_capture[j],"before");}
 if(capture)PortProjectileCapture(kind,capture,0,"before");
 if(material)PortProjectileCapture(kind,material,1,"before");
 PortProjectileHostSave(words);
 if(port_mixed)native=port_native_projectile(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,words,8,kind,a,b);
 port_projectile_total[i]++;
 if(native==-2) {
  port_projectile_c[i]++;port_projectile_refused[i]+=port_mixed;
  port_projectile_depth++;PortProjectileC(kind,a,b);port_projectile_depth--;
  port_projectile_active_c[i]+=active;port_projectile_9642_c[i]+=combat;
  for(unsigned j=0;j<9;j++){port_projectile_id_c[j]+=ids[j];port_projectile_id_refused[j]+=port_mixed?ids[j]:0;}
  if(allocation_id<9){port_projectile_alloc_c[allocation_id]++;port_projectile_alloc_refused[allocation_id]+=port_mixed;}
 } else {
  if(native<0){fprintf(stderr,"accepted projectile error %u %u %u\n",kind,a,b);exit(4);}
  PortProjectileHostLoad(words);port_projectile_rust[i]++;
  port_projectile_active_rust[i]+=active;port_projectile_9642_rust[i]+=combat;
  for(unsigned j=0;j<9;j++)port_projectile_id_rust[j]+=ids[j];
  if(allocation_id<9)port_projectile_alloc_rust[allocation_id]++;
 }
 if(allocation_capture)PortProjectileAllocCapture(allocation_id,allocation_capture,"after");
 for(unsigned j=0;j<9;j++)if(id_capture[j])PortProjectileIdCapture(j,id_capture[j],"after");
 if(capture)PortProjectileCapture(kind,capture,0,"after");
 if(material)PortProjectileCapture(kind,material,1,"after");
 PortEffect(port_effects,"projectile %u %u %u\n",kind,a,b);
}
void SpawnEprojWithRoomGfx(uint16 definition,uint16 parameter){PortProjectile(1,definition,parameter);}
void EprojRunAll(void){PortProjectile(2,0,0);}
void SpawnEprojWithGfx(uint16 parameter,uint16 enemy,uint16 definition){PortProjectile(3,parameter,(uint32_t)enemy|((uint32_t)definition<<16));}
static void PortProjectileCounters(void) {
 for(unsigned i=0;i<3;i++)printf("projectile-hit %u %lu %lu %lu %lu %lu %lu %lu %lu\n",i,
 port_projectile_total[i],port_projectile_c[i],port_projectile_rust[i],port_projectile_refused[i],
 port_projectile_active_c[i],port_projectile_active_rust[i],port_projectile_9642_c[i],port_projectile_9642_rust[i]);
 for(unsigned i=0;i<9;i++)printf("projectile-id-hit %u %lu %lu %lu\n",port_projectile_ids[i],port_projectile_id_c[i],port_projectile_id_rust[i],port_projectile_id_refused[i]);
 for(unsigned i=0;i<9;i++)printf("projectile-allocation-hit %u %lu %lu %lu %lu\n",port_projectile_ids[i],port_projectile_alloc_total[i],port_projectile_alloc_c[i],port_projectile_alloc_rust[i],port_projectile_alloc_refused[i]);
}
static void PortProjectileState(const char *name) {
 uint16 words[8];uint8 bytes[16];PortProjectileHostSave(words);
 for(unsigned i=0;i<8;i++){bytes[i*2]=words[i];bytes[i*2+1]=words[i]>>8;}
 FILE *f=fopen(name,"wb");if(!f || fwrite(bytes,1,sizeof(bytes),f)!=sizeof(bytes) || fclose(f))exit(2);
}
'''


def generate_family_abi(manifest, module, index):
    """Generate the bounded synchronous ABI used by the live host."""
    if manifest.get('projectile_host'):
        raise ValueError('projectile host requires explicit --projectile-verdict ABI')
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


def parse_projectile_counts(stdout, mode):
    rows = [line.split()[1:] for line in stdout.splitlines() if line.startswith('projectile-hit ')]
    if len(rows) != 3:
        raise ValueError('missing native projectile ownership counters')
    counts = {}
    fields = ('total', 'c', 'rust', 'refused', 'active_c', 'active_rust', '9642_c', '9642_rust')
    for row in rows:
        if len(row) != 9:
            raise ValueError('invalid native projectile counter shape')
        kind, *values = map(int, row)
        if kind not in (0, 1, 2) or kind in counts or any(v < 0 for v in values):
            raise ValueError('invalid native projectile counter kind')
        entry = dict(zip(fields, values))
        if (entry['total'] != entry['c'] + entry['rust']
                or entry['refused'] != (entry['c'] if mode == 'mixed' else 0)
                or entry['9642_c'] > entry['active_c'] or entry['active_c'] > entry['c']
                or entry['9642_rust'] > entry['active_rust'] or entry['active_rust'] > entry['rust']
                or mode == 'c' and entry['rust']):
            raise ValueError('invalid native projectile ownership counts')
        counts[kind] = entry
    return counts


def parse_projectile_id_counts(stdout, mode):
    rows = [line.split()[1:] for line in stdout.splitlines() if line.startswith('projectile-id-hit ')]
    counts = {}
    for row in rows:
        if len(row) != 4:
            raise ValueError('invalid projectile ID counter shape')
        projectile, c, rust, refused = map(int, row)
        if (projectile not in OBSERVED_PROJECTILE_IDS or projectile in counts
                or min(c, rust, refused) < 0 or refused != (c if mode == 'mixed' else 0)
                or mode == 'c' and rust):
            raise ValueError('invalid projectile ID ownership counts')
        counts[projectile] = {'c': c, 'rust': rust, 'refused': refused}
    if set(counts) != set(OBSERVED_PROJECTILE_IDS):
        raise ValueError('missing observed projectile ID counters')
    return counts


def require_projectile_id_admission(hits, inventories, frames):
    if hits['c-0'] != hits['c-1'] or hits['mixed-0'] != hits['mixed-1']:
        raise ValueError('projectile ID ownership differs across independent replays')
    required = set(OBSERVED_PROJECTILE_IDS if frames == 129712 else
                   (0x9642, 0x9650, 0x966c, 0x967a, 0xa387, 0xa395))
    for projectile in OBSERVED_PROJECTILE_IDS:
        baseline, mixed = hits['c-0'][projectile], hits['mixed-0'][projectile]
        if baseline['c'] != mixed['c'] + mixed['rust']:
            raise ValueError('projectile ID scheduler traversal counts diverged')
        if baseline['c']:
            required.add(projectile)
        if projectile in required:
            if not mixed['rust'] or mixed['c']:
                raise ValueError(f'observed projectile ID lacks exclusive Rust scheduler execution: {projectile:04x}')
            for phase in ('before', 'after'):
                for suffix in ('wram', 'host'):
                    member = f'fixtures/projectile-id-{projectile:04x}-001-{phase}.{suffix}'
                    if any(member not in inventory for inventory in inventories.values()):
                        raise ValueError('missing per-ID WRAM/eight-word host capture')


def parse_projectile_allocation_counts(stdout, mode):
    rows = [line.split()[1:] for line in stdout.splitlines() if line.startswith('projectile-allocation-hit ')]
    counts = {}
    for row in rows:
        if len(row) != 5:
            raise ValueError('invalid projectile allocation counter shape')
        projectile, total, c, rust, refused = map(int, row)
        if (projectile not in OBSERVED_PROJECTILE_IDS or projectile in counts
                or min(total, c, rust, refused) < 0 or total != c + rust
                or refused != (c if mode == 'mixed' else 0) or mode == 'c' and rust):
            raise ValueError('invalid projectile allocation ownership counts')
        counts[projectile] = {'total': total, 'c': c, 'rust': rust, 'refused': refused}
    if set(counts) != set(OBSERVED_PROJECTILE_IDS):
        raise ValueError('missing observed projectile allocation counters')
    return counts


def require_projectile_allocation_admission(hits, inventories, frames):
    if hits['c-0'] != hits['c-1'] or hits['mixed-0'] != hits['mixed-1']:
        raise ValueError('projectile allocation ownership differs across independent replays')
    for projectile in OBSERVED_PROJECTILE_IDS:
        baseline, mixed = hits['c-0'][projectile], hits['mixed-0'][projectile]
        if baseline['total'] != mixed['total']:
            raise ValueError('projectile allocation traversal counts diverged')
        if baseline['total'] and (mixed['c'] or not mixed['rust']):
            raise ValueError(f'observed projectile allocation lacks exclusive Rust execution: {projectile:04x}')
        if baseline['total']:
            for phase in ('before', 'after'):
                for suffix in ('wram', 'host'):
                    member = f'fixtures/projectile-allocation-{projectile:04x}-001-{phase}.{suffix}'
                    if any(member not in inventory for inventory in inventories.values()):
                        raise ValueError('missing native allocation WRAM/eight-word host capture')
    expected = {0x9642: 16, 0xa387: 4, 0xa395: 4} if frames == 129712 else {}
    if not expected:
        for projectile in (0x9642, 0xa387, 0xa395):
            if not hits['mixed-0'][projectile]['rust']:
                raise ValueError('native allocation lacks Ridley/Ceres exercise')
    for projectile, total in expected.items():
        if hits['mixed-0'][projectile]['rust'] != total:
            raise ValueError(f'full-route native allocation count differs: {projectile:04x}')


def require_projectile_admission(hits, inventories, fallback_calls):
    """Empty scheduler passes cannot admit the observed combat lifecycle."""
    if hits['mixed-0'] != hits['mixed-1'] or hits['c-0'] != hits['c-1']:
        raise ValueError('native projectile ownership differs across independent replays')
    if fallback_calls:
        raise ValueError('native projectile admission requires zero CPU fallback')
    for kind in (0, 1, 2):
        baseline, mixed = hits['c-0'][kind], hits['mixed-0'][kind]
        if (baseline['total'] != mixed['total']
                or baseline['active_c'] != mixed['active_c'] + mixed['active_rust']
                or baseline['9642_c'] != mixed['9642_c'] + mixed['9642_rust']):
            raise ValueError('native projectile traversal counts diverged')
        if not mixed['total'] or mixed['rust'] != mixed['total'] or mixed['c'] or mixed['refused']:
            raise ValueError('complete observed native call ownership requires exclusive Rust and zero support refusals')
        if not mixed['active_rust']:
            raise ValueError('native projectile entry lacks nonempty execution')
        if kind == 2:
            continue
        if not mixed['9642_rust'] or mixed['9642_c']:
            raise ValueError('native projectile allocator/scheduler lacks exclusive nonempty 9642 execution')
        for phase in ('before', 'after'):
            for suffix in ('wram', 'host'):
                material = f'fixtures/projectile-9642-{kind+1}-001-{phase}.{suffix}'
                if any(material not in inventory for inventory in inventories.values()):
                    raise ValueError('missing material 9642 WRAM/eight-word host capture')


def generate_active_eproj_capture(symbol_count):
    """Retain bounded active-pool calls without changing gameplay state."""
    return f'''static unsigned port_active_captures[{symbol_count}];
static unsigned PortActiveSequence(unsigned routine) {{
 if(port_active_captures[routine]>=8)return 0;
 for(unsigned offset=0x1997;offset<0x1997+36;offset+=2) {{
  if(g_ram[offset] || g_ram[offset+1])return ++port_active_captures[routine];
 }}
 return 0;
}}
static void PortActiveCapture(unsigned routine,unsigned sequence,const char *phase) {{
 char name[120];snprintf(name,sizeof(name),"fixtures/active-port-%03u-%03u-%s.wram",routine,sequence,phase);
 FILE *f=fopen(name,"wb");if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f))exit(2);
}}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--family-verdict', type=Path, action='append', default=[])
    parser.add_argument('--collision-verdict', type=Path, action='append', default=[])
    parser.add_argument('--dispatcher-verdict', type=Path, action='append', default=[],
                        help='Authenticated bank86 dispatcher evidence for safe movement and extent leaves')
    parser.add_argument('--storage', choices=('packed', 'plain', 'gzip'), default='packed', help='Storage for future observations (default: shared packed store)')
    parser.add_argument('--evidence-store', type=Path, default=ROOT / 'out/evidence', help='Shared immutable evidence store')
    parser.add_argument('--compress-effects', action='store_true', help='Losslessly gzip every ordered hardware event')
    parser.add_argument('--dedup-anchor-verdict', type=Path, help='Share byte-identical finished logs with a sealed prior live run')
    parser.add_argument('--plm-verdict', type=Path)
    parser.add_argument('--projectile-verdict', type=Path,
                        help='Authenticated host-aware native projectile allocator/scheduler evidence')
    parser.add_argument('--room-plm', action='store_true', help='Require native room/hardcoded setup hits')
    parser.add_argument('--capture-active-eprojs', action='store_true',
                        help='Retain the first eight outer calls per symbol with a nonempty enemy projectile pool')
    parser.add_argument('--frames', type=int, default=9000)
    parser.add_argument('--accepted', type=Path, default=ROOT / 'out/port/decompressor-offline/decomp-acceptance.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.compress_effects:
        args.storage = "gzip"
    args.compress_effects = args.storage == "gzip"
    packed = args.storage == "packed"
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
        verdict.update(rom_sha256=digest(rom), inputs_sha256=digest(inputs), frames=args.frames, hardware_storage=args.storage,
                       active_eproj_captures=args.capture_active_eprojs)
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
        projectile_contract = None
        if args.projectile_verdict:
            projectile_contract = load_projectile_contract(args.projectile_verdict, verdict['source'])
            verdict['projectile_predecessor'] = {'path': str(args.projectile_verdict.resolve()),
                                               'sha256': digest(args.projectile_verdict)}
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
        if not families and not projectile_contract:
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
        if packed:
            import port_evidence
            evidence_binary = port_evidence.build(run, execute)
            verdict['evidence_reader_sha256'] = stream_digest(evidence_binary)
            verdict['evidence_storage'] = {'format': 'sm-evidence-v1', 'subjects': {}}
        rust = [f'#[path={json.dumps(str(ROOT / "crates/port/src/lib.rs"))}] mod memory;', 'use memory::*;']
        if args.plm_verdict:
            rust.append(generate_plm_abi(args.room_plm))
        if projectile_contract:
            rust.append(generate_projectile_abi())
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
                if projectile_contract and symbol in PROJECTILE_ENTRIES:
                    raise ValueError('native projectile entries require only --projectile-verdict')
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
                if family_index in dispatcher_indexes and routine['return'] == 'rom_ptr':
                    c_type = 'const uint8_t *'
                    declaration = declaration.replace('const uint16_t *', c_type)
                arguments = ','.join(['a', 'b'][:count])
                actual_args = ['a' if count > 0 else '0', 'b' if count > 1 else '0']
                if 'rom_ptr' in types:
                    actual_args = [f'PortPointerResult({name},a)' if kind == 'rom_ptr' else name for kind, name in zip(types, ['a', 'b'])]
                if family_index in dispatcher_indexes and 'rom_ptr' in types:
                    actual_args = [f'PortBank86PointerResult({name})' if kind == 'rom_ptr' else name for kind, name in zip(types, ['a', 'b'])]
                actual = ','.join(actual_args)
                if routine.get('packed_u16_args'):
                    if types != ['uint32', 'uint32'] or routine['packed_u16_args'] != 4:
                        raise ValueError('invalid packed four-word signature')
                    declaration = 'uint16_t a,uint16_t b,uint16_t c,uint16_t d'
                    arguments = 'a,b,c,d'
                    actual = '(uint32_t)a|((uint32_t)b<<16),(uint32_t)c|((uint32_t)d<<16)'
                c_call = f'C_{symbol}({arguments})'
                reference = f'{c_call};' if c_type == 'void' else f'result=(uint32_t){c_call};'
                returned = '' if c_type == 'void' else f'return ({c_type})result;'
                if routine['return'] == 'rom_ptr':
                    reference = f'const uint16_t *reference={c_call};result=PortPointerResult(reference,a);'
                    if family_index in dispatcher_indexes:
                        reference = f'{c_type}reference={c_call};result=PortBank86PointerResult(reference);'
                    returned = 'return PortPointerFromResult((uint32_t)result);'
                    if family_index in dispatcher_indexes:
                        returned = 'return (const uint8_t *)PortPointerFromResult((uint32_t)result);'
                active_before = (f'unsigned active_capture=outer?PortActiveSequence({index}):0;\n'
                                 f' if(active_capture)PortActiveCapture({index},active_capture,"before");'
                                 if args.capture_active_eprojs else '')
                active_after = (f'if(active_capture)PortActiveCapture({index},active_capture,"after");'
                                if args.capture_active_eprojs else '')
                wrappers.append(f'''extern {c_type} C_{symbol}({declaration});
{c_type} {symbol}({declaration}) {{
 int64_t result=0;
 int outer=port_call_depth++==0;
 unsigned long capture=outer?++port_outer_hits[{index}]:0;
 if(outer && capture<=8)PortCapture({index},capture,"before");
 {active_before}
 if(port_mixed) {{result={abi_symbol}(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,{routine_index},{actual}); if(result<0) {{fprintf(stderr,"Rust family error {symbol}\\n");exit(4);}} port_rust_hits[{index}]++;}}
 else {{{reference} port_c_hits[{index}]++;}}
 port_call_depth--;
 {active_after}
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
        if projectile_contract:
            bank=source/'src/sm_86.c'
            text=bank_texts.get(bank,bank.read_text())
            for symbol in PROJECTILE_ENTRIES:
                pattern=definition_pattern(symbol)
                if len(pattern.findall(text))!=1:raise ValueError('projectile definition count differs')
                text=pattern.sub(lambda m:m.group(1)+'PortC_'+symbol+m.group(2),text)
            bank_texts[bank]=text+generate_projectile_bank_adapters()
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
''' + generate_effect_writer(args.compress_effects, packed) + '''
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
 PortEffect(port_effects,"plm %u %d %d %u\\n",header,before,cur_coll_amt32?*cur_coll_amt32:0,result);
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
static uint32_t PortBank86PointerResult(const uint8_t *pointer) {
 if(!pointer)return 0;
 uintptr_t p=(uintptr_t)pointer,base=(uintptr_t)g_rom+0x30000;
 if(p<0x10000){if(p<0x8000)exit(4);return 0x860000|(uint32_t)p;}
 if(p<base || p>=base+0x8000)exit(4);
 return 0x868000+(uint32_t)(p-base);
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
''' + (generate_positioned_plm_host() if args.room_plm else '')
        + (generate_projectile_host() if projectile_contract else '') + '''
static unsigned port_frame;
void __real_ppu_write(Ppu *,uint8_t,uint8_t);
void __wrap_ppu_write(Ppu *ppu,uint8_t adr,uint8_t val) {
 PortEffect(port_effects,"ppu %u %d %u %u\\n",port_frame,ppu==g_snes->my_ppu,adr,val);
 __real_ppu_write(ppu,adr,val);
}
void __real_RtlApuWrite(uint32 adr,uint8 val);
void __wrap_RtlApuWrite(uint32 adr,uint8 val) {
 PortEffect(port_effects,"apu %u %u %u\\n",port_frame,adr,val);
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
''' + ('  snprintf(name,sizeof(name),"frame-%06u.host",port_frame);PortProjectileState(name);\n' if projectile_contract else '') + '''
 }
 return result;
}
''' + f'static unsigned long port_c_hits[{len(symbols)}],port_rust_hits[{len(symbols)}],port_outer_hits[{len(symbols)}];\n'
        + (generate_active_eproj_capture(len(symbols)) if args.capture_active_eprojs else '') + '\n'.join(wrappers) + '''
int main(int argc,char **argv) {
 if(argc!=6) return 2;
 port_mixed=!strcmp(argv[1],"mixed");
 if(getenv("SM_EVIDENCE_STAGING") && chdir(getenv("SM_EVIDENCE_STAGING")))return 2;
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 port_calls=fopen("routine-calls.log","wb");if(!port_calls)return 2;
 char *original[]={argv[0],argv[2],argv[3],argv[4],argv[5],NULL};
 int result=retained_main(5,original);
''' + (' PortProjectileState("projectile-terminal.host");PortProjectileCounters();\n' if projectile_contract else '') + '''
 if(PortEffectsClose())return 2;
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
        build = ['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source / 'src'), *map(str, units), str(host), str(run / 'libfamilies.a'), str(run / 'target/release/libdecomp.a'), str(run / 'librng.a'), '-Wl,--gc-sections', '-Wl,--wrap=Call', '-Wl,--wrap=cpu_runOpcode', '-Wl,--wrap=DecompressToMem', '-Wl,--wrap=ppu_write', '-Wl,--wrap=RtlApuWrite', '-Wl,--wrap=RtlRunFrame', '-Wl,--wrap=SpawnPLM', '-lm', '-ldl', '-lpthread', *(['-lz'] if args.compress_effects else []), '-o', str(run / 'live')]
        execute(build, run, 'host-build')
        inventories, hits = {}, {}
        for mode in ('c', 'mixed'):
            for repeat in range(2):
                name = f'{mode}-{repeat}'
                directory = run / name
                (directory / 'fixtures').mkdir(parents=True)
                (directory / 'saves').mkdir()
                if packed:
                    command = [str(run / 'live'), mode, str(rom), str(inputs), str(args.frames), str(directory / '.evidence-stage/terminal.bin')]
                    verdict['commands'].append({'argv': command, 'cwd': str(directory)})
                    provenance = {'subject': name, 'binary_sha256': stream_digest(run / 'live'),
                                  'reader_sha256': verdict['evidence_reader_sha256'],
                                  'rom_sha256': verdict['rom_sha256'], 'inputs_sha256': verdict['inputs_sha256'],
                                  'frames': args.frames, 'source': verdict['source']}
                    pointer = port_evidence.packed_replay(command, directory, evidence_binary, args.evidence_store.resolve(),
                                                         provenance, max(240, args.frames / 40))
                    inventories[name] = {k: v for k, v in pointer['artifacts'].items()
                                         if Path(k).name != 'calls.jsonl' and Path(k).suffix != '.rom'}
                    verdict['evidence_storage']['subjects'][name] = {'run': pointer['run'], 'observations': pointer['observations'],
                        'pointer_sha256': stream_digest(directory / 'evidence.json')}
                    verdict.setdefault('storage_metrics', {})[name] = {'logical_bytes': pointer['bytes'], 'pipeline_seconds': pointer['pipeline_seconds'],
                        'producer_seconds': pointer['producer_seconds'], 'seal_seconds': pointer['seal_seconds'],
                        'stored_added_bytes': pointer['stored_added_bytes']}
                else:
                    execute([str(run / 'live'), mode, str(rom), str(inputs), str(args.frames), str(directory / 'terminal.bin')], directory, 'replay')
                    inventories[name] = {str(p.relative_to(directory)): stream_digest(p) for p in directory.rglob('*') if p.is_file() and p.name not in ('replay.stdout', 'replay.stderr', 'calls.jsonl') and p.suffix != '.rom'}
                    verdict.setdefault('rom_fixture_cleanup', {})[name] = reclaim_duplicate_rom_fixtures(directory, inventories[name])
                    if args.dedup_anchor_verdict:
                        reclaimed = share_finished_outputs(directory, inventories[name], args.dedup_anchor_verdict)
                        verdict.setdefault('finished_output_dedup', {})[name] = reclaimed
                        verdict['dedup_anchor'] = {'path': str(args.dedup_anchor_verdict.resolve()),
                                                  'sha256': stream_digest(args.dedup_anchor_verdict),
                                                  'outputs': {key: inventories[name][key] for key in (hardware_output(inventories[name]), 'rom-view.bin')},
                                                  'scope': 'historical immutable bytes only; no current-source admission'}
                    if args.compress_effects and name != 'c-0':
                        reclaimed = share_finished_baseline(directory, inventories[name], run / 'c-0', inventories['c-0'])
                        verdict.setdefault('finished_baseline_sharing', {})[name] = {'bytes_reclaimed': reclaimed, 'baseline': 'c-0', 'inventory': inventories['c-0'], 'scope': 'finished immutable observations; gameplay admission checked separately'}
                    if name != 'mixed-1':
                        prior_rom_peak = max(row['snapshot_count'] for row in verdict['rom_fixture_cleanup'].values()) * (directory / 'rom-view.bin').stat().st_size
                        next_peak = (directory / hardware_output(inventories[name])).stat().st_size + prior_rom_peak + 512 * 1024 * 1024
                        if shutil.disk_usage(run).free < next_peak:
                            raise ValueError('insufficient space for another independent replay log and ROM snapshots')
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
                if projectile_contract:
                    verdict.setdefault('projectile_hits', {})[name] = parse_projectile_counts((directory/'replay.stdout').read_text(), mode)
                    verdict.setdefault('projectile_id_hits', {})[name] = parse_projectile_id_counts((directory/'replay.stdout').read_text(), mode)
                    verdict.setdefault('projectile_allocation_hits', {})[name] = parse_projectile_allocation_counts((directory/'replay.stdout').read_text(), mode)
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
        for symbol in dispatcher_symbols:
            counts = [hits[name][symbol]['c' if name.startswith('c-') else 'rust']
                      for name in ('c-0', 'c-1', 'mixed-0', 'mixed-1')]
            if not counts[0] or len(set(counts)) != 1:
                raise ValueError(f'dispatcher leaf lacks matching nonzero exclusive execution: {symbol}')
        if not sum(row['rust'] for row in hits['mixed-0'].values()) and not projectile_contract:
            raise ValueError('no Rust gameplay family executed')
        if projectile_contract:
            require_projectile_admission(verdict['projectile_hits'], inventories,
                                         verdict['progress']['mixed-0'][-1]['fallback_calls'])
            require_projectile_id_admission(verdict['projectile_id_hits'], inventories, args.frames)
            require_projectile_allocation_admission(verdict['projectile_allocation_hits'], inventories, args.frames)
            verdict['projectile_native_installed'] = True
            verdict['projectile_abi'] = 'synchronous-eight-word-host-v2-with-gfx'
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
    shutil.rmtree(run / 'evidence-build', ignore_errors=True)
    if verdict.get('status') == 'mixed-C-Rust-integration':
        for archive in ('libfamilies.a', 'librng.a'):
            (run / archive).unlink(missing_ok=True)
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
    print(json.dumps({key: verdict[key] for key in ('status', 'run', 'error') if key in verdict}))
    return 0 if verdict['live_exercised'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
