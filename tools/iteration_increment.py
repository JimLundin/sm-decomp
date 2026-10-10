#!/usr/bin/env python3
"""Prepare exactly one measured supported routine and validate it through cold acceptance."""
import argparse
import json
import os
import re
import sys
from pathlib import Path

from readiness import Run, digest, source_inventory, read_verdict
from readiness_contract import DOMAIN
from readiness_coverage import comparable_progress


def require_increment(increment):
    kind=(increment or {}).get('kind')
    required={'supported-expansion':('baseline_path','preparation_path','chosen'),
        'timed-pattern-expansion':('baseline_path','pattern_preparation_path','pattern_proof_path','opcode'),
        'correctness-fix':('baseline_path','rationale','reproduction')}
    if kind not in required or any(not increment.get(key) for key in required[kind]):
        raise ValueError('Explicit verified bounded increment metadata is required; use recomp-iteration or a sealed preparation')
    return kind


def progress_report(coverage, increment=None):
    """Expansion is progress only when comparable measured generated work grows."""
    kind = (increment or {}).get('kind')
    if kind not in ('supported-expansion','timed-pattern-expansion','correctness-fix'):
        return {'schema_version':1,'status':'uncovered','complete':True,'kind':kind,'reason':'Explicit bounded increment metadata is required'}
    progress = coverage.get('progress',{})
    valid = coverage.get('status') == 'pass' and coverage.get('complete') is True
    reason = 'Expansion requires positive observed generated-instruction gain in the identical validated domain'
    valid = valid and progress.get('comparable') is True and isinstance(progress.get('generated_delta'),int)
    if kind == 'correctness-fix':
        reproduction = (increment or {}).get('reproduction',{})
        valid = valid and reproduction.get('status') == 'pass' and reproduction.get('complete') is True and bool(reproduction.get('evidence')) and bool(reproduction.get('sha256')) and (increment or {}).get('reproduction_verified') is True and bool((increment or {}).get('rationale'))
        reason = 'Correctness repair requires an explicit rationale and verified independent reproduction evidence'
    else:
        valid = valid and progress.get('generated_delta',0) > 0
    return {'schema_version':1,'status':'pass' if valid else 'uncovered','complete':True,
        'kind':kind,'generated_instructions':coverage.get('generated_instructions'),
        'interpreted_instructions':coverage.get('interpreted_instructions'),
        'fallback_fraction':coverage.get('fallback_fraction'),'validated_distinct_keys':coverage.get('validated_distinct_keys'),
        'progress':progress,'reason':'Comparable observed instruction progress verified' if valid else reason}


def progress_gate(run):
    increment=run.manifest.get('increment',{})
    if increment.get('kind')=='correctness-fix':
        verify_reproduction(run.path,increment)
        increment['reproduction_verified']=True;run.save_manifest()
    report = progress_report(run.checks.get('changed-entry-coverage',{}),run.manifest.get('increment'))
    run.record_check('increment-progress',report)
    return report


