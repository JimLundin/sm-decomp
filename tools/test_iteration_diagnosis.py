"""Immutable experiment history at the public evidence seam."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from readiness import Run, digest, write_json


class HistoryTests(unittest.TestCase):
    def candidate(self,root,number):
        (root/'candidate.py').write_text('candidate = {}\n'.format(number))
        run=Run.create(root,'iteration',{'domain':'Japan/USA any%','frames':18000},'accepted-parent')
        run.capture_source()
        run.record_check('generated-parity',{'status':'divergence','complete':True,'first_divergence':{'input':2,'region':'WRAM'}})
        run.finalize('divergence')
        return run

    def test_no_new_evidence_survives_candidate_retries_and_escalates_exactly_third(self):
        from iteration_diagnosis import append_experiment, verify_history
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            history=[]
            for number in range(3):
                candidate=self.candidate(root,number)
                before=(candidate.path/'verdict.json').read_bytes()
                report=append_experiment(root,'session-one','increment-one',candidate.path,
                    'Store timing changes effect','Independent ordered bus trace differs',
                    'Same mismatch; no independent observation yet')
                history.append(report)
                self.assertEqual(report['consecutive_without_evidence'],number+1)
                self.assertEqual(report['decision_required'],number==2)
                self.assertEqual((candidate.path/'verdict.json').read_bytes(),before)
            (root/'out/iteration-history/increment-one/20261010-candidate-diagnosis.json').write_text('{"next_action":"repair"}')
            verified=verify_history(root,'increment-one')
            self.assertEqual(len(verified),3)
            self.assertEqual({item['session_id'] for item in verified},{'session-one'})
            self.assertTrue((Path(history[-1]['run_path'])/'diagnostics/decision.md').exists())
            self.assertFalse((root/'out/accepted.json').exists())


class ObservationTests(unittest.TestCase):
    candidate=HistoryTests.candidate
    def observation(self,root,value,elapsed=1):
        run=Run.create(root,'diagnostic',{'instructions':1},'accepted-parent');run.capture_source()
        executable=run.path/'generated/probe'
        executable.write_text('#!/usr/bin/env python3\nimport json,sys\nfrom pathlib import Path\n'
            'Path(sys.argv[1]).write_text(json.dumps('+repr({'status':'divergence','complete':True,
                'first_mismatch':{'instruction':3,'region':'ordered bus','reference':{'value':1},'subject':{'value':value}},
                'elapsed_seconds':elapsed})+'))\n')
        executable.chmod(0o755)
        report=run.path/'diagnostics/observation.json'
        stage=run.run_stage('probe',[executable,report],exit_statuses={1:'divergence'})
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest();run.finalize('pass',mandatory_checks=['probe'])
        return {'run_path':str(run.path),'evidence':'diagnostics/observation.json','sha256':digest(report),
                'process':'logs/probe.json','process_sha256':digest(run.path/'logs/probe.json'),
                'executable':'generated/probe','executable_sha256':digest(executable),'kind':'first-mismatch'}

    def test_history_rejects_a_mutated_observation_and_command_checks_escalation(self):
        from iteration_diagnosis import append_experiment,verify_history
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            failed=self.candidate(root,0)
            descriptor=self.observation(root,2)
            append_experiment(root,'session','increment',failed.path,'Timing changes effect','Trace differs','Observed mismatch',[descriptor])
            artifact=Path(descriptor['run_path'])/descriptor['evidence']
            actual=json.loads(artifact.read_text());actual['first_mismatch']['subject']['value']=99
            write_json(artifact,actual)
            with self.assertRaisesRegex(ValueError,'checksum'):
                verify_history(root,'increment')
            command=subprocess.run(['python3',str(Path(__file__).with_name('iteration_diagnosis.py')),
                '--root',str(root),'--increment-id','increment','--history'],capture_output=True,text=True)
            self.assertEqual(command.returncode,2)
            self.assertIn('checksum',command.stderr)

    def test_failure_routing_uses_passing_controls_and_independent_sync_observations(self):
        from iteration_diagnosis import failure_summary
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            for predecessor,expected in [('pass','increment'),('divergence','shared paths/input/cache/harness')]:
                run=Run.create(root,'iteration',{'frames':18000},'accepted-parent');run.capture_source()
                run.record_check('instruction-diagnosis',{'status':'divergence','complete':True,'first_mismatch':{'instruction':3,'region':'bus'}})
                run.record_check('failure-controls',{'status':'divergence','complete':True,'controls':[{
                    'predecessor':{'status':predecessor,'complete':True},'disabled':{'status':'pass','complete':True}}]})
                run.finalize('divergence')
                manifest,summary=failure_summary(run.path)
                self.assertEqual(summary['routing']['route'],expected)
                self.assertEqual(summary['earliest_disagreement']['instruction'],3)
            run=Run.create(root,'iteration',{'frames':18000},'accepted-parent');run.capture_source()
            run.record_check('synchronization',{'status':'divergence','complete':True})
            run.finalize('divergence')
            self.assertEqual(failure_summary(run.path)[1]['routing']['route'],'independent hardware/input observation')

    def test_substantive_observation_resets_streak_but_new_paths_and_timing_do_not(self):
        from iteration_diagnosis import append_experiment
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            for number in range(2):
                failed=self.candidate(root,number)
                append_experiment(root,'session','increment',failed.path,'Timing changes effect','Trace differs','No observation yet')
            failed=self.candidate(root,2)
            report=append_experiment(root,'session','increment',failed.path,'Timing changes effect','Trace differs','Bus mismatch observed',[self.observation(root,2)])
            self.assertEqual(report['consecutive_without_evidence'],0)
            self.assertTrue(report['new_discriminating_evidence'])
            failed=self.candidate(root,3)
            report=append_experiment(root,'session','increment',failed.path,'Timing changes effect','Trace differs','Repeated result',[self.observation(root,2,elapsed=99)])
            self.assertEqual(report['consecutive_without_evidence'],1)
            self.assertFalse(report['new_discriminating_evidence'])
            failed=self.candidate(root,4)
            report=append_experiment(root,'session','increment',failed.path,'Timing changes effect','Trace differs','Changed actual bus result',[self.observation(root,3)])
            self.assertEqual(report['consecutive_without_evidence'],0)


if __name__=='__main__':unittest.main()
