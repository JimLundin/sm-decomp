#!/usr/bin/env python3
"""Choose one measured routine/width increment without changing config or acceptance."""
import argparse
import hashlib
import json
import os
import sys
from pathlib import Path

from readiness import Run, digest, preflight, write_json
from readiness_contract import DOMAIN
from readiness_reuse import safe_artifact, verify_accepted


def bind_profile(run, owner, manifest, report_path, profile_path):
    """Import complete address/MX/opcode counts only from an exact cold replay."""
    report = json.loads(Path(report_path).read_text())
    profile = json.loads(Path(profile_path).read_text())
    inputs = manifest['identities']['inputs']
    binary_id = manifest['identities']['generated-binary']
    canonical = safe_artifact(owner, inputs['input_evidence'])
    if digest(canonical) != inputs['input_sha256']:
        raise ValueError('Baseline canonical input checksum differs')
    selected = b''.join(canonical.read_bytes().splitlines(keepends=True)[:DOMAIN['frames']])
    expected = {name:inputs[name] for name in ('rom_sha256','movie_sha256','input_sha256')}
    expected.update(binary_sha256=binary_id['sha256'], compiled_rom_sha256=inputs['rom_sha256'],
                    selected_input_sha256=hashlib.sha256(selected).hexdigest())
    if (report.get('status') != 'pass' or report.get('complete') is not True
            or report.get('expected_frames') != DOMAIN['frames'] or report.get('completed_frames') != DOMAIN['frames']
            or report.get('matched_frames') != DOMAIN['frames'] or report.get('mode') != 'generated'
            or report.get('profile') != inputs['profile'] or report.get('seed') != inputs['seed']
            or report.get('controller') != 'P1' or report.get('regions') != ['CPU+clocks','WRAM','framebuffer']
            or report.get('available_frames') != inputs['frames']
            or any(report.get('identities',{}).get(name) != value for name,value in expected.items())
            or report.get('coverage_sha256') != digest(profile_path)):
        raise ValueError('Instruction profile lacks complete identity-bound accepted prefix replay')
    for category,total in (('compiled','generated_instructions'),('interpreted','interpreted_instructions')):
        rows = profile.get(category)
        if not isinstance(rows,list) or any(not isinstance(row.get('count'),int) or isinstance(row['count'],bool)
              or row['count'] < 0 or not isinstance(row.get('address'),int) or not 0 <= row['address'] <= 0xffffff
              or row.get('mx') not in range(4) or not isinstance(row.get('emulation'),bool)
              or not isinstance(row.get('opcode'),int) or not 0 <= row['opcode'] <= 255
              or not isinstance(row.get('pattern'),str) for row in rows):
            raise ValueError('Instruction profile has malformed address/MX/opcode counts')
        if sum(row['count'] for row in rows) != profile.get(total):
            raise ValueError('Instruction profile totals differ from individual counts')
    if (profile.get('generated_dispatches') != report.get('generated_steps')
            or profile.get('interpreted_dispatches') != report.get('interpreted_steps')
            or profile.get('noninstruction_dispatches') != report.get('generated_steps') + report.get('interpreted_steps')
                - profile['generated_instructions'] - profile['interpreted_instructions']):
        raise ValueError('Instruction profile dispatch totals differ from actual replay')
    target = run.path/'generated/selection-profile.json'
    run.record_artifact('selection-profile',profile_path,str(target.relative_to(run.path)))
    run.guard_input(target)
    provenance = {'schema_version':1,'baseline_run_id':manifest['run_id'],
                  'baseline_source_sha256':manifest['identities']['source']['sha256'],
                  'profile_sha256':digest(target),'report_sha256':digest(report_path),
                  'identities':expected,'domain':dict(DOMAIN)}
    write_json(run.path/'checks/selection-profile-provenance.json',provenance)
    run.record_check('selection-profile',dict(provenance,status='pass',complete=True))
    return target


def obtain_profile(run, owner, verified, timeout=None):
    """Use retained complete counts; otherwise replay the preserved accepted binary."""
    manifest = verified['manifest']
    gate = verified['verdict']['checks']['generated-parity']
    report_path = safe_artifact(owner,gate['evidence'])
    raw = json.loads(report_path.read_text())
    profile_path = report_path.parent/'coverage.json'
    if profile_path.exists() and raw.get('coverage_sha256'):
        run.manifest['selection_profile_origin'] = 'retained accepted cold replay'
        run.save_manifest()
        return bind_profile(run,owner,manifest,report_path,profile_path)
    # Old accepted formats may lack instruction counts. Refresh through its own
    # executable rather than silently attributing current source to old execution.
    binary = safe_artifact(owner,manifest['identities']['generated-binary']['evidence'])
    if digest(binary) != manifest['identities']['generated-binary']['sha256']:
        raise ValueError('Accepted profile refresh executable checksum differs')
    run.guard_input(binary)
    output = run.path/'diagnostics/baseline-profile-refresh'
    result = run.run_stage('baseline-profile-refresh',[binary,run.root/'movies/super-metroid-any-3653.lsmv',
                          '--generated','--frames',str(DOMAIN['frames']),'--rom',run.root/'rom/sm.sfc','--out',output],timeout=timeout)
    if result['status'] != 'pass':
        raise ValueError('Accepted instruction profile refresh failed; inspect retained process logs')
    run.manifest['selection_profile_origin'] = 'fresh replay of preserved accepted executable'
    run.save_manifest()
    return bind_profile(run,owner,manifest,output/'parity.json',output/'coverage.json')