def verify_focus_execution(owner,evidence,sha256=None,sealed=True):
    """Verify a preserved actual focus process, its binary, inputs and interval."""
    from readiness_orchestration import source_snapshot
    from readiness_reuse import safe_artifact,verify_owned_evidence
    owner=Path(owner).resolve();manifest,_=source_snapshot(owner)
    if sealed:verify_owned_evidence(owner,manifest)
    path=safe_artifact(owner,evidence)
    if sha256 and digest(path)!=sha256:raise ValueError('Focused execution report checksum differs')
    actual=json.loads(path.read_text())
    if path.name!='instruction.json' or not path.parent.name.endswith('-instructions'):
        raise ValueError('Focused execution must use actual production diagnostic evidence')
    prefix=path.parent.name[:-len('instructions')]
    binary=safe_artifact(owner,'generated/'+prefix+'focused-replay')
    process=json.loads(safe_artifact(owner,'logs/'+prefix+'focused-replay.json').read_text())
    identities=manifest['identities']
    prepared=identities.get('focused-prepared-binary',{})
    if prepared.get('sha256')!=digest(binary):raise ValueError('Focused execution lacks preserved candidate build identity')
    candidate_steps=identities.get('generated-generated-steps',{})
    steps=safe_artifact(owner,candidate_steps['evidence'])
    if digest(steps)!=candidate_steps.get('sha256'):raise ValueError('Focused generated inputs differ from preserved source build')
    build_inputs=manifest.get('parity_build_inputs',{}).get('generated',{})
    build=next((row for row in manifest.get('commands',[]) if row.get('command')==['cargo','build','--release','-p','diff','--bin','focused-replay','--locked','--offline']
        and row.get('complete') is True and row.get('returncode')==0),None)
    if not build or build_inputs.get('steps')!=candidate_steps['evidence'] or build_inputs.get('environment',{}).get('SM_RECOMP_GENERATED_STEPS')!=str(owner/build_inputs.get('steps','')):
        raise ValueError('Focused execution lacks exact generated-source build provenance')
    command=process.get('command',[])
    if (process not in manifest.get('commands',[]) or process.get('complete') is not True
            or process.get('returncode')!=(0 if actual.get('status')=='pass' else 1)
            or not command or Path(command[0]).resolve()!=binary.resolve()
            or actual.get('binary_sha256')!=digest(binary) or actual.get('complete') is not True):
        raise ValueError('Focused execution lacks actual completed preserved binary/process provenance')
    for field in ('stdout','stderr'):safe_artifact(owner,process[field])
    for flag,field in (('--out',None),('--start-input','start_input_index'),('--start-instruction','start_instruction'),('--instructions','expected_instructions')):
        if flag not in command:raise ValueError('Focused process interval command missing')
        value=command[command.index(flag)+1]
        if flag=='--out' and Path(value).resolve()!=path.parent.resolve():raise ValueError('Focused process output differs')
        if field and actual.get(field) is not None and int(value)!=actual[field]:raise ValueError('Focused process interval differs')
    if (actual.get('scope')=='until-frame-end')!=('--until-frame-end' in command):raise ValueError('Focused frame scope differs')
    inputs=manifest['identities']['inputs']
    if any(actual.get('identities',{}).get(key)!=inputs[key] for key in ('rom_sha256','movie_sha256','input_sha256')):
        raise ValueError('Focused source input identities differ')
    disabled=actual.get('disabled_keys',[])
    if disabled:
        if '--disabled-keys' not in command:raise ValueError('Disabled control lacks actual disabled dispatch')
        disabled_path=Path(command[command.index('--disabled-keys')+1])
        if json.loads(safe_artifact(owner,str(disabled_path.relative_to(owner))).read_text())!=disabled:raise ValueError('Disabled control keys differ')
    elif '--disabled-keys' in command:raise ValueError('Focused disabled dispatch differs')
    mismatch=actual.get('first_mismatch')
    if actual.get('status')=='divergence':
        if not mismatch:raise ValueError('Focused failure lacks actual first mismatch')
        from iteration_diagnosis import verify_observation
        verify_observation({'run_path':str(owner),'evidence':evidence,'sha256':digest(path),
            'process':str((owner/'logs'/(prefix+'focused-replay.json')).relative_to(owner)),
            'process_sha256':digest(owner/'logs'/(prefix+'focused-replay.json')),
            'executable':str(binary.relative_to(owner)),'executable_sha256':digest(binary),'kind':'first-mismatch'})
        for side in ('reference','subject'):
            trace=mismatch[side]
            for phase in ('before','after'):
                expected=trace.get('hardware_'+phase+'_sha256')
                if expected and digest(safe_artifact(owner,str((path.parent/(side+'-'+phase+'.hardware.bin')).relative_to(owner))))!=expected:
                    raise ValueError('Focused mismatch hardware snapshot differs')
    return manifest,actual


def verify_reproduction(owner,increment):
    """Bind the actual historical first mismatch, matched controls and repaired focus."""
    from readiness_reuse import safe_artifact,verify_accepted
    from readiness_orchestration import source_snapshot
    reproduction=increment.get('reproduction',{})
    artifact=safe_artifact(owner,reproduction['evidence'])
    if digest(artifact)!=reproduction['sha256']:raise ValueError('Correctness reproduction checksum differs')
    actual=json.loads(artifact.read_text())
    if actual.get('status')!='pass' or actual.get('complete') is not True or actual.get('kind')!='focused-instruction-regression':
        raise ValueError('Correctness repair requires a verified focused regression')
    failure_path=Path(actual['failed_candidate'])
    if read_verdict(failure_path).get('status')!='divergence':raise ValueError('Correctness reproduction lacks preserved failing candidate')
    failed_manifest,failed=verify_focus_execution(failure_path,actual['failure_evidence'],actual['failure_sha256'])
    if failed.get('status')!='divergence' or failed.get('disabled_keys'):raise ValueError('Correctness reproduction is not a candidate failure')
    mismatch=failed['first_mismatch'];key=mismatch['subject'].get('generated_key')
    if not key:raise ValueError('Correctness reproduction lacks affected generated key')
    baseline=verify_accepted(increment['baseline_path'],require_pointer=True)
    if failed_manifest.get('accepted_parent')!=baseline['manifest']['run_id']:raise ValueError('Correctness failure predecessor differs')
    fixed_manifest,fixed=verify_focus_execution(owner,actual['fixed_evidence'],actual['fixed_sha256'],sealed=False)
    reports=[fixed]
    for name in ('predecessor','disabled_entry'):
        control=actual['controls'][name]
        control_owner=Path(control.get('run_path',failure_path))
        manifest,report=verify_focus_execution(control_owner,control['evidence'],control['sha256'])
        if name=='predecessor':
            if manifest['identities']['source']['sha256']!=baseline['manifest']['identities']['source']['sha256'] or report.get('disabled_keys'):
                raise ValueError('Correctness predecessor control is not rebuilt accepted source')
        elif (manifest['identities']['source']['sha256']!=failed_manifest['identities']['source']['sha256']
                or {'address':key[0],'mx':key[1]} not in report.get('disabled_keys',[])):
            raise ValueError('Correctness disabled control does not disable affected candidate key')
        reports.append(report)
    for report in reports:
        if (report.get('status')!='pass' or report.get('complete') is not True
                or report.get('start_input_index')!=failed.get('start_input_index')
                or report.get('start_instruction')!=failed.get('start_instruction')
                or report.get('identities')!=failed.get('identities')
                or report.get('completed_instructions',0)<failed.get('completed_instructions',0)):
            raise ValueError('Correctness repaired/control interval or input differs from localized failure')
        expected=failed.get('expected_instructions')
        if expected is not None and report.get('expected_instructions')!=expected:raise ValueError('Correctness control interval differs')
    if fixed.get('disabled_keys') or key not in fixed.get('executed_generated_keys',[]):raise ValueError('Correctness repaired focus did not execute affected key')
    focus=json.loads(safe_artifact(owner,'checks/instruction-diagnosis.json').read_text())
    if not any(row.get('evidence')==actual['fixed_evidence'] and row.get('status')=='pass' for row in focus.get('reports',[])):
        raise ValueError('Correctness reproduction is outside verified mandatory focus')
    if fixed_manifest.get('accepted_parent')!=failed_manifest.get('accepted_parent'):raise ValueError('Correctness repaired predecessor differs')
    return True


