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
