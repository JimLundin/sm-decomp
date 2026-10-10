"""Selection command and its identity-bound instruction profile evidence seam."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from readiness import Run, digest, write_json


class SelectionTests(unittest.TestCase):
    def test_invalid_baseline_is_preserved_as_error_without_changing_acceptance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'out').mkdir()
            pointer = root/'out/accepted.json'
            pointer.write_text('{"run_path":"/missing/baseline","run_id":"bad"}')
            before = pointer.read_bytes()
            command = subprocess.run([sys.executable,str(Path(__file__).with_name('iteration_selection.py')),
                '--root',str(root)],capture_output=True,text=True)
            self.assertEqual(command.returncode,2,command.stderr)
            self.assertEqual(pointer.read_bytes(),before)
            runs = list((root/'out/runs').iterdir())
            self.assertEqual(len(runs),1)
            verdict = json.loads((runs[0]/'verdict.json').read_text())
            self.assertEqual(verdict['status'],'error')
            self.assertIn('readiness',verdict['reason'].lower())


class SelectionEvidenceTests(unittest.TestCase):
    def test_selection_cannot_certify_invented_gain_or_an_unranked_choice(self):
        from iteration_selection import verify_selection_result
        report = {'schema_version':1,'outcome':'supported-routine','chosen':{'address':0x808000,'mx':3,
            'variant':'m1x1','estimated_compiled_step_gain':100,'new_executed_keys':[
                {'address':0x808000,'mx':3,'opcode':0xea,'bytes':[0xea],'count':7}],
            'deferred_keys':[],'overlapping_keys':[]},'ranked':[], 'observed_unsupported_patterns':[]}
        with self.assertRaisesRegex(ValueError,'gain|ranked'):
            verify_selection_result(report)


class ProfileRefreshTests(unittest.TestCase):
    def test_missing_instruction_counts_are_refreshed_by_exact_preserved_executable(self):
        from iteration_selection import obtain_profile
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            run = Run.create(root,'iteration-selection',{'domain':'Japan/USA any%','frames':18000})
            run.capture_source()
            owner = root/'baseline'
            (owner/'generated').mkdir(parents=True)
            (owner/'generated/movie.inputs').write_text('0000\n'*18000)
            inputs = {'rom_sha256':'a'*64,'movie_sha256':'b'*64,'input_sha256':digest(owner/'generated/movie.inputs'),
                'input_evidence':'generated/movie.inputs','frames':18000,'seed':0,'profile':'lsnes-profile'}
            binary = owner/'generated/diff'
            # External executable boundary emits a complete known replay, not an
            # internal collaborator mock. No ROM material is needed by this fixture.
            binary.write_text("#!/usr/bin/env python3\nimport hashlib,json,sys\nfrom pathlib import Path\n"
                "out=Path(sys.argv[sys.argv.index('--out')+1]);out.mkdir(parents=True)\n"
                "profile=" + repr({'schema_version':1,'compiled':[], 'interpreted':[{'address':0x808000,'mx':3,'opcode':0xea,'emulation':False,'pattern':'NOP/Imp','count':7}], 'generated_instructions':0,'interpreted_instructions':7,'generated_dispatches':0,'interpreted_dispatches':9,'noninstruction_dispatches':2}) + "\n"
                "(out/'coverage.json').write_text(json.dumps(profile))\n"
                "report=" + repr({'status':'pass','complete':True,'mode':'generated','expected_frames':18000,'completed_frames':18000,
                    'matched_frames':18000,'available_frames':18000,'seed':0,'profile':'lsnes-profile','controller':'P1',
                    'regions':['CPU+clocks','WRAM','framebuffer'],'generated_steps':0,'interpreted_steps':9,
                    'identities':{'rom_sha256':'a'*64,'compiled_rom_sha256':'a'*64,'movie_sha256':'b'*64,
                        'input_sha256':inputs['input_sha256'],'selected_input_sha256':inputs['input_sha256']}}) + "\n"
                "report['identities']['binary_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()\n"
                "report['coverage_sha256']=hashlib.sha256((out/'coverage.json').read_bytes()).hexdigest()\n"
                "(out/'parity.json').write_text(json.dumps(report))\n")
            binary.chmod(0o755)
            (owner/'parity.json').write_text('{}')
            manifest = {'run_id':'accepted-fixture','identities':{'inputs':inputs,'source':{'sha256':'c'*64},
                'generated-binary':{'sha256':digest(binary),'evidence':'generated/diff'}}}
            profile = obtain_profile(run,owner,{'manifest':manifest,'verdict':{'checks':{'generated-parity':{'evidence':'parity.json'}}}})
            self.assertEqual(json.loads(profile.read_text())['interpreted_instructions'],7)
            self.assertEqual(run.checks['selection-profile']['baseline_run_id'],'accepted-fixture')
            self.assertEqual(run.manifest['selection_profile_origin'],'fresh replay of preserved accepted executable')
            self.assertEqual(json.loads((run.path/'logs/baseline-profile-refresh.json').read_text())['returncode'],0)
            # Digest-valid counts still cannot override the independently reported
            # instruction totals or use a different input identity.
            from iteration_selection import bind_profile
            raw_path = run.path/'diagnostics/baseline-profile-refresh/parity.json'
            profile_path = raw_path.parent/'coverage.json'
            raw = json.loads(raw_path.read_text())
            bad_profile = json.loads(profile_path.read_text())
            bad_profile['interpreted_instructions'] = 8
            write_json(profile_path,bad_profile)
            raw['coverage_sha256'] = digest(profile_path)
            write_json(raw_path,raw)
            other = Run.create(root,'iteration-selection',{'frames':18000})
            with self.assertRaisesRegex(ValueError,'totals'):
                bind_profile(other,owner,manifest,raw_path,profile_path)
            bad_profile['interpreted_instructions'] = 7
            write_json(profile_path,bad_profile)
            raw['coverage_sha256'] = digest(profile_path)
            raw['identities']['input_sha256'] = '0'*64
            write_json(raw_path,raw)
            with self.assertRaisesRegex(ValueError,'identity-bound'):
                bind_profile(other,owner,manifest,raw_path,profile_path)


if __name__ == '__main__':
    unittest.main()