def verify_progress(owner,verified):
    """Recompute the extra gate from sealed real profiles before pointer publication."""
    manifest = verified['manifest']
    if manifest.get('type') != 'iteration':
        return
    require_increment(manifest.get('increment'))
    if manifest.get('increment',{}).get('kind')=='supported-expansion':
        verify_supported_increment(owner,verified)
    checks = verified['verdict']['checks']
    coverage = checks['changed-entry-coverage']
    report = checks.get('increment-progress')
    if report is None or report != progress_report(coverage,manifest.get('increment')) or report['status'] != 'pass':
        raise ValueError('Iteration lacks passing verified positive instruction progress')
    if manifest.get('increment',{}).get('kind') == 'correctness-fix':
        verify_reproduction(owner,manifest['increment'])
    parent = coverage.get('predecessor')
    if not parent:
        raise ValueError('Iteration progress lacks a comparable accepted predecessor')
    from readiness_reuse import verify_accepted
    origin = manifest.get('increment',{}).get('baseline_path')
    if origin is None:
        root = Path(owner).parent.parent.parent
        for pointer in [root/'out/accepted.json'] + list((root/'out/accepted-history').glob('*.json')):
            if pointer.exists():
                record=json.loads(pointer.read_text())
                if record.get('run_id') == parent['run_id']:
                    origin=record['run_path'];break
    if origin is None:
        raise ValueError('Iteration progress predecessor lineage is missing')
    accepted=verify_accepted(origin,require_pointer=True)
    if accepted['manifest']['run_id'] != parent['run_id'] or manifest.get('accepted_parent') != parent['run_id']:
        raise ValueError('Iteration progress predecessor identity differs')
    baseline=json.loads((Path(origin)/'coverage.json').read_text())
    if baseline != accepted['verdict']['checks']['changed-entry-coverage']:
        raise ValueError('Iteration progress baseline coverage differs from accepted evidence')
    if comparable_progress(coverage,baseline) != coverage.get('progress'):
        raise ValueError('Iteration progress delta differs from actual comparable counts')



