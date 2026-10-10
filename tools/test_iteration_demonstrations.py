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


if __name__=='__main__':unittest.main()
