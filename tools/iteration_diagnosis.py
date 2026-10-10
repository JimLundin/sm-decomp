#!/usr/bin/env python3
"""Immutable diagnostic experiment lineage across retries of one unresolved increment."""
import argparse
import fcntl
import hashlib
import json
import re
import sys
from pathlib import Path

from readiness import Run, digest, read_verdict, write_json
from readiness_contract import DOMAIN
from readiness_orchestration import source_snapshot
from readiness_reuse import safe_artifact, seal_owned_evidence, verify_owned_evidence


def history_directory(root,increment_id):
    if not re.fullmatch(r'[A-Za-z0-9_-]{1,96}',increment_id):
        raise ValueError('Increment ID must be stable safe text')
    return Path(root).resolve()/'out/iteration-history'/increment_id


def verify_history(root,increment_id):
    """Reverify ordered immutable references, source snapshots and evidence hashes."""
    directory=history_directory(root,increment_id)
    previous=None;records=[];parent=None
    for index,path in enumerate(sorted(path for path in directory.glob('*.json') if re.fullmatch(r'[0-9]{4}\.json',path.name)),1):
        reference=json.loads(path.read_text())
        if path.name!='{:04d}.json'.format(index) or reference.get('sequence')!=index or reference.get('previous_sha256')!=previous:
            raise ValueError('Experiment history ordering/hash chain differs')
        owner=Path(reference['run_path'])
        manifest,inventory=source_snapshot(owner)
        verdict=read_verdict(owner)
        if (manifest.get('type')!='iteration-experiment' or verdict.get('status')!='pass'
                or manifest.get('increment_id')!=increment_id or digest(owner/'experiment.json')!=reference['experiment_sha256']):
            raise ValueError('Experiment history run identity/verdict differs')
        verify_owned_evidence(owner,manifest)
        record=json.loads((owner/'experiment.json').read_text())
        candidate_manifest,_=failure_summary(Path(record['candidate_path']))
        if candidate_manifest['run_id']!=record['candidate_run_id'] or candidate_manifest['identities']['source']['sha256']!=record['candidate_source_sha256']:
            raise ValueError('Experiment preserved candidate identity differs')
        if record.get('increment_id')!=increment_id or record.get('sequence')!=index or record.get('previous_sha256')!=previous:
            raise ValueError('Experiment record identity/order differs')
        if parent is None:parent=record['accepted_parent']
        if record['accepted_parent']!=parent:
            raise ValueError('Unresolved increment predecessor changed across experiments')
        for observation in record.get('observations',[]):
            verified_observation,fingerprint=verify_observation(observation['descriptor'])
            if verified_observation!=observation:
                raise ValueError('Experiment substantive observation changed')
        if record.get('observation_fingerprints')!=[row['fingerprint'] for row in record.get('observations',[])]:
            raise ValueError('Experiment fingerprint list differs')
        records.append(record)
        previous=digest(path)
    # Streak and novelty are derived from substantive fingerprints across the
    # entire history; a restart cannot reset either.
    seen=set();streak=0
    for record in records:
        fingerprints=set(record.get('observation_fingerprints',[]))
        novel=bool(fingerprints-seen)
        streak=0 if novel else streak+1
        if record['new_discriminating_evidence']!=novel or record['consecutive_without_evidence']!=streak or record['decision_required']!=(streak>=3):
            raise ValueError('Experiment novelty/streak differs from preserved evidence')
        seen.update(fingerprints)
    return records


def failure_summary(owner):
    manifest,inventory=source_snapshot(owner)
    verdict=read_verdict(owner)
    if verdict.get('status') not in ('divergence','uncovered','error','incomplete') or verdict.get('run_id')!=manifest['run_id']:
        raise ValueError('Diagnosis requires a preserved terminal failed candidate')
    checks=verdict.get('checks',{})
    priority=('instruction-diagnosis','generated-parity','synchronization','changed-entry-coverage')
    selected=next((dict(checks[name],stage=name) for name in priority if checks.get(name,{}).get('status') in ('divergence','uncovered','error','incomplete')),
                  {'status':verdict['status'],'stage':verdict.get('stage'),'reason':verdict.get('reason')})
    controls=checks.get('failure-controls',{})
    from readiness_controls import route_failure
    matched=controls.get('controls',[])
    routing=route_failure(selected,
        matched[0].get('predecessor') if matched else None,matched[0].get('disabled') if matched else None,
        matched[0].get('candidate') if matched else selected if selected.get('first_mismatch') else None)
    if selected['status'] in ('error','incomplete'):
        routing={'route':'setup/build/budget','conclusion':'Candidate validation did not complete; preserve stage logs and repair setup or budgets',
                 'next_experiment':'Repair the first retained setup/build/budget failure and rerun fresh candidate validation'}
    earliest=selected.get('first_mismatch',selected.get('first_divergence'))
    if not earliest:
        earliest=next((report.get('first_mismatch') for report in selected.get('reports',[]) if report.get('first_mismatch')),None)
    for matched_control in matched:
        actual=matched_control.get('candidate',{}).get('first_mismatch')
        if actual:
            earliest=actual;break
    return manifest,{'failure':selected,'routing':routing,'controls':controls,
                     'earliest_disagreement':earliest}


