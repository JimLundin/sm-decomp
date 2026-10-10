#!/usr/bin/env python3
"""Bounded explicit recompilation sessions with exact-source child evidence."""
import argparse
import fcntl
import json
import os
import re
import sys
import uuid
from pathlib import Path

from readiness import Run, digest, utc_now, write_json
from readiness_contract import DOMAIN


def durable_json(path,value):
    path=Path(path);temporary=path.with_suffix(path.suffix+'.tmp')
    with temporary.open('w') as target:
        json.dump(value,target,indent=2,sort_keys=True);target.write('\n');target.flush();os.fsync(target.fileno())
    os.replace(temporary,path)
    directory=os.open(path.parent,os.O_DIRECTORY)
    try:os.fsync(directory)
    finally:os.close(directory)


def record_rejected_pointer(root,reason):
    """Retain exact rejected bytes before full fresh readiness; never delete them."""
    from readiness_reuse import seal_owned_evidence
    root=Path(root).resolve();pointer=root/'out/accepted.json'
    run=Run.create(root,'accepted-pointer-recovery',dict(DOMAIN))
    run.capture_source()
    run.record_artifact('rejected-pointer',pointer,'diagnostics/rejected-pointer.bin')
    run.record_check('rejected-pointer',{'status':'pass','complete':True,'reason':reason,'sha256':digest(pointer)})
    seal_owned_evidence(run);run.save_manifest()
    run.finalize('pass','rejected-pointer','Exact invalid pointer preserved; complete fresh readiness required',mandatory_checks=['rejected-pointer'])
    return run.path


def verify_recovery_pointer(root,owner):
    from readiness_orchestration import source_snapshot
    from readiness_reuse import verify_owned_evidence
    from readiness import read_verdict
    owner=Path(owner);manifest,inventory=source_snapshot(owner);verify_owned_evidence(owner,manifest)
    if manifest.get('type')!='accepted-pointer-recovery' or read_verdict(owner).get('status')!='pass':
        raise ValueError('Pointer recovery evidence invalid')
    preserved=owner/'diagnostics/rejected-pointer.bin'
    identity=manifest['identities']['rejected-pointer']['sha256']
    current=Path(root)/'out/accepted.json'
    if digest(preserved)!=identity or not current.exists() or digest(current)!=identity or current.read_bytes()!=preserved.read_bytes():
        raise ValueError('Rejected accepted pointer changed during recovery; verify competing state')
    return preserved