def verify_supported_increment(owner,verified):
    from readiness_orchestration import source_snapshot
    from readiness_reuse import verify_owned_evidence,verify_accepted,safe_artifact
    increment=verified['manifest']['increment']
    preparation=Path(increment['preparation_path'])
    manifest,inventory=source_snapshot(preparation);verify_owned_evidence(preparation,manifest)
    verdict=read_verdict(preparation)
    report=verdict['checks']['candidate-proposal']
    if verdict.get('status')!='pass' or manifest.get('accepted_parent')!=verified['manifest']['accepted_parent']:
        raise ValueError('Supported increment preparation identity differs')
    config=Path(owner)/'source/files/config/functions.toml'
    if digest(config)!=report['proposed_config_sha256']:
        raise ValueError('Accepted supported config differs from sealed bounded proposal')
    _,candidate_inventory=source_snapshot(owner)
    expected={row['path']:(row['sha256'],row['mode']) for row in inventory['files'] if row['path']!='config/functions.toml'}
    current_source={row['path']:(row['sha256'],row['mode']) for row in candidate_inventory['files'] if row['path']!='config/functions.toml'}
    if current_source!=expected:raise ValueError('Supported candidate maintained source differs from bounded preparation')
    ancestor=preparation;seen=set();deferred=set()
    from readiness_coverage import _table_rows
    while True:
        if str(ancestor) in seen:raise ValueError('Supported preparation lineage is cyclic')
        seen.add(str(ancestor));ancestor_manifest,_=source_snapshot(ancestor);verify_owned_evidence(ancestor,ancestor_manifest)
        ancestor_report=read_verdict(ancestor)['checks']['candidate-proposal']
        if read_verdict(ancestor).get('status')!='pass' or ancestor_manifest.get('increment',{}).get('chosen')!=increment.get('chosen'):
            raise ValueError('Supported preparation lineage changes selected choice')
        parent=ancestor_report.get('parent_preparation_path') or ancestor_report.get('pruning_parent_path')
        if not parent:break
        parent=Path(parent);parent_manifest,_=source_snapshot(parent);verify_owned_evidence(parent,parent_manifest)
        parent_report=read_verdict(parent)['checks']['candidate-proposal']
        parent_steps=parent/'generated/proposed-steps.rs'
        if not parent_steps.exists():parent_steps=parent/'generated/steps.rs'
        ancestor_steps=ancestor/'generated/proposed-steps.rs'
        if not ancestor_steps.exists():ancestor_steps=ancestor/'generated/steps.rs'
        parent_keys={(row['address'],row['mx']) for row in _table_rows(parent_steps.read_text())}
        ancestor_keys={(row['address'],row['mx']) for row in _table_rows(ancestor_steps.read_text())}
        if ancestor_report.get('parent_preparation_path'):
            if (digest(parent/'checks/candidate-proposal.json')!=ancestor_report['parent_proposal_sha256']
                    or parent_report['proposed_config_sha256']!=ancestor_report['proposed_config_sha256'] or ancestor_keys!=parent_keys):
                raise ValueError('Supported semantic repair changes selected config or keys')
        else:
            failed=Path(ancestor_report['pruning_failed_path']);failed_manifest,_=source_snapshot(failed);verify_owned_evidence(failed,failed_manifest)
            failed_report=read_verdict(failed)['checks']['changed-entry-coverage']
            required={(row['address'],row['mx']):row['reason'] for row in failed_report['required_keys']}
            remove={(row['address'],row['mx']) for row in ancestor_report['deferred']}
            uncovered={(row['address'],row['mx']) for row in failed_report['uncovered_keys']}
            if (digest(parent/'checks/candidate-proposal.json')!=ancestor_report['pruning_parent_sha256'] or remove!=uncovered
                    or any(required.get(key)!='added' for key in remove) or ancestor_keys!=parent_keys-remove
                    or failed_manifest.get('increment',{}).get('preparation_path')!=str(parent)):
                raise ValueError('Supported deferred scope lacks immutable cold-unexecuted new-key evidence')
            deferred.update(remove)
        ancestor=parent
    selection=Path(increment['selection_path'])
    selection_manifest,_=source_snapshot(selection);verify_owned_evidence(selection,selection_manifest)
    from iteration_selection import verify_selection_result
    selected=verify_selection_result(json.loads((selection/'selection.json').read_text()))
    gate=read_verdict(selection)['checks']['measured-selection']
    if (read_verdict(selection).get('status')!='pass' or selection_manifest.get('type')!='iteration-selection'
            or digest(selection/'selection.json')!=gate['sha256'] or digest(selection/'selection.json')!=increment['selection_sha256']
            or selected.get('chosen')!=increment.get('chosen') or selected.get('baseline',{}).get('run_path')!=increment['baseline_path']):
        raise ValueError('Supported accepted choice is not the verified measured selection')
    if increment.get('chosen')!=manifest.get('increment',{}).get('chosen'):
        raise ValueError('Supported chosen selection differs from sealed preparation')
    baseline=verify_accepted(increment['baseline_path'],require_pointer=True)
    identity=baseline['manifest']['identities']['generated-key-inventory']
    previous=json.loads(safe_artifact(increment['baseline_path'],identity['evidence']).read_text())
    prior={(key['address'],key['mx']) for key in previous['keys']}
    proposed=preparation/'generated/proposed-steps.rs'
    if not proposed.exists():proposed=preparation/'generated/steps.rs'
    from readiness_coverage import _table_rows
    expected={(key['address'],key['mx']) for key in _table_rows(proposed.read_text())}
    actual=json.loads((Path(owner)/'generated/key-inventory.json').read_text())
    current={(key['address'],key['mx']) for key in actual['keys']}
    if current!=expected or not prior<=current:
        raise ValueError('Supported accepted table differs from baseline plus one selected bounded slice')
    chosen=increment['chosen']
    desired={(key['address'],key['mx']) for key in chosen['new_executed_keys']}
    if current-prior!=desired-deferred or not current-prior:
        raise ValueError('Supported accepted expansion exceeds selected observed keys')
    return True


