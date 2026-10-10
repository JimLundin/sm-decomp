#!/usr/bin/env python3
"""Retained isolated iteration demonstrations; never publishes acceptance."""
import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from readiness import Run,digest,write_json,read_verdict


class PolicyOperations:
    """Symbolic external results at the public Session operation seam."""
    def __init__(self,case):self.counter=0;self.events=[];self.case=case
    def run(self,name,payload):
        self.events.append({'operation':name,'payload':payload})
        state={'run_id':'symbolic-baseline'+str(self.counter),'metrics':{'generated':10+self.counter*10,'interpreted':90-self.counter*10,'fraction':.1+self.counter*.1},'domain':{'frames':18000}}
        if name=='baseline':return state
        if name=='select':
            if self.case=='user-stop':raise KeyboardInterrupt
            return {'outcome':'supported-routine','selection_path':'symbolic-selection','chosen':{'address':0x808000,'variant':'m1x1'}}
        if name=='prepare':return {'preparation_path':'symbolic-preparation','baseline_path':payload['baseline']['run_id']}
        if name=='validate':
            if self.case=='failed-publication':return {'published':False,'status':'repair-needed','candidate_path':'symbolic-rejected-candidate','reason':'simulated publication refusal'}
            self.counter+=1
            return dict(state,published=True,run_id='symbolic-baseline'+str(self.counter),metrics={'generated':10+self.counter*10,'interpreted':90-self.counter*10,'fraction':.1+self.counter*.1})
        raise ValueError(name)


def policy(output):
    from iteration_session import Session
    output=Path(output);output.mkdir()
    results={}
    for case,count,target in [('default-one',1,None),('count2',2,None),('target-already',None,10),('target-crossing',None,15),('failed-publication',1,None),('user-stop',1,None)]:
        root=output/case;root.mkdir();subprocess.run(['git','init','-q',str(root)],check=True)
        operations=PolicyOperations(case);session=Session.create(root,count=count,target=target);result=session.run(operations)
        results[case]={'status':result['status'],'accepted_count':result['accepted_count'],'events':operations.events,'state_path':str((session.path/'state.json').relative_to(output)),
            'increment_id':result['increment_id'],'session_id':result['session_id']}
    report={'schema_version':1,'scope':'session policy simulation; acceptance not certified','production_accepted_count':0,'cases':results}
    verify_policy(report,output);write_json(output/'report.json',report);return report


def verify_policy(report,output):
    expected={'default-one':('complete',1),'count2':('complete',2),'target-already':('complete',0),'target-crossing':('complete',1),'failed-publication':('repair-needed',0),'user-stop':('stopped',0)}
    if report.get('scope')!='session policy simulation; acceptance not certified' or report.get('production_accepted_count')!=0 or set(report.get('cases',{}))!=set(expected):raise ValueError('Policy scope differs')
    for case,(status,count) in expected.items():
        row=report['cases'][case];state=json.loads((Path(output)/row['state_path']).read_text())
        if row['status']!=status or row['accepted_count']!=count or state['status']!=status or state['accepted_count']!=count:raise ValueError('Policy count/stopping differs: '+case)
        validations=[event for event in row['events'] if event['operation']=='validate']
        if len(validations)!=(1 if case=='failed-publication' else count):raise ValueError('Policy extra candidate work: '+case)
        if case in ('failed-publication','user-stop') and not row['increment_id']:raise ValueError('Unresolved policy increment lost')
    return True


