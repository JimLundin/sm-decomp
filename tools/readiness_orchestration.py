#!/usr/bin/env python3
"""Complete readiness gates, atomic acceptance, diagnostic replay and isolated recovery."""
import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
import time
import uuid
from pathlib import Path
from readiness import Run, digest, inventory_digest, preflight, read_verdict, source_inventory, write_json
from readiness_contract import DOMAIN, MANDATORY_CHECKS, validate_gate_scope
from readiness_reuse import (accepted_pointer, apply_reuse, plan_reuse, record_dependency_policy,
                              seal_gate_evidence, verify_accepted, verify_check)


def source_snapshot(owner):
    owner=Path(owner)
    manifest=json.loads((owner/'manifest.json').read_text())
    inventory=json.loads((owner/'source/inventory.json').read_text())
    if inventory.get('schema_version')!=1 or inventory_digest(inventory['files'])!=inventory.get('sha256') or inventory['sha256']!=manifest['identities']['source']['sha256']:
        raise ValueError('Source inventory identity differs')
    for entry in inventory['files']:
        relative=Path(entry['path']);path=owner/'source/files'/relative
        if relative.is_absolute() or '..' in relative.parts or path.is_symlink() or digest(path)!=entry['sha256'] or path.stat().st_mode&0o777!=entry['mode']:
            raise ValueError('Source snapshot is unsafe or corrupt: '+str(relative))
    return manifest,inventory


def restore(owner,destination,accepted=False):
    owner=Path(owner).resolve();destination=Path(destination).resolve()
    if destination.exists():raise ValueError('Restore destination must not already exist')
    manifest,inventory=source_snapshot(owner)
    if accepted:
        verified=verify_accepted(owner,require_pointer=True);verify_origins(owner,verified)
    elif not (owner/'verdict.json').is_file() or manifest.get('state')!='complete':
        raise ValueError('Selected run has no terminal preserved snapshot')
    if read_verdict(owner).get('run_id')!=manifest['run_id']:raise ValueError('Selected snapshot has invalid terminal verdict')
    saved=Run(owner.parent.parent.parent,owner,manifest)
    saved.restore_source(destination)
    subprocess.run(['git','init','-q',str(destination)],check=True)
    print('Restored: '+str(destination),flush=True)
    return destination


