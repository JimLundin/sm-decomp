"""Retained synthetic policy demonstrations through the public command seam."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class DemonstrationTests(unittest.TestCase):
    def test_policy_command_retains_count_target_stop_and_failed_publication_without_acceptance(self):
        root=Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temporary:
            output=Path(temporary)/'policy'
            command=[sys.executable,str(root/'tools/iteration_demonstrations.py'),'policy','--out',str(output)]
            result=subprocess.run(command,cwd=root,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            report=json.loads((output/'report.json').read_text())
            self.assertEqual(report['scope'],'session policy simulation; acceptance not certified')
            self.assertEqual(report['production_accepted_count'],0)
            self.assertEqual(report['cases']['count2']['accepted_count'],2)
            self.assertEqual(report['cases']['failed-publication']['accepted_count'],0)
            self.assertEqual(report['cases']['user-stop']['status'],'stopped')
            self.assertTrue((output/'count2'/'out/iteration-sessions').is_dir())
            result=subprocess.run(command,cwd=root,capture_output=True,text=True)
            self.assertEqual(result.returncode,2)

    def test_probe_verifier_rejects_forged_embedded_report_and_snapshot(self):
        from iteration_demonstrations import verify_probe
        from readiness import digest,write_json
        with tempfile.TemporaryDirectory() as temporary:
            output=Path(temporary);binary=output/'probe';binary.write_bytes(b'preserved binary')
            (output/'fixture.sfc').write_bytes(b'rom');(output/'fixture.inputs').write_bytes(b'0000\n')
            report={'status':'pass','complete':True,'accepted_count':0,'source_sha256':'a'*64,'binary_sha256':digest(binary),
                'rom_sha256':digest(output/'fixture.sfc'),'input_sha256':digest(output/'fixture.inputs')}
            for name in ('early','candidate','predecessor','disabled','warm'):
                (output/name).mkdir();value={'status':'pass','complete':True,'start_input_index':0,'start_instruction':0,'completed_instructions':87}
                write_json(output/name/'instruction.json',value);report[name]=value
            report['predecessor']=dict(report['predecessor'],start_instruction=999)
            with self.assertRaisesRegex(ValueError,'embedded report differs'):
                verify_probe(report,output,'a'*64,binary)

    def test_private_failure_history_rejects_publication_and_preserves_three_candidates(self):
        from iteration_demonstrations import failed_lineage
        from iteration_diagnosis import verify_history
        with tempfile.TemporaryDirectory() as temporary:
            output=Path(temporary)/'lineage';report=failed_lineage(output)
            self.assertEqual(report['publication_returncode'],2)
            self.assertEqual(report['pointer_before_sha256'],report['pointer_after_sha256'])
            self.assertEqual(len(report['candidates']),3)
            history=verify_history(report['history_root'],'synthetic-increment')
            self.assertEqual([row['consecutive_without_evidence'] for row in history],[1,2,3])
            self.assertEqual([row['decision_required'] for row in history],[False,False,True])


if __name__=='__main__':unittest.main()