def verify_selection_result(report):
    """Reject malformed executable output before treating it as a measured choice."""
    if report.get('schema_version') != 1 or report.get('outcome') not in (
            'supported-routine','needs-timed-pattern','no-covered-candidate'):
        raise ValueError('Selection result schema/outcome differs')
    ranked = report.get('ranked')
    if not isinstance(ranked,list):
        raise ValueError('Selection ranked alternatives missing')
    if report['outcome'] == 'supported-routine':
        if not ranked or report.get('chosen') != ranked[0]:
            raise ValueError('Selection choice must be the first ranked candidate')
    elif report.get('chosen') is not None or ranked:
        raise ValueError('No-work selection cannot include a chosen/ranked candidate')
    previous = None
    for candidate in ranked:
        address,mx = candidate.get('address'),candidate.get('mx')
        if not isinstance(address,int) or isinstance(address,bool) or not 0 <= address <= 0xffffff or mx not in range(4):
            raise ValueError('Selection candidate address/width invalid')
        if candidate.get('variant') != 'm{}x{}'.format(mx >> 1,mx & 1):
            raise ValueError('Selection candidate width name differs')
        keys = candidate.get('new_executed_keys')
        if not isinstance(keys,list) or not keys:
            raise ValueError('Selection candidate lacks newly executed keys')
        seen = set()
        gain = 0
        for key in keys + candidate.get('deferred_keys',[]) + candidate.get('overlapping_keys',[]):
            identity = (key.get('address'),key.get('mx'))
            count = key.get('count')
            values = key.get('bytes')
            if (identity in seen or not isinstance(identity[0],int) or not 0 <= identity[0] <= 0xffffff
                    or identity[1] not in range(4) or not isinstance(count,int) or isinstance(count,bool) or count < 0
                    or not isinstance(values,list) or not values or values[0] != key.get('opcode')
                    or any(not isinstance(value,int) or not 0 <= value <= 255 for value in values)):
                raise ValueError('Selection key count/bytes/identity invalid')
            seen.add(identity)
        if any(key['count'] <= 0 for key in keys):
            raise ValueError('Selection newly compiled key was not observed')
        gain = sum(key['count'] for key in keys)
        if candidate.get('estimated_compiled_step_gain') != gain or gain <= 0:
            raise ValueError('Selection estimated gain differs from actual key counts')
        complexity = candidate.get('complexity',{})
        if any(not isinstance(complexity.get(name),int) or complexity[name] <= 0 for name in ('instruction_keys','blocks')):
            raise ValueError('Selection bounded CFG complexity invalid')
        order = (-gain,complexity['instruction_keys'],complexity['blocks'],address,mx)
        if previous is not None and previous > order:
            raise ValueError('Selection alternatives are not in stable measured order')
        previous = order
    if report['outcome'] == 'needs-timed-pattern' and not report.get('observed_unsupported_patterns'):
        raise ValueError('Timed pattern route lacks observed unsupported work')
    return report