def publish_acceptance(owner,root=None,rejected_pointer=None):
    """Reverify a complete run and current source under the acceptance lock."""
    owner=Path(owner).resolve();root=Path(root).resolve() if root else owner.parent.parent.parent
    output=root/'out';output.mkdir(exist_ok=True)
    with (output/'acceptance.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX)
        verified=verify_accepted(owner)
        verify_origins(owner,verified)
        if source_inventory(root)!=verified['inventory']['files']:
            raise ValueError('Current source differs from the proposed accepted snapshot')
        inputs=verified['manifest']['identities']['inputs']
        for path,name in ((root/'rom/sm.sfc','rom_sha256'),(root/'movies/super-metroid-any-3653.lsmv','movie_sha256')):
            if digest(path)!=inputs[name]:raise ValueError('Current input differs before acceptance: '+str(path))
        previous=output/'accepted.json';history=output/'accepted-history';history.mkdir(exist_ok=True)
        if rejected_pointer is not None:
            from iteration_session import verify_recovery_pointer
            preserved=verify_recovery_pointer(root,rejected_pointer)
            if verified['manifest'].get('accepted_parent') is not None:
                raise ValueError('Recovered fresh readiness cannot inherit rejected pointer lineage')
            rejected=output/'rejected-pointers';rejected.mkdir(exist_ok=True)
            retained=rejected/(Path(rejected_pointer).name+'-'+uuid.uuid4().hex[:8]+'.bin')
            with retained.open('xb') as handle:handle.write(preserved.read_bytes());handle.flush();os.fsync(handle.fileno())
        elif previous.exists():
            old=json.loads(previous.read_text());verify_accepted(old['run_path'],require_pointer=True)
            if verified['manifest'].get('accepted_parent')!=old['run_id']:
                raise ValueError('Accepted predecessor changed during validation')
            retained=history/(old['run_id']+'-'+uuid.uuid4().hex[:8]+'.json')
            with retained.open('xb') as handle:handle.write(previous.read_bytes());handle.flush();os.fsync(handle.fileno())
        elif verified['manifest'].get('accepted_parent') is not None:
            raise ValueError('Declared accepted predecessor is no longer current')
        # Rehash the sealed terminal evidence immediately before publication.
        verified=verify_accepted(owner);verify_origins(owner,verified)
        if source_inventory(root)!=verified['inventory']['files']:raise ValueError('Source changed before acceptance')
        for path,name in ((root/'rom/sm.sfc','rom_sha256'),(root/'movies/super-metroid-any-3653.lsmv','movie_sha256')):
            if digest(path)!=inputs[name]:raise ValueError('Input changed before acceptance')
        if rejected_pointer is not None:
            verify_recovery_pointer(root,rejected_pointer)
        temporary=output/('accepted-'+uuid.uuid4().hex+'.tmp')
        with temporary.open('x') as handle:
            json.dump(accepted_pointer(owner,verified),handle,sort_keys=True,indent=2);handle.write('\n');handle.flush();os.fsync(handle.fileno())
        os.replace(temporary,previous)
        descriptor=os.open(output,os.O_DIRECTORY)
        try:os.fsync(descriptor)
        finally:os.close(descriptor)
    return previous


def missing(run,name,reason,status='incomplete'):
    if name not in run.checks:run.record_check(name,{'schema_version':1,'status':status,'complete':False,'reason':reason})
    return run.checks[name]


def outcome(run):
    failures=[]
    for name in MANDATORY_CHECKS:
        report=run.checks.get(name,{})
        if report.get('status')=='pass' and report.get('complete') is True:
            try:validate_gate_scope(name,report)
            except ValueError as error:failures.append(('incomplete',name,str(error)))
        else:failures.append((report.get('status','incomplete'),name,report.get('reason','Mandatory gate did not complete')))
    for name,report in run.checks.items():
        if name not in MANDATORY_CHECKS and (report.get('status')!='pass' or report.get('complete') is not True):
            failures.append((report.get('status','incomplete'),name,report.get('reason','Recorded stage did not complete')))
    if not failures:return 'pass','readiness','Every mandatory gate completed and passed'
    order={'error':0,'incomplete':1,'divergence':2,'uncovered':3}
    status,name,reason=min(failures,key=lambda row:order.get(row[0],0))
    return status if status in order else 'incomplete',name,reason


def brief(run,status,reason):
    source=run.manifest.get('identities',{}).get('source',{}).get('sha256','unavailable')
    coverage=run.checks.get('changed-entry-coverage',{})
    text='{}: {}\n\nSource: `{}`\nDomain: Japan/USA any%, input indices 0..17999, clean power-on, seed 0, P1, lsnes-profile.\n\n{}\n\n'.format('Validation passed; acceptance pending' if status=='pass' else 'Not ready',status,source,reason)
    text+='| Gate | Outcome | Evidence |\n| --- | --- | --- |\n'
    for name in MANDATORY_CHECKS:
        report=run.checks[name];scope=''
        if 'completed_frames' in report:scope=' ({}/{})'.format(report['completed_frames'],report.get('expected_frames',report.get('completed_frames')))
        elif 'run' in report:scope=' ({}/{})'.format(report['run'],report.get('expected'))
        text+='| {} | {}{} | [report](checks/{}.json) |\n'.format(name,report['status'],scope,name)
    text+='\nActual generated instructions: {}; interpreted: {}; validated distinct keys: {}; fallback fraction: {}.\n'.format(coverage.get('generated_instructions','unavailable'),coverage.get('interpreted_instructions','unavailable'),coverage.get('validated_distinct_keys','unavailable'),coverage.get('fallback_fraction','unavailable'))
    text+='Progress: {}.\n'.format(json.dumps(coverage.get('progress',{'comparable':False,'reason':'No comparable coverage'}),sort_keys=True))
    if run.manifest.get('increment'):
        text+='Increment: {}. [Observed progress](checks/increment-progress.json) · [Preserved preparation]({}/brief.md).\n'.format(json.dumps(run.manifest['increment'],sort_keys=True),run.manifest['increment'].get('preparation_path','unavailable'))
    declaration=run.checks.get('instruction',{}).get('declaration',{})
    text+='Supported native patterns: {}. ADC overflow remains unsupported debt. Historical later-domain synchronization is noncertifying.\n\nTargets: cold replay 1200s, warm focus 60s. Overruns are reported; timeouts are incomplete.\n'.format(', '.join(row['mnemonic']+'/'+row['mode'] for row in declaration.get('patterns',[])) or 'see preserved instruction evidence')
    for name in ('fallback-only-parity','generated-parity','instruction-diagnosis'):
        report=run.checks[name]
        text+='{}: total {}s, warm {}s, target overrun {}.\n'.format(name,report.get('total_seconds','unavailable'),report.get('warm_replay_seconds','unavailable'),report.get('warm_target_overrun',report.get('feedback_target_overrun','unavailable')))
    text+='\n| Timed work | Seconds |\n| --- | --- |\n'
    for name in ('check','lint','pattern-validation','sync-reference-build','sync-reference-replay','feedback-build'):
        report=run.checks.get(name,{})
        text+='| {} | {} |\n'.format(name,report.get('elapsed_seconds','unavailable'))
    for mode in ('fallback-only','generated'):
        for name,seconds in run.checks.get(mode+'-parity',{}).get('timings',{}).items():
            text+='| {} {} | {} |\n'.format(mode,name,seconds)
    for index,report in enumerate(run.checks.get('instruction-diagnosis',{}).get('reports',[])):
        text+='| Paired cache {} creation | {} |\n'.format(index,report.get('cache',{}).get('cache_creation_seconds','unavailable'))
        text+='| Focus {} warm replay | {} |\n'.format(index,report.get('warm_replay_seconds','unavailable'))
    text+='| Feedback demonstrations | {} |\n'.format(run.checks['feedback-demonstrations'].get('total_seconds','unavailable'))
    from datetime import datetime
    text+='| Total suite through verdict | {:.3f} |\n'.format((datetime.now().astimezone()-datetime.fromisoformat(run.manifest['started_utc'])).total_seconds())
    debt=run.manifest.get('historical_debt')
    if debt:text+='\n[Later hardware debt and identities]({}): label mismatch 18,195 versus 18,196; later X=44 versus 43 at frame 35,203. Later gameplay, low%, 100%, different ROM/reset, full completion and zero interpreter execution require separate validation.\n'.format(debt)
    text+='\n[Manifest](manifest.json) · [Verdict](verdict.json) · [Coverage](coverage.json) · [Logs](logs/) · [Acceptance](acceptance-result.json)\n'
    return text


def run_readiness(root,predecessor=None,original_run=None,timeout=None,jobs=4,iteration=False,increment=None,rejected_pointer=None,allocated=None):
    if iteration:
        from iteration_increment import require_increment
        require_increment(increment)
    root=Path(root).resolve()
    pointer=root/'out/accepted.json'
    if predecessor is None and pointer.exists() and rejected_pointer is None:
        try:predecessor=Path(json.loads(pointer.read_text())['run_path'])
        except (OSError,ValueError,KeyError):predecessor=pointer
    run=Run.create(root,'iteration' if iteration else 'readiness',dict(DOMAIN),Path(predecessor).name if predecessor else None,
        metadata={name:increment[name] for name in ('session_id','increment_id','operation_id') if increment and increment.get(name)})
    if rejected_pointer is not None:
        run.manifest['rejected_pointer_recovery']=str(rejected_pointer);run.save_manifest()
    if increment is not None:
        run.manifest['increment']=increment;run.save_manifest()
        for name in ('session_id','increment_id','operation_id'):
            if increment.get(name):run.manifest[name]=increment[name]
        run.save_manifest()
    if allocated is not None:allocated(run)
    print('Run: '+str(run.path),flush=True);print('Brief: '+str(run.path/'brief.md'),flush=True)
    try:
        failures=preflight(run)
        run.stage_timeout=timeout
        if iteration and predecessor is None:failures.append('Iteration validation requires an accepted predecessor')
        if predecessor is not None:
            try:verify_accepted(predecessor,require_pointer=True)
            except Exception as error:failures.append('Predecessor verification failed: '+str(error))
        if failures:
            for name in MANDATORY_CHECKS:missing(run,name,'Preflight/predecessor failed: '+'; '.join(failures))
        else:
            record_dependency_policy(run);decisions=plan_reuse(run,predecessor)
            write_json(run.path/'checks/reuse-decisions.json',decisions)
            def gate(name,operation):
                try:return operation()
                except KeyboardInterrupt:raise
                except Exception as error:
                    failed=next((report for report in reversed(list(run.checks.values())) if report.get('status') in ('incomplete','error')),None)
                    return missing(run,name,str(error),failed['status'] if failed else 'error')
            for name in ('check','lint'):
                gate(name,lambda name=name:apply_reuse(run,predecessor,name) if decisions[name]['action']=='reuse' else run.run_stage(name,['just',name],timeout=timeout))
            gate('instruction',lambda:pattern_gates(run,timeout))
            missing(run,'timing','Pattern instruction/timing process did not complete')
            from readiness_parity import run_parity
            for mode in ('fallback-only','generated'):
                name=mode+'-parity'
                gate(name,lambda mode=mode,name=name:apply_reuse(run,predecessor,name) if mode=='fallback-only' and decisions[name]['action']=='reuse' else run_parity(run,mode,18000,timeout,predecessor if iteration and mode=='generated' else None))
            missing(run,'changed-entry-coverage','Generated replay produced no complete coverage')
            from readiness_sync import synchronization
            gate('synchronization',lambda:apply_reuse(run,predecessor,'synchronization') if predecessor and decisions['synchronization']['action']=='reuse' else synchronization(run,original_run,18000,jobs))
            gate('instruction-diagnosis',lambda:focused_gates(run,timeout))
            missing(run,'checkpoint-validation','Focused candidate did not complete validated checkpoint creation')
            if iteration:
                from iteration_increment import progress_gate
                gate('increment-progress',lambda:progress_gate(run))
            gate('feedback-demonstrations',lambda:feedback_gate(run,timeout))
            validation_failure=next((dict(report,stage=name) for name,report in run.checks.items() if name in ('generated-parity','changed-entry-coverage','synchronization','instruction-diagnosis') and report.get('status') in ('divergence','uncovered')),None)
            if validation_failure:
                from readiness_controls import diagnose_failure
                gate('failure-controls',lambda:diagnose_failure(run,validation_failure,predecessor,timeout=timeout))
        for name in MANDATORY_CHECKS:missing(run,name,'Mandatory gate did not run')
        run.check_sources();status,stage,reason=outcome(run)
        summary=brief(run,status,reason);seal_gate_evidence(run)
        if status=='pass':verify_origins(run.path,{'manifest':run.manifest,'verdict':{'checks':run.checks},'inventory':{'files':run.source_files}})
        code=run.finalize(status,stage,reason,MANDATORY_CHECKS,brief_text=summary)
        if iteration and code!=0 and increment and increment.get('session_id') and increment.get('increment_id'):
            from iteration_diagnosis import append_experiment
            try:
                history=append_experiment(root,increment['session_id'],increment['increment_id'],run.path,
                    'Selected increment changes the first observed execution effect',
                    'Matched predecessor/disabled controls isolate changed instruction or shared path',
                    'Candidate validation '+status+' at '+str(stage))
                write_json(root/'out/iteration-history'/increment['increment_id']/(run.manifest['run_id']+'-diagnosis.json'),{'experiment_run':history['run_path'],'next_action':history['next_action'],'decision_required':history['decision_required']})
            except Exception as error:
                write_json(root/'out/iteration-history'/increment['increment_id']/(run.manifest['run_id']+'-diagnosis.json'),{'status':'error','reason':str(error)})
        if code==0:
            try:
                pointer=publish_acceptance(run.path,root,rejected_pointer);write_json(run.path/'acceptance-result.json',{'status':'pass','accepted':True,'pointer':str(pointer)})
            except Exception as error:
                write_json(run.path/'acceptance-result.json',{'status':'error','accepted':False,'reason':str(error)});print('Acceptance failed: '+str(error),file=sys.stderr);return 2
        return code
    except (Exception,KeyboardInterrupt) as error:
        for name in MANDATORY_CHECKS:missing(run,name,'Interrupted' if isinstance(error,KeyboardInterrupt) else str(error))
        status='incomplete' if isinstance(error,KeyboardInterrupt) else 'error'
        summary=brief(run,status,str(error));code=run.finalize(status,'readiness',str(error),MANDATORY_CHECKS,brief_text=summary);return code


def main(argv=None):
    arguments=list(sys.argv[1:] if argv is None else argv)
    if arguments[:1]==['replay']:
        from readiness_diagnostics import main as replay
        previous=sys.argv;sys.argv=[previous[0]]+arguments[1:]
        try:return replay()
        finally:sys.argv=previous
    parser=argparse.ArgumentParser(description=__doc__);sub=parser.add_subparsers(dest='command',required=True)
    for name in ('readiness','validate-iteration'):
        command=sub.add_parser(name);command.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent)
        command.add_argument('--predecessor',type=Path);command.add_argument('--original-run',type=Path);command.add_argument('--timeout',type=float);command.add_argument('--jobs',type=int,default=4)
    sub.choices['validate-iteration'].add_argument('--increment',type=Path,help='Explicit sealed preparation metadata JSON')
    command=sub.add_parser('restore');command.add_argument('--run',type=Path,required=True);command.add_argument('--destination',type=Path,required=True);command.add_argument('--accepted',action='store_true')
    command=sub.add_parser('accept');command.add_argument('--run',type=Path,required=True)
    command=sub.add_parser('verify-patterns');command.add_argument('--run',type=Path,required=True);command.add_argument('--output',type=Path,required=True);command.add_argument('--development-scope',action='store_true')
    command=sub.add_parser('replay');command.add_argument('arguments',nargs=argparse.REMAINDER)
    args=parser.parse_args(argv)
    try:
        if args.command=='verify-patterns':
            manifest=json.loads((args.run/'manifest.json').read_text());verify_pattern_evidence(args.output.resolve(),manifest,not args.development_scope);print('Verified pattern evidence');return 0
        if args.command=='restore':restore(args.run,args.destination,args.accepted);return 0
        if args.command=='accept':print('Accepted: '+str(publish_acceptance(args.run)));return 0
        if args.command=='replay':
            from readiness_diagnostics import main as replay
            previous=sys.argv;sys.argv=[previous[0]]+args.arguments
            try:return replay()
            finally:sys.argv=previous
        return run_readiness(args.root,args.predecessor,args.original_run,args.timeout,args.jobs,args.command=='validate-iteration',json.loads(args.increment.read_text()) if getattr(args,'increment',None) else None)
    except (Exception,KeyboardInterrupt) as error:print(str(error),file=sys.stderr);return 2


