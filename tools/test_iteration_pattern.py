"""Real command/evidence behavior for prospective bodies and guarded promotion."""
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class ProspectivePatternTests(unittest.TestCase):
    def test_prospective_ad_executes_without_enabling_production_and_retains_proof(self):
        root=Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary);corpus=directory/'corpus';corpus.mkdir()
            case=json.loads((root/'deps/65816/v1/ad.n.json').read_text())[0]
            (corpus/'ad.n.json').write_text(json.dumps([case]))
            command=[sys.executable,str(root/'tools/readiness_patterns.py'),'--corpus',str(corpus),
                '--prospective','ad','--out',str(directory/'proof')]
            result=subprocess.run(command,cwd=root,capture_output=True,text=True,
                env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0'))
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            report=json.loads((directory/'proof/instruction.json').read_text())
            self.assertEqual((report['expected'],report['passed'],report['failed'],report['skipped']),(9,9,0,0))
            self.assertEqual(report['prospective_opcode'],'ad')
            declaration=json.loads((root/'crates/recomp/src/timed_patterns.json').read_text())
            self.assertNotIn('ad',[row['opcode'] for row in declaration['patterns']])
            (directory/'manifest.json').write_text(json.dumps({'identities':{'source':{'sha256':report['source_sha256']},'compiler':{'version':report['compiler']}}}))
            result=subprocess.run([sys.executable,str(root/'tools/readiness_orchestration.py'),'verify-patterns',
                '--run',str(directory),'--output',str(directory/'proof'),'--development-scope'],cwd=root,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            case['final']['a'] ^= 1
            (corpus/'ad.n.json').write_text(json.dumps([case]));command[-1]=str(directory/'bad')
            result=subprocess.run(command,cwd=root,capture_output=True,text=True,
                env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0'))
            self.assertEqual(result.returncode,1,result.stdout+result.stderr)

    def test_promotion_without_complete_proof_preserves_source(self):
        root=Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary);(directory/'selection').mkdir();(directory/'proof').mkdir()
            declaration=root/'crates/recomp/src/timed_patterns.json';before=declaration.read_bytes()
            result=subprocess.run([sys.executable,str(root/'tools/iteration_pattern.py'),'promote',
                '--selection',str(directory/'selection'),'--proof',str(directory/'proof')],cwd=root,capture_output=True,text=True)
            self.assertEqual(result.returncode,2)
            self.assertEqual(declaration.read_bytes(),before)


if __name__=='__main__':unittest.main()
