"""Live port contract controls through the public command."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class LivePortTests(unittest.TestCase):
    def test_positioned_plm_abi_rejects_aliases_and_commits_only_success(self):
        import sys
        sys.path.insert(0, str(ROOT/'tools'))
        from port_live import generate_plm_abi
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            source=root/'control.rs'
            source.write_text('#[path='+json.dumps(str(ROOT/'crates/port/src/lib.rs'))+'] mod memory;use memory::*;\n'+generate_plm_abi(True)+r"""
fn main(){
 let mut ram=vec![0u8;WRAM_SIZE];let mut rom=vec![0u8;0x300000];
 rom[0x21000..0x21004].copy_from_slice(&[0x27,0xab,0x00,0x80]);
 Wram::new(&mut ram).unwrap().set_word(0x7a5,16);
 unsafe {
 assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,ram.as_ptr(),WRAM_SIZE,2,0x9000,0x0203),-1);
 assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),2,0x9000,0x0203),1);
 assert_eq!(Wram::new(&mut ram).unwrap().word(0x1c87+78),70);
 let before=ram.clone();
 rom[0x21000..0x21002].copy_from_slice(&[0xff,0xff]);
 assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),2,0x9000,0x0203),-2);
 assert_eq!(ram,before);
 assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),1,0xffff,0),-1);
 assert_eq!(ram,before);
 }
}
""")
            result=subprocess.run(['rustc','--edition=2024',str(source),'-o',str(root/'control')],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            result=subprocess.run([str(root/'control')],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)

    def test_pointer_definition_rename_preserves_calls_and_declarations(self):
        import sys
        sys.path.insert(0, str(ROOT / 'tools'))
        from port_live import definition_pattern
        text = ('const uint16 *EnemyInstr_Goto(uint16 k, const uint16 *jp);\n'
                'uint16 Other_EnemyInstr_Goto(uint16 k, const uint16 *jp) {return 0;}\n'
                'const uint16 *EnemyInstr_Goto(uint16 k, const uint16 *jp) {\n'
                '  return EnemyInstr_Goto(k, jp);\n}\n')
        renamed, count = definition_pattern('EnemyInstr_Goto').subn(r'\1C_EnemyInstr_Goto\2', text)
        self.assertEqual(count, 1)
        self.assertIn('const uint16 *C_EnemyInstr_Goto(', renamed)
        self.assertIn('return EnemyInstr_Goto(k, jp);', renamed)
        self.assertIn('const uint16 *EnemyInstr_Goto(uint16 k, const uint16 *jp);', renamed)

    def test_native_plm_rejection_is_transactional_and_unsupported_is_classified(self):
        import sys
        sys.path.insert(0, str(ROOT / 'tools'))
        from port_live import generate_plm_abi
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            source = run / 'abi.rs'
            source.write_text('''const WRAM_SIZE:usize=0x20000;
struct Wram<'a>(&'a mut [u8]);
impl<'a> Wram<'a>{fn new(r:&'a mut [u8])->Result<Self,&'static str>{Ok(Self(r))}}
mod memory {pub mod plm {
 use crate::Wram;
 pub fn supports_header(_: &[u8],header:u16)->Result<bool,&'static str>{Ok(header!=0)}
 pub fn spawn(r:&mut Wram,_:&[u8],header:u16,amount:&mut i32)->Result<u8,&'static str>{
  r.0[0]=99;*amount=77;if header==1{Err("setup error")}else{Ok(3)}
 }
}}
''' + generate_plm_abi())
            library = run / 'libabi.a'
            result = subprocess.run(['rustc', '--edition=2024', '--crate-type=staticlib', str(source), '-o', str(library)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            caller = run / 'control.c'
            caller.write_text('''#include <stdint.h>
#include <stddef.h>
static uint8_t ram[0x20000],rom[8];
extern int32_t port_native_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);
int main(void){int32_t amount=12;
 if(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),0,&amount)!=-2)return 1;
 if(ram[0]||amount!=12)return 2;
 if(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),1,&amount)!=-1)return 3;
 if(ram[0]||amount!=12)return 4;
 if(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),2,(int32_t*)(ram+1))!=-1)return 5;
 if(port_native_plm(ram,sizeof(ram),ram+1,8,2,&amount)!=-1)return 6;
 if(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),2,&amount)!=3)return 7;
 if(ram[0]!=99||amount!=77)return 8;return 0;}
''')
            executable = run / 'control'
            result = subprocess.run(['cc', str(caller), str(library), '-ldl', '-lpthread', '-lm', '-o', str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_missing_family_evidence_cannot_start_replay(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'output'
            result = subprocess.run([
                'python3', str(ROOT / 'tools/port_live.py'), '--family-verdict',
                str(Path(temporary) / 'missing.json'), '--output', str(output),
            ], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            verdicts = list(output.glob('*/verdict.json'))
            self.assertEqual(len(verdicts), 1)
            verdict = json.loads(verdicts[0].read_text())
            self.assertEqual(verdict['status'], 'rejected')
            self.assertFalse(verdict['live_exercised'])
            self.assertIn('error', verdict)


if __name__ == '__main__':
    unittest.main()

class LiveEvidenceControls(unittest.TestCase):
    def test_tampered_successful_family_outputs_reject_before_live_builds(self):
        import sys
        sys.path.insert(0, str(ROOT / 'tools'))
        import port_family
        source = Path('/home/jim/sm-decomp/deps/snesrev-sm')
        if not source.exists():
            self.skipTest('pinned source not prepared')
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            module = run / 'family.rs'
            module.write_text('use crate::Wram;pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&\'static str>{r.set_word(0x5e5,273);Ok(273)}')
            manifest = run / 'manifest.json'
            manifest.write_text(json.dumps({'family':'tamper-control','rust_module':str(module),'bank':'sm_80.c','routines':[{'symbol':'NextRandom','args':0,'return':'uint16','cases':[{'name':'known','expected_return':273}]}]}))
            evidence = port_family.verify(manifest, source, run / 'offline')
            self.assertEqual(evidence['status'],'offline-verified')
            verdict_path = Path(evidence['run']) / 'verdict.json'
            (verdict_path.parent/'known'/'rust-after.wram').write_bytes(bytes(0x20000))
            script = f'''import sys
sys.path.insert(0,{str(ROOT/'tools')!r})
import port_live
port_live.ROOT=__import__('pathlib').Path('/home/jim/sm-decomp')
import decomp_routine
decomp_routine.ROOT=port_live.ROOT
sys.argv=['port_live','--family-verdict',{str(verdict_path)!r},'--output',{str(run/'live')!r}]
raise SystemExit(port_live.main())
'''
            result = subprocess.run(['python3','-c',script],capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0,result.stdout+result.stderr)
            verdict = json.loads(next((run/'live').glob('*/verdict.json')).read_text())
            self.assertIn('artifact',verdict['error'])
            self.assertEqual(verdict['commands'],[])
            self.assertFalse(verdict['live_exercised'])

    def test_generated_family_abi_rejects_bad_extents_aliases_and_panics(self):
        import sys
        sys.path.insert(0,str(ROOT/'tools'))
        from port_live import generate_family_abi
        with tempfile.TemporaryDirectory() as temporary:
            run=Path(temporary); module=run/'panic.rs'
            module.write_text('use crate::Wram;pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&\'static str>{r.set_byte(1,99);panic!("candidate panic") }')
            source=run/'abi.rs'
            source.write_text(f'#[path={json.dumps(str(ROOT/"crates/port/src/lib.rs"))}] mod memory;use memory::*;\n'+generate_family_abi({'routines':[{'symbol':'PanicControl'}]},module,0))
            library=run/'libabi.a'
            result=subprocess.run(['rustc','--edition=2024','--crate-type=staticlib',str(source),'-o',str(library)],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            caller=run/'control.c'
            caller.write_text('''#include <stdint.h>
#include <stddef.h>
#include <string.h>
static unsigned char ram[0x20000],rom[32];
extern int64_t port_family_0(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);
int main(void){
 memset(ram,165,sizeof(ram));
 if(port_family_0(0,sizeof(ram),rom,sizeof(rom),0,0,0)!=-1)return 1;
 if(port_family_0(ram,1,rom,sizeof(rom),0,0,0)!=-1)return 2;
 if(port_family_0(ram,sizeof(ram),0,sizeof(rom),0,0,0)!=-1)return 3;
 if(port_family_0(ram,sizeof(ram),rom,0,0,0,0)!=-1)return 4;
 if(port_family_0(ram,sizeof(ram),rom,0x400001,0,0,0)!=-1)return 5;
 if(port_family_0(ram,sizeof(ram),ram+1,32,0,0,0)!=-1)return 6;
 if(port_family_0(ram,sizeof(ram),rom,sizeof(rom),999,0,0)!=-1)return 7;
 for(size_t i=0;i<sizeof(ram);i++)if(ram[i]!=165)return 8;
 if(port_family_0(ram,sizeof(ram),rom,sizeof(rom),0,0,0)!=-1)return 9;
 return 0;}
''')
            executable=run/'control'
            result=subprocess.run(['cc',str(caller),str(library),'-ldl','-lpthread','-lm','-o',str(executable)],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            result=subprocess.run([str(executable)],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            self.assertIn('candidate panic',result.stderr)

    def test_offline_incomplete_wrong_candidate_rejects_observed_gameplay(self):
        import os
        if os.environ.get('SM_PORT_LIVE_CONTROLS') != '1':
            self.skipTest('set SM_PORT_LIVE_CONTROLS=1 for compiled gameplay control')
        import sys
        sys.path.insert(0,str(ROOT/'tools'))
        from port_family import verify
        source=Path('/home/jim/sm-decomp/deps/snesrev-sm')
        accepted=Path('/home/jim/sm-decomp/out/port/decompressor-offline/decomp-acceptance.json')
        with tempfile.TemporaryDirectory() as temporary:
            run=Path(temporary);module=run/'wrong.rs'
            # Valid cursor512 is a defined empty loop; this incomplete candidate
            # deliberately omits the visible unused-sprite clearing on live calls.
            module.write_text('use crate::Wram;pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&\'static str>{r.set_word(0x590,0);Ok(0)}')
            manifest=run/'manifest.json'
            manifest.write_text(json.dumps({'family':'wrong-gameplay-control','rust_module':str(module),'bank':'sm_80.c','routines':[{'symbol':'ClearUnusedOam','args':0,'return':'void','cases':[{'name':'empty-loop','writes':[[0x590,[0,2]]]}]}]}))
            previous=verify(manifest,source,run/'offline')
            self.assertEqual(previous['status'],'offline-verified',previous.get('error'))
            script=f'''import sys
sys.path.insert(0,{str(ROOT/'tools')!r})
import port_live
port_live.ROOT=__import__('pathlib').Path('/home/jim/sm-decomp')
import decomp_routine
decomp_routine.ROOT=port_live.ROOT
sys.argv=['port_live','--family-verdict',{str(Path(previous['run'])/'verdict.json')!r},'--accepted',{str(accepted)!r},'--frames','200','--output',{str(run/'live')!r}]
raise SystemExit(port_live.main())
'''
            result=subprocess.run(['python3','-c',script],capture_output=True,text=True,timeout=240)
            self.assertNotEqual(result.returncode,0,result.stdout+result.stderr)
            verdict=json.loads(next((run/'live').glob('*/verdict.json')).read_text())
            self.assertFalse(verdict['live_exercised'])
            self.assertIn('diverged',verdict['error'])

class DispatcherAdmissionControls(unittest.TestCase):
    def test_dispatcher_contract_filters_shim_and_rejects_scheduler(self):
        import sys
        from unittest.mock import patch
        sys.path.insert(0, str(ROOT / 'tools'))
        from port_live import load_dispatcher_contract
        source = {'identity': 'pinned'}
        evidence = {'source': source, 'implementation': {'module': str(ROOT / 'crates/port/src/enemy_projectile_instructions.rs')}}
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            path = run / 'verdict.json'
            routines = [{'symbol': 'EprojInstruction', 'args': 2, 'return': 'rom_ptr'},
                        {'symbol': 'MoveEprojWithVelocityY', 'args': 1, 'return': 'uint16'}]
            manifest = {'bank': 'sm_86.c', 'rom': {}, 'routines': routines}
            (run / 'manifest.json').write_text(json.dumps(manifest))
            with patch('port_plm_instructions.verify_plm_instruction_evidence', return_value=evidence):
                selected, _ = load_dispatcher_contract(path, source)
                self.assertFalse(selected['routines'][0]['live_eligible'])
                self.assertEqual(selected['routines'][1]['symbol'], 'MoveEprojWithVelocityY')
                routines.append({'symbol': 'EprojRunAll', 'args': 0, 'return': 'void'})
                (run / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError, 'support selection'):
                    load_dispatcher_contract(path, source)

    def test_dispatcher_exact_leaf_signatures(self):
        import sys
        from unittest.mock import patch
        sys.path.insert(0, str(ROOT / 'tools'))
        from port_live import DISPATCHER_LEAVES, load_dispatcher_contract
        source = {'identity': 'pinned'}
        evidence = {'source': source, 'implementation': {
            'module': str(ROOT / 'crates/port/src/enemy_projectile_instructions.rs')}}
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            for symbol, returned in DISPATCHER_LEAVES.items():
                with self.subTest(symbol=symbol):
                    routine = {'symbol': symbol, 'args': 1, 'return': returned}
                    manifest = {'bank': 'sm_86.c', 'rom': {}, 'routines': [routine]}
                    with patch('port_plm_instructions.verify_plm_instruction_evidence', return_value=evidence):
                        (run / 'manifest.json').write_text(json.dumps(manifest))
                        selected, _ = load_dispatcher_contract(run / 'verdict.json', source)
                        self.assertEqual(selected['routines'][0]['return'], returned)
                        routine['return'] = 'uint16' if returned == 'void' else 'void'
                        (run / 'manifest.json').write_text(json.dumps(manifest))
                        with self.assertRaisesRegex(ValueError, 'signature differs'):
                            load_dispatcher_contract(run / 'verdict.json', source)
                        routine['return'] = returned
                        routine['arg_types'] = ['uint32']
                        (run / 'manifest.json').write_text(json.dumps(manifest))
                        with self.assertRaisesRegex(ValueError, 'signature differs'):
                            load_dispatcher_contract(run / 'verdict.json', source)

    def test_dispatcher_leaf_abi_compiles_with_gameplay_dependencies(self):
        import sys
        sys.path.insert(0, str(ROOT / 'tools'))
        from port_live import generate_family_abi
        manifest = {'rom': {}, 'routines': [{'symbol': 'EprojInstruction'}, {'symbol': 'MoveEprojWithVelocityY'},
                                            {'symbol': 'Eproj_FuncE73E_MoveXY'}, {'symbol': 'Eproj_FuncE722'}]}
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            source = run / 'leaf.rs'
            source.write_text('#[path=' + json.dumps(str(ROOT / 'crates/port/src/lib.rs')) + '] mod memory;use memory::*;\n'
                              + generate_family_abi(manifest, ROOT / 'crates/port/src/enemy_projectile_instructions.rs', 0)
                              + '''\nfn main(){let mut bytes=vec![0u8;WRAM_SIZE];let rom=[0u8;1];
 {let mut r=Wram::new(&mut bytes).unwrap();r.set_word(0x1a93,10);r.set_word(0x1adb,0x0180);}
 unsafe{assert_eq!(port_family_0(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),1,1,0,0),11);}
 assert_eq!(Wram::new(&mut bytes).unwrap().byte(0x1a70),128);
 let before=bytes.clone();unsafe{assert_eq!(port_family_0(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),1,1,35,0),-1);}
 assert_eq!(bytes,before);
 {let mut r=Wram::new(&mut bytes).unwrap();r.set_word(0x97dc,0);r.set_word(0x1a4b,10);r.set_word(0x1a27,0);r.set_word(0x1a93,20);r.set_word(0x1a6f,0);r.set_word(0x1ab7,0);r.set_word(0x1aff,1);r.set_word(0x1adb,0);r.set_word(0x1b23,1);}
 unsafe{assert_eq!(port_family_0(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),1,2,0,0),0);}
 {let r=Wram::new(&mut bytes).unwrap();assert_eq!(r.word(0x1a4b),10);assert_eq!(r.word(0x1a27),1);assert_eq!(r.word(0x1a93),19);assert_eq!(r.word(0x1a6f),65535);}
 unsafe{assert_eq!(port_family_0(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),1,3,0,0),0);}
 Wram::new(&mut bytes).unwrap().set_word(0x1a4b,512);
 unsafe{assert_eq!(port_family_0(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),1,3,0,0),1);}
 for id in [2,3]{let before=bytes.clone();unsafe{assert_eq!(port_family_0(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),1,id,35,0),-1);}assert_eq!(bytes,before);}}
\n''')
            result = subprocess.run(['rustc', '--edition=2024', str(source), '-o', str(run / 'leaf')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(run / 'leaf')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)


class FinishedOutputDedupControls(unittest.TestCase):
    def test_sealed_sharing_preserves_mismatches_and_rejects_mutation(self):
        import sys
        sys.path.insert(0, str(ROOT / 'tools'))
        from port_live import share_finished_outputs, stream_digest
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            anchor = run / 'anchor'
            (anchor / 'c-0').mkdir(parents=True)
            current = run / 'current'
            current.mkdir()
            for name in ('hardware.log', 'rom-view.bin'):
                (anchor / 'c-0' / name).write_bytes(b'original')
                (current / name).write_bytes(b'original')
            inventory = {name: stream_digest(current / name) for name in ('hardware.log', 'rom-view.bin')}
            verdict = anchor / 'verdict.json'
            (anchor / 'live').write_bytes(b'binary')
            verdict.write_text(json.dumps({'status': 'mixed-C-Rust-integration', 'artifacts': inventory,
                                           'binary_sha256': stream_digest(anchor / 'live')}))
            (current / 'hardware.log').write_bytes(b'mutated!')
            mismatch_inventory = dict(inventory, **{'hardware.log': stream_digest(current / 'hardware.log')})
            self.assertEqual(share_finished_outputs(current, mismatch_inventory, verdict), 8)
            self.assertNotEqual((current / 'hardware.log').stat().st_ino,
                                (anchor / 'c-0' / 'hardware.log').stat().st_ino)
            self.assertEqual((current / 'hardware.log').read_bytes(), b'mutated!')
            with self.assertRaisesRegex(ValueError, 'finished output seal differs'):
                share_finished_outputs(current, inventory, verdict)
            (current / 'hardware.log').write_bytes(b'original')
            self.assertEqual(share_finished_outputs(current, inventory, verdict), 8)
            for name, expected in inventory.items():
                self.assertEqual(stream_digest(current / name), expected)
                self.assertEqual((current / name).stat().st_ino, (anchor / 'c-0' / name).stat().st_ino)
            # Replace the anchor inode instead of mutating any already shared file.
            (anchor / 'c-0' / 'hardware.log').unlink()
            (anchor / 'c-0' / 'hardware.log').write_bytes(b'mutated!')
            with self.assertRaisesRegex(ValueError, 'anchor output seal differs'):
                share_finished_outputs(current, inventory, verdict)
