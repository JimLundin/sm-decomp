"""Public readiness commands and recoverable evidence, using real filesystem/processes."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from readiness import Run


class ReadinessCommandTests(unittest.TestCase):
    def test_missing_setup_retains_every_mandatory_gate_and_cannot_accept(self):
        from readiness_contract import MANDATORY_CHECKS
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'README.md').write_text('fixture source\n')
            subprocess.run(['git','-C',str(root),'add','README.md'],check=True)
            command=[sys.executable,str(Path(__file__).with_name('readiness_orchestration.py')),'readiness','--root',str(root)]
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(result.returncode,2,result.stdout+result.stderr)
            runs=list((root/'out/runs').iterdir())
            self.assertEqual(len(runs),1)
            verdict=json.loads((runs[0]/'verdict.json').read_text())
            self.assertEqual(set(verdict['mandatory_checks']),set(MANDATORY_CHECKS))
            self.assertEqual(set(MANDATORY_CHECKS)-set(verdict['checks']),set())
            self.assertFalse(verdict['acceptance_eligible'])
            self.assertFalse((root/'out/accepted.json').exists())
            self.assertIn('Brief:',result.stdout)

    def test_restore_selected_snapshot_never_overwrites_existing_work(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)/'repository';root.mkdir()
            subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'README.md').write_text('known source\n')
            subprocess.run(['git','-C',str(root),'add','README.md'],check=True)
            run=Run.create(root,'diagnostic',{'instructions':1});run.capture_source()
            run.finalize('incomplete','fixture','selected candidate interrupted')
            destination=Path(temporary)/'recovered'
            command=[sys.executable,str(Path(__file__).with_name('readiness_orchestration.py')),'restore','--run',str(run.path),'--destination',str(destination)]
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            self.assertEqual((destination/'README.md').read_text(),'known source\n')
            (destination/'README.md').write_text('user changes\n')
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(result.returncode,2)
            self.assertEqual((destination/'README.md').read_text(),'user changes\n')

    def test_incomplete_run_cannot_replace_an_existing_accepted_pointer(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            run=Run.create(root,'readiness',{'domain':'Japan/USA any%','frames':18000});run.capture_source()
            run.finalize('incomplete','feedback-demonstrations','missing mandatory demonstrations')
            pointer=root/'out/accepted.json';pointer.write_text('{"prior":"must remain"}\n');before=pointer.read_bytes()
            command=[sys.executable,str(Path(__file__).with_name('readiness_orchestration.py')),'accept','--run',str(run.path)]
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(result.returncode,2)
            self.assertEqual(pointer.read_bytes(),before)


class TruncatedEvidenceTests(unittest.TestCase):
    def test_validated_labels_without_terminal_registry_cannot_accept(self):
        from readiness_contract import MANDATORY_CHECKS
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            run=Run.create(root,'readiness',{'domain':'Japan/USA any%','frames':18000});run.capture_source()
            run.record_check('preflight',{'status':'pass','complete':True,'failures':[]})
            # A positive setup claim cannot invent the remaining mandatory work.
            run.finalize('pass','fixture','setup claim only',['preflight'])
            command=[sys.executable,str(Path(__file__).with_name('readiness_orchestration.py')),'accept','--run',str(run.path)]
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(result.returncode,2)
            self.assertFalse((root/'out/accepted.json').exists())

    def test_readiness_domain_cannot_be_shortened_from_the_public_command(self):
        result=subprocess.run([sys.executable,str(Path(__file__).with_name('readiness_orchestration.py')),'readiness','--frames','1'],capture_output=True,text=True)
        self.assertEqual(result.returncode,2)
        self.assertIn('unrecognized arguments',result.stderr)