def candidate_observations(owner):
    """Find actual focused mismatch evidence already retained by candidate controls."""
    owner=Path(owner)
    verdict=read_verdict(owner)
    descriptors=[]
    reports=verdict.get('checks',{}).get('instruction-diagnosis',{}).get('reports',[])
    controls=verdict.get('checks',{}).get('failure-controls',{}).get('controls',[])
    reports+= [item.get('candidate',{}) for item in controls]
    for report in reports:
        relative=report.get('evidence')
        if report.get('status')!='divergence' or not relative:continue
        artifact=safe_artifact(owner,relative)
        actual=json.loads(artifact.read_text())
        if not actual.get('first_mismatch'):continue
        prefix=Path(relative).parent.name[:-len('-instructions')]
        process=owner/'logs'/(prefix+'-focused-replay.json')
        binary=owner/'generated'/(prefix+'-focused-replay')
        if not process.exists() or not binary.exists():continue
        descriptors.append({'run_path':str(owner),'evidence':relative,'sha256':digest(artifact),
            'process':str(process.relative_to(owner)),'process_sha256':digest(process),
            'executable':str(binary.relative_to(owner)),'executable_sha256':digest(binary),'kind':'first-mismatch'})
    return descriptors


def append_experiment(root,session_id,increment_id,candidate,hypothesis,discriminator,result,evidence=None):
    """Append one exact-source experiment and return resumable next action."""
    if not hypothesis.strip() or not discriminator.strip() or not result.strip():
        raise ValueError('Hypothesis, predicted discriminator and experiment result required')
    if not re.fullmatch(r'[A-Za-z0-9_-]{1,96}',session_id):raise ValueError('Stable session ID required')
    directory=history_directory(root,increment_id);directory.mkdir(parents=True,exist_ok=True)
    with (directory/'history.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX)
        history=verify_history(root,increment_id)
        candidate=Path(candidate).resolve()
        manifest,summary=failure_summary(candidate)
        if history and any(item['session_id']!=session_id or item['accepted_parent']!=manifest.get('accepted_parent') for item in history):
            raise ValueError('Unresolved increment session/predecessor identity changed')
        run=Run.create(root,'iteration-experiment',dict(DOMAIN),manifest.get('accepted_parent'))
        print('Experiment: '+str(run.path),flush=True)
        try:
            run.capture_source()
            run.manifest.update(session_id=session_id,increment_id=increment_id,candidate_run_id=manifest['run_id'])
            run.save_manifest()
            run.record_artifact('failed-candidate-verdict',candidate/'verdict.json','diagnostics/candidate-verdict.json')
            before={row['path']:row['sha256'] for row in json.loads((candidate/'source/inventory.json').read_text())['files']}
            after={row['path']:row['sha256'] for row in run.source_files}
            write_json(run.path/'diagnostics/source-diff.json',{'candidate_source_sha256':manifest['identities']['source']['sha256'],
                'current_source_sha256':run.manifest['identities']['source']['sha256'],
                'changed':[{'path':path,'before':before.get(path),'after':after.get(path)} for path in sorted(set(before)|set(after)) if before.get(path)!=after.get(path)]})
            fingerprints=[];observations=[]
            for descriptor in candidate_observations(candidate)+(evidence or []):
                observation,fingerprint=verify_observation(descriptor)
                target='diagnostics/observation-{:02d}.json'.format(len(observations)+1)
                run.record_artifact('observation-'+str(len(observations)+1),Path(descriptor['run_path'])/descriptor['evidence'],target)
                observations.append(observation);fingerprints.append(fingerprint)
            seen={value for item in history for value in item['observation_fingerprints']}
            novel=bool(set(fingerprints)-seen)
            streak=0 if novel else (history[-1]['consecutive_without_evidence'] if history else 0)+1
            sequence=len(history)+1
            previous=digest(directory/'{:04d}.json'.format(sequence-1)) if history else None
            decision=streak>=3
            route=summary['routing']
            action='decision-required' if decision else 'prune-unexecuted-keys' if route['route']=='coverage' else 'independent-observation' if route['route']=='independent hardware/input observation' else 'localize-failure' if route['route']=='failure not reproduced' else 'repair-and-validate'
            record={'schema_version':2,'session_id':session_id,'increment_id':increment_id,'sequence':sequence,
                'previous_sha256':previous,'candidate_run_id':manifest['run_id'],'candidate_path':str(candidate),
                'candidate_source_sha256':manifest['identities']['source']['sha256'],'experiment_source_sha256':run.manifest['identities']['source']['sha256'],
                'accepted_parent':manifest.get('accepted_parent'),'hypothesis':hypothesis,'predicted_discriminator':discriminator,
                'result':result,'observations':observations,'observation_fingerprints':fingerprints,
                'new_discriminating_evidence':novel,'consecutive_without_evidence':streak,'decision_required':decision,
                'failure':summary,'next_action':action,'next_experiment':route['next_experiment'],'run_path':str(run.path),
                'resume':{'command':'just recomp-iteration --resume SESSION_ID','session_id':session_id,'increment_id':increment_id}}
            write_json(run.path/'experiment.json',record)
            text='{}\n\nUnresolved increment: {}. Candidate: {}. Accepted predecessor: {}.\n\nEarliest disagreement: {}.\n\nControls establish: {}.\n\nHypothesis: {}. Predicted discriminator: {}. Result: {}.\n\nConsecutive experiments without new discriminating evidence: {}.\n\nNext experiment: {}.\n\n[Immutable record](experiment.json) · [Candidate]({}/brief.md) · [Evidence](diagnostics/)\n'.format(
                'Decision required' if decision else 'Routine diagnosis continues',increment_id,manifest['run_id'],manifest.get('accepted_parent'),
                json.dumps(summary['earliest_disagreement'],sort_keys=True),route['conclusion'],hypothesis,discriminator,result,streak,route['next_experiment'],candidate)
            if decision:
                choices=[route['next_experiment'],'Inspect preserved candidate source and paired input/cache/hardware assumptions','Defer this increment and retain accepted predecessor']
                record['choices']=choices;write_json(run.path/'experiment.json',record)
                write_json(run.path/'diagnostics/decision.json',{'choices':choices,'increment_id':increment_id,'accepted_parent':manifest.get('accepted_parent')})
                (run.path/'diagnostics/decision.md').write_text(text+'\nChoices:\n'+''.join('- '+choice+'\n' for choice in choices))
            run.record_check('experiment-record',{'status':'pass','complete':True,'sequence':sequence,'sha256':digest(run.path/'experiment.json')})
            run.check_sources();seal_owned_evidence(run);run.save_manifest()
            run.finalize('pass','experiment-record','Diagnostic experiment retained; does not certify acceptance',mandatory_checks=['experiment-record'],brief_text=text)
            reference={'schema_version':2,'sequence':sequence,'run_path':str(run.path),'experiment_sha256':digest(run.path/'experiment.json'),'previous_sha256':previous}
            path=directory/'{:04d}.json'.format(sequence)
            with path.open('x') as target:json.dump(reference,target,indent=2,sort_keys=True);target.write('\n')
            return record
        except Exception as error:
            if run.manifest['state']=='running':run.finalize('error','experiment-record',str(error))
            raise


def substantive(value):
    """Strip lifecycle/provenance/timing metadata from observable execution facts."""
    ignored={'run_id','candidate_run_id','utc','started_utc','finished_utc','elapsed_seconds','total_seconds',
        'warm_replay_seconds','binary_sha256','source_sha256','rerun','evidence','command','path','cache'}
    if isinstance(value,dict):return {key:substantive(item) for key,item in sorted(value.items()) if key not in ignored}
    if isinstance(value,list):return [substantive(item) for item in value]
    return value


def verify_observation(descriptor):
    """Only actual process-bound execution observations can reset the streak."""
    owner=Path(descriptor['run_path']).resolve()
    manifest,inventory=source_snapshot(owner)
    if read_verdict(owner).get('run_id')!=manifest['run_id']:
        raise ValueError('Observation run is incomplete')
    verify_owned_evidence(owner,manifest)
    path=safe_artifact(owner,descriptor['evidence'])
    if digest(path)!=descriptor['sha256']:
        raise ValueError('Observation artifact checksum differs')
    process_path=safe_artifact(owner,descriptor['process'])
    if digest(process_path)!=descriptor['process_sha256']:
        raise ValueError('Observation process checksum differs')
    process=json.loads(process_path.read_text())
    binary=safe_artifact(owner,descriptor['executable'])
    if digest(binary)!=descriptor['executable_sha256'] or not process.get('command') or Path(process['command'][0]).resolve()!=binary.resolve():
        raise ValueError('Observation is not bound to the executed preserved binary')
    if process not in manifest.get('commands',[]) or process.get('complete') is not True or process.get('returncode') not in (0,1):
        raise ValueError('Observation execution did not complete with preserved provenance')
    for field in ('stdout','stderr'):
        if not safe_artifact(owner,process[field]).is_file():raise ValueError('Observation process log missing')
    actual=json.loads(path.read_text())
    kind=descriptor.get('kind')
    if kind=='first-mismatch':
        value=actual.get('first_mismatch')
        if actual.get('status')!='divergence' or not isinstance(value,dict) or not value:
            raise ValueError('Observation lacks an actual first instruction/state/bus mismatch')
        facts={'kind':kind,'mismatch':substantive(value)}
    elif kind=='controls':
        controls=actual.get('controls',[])
        if not controls:raise ValueError('Observation lacks matched controls')
        if not any(control.get('candidate',{}).get('status')=='divergence' and control.get('candidate',{}).get('first_mismatch') for control in controls):
            raise ValueError('Passing candidate controls are not a reproduced failure or new discriminating evidence')
        facts={'kind':kind,'controls':[]}
        for control in controls:
            row={'interval':substantive(control.get('interval',{}))}
            for name in ('candidate','predecessor','disabled'):
                report=control.get(name,{})
                if report.get('complete') is not True or report.get('status') not in ('pass','divergence'):
                    raise ValueError('Incomplete controls are not discriminating evidence')
                row[name]={'status':report['status'],'first_mismatch':substantive(report.get('first_mismatch'))}
            facts['controls'].append(row)
    elif kind=='coverage':
        if actual.get('complete') is not True or not isinstance(actual.get('uncovered_keys'),list):
            raise ValueError('Observation lacks complete changed-key coverage')
        facts={'kind':kind,'uncovered_keys':substantive(actual['uncovered_keys']),
               'executed':[{key:row[key] for key in ('address','mx','count')} for row in actual.get('executed',[])]}
    elif kind=='independent-hardware':
        observations=actual.get('observations')
        if actual.get('complete') is not True or not actual.get('independent_source') or not isinstance(observations,list) or not observations:
            raise ValueError('Independent hardware observation provenance/effects missing')
        facts={'kind':kind,'independent_source':actual['independent_source'],'observations':substantive(observations)}
    else:
        raise ValueError('Only instruction mismatch, controls, coverage or independent hardware facts can be discriminating')
    fingerprint=hashlib.sha256(json.dumps(facts,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    return {'descriptor':descriptor,'facts':facts,'fingerprint':fingerprint},fingerprint


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--session-id')
    parser.add_argument('--increment-id',required=True)
    parser.add_argument('--candidate',type=Path)
    parser.add_argument('--hypothesis')
    parser.add_argument('--discriminator')
    parser.add_argument('--result')
    parser.add_argument('--evidence',type=Path)
    parser.add_argument('--history',action='store_true')
    args=parser.parse_args(argv)
    try:
        if args.history:
            history=verify_history(args.root,args.increment_id)
            print(json.dumps({'increment_id':args.increment_id,'experiments':history,'latest':history[-1] if history else None},indent=2,sort_keys=True))
            return 0
        if not all((args.session_id,args.candidate,args.hypothesis,args.discriminator,args.result)):
            raise ValueError('--session-id, --candidate, --hypothesis, --discriminator and --result required')
        evidence=json.loads(args.evidence.read_text()) if args.evidence else []
        report=append_experiment(args.root,args.session_id,args.increment_id,args.candidate,args.hypothesis,args.discriminator,args.result,evidence)
        print(json.dumps({'run_path':report['run_path'],'next_action':report['next_action'],
            'decision_required':report['decision_required'],'consecutive_without_evidence':report['consecutive_without_evidence'],
            'resume':report['resume']},sort_keys=True))
        return 1 if report['decision_required'] else 0
    except Exception as error:
        print('Iteration diagnosis failed: '+str(error),file=sys.stderr);return 2


if __name__=='__main__':raise SystemExit(main())
