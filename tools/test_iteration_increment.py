"""Supported increment at the approved immutable command/evidence boundaries."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from readiness import Run


class IncrementProgressTests(unittest.TestCase):
    def test_pruning_cannot_remove_an_unexecuted_legacy_body(self):
        from iteration_increment import prune_uncovered
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            failed=Run.create(root,'iteration',{'domain':'Japan/USA any%','frames':18000})
            failed.capture_source()
            failed.record_check('changed-entry-coverage',{'status':'uncovered','complete':True,
                'required_keys':[{'address':0x808000,'mx':3,'reason':'body changed'}],
                'uncovered_keys':[{'address':0x808000,'mx':3}]})
            failed.finalize('uncovered')
            with self.assertRaisesRegex(ValueError,'Only newly added'):
                prune_uncovered(root,failed.path)
            retries=[path for path in (root/'out/runs').iterdir() if path!=failed.path]
            self.assertEqual(len(retries),1)
            self.assertEqual(json.loads((retries[0]/'verdict.json').read_text())['status'],'error')

    def test_correctness_fix_cannot_self_certify_zero_gain(self):
        from iteration_increment import progress_report
        report=progress_report({'status':'pass','complete':True,'progress':{'comparable':True,'generated_delta':0}},
            {'kind':'correctness-fix','rationale':'repair','reproduction':{'status':'pass','complete':True,'evidence':'made-up.json','sha256':'a'*64}})
        self.assertEqual(report['status'],'uncovered')
        self.assertIn('verified',report['reason'])

    def test_expansion_cannot_accept_zero_gain_even_if_coverage_passed(self):
        from iteration_increment import progress_gate
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            run = Run.create(root,'iteration',{'domain':'Japan/USA any%','frames':18000})
            run.capture_source()
            run.manifest['increment'] = {'kind':'supported-expansion'}
            run.record_check('changed-entry-coverage',{'status':'pass','complete':True,
                'required_keys':[{'address':0x808000,'mx':3,'reason':'added'}],
                'progress':{'comparable':True,'generated_delta':0,'interpreted_delta':0},
                'generated_instructions':10,'interpreted_instructions':90})
            report = progress_gate(run)
            self.assertEqual(report['status'],'uncovered')
            self.assertIn('positive',report['reason'])
            self.assertEqual(run.finalize('pass',mandatory_checks=['increment-progress']),1)
            self.assertFalse(json.loads((run.path/'verdict.json').read_text())['acceptance_eligible'])


class PreparationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.repository=Path(__file__).resolve().parent.parent
        cls.target=tempfile.TemporaryDirectory(prefix='increment-generator-test-')
        environment=dict(__import__('os').environ,CARGO_TARGET_DIR=cls.target.name,CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0')
        subprocess.run(['cargo','build','-p','recomp','--locked','--offline'],cwd=cls.repository,env=environment,check=True,capture_output=True)
        cls.generator=Path(cls.target.name)/'debug/recomp'

    @classmethod
    def tearDownClass(cls):
        cls.target.cleanup()

    def test_measured_revisit_lifts_only_selected_exclusion_and_preserves_reviewed_variants(self):
        from iteration_increment import propose_candidate
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'config').mkdir();config=root/'config/functions.toml'
            config.write_text('# Reviewed routine\n[[function]]\naddr = "808000"\nname = "Existing"\nvariants = ["m1x1", "m0x0"]\n'
                '\n# Now observed key\n[[excluded_step]]\naddr = "808001"\nvariant = "m1x1"\nreason = "deferred unexecuted proposed key"\n'
                '\n# Keep this deferred key\n[[excluded_step]]\naddr = "808002"\nvariant = "m1x1"\nreason = "deferred unexecuted proposed key"\n')
            before=config.read_bytes()
            run=Run.create(root,'increment-preparation',{'domain':'fixture','frames':2});run.capture_source()
            rom=bytearray([0xff])*(3*1024*1024);rom[:4]=bytes([0xea,0xca,0xea,0x60])
            (root/'rom').mkdir();(root/'rom/sm.sfc').write_bytes(rom)
            (run.path/'generated/selection.json').write_text(json.dumps({'chosen':{
                'address':0x808000,'mx':3,'variant':'m1x1','action':'enable-deferred-keys',
                'new_executed_keys':[{'address':0x808001,'mx':3,'opcode':0xca,'bytes':[0xca],'count':20}]}}))
            baseline={'keys':[{'address':0x808000,'mx':3,'opcode':0xea,'bytes':[0xea]},
                {'address':0x808000,'mx':0,'opcode':0xea,'bytes':[0xea]},
                {'address':0x808001,'mx':0,'opcode':0xca,'bytes':[0xca]},
                {'address':0x808002,'mx':0,'opcode':0xea,'bytes':[0xea]}]}
            proposal=propose_candidate(run,baseline,self.generator)
            self.assertEqual(config.read_bytes(),before)
            self.assertIn('variants = ["m1x1", "m0x0"]',proposal.read_text())
            self.assertIn('# Now observed key',proposal.read_text())
            self.assertIn('# Keep this deferred key',proposal.read_text())
            self.assertNotIn('addr = "808001"',proposal.read_text())
            self.assertIn('addr = "808002"',proposal.read_text())
            table=(run.path/'generated/proposed-steps.rs').read_text()
            self.assertIn('step_808001_3',table)
            self.assertNotIn('step_808002_3',table)

    def test_preparation_keeps_root_untouched_and_physically_prunes_unselected_keys(self):
        from iteration_increment import propose_candidate
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'config').mkdir()
            config=root/'config/functions.toml'
            config.write_text('# User comment retained\n[[function]]\naddr = "808020"\nname = "Legacy"\nvariants = ["m1x1"]\n')
            before=config.read_bytes()
            run=Run.create(root,'increment-preparation',{'domain':'fixture','frames':2})
            run.capture_source()
            # Synthetic LoROM has NOP/DEX/RTS in new routine and a legacy NOP.
            rom=bytearray([0xff])*(3*1024*1024)
            rom[:3]=bytes([0xea,0xca,0x60]);rom[32:34]=bytes([0xea,0x60])
            (root/'rom').mkdir();(root/'rom/sm.sfc').write_bytes(rom)
            (run.path/'generated/selection.json').write_text(json.dumps({'chosen':{'address':0x808000,'mx':3,'variant':'m1x1',
                'new_executed_keys':[{'address':0x808000,'mx':3,'opcode':0xea,'bytes':[0xea],'count':2}],
                'deferred_keys':[{'address':0x808001,'mx':3,'opcode':0xca,'bytes':[0xca],'count':0}]}}))
            baseline={'keys':[{'address':0x808020,'mx':3,'opcode':0xea,'bytes':[0xea]}]}
            generator=self.generator
            proposal=propose_candidate(run,baseline,generator)
            self.assertEqual(config.read_bytes(),before)
            self.assertIn('User comment retained',proposal.read_text())
            table=(run.path/'generated/proposed-steps.rs').read_text()
            self.assertIn('step_808000_3',table)
            self.assertIn('step_808020_3',table)
            self.assertNotIn('step_808001_3',table)
            manifest=json.loads((run.path/'checks/candidate-proposal.json').read_text())
            self.assertEqual(manifest['deferred'][0]['address'],0x808001)
            self.assertEqual(manifest['actual_new_keys'],[{'address':0x808000,'mx':3}])


if __name__ == '__main__':
    unittest.main()