def failed_lineage(output):
    """Real immutable failure lifecycle with explicitly simulated execution facts."""
    from iteration_diagnosis import append_experiment,verify_history
    output=Path(output);output.mkdir();root=output/'repository';root.mkdir()
    subprocess.run(['git','init','-q',str(root)],check=True)
    (root/'fixture.py').write_text('synthetic_failure = True\n')
    records=[];candidates=[]
    for index in range(3):
        candidate=Run.create(root,'iteration',{'frames':18000},'symbolic-preserved-parent');candidate.capture_source()
        candidate.record_check('generated-parity',{'status':'divergence','complete':True,
            'scope':'policy simulation; no real parity claimed','first_divergence':{'frame':0}})
        candidate.finalize('divergence','generated-parity','Simulated unresolved execution failure')
        candidates.append(str(candidate.path))
        records.append(append_experiment(root,'synthetic-session','synthetic-increment',candidate.path,
            'Later key may change bus effects','Cold observed late instruction differs','No new discriminating evidence'))
    verified=verify_history(root,'synthetic-increment')
    if [row['consecutive_without_evidence'] for row in verified]!=[1,2,3] or [row['decision_required'] for row in verified]!=[False,False,True]:raise ValueError('Three-experiment rule differs')
    pointer=root/'out/accepted.json';pointer.write_bytes(b'{"synthetic_predecessor":"preserve"}\n');before=digest(pointer)
    command=[sys.executable,str(Path(__file__).with_name('readiness_orchestration.py')),'accept','--run',candidates[-1]]
    process=subprocess.run(command,capture_output=True,text=True)
    write_json(output/'publication.json',{'command':command,'returncode':process.returncode,'stdout':process.stdout,'stderr':process.stderr})
    if process.returncode!=2 or digest(pointer)!=before:raise ValueError('Private failed publication changed pointer')
    report={'schema_version':1,'scope':'simulated failures with real lifecycle and publication refusal; acceptance not certified','accepted_count':0,
        'candidates':candidates,'records':records,'history_root':str(root),'publication_returncode':process.returncode,'pointer_before_sha256':before,'pointer_after_sha256':digest(pointer)}
    write_json(output/'report.json',report);return report


def verify_probe(report,output,source,binary):
    output=Path(output)
    if report.get('status')!='pass' or report.get('complete') is not True or report.get('accepted_count')!=0 or report.get('source_sha256')!=source or report.get('binary_sha256')!=digest(binary):raise ValueError('Synthetic probe scope/source differs')
    if report.get('rom_sha256')!=digest(output/'fixture.sfc') or report.get('input_sha256')!=digest(output/'fixture.inputs'):raise ValueError('Synthetic probe material differs')
    for name in ('early','candidate','predecessor','disabled','warm'):
        actual=json.loads((output/name/'instruction.json').read_text())
        if report.get(name)!=actual:raise ValueError('Probe embedded report differs from actual execution: '+name)
        if actual.get('complete') is not True or actual.get('start_input_index')!=0 or actual.get('start_instruction')!=0:
            raise ValueError('Probe matching execution interval incomplete: '+name)
    if report['early']['status']!='pass' or report['early']['completed_instructions']!=65:raise ValueError('Early occurrence differs')
    for name in ('candidate','warm'):
        row=report[name];mismatch=row.get('first_mismatch',{});bus=row.get('first_bus_difference',{})
        if (row.get('status')!='divergence' or row.get('completed_instructions')!=87
                or mismatch.get('instruction')!=86 or mismatch.get('input_index')!=0 or mismatch.get('pad')!=0
                or mismatch.get('subject',{}).get('generated_key')!=[0x8004,3]
                or any(bus.get(side,{}).get(field)!=value for side,value in (('reference',0),('subject',0x99)) for field in ('value',))
                or any(bus.get(side,{}).get('kind')!='write' or bus.get(side,{}).get('address')!=0x10 for side in ('reference','subject'))):
            raise ValueError('Late failure did not reproduce actual literal store disagreement')
        for side in ('reference','subject'):
            for phase in ('before','after'):
                if digest(output/name/(side+'-'+phase+'.hardware.bin'))!=mismatch[side]['hardware_'+phase+'_sha256']:
                    raise ValueError('Late mismatch snapshot differs')
    for name in ('predecessor','disabled'):
        row=report[name]
        if row.get('status')!='pass' or row.get('completed_instructions')!=87 or row.get('expected_instructions')!=87:raise ValueError('Matching late controls incomplete')
    cache=report.get('cache',{})
    record=json.loads((output/'divergent-cache/cache.json').read_text())
    validation=json.loads((output/'divergent-cache/validation.json').read_text());pair=json.loads((output/'divergent-cache/pair.json').read_text())
    if (cache.get('validation')!=validation or validation.get('status')!='pass'
            or validation.get('complete') is not True or validation.get('observed_rollovers')!=2 or validation.get('completed_frames')!=1
            or validation.get('candidate_reference_comparison')!='divergence'
            or validation.get('pair_sha256')!=pair.get('pair_sha256')
            or pair['identities'].get('rom_sha256')!=report['rom_sha256'] or pair['identities'].get('input_sha256')!=report['input_sha256']
            or pair['identities'].get('source_sha256')!=source):raise ValueError('Actual paired cache divergence evidence differs')
    if record.get('pair_file_sha256')!=digest(output/'divergent-cache/pair.json') or record.get('validation_file_sha256')!=digest(output/'divergent-cache/validation.json'):
        raise ValueError('Actual paired cache files differ')
    from paired_checkpoint_verify import verify_pair
    verify_pair(pair,validation,{'cache_history_sha256':pair['history_sha256']})
    return True