def verify_pattern_evidence(output,manifest,strict=True):
    from readiness_patterns import OPCODES
    output=Path(output);report=json.loads((output/'instruction.json').read_text())
    if report.get('source_sha256')!=manifest['identities']['source']['sha256'] or inventory_digest(report.get('source_inventory',[]))!=report.get('source_sha256'):
        raise ValueError('Pattern source identity differs from parent')
    if report.get('compiler','').strip()!=manifest['identities']['compiler']['version'].strip():raise ValueError('Pattern compiler differs')
    if report.get('status')!='pass' or report.get('failed')!=0 or report.get('skipped')!=0 or report.get('passed')!=report.get('expected'):raise ValueError('Pattern successful terminal counts missing')
    if strict:validate_gate_scope('instruction',dict(report,complete=True))
    declaration = None
    from pattern_contract import DECLARATION
    candidate_declared = any(row['path'] == DECLARATION for row in report.get('source_inventory', []))
    if candidate_declared and report.get('schema_version') != 2:
        raise ValueError('Candidate declaration cannot downgrade pattern evidence contract')
    if report.get('schema_version') == 2:
        from pattern_contract import load_declaration, identity, DECLARATION, FIXTURES
        snapshot = output.parent.parent / 'source/files'
        if snapshot.is_dir():
            declaration = load_declaration(snapshot, prospective=report.get('prospective_opcode'))
        else:
            # Standalone verification must still bind the declaration to the
            # preserved source inventory, even without a parent source copy.
            raw_declaration=json.loads((output/'declaration.json').read_text())
            from pattern_contract import project_declaration
            declaration=project_declaration(raw_declaration,report.get('prospective_opcode'))
        if report['declaration'] != declaration or identity(declaration) != report['declaration_sha256']:
            raise ValueError('Pattern declaration differs from candidate source')
        declared_source = next((row for row in report['source_inventory'] if row['path'] == DECLARATION), {})
        fixture_source = next((row for row in report['source_inventory'] if row['path'] == FIXTURES), {})
        # Raw declaration identity is recorded separately from canonical JSON.
        if declared_source.get('sha256') != report.get('declaration_file_sha256') or fixture_source.get('sha256') != report.get('fixtures_sha256'):
            raise ValueError('Pattern declaration/fixture source binding differs')
        if digest(output/'declaration.json') != declared_source['sha256'] or digest(output/'independent-fixtures.json') != fixture_source['sha256']:
            raise ValueError('Pattern preserved source declaration/fixtures differ')
    generator=output/report['generator']['path']
    if digest(generator)!=report['generator']['sha256']:raise ValueError('Pattern generator executable differs')
    expected=[json.loads(line) for line in (output/'expected-cases.jsonl').read_text().splitlines()]
    rows=[json.loads(line) for line in (output/'cases.jsonl').read_text().splitlines()]
    if digest(output/'expected-cases.jsonl')!=report['expected_inventory_sha256'] or len(expected)!=report['expected'] or len(rows)!=report['run']:
        raise ValueError('Pattern expected/completed ledger differs')
    if len(rows)!=len(expected):raise ValueError('Pattern ledger truncated')
    if declaration:
        from pattern_contract import identity, pattern_conditions
        patterns={row['opcode']:row for row in declaration['patterns']}
        fixtures={case['name']:case for case in json.loads((output/'independent-fixtures.json').read_text())}
        complete=[]
        for corpus in report['corpus']:
            path=output/'corpus'/(corpus['opcode']+'.n.json')
            if digest(path)!=corpus['sha256']:raise ValueError('Pattern preserved complete corpus differs')
            cases=json.loads(path.read_text())
            if len(cases)!=corpus['cases']:raise ValueError('Pattern available corpus count differs')
            for case in cases:complete.append((corpus['opcode'],case))
        for item in report['corpus']:
            opcode=item['opcode']
            for name in patterns[opcode]['independent_fixtures']:
                complete.append((opcode,fixtures[name]))
        reconstructed=[{'name':case['name'],'case_sha256':identity(case),'opcode':opcode,'conditions':pattern_conditions(case,patterns[opcode])} for opcode,case in complete]
        if expected!=reconstructed:raise ValueError('Pattern ordered complete corpus/independent fixture scope differs')
    combined=[]
    for batch in sorted(output.glob('batch-*')):
        identity=json.loads((batch/'identities.json').read_text())
        for path,name in ((batch/'bodies.rs','bodies_sha256'),(batch/'prepared.json','prepared_sha256'),(batch/identity['binary'],'binary_sha256'),(batch/'input.json','input_sha256'),(batch/'cases.jsonl','ledger_sha256')):
            if path.parent!=batch or digest(path)!=identity[name]:raise ValueError('Pattern batch artifact changed: '+str(path))
        process=json.loads((batch/'execute.json').read_text())
        for kind in ('execute','generate','build'):
            saved=json.loads((batch/(kind+'.json')).read_text())
            if saved.get('signal') is not None or saved.get('complete') is not True:raise ValueError('Pattern process interrupted')
            for field in ('stdout','stderr'):
                if not (batch/saved[field]).is_file():raise ValueError('Pattern process log missing')
        if Path(process['command'][0])!=batch/identity['binary'] or process['command'][1:]!=[str(batch/'prepared.json'),str(batch/'cases.jsonl')] or process['returncode'] not in (0,1):
            raise ValueError('Pattern executed command differs from preserved executable')
        generated=json.loads((batch/'generate.json').read_text())
        if Path(generated['command'][0])!=generator or generated['returncode']!=0 or json.loads((batch/'build.json').read_text())['returncode']!=0:
            raise ValueError('Pattern generation/build did not complete')
        combined.extend(json.loads(line) for line in (batch/'cases.jsonl').read_text().splitlines())
        if declaration:
            from pattern_contract import identity, pattern_conditions
            inputs=json.loads((batch/'input.json').read_text())
            patterns={row['opcode']:row for row in declaration['patterns']}
            offset=len(combined)-len(inputs)
            for case,expected_case in zip(inputs,expected[offset:]):
                opcode=case['opcode']
                fields={key:value for key,value in case.items() if key not in ('case_sha256','opcode','conditions','independent_fixture')}
                if identity(fields)!=case['case_sha256'] or expected_case.get('opcode')!=opcode or expected_case.get('conditions')!=pattern_conditions(case,patterns[opcode]):
                    raise ValueError('Pattern input identity/declared conditions differ')
    if combined!=rows:raise ValueError('Pattern batch ledger differs from combined ledger')
    counts={opcode:0 for opcode in report['opcodes']}
    coverage={opcode:{} for opcode in counts}
    for expected_row,row in zip(expected,rows):
        if row.get('name')!=expected_row['name'] or row.get('case_sha256')!=expected_row['case_sha256'] or row.get('status')!='pass':raise ValueError('Pattern ordered case identity/outcome differs')
        if row['opcode'] not in counts:raise ValueError('Unexpected validated opcode')
        counts[row['opcode']]+=1
        for condition in expected_row.get('conditions', []):
            coverage[row['opcode']][condition]=coverage[row['opcode']].get(condition,0)+1
    if any(report['opcodes'][key]['run']!=count for key,count in counts.items() if key in report['opcodes']):raise ValueError('Pattern opcode counts differ')
    if declaration:
        for opcode, conditions in report['coverage'].items():
            if any(value.get('expected') != coverage[opcode].get(condition,0) or value.get('passed') != coverage[opcode].get(condition,0) for condition,value in conditions.items()):
                raise ValueError('Pattern declared condition ledger differs')
    return report


