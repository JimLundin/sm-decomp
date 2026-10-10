"""Compiled upstream C is the independent oracle at the approved full-WRAM seam."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path('/home/jim/sm-decomp/deps/snesrev-sm')

class FamilyCliTests(unittest.TestCase):
    def test_compares_real_bank_and_rejects_wrong_rust_with_retained_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            module = run / 'rng.rs'
            module.write_text(f'''#[path = {json.dumps(str(ROOT / 'tools/decomp/next_random.rs'))}] mod rng;
use crate::Wram;
pub fn dispatch(ram: &mut Wram, _: &str, _: u16, _: u16) -> Result<u32, &'static str> {{
let seed=ram.word(0x5e5); let mut bytes=vec![0;0x20000]; bytes[0x5e5..0x5e7].copy_from_slice(&seed.to_le_bytes());
let value=rng::next_random(&mut bytes)?; ram.set_word(0x5e5,value); Ok(value.into()) }}
''')
            manifest = run / 'family.json'
            manifest.write_text(json.dumps({'family':'rng-control','rust_module':str(module),'bank':'sm_80.c','routines':[{'symbol':'NextRandom','args':0,'return':'uint16','cases':[{'name':'known-zero','fill':165,'writes':[[0x5e5,[0,0]]],'expected_return':273}]}]}))
            def execute(output):
                result = subprocess.run(['python3',str(ROOT/'tools/port_family.py'),'verify','--manifest',str(manifest),'--source',str(SOURCE),'--output',str(output)],capture_output=True,text=True)
                verdict_path = next(output.glob('*/verdict.json'))
                return result, json.loads(verdict_path.read_text()), verdict_path.parent
            result, verdict, _ = execute(run/'good')
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            self.assertEqual(verdict['status'],'offline-verified')
            module.write_text(module.read_text().replace('Ok(value.into())','Ok(u32::from(value)^1)'))
            result, verdict, evidence = execute(run/'bad')
            self.assertNotEqual(result.returncode,0)
            self.assertEqual(verdict['status'],'rejected')
            self.assertIn('return',verdict['error'])
            self.assertTrue((evidence/'known-zero'/'c-after.wram').exists())
            self.assertTrue((evidence/'known-zero'/'rust-after.wram').exists())
            module.write_text(module.read_text().replace('Ok(u32::from(value)^1)', 'ram.set_byte(0x10000,0); Ok(value.into())'))
            result, verdict, evidence = execute(run/'bad-memory')
            self.assertNotEqual(result.returncode,0)
            self.assertIn('WRAM mismatch',verdict['error'])
            self.assertEqual(verdict['cases'][0]['first_mismatch']['offset'],0x10000)

    def test_inventory_records_real_definitions_addresses_and_calls_without_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'inventory.json'
            result = subprocess.run(['python3',str(ROOT/'tools/port_family.py'),'inventory','--source',str(SOURCE),'--output',str(output)],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            inventory = json.loads(output.read_text())
            routines = {r['symbol']:r for r in inventory['routines']}
            self.assertEqual(routines['NextRandom']['original_address'],'0x808111')
            self.assertIn('Samus_HandleAnimDelay',routines['Samus_Animate']['calls'])
            self.assertTrue(all(r['translation_status']=='unverified' for r in inventory['routines']))
            self.assertGreater(len(inventory['routines']),3000)

    def test_rejecting_candidate_domain_stops_before_unchecked_c_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            run=Path(directory)
            module=run/'reject.rs'
            module.write_text("use crate::Wram; pub fn dispatch(_: &mut Wram,_:&str,_:u16,_:u16)->Result<u32,&'static str>{Err(\"invalid indexed state\")}")
            manifest=run/'family.json'
            manifest.write_text(json.dumps({'family':'invalid-domain','rust_module':str(module),'bank':'sm_80.c','routines':[{'symbol':'SetBossBitForCurArea','args':1,'return':'void','cases':[{'name':'bad-area','writes':[[0x79f,[255,255]]]}]}]}))
            result=subprocess.run(['python3',str(ROOT/'tools/port_family.py'),'verify','--manifest',str(manifest),'--source',str(SOURCE),'--output',str(run/'out')],capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            evidence=next((run/'out').iterdir())
            self.assertTrue((evidence/'bad-area'/'rust.stderr').exists())
            self.assertFalse((evidence/'bad-area'/'c-after.wram').exists())

    def test_capture_digest_and_expected_full_memory_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            run=Path(directory)
            module=run/'same.rs'
            module.write_text("use crate::Wram; pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&'static str>{r.set_word(0x5e5,273);Ok(273)}")
            before=run/'before.wram'; before.write_bytes(bytes(0x20000))
            after=run/'after.wram'; data=bytearray(before.read_bytes());data[1509:1511]=(273).to_bytes(2,'little');after.write_bytes(data)
            import hashlib
            manifest=run/'family.json'
            case={'name':'captured','before_file':str(before),'before_sha256':hashlib.sha256(before.read_bytes()).hexdigest(),'expected_after_file':str(after),'expected_after_sha256':hashlib.sha256(after.read_bytes()).hexdigest(),'expected_return':273}
            config={'family':'capture','rust_module':str(module),'bank':'sm_80.c','routines':[{'symbol':'NextRandom','args':0,'return':'uint16','cases':[case]}]}
            def execute(label):
                manifest.write_text(json.dumps(config))
                result=subprocess.run(['python3',str(ROOT/'tools/port_family.py'),'verify','--manifest',str(manifest),'--source',str(SOURCE),'--output',str(run/label)],capture_output=True,text=True)
                return result, json.loads(next((run/label).glob('*/verdict.json')).read_text())
            result, verdict=execute('good'); self.assertEqual(result.returncode,0,result.stdout)
            case['before_sha256']='0'*64
            result, verdict=execute('wrong-digest');self.assertNotEqual(result.returncode,0);self.assertIn('identity',verdict['error'])

    def test_signed_argument_and_rom_input_execute_actual_support_source(self):
        with tempfile.TemporaryDirectory() as directory:
            run=Path(directory)
            module=run/'signed.rs'
            # SnesDivide is actual pinned sm_rtl.c and independently 65535/2=32767.
            module.write_text("use crate::Wram; pub fn dispatch_rom(_: &mut Wram,rom:&[u8],_:&str,a:i32,b:i32)->Result<u32,&'static str>{if rom.len()!=0x300000{return Err(\"ROM extent\")};Ok((a as u16/b as u16).into())}")
            rom=run/'rom.bin';rom.write_bytes(bytes(0x300000))
            import hashlib
            manifest=run/'family.json'
            manifest.write_text(json.dumps({'family':'signed-rom','rust_module':str(module),'bank':'sm_rtl.c','c_support':['sm_rtl.c'],'rom':{'file':str(rom),'sha256':hashlib.sha256(rom.read_bytes()).hexdigest()},'routines':[{'symbol':'SnesDivide','arg_types':['uint16','uint8'],'return':'uint16','cases':[{'name':'negative-raw-word','a':65535,'b':2,'expected_return':32767}]}]}))
            result=subprocess.run(['python3',str(ROOT/'tools/port_family.py'),'verify','--manifest',str(manifest),'--source',str(SOURCE),'--output',str(run/'out')],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)

    def test_negative_movement_argument_preserves_raw_signed_bits(self):
        with tempfile.TemporaryDirectory() as directory:
            run=Path(directory)
            module=run/'move.rs'
            module.write_text("use crate::Wram; pub fn dispatch_i32(r:&mut Wram,_:&str,a:i32,_:i32)->Result<u32,&'static str>{r.set_word(0xaf6,(a as u32>>16) as u16);r.set_word(0xaf8,a as u16);r.set_word(0xdae,(a as u32>>16) as u16);r.set_word(0xdb0,a as u16);Ok(0)}")
            manifest=run/'family.json'
            manifest.write_text(json.dumps({'family':'signed-movement','rust_module':str(module),'bank':'sm_90.c','routines':[{'symbol':'Samus_MoveRight_NoColl','arg_types':['int32'],'return':'void','cases':[{'name':'negative-one-fraction','a':-1,'expected_return':0}]}]}))
            result=subprocess.run(['python3',str(ROOT/'tools/port_family.py'),'verify','--manifest',str(manifest),'--source',str(SOURCE),'--output',str(run/'out')],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)

    def test_live_consumer_rejects_tampered_retained_family_outputs(self):
        import sys
        sys.path.insert(0,str(ROOT/'tools'))
        from port_family import verify, verify_family_evidence
        with tempfile.TemporaryDirectory() as directory:
            run=Path(directory); module=run/'same.rs'
            module.write_text("use crate::Wram; pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&'static str>{r.set_word(0x5e5,273);Ok(273)}")
            manifest=run/'family.json'
            manifest.write_text(json.dumps({'family':'auth','rust_module':str(module),'bank':'sm_80.c','routines':[{'symbol':'NextRandom','args':0,'return':'uint16','cases':[{'name':'known','expected_return':273}]}]}))
            verdict=verify(manifest,SOURCE,run/'out'); evidence=Path(verdict['run'])
            verify_family_evidence(evidence/'verdict.json')
            (evidence/'known'/'c-after.wram').write_bytes(bytes(0x20000))
            with self.assertRaisesRegex(ValueError,'artifact'):
                verify_family_evidence(evidence/'verdict.json')

    def test_imported_module_identity_is_required_for_family_acceptance(self):
        import sys
        sys.path.insert(0,str(ROOT/'tools'))
        from port_family import verify, verify_family_evidence
        with tempfile.TemporaryDirectory() as directory:
            run=Path(directory)
            dependency=run/'logic.rs';dependency.write_text('pub fn value()->u32{273}')
            module=run/'family.rs';module.write_text(f'#[path={json.dumps(str(dependency))}]mod logic;use crate::Wram;pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&\'static str>{{r.set_word(0x5e5,logic::value() as u16);Ok(logic::value())}}')
            manifest=run/'manifest.json';manifest.write_text(json.dumps({'family':'dependencies','rust_module':str(module),'rust_dependencies':[str(dependency)],'bank':'sm_80.c','routines':[{'symbol':'NextRandom','args':0,'return':'uint16','cases':[{'name':'known','expected_return':273}]}]}))
            verdict=verify(manifest,SOURCE,run/'out');self.assertEqual(verdict['status'],'offline-verified',verdict.get('error'))
            dependency.write_text('pub fn value()->u32{274}')
            with self.assertRaisesRegex(ValueError,'dependency'):
                verify_family_evidence(Path(verdict['run'])/'verdict.json')