def propose_candidate(run,baseline,generator,timeout=None):
    """Generate in isolation, then exclude every new key outside the measured choice."""
    from readiness_coverage import _table_rows
    selection=json.loads((run.path/'generated/selection.json').read_text())
    chosen=selection['chosen']
    current=run.root/'config/functions.toml'
    proposal=run.path/'generated/functions.toml'
    source=current.read_text()
    addition='\n\n# One measured routine/width increment; retained preparation identifies evidence.\n[[function]]\naddr = "{:06X}"\nvariants = ["{}"]\n'.format(chosen['address'],chosen['variant'])
    # Existing address with another width requires extending that one declaration.
    found=False
    parts=re.split(r'(?=^\[\[[^\n]+\]\]\s*$)',source,flags=re.M)
    for index,part in enumerate(parts):
        if not part.startswith('[[function]]'):continue
        match=re.search(r'^addr\s*=\s*"([^"]+)"',part,re.M)
        if match and int(re.sub(r'[^0-9a-fA-F]','',match.group(1)),16)==chosen['address']:
            variants=re.search(r'^variants\s*=\s*\[([^]]*)\]',part,re.M)
            if not variants:raise ValueError('Existing routine declaration lacks explicit variants')
            values=re.findall(r'"(m[01]x[01])"',variants.group(1))
            if chosen['variant'] in values:
                if chosen.get('action')!='enable-deferred-keys':raise ValueError('Selected routine/width is already reviewed')
            else:
                if chosen.get('action')=='enable-deferred-keys':raise ValueError('Deferred revisit requires an existing reviewed routine/width')
                values.append(chosen['variant'])
                parts[index]=part[:variants.start()]+ 'variants = ['+', '.join(json.dumps(value) for value in values)+']'+part[variants.end():]
            found=True;break
    if chosen.get('action')=='enable-deferred-keys' and not found:
        raise ValueError('Deferred revisit routine is no longer reviewed')
    proposed=''.join(parts) if found else source+addition
    # Lift only measured keys; keep unrelated exclusions and user comments.
    desired={(row['address'],row['mx']):row for row in chosen['new_executed_keys']}
    sections=re.split(r'(?=^\[\[[^\n]+\]\]\s*$)',proposed,flags=re.M)
    for index,section in enumerate(sections):
        if not section.startswith('[[excluded_step]]'):continue
        address=re.search(r'^addr\s*=\s*"([^"]+)"',section,re.M)
        variant=re.search(r'^variant\s*=\s*"m([01])x([01])"',section,re.M)
        if address and variant:
            key=(int(re.sub(r'[^0-9a-fA-F]','',address.group(1)),16),int(variant.group(1))*2+int(variant.group(2)))
            if key in desired:
                sections[index]=''.join(line for line in section.splitlines(keepends=True)
                    if not re.match(r'^(?:\[\[excluded_step\]\]|\s*(?:addr|variant|reason)\s*=)',line))
    proposal.write_text(''.join(sections))
    run.guard_input(generator)
    first=run.path/'generated/unpruned-steps.rs'
    result=run.run_stage('proposed-generation',[generator,'--rom',run.root/'rom/sm.sfc','--functions',proposal,
        '--out',run.path/'generated/proposed-game.rs','--steps-out',first],timeout=timeout)
    if result['status']!='pass':raise ValueError('Candidate generation failed; preserved stage logs explain the error')
    all_rows=_table_rows(first.read_text())
    previous={(row['address'],row['mx']) for row in baseline['keys']}
    actual={(row['address'],row['mx']):row for row in all_rows}
    if not previous <= set(actual):raise ValueError('Candidate proposal removes a legacy baseline key')
    if not set(desired) <= set(actual):raise ValueError('Measured selected key is absent from candidate CFG')
    for key,row in desired.items():
        if key in previous or actual[key]['bytes']!=row['bytes'] or actual[key]['opcode']!=row['opcode']:
            raise ValueError('Selected key bytes/opcode differ from measured proposal')
    deferred=[row for key,row in sorted(actual.items()) if key not in previous and key not in desired]
    if deferred:
        with proposal.open('a') as target:
            for row in deferred:
                target.write('\n[[excluded_step]]\naddr = "{:06X}"\nvariant = "m{}x{}"\nreason = "deferred unexecuted proposed key"\n'.format(row['address'],row['mx'] >> 1,row['mx'] & 1))
    final=run.path/'generated/proposed-steps.rs'
    result=run.run_stage('pruned-generation',[generator,'--rom',run.root/'rom/sm.sfc','--functions',proposal,
        '--out',run.path/'generated/pruned-game.rs','--steps-out',final],timeout=timeout)
    if result['status']!='pass':raise ValueError('Pruned candidate regeneration failed')
    final_rows=_table_rows(final.read_text())
    if {(row['address'],row['mx']) for row in final_rows} != previous | set(desired):
        raise ValueError('Actual proposed compiled table differs from baseline plus measured selected keys')
    report={'schema_version':1,'status':'pass','complete':True,'selection_sha256':digest(run.path/'generated/selection.json'),
        'prior_config_sha256':digest(current),'proposed_config_sha256':digest(proposal),
        'steps_sha256':digest(final),'actual_new_keys':[{'address':key[0],'mx':key[1]} for key in sorted(desired)],
        'deferred':deferred,'preserved_baseline_keys':len(previous)}
    run.record_check('candidate-proposal',report)
    run.record_artifact('proposed-config',proposal)
    run.record_artifact('proposed-steps',final)
    return proposal