def pattern_gates(run,timeout=None):
    from pattern_contract import load_declaration
    declaration=load_declaration(run.root)
    corpus=run.root/'deps/65816/v1'
    for pattern in declaration['patterns']:run.guard_input(corpus/pattern['corpus'])
    output=run.path/'checks/patterns'
    stage=run.run_stage('pattern-validation',[sys.executable,run.root/'tools/readiness_patterns.py','--out',output,'--timing'],timeout=timeout,exit_statuses={1:'divergence',2:'error'})
    raw_path=output/'instruction.json'
    if not raw_path.is_file():
        missing(run,'instruction','No pattern completion report',stage['status'] if stage['status']!='pass' else 'incomplete');missing(run,'timing','No timing completion report');return run.checks['instruction']
    report=json.loads(raw_path.read_text());report['complete']=report.get('run')==report.get('expected') and report.get('status') in ('pass','divergence')
    if stage['status'] not in ('pass','divergence'):report.update(status=stage['status'],complete=False,reason=stage.get('reason','Pattern process did not complete'))
    if report.get('status')=='pass':
        verify_pattern_evidence(output,run.manifest)
        dependency=next(item for item in json.loads((run.root/'tools/dependencies.json').read_text()) if item['name']=='65816')
        if report['pinned_dependency']!=dependency:raise ValueError('Pattern corpus provenance differs')
        for item in report['corpus']:
            cases=json.loads((corpus/(item['opcode']+'.n.json')).read_text())
            if digest(corpus/(item['opcode']+'.n.json'))!=item['sha256'] or item['cases']!=len(cases):raise ValueError('Pattern consumed corpus identity/count differs')
    report['evidence']='checks/patterns/instruction.json';run.record_check('instruction',report)
    timing_path=output/'timing.json'
    if timing_path.is_file():
        timing=json.loads(timing_path.read_text());timing['evidence']='checks/patterns/timing.json'
        if timing.get('status')=='pass':verify_timing(output,timing,run.path/'source/files')
        run.record_check('timing',timing)
    else:missing(run,'timing','Timing validation did not finish')
    for index,path in enumerate(sorted(output.rglob('*'))):
        if path.is_file() and 'build' not in path.relative_to(output).parts:
            identity=run.record_artifact('pattern-artifact-'+str(index),path);identity['evidence']=path.relative_to(run.path).as_posix()
    run.save_manifest();run.check_sources();return report


