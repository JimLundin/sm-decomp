"""Compiled controls for synchronous native enemy projectile live ownership."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from port_live import (generate_projectile_abi, generate_projectile_bank_adapters,
                       generate_projectile_host, load_projectile_contract,
                       parse_projectile_counts, require_projectile_admission,
                       OBSERVED_PROJECTILE_IDS, parse_projectile_id_counts,
                       require_projectile_id_admission)


class ProjectileLiveControls(unittest.TestCase):
    def test_installed_abi_preserves_refusal_and_absent_host_saturation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'installed.rs'
            source.write_text('#[path=' + json.dumps(str(ROOT / 'crates/port/src/lib.rs'))
                              + '] mod memory;use memory::*;\n' + generate_projectile_abi() + r'''
fn main(){
 let mut ram=vec![0u8;WRAM_SIZE];let rom=std::fs::read("''' + str(ROOT / 'rom/sm.sfc') + r'''").unwrap();let mut host=[1,2,3,4,5,6,7,8];
 unsafe {
  let before=ram.clone();let hbefore=host;
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),host.as_mut_ptr(),8,1,0xffff,0x1234),-2);
  assert_eq!(ram,before);assert_eq!(host,hbefore);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,2,0,0),0);
  assert_eq!(ram,before);
  for k in (0..36).step_by(2){Wram::new(&mut ram).unwrap().set_word(0x1997+k,1);}
  let before=ram.clone();
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,1,0xffff,0xabcd),0);
  let mut expected=before;expected[0x1993]=0xcd;expected[0x1994]=0xab;assert_eq!(ram,expected);assert_eq!(host,hbefore);
  ram.fill(0);{let mut r=Wram::new(&mut ram).unwrap();r.set_word(0xf7a,200);r.set_word(0xf7e,100);r.set_word(0x1995,1);r.set_word(0x7832,0x321);r.set_word(0x7834,0x456);}
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),host.as_mut_ptr(),8,1,0x9642,1),0);
  let r=Wram::new(&mut ram).unwrap();assert_eq!(r.word(0x1997+34),0x9642);assert_eq!(r.word(0x1a4b+34),225);assert_eq!(r.word(0x1a93+34),57);assert_eq!(host,hbefore);
 }
}
''')
            build = subprocess.run(['rustc', '--edition=2024', str(source), '-o', str(root / 'installed')], capture_output=True, text=True)
            self.assertEqual(build.returncode, 0, build.stderr)
            result = subprocess.run([str(root / 'installed')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_host_contract_requires_authenticated_native_pair(self):
        identity = {'identity': 'pinned'}
        evidence = {'source': identity, 'implementation': {
            'module': str(ROOT / 'crates/port/src/observed_projectile_ownership.rs')}}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = {'bank': 'sm_86.c', 'rom': {}, 'projectile_host': True, 'routines': [
                {'symbol': 'SpawnEprojWithRoomGfx', 'args': 2, 'return': 'void'},
                {'symbol': 'EprojRunAll', 'args': 0, 'return': 'void'},
                {'symbol': 'EprojInstruction', 'live_eligible': False}]}
            path = root / 'manifest.json'
            with patch('port_plm_instructions.verify_plm_instruction_evidence', return_value=evidence) as verify:
                path.write_text(json.dumps(manifest))
                load_projectile_contract(root / 'verdict.json', identity)
                verify.assert_called_once()
                for key, value in [('projectile_host', False), ('bank', 'sm_84.c')]:
                    changed = dict(manifest, **{key: value})
                    path.write_text(json.dumps(changed))
                    with self.assertRaisesRegex(ValueError, 'host-aware'):
                        load_projectile_contract(root / 'verdict.json', identity)
                manifest['routines'][1]['arg_types'] = ['uint16']
                path.write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError, 'signature'):
                    load_projectile_contract(root / 'verdict.json', identity)

    def test_abi_pointer_boundaries_refusal_and_failure_are_transactional(self):
        # Controlled implementation isolates the public pointer/transaction ABI
        # from real support semantics; the separate ownership tests cover those.
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'abi.rs'
            source.write_text(r'''
mod memory {
 pub const WRAM_SIZE:usize=0x20000;
 pub struct Wram<'a>{pub data:&'a mut [u8]}
 impl<'a> Wram<'a>{pub fn new(data:&'a mut [u8])->Result<Self,()>{Ok(Self{data})}}
 pub mod enemy_projectiles {
  #[derive(Default)] pub struct ProjectileHost(pub [u16;8]);
  impl ProjectileHost{pub fn from_words(w:[u16;8])->Self{Self(w)} pub fn words(&self)->[u16;8]{self.0}}
 }
 pub mod observed_projectile_ownership {
  use super::{Wram,enemy_projectiles::ProjectileHost};
  pub fn supports_call(r: &Wram, _: &[u8],name:&str,a:i32,_:i32)->Result<bool,&'static str>{
   Ok(name=="EprojRunAll" && r.data[0x198e]&0x80==0 || (0..18).all(|i|r.data[0x1997+i*2]!=0) || a!=0xffff)
  }
  pub fn dispatch_host(r:&mut Wram,_:&[u8],host:&mut ProjectileHost,name:&str,a:i32,b:i32)->Result<u32,&'static str>{
   if name=="EprojRunAll" && r.data[0x198e]&0x80==0{return Ok(0);}
   if (0..18).all(|i|r.data[0x1997+i*2]!=0){r.data[0x1993]=b as u8;r.data[0x1994]=(b>>8)as u8;return Ok(0);}
   r.data[100]=9;host.0[0]=77;
   match a{0xfffe=>Err("accepted failure"),0xfffd=>panic!("accepted panic"),_=>Ok(0)}
  }
 }
}
use memory::*;
''' + generate_projectile_abi() + r'''
fn main(){
 let mut ram=vec![0u8;WRAM_SIZE];let rom=[0u8;32];let mut host=[1,2,3,4,5,6,7,8];let before=ram.clone();let host_before=host;
 unsafe {
  assert_eq!(port_native_projectile(std::ptr::null_mut(),WRAM_SIZE,rom.as_ptr(),32,host.as_mut_ptr(),8,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE-1,rom.as_ptr(),32,host.as_mut_ptr(),8,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,ram.as_ptr(),WRAM_SIZE,host.as_mut_ptr(),8,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),32,ram.as_mut_ptr().cast(),8,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),32,rom.as_ptr().cast_mut().cast(),8,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),32,host.as_mut_ptr(),7,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),32,host.as_mut_ptr().cast::<u8>().add(1).cast(),8,1,0,0),-1);
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),32,host.as_mut_ptr(),8,3,0,0),-1);
  for a in [0xffff,0xfffe,0xfffd]{
   assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),32,host.as_mut_ptr(),8,1,a,0),if a==0xffff{-2}else{-1});
   assert_eq!(ram,before);assert_eq!(host,host_before);
  }
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,2,0,0),0);
  assert_eq!(ram,before);assert_eq!(host,host_before);
  for i in 0..18{ram[0x1997+i*2]=1;}
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,1,0xffff,0xabcd),0);
  assert_eq!(&ram[0x1993..0x1995],&[0xcd,0xab]);assert_eq!(host,host_before);
  ram.copy_from_slice(&before);
  ram[0x198e]=0x80;
  assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,host.as_mut_ptr(),8,2,0,0),0);
 }
 assert_eq!(ram[100],9);assert_eq!(host,[77,2,3,4,5,6,7,8]);
}
''')
            build = subprocess.run(['rustc', '--edition=2024', str(source), '-o', str(root / 'abi')], capture_output=True, text=True)
            self.assertEqual(build.returncode, 0, build.stderr)
            result = subprocess.run([str(root / 'abi')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_same_bank_host_publication_and_accepted_error_never_retry_c(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'fixtures').mkdir()
            bank = root / 'sm_86.c'
            original = (ROOT / 'deps/snesrev-sm/src/sm_86.c').read_text()
            original = original.replace('void SpawnEprojWithRoomGfx(', 'void ControlUnused_SpawnEprojWithRoomGfx(')
            original = original.replace('void EprojRunAll(', 'void ControlUnused_EprojRunAll(')
            bank.write_text(original + generate_projectile_bank_adapters() + r'''
void ControlSameBank(void) {
 eproj_spawn_pt=(Point16U){123,456};eproj_spawn_rect=(Rect16U){3,4,5,6};
 eproj_spawn_r22=7;eproj_spawn_varE24=8;SpawnEprojWithRoomGfx(0x9642,9);
}
''')
            source = root / 'host.c'
            source.write_text(r'''
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
typedef uint8_t uint8;typedef uint16_t uint16;typedef int32_t int32;
unsigned char g_ram[0x20000];const unsigned char *g_rom;
static struct {size_t romSize;} cart;
static struct {void *unused;typeof(&cart) cart;} snes={0,&cart};
static typeof(&snes) g_snes=&snes;
#define GET_WORD(p) ((uint16)((p)[0]|((p)[1]<<8)))
static int port_mixed=1;
#define PortEffect(stream,...) ((void)0)
#define port_effects 0
static int mode;
void PortC_SpawnEprojWithRoomGfx(uint16 a,uint16 b){FILE *f=fopen("c-retry","wb");assert(f);fclose(f);}
void PortC_EprojRunAll(void){PortC_SpawnEprojWithRoomGfx(0,0);}
int32_t port_native_projectile(uint8_t*r,size_t n,const uint8_t*o,size_t z,uint16_t*h,size_t hlen,uint16_t k,uint16_t a,uint16_t b){
 assert(hlen==8);if(k==2)return 0;assert(k==1);assert(a==0x9642);assert(b==9);
 assert(h[0]==123 && h[1]==456);for(unsigned i=2;i<8;i++)assert(h[i]==i+1);
 if(mode==1)return -1;if(mode==2)return -2;
 h[0]=321;h[1]=654;h[6]=0xabcd;r[0x1997]=0x42;r[0x1998]=0x96;return 0;
}
''' + generate_projectile_host() + r'''
void ControlSameBank(void);
int main(int argc,char **argv){
 mode=argc>1?atoi(argv[1]):0;ControlSameBank();uint16 words[8];PortProjectileHostSave(words);
 if(mode==0){assert(words[0]==321 && words[1]==654 && words[6]==0xabcd);assert(port_projectile_rust[0]==1 && port_projectile_9642_rust[0]==1);
 g_ram[0x198e]=0x80;for(unsigned i=0;i<5;i++)EprojRunAll();assert(port_projectile_id_rust[0]==5);assert(port_projectile_id_capture[0]==2);}
 if(mode==2){assert(port_projectile_refused[0]==1 && port_projectile_c[0]==1);assert(words[0]==123 && words[1]==456);}
 PortProjectileState("terminal.host");return 0;
}
''')
            result = subprocess.run(['cc', '-std=gnu11', '-ffunction-sections', '-fdata-sections', '-iquote', str(ROOT / 'deps/snesrev-sm/src'), str(bank), str(source), '-Wl,--gc-sections', '-o', str(root / 'host')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(root / 'host')], cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse((root / 'c-retry').exists())
            before = (root / 'fixtures/projectile-9642-1-001-before.host').read_bytes()
            after = (root / 'fixtures/projectile-9642-1-001-after.host').read_bytes()
            self.assertEqual(len(before), 16)
            self.assertEqual(len(after), 16)
            self.assertEqual(before[:4], b'\x7b\x00\xc8\x01')
            self.assertEqual(after[:4], b'\x41\x01\x8e\x02')
            self.assertEqual(len((root / 'fixtures/projectile-id-9642-002-after.host').read_bytes()), 16)
            self.assertFalse((root / 'fixtures/projectile-id-9642-003-before.wram').exists())
            result = subprocess.run([str(root / 'host'), '1'], cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 4)
            self.assertIn('accepted projectile error', result.stderr)
            self.assertFalse((root / 'c-retry').exists())
            result = subprocess.run([str(root / 'host'), '2'], cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / 'c-retry').exists())

    def test_nonempty_9642_admission_rejects_empty_traversals_and_refusals(self):
        baseline = parse_projectile_counts('projectile-hit 0 10 10 0 0 4 0 2 0\nprojectile-hit 1 100 100 0 0 20 0 5 0\n', 'c')
        mixed = parse_projectile_counts('projectile-hit 0 10 1 9 1 0 4 0 2\nprojectile-hit 1 100 0 100 0 0 20 0 5\n', 'mixed')
        hits = {'c-0': baseline, 'c-1': baseline, 'mixed-0': mixed, 'mixed-1': mixed}
        inventory = {f'fixtures/projectile-9642-{kind}-001-{phase}.{suffix}': 'sealed'
                     for kind in (1, 2) for phase in ('before', 'after') for suffix in ('wram', 'host')}
        inventories = {name: inventory for name in hits}
        require_projectile_admission(hits, inventories, 0)
        with self.assertRaisesRegex(ValueError, 'zero CPU'):
            require_projectile_admission(hits, inventories, 1)
        with self.assertRaisesRegex(ValueError, 'material'):
            require_projectile_admission(hits, {name: {} for name in hits}, 0)
        for field in ('active_rust', '9642_rust', '9642_c'):
            changed = {kind: dict(row) for kind, row in mixed.items()}
            changed[1][field] = 1 if field == '9642_c' else 0
            with self.assertRaises(ValueError):
                require_projectile_admission(dict(hits, **{'mixed-0': changed, 'mixed-1': changed}), inventories, 0)

    def test_per_id_full_route_requires_all_nine_and_material_host_captures(self):
        baseline = parse_projectile_id_counts('\n'.join(f'projectile-id-hit {p} 3 0 0' for p in OBSERVED_PROJECTILE_IDS), 'c')
        mixed = parse_projectile_id_counts('\n'.join(f'projectile-id-hit {p} 0 3 0' for p in OBSERVED_PROJECTILE_IDS), 'mixed')
        hits = {'c-0': baseline, 'c-1': baseline, 'mixed-0': mixed, 'mixed-1': mixed}
        inventory = {f'fixtures/projectile-id-{p:04x}-001-{phase}.{suffix}': 'sealed'
                     for p in OBSERVED_PROJECTILE_IDS for phase in ('before', 'after') for suffix in ('wram', 'host')}
        inventories = {name: inventory for name in hits}
        require_projectile_id_admission(hits, inventories, 129712)
        for p in OBSERVED_PROJECTILE_IDS:
            changed = {key: dict(row) for key, row in mixed.items()}
            changed[p] = {'c': 3, 'rust': 0, 'refused': 3}
            with self.assertRaisesRegex(ValueError, 'exclusive'):
                require_projectile_id_admission(dict(hits, **{'mixed-0': changed, 'mixed-1': changed}), inventories, 129712)
        with self.assertRaisesRegex(ValueError, 'capture'):
            require_projectile_id_admission(hits, {name: {} for name in hits}, 129712)


if __name__ == '__main__':
    unittest.main()
