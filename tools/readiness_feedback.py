#!/usr/bin/env python3
"""Execute isolated fault controls and preserve their actual evidence origins."""
import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

from readiness import Run, digest, read_verdict, write_json

CASES = {
    'correct-execution': 'pass', 'wrong-instruction': 'divergence',
    'transient-effect': 'divergence', 'uncovered-key': 'uncovered',
    'empty-compilation': 'uncovered', 'preflight-build-failure': 'error',
    'interrupted-truncated': 'incomplete', 'incompatible-stale': 'incomplete',
    'paired-cache': 'pass',
}
LIFECYCLE = {'preflight-build-failure', 'interrupted-truncated', 'incompatible-stale'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def assertions(case, actual):
    """Recompute demonstration assertions from executed reports at publication."""
    require(actual.get('status') == CASES[case] and actual.get('complete') is True,
            'Unexpected feedback outcome: ' + case)
    if case in ('correct-execution', 'wrong-instruction', 'transient-effect'):
        require(actual.get('frame_end_equal') is True, 'Fixture frame-end parity missing')
        if case == 'correct-execution':
            require(actual.get('completed_instructions') == 8 and actual.get('executed_generated_keys'),
                    'Correct fixture did not execute generated instructions')
            return ['fresh generated execution', 'complete instruction and frame parity']
        first = actual.get('first_mismatch', {})
        require(first.get('instruction') == 3 and first.get('subject', {}).get('generated_key') == [0x8004, 3]
                and actual.get('altered_key') == [0x8004, 3] and actual.get('preceding_matching_boundary')
                and actual.get('first_bus_difference', {}).get('reference', {}).get('value') == 0x12
                and actual.get('first_bus_difference', {}).get('subject', {}).get('value') == 0x99,
                'Wrong store was not attributed at its earliest observable write')
        return ['first wrong write names altered generated key', 'matching preceding boundary',
                'transient write detected despite matching frame-end CPU/clocks/WRAM/video']
    if case in ('uncovered-key', 'empty-compilation'):
        coverage = actual.get('coverage', {})
        require(coverage.get('status') == 'uncovered' and actual.get('matched_frames') == 2,
                'Actual coverage refusal missing')
        if case == 'uncovered-key':
            require(coverage.get('generated_instructions', 0) > 0 and coverage.get('uncovered_keys') == [{'address':0x8100,'mx':3}],
                    'Old execution masked new unexecuted entry')
            return ['real old generated execution', 'added unexecuted key refuses acceptance']
        require(coverage.get('generated_instructions') == 0 and actual['profile']['compiled'] == [],
                'Empty compilation did not refuse fallback execution')
        return ['empty table refuses acceptance despite complete fallback parity']
    if case == 'paired-cache':
        require(actual['warm']['status'] == actual['cold']['status'] == 'pass'
                and actual['warm']['history'] == actual['cold']['history'], 'Warm/cold trace differs')
        for name, action in (('created','created'),('reused','reused'),('corrupt','rebuilt'),('incompatible','rebuilt')):
            cache = actual[name]
            require(cache['action'] == action and cache['validation']['completed_frames'] == 1
                    and cache['validation']['observed_rollovers'] == 2, 'Incomplete cache control')
            if action == 'rebuilt':require(bool(cache.get('rejection')), 'Missing cache rejection reason')
        return ['cold next complete frame validates both sides', 'warm/cold traces and outcomes match',
                'corrupt and incompatible caches rejected and rebuilt']
    observations = actual.get('observations', {})
    expected = {'preflight-build-failure': {'preflight':'error','build':'error'},
                'interrupted-truncated': {'signal':'incomplete','timeout':'incomplete','truncated':'incomplete','unfinished':'incomplete'},
                'incompatible-stale': {'stale':'incomplete','identity':'error','source':'error','input':'error'}}[case]
    require(set(observations) == set(expected), 'Lifecycle controls omitted')
    for name, status in expected.items():
        observation = observations[name]
        require(observation['status'] == status and observation['accept_returncode'] == 2
                and observation['pointer_unchanged'] is True, 'Failure control allowed publication: ' + name)
    return [name + ' refuses publication and preserves prior pointer' for name in expected]


def lifecycle(case, output):
    """Run real lifecycle and acceptance commands in a private repository."""
    from readiness import verify_movie_inputs
    import zipfile
    output.mkdir()
    root = output / 'repository'; root.mkdir()
    subprocess.run(['git','init','-q',str(root)],check=True)
    (root/'config').mkdir();(root/'config/functions.toml').write_text('# isolated fixture\n')
    pointer = root/'out/accepted.json';pointer.parent.mkdir()
    pointer.write_text('{"fixture_predecessor":"must stay unchanged"}\n')
    before = pointer.read_bytes()
    observations = {}
    command = [sys.executable, str(Path(__file__).with_name('readiness_orchestration.py')), 'accept']

    def reject(name, run):
        result = subprocess.run(command + ['--run',str(run.path)],capture_output=True,text=True)
        write_json(output/(name+'-accept.json'), {'command':command+['--run',str(run.path)],
            'returncode':result.returncode,'stdout':result.stdout,'stderr':result.stderr})
        observations[name] = {'status':read_verdict(run.path)['status'],'accept_returncode':result.returncode,
                              'pointer_unchanged':pointer.read_bytes()==before,'run':str(run.path)}

    def new():
        run=Run.create(root,'readiness',{'domain':'Japan/USA any%','frames':18000});run.capture_source();return run

    if case == 'preflight-build-failure':
        command_preflight=[sys.executable,str(Path(__file__).with_name('readiness.py')),'preflight','--root',str(root)]
        result=subprocess.run(command_preflight,capture_output=True,text=True)
        write_json(output/'preflight-command.json',{'command':command_preflight,'returncode':result.returncode,
                                                  'stdout':result.stdout,'stderr':result.stderr})
        path=next((root/'out/runs').iterdir());manifest=json.loads((path/'manifest.json').read_text())
        reject('preflight',Run(root,path,manifest))
        run=new();stage=run.run_stage('build',[sys.executable,'-c',"import sys; print('deliberate build failure', file=sys.stderr); sys.exit(7)"])
        require(stage['returncode']==7 and 'binary' not in run.manifest['identities'],'Build failure fabricated binary')
        run.finalize('error','build','Deliberate build failure');reject('build',run)
    elif case == 'interrupted-truncated':
        for name, script, timeout in [('signal','import os, signal; os.kill(os.getpid(), signal.SIGTERM)',None),
                                      ('timeout','import time; time.sleep(10)',0.05)]:
            run=new();stage=run.run_stage(name,[sys.executable,'-c',script],timeout=timeout)
            require(stage['status']=='incomplete' and stage['complete'] is False,'Interruption reported complete')
            run.finalize('pass',mandatory_checks=[name]);reject(name,run)
        from readiness_sync import verified_capture
        run=new();capture=run.path/'generated/capture.csv'
        capture.write_text('frame,event\n272,1\n')
        write_json(capture.with_suffix('.capture.json'),{'schema_version':1,'role':'candidate',
            'run_id':run.manifest['run_id'],'checkpoint_sha256':digest(capture),'completed_frames':1})
        try:verified_capture(capture,'candidate',{},18000)
        except ValueError as error:
            require('Truncated' in str(error),'Capture did not fail for truncated scope')
            run.record_check('capture',{'status':'incomplete','complete':False,'expected_frames':18000,
                                      'completed_frames':1,'reason':str(error)})
        else:raise ValueError('Truncated capture was accepted')
        run.finalize('pass',mandatory_checks=['capture']);reject('truncated',run)
        reject('unfinished',new())
    else:
        old=new();old.record_check('preflight',{'status':'pass','complete':True});old.finalize('pass',mandatory_checks=['preflight'])
        run=new();(run.path/'verdict.json').write_bytes((old.path/'verdict.json').read_bytes());reject('stale',run)
        (root/'rom').mkdir();(root/'rom/sm.sfc').write_bytes(b'fixture ROM')
        (root/'movies').mkdir();movie=root/'movies/fixture.lsmv'
        with zipfile.ZipFile(movie,'w') as archive:archive.writestr('rom.sha256','0'*64)
        run=new()
        try:verify_movie_inputs(run,{'file':movie.name,'rom_sha256':'0'*64,'sha256':digest(movie),'frames':1})
        except RuntimeError as error:run.finalize('error','identity',str(error))
        else:raise ValueError('Incompatible ROM passed identity verification')
        reject('identity',run)
        for name in ('source','input'):
            run=new()
            path=root/'config/functions.toml' if name=='source' else root/'rom/sm.sfc'
            if name=='input':run.guard_input(path)
            path.write_bytes(path.read_bytes()+b'changed')
            try:run.check_sources()
            except RuntimeError as error:run.finalize('error',name,str(error))
            else:raise ValueError('Contamination passed')
            reject(name,run)
    # Never invoke the real workspace's accepted pointer from fault controls.
    require(pointer.read_bytes()==before,'Private predecessor was replaced')
    report={'schema_version':1,'status':CASES[case],'complete':True,'observations':observations}
    assertions(case,report);write_json(output/'report.json',report)


def coverage_control(case, output, parent):
    """Evaluate an actual synthetic replay profile against exact emitter bodies."""
    from readiness_coverage import generated_inventory, evaluate_coverage
    actual=json.loads((output/'report.json').read_text())
    root=output/'coverage-repository';root.mkdir()
    subprocess.run(['git','init','-q',str(root)],check=True)
    (root/'config').mkdir();(root/'config/functions.toml').write_text('# synthetic emitter fixture\n')
    inputs=actual['identities']
    def candidate(source):
        run=Run.create(root,'feedback-fixture',{'frames':2});run.capture_source()
        run.manifest['identities']['inputs']=inputs
        steps=run.path/'generated/steps.rs';steps.write_text(source+'\npub const ROM_SHA256: Option<&str> = Some("'+inputs['rom_sha256']+'");\n')
        return run, generated_inventory(run,steps)
    source=(output/'steps.rs').read_text()
    predecessor=None
    if case=='uncovered-key':
        # Remove only the unreachable entry; all existing generated bodies remain identical.
        previous='\n'.join(line for line in source.splitlines() if '(0x008100,' not in line)+'\n'
        old,_=candidate(previous);old.finalize('pass');predecessor=old.path
    else:
        source='pub static STEP_TABLE: &[runtime::StepEntry] = &[];\n'
    run,inventory=candidate(source)
    profile=dict(actual['profile'],complete=True,expected_frames=2,completed_frames=2,candidate_inventory_sha256=digest(inventory))
    path=run.path/'checks/profile.json';write_json(path,profile)
    actual['coverage']=evaluate_coverage(run,path,inventory,predecessor)
    actual['status']=actual['coverage']['status'];run.finalize(actual['status'])
    actual['parent_source_sha256']=parent
    write_json(output/'coverage-control.json',actual)


def verify_feedback(owner, manifest, report):
    """Verify real process, binary, fixture inputs, and derived assertions."""
    from readiness_contract import validate_gate_scope
    from readiness_reuse import safe_artifact
    validate_gate_scope('feedback-demonstrations',report)
    source=manifest['identities']['source']['sha256'];hardware=manifest['identities']['hardware']['source_sha256']
    binary=safe_artifact(owner,'generated/feedback-probe')
    require(digest(binary)==manifest['identities']['feedback-binary']['sha256'],'Feedback executable differs')
    build=json.loads(safe_artifact(owner,'logs/feedback-build.json').read_text())
    require(build.get('status')=='pass' and build.get('complete') is True and build.get('returncode')==0
            and build.get('command')==['cargo','build','--release','-p','snes','--bin','feedback-probe','--locked','--offline']
            and build in manifest['commands'],'Feedback build origin differs')
    for case, expected in CASES.items():
        detail=report['cases'][case];path=safe_artifact(owner,detail['evidence']['path'])
        filename='coverage-control.json' if case in ('uncovered-key','empty-compilation') else 'report.json'
        require(path==Path(owner)/'diagnostics/feedback'/case/filename
                and digest(path)==detail['evidence']['sha256'],'Feedback report identity differs')
        actual=json.loads(path.read_text())
        require(detail['assertions']==assertions(case,actual) and detail['expected_outcome']==expected
                and detail['observed_outcome']==actual['status'],'Feedback assertions differ')
        require(detail['provenance']=={'source_sha256':source,'hardware_sha256':hardware,'synthetic':True},
                'Feedback source/hardware identity differs')
        require(len(detail['processes'])==1,'Feedback process origin missing')
        observed=detail['processes'][0];process=json.loads(safe_artifact(owner,'logs/'+observed['name']+'.json').read_text())
        require(process in manifest['commands'] and process['complete'] is True and process['status']=='pass'
                and process['returncode']==observed['observed_returncode'],'Feedback process origin differs')
        if case in LIFECYCLE:
            require(process['command'][1:]==[str(Path(process['cwd'])/'tools/readiness_feedback.py'),'--case',case,'--out',str(path.parent)]
                    and process['returncode']==0,'Lifecycle command scope differs')
            for observation in actual['observations'].values():
                candidate=Path(observation['run'])
                require(path.parent in candidate.parents and read_verdict(candidate)['status']==observation['status'],
                        'Lifecycle preserved verdict differs')
        else:
            require(process['command']==[str(binary),case,str(path.parent),source,hardware]
                    and actual['binary_sha256']==digest(binary),'Feedback executed binary/scope differs')
            ids=actual['identities']
            require(ids['source_sha256']==source and ids['hardware_sha256']==hardware
                    and ids['rom_sha256']==digest(path.parent/'fixture.sfc')
                    and ids['input_sha256']==digest(path.parent/'fixture.inputs'),'Feedback fixture identity differs')
            if case in ('uncovered-key','empty-compilation'):
                coverage=actual['coverage']
                profiles=list((path.parent/'coverage-repository/out/runs').glob('*/checks/profile.json'))
                require(len(profiles)==1 and digest(profiles[0])==coverage['profile_sha256'],
                        'Feedback coverage actual profile differs')
                original=json.loads((path.parent/'report.json').read_text())
                require(actual['profile']==original['profile'] and actual['identities']==original['identities'],
                        'Feedback coverage differs from executed probe profile')
    return True


def run_demonstrations(run, timeout=None):
    started=time.monotonic();output=run.path/'diagnostics/feedback';output.mkdir()
    build=run.path/'build/feedback'
    env=dict(os.environ,CARGO_TARGET_DIR=str(build))
    stage=run.run_stage('feedback-build',['cargo','build','--release','-p','snes','--bin','feedback-probe','--locked','--offline'],env=env,timeout=timeout)
    cases={};source=run.manifest['identities']['source']['sha256'];hardware=run.manifest['identities']['hardware']['source_sha256']
    binary=run.path/'generated/feedback-probe'
    if stage['status']=='pass':
        run.record_artifact('feedback-binary',build/'release/feedback-probe','generated/feedback-probe');run.guard_input(binary)
    for case,expected in CASES.items():
        directory=output/case;processes=[]
        try:
            if case in LIFECYCLE:
                command=[sys.executable,run.root/'tools/readiness_feedback.py','--case',case,'--out',directory]
                allowed=[0]
            else:
                require(stage['status']=='pass','Feedback executable did not build')
                command=[binary,case,directory,source,hardware];allowed=[1 if expected in ('divergence','uncovered') else 0]
                if case=='uncovered-key':allowed=[0]
            name='feedback-'+case
            process=run.run_stage(name,command,timeout=timeout,exit_statuses={1:'pass'} if allowed==[1] else None)
            processes.append({'name':name,'allowed_returncodes':allowed,'observed_returncode':process['returncode']})
            require(process['returncode'] in allowed and process['complete'] is True and process['status']=='pass','Feedback process did not complete expected outcome')
            if case in ('uncovered-key','empty-compilation'):coverage_control(case,directory,source)
            path=directory/('coverage-control.json' if case in ('uncovered-key','empty-compilation') else 'report.json')
            actual=json.loads(path.read_text());checks=assertions(case,actual)
            cases[case]={'status':'pass','complete':True,'expected_outcome':expected,'observed_outcome':actual['status'],
                         'assertions':checks,'evidence':{'path':path.relative_to(run.path).as_posix(),'sha256':digest(path)},
                         'processes':processes,'provenance':{'source_sha256':source,'hardware_sha256':hardware,'synthetic':True}}
        except (Exception,KeyboardInterrupt) as error:
            cases[case]={'status':'incomplete' if isinstance(error,KeyboardInterrupt) or any(run.checks[p['name']]['status']=='incomplete' for p in processes) else 'error',
                         'complete':False,'reason':str(error),'expected_outcome':expected,'processes':processes}
            if isinstance(error,KeyboardInterrupt):break
    status='pass' if len(cases)==len(CASES) and all(case['status']=='pass' for case in cases.values()) else 'incomplete' if any(case['status']=='incomplete' for case in cases.values()) else 'error'
    report={'schema_version':1,'status':status,'complete':status=='pass','expected_cases':len(CASES),
            'completed_cases':sum(case['status']=='pass' for case in cases.values()),'cases':cases,'total_seconds':time.monotonic()-started,
            'scope':'synthetic fault controls; full domain acceptance requires every mandatory gate'}
    if status=='pass':verify_feedback(run.path,run.manifest,report)
    write_json(output/'report.json',report);run.check_sources();run.record_check('feedback-demonstrations',report)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--case',choices=sorted(LIFECYCLE));parser.add_argument('--out',type=Path)
    parser.add_argument('--timeout',type=float);args=parser.parse_args()
    if args.case:
        lifecycle(args.case,args.out);return 0
    from readiness import preflight
    run=Run.create(Path(__file__).resolve().parent.parent,'feedback-demonstrations',{'synthetic_cases':len(CASES)})
    print('Run: '+str(run.path),flush=True);print('Brief: '+str(run.path/'brief.md'),flush=True)
    try:
        failures=preflight(run)
        if failures:return run.finalize('error','preflight','; '.join(failures))
        report=run_demonstrations(run,args.timeout)
        return run.finalize(report['status'],'feedback-demonstrations','Synthetic fault controls; full readiness remains separate',['preflight','feedback-demonstrations'])
    except KeyboardInterrupt:return run.finalize('incomplete','feedback-demonstrations','Interrupted')
    except Exception as error:return run.finalize('error','feedback-demonstrations',str(error))


if __name__=='__main__':raise SystemExit(main())