def verify_timing(output,report,source):
    validate_gate_scope('timing',report)
    process_path=Path(output)/report['process_evidence'];process=json.loads(process_path.read_text())
    expected=['cargo','test','-p','snes','--test','integration','--test','irq_mask_timing','--test','interrupt_timing','--test','dma_timing','--test','hdma_timing','--test','generated_bus_timing','--locked','--offline']
    if report.get('schema_version') == 2:
        from pattern_contract import load_declaration, identity
        declaration=load_declaration(source,prospective=report.get('prospective_opcode'))
        if declaration != report['declaration'] or identity(declaration) != report['declaration_sha256']:
            raise ValueError('Timing declaration differs from candidate source')
        expected=['cargo','test','-p','snes']
        for test in declaration['timing_tests']:expected.extend(['--test',test])
        expected.extend(['--locked','--offline'])
    if digest(process_path)!=report['process_sha256'] or process.get('command')!=expected or process.get('returncode')!=0 or process.get('complete') is not True:
        raise ValueError('Timing actual process scope incomplete')
    for name in ('stdout','stderr'):
        path=Path(output)/process[name]
        if not path.is_file():raise ValueError('Timing process log missing')
    for name,identity in report['oracles'].items():
        if digest(Path(source)/name)!=identity:raise ValueError('Timing oracle differs from frozen source')


