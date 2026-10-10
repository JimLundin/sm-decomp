"""Actual-C synchronous callback at the agreed canonical WRAM seam."""
import json
from pathlib import Path
import tempfile
import unittest

import port_collision

SOURCE = Path('/home/jim/sm-decomp/deps/snesrev-sm')
ROM = Path('/home/jim/sm-decomp/rom/sm.sfc')

class CollisionCallbackTests(unittest.TestCase):
    def test_null_context_generator_uses_distinct_callback_signature(self):
        generated = port_collision.generate_collision_abi(
            {'plm_amount_context': 'none', 'routines': [{'symbol': 'Flight'}]}, Path('/tmp/flight.rs'))
        self.assertIn('fn sm_rust_collision_spawn_plm_without_amount(', generated)
        self.assertIn("header:u16)->Result<u8", generated)
        self.assertNotIn('amount:&mut i32', generated)
        with self.assertRaisesRegex(ValueError, 'amount context'):
            port_collision.generate_collision_abi({'plm_amount_context': 'unknown'}, Path('/tmp/flight.rs'))

    def test_actual_quicksand_callback_mutates_amount_and_full_ram_and_rejects_control(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            module = tmp / 'probe.rs'
            module.write_text('''use crate::Wram;
pub trait Host {fn spawn_plm(&mut self,ram:&mut Wram<'_>,header:u16,amount:&mut i32)->Result<u8,&'static str>;}
pub fn dispatch_host(ram:&mut Wram, _: &[u8],host:&mut impl Host,_:&str,a:i32,b:i32)->Result<u32,&'static str>{
let mut amount=a;host.spawn_plm(ram,b as u16,&mut amount)?;Ok(amount as u32)}
''')
            manifest = tmp / 'probe.json'
            config = {'family':'collision-callback-probe','infrastructure_probe':True,'rust_module':str(module),
                      'rom':{'file':str(ROM),'sha256':port_collision.digest(ROM)},
                      'routines':[{'symbol':'CollisionCallbackProbe','arg_types':['int32','uint16'],'return':'uint32',
                                   'cases':[{'name':'quicksand-clamp','a':65536,'b':0xb72b,
                                             'writes':[[0xb02,[3,0]]],'expected_return':0x3000,'minimum_callbacks':1}]}]}
            manifest.write_text(json.dumps(config))
            verdict = port_collision.verify(manifest,SOURCE,tmp/'good')
            self.assertEqual(verdict['status'],'offline-verified',verdict.get('error'))
            self.assertEqual(verdict['cases'][0]['callbacks'],1)
            self.assertFalse(verdict['gameplay_translation'])
            evidence=Path(verdict['run'])/'verdict.json'
            port_collision.verify_collision_evidence(evidence)
            original_evidence=evidence.read_text()
            for filename in ('before.wram','c-after.wram','rust-after.wram','c.stdout','rust.stdout'):
                incomplete=json.loads(original_evidence)
                del incomplete['artifacts']['quicksand-clamp/'+filename]
                evidence.write_text(json.dumps(incomplete))
                with self.subTest(missing=filename):
                    with self.assertRaisesRegex(ValueError,'case artifact.*incomplete'):
                        port_collision.verify_collision_evidence(evidence)
            evidence.write_text(original_evidence)
            callback=Path(verdict['run'])/'quicksand-clamp'/'rust'/'callbacks.txt'
            original=callback.read_bytes();callback.write_bytes(b'')
            with self.assertRaisesRegex(ValueError,'artifact'):
                port_collision.verify_collision_evidence(evidence)
            callback.write_bytes(original)
            module.write_text(module.read_text().replace('Ok(amount as u32)','ram.set_byte(0x10000,1);Ok(amount as u32)'))
            verdict = port_collision.verify(manifest,SOURCE,tmp/'wrong')
            self.assertEqual(verdict['status'],'rejected')
            self.assertIn('WRAM mismatch',verdict['error'])
            self.assertTrue((Path(verdict['run'])/'quicksand-clamp'/'c-after.wram').exists())