def prepare(root,selection_owner,timeout=None):
    """Allocate before setup/mutation and retain selected exact source inputs."""
    root=Path(root).resolve();selection_owner=Path(selection_owner).resolve()
    run=Run.create(root,'increment-preparation',dict(DOMAIN))
    print('Preparation: '+str(run.path),flush=True)
    try:
        run.capture_source()
        from readiness import read_verdict
        from iteration_selection import verify_selection_result
        from readiness_reuse import safe_artifact,verify_accepted
        from readiness_orchestration import source_snapshot,verify_origins
        manifest,inventory=source_snapshot(selection_owner)
        from readiness_reuse import verify_owned_evidence
        verify_owned_evidence(selection_owner,manifest)
        verdict=read_verdict(selection_owner)
        if manifest.get('type')!='iteration-selection' or verdict.get('status')!='pass' or verdict.get('acceptance_eligible'):
            raise ValueError('Selection is not a complete measured selection run')
        report=verify_selection_result(json.loads((selection_owner/'selection.json').read_text()))
        if report['outcome']!='supported-routine':raise ValueError('Selection requires a different route: '+report['outcome'])
        raw=json.loads((selection_owner/'checks/selection.json').read_text())
        if any(report.get(name)!=raw.get(name) for name in ('chosen','ranked','rejected','outcome','ranking_policy','scope_policy')):
            raise ValueError('Measured selection differs from preserved production ranking output')
        profile=selection_owner/'generated/selection-profile.json'
        if digest(profile)!=report['profile_sha256']:
            raise ValueError('Measured interpreted instruction profile checksum differs')
        gate=verdict['checks']['measured-selection']
        if digest(selection_owner/'selection.json')!=gate['sha256'] or report['source_sha256']!=run.manifest['identities']['source']['sha256']:
            raise ValueError('Measured selection source/result differs from current candidate preparation')
        baseline_path=Path(report['baseline']['run_path'])
        accepted=verify_accepted(baseline_path,require_pointer=True);verify_origins(baseline_path,accepted)
        if accepted['manifest']['run_id']!=report['baseline']['run_id']:
            raise ValueError('Measured selection accepted baseline differs')
        run.manifest['accepted_parent']=accepted['manifest']['run_id'];run.save_manifest()
        failures=preflight_existing_snapshot(run)
        if failures:raise ValueError('Candidate preparation preflight failed: '+'; '.join(failures))
        run.record_artifact('measured-selection',selection_owner/'selection.json','generated/selection.json')
        identity=accepted['manifest']['identities']['generated-key-inventory']
        baseline=json.loads(safe_artifact(baseline_path,identity['evidence']).read_text())
        run.record_artifact('baseline-inventory',safe_artifact(baseline_path,identity['evidence']),'generated/baseline-inventory.json')
        run.guard_input(root/'rom/sm.sfc')
        environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/'build/generator'),CARGO_INCREMENTAL='0')
        build=run.run_stage('proposal-generator-build',['cargo','build','--release','-p','recomp','--locked','--offline'],env=environment,timeout=timeout)
        if build['status']!='pass':raise ValueError('Proposal generator build failed')
        generator=run.path/'build/generator/release/recomp'
        run.record_artifact('proposal-generator',generator,'generated/recomp')
        proposal=propose_candidate(run,baseline,run.path/'generated/recomp',timeout)
        run.check_sources()
        run.manifest['increment']={'kind':'supported-expansion','baseline_path':str(baseline_path),
            'selection_path':str(selection_owner),'selection_sha256':digest(selection_owner/'selection.json'),
            'preparation_path':str(run.path),'chosen':report['chosen'],'deferred':run.checks['candidate-proposal']['deferred']}
        run.save_manifest()
        summary='Prepared one selected routine/width; reviewed root configuration remains unchanged.\n\n[Proposal](generated/functions.toml) · [Exact compiled keys and deferred inventory](checks/candidate-proposal.json) · [Selected evidence](generated/selection.json) · [Source before mutation](source/inventory.json)\n'
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest()
        run.finalize('pass','candidate-proposal','One bounded candidate prepared',mandatory_checks=['candidate-proposal'],brief_text=summary)
        return run.path
    except Exception as error:
        run.finalize('error','candidate-proposal',str(error));raise


def preflight_existing_snapshot(run):
    # Preflight owns its own snapshot. Preparation keeps its earlier source
    # unchanged and links a distinct completed setup run.
    from readiness import preflight
    setup=Run.create(run.root,'increment-preflight',dict(DOMAIN),run.manifest.get('accepted_parent'))
    failures=preflight(setup)
    setup.finalize('error' if failures else 'pass','preflight','; '.join(failures) if failures else 'Setup verified',mandatory_checks=['preflight'])
    run.check_sources()
    run.manifest['preflight_path']=str(setup.path);run.save_manifest()
    return failures