def focused_gates(run,timeout=None):
    from readiness_diagnostics import diagnose,discover_intervals
    coverage=run.checks.get('changed-entry-coverage',{})
    keys=[{'address':row['address'],'mx':row['mx']} for row in coverage.get('required_keys',[])]
    if not keys and coverage.get('executed'):keys=[{'address':coverage['executed'][0]['address'],'mx':coverage['executed'][0]['mx']}]
    if not keys:return missing(run,'instruction-diagnosis','No actual compiled execution available for focus','uncovered')
    discovery=discover_intervals(run,keys,18000,timeout)
    if discovery.get('status')!='pass':return missing(run,'instruction-diagnosis','Covered instruction discovery failed',discovery.get('status','incomplete'))
    merged=[]
    for interval in sorted(discovery['intervals'],key=lambda item:(item['start_input'],item['start_instruction'])):
        interval=dict(interval,keys=[interval['key']])
        if merged and interval['start_input']==merged[-1]['start_input'] and interval['start_instruction']<=merged[-1]['start_instruction']+merged[-1]['instructions']:
            end=max(merged[-1]['start_instruction']+merged[-1]['instructions'],interval['start_instruction']+interval['instructions'])
            merged[-1]['instructions']=end-merged[-1]['start_instruction'];merged[-1]['keys'].extend(interval['keys'])
        else:merged.append(interval)
    discovery['intervals']=merged
    reports=[];validations=[]
    for index,interval in enumerate(discovery['intervals']):
        namespace='mandatory-focus-'+str(index)
        report=diagnose(run,interval['start_input'],interval['instructions'],'generated',timeout,interval['start_instruction'],run.path/'cache'/namespace,namespace)
        reports.append(report)
        if report.get('cache',{}).get('status')=='pass':validations.append(report['cache']['validation'])
    status='pass' if reports and all(report.get('status')=='pass' and report.get('complete') is True for report in reports) else next((report.get('status','incomplete') for report in reports if report.get('status')!='pass'),'incomplete')
    result={'schema_version':1,'status':status,'complete':bool(reports) and all(report.get('complete') is True for report in reports),
            'expected_instructions':sum(report.get('expected_instructions',0) for report in reports),'completed_instructions':sum(report.get('completed_instructions',0) for report in reports),
            'intervals':discovery['intervals'],'reports':reports,'discovery_evidence':'diagnostics/discovery/discovery.json','replay':'warm',
            'warm_replay_seconds':sum(report.get('warm_replay_seconds',0) for report in reports),'total_seconds':sum(report.get('total_seconds',0) for report in reports)}
    result['warm_target_overrun']=result['warm_replay_seconds']>60
    write_json(run.path/'diagnostics/focused-summary.json',result);result['evidence']='diagnostics/focused-summary.json';run.record_check('instruction-diagnosis',result)
    if validations and len(validations)==len(reports):
        cache=dict(validations[0],validations=validations,intervals=discovery['intervals'],evidence='diagnostics/mandatory-focus-0-cache/validation.json')
        run.record_check('checkpoint-validation',cache)
    return result


def feedback_gate(run,timeout=None):
    from readiness_feedback import run_demonstrations
    report=run_demonstrations(run,timeout=timeout)
    if 'feedback-demonstrations' not in run.checks:run.record_check('feedback-demonstrations',report)
    return report


