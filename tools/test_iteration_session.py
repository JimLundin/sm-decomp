"""Bounded session behavior through named operation/evidence seam."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class FixtureOperations:
    """External operation results; these fixtures never impersonate real acceptance."""
    def __init__(self):self.counter=0;self.events=[]
    def run(self,name,payload):
        self.events.append({'operation':name,'payload':payload})
        if name=='baseline':return {'run_id':'baseline'+str(self.counter),'metrics':{'generated':10+self.counter*10,'interpreted':90-self.counter*10,'fraction':(10+self.counter*10)/100},'domain':{'frames':18000}}
        if name=='select':return {'outcome':'supported-routine','selection_path':'fixture-selection','chosen':{'address':0x808000,'variant':'m1x1'}}
        if name=='prepare':return {'preparation_path':'fixture-preparation','baseline_path':payload['baseline']['run_id']}
        if name=='validate':
            self.counter+=1
            return {'published':True,'run_id':'baseline'+str(self.counter),'metrics':{'generated':10+self.counter*10,'interpreted':90-self.counter*10,'fraction':(10+self.counter*10)/100},'domain':{'frames':18000}}
        raise ValueError(name)


class RecoveryTests(unittest.TestCase):
    def test_rejected_pointer_preserves_bytes_and_racing_replacement_is_rejected(self):
        from iteration_session import record_rejected_pointer,verify_recovery_pointer
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'out').mkdir();pointer=root/'out/accepted.json';pointer.write_bytes(b'{corrupt private pointer\n')
            recovery=record_rejected_pointer(root,'Malformed accepted pointer')
            self.assertEqual((recovery/'diagnostics/rejected-pointer.bin').read_bytes(),pointer.read_bytes())
            verify_recovery_pointer(root,recovery)
            pointer.write_bytes(b'{"run_id":"competing"}')
            with self.assertRaisesRegex(ValueError,'changed'):
                verify_recovery_pointer(root,recovery)
            self.assertEqual(pointer.read_bytes(),b'{"run_id":"competing"}')


class SessionTests(unittest.TestCase):
    def test_default_request_accepts_one_and_explicit_count_continues_same_lineage(self):
        from iteration_session import Session
        for count in (1,2):
            with tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
                operations=FixtureOperations()
                session=Session.create(root,count=count)
                result=session.run(operations)
                self.assertEqual(result['status'],'complete')
                self.assertEqual(result['accepted_count'],count)
                self.assertEqual(len([event for event in operations.events if event['operation']=='validate']),count)
                choices=[event['payload']['baseline']['run_id'] for event in operations.events if event['operation']=='select']
                self.assertEqual(choices,['baseline'+str(number) for number in range(count)])
                self.assertTrue((session.path/'brief.md').exists())


    def test_target_already_met_does_no_candidate_work_and_failed_publication_never_counts(self):
        from iteration_session import Session
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            operations=FixtureOperations()
            result=Session.create(root,count=None,target=10).run(operations)
            self.assertEqual(result['accepted_count'],0)
            self.assertEqual([event['operation'] for event in operations.events],['baseline'])
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            operations=FixtureOperations()
            result=Session.create(root,count=None,target=15).run(operations)
            self.assertEqual(result['accepted_count'],1)
            self.assertEqual(result['status'],'complete')
        class FailedPublication(FixtureOperations):
            def run(self,name,payload):
                result=super().run(name,payload)
                if name=='validate':return {'published':False,'status':'repair-needed','reason':'Publication failed','candidate_path':'preserved-candidate'}
                return result
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            session=Session.create(root)
            result=session.run(FailedPublication())
            self.assertEqual(result['accepted_count'],0)
            self.assertEqual(result['status'],'repair-needed')
            self.assertIsNotNone(result['increment_id'])
            self.assertEqual(result['candidate']['candidate_path'],'preserved-candidate')

    def test_resume_validates_preserved_increment_without_reselecting_or_resetting_count(self):
        from iteration_session import Session
        class RepairOperations(FixtureOperations):
            def __init__(self):super().__init__();self.failed=True
            def run(self,name,payload):
                if name=='validate' and self.failed:
                    self.events.append({'operation':name,'payload':payload});self.failed=False
                    return {'published':False,'status':'repair-needed','candidate_path':'failed-source','reason':'Repair semantics'}
                if name=='retry':
                    self.events.append({'operation':name,'payload':payload});self.counter+=1
                    return {'published':True,'run_id':'baseline1','metrics':{'generated':20,'interpreted':80,'fraction':.2},'domain':{'frames':18000}}
                return super().run(name,payload)
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            operations=RepairOperations();session=Session.create(root)
            first=session.run(operations);increment=first['increment_id']
            resumed=Session.resume(root,first['session_id']);result=resumed.run(operations)
            self.assertEqual(result['status'],'complete')
            self.assertEqual(result['accepted_count'],1)
            retries=[event for event in operations.events if event['operation']=='retry']
            self.assertEqual(retries[0]['payload']['increment_id'],increment)
            self.assertEqual(len([event for event in operations.events if event['operation']=='select']),1)

    def test_stop_preserves_progress_and_cli_rejects_ambiguous_target(self):
        from iteration_session import Session
        class StopOperations(FixtureOperations):
            def run(self,name,payload):
                if name=='select':raise KeyboardInterrupt
                return super().run(name,payload)
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
            result=Session.create(root).run(StopOperations())
            self.assertEqual(result['status'],'stopped')
            self.assertEqual(result['accepted_count'],0)
            self.assertIsNotNone(result['increment_id'])
            command=subprocess.run(['python3',str(Path(__file__).with_name('iteration_session.py')),
                '--root',str(root),'--count','1','--target-generated','100'],capture_output=True,text=True)
            self.assertEqual(command.returncode,2)
            self.assertIn('not allowed',command.stderr)

    def test_invalid_request_rejects_before_creating_session(self):
        from iteration_session import Session
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for count,target in [(0,None),(1,100),(None,-1)]:
                with self.assertRaises(ValueError):Session.create(root,count,target)
            self.assertFalse((root/'out').exists())


if __name__=='__main__':unittest.main()
