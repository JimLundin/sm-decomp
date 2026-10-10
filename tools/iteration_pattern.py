#!/usr/bin/env python3
"""Prove one prospective timed pattern, then prepare a guarded cold candidate."""
import argparse
import json
import os
import sys
from pathlib import Path

from readiness import Run, digest, source_inventory, write_json, read_verdict
from readiness_contract import DOMAIN
from pattern_contract import DECLARATION, load_declaration


def proof(root, opcode='ad', timeout=None):
    root=Path(root).resolve()
    run=Run.create(root,'pattern-proof',dict(DOMAIN))
    print('Pattern proof: '+str(run.path),flush=True)
    try:
        run.capture_source()
        declaration=load_declaration(root,prospective=opcode)
        output=run.path/'checks/patterns'
        stage=run.run_stage('prospective-pattern-proof',[sys.executable,root/'tools/readiness_patterns.py',
            '--prospective',opcode,'--out',output,'--timing'],timeout=timeout,
            env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0'))
        if stage['status']!='pass':raise ValueError('Prospective proof did not complete; inspect preserved logs')
        from readiness_orchestration import verify_pattern_evidence,verify_timing
        instruction=json.loads((output/'instruction.json').read_text())
        # Parent proof source/compiler are exact identities, rather than an
        # inherited accepted run mislabeled as this candidate.
        run.manifest['identities']['compiler']={'version':instruction['compiler']}
        run.save_manifest()
        verify_pattern_evidence(output,run.manifest)
        timing=json.loads((output/'timing.json').read_text())
        verify_timing(output,timing,run.path/'source/files')
        if timing['source_sha256']!=instruction['source_sha256']:raise ValueError('Proof timing source differs')
        report={'status':'pass','complete':True,'opcode':opcode,'declaration':declaration,
            'source_sha256':run.manifest['identities']['source']['sha256'],
            'instruction_sha256':digest(output/'instruction.json'),'timing_sha256':digest(output/'timing.json'),
            'production_enabled':False,'expected':instruction['expected'],'passed':instruction['passed']}
        run.record_check('prospective-proof',report)
        run.check_sources()
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest()
        run.finalize('pass','prospective-proof','Complete prospective proof; production support remains disabled',
            mandatory_checks=['prospective-proof'],brief_text='Complete prospective '+opcode.upper()+' proof. Production support remains disabled.\n\n[Instruction corpus](checks/patterns/instruction.json) · [Independent timing](checks/patterns/timing.json) · [Source](source/inventory.json)\n')
        return run.path
    except Exception as error:
        run.finalize('error','prospective-proof',str(error));raise


def verify_proof(owner):
    from readiness_orchestration import source_snapshot,verify_pattern_evidence,verify_timing
    owner=Path(owner).resolve();manifest,inventory=source_snapshot(owner);verdict=read_verdict(owner)
    from readiness_reuse import verify_owned_evidence
    verify_owned_evidence(owner,manifest)
    if manifest.get('type')!='pattern-proof' or verdict.get('status')!='pass' or verdict.get('acceptance_eligible'):
        raise ValueError('Prospective proof is not a completed independent scoped run')
    report=verdict['checks']['prospective-proof'];output=owner/'checks/patterns'
    if report.get('status')!='pass' or report.get('complete') is not True or report.get('production_enabled') is not False:
        raise ValueError('Prospective proof completion missing')
    for name in ('instruction','timing'):
        if digest(output/(name+'.json'))!=report[name+'_sha256']:raise ValueError('Prospective proof checksum differs')
    instruction=verify_pattern_evidence(output,manifest)
    timing=json.loads((output/'timing.json').read_text());verify_timing(output,timing,owner/'source/files')
    if instruction.get('prospective_opcode')!=report['opcode'] or timing.get('prospective_opcode')!=report['opcode'] or timing['source_sha256']!=manifest['identities']['source']['sha256']:
        raise ValueError('Prospective proof declaration/source differs')
    return manifest,inventory,report


def prepare(root, selection_owner, proof_owner, timeout=None):
    """Prove exhaustion and preserve the exact proposal before any mutation."""
    root=Path(root).resolve();selection_owner=Path(selection_owner).resolve();proof_owner=Path(proof_owner).resolve()
    run=Run.create(root,'pattern-preparation',dict(DOMAIN))
    print('Pattern preparation: '+str(run.path),flush=True)
    try:
        run.capture_source()
        from readiness_orchestration import source_snapshot,verify_origins
        from readiness_reuse import verify_accepted,safe_artifact
        from iteration_selection import verify_selection_result
        proof_manifest,proof_inventory,proof_report=verify_proof(proof_owner)
        if source_inventory(root)!=proof_inventory['files']:raise ValueError('Prospective proof source changed; rerun complete proof')
        selection_manifest,selection_inventory=source_snapshot(selection_owner)
        from readiness_reuse import verify_owned_evidence
        verify_owned_evidence(selection_owner,selection_manifest)
        selected=verify_selection_result(json.loads((selection_owner/'selection.json').read_text()))
        verdict=read_verdict(selection_owner)
        if selection_manifest.get('type')!='iteration-selection' or verdict.get('status')!='pass' or selected['outcome']!='needs-timed-pattern':
            raise ValueError('Useful supported routine/width additions must be exhausted first')
        raw=json.loads((selection_owner/'checks/selection.json').read_text())
        if raw['outcome']!=selected['outcome'] or raw['ranked']!=selected['ranked'] or raw['observed_unsupported_patterns']!=selected['observed_unsupported_patterns']:
            raise ValueError('Measured selection differs from preserved executed ranking')
        process=next((row for row in selection_manifest.get('commands',[]) if row.get('name')=='selection-ranking'),None)
        if process is None:
            process=json.loads((selection_owner/'logs/selection-ranking.json').read_text())
        if process.get('status')!='pass' or process.get('complete') is not True or process.get('returncode')!=0:
            raise ValueError('Measured supported exhaustion ranking did not execute')
        if digest(selection_owner/'selection.json')!=verdict['checks']['measured-selection']['sha256'] or selected['source_sha256']!=run.manifest['identities']['source']['sha256']:
            raise ValueError('Measured selection source/result differs')
        opcode=proof_report['opcode'];declaration=load_declaration(root,prospective=opcode);pattern=declaration['patterns'][0]
        observed=next((row for row in selected['observed_unsupported_patterns'] if row['pattern']==pattern['mnemonic']+'/'+pattern['mode']),None)
        if not observed or observed['count']<=0:raise ValueError('Proved pattern was not observed in the validated domain')
        baseline_path=Path(selected['baseline']['run_path']);baseline=verify_accepted(baseline_path,require_pointer=True)
        verify_origins(baseline_path,baseline)
        if baseline['manifest']['run_id']!=selected['baseline']['run_id']:raise ValueError('Selected accepted predecessor changed')
        profile=json.loads((selection_owner/'generated/selection-profile.json').read_text())
        keys=[row for row in profile['interpreted'] if row['opcode']==int(opcode,16) and not row['emulation'] and row['count']>0]
        keys.sort(key=lambda row:(-row['count'],row['address'],row['mx']))
        if not keys:raise ValueError('No real-ROM instruction key for proved pattern')
        chosen=keys[0]
        proposed=json.loads((root/DECLARATION).read_text())
        proposed['patterns'].append(pattern)
        proposed['prospective_patterns']=[row for row in proposed['prospective_patterns'] if row['opcode']!=opcode]
        for field in ('timing_tests','timing_oracles','hardware_conditions'):
            proposed[field]=list(dict.fromkeys(proposed[field]+pattern[field]))
        declaration_path=run.path/'generated/timed_patterns.json';write_json(declaration_path,proposed)
        config=run.path/'generated/functions.toml';config.write_bytes((root/'config/functions.toml').read_bytes())
        # One observed instruction/MX slice. Additional CFG bodies are excluded
        # below; the source-owned table must match the reviewed bounded choice.
        with config.open('a') as target:
            target.write('\n[[function]]\naddr = "{:06X}"\nvariants = ["m{}x{}"]\n'.format(chosen['address'],chosen['mx']>>1,chosen['mx']&1))
        environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/'build/generator'),CARGO_INCREMENTAL='0')
        stage=run.run_stage('pattern-proposal-generator',['cargo','build','--release','-p','recomp','--locked','--offline'],env=environment,timeout=timeout)
        if stage['status']!='pass':raise ValueError('Prospective proposal build failed')
        generator=run.path/'build/generator/release/recomp';run.record_artifact('proposal-generator',generator,'generated/recomp')
        generated=run.path/'generated/proposed-steps.rs'
        command=[run.path/'generated/recomp','--rom',root/'rom/sm.sfc','--functions',config,'--out',run.path/'generated/game.rs','--steps-out',generated,'--prospective-pattern',opcode]
        stage=run.run_stage('pattern-proposal-generation',command,timeout=timeout)
        if stage['status']!='pass':raise ValueError('Prospective proposal generation failed')
        from readiness_coverage import _table_rows
        prior=json.loads(safe_artifact(baseline_path,baseline['manifest']['identities']['generated-key-inventory']['evidence']).read_text())
        previous={(row['address'],row['mx']) for row in prior['keys']}
        actual={(row['address'],row['mx']):row for row in _table_rows(generated.read_text())}
        desired=(chosen['address'],chosen['mx'])
        if desired not in actual or actual[desired]['opcode']!=int(opcode,16) or not previous<=set(actual):raise ValueError('Prospective table omits selected or accepted keys')
        deferred=[row for key,row in sorted(actual.items()) if key not in previous and key!=desired]
        with config.open('a') as target:
            for row in deferred:target.write('\n[[excluded_step]]\naddr = "{:06X}"\nvariant = "m{}x{}"\nreason = "deferred outside bounded timed-pattern increment"\n'.format(row['address'],row['mx']>>1,row['mx']&1))
        stage=run.run_stage('pattern-pruned-generation',command,timeout=timeout)
        if stage['status']!='pass' or {(row['address'],row['mx']) for row in _table_rows(generated.read_text())}!=previous|{desired}:
            raise ValueError('Prospective pruned table differs from one measured pattern key')
        report={'status':'pass','complete':True,'opcode':opcode,'proof_path':str(proof_owner),'proof_report_sha256':digest(proof_owner/'checks/prospective-proof.json'),
            'selection_path':str(selection_owner),'selection_sha256':digest(selection_owner/'selection.json'),
            'prior_declaration_sha256':digest(root/DECLARATION),'proposed_declaration_sha256':digest(declaration_path),
            'prior_config_sha256':digest(root/'config/functions.toml'),'proposed_config_sha256':digest(config),
            'steps_sha256':digest(generated),'chosen':chosen,'deferred':deferred,
            'rationale':'Supported additions exhausted; one observed native LDA absolute key, no addressing penalty; unrelated unsupported patterns remain interpreted'}
        run.record_check('pattern-proposal',report)
        run.manifest.update(accepted_parent=baseline['manifest']['run_id'],increment={'kind':'timed-pattern-expansion','baseline_path':str(baseline_path),
            'preparation_path':str(run.path),'pattern_preparation_path':str(run.path),'selection_path':str(selection_owner),'pattern_proof_path':str(proof_owner),'opcode':opcode,'chosen':chosen,'deferred':deferred})
        run.save_manifest();run.check_sources()
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest()
        run.finalize('pass','pattern-proposal','Bounded promotion prepared; cold candidate gates still required',mandatory_checks=['pattern-proposal'])
        return run.path
    except Exception as error:
        run.finalize('error','pattern-proposal',str(error));raise