def verify_origins(owner,verified):
    from iteration_increment import verify_progress
    verify_progress(owner,verified)
    owner=Path(owner);manifest=verified['manifest'];checks=verified['verdict']['checks']
    if manifest.get('increment',{}).get('kind') == 'timed-pattern-expansion':
        from iteration_pattern import verify_increment
        verify_increment(owner,verified)
    for name in MANDATORY_CHECKS:validate_gate_scope(name,checks[name],manifest.get('contract_version',1))
    preflight=checks['preflight']
    if preflight.get('failures')!=[]:raise ValueError('Preflight did not establish setup')
    for name in ('doctor','compiler'):
        process=json.loads((owner/'logs'/(name+'.json')).read_text())
        if process.get('returncode')!=0 or process.get('status')!='pass' or process.get('complete') is not True:raise ValueError('Preflight process did not succeed: '+name)
    for name in ('input-identity','hardware-identity'):
        if checks.get(name,{}).get('status')!='pass' or checks[name].get('complete') is not True:raise ValueError('Preflight material identity not established')
    class Context:pass
    context=Context();context.root=owner.parent.parent.parent;context.path=owner;context.manifest=manifest;context.source_files=verified['inventory']['files']
    # Reconstruct expected hardware from the archive recipe without altering
    # accepted evidence; current hardware must remain the verified profile.
    if context.root.is_dir() and (context.root/'deps/rustysnes').exists():
        expected={entry['path']:entry['sha256'] for entry in manifest['identities']['hardware']['files']}
        for relative,identity in expected.items():
            if digest(context.root/'deps/rustysnes'/relative)!=identity:raise ValueError('Hardware changed before acceptance: '+relative)

    for name in ('check','lint','fallback-only-parity','synchronization'):verify_check(owner,name,verified)
    patterns=owner/'checks/patterns';verify_pattern_evidence(patterns,manifest);verify_timing(patterns,checks['timing'],owner/'source/files')
    if checks['timing'].get('schema_version') == 2 and checks['timing'].get('source_sha256') != manifest['identities']['source']['sha256']:
        raise ValueError('Timing source identity differs from accepted candidate')
    inputs=manifest['identities']['inputs']
    from readiness_reuse import safe_artifact
    entries=json.loads((owner/'source/files/movies/manifest.json').read_text());entry=next(row for row in entries if row['file']=='super-metroid-any-3653.lsmv')
    expected_inputs={'rom_sha256':entry['rom_sha256'],'movie_sha256':entry['sha256'],'frames':entry['frames'],'seed':0,'profile':'lsnes-profile','controller':'P1 SNES gamepad','region':'NTSC','reset':'clean default power-on; no reset/subframe records'}
    if any(inputs.get(name)!=value for name,value in expected_inputs.items()):raise ValueError('Preflight pinned initial-domain inputs differ')
    if digest(safe_artifact(owner,inputs['input_evidence']))!=inputs['input_sha256']:raise ValueError('Canonical input evidence differs')
    compiler=json.loads((owner/'logs/compiler.json').read_text())
    if (owner/compiler['stdout']).read_text().strip()!=manifest['identities']['compiler']['version']:raise ValueError('Compiler provenance differs')
    dependency=next(row for row in json.loads((owner/'source/files/tools/dependencies.json').read_text()) if row['name']=='rustysnes')
    hardware=manifest['identities']['hardware']
    if hardware['revision']!=dependency['revision'] or hardware['archive_sha256']!=dependency['sha256'] or inventory_digest(hardware['files'])!=hardware['source_sha256']:raise ValueError('Pinned hardware provenance differs')
    for patch in hardware['patches']:
        if digest(owner/'source/files'/patch['path'])!=patch['sha256']:raise ValueError('Hardware patch recipe differs')

    raw_path=safe_artifact(owner,checks['generated-parity']['evidence']);raw=json.loads(raw_path.read_text())
    identity=manifest['identities']['generated-parity-report'];binary_id=manifest['identities']['generated-binary'];binary=safe_artifact(owner,binary_id['evidence'])
    if digest(raw_path)!=identity['sha256'] or digest(binary)!=binary_id['sha256'] or raw['identities']['binary_sha256']!=binary_id['sha256']:
        raise ValueError('Generated parity actual executable/report identity differs')
    canonical=safe_artifact(owner,inputs['input_evidence']);selected=b''.join(canonical.read_bytes().splitlines(keepends=True)[:18000])
    import hashlib
    expected={'rom_sha256':inputs['rom_sha256'],'movie_sha256':inputs['movie_sha256'],'input_sha256':inputs['input_sha256'],
              'selected_input_sha256':hashlib.sha256(selected).hexdigest(),'compiled_rom_sha256':inputs['rom_sha256']}
    if any(raw['identities'].get(name)!=value for name,value in expected.items()) or raw.get('mode')!='generated' or raw.get('seed')!=0 or raw.get('controller')!='P1' or raw.get('profile')!='lsnes-profile' or raw.get('regions')!=['CPU+clocks','WRAM','framebuffer']:
        raise ValueError('Generated cold replay scope/input differs')
    validate_gate_scope('generated-parity',raw)
    process=json.loads((owner/'logs/generated-replay.json').read_text())
    if process.get('returncode')!=0 or process.get('complete') is not True or Path(process['command'][0]).name!=binary.name or '--generated' not in process['command'] or process['command'][process['command'].index('--frames')+1]!='18000':
        raise ValueError('Generated parity cold process did not complete expected command')
    coverage=checks['changed-entry-coverage'];profile=owner/'checks/generated-profile-bound.json';inventory=owner/'generated/key-inventory.json'
    if digest(profile)!=coverage['profile_sha256'] or digest(inventory)!=coverage['candidate_inventory_sha256'] or coverage['candidate_source_sha256']!=manifest['identities']['source']['sha256']:
        raise ValueError('Coverage actual profile/inventory/source differs')
    for item in ('game','steps'):
        generated_identity=manifest['identities']['generated-generated-'+item];path=safe_artifact(owner,generated_identity['evidence'])
        if digest(path)!=generated_identity['sha256']:raise ValueError('Generated compiled input changed')
    inventory_record=json.loads(inventory.read_text())
    from readiness_coverage import _table_rows
    step_path=safe_artifact(owner,inventory_record['steps_evidence'])
    if digest(step_path)!=inventory_record['steps_sha256'] or sorted(_table_rows(step_path.read_text()),key=lambda row:(row['address'],row['mx']))!=inventory_record['keys']:raise ValueError('Generated table/body inventory differs')
    actual_profile=json.loads((owner/raw['coverage_evidence']).read_text()) if 'coverage_evidence' in raw else json.loads((raw_path.parent/'coverage.json').read_text())
    if digest(raw_path.parent/'coverage.json')!=raw['coverage_sha256'] or actual_profile['generated_instructions']!=coverage['generated_instructions']:
        raise ValueError('Coverage generated replay totals differ')
    from readiness_coverage import evaluate_coverage
    context.checks={};context.record_check=lambda name,report:context.checks.setdefault(name,report)
    context.check_sources=lambda:None
    # Evaluation writes evidence, so execute its public verifier in an isolated
    # copy of the preserved run; accepted artifacts remain immutable.
    import tempfile
    with tempfile.TemporaryDirectory(prefix='readiness-coverage-verify-') as temporary:
        clone=Path(temporary)/'run';clone.mkdir()
        for relative in ('source','generated','checks'):
            shutil.copytree(owner/relative,clone/relative,ignore=shutil.ignore_patterns('build','original-build','reference-target'))
        context.path=clone
        predecessor=None
        parent=coverage.get('predecessor')
        if parent:
            root=context.root;pointer_paths=[root/'out/accepted.json']+list((root/'out/accepted-history').glob('*.json'))
            for pointer_path in pointer_paths:
                record=json.loads(pointer_path.read_text())
                if record.get('run_id')==parent['run_id']:predecessor=Path(record['run_path']);break
            if predecessor is None:raise ValueError('Coverage predecessor pointer lineage missing')
            verify_accepted(predecessor,require_pointer=True)
        computed=evaluate_coverage(context,clone/'checks/generated-profile-bound.json',clone/'generated/key-inventory.json',predecessor)
        if computed!=coverage:raise ValueError('Actual changed-entry profile/requirements differ')
    context.path=owner
    discovery=json.loads(safe_artifact(owner,checks['instruction-diagnosis']['discovery_evidence']).read_text())
    if discovery.get('status')!='pass' or discovery.get('complete') is not True or discovery.get('unexecuted')!=[]:raise ValueError('Focused actual key discovery incomplete')
    if any(discovery['identities'][name]!=inputs[name] for name in ('rom_sha256','movie_sha256','input_sha256')):raise ValueError('Discovery input differs')
    focus=checks['instruction-diagnosis'];reports=focus.get('reports',[focus])
    if not reports or len(focus.get('intervals',[]))!=len(reports):raise ValueError('Focus lacks actual covered intervals')
    for index,report in enumerate(reports):
        path=safe_artifact(owner,report['evidence']);actual=json.loads(path.read_text());binary=safe_artifact(owner,'generated/mandatory-focus-'+str(index)+'-focused-replay')
        process=json.loads((owner/('logs/mandatory-focus-'+str(index)+'-focused-replay.json')).read_text())
        if process.get('returncode')!=0 or process.get('complete') is not True or process.get('status')!='pass' or Path(process['command'][0]).name!=binary.name:raise ValueError('Focused actual process did not complete')
        if actual['binary_sha256']!=digest(binary) or actual.get('disabled_keys')!=[] or actual.get('replay')!='warm' or actual.get('completed_instructions')!=report.get('completed_instructions'):
            raise ValueError('Focused actual candidate execution differs')
        if any(actual['identities'].get(name)!=inputs[name] for name in ('rom_sha256','movie_sha256','input_sha256')):raise ValueError('Focused input differs')
        interval=focus['intervals'][index]
        executed=actual.get('executed_generated_keys',[])
        for key in interval.get('keys',[interval['key']]):
            if [key['address'],key['mx']] not in executed:raise ValueError('Focused interval did not execute target generated key')
        validation_path=owner/('diagnostics/mandatory-focus-'+str(index)+'-cache/validation.json');pair_path=validation_path.with_name('pair.json');record_path=validation_path.with_name('cache.json')
        validation=json.loads(validation_path.read_text());pair=json.loads(pair_path.read_text());record=json.loads(record_path.read_text())
        validate_gate_scope('checkpoint-validation',validation)
        from paired_checkpoint_verify import verify_pair
        verify_pair(pair,validation,actual)
        if record['pair_file_sha256']!=digest(pair_path) or record['validation_file_sha256']!=digest(validation_path) or validation['pair_sha256']!=pair['pair_sha256'] or actual['cache_pair_sha256']!=pair['pair_sha256']:
            raise ValueError('Paired cache validation/pair identity differs')
        for name in ('rom_sha256','movie_sha256','input_sha256'):
            if validation['identities'][name]!=inputs[name]:raise ValueError('Checkpoint input identity differs')
        if validation['identities']['source_sha256']!=manifest['identities']['source']['sha256'] or validation['identities']['hardware_sha256']!=manifest['identities']['hardware']['source_sha256']:
            raise ValueError('Checkpoint source/hardware differs')
    feedback=checks['feedback-demonstrations']
    from readiness_feedback import verify_feedback
    verify_feedback(owner,manifest,feedback)
    for name,case in feedback['cases'].items():
        evidence=case.get('evidence',{});processes=case.get('processes',[])
        if not evidence.get('path') or not evidence.get('sha256') or not processes or not case.get('assertions') or not case.get('provenance'):
            raise ValueError('Feedback lacks real executed artifact origin: '+name)
        path=safe_artifact(owner,evidence['path'])
        if digest(path)!=evidence['sha256']:raise ValueError('Feedback artifact differs: '+name)
        actual=json.loads(path.read_text())
        if actual.get('status')!=case.get('observed_outcome') or case.get('observed_outcome')!=case.get('expected_outcome'):
            raise ValueError('Feedback artifact outcome differs: '+name)
        for process in processes:
            execution=json.loads(safe_artifact(owner,'logs/'+process['name']+'.json').read_text())
            allowed=process.get('allowed_returncodes',[process.get('expected_returncode')])
            if execution.get('returncode') not in allowed or execution.get('returncode')!=process.get('observed_returncode') or execution.get('complete') is not True:
                raise ValueError('Feedback expected process outcome differs: '+name)
    return True


if __name__=='__main__':raise SystemExit(main())