def apply_preparation(root,owner):
    """Apply only the reviewed config; keep unrelated source and prior evidence."""
    root=Path(root).resolve();owner=Path(owner).resolve()
    from readiness_orchestration import source_snapshot
    from readiness import read_verdict
    manifest,inventory=source_snapshot(owner)
    from readiness_reuse import verify_owned_evidence
    verify_owned_evidence(owner,manifest)
    verdict=read_verdict(owner)
    if manifest.get('type')!='increment-preparation' or verdict['status']!='pass':
        raise ValueError('Candidate preparation did not complete')
    pointer=json.loads((root/'out/accepted.json').read_text())
    if pointer.get('run_id')!=manifest.get('accepted_parent'):
        raise ValueError('Current accepted predecessor changed before candidate application')
    if source_inventory(root)!=inventory['files']:
        raise ValueError('User source/config changed since preparation; preserve work and select again')
    proposal=owner/'generated/functions.toml'
    check=verdict['checks']['candidate-proposal']
    if digest(proposal)!=check['proposed_config_sha256'] or digest(root/'config/functions.toml')!=check['prior_config_sha256']:
        raise ValueError('Prepared configuration checksum differs')
    temporary=root/'config/.functions.increment-'+__import__('uuid').uuid4().hex+'.tmp'
    with temporary.open('xb') as target:
        target.write(proposal.read_bytes());target.flush();os.fsync(target.fileno())
    temporary.chmod((root/'config/functions.toml').stat().st_mode & 0o777)
    os.replace(temporary,root/'config/functions.toml')
    return manifest['increment']


def prepare_repair(root,failed_owner,timeout=None):
    """Seal repaired source while preserving the selected exact config/key scope."""
    from readiness_orchestration import source_snapshot
    from readiness_reuse import verify_owned_evidence,safe_artifact,seal_owned_evidence
    from readiness_coverage import _table_rows
    root=Path(root).resolve();failed_owner=Path(failed_owner).resolve()
    failed_manifest,_=source_snapshot(failed_owner);verify_owned_evidence(failed_owner,failed_manifest)
    increment=dict(failed_manifest['increment'])
    parent=Path(increment['preparation_path']);parent_manifest,_=source_snapshot(parent);verify_owned_evidence(parent,parent_manifest)
    report=read_verdict(parent)['checks']['candidate-proposal']
    if digest(root/'config/functions.toml')!=report['proposed_config_sha256']:
        raise ValueError('Semantic retry config differs from sealed selected proposal; prepare a bounded proposal first')
    run=Run.create(root,'increment-preparation',dict(DOMAIN),failed_manifest['accepted_parent'])
    try:
        run.capture_source()
        proposal=run.path/'generated/functions.toml';proposal.write_bytes((root/'config/functions.toml').read_bytes())
        environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/'build/generator'),CARGO_INCREMENTAL='0')
        stage=run.run_stage('repair-generator-build',['cargo','build','--release','-p','recomp','--locked','--offline'],env=environment,timeout=timeout)
        if stage['status']!='pass':raise ValueError('Repaired generator build failed')
        run.record_artifact('repair-generator',run.path/'build/generator/release/recomp','generated/recomp')
        steps=run.path/'generated/proposed-steps.rs'
        stage=run.run_stage('repaired-generation',[run.path/'generated/recomp','--rom',root/'rom/sm.sfc','--functions',proposal,
            '--out',run.path/'generated/game.rs','--steps-out',steps],timeout=timeout)
        if stage['status']!='pass':raise ValueError('Repaired generation failed')
        original=parent/'generated/proposed-steps.rs'
        if not original.exists():original=parent/'generated/steps.rs'
        old={(row['address'],row['mx']):row['bytes'] for row in _table_rows(original.read_text())}
        actual={(row['address'],row['mx']):row['bytes'] for row in _table_rows(steps.read_text())}
        if actual!=old:raise ValueError('Repaired table changes selected keys/instruction bytes beyond semantic repair')
        before={row['path']:row['sha256'] for row in json.loads((failed_owner/'source/inventory.json').read_text())['files']}
        after={row['path']:row['sha256'] for row in run.source_files}
        changes=[{'path':path,'before':before.get(path),'after':after.get(path)} for path in sorted(set(before)|set(after)) if before.get(path)!=after.get(path)]
        run.record_check('candidate-proposal',dict(report,prior_config_sha256=digest(root/'config/functions.toml'),
            proposed_config_sha256=digest(proposal),steps_sha256=digest(steps),repair_source_diff=changes,
            parent_preparation_path=str(parent),parent_proposal_sha256=digest(parent/'checks/candidate-proposal.json'),failed_candidate=str(failed_owner)))
        increment.update(preparation_path=str(run.path),repair_parent=str(parent))
        run.manifest['increment']=increment;run.check_sources();seal_owned_evidence(run);run.save_manifest()
        run.finalize('pass','candidate-proposal','Same selected table prepared with exact repaired source; full affected gates still mandatory',mandatory_checks=['candidate-proposal'])
        return run.path
    except Exception as error:
        if run.manifest['state']=='running':run.finalize('error','candidate-proposal',str(error))
        raise


