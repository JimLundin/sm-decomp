#!/usr/bin/env python3
"""Prepare exactly one measured supported routine and validate it through cold acceptance."""
import argparse
import json
import os
import re
import sys
from pathlib import Path

from readiness import Run, digest, source_inventory
from readiness_contract import DOMAIN
from readiness_coverage import comparable_progress


def progress_report(coverage, increment=None):
    """Expansion is progress only when comparable measured generated work grows."""
    kind = (increment or {}).get('kind','supported-expansion')
    if kind not in ('supported-expansion','timed-pattern-expansion','correctness-fix'):
        return {'schema_version':1,'status':'uncovered','complete':True,'kind':kind,'reason':'Unknown increment kind'}
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


def verify_reproduction(owner,increment):
    """Bind a focused production instruction/bus regression and cold controls."""
    from readiness_reuse import safe_artifact
    reproduction=increment.get('reproduction',{})
    artifact=safe_artifact(owner,reproduction['evidence'])
    if digest(artifact)!=reproduction['sha256']:
        raise ValueError('Correctness reproduction checksum differs')
    actual=json.loads(artifact.read_text())
    if (actual.get('status')!='pass' or actual.get('complete') is not True
            or actual.get('kind')!='focused-instruction-regression'):
        raise ValueError('Correctness repair requires a verified focused regression')
    from readiness_orchestration import source_snapshot
    from readiness import read_verdict
    failure_path=Path(actual['failed_candidate'])
    source_snapshot(failure_path)
    failure=read_verdict(failure_path)
    if failure.get('status')!='divergence':
        raise ValueError('Correctness reproduction lacks preserved failing candidate')
    diagnostic=json.loads(safe_artifact(failure_path,actual['failure_evidence']).read_text())
    if digest(safe_artifact(failure_path,actual['failure_evidence']))!=actual['failure_sha256'] or diagnostic.get('status')!='divergence':
        raise ValueError('Correctness reproduction failing instruction evidence differs')
    for name in ('predecessor','disabled_entry'):
        control=actual['controls'][name]
        path=safe_artifact(failure_path,control['evidence'])
        report=json.loads(path.read_text())
        if digest(path)!=control['sha256'] or report.get('status')!='pass' or report.get('complete') is not True:
            raise ValueError('Correctness reproduction control is not a complete pass')
    fixed=safe_artifact(owner,actual['fixed_evidence'])
    report=json.loads(fixed.read_text())
    if digest(fixed)!=actual['fixed_sha256'] or report.get('status')!='pass' or report.get('complete') is not True or report.get('completed_instructions',0)<=0:
        raise ValueError('Correctness reproduction repaired focus did not pass')
    if (report.get('start_input_index'),report.get('start_instruction'),report.get('expected_instructions')) != (
            diagnostic.get('start_input_index'),diagnostic.get('start_instruction'),diagnostic.get('expected_instructions')):
        raise ValueError('Correctness reproduction repaired interval differs from failure')
    # Full origin verification later validates this same actual focus executable,
    # subprocess, checkpoint, trace/state and candidate source identity.
    focus=Path(owner)/'checks/instruction-diagnosis.json'
    if not focus.exists() or not any(row.get('evidence')==actual['fixed_evidence'] for row in json.loads(focus.read_text()).get('reports',[])):
        raise ValueError('Correctness reproduction is outside verified mandatory focus')
    return True


def verify_progress(owner,verified):
    """Recompute the extra gate from sealed real profiles before pointer publication."""
    manifest = verified['manifest']
    if manifest.get('type') != 'iteration':
        return
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
    parts=re.split(r'(?=^\[\[function\]\]\s*$)',source,flags=re.M)
    for index,part in enumerate(parts):
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
    verdict=read_verdict(owner)
    if manifest.get('type')!='increment-preparation' or verdict['status']!='pass':
        raise ValueError('Candidate preparation did not complete')
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
            'deferred':[candidate[key] for key in sorted(remove)],'failed_run_id':manifest['run_id']}
        run.record_check('candidate-proposal',report)
        increment=dict(increment,preparation_path=str(run.path),failed_candidate=str(failed_owner),
            deferred=increment.get('deferred',[])+report['deferred'])
        run.manifest.update(type='increment-preparation',accepted_parent=manifest['accepted_parent'],increment=increment)
        run.save_manifest();run.check_sources()
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