def run_demonstrations(root, feedback_owner=None, timeout=None):
    root=Path(root).resolve();pointer=root/'out/accepted.json';before=pointer.read_bytes() if pointer.exists() else None
    run=Run.create(root,'iteration-demonstrations',{'scope':'synthetic nonaccepting'})
    print('Demonstrations: '+str(run.path),flush=True)
    try:
        run.capture_source();source=run.manifest['identities']['source']['sha256']
        policy_output=run.path/'diagnostics/policy'
        stage=run.run_stage('policy-demonstrations',[sys.executable,root/'tools/iteration_demonstrations.py','policy','--out',policy_output],timeout=timeout)
        if stage['status']!='pass':raise ValueError('Session policy demonstration failed')
        policy_report=json.loads((policy_output/'report.json').read_text());verify_policy(policy_report,policy_output)
        lineage=failed_lineage(run.path/'diagnostics/lineage')
        environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/'build/probe'),CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0')
        stage=run.run_stage('iteration-probe-build',['cargo','build','--release','-p','snes','--bin','iteration-probe','--locked','--offline'],env=environment,timeout=timeout)
        if stage['status']!='pass':raise ValueError('Late occurrence probe build failed')
        run.record_artifact('iteration-probe',run.path/'build/probe/release/iteration-probe','generated/iteration-probe')
        probe_output=run.path/'diagnostics/late-occurrence'
        stage=run.run_stage('late-occurrence-probe',[run.path/'generated/iteration-probe',probe_output,source],timeout=timeout)
        if stage['status']!='pass':raise ValueError('Late occurrence probe failed')
        probe=json.loads((probe_output/'report.json').read_text());verify_probe(probe,probe_output,source,run.path/'generated/iteration-probe')
        feedback=None
        if feedback_owner:
            from readiness_reuse import verify_accepted
            from readiness_feedback import verify_feedback
            owner=Path(feedback_owner).resolve();verified=verify_accepted(owner,require_pointer=True)
            summary=verified['verdict']['checks']['feedback-demonstrations'];verify_feedback(owner,verified['manifest'],summary)
            feedback={'run_path':str(owner),'run_id':verified['manifest']['run_id'],'summary':summary}
        if (pointer.read_bytes() if pointer.exists() else None)!=before:raise ValueError('Workspace accepted pointer changed during demonstrations')
        report={'schema_version':1,'type':'iteration-demonstrations','scope':'synthetic nonaccepting','status':'pass','complete':True,'accepted_count':0,
            'source_sha256':source,'pointer_before_sha256':__import__('hashlib').sha256(before).hexdigest() if before else None,
            'pointer_after_sha256':digest(pointer) if pointer.exists() else None,'policy':policy_report,'lineage':lineage,'late_occurrence':probe,'verified_readiness_feedback':feedback}
        write_json(run.path/'iteration-demonstrations.json',report);run.record_artifact('demonstration-report',run.path/'iteration-demonstrations.json')
        run.record_check('iteration-demonstrations',{'status':'pass','complete':True,'accepted_count':0,'evidence':'iteration-demonstrations.json','sha256':digest(run.path/'iteration-demonstrations.json')})
        run.check_sources()
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest()
        run.finalize('pass','iteration-demonstrations','Synthetic demonstrations passed; production acceptance remains separate',mandatory_checks=['iteration-demonstrations'],
            brief_text='Synthetic iteration demonstrations passed; accepted count 0.\n\n[Retained case evidence](iteration-demonstrations.json) · [Late mismatch and controls](diagnostics/late-occurrence/report.json) · [Session policy simulation](diagnostics/policy/report.json) · [Three-experiment history and private publication refusal](diagnostics/lineage/report.json)\n')
        return run.path
    except Exception as error:
        unchanged=(pointer.read_bytes() if pointer.exists() else None)==before
        run.finalize('error','iteration-demonstrations',str(error)+'; workspace pointer unchanged='+str(unchanged));raise