def prune_uncovered(root,failed_owner,timeout=None):
    """Make a new source-bound retry removing only newly added cold-unexecuted keys."""
    from readiness_orchestration import source_snapshot
    from readiness import read_verdict
    from readiness_reuse import safe_artifact,verify_accepted
    from readiness_coverage import _table_rows
    root=Path(root).resolve();failed_owner=Path(failed_owner).resolve()
    run=Run.create(root,'increment-pruning',dict(DOMAIN))
    try:
        run.capture_source()
        manifest,inventory=source_snapshot(failed_owner)
        verdict=read_verdict(failed_owner)
        coverage=verdict.get('checks',{}).get('changed-entry-coverage',{})
        if manifest.get('type')!='iteration' or verdict.get('status')!='uncovered' or coverage.get('complete') is not True:
            raise ValueError('Pruning requires a preserved complete cold uncovered candidate')
        if source_inventory(root)!=inventory['files']:
            raise ValueError('Current candidate changed since failed cold evidence; preserve edits and diagnose')
        required={(key['address'],key['mx']):key['reason'] for key in coverage.get('required_keys',[])}
        remove={(key['address'],key['mx']) for key in coverage.get('uncovered_keys',[])}
        if not remove or any(required.get(key)!='added' for key in remove):
            raise ValueError('Only newly added unexecuted keys may be deferred; affected legacy bodies require diagnosis')
        increment=manifest.get('increment',{})
        baseline=verify_accepted(increment['baseline_path'],require_pointer=True)
        expected=baseline['manifest']['identities']['generated-key-inventory']
        old=json.loads(safe_artifact(increment['baseline_path'],expected['evidence']).read_text())
        previous={(key['address'],key['mx']) for key in old['keys']}
        if remove & previous:raise ValueError('Cannot prune legacy accepted keys')
        candidate_inventory=json.loads((failed_owner/'generated/key-inventory.json').read_text())
        candidate={(key['address'],key['mx']):key for key in candidate_inventory['keys']}
        proposal=run.path/'generated/functions.toml'
        proposal.write_bytes((root/'config/functions.toml').read_bytes())
        with proposal.open('a') as target:
            for address,mx in sorted(remove):
                target.write('\n[[excluded_step]]\naddr = "{:06X}"\nvariant = "m{}x{}"\nreason = "deferred after complete cold unexecuted key evidence"\n'.format(address,mx >> 1,mx & 1))
        environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/'build/generator'),CARGO_INCREMENTAL='0')
        stage=run.run_stage('pruning-generator-build',['cargo','build','--release','-p','recomp','--locked','--offline'],env=environment,timeout=timeout)
        if stage['status']!='pass':raise ValueError('Pruning generator build failed')
        run.record_artifact('pruning-generator',run.path/'build/generator/release/recomp','generated/recomp')
        result=run.run_stage('pruned-regeneration',[run.path/'generated/recomp','--rom',root/'rom/sm.sfc','--functions',proposal,
            '--out',run.path/'generated/game.rs','--steps-out',run.path/'generated/steps.rs'],timeout=timeout)
        if result['status']!='pass':raise ValueError('Pruned regeneration failed')
        actual={(key['address'],key['mx']) for key in _table_rows((run.path/'generated/steps.rs').read_text())}
        if actual!=set(candidate)-remove:raise ValueError('Pruned compiled table differs beyond deferred new keys')
        report={'status':'pass','complete':True,'prior_config_sha256':digest(root/'config/functions.toml'),
            'proposed_config_sha256':digest(proposal),'steps_sha256':digest(run.path/'generated/steps.rs'),
            'deferred':[candidate[key] for key in sorted(remove)],'failed_run_id':manifest['run_id'],
            'pruning_parent_path':increment['preparation_path'],'pruning_failed_path':str(failed_owner),
            'pruning_parent_sha256':digest(Path(increment['preparation_path'])/'checks/candidate-proposal.json')}
        run.record_check('candidate-proposal',report)
        increment=dict(increment,preparation_path=str(run.path),failed_candidate=str(failed_owner),
            deferred=increment.get('deferred',[])+report['deferred'])
        run.manifest.update(type='increment-preparation',accepted_parent=manifest['accepted_parent'],increment=increment)
        run.save_manifest();run.check_sources()
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest()
        run.finalize('pass','candidate-proposal','Pruned new unexecuted keys; full fresh validation still required',mandatory_checks=['candidate-proposal'])
        return run.path
    except Exception as error:
        run.finalize('error','candidate-proposal',str(error));raise


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--selection',type=Path)
    parser.add_argument('--preparation',type=Path)
    parser.add_argument('--prune-uncovered',type=Path)
    parser.add_argument('--prepare-only',action='store_true')
    parser.add_argument('--original-run',type=Path)
    parser.add_argument('--timeout',type=float)
    parser.add_argument('--jobs',type=int,default=4)
    args=parser.parse_args(argv)
    try:
        if args.preparation is None and args.selection is None and args.prune_uncovered is None:raise ValueError('--selection, --preparation or --prune-uncovered required')
        owner=prune_uncovered(args.root,args.prune_uncovered,args.timeout) if args.prune_uncovered else args.preparation or prepare(args.root,args.selection,args.timeout)
        if args.prepare_only:return 0
        increment=apply_preparation(args.root,owner)
        from readiness_orchestration import run_readiness
        return run_readiness(args.root,Path(increment['baseline_path']),args.original_run,args.timeout,args.jobs,True,increment)
    except Exception as error:
        print('Supported increment failed: '+str(error),file=sys.stderr);return 2


if __name__ == '__main__':
    raise SystemExit(main())