def apply_preparation(root, owner):
    from readiness_orchestration import source_snapshot
    root=Path(root).resolve();owner=Path(owner).resolve();manifest,inventory=source_snapshot(owner);verdict=read_verdict(owner)
    if manifest.get('type')!='pattern-preparation' or verdict.get('status')!='pass':raise ValueError('Pattern preparation did not complete')
    from readiness_reuse import verify_owned_evidence
    verify_owned_evidence(owner,manifest)
    if source_inventory(root)!=inventory['files']:raise ValueError('Candidate changed after prepared promotion')
    report=verdict['checks']['pattern-proposal'];verify_proof(report['proof_path'])
    for relative,name,artifact in ((DECLARATION,'declaration','timed_patterns.json'),('config/functions.toml','config','functions.toml')):
        proposal=owner/'generated'/artifact
        if digest(root/relative)!=report['prior_'+name+'_sha256'] or digest(proposal)!=report['proposed_'+name+'_sha256']:
            raise ValueError('Prepared promotion identity differs')
    # Both previous files remain preserved by the preparation source snapshot.
    for relative,artifact in ((DECLARATION,'timed_patterns.json'),('config/functions.toml','functions.toml')):
        (root/relative).write_bytes((owner/'generated'/artifact).read_bytes())
    return manifest['increment']