def select(run, owner, timeout=None):
    """Run selection inside an allocated immutable evidence lifecycle."""
    verified = verify_accepted(owner,require_pointer=True)
    from readiness_orchestration import verify_origins
    verify_origins(owner,verified)
    for artifact in verified['guarded']:
        run.guard_input(artifact)
    manifest = verified['manifest']
    run.manifest['accepted_parent'] = manifest['run_id']
    run.manifest['selection_baseline'] = {'run_id':manifest['run_id'],'run_path':str(Path(owner).resolve()),
        'source_sha256':manifest['identities']['source']['sha256'],'domain':dict(DOMAIN)}
    run.save_manifest()
    run.record_check('selection-baseline',{'status':'pass','complete':True,'run_id':manifest['run_id']})
    failures = preflight(run)
    if failures:
        raise ValueError('Readiness preflight required: '+'; '.join(failures))
    actual_inputs = run.manifest['identities']['inputs']
    for name in ('rom_sha256','movie_sha256','input_sha256','seed','profile','region','controller','reset'):
        if actual_inputs[name] != manifest['identities']['inputs'][name]:
            raise ValueError('Current input domain differs from baseline; independently validate readiness before selection')
    # Configuration changes are preserved, but they cannot borrow an old key
    # inventory/profile as if they were the already accepted predecessor.
    if digest(run.root/'config/functions.toml') != digest(Path(owner)/'source/files/config/functions.toml'):
        raise ValueError('Current reviewed functions differ from accepted baseline; validate readiness before measured selection')
    if digest(run.root/'crates/recomp/src/step.rs') != digest(Path(owner)/'source/files/crates/recomp/src/step.rs'):
        raise ValueError('Timed instruction support differs from accepted validation; refresh pattern/readiness gates before selection')
    current_shared = {entry['path']:entry['sha256'] for entry in run.source_files if entry['path'].startswith(
        ('crates/snes/src/','crates/runtime/src/','crates/diff/src/','tools/patches/'))}
    baseline_shared = {entry['path']:entry['sha256'] for entry in verified['inventory']['files'] if entry['path'].startswith(
        ('crates/snes/src/','crates/runtime/src/','crates/diff/src/','tools/patches/'))}
    if current_shared != baseline_shared:
        raise ValueError('Shared execution/observer inputs changed; refresh readiness before selection')
    profile = obtain_profile(run,owner,verified,timeout)
    inventory = safe_artifact(owner,manifest['identities']['generated-key-inventory']['evidence'])
    run.record_artifact('selection-baseline-inventory',inventory,'generated/selection-baseline-inventory.json')
    raw = safe_artifact(owner,verified['verdict']['checks']['generated-parity']['evidence'])
    discoveries = raw.parent/'misses.csv'
    if not discoveries.exists() and (run.path/'diagnostics/baseline-profile-refresh/misses.csv').exists():
        discoveries = run.path/'diagnostics/baseline-profile-refresh/misses.csv'
    if not discoveries.exists():
        raise ValueError('Baseline call-target discovery missing; refresh accepted profile before selecting routine roots')
    run.record_artifact('selection-discovery',discoveries,'generated/selection-discovery.csv')
    environment = dict(os.environ,CARGO_TARGET_DIR=str(run.path/'build/selection'),CARGO_INCREMENTAL='0')
    build = run.run_stage('selection-generator-build',['cargo','build','--release','-p','recomp','--locked','--offline'],env=environment,timeout=timeout)
    if build['status'] != 'pass':
        raise ValueError('Selection generator build failed; inspect retained process logs')
    binary = run.path/'build/selection/release/recomp'
    run.record_artifact('selection-generator',binary,'generated/selection-generator')
    output = run.path/'checks/selection.json'
    result = run.run_stage('selection-ranking',[run.path/'generated/selection-generator','--rom',run.root/'rom/sm.sfc',
        '--functions',run.root/'config/functions.toml','--selection-profile',profile,
        '--selection-inventory',run.path/'generated/selection-baseline-inventory.json',
        '--selection-entries',run.path/'generated/selection-discovery.csv','--selection-out',output],timeout=timeout)
    if result['status'] != 'pass':
        raise ValueError('Measured routine selection failed; inspect retained process logs')
    report = verify_selection_result(json.loads(output.read_text()))
    report.update(baseline=run.manifest['selection_baseline'],source_sha256=run.manifest['identities']['source']['sha256'],
                  profile_sha256=digest(profile),baseline_inventory_sha256=digest(inventory),
                  discovery_sha256=digest(discoveries),identities=actual_inputs)
    # Command result is retained verbatim; the separately bound report adds provenance.
    write_json(run.path/'selection.json',report)
    run.record_artifact('selection-result',run.path/'selection.json')
    run.record_check('measured-selection',{'status':'pass','complete':True,'outcome':report['outcome'],
                                         'evidence':'selection.json','sha256':digest(run.path/'selection.json')})
    run.check_sources()
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--predecessor',type=Path)
    parser.add_argument('--timeout',type=float)
    args = parser.parse_args(argv)
    run = Run.create(args.root,'iteration-selection',dict(DOMAIN))
    print('Run: '+str(run.path),flush=True)
    print('Brief: '+str(run.path/'brief.md'),flush=True)
    try:
        owner = args.predecessor
        if owner is None:
            pointer = json.loads((run.root/'out/accepted.json').read_text())
            owner = Path(pointer['run_path'])
        report = select(run,owner,args.timeout)
        chosen = report['chosen']
        reason = ('Selected ${:06X} {} with {} newly compiled observed instructions'.format(
            chosen['address'],chosen['variant'],chosen['estimated_compiled_step_gain']) if chosen else report['outcome'])
        from readiness_reuse import seal_owned_evidence
        seal_owned_evidence(run);run.save_manifest()
        return run.finalize('pass','measured-selection',reason,mandatory_checks=['selection-baseline','selection-profile','measured-selection'],
            brief_text=reason+'\n\nDomain: Japan/USA any%, input indices 0..17999; baseline '+report['baseline']['run_id']+
            '.\n\n[Ranked choice and exact keys](selection.json) · [Profile provenance](checks/selection-profile-provenance.json) · [Manifest](manifest.json) · [Logs](logs/)\n')
    except KeyboardInterrupt:
        return run.finalize('incomplete','measured-selection','Selection interrupted; acceptance unchanged')
    except Exception as error:
        return run.finalize('error','measured-selection','Readiness baseline/setup required before selection: '+str(error))


if __name__ == '__main__':
    raise SystemExit(main())