class Session:
    """Nonsource-validating coordinator; exact source lives in each child run."""
    @classmethod
    def create(cls,root,count=1,target=None):
        if count is not None and (not isinstance(count,int) or isinstance(count,bool) or count<=0):raise ValueError('Increment count must be a positive integer')
        if target is not None and (not isinstance(target,int) or isinstance(target,bool) or target<=0):raise ValueError('Generated instruction target must be a positive integer')
        if target is not None and count is not None:raise ValueError('Specify count or target, not both')
        root=Path(root).resolve();session_id='session-'+uuid.uuid4().hex
        path=root/'out/iteration-sessions'/session_id;path.mkdir(parents=True)
        state={'schema_version':1,'session_id':session_id,'started_utc':utc_now(),'status':'running','count':count,'target':target,
            'accepted_count':0,'increment_id':None,'baseline':None,'initial_baseline':None,'selection':None,'preparation':None,'candidate':None,'events':[]}
        session=cls(root,path,state);session.save();return session

    @classmethod
    def resume(cls,root,session_id):
        if not re.fullmatch(r'session-[0-9a-f]{32}',session_id):raise ValueError('Invalid session ID')
        path=Path(root).resolve()/'out/iteration-sessions'/session_id
        state=json.loads((path/'state.json').read_text())
        if state.get('session_id')!=session_id:raise ValueError('Session identity differs')
        return cls(Path(root).resolve(),path,state)

    def __init__(self,root,path,state):self.root,self.path,self.state=root,path,state
    def save(self):durable_json(self.path/'state.json',self.state)

    def apply_result(self,name,result):
        if name=='baseline':
            self.state['baseline']=result
            if self.state['initial_baseline'] is None:self.state['initial_baseline']=result
        elif name=='select':self.state['selection']=result
        elif name=='prepare':self.state['preparation']=result
        elif name in ('validate','retry'):
            self.state['candidate']=result
            if result.get('published'):
                counted=self.state.setdefault('counted_publications',[])
                if result['run_id'] not in counted:
                    counted.append(result['run_id']);self.state['accepted_count']+=1
                self.state['baseline']=result
                self.state.update(increment_id=None,selection=None,preparation=None,candidate=None)

    def operation(self,operations,name,payload):
        pending=self.state.get('pending')
        if pending is None:
            sequence=len(self.state['events'])+1
            journal=self.path/'{:04d}-{}.request.json'.format(sequence,name)
            pending={'sequence':sequence,'operation':name,'payload':payload,'journal':journal.name,'operation_id':uuid.uuid4().hex}
            durable_json(journal,pending)
            self.state['pending']=pending;self.save()
        else:
            name,payload=pending['operation'],pending['payload']
            journal=self.path/pending['journal']
        result_path=self.path/'{:04d}-{}.json'.format(pending['sequence'],name)
        if result_path.exists():result=json.loads(result_path.read_text())['result']
        else:
            result=operations.recover(name,payload,journal) if hasattr(operations,'recover') else None
            if result is None:
                if hasattr(operations,'bind_journal'):operations.bind_journal(journal)
                result=operations.run(name,payload)
            event={'sequence':pending['sequence'],'operation':name,'payload':payload,'result':result,'utc':utc_now()}
            durable_json(result_path,event)
        self.apply_result(name,result)
        self.state['events'].append({'sequence':pending['sequence'],'operation':name,'evidence':result_path.name})
        self.state['pending']=None;self.save();return result

    def finish(self,status,reason):
        self.state.update(status=status,reason=reason,finished_utc=utc_now());self.save()
        baseline=self.state.get('baseline') or {};initial=self.state.get('initial_baseline') or {}
        text='{}: {} accepted increment(s), requested {}.\n\n{}\n\nSession: {}. Unresolved increment: {}.\n\nDomain: Japan/USA any%, input indices 0..17999, clean power-on, seed 0, P1, lsnes-profile.\n\n'.format(
            status,self.state['accepted_count'],self.state['count'] if self.state['count'] is not None else 'generated instructions ≥ '+str(self.state['target']),reason,self.state['session_id'],self.state.get('increment_id'))
        before=initial.get('metrics',{});after=baseline.get('metrics',{})
        text+='Generated instructions: {} → {}. Interpreted: {} → {}. Generated fraction: {} → {}.\n\n'.format(before.get('generated','unavailable'),after.get('generated','unavailable'),before.get('interpreted','unavailable'),after.get('interpreted','unavailable'),before.get('fraction','unavailable'),after.get('fraction','unavailable'))
        text+='Validated distinct keys: {} → {}.\n\n'.format(before.get('validated_distinct_keys','unavailable'),after.get('validated_distinct_keys','unavailable'))
        patterns=baseline.get('patterns',{}).get('patterns',[])
        text+='Declared patterns: {}. [Full per-key, pattern, deferred/unsupported and reuse evidence](state.json).\n\n'.format(', '.join(row['mnemonic']+'/'+row['mode'] for row in patterns) or 'see accepted instruction evidence')
        reused=baseline.get('reused_evidence',[])
        text+='Reused gates: {}.\n\n'.format(', '.join(str(row.get('check',row.get('name','gate')))+' from '+str(row.get('run_id',row.get('source_run_id','verified predecessor'))) for row in reused) or 'none')
        from datetime import datetime
        elapsed=(datetime.fromisoformat(self.state['finished_utc'])-datetime.fromisoformat(self.state['started_utc'])).total_seconds()
        text+='Total session time including readiness, preparation and retries: {:.1f}s.\n\n'.format(elapsed)
        text+='| Completed validation work | Seconds |\n| --- | --- |\n'
        for name,report in baseline.get('timings',{}).items():
            seconds=report.get('total_seconds',report.get('elapsed_seconds'))
            if seconds is not None:text+='| {} | {:.1f} |\n'.format(name,seconds)
            for stage,duration in report.get('timings',{}).items():text+='| {} {} | {:.1f} |\n'.format(name,stage,duration)
        text+='\n'
        text+='Warm focus target: 60s. Each cold replay target: 1200s. Actual timeouts remain incomplete.\n\n'
        for event in self.state['events']:text+='- [{}]({})\n'.format(event['operation'],event['evidence'])
        text+='\n[Session state](state.json) · [Accepted evidence and later hardware debt]({}/brief.md)\n'.format(baseline.get('run_path','unavailable'))
        for event in self.state['events']:
            evidence=json.loads((self.path/event['evidence']).read_text()).get('result',{})
            if evidence.get('source_diff_text') is not None:
                artifact=self.path/(str(event['sequence'])+'-source.diff')
                artifact.write_text(evidence['source_diff_text'])
                text+='\n[Candidate source diff]({})\n'.format(artifact.name)
            if evidence.get('candidate_path'):
                text+='\n[Preserved candidate]({}/brief.md) · [Verdict]({}/verdict.json) · [Source snapshot]({}/source/inventory.json).\n'.format(evidence['candidate_path'],evidence['candidate_path'],evidence['candidate_path'])
            if evidence.get('diagnostics'):
                diagnostic=evidence['diagnostics'];failure=diagnostic.get('failure',{})
                text+='\nEarliest disagreement: {}. Controls: {}. Next action: {}.\n'.format(json.dumps(failure.get('earliest_disagreement'),sort_keys=True),failure.get('routing',{}).get('conclusion','unresolved'),diagnostic.get('next_action','repair-and-validate'))
                text+='[Ordered diagnostic history]({}/brief.md). Preserved predecessor: {}.\n'.format(diagnostic.get('run_path','unavailable'),diagnostic.get('accepted_parent'))
        (self.path/'brief.md').write_text(text)
        return self.state

    def run(self,operations):
        with (self.path/'session.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX)
            self.state=json.loads((self.path/'state.json').read_text())
            if self.state['status']=='complete':return self.state
            self.state['status']='running';self.save()
            try:
                if self.state.get('pending'):
                    pending=self.state['pending']
                    self.operation(operations,pending['operation'],pending['payload'])
                while True:
                    if self.state.get('candidate'):
                        payload={'baseline':self.state['baseline'],'session_id':self.state['session_id'],'increment_id':self.state['increment_id'],
                            'candidate':self.state['candidate'],'preparation':self.state['preparation'],'selection':self.state['selection']}
                        candidate=self.operation(operations,'retry',payload)
                        if not candidate.get('published'):return self.finish(candidate.get('status','repair-needed'),candidate.get('reason','Next diagnostic experiment required'))
                    if self.state['increment_id'] is None:
                        baseline=self.operation(operations,'baseline',{'previous':self.state['baseline'],'session_id':self.state['session_id']})
                        if baseline.get('status') in ('error','incomplete'):return self.finish(baseline['status'],baseline.get('reason','Baseline unavailable'))
                        done=(self.state['count'] is not None and self.state['accepted_count']>=self.state['count']) or (self.state['target'] is not None and baseline['metrics']['generated']>=self.state['target'])
                        if done:return self.finish('complete','Requested measured scope completed')
                        self.state['increment_id']='increment-'+uuid.uuid4().hex;self.save()
                    payload={'baseline':self.state['baseline'],'session_id':self.state['session_id'],'increment_id':self.state['increment_id']}
                    selection=self.state['selection'] or self.operation(operations,'select',payload)
                    if selection.get('outcome')=='no-covered-candidate':return self.finish('decision-required','No remaining covered candidate; independently validate a concrete domain extension')
                    if selection.get('outcome') not in ('supported-routine','needs-timed-pattern'):return self.finish(selection.get('status','error'),selection.get('reason','Measured selection failed'))
                    preparation=self.state['preparation'] or self.operation(operations,'prepare',dict(payload,selection=selection))
                    if preparation.get('status') in ('error','incomplete','decision-required'):return self.finish(preparation['status'],preparation.get('reason','Candidate preparation failed'))
                    candidate=self.operation(operations,'validate',dict(payload,selection=selection,preparation=preparation))
                    if not candidate.get('published'):
                        return self.finish(candidate.get('status','repair-needed'),candidate.get('reason','Candidate needs an evidence-based repair and fresh validation'))
            except KeyboardInterrupt:return self.finish('stopped','User stopped the current operation; accepted evidence and candidate remain preserved')
            except Exception as error:return self.finish('error',str(error))


class ProductionOperations:
    """Hardwired production gates; fixture runners exist only in the session API."""
    def __init__(self,root,original_run=None,timeout=None,jobs=4):
        self.root=Path(root).resolve();self.original_run=original_run;self.timeout=timeout;self.jobs=jobs

    def bind_journal(self,path):
        self.journal=Path(path)
        self.progress=self.journal.with_name(self.journal.name.replace('.request.json','.progress.json'))
        if not self.progress.exists():durable_json(self.progress,{'request_sha256':digest(self.journal)})

    def allocated(self,run):
        durable_json(run.path/'manifest.json',run.manifest)
        self.journal_update(child_path=str(run.path))

    def journal_update(self,**values):
        record=json.loads(self.progress.read_text());record.update(values);durable_json(self.progress,record)

    def recover(self,name,payload,journal):
        self.bind_journal(journal)
        request=json.loads(journal.read_text());progress=json.loads(self.progress.read_text())
        if (progress.get('request_sha256')!=digest(journal) or request.get('operation')!=name or request.get('payload')!=payload):
            raise ValueError('Interrupted operation request/journal identity differs')
        record=dict(request,**progress)
        if name not in ('validate','retry') or not record.get('increment'):return None
        from readiness import read_verdict
        increment=record['increment'];child=record.get('child_path')
        matches=[]
        for manifest_path in (self.root/'out/runs').glob('*/manifest.json'):
            candidate_manifest=json.loads(manifest_path.read_text())
            if candidate_manifest.get('operation_id')==record['operation_id']:matches.append(manifest_path.parent)
        if matches:
            pointer=json.loads((self.root/'out/accepted.json').read_text())
            own_published=[path for path in matches if path.name==pointer.get('run_id')]
            recovered=own_published[0] if own_published else sorted(matches)[-1]
            if child!=str(recovered):child=str(recovered);self.journal_update(child_path=child)
        if child:
            candidate=Path(child)
            manifest=json.loads((candidate/'manifest.json').read_text())
            if any(manifest.get(key)!=value for key,value in (
                ('operation_id',record['operation_id']),('session_id',payload['session_id']),
                ('increment_id',payload['increment_id']),('accepted_parent',payload['baseline']['run_id']))):
                raise ValueError('Interrupted child does not belong to this exact session operation')
            pointer=json.loads((self.root/'out/accepted.json').read_text())
            if pointer.get('run_id')==candidate.name:
                return self.published_result(candidate,increment)
            from readiness import read_verdict
            verdict=read_verdict(candidate)
            if verdict.get('status') in ('divergence','uncovered','error'):
                return {'published':False,'status':'repair-needed','candidate_path':str(candidate),'reason':'Interrupted operation retained its failed exact-source candidate','preparation_path':payload['preparation']['preparation_path']}
            if manifest.get('state')=='running':
                from readiness import source_inventory
                from readiness_orchestration import source_snapshot
                if not (candidate/'source/inventory.json').exists():
                    abandoned=Run(self.root,candidate,manifest);abandoned.capture_source();abandoned.finalize('incomplete','readiness','Interrupted before source capture')
                    return self.validate_candidate(increment,payload)
                _,inventory=source_snapshot(candidate)
                if source_inventory(self.root)!=inventory['files']:raise ValueError('Source changed after interrupted candidate; retained child requires diagnosis')
                abandoned=Run(self.root,candidate,manifest)
                abandoned.finalize('incomplete','readiness','Interrupted before operation completed')
        else:
            # Mutation completed before allocation: retain its exact source now,
            # after verifying it is precisely the sealed prepared proposal.
            from readiness import source_inventory
            preparation=Path(increment.get('preparation_path',payload['preparation']['preparation_path']))
            from readiness_orchestration import source_snapshot
            from readiness_reuse import verify_owned_evidence
            prepared_manifest,prepared_source=source_snapshot(preparation);verify_owned_evidence(preparation,prepared_manifest)
            expected={row['path']:(row['sha256'],row['mode']) for row in prepared_source['files']}
            report=read_verdict(preparation)['checks'];proposal=report.get('candidate-proposal',report.get('pattern-proposal'))
            expected['config/functions.toml']=(proposal['proposed_config_sha256'],expected['config/functions.toml'][1])
            if 'proposed_declaration_sha256' in proposal:
                path='crates/recomp/src/timed_patterns.json';expected[path]=(proposal['proposed_declaration_sha256'],expected[path][1])
            actual={row['path']:(row['sha256'],row['mode']) for row in source_inventory(self.root)}
            prior={row['path']:(row['sha256'],row['mode']) for row in prepared_source['files']}
            if actual==prior:return None
            if actual!=expected:
                preserved=Run.create(self.root,'iteration',dict(DOMAIN),payload['baseline']['run_id'],metadata={
                    'increment':increment,'session_id':payload['session_id'],'increment_id':payload['increment_id'],'operation_id':record['operation_id']})
                preserved.capture_source();preserved.finalize('error','candidate-application','Interrupted mutation differs from sealed preparation')
                self.journal_update(child_path=str(preserved.path))
                return {'published':False,'status':'repair-needed','candidate_path':str(preserved.path),
                    'reason':'Interrupted candidate mutation differs from sealed preparation; exact source retained for diagnosis'}
            preserved=Run.create(self.root,'iteration',dict(DOMAIN),payload['baseline']['run_id'])
            preserved.manifest.update(increment=increment,session_id=payload['session_id'],increment_id=payload['increment_id'],operation_id=record['operation_id'])
            preserved.capture_source();preserved.finalize('incomplete','readiness','Interrupted after applying prepared source')
            self.journal_update(child_path=str(preserved.path))
        return self.validate_candidate(increment,payload)

    def accepted(self,verify_material=True):
        from readiness_reuse import verify_accepted,accepted_pointer
        from readiness_orchestration import verify_origins
        pointer=json.loads((self.root/'out/accepted.json').read_text())
        owner=Path(pointer['run_path']);verified=verify_accepted(owner,require_pointer=True);verify_origins(owner,verified)
        expected_pointer=accepted_pointer(owner,verified)
        if any(pointer.get(key)!=value for key,value in expected_pointer.items() if key!='accepted_utc') or not pointer.get('accepted_utc'):
            raise ValueError('Current accepted pointer fields differ from verified accepted origin')
        coverage=verified['verdict']['checks']['changed-entry-coverage']
        total=coverage['generated_instructions']+coverage['interpreted_instructions']
        inputs=verified['manifest']['identities']['inputs']
        if verify_material:
            for relative,key in (('rom/sm.sfc','rom_sha256'),('movies/super-metroid-any-3653.lsmv','movie_sha256')):
                if not (self.root/relative).is_file():raise ValueError('Missing required input: '+str(self.root/relative))
                if digest(self.root/relative)!=inputs[key]:raise ValueError('Current accepted material input differs: '+relative)
        return {'run_id':pointer['run_id'],'run_path':str(owner),'metrics':{'generated':coverage['generated_instructions'],
            'interpreted':coverage['interpreted_instructions'],'fraction':coverage['generated_instructions']/total if total else 0,
            'validated_distinct_keys':coverage['validated_distinct_keys'],'per_key':coverage['executed'],
            'deferred':coverage.get('deferred',[]),'deferred_new_keys':verified['manifest'].get('increment',{}).get('deferred',[]),
            'unvalidated_legacy':coverage.get('unexecuted',[]),'unsupported':coverage.get('unsupported',[]),'progress':coverage.get('progress')},
            'domain':coverage['domain'],'patterns':verified['verdict']['checks']['instruction'].get('declaration',{}),
            'reused_evidence':verified['manifest'].get('reused_evidence',[]),
            'timings':{name:{key:report[key] for key in ('timings','elapsed_seconds','total_seconds','warm_replay_seconds','feedback_target_overrun','warm_target_overrun') if key in report}
                for name,report in verified['verdict']['checks'].items()},'source_sha256':verified['manifest']['identities']['source']['sha256']}

    def baseline(self):
        from readiness import source_inventory
        from readiness_orchestration import run_readiness
        pointer=self.root/'out/accepted.json';predecessor=None;recovery=None
        try:baseline=self.accepted(False);predecessor=Path(baseline['run_path'])
        except Exception as error:
            if pointer.exists():recovery=record_rejected_pointer(self.root,str(error))
            baseline=None
        for relative in ('rom/sm.sfc','movies/super-metroid-any-3653.lsmv'):
            if not (self.root/relative).is_file():
                return {'status':'error','reason':'Missing required input: '+str(self.root/relative),'recovery_path':str(recovery) if recovery else None}
        # A verified old baseline with changed current source requires refreshed
        # exact gates before selection. Invalid pointers provide no reusable lineage.
        changed=baseline is not None and source_inventory(self.root)!=json.loads((Path(baseline['run_path'])/'source/inventory.json').read_text())['files']
        if baseline is None or changed:
            code=run_readiness(self.root,predecessor,self.original_run,self.timeout,self.jobs,False,rejected_pointer=recovery)
            if code!=0:
                runs=sorted((self.root/'out/runs').iterdir())
                latest=runs[-1] if runs else None
                return {'status':'incomplete' if code==1 else 'error','reason':'Required readiness refresh failed; inspect retained setup/gate evidence',
                    'run_path':str(latest) if latest else None,'recovery_path':str(recovery) if recovery else None}
            baseline=self.accepted()
        return self.accepted()

    def run(self,name,payload):
        if name=='baseline':return self.baseline()
        if name=='select':
            from iteration_selection import main
            before={path.name for path in (self.root/'out/runs').iterdir()}
            code=main(['--root',str(self.root),'--predecessor',payload['baseline']['run_path']]+(['--timeout',str(self.timeout)] if self.timeout else []))
            added=[path for path in (self.root/'out/runs').iterdir() if path.name not in before]
            owner=added[-1] if added else None
            if code!=0:return {'status':'error','reason':'Measured selection failed','selection_path':str(owner)}
            report=json.loads((owner/'selection.json').read_text());report['selection_path']=str(owner);return report
        if name=='prepare':
            selection=payload['selection'];owner=Path(selection['selection_path'])
            if selection['outcome']=='supported-routine':
                from iteration_increment import prepare
                preparation=prepare(self.root,owner,self.timeout)
            else:
                from iteration_pattern import proof,prepare
                observed=selection.get('observed_unsupported_patterns',[])
                if not any(row.get('pattern')=='LDA/Abs' and row.get('count',0)>0 for row in observed):
                    return {'status':'decision-required','reason':'Covered unsupported work needs one bounded implemented/proved pattern; available prospective LDA absolute has no observed execution',
                        'observed_patterns':observed}
                proved=proof(self.root,'ad',self.timeout)
                preparation=prepare(self.root,owner,proved,self.timeout)
            return {'preparation_path':str(preparation),'route':selection['outcome']}
        if name=='retry':
            from iteration_diagnosis import verify_history
            history=verify_history(self.root,payload['increment_id'])
            if history and history[-1]['decision_required']:
                return {'published':False,'status':'decision-required','reason':history[-1]['next_experiment'],'diagnostics':history[-1],
                    'candidate_path':payload['candidate']['candidate_path']}
            baseline=self.accepted()
            if baseline['run_id']!=payload['baseline']['run_id']:raise ValueError('Accepted predecessor changed during unresolved increment')
            candidate=Path(payload['candidate']['candidate_path'])
            manifest=json.loads((candidate/'manifest.json').read_text());increment=dict(manifest['increment'])
            from readiness import source_inventory,read_verdict
            previous=json.loads((candidate/'source/inventory.json').read_text())['files']
            if source_inventory(self.root)==previous:
                if history and history[-1]['next_action']=='prune-unexecuted-keys':
                    from iteration_increment import prune_uncovered,apply_preparation
                    proposal=prune_uncovered(self.root,candidate,self.timeout)
                    increment=dict(json.loads((proposal/'manifest.json').read_text())['increment'])
                    if hasattr(self,'journal'):self.journal_update(increment=increment)
                    apply_preparation(self.root,proposal)
                elif read_verdict(candidate).get('status')=='incomplete':
                    return self.validate_candidate(increment,payload)
                else:
                    return {'published':False,'status':'repair-needed','candidate_path':str(candidate),
                        'reason':history[-1]['next_experiment'] if history else 'Apply the bounded affected-layer repair or new discriminating experiment before resume',
                        'diagnostics':history[-1] if history else None}
            elif increment.get('kind')=='supported-expansion':
                from iteration_increment import prepare_repair
                prepared=prepare_repair(self.root,candidate,self.timeout)
                increment=dict(json.loads((prepared/'manifest.json').read_text())['increment'])
            increment.update(session_id=payload['session_id'],increment_id=payload['increment_id'])
            return self.validate_candidate(increment,payload)
        if name=='validate':
            baseline=self.accepted()
            if baseline['run_id']!=payload['baseline']['run_id']:raise ValueError('Current accepted predecessor changed before candidate mutation')
            preparation=payload['preparation']
            from iteration_increment import apply_preparation
            if preparation.get('route')=='needs-timed-pattern':
                from iteration_pattern import apply_preparation
            increment=dict(json.loads((Path(preparation['preparation_path'])/'manifest.json').read_text())['increment'])
            increment.update(session_id=payload['session_id'],increment_id=payload['increment_id'])
            if hasattr(self,'journal'):
                increment['operation_id']=json.loads(self.journal.read_text())['operation_id']
                self.journal_update(increment=increment)
            apply_preparation(self.root,Path(preparation['preparation_path']))
            increment.update(session_id=payload['session_id'],increment_id=payload['increment_id'])
            return self.validate_candidate(increment,payload)
        raise ValueError('Unknown session operation '+name)

    def validate_candidate(self,increment,payload):
        from readiness_orchestration import run_readiness
        before={path.name for path in (self.root/'out/runs').iterdir()}
        if hasattr(self,'journal'):
            increment['operation_id']=json.loads(self.journal.read_text())['operation_id']
            self.journal_update(increment=increment)
        code=run_readiness(self.root,Path(increment['baseline_path']),self.original_run,self.timeout,self.jobs,True,increment,
            allocated=lambda run:self.allocated(run) if hasattr(self,'journal') else None)
        added=[path for path in (self.root/'out/runs').iterdir() if path.name not in before]
        candidates=[path for path in added if json.loads((path/'manifest.json').read_text()).get('type')=='iteration']
        candidate=candidates[-1] if candidates else None
        if code!=0:
            if candidate and any(row.get('reason')=='Stage interrupted' for row in json.loads((candidate/'verdict.json').read_text()).get('checks',{}).values()):
                return {'published':False,'status':'stopped','candidate_path':str(candidate),'reason':'User stopped validation; logs and prior acceptance preserved'}
            from iteration_diagnosis import verify_history
            history=verify_history(self.root,payload['increment_id']);latest=history[-1] if history else None
            return {'published':False,'status':'decision-required' if latest and latest['decision_required'] else 'repair-needed',
                'candidate_path':str(candidate),'reason':latest['next_experiment'] if latest else 'Candidate validation failed; preserve evidence and repair through affected layer',
                'diagnostics':latest}
        accepted=self.accepted()
        if candidate is None or accepted['run_id']!=candidate.name:
            raise ValueError('Published result is not this session candidate; competing acceptance cannot count')
        acceptance=json.loads((candidate/'acceptance-result.json').read_text())
        if acceptance.get('accepted') is not True:raise ValueError('Validation did not publish acceptance')
        return self.published_result(candidate,increment)

    def published_result(self,candidate,increment):
        accepted=self.accepted()
        accepted['published']=True
        # Preserve source delta against accepted predecessor, not merely Git.
        from readiness import source_inventory
        old={row['path']:row['sha256'] for row in json.loads((Path(increment['baseline_path'])/'source/inventory.json').read_text())['files']}
        new={row['path']:row['sha256'] for row in json.loads((candidate/'source/inventory.json').read_text())['files']}
        accepted['source_diff']=[{'path':path,'before':old.get(path),'after':new.get(path)} for path in sorted(set(old)|set(new)) if old.get(path)!=new.get(path)]
        import difflib
        changes=[]
        for row in accepted['source_diff']:
            path=row['path'];old_path=Path(increment['baseline_path'])/'source/files'/path;new_path=candidate/'source/files'/path
            old_text=old_path.read_text(errors='replace') if old_path.exists() else ''
            new_text=new_path.read_text(errors='replace') if new_path.exists() else ''
            changes.extend(difflib.unified_diff(old_text.splitlines(keepends=True),new_text.splitlines(keepends=True),fromfile='accepted/'+path,tofile='candidate/'+path))
        accepted['source_diff_text']=''.join(changes)
        return accepted


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent)
    request=parser.add_mutually_exclusive_group()
    request.add_argument('--count',type=int)
    request.add_argument('--target-generated',type=int)
    parser.add_argument('--resume')
    parser.add_argument('--original-run',type=Path)
    parser.add_argument('--timeout',type=float)
    parser.add_argument('--jobs',type=int,default=4)
    args=parser.parse_args(argv)
    try:
        if args.timeout is not None and args.timeout<=0:raise ValueError('Timeout must be positive')
        if args.jobs<=0:raise ValueError('Jobs must be positive')
        if args.resume and (args.count is not None or args.target_generated is not None):raise ValueError('Resume preserves the original request')
        session=Session.resume(args.root,args.resume) if args.resume else Session.create(args.root,count=args.count if args.count is not None else None if args.target_generated is not None else 1,target=args.target_generated)
        print('Session: '+str(session.path),flush=True)
        result=session.run(ProductionOperations(args.root,args.original_run,args.timeout,args.jobs))
        print('Brief: '+str(session.path/'brief.md'),flush=True)
        return 0 if result['status']=='complete' else 1 if result['status'] in ('repair-needed','decision-required','stopped') else 2
    except Exception as error:print('Iteration session failed: '+str(error),file=sys.stderr);return 2


if __name__=='__main__':raise SystemExit(main())