def verify_increment(owner, verified):
    """Acceptance rechecks the exact prospective proof and declared promotion."""
    increment=verified['manifest'].get('increment',{})
    if increment.get('kind')!='timed-pattern-expansion':return
    preparation=Path(increment['pattern_preparation_path']);report=read_verdict(preparation)['checks']['pattern-proposal']
    from readiness_orchestration import source_snapshot
    from readiness_reuse import verify_owned_evidence
    preparation_manifest,_=source_snapshot(preparation);verify_owned_evidence(preparation,preparation_manifest)
    _,_,proof_report=verify_proof(increment['pattern_proof_path'])
    if proof_report['opcode']!=increment['opcode'] or report['opcode']!=increment['opcode']:raise ValueError('Promoted pattern proof differs')
    snapshot=Path(owner)/'source/files'
    if digest(snapshot/DECLARATION)!=report['proposed_declaration_sha256']:raise ValueError('Promoted declaration differs from independently proved proposal')
    for entry in json.loads((preparation/'source/inventory.json').read_text())['files']:
        if entry['path'] in (DECLARATION,'config/functions.toml'):continue
        if digest(snapshot/entry['path'])!=entry['sha256']:raise ValueError('Promoted emitter/runtime source differs from proved preparation')
    coverage=verified['verdict']['checks']['changed-entry-coverage'];chosen=report['chosen']
    if not any(row['address']==chosen['address'] and row['mx']==chosen['mx'] and row['count']>0 for row in coverage.get('executed',[])):
        raise ValueError('Promoted pattern lacks real-ROM generated execution')
    instruction=verified['verdict']['checks']['instruction'];timing=verified['verdict']['checks']['timing']
    if increment['opcode'] not in instruction['opcodes'] or timing['source_sha256']!=verified['manifest']['identities']['source']['sha256']:
        raise ValueError('Promoted candidate requires fresh complete pattern/timing source proof')


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__);sub=parser.add_subparsers(dest='command',required=True)
    check=sub.add_parser('prove');check.add_argument('--opcode',default='ad');check.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent);check.add_argument('--timeout',type=float)
    promote=sub.add_parser('promote');promote.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent);promote.add_argument('--selection',type=Path,required=True);promote.add_argument('--proof',type=Path,required=True);promote.add_argument('--timeout',type=float)
    args=parser.parse_args(argv)
    try:
        if args.command=='prove':proof(args.root,args.opcode,args.timeout);return 0
        owner=prepare(args.root,args.selection,args.proof,args.timeout);increment=apply_preparation(args.root,owner)
        from readiness_orchestration import run_readiness
        return run_readiness(args.root,Path(increment['baseline_path']),timeout=args.timeout,iteration=True,increment=increment)
    except Exception as error:print(str(error),file=sys.stderr);return 2


if __name__=='__main__':raise SystemExit(main())
