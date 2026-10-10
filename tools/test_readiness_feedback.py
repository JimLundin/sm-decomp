"""Real emitted execution and immutable evidence at the feedback gate seam."""
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from readiness import Run, digest
from readiness_feedback import CASES, run_demonstrations, verify_feedback


class FeedbackTests(unittest.TestCase):
    def test_real_fault_controls_preserve_origins_and_refuse_altered_evidence(self):
        root=Path(__file__).resolve().parent.parent
        # The parent source is the real maintained tree; all mutated fault
        # candidates, generated inputs and build output are private.
        run=Run.create(root,'feedback-test',{'synthetic_cases':len(CASES)})
        self.addCleanup(shutil.rmtree,run.path)
        run.capture_source()
        run.manifest['identities']['hardware']={'source_sha256':'a'*64}
        run.save_manifest()
        report=run_demonstrations(run,timeout=120)
        self.assertEqual(report['status'],'pass',json.dumps(report,indent=2))
        self.assertEqual(report['completed_cases'],len(CASES))
        self.assertTrue(verify_feedback(run.path,run.manifest,report))
        self.assertFalse((run.path/'out/accepted.json').exists())
        transient=report['cases']['transient-effect']
        path=run.path/transient['evidence']['path']
        actual=json.loads(path.read_text())
        self.assertTrue(actual['frame_end_equal'])
        self.assertEqual(actual['first_mismatch']['subject']['generated_key'],[0x8004,3])
        actual['first_mismatch']['instruction']=4
        path.write_text(json.dumps(actual))
        transient['evidence']['sha256']=digest(path)
        with self.assertRaisesRegex(ValueError,'earliest'):
            verify_feedback(run.path,run.manifest,report)

    def test_public_lifecycle_controls_have_expected_failures_and_no_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            output=Path(temporary)/'controls'
            result=subprocess.run(['python3',str(Path(__file__).with_name('readiness_feedback.py')),
                                   '--case','incompatible-stale','--out',str(output)],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            report=json.loads((output/'report.json').read_text())
            self.assertEqual(set(report['observations']),{'stale','identity','source','input'})
            self.assertTrue(all(item['accept_returncode']==2 and item['pointer_unchanged']
                                for item in report['observations'].values()))


if __name__=='__main__':unittest.main()