def verify_demonstrations(owner):
    from readiness_orchestration import source_snapshot
    from readiness_reuse import verify_owned_evidence
    from iteration_diagnosis import verify_history
    owner=Path(owner).resolve();manifest,_=source_snapshot(owner);verify_owned_evidence(owner,manifest)
    if manifest['type']!='iteration-demonstrations' or read_verdict(owner)['status']!='pass':raise ValueError('Demonstrations did not complete')
    report=json.loads((owner/'iteration-demonstrations.json').read_text())
    gate=read_verdict(owner)['checks']['iteration-demonstrations']
    if digest(owner/'iteration-demonstrations.json')!=gate['sha256']:raise ValueError('Demonstration report checksum differs')
    if report.get('accepted_count')!=0 or report.get('scope')!='synthetic nonaccepting' or report.get('source_sha256')!=manifest['identities']['source']['sha256'] or report['pointer_before_sha256']!=report['pointer_after_sha256']:raise ValueError('Demonstration acceptance/source/pointer scope differs')
    verify_policy(report['policy'],owner/'diagnostics/policy')
    verify_probe(report['late_occurrence'],owner/'diagnostics/late-occurrence',report['source_sha256'],owner/'generated/iteration-probe')
    lineage=report['lineage'];history=verify_history(lineage['history_root'],'synthetic-increment')
    if [row['consecutive_without_evidence'] for row in history]!=[1,2,3] or lineage['publication_returncode']!=2 or lineage['pointer_before_sha256']!=lineage['pointer_after_sha256']:raise ValueError('Escalation/publication refusal differs')
    for name in ('policy-demonstrations','iteration-probe-build','late-occurrence-probe'):
        process=json.loads((owner/'logs'/(name+'.json')).read_text())
        if process not in manifest['commands'] or process.get('returncode')!=0 or process.get('complete') is not True:raise ValueError('Demonstration process origin differs')
        if name=='late-occurrence-probe' and process['command']!=[str(owner/'generated/iteration-probe'),str(owner/'diagnostics/late-occurrence'),report['source_sha256']]:raise ValueError('Probe invocation differs')
    if report.get('verified_readiness_feedback'):
        from readiness_reuse import verify_accepted
        from readiness_feedback import verify_feedback
        linked=report['verified_readiness_feedback'];verified=verify_accepted(linked['run_path'],require_pointer=True)
        verify_feedback(linked['run_path'],verified['manifest'],linked['summary'])
    return report


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__);sub=parser.add_subparsers(dest='command',required=True)
    command=sub.add_parser('policy');command.add_argument('--out',type=Path,required=True)
    command=sub.add_parser('run');command.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent);command.add_argument('--feedback-owner',type=Path);command.add_argument('--timeout',type=float)
    command=sub.add_parser('verify');command.add_argument('--run',type=Path,required=True)
    args=parser.parse_args(argv)
    try:
        if args.command=='policy':policy(args.out)
        elif args.command=='verify':verify_demonstrations(args.run)
        else:run_demonstrations(args.root,args.feedback_owner,args.timeout)
        return 0
    except Exception as error:print(str(error),file=sys.stderr);return 2


if __name__=='__main__':raise SystemExit(main())
