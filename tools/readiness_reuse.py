#!/usr/bin/env python3
"""Conservative dependency policies and independently verified evidence reuse."""
import json
from pathlib import Path
from readiness import digest, inventory_digest, read_verdict, write_json
from readiness_contract import CONTRACT_VERSION, DOMAIN, MANDATORY_CHECKS, CANDIDATE_CHECKS, REUSABLE_CHECKS, validate_gate_scope

POLICY_VERSION = 1
CPU_ONLY = ('config/functions.toml', 'config/jump_tables.toml')


def executable(entry):
    # Only known documentation text is irrelevant. Unknown source/build inputs
    # remain dependencies, including additions and removals in the snapshot.
    path = Path(entry['path'])
    return not (path.suffix in ('.md', '.rst') and (path.parts[0] in ('docs', 'workflows') or path.name in ('README.md', 'NOTES.md')))


def shared(entry):
    path = entry['path']
    return executable(entry) and path not in CPU_ONLY and not path.startswith('crates/recomp/src/')


def settings(run):
    identities = run.manifest['identities']
    inputs = {name: value for name, value in identities['inputs'].items()
              if name not in ('input_evidence', 'compiled_table_rom', 'prior_compiled_table_sha256')}
    return {'stage_commands': {'check':['just','check'],'lint':['just','lint']}, 'inputs': inputs, 'hardware': identities['hardware'], 'compiler': identities['compiler'],
            'build_settings': run.manifest['build_settings']}


def dependency_policy(manifest, source_files, source_root):
    policies = {}
    class Context:
        pass
    context = Context()
    context.manifest = manifest
    referenced_docs = set()
    for entry in source_files:
        if not executable(entry):
            continue
        contents = (Path(source_root) / entry['path']).read_text(errors='ignore')
        for document in source_files:
            if not executable(document) and (document['path'] in contents or Path(document['path']).name in contents):
                referenced_docs.add(document['path'])
    for name in REUSABLE_CHECKS:
        files = [entry for entry in source_files if entry['path'] in referenced_docs or (shared(entry) if name in ("synchronization", "fallback-only-parity") else executable(entry))]
        policies[name] = {"files": files, "files_sha256": inventory_digest(files), "settings": settings(context)}
    return policies


def record_dependency_policy(run):
    """Preserve exact dependency inventories before checks; later consumers rederive them."""
    run.check_sources()
    policies = dependency_policy(run.manifest, run.source_files, run.path / 'source/files')
    report = {'schema_version': POLICY_VERSION, 'contract_version': CONTRACT_VERSION,
              'source_sha256': run.manifest['identities']['source']['sha256'], 'checks': policies,
              'candidate_checks_always_fresh': list(CANDIDATE_CHECKS),
              'cache_policy': 'exact complete identities and cold-history validation; changed candidate rebuilds cold'}
    path = run.path / 'checks/dependency-policy.json'
    write_json(path, report)
    run.record_artifact('dependency-policy', path)
    run.manifest['identities']['dependency-policy']['evidence'] = 'checks/dependency-policy.json'
    run.save_manifest()
    return report


def plan_reuse(run, predecessor=None):
    result = {name: {'action': 'fresh', 'reason': 'Candidate-specific evidence always required'} for name in CANDIDATE_CHECKS}
    for name in REUSABLE_CHECKS:
        result[name] = {'action': 'fresh', 'reason': 'No verified accepted predecessor'}
    if predecessor is None:
        return result
    try:
        accepted = verify_accepted(predecessor, require_pointer=True)
        old = verify_policy(predecessor, accepted)
        current = verify_policy(run.path, {'manifest': run.manifest, 'inventory': {'files': run.source_files}})
    except (OSError, ValueError, KeyError, TypeError, StopIteration) as error:
        for name in REUSABLE_CHECKS:
            result[name]['reason'] = 'Accepted origin/policy cannot be verified: ' + str(error)
        return result
    for name in REUSABLE_CHECKS:
        if old['checks'][name] != current['checks'][name]:
            before = {entry['path']: entry['sha256'] for entry in old['checks'][name]['files']}
            after = {entry['path']: entry['sha256'] for entry in current['checks'][name]['files']}
            result[name] = {'action': 'fresh', 'reason': 'Executable/build dependencies or settings changed',
                            'changed_dependencies': sorted(path for path in set(before) | set(after) if before.get(path) != after.get(path))}
            continue
        try:
            evidence = verify_check(predecessor, name, accepted)
            result[name] = {'action': 'reuse', 'reason': 'Verified accepted scope and unchanged dependencies',
                            'run_id': accepted['manifest']['run_id'], 'dependency_sha256': inventory_digest(old['checks'][name]),
                            'report': evidence['report'], 'artifacts': evidence['artifacts']}
        except (OSError, ValueError, KeyError, TypeError, StopIteration) as error:
            result[name]['reason'] = 'Accepted check origin cannot be verified: ' + str(error)
    return result


def verify_policy(owner, accepted):
    manifest = accepted['manifest']
    identity = manifest['identities']['dependency-policy']
    path = safe_artifact(owner, identity['evidence'])
    if digest(path) != identity['sha256']:
        raise ValueError('Dependency policy checksum differs')
    policy = json.loads(path.read_text())
    expected = dependency_policy(manifest, accepted['inventory']['files'], Path(owner) / 'source/files')
    if (policy.get('schema_version') != POLICY_VERSION or policy.get('contract_version') != CONTRACT_VERSION
            or policy.get('source_sha256') != manifest['identities']['source']['sha256'] or policy.get('checks') != expected):
        raise ValueError('Dependency policy does not match actual source/settings')
    return policy


def verify_check(owner, name, accepted):
    owner = Path(owner)
    manifest = accepted['manifest']
    report = accepted['verdict']['checks'][name]
    artifacts = [owner / 'checks' / (name + '.json')]
    if report.get('reused_from'):
        previous = Path(report['reused_from']['path'])
        if previous.resolve() == owner.resolve():
            raise ValueError('Cyclic reused evidence origin')
        prior = verify_accepted(previous, require_pointer=True)
        if prior['manifest']['started_utc'] >= manifest['started_utc']:
            raise ValueError('Reused evidence must precede the current run')
        previous_policy = verify_policy(previous, prior)
        owner_policy = verify_policy(owner, accepted)
        if (prior['manifest']['run_id'] != report['reused_from']['run_id'] or
                previous_policy['checks'][name] != owner_policy['checks'][name]):
            raise ValueError('Reused check origin/dependencies differ')
        verified = verify_check(previous, name, prior)
        for field in ('status', 'complete', 'expected_frames', 'completed_frames', 'events', 'original_sha256', 'candidate_sha256'):
            if field in verified['report'] and report.get(field) != verified['report'][field]:
                raise ValueError('Reused report does not match original evidence scope')
        artifacts.extend(verified['artifacts'])
        return {'report': report, 'artifacts': artifacts}
    if name in ('check', 'lint'):
        process_path = owner / 'logs' / (name + '.json')
        if not process_path.is_file():
            raise ValueError('Required successful process report missing')
        process = json.loads(process_path.read_text())
        if process.get('command') != ['just', name] or process not in manifest.get('commands', []):
            raise ValueError('Successful process invocation differs from verified check settings')
        if process != report or process.get('status') != 'pass' or process.get('complete') is not True or process.get('returncode') != 0:
            raise ValueError('Successful process report differs')
        logs = manifest.get('process_evidence', {}).get(name)
        if not logs:
            raise ValueError('Successful process logs are not digest-bound')
        for field in ('stdout', 'stderr'):
            path = safe_artifact(owner, process[field])
            if digest(path) != logs[field]:
                raise ValueError('Process output log checksum differs')
            artifacts.append(path)
        artifacts.append(process_path)
    elif name == 'fallback-only-parity':
        evidence = manifest['identities']['fallback-only-parity-report']
        raw_path = safe_artifact(owner, report['evidence'])
        if digest(raw_path) != evidence['sha256']:
            raise ValueError('Fallback parity report checksum differs')
        raw = json.loads(raw_path.read_text())
        required = {'mode': 'fallback-only', 'expected_frames': 18000, 'completed_frames': 18000,
                    'matched_frames': 18000, 'available_frames': manifest['identities']['inputs']['frames'],
                    'generated_steps': 0, 'seed': 0, 'controller': 'P1', 'profile': 'lsnes-profile',
                    'regions': ['CPU+clocks', 'WRAM', 'framebuffer'], 'status': 'pass', 'complete': True}
        if any(raw.get(key) != value for key, value in required.items()) or raw.get('interpreted_steps', 0) <= 0:
            raise ValueError('Fallback evidence does not complete exact cold parity scope')
        binary_id = manifest['identities']['fallback-only-binary']
        binary = safe_artifact(owner, binary_id['evidence'])
        if digest(binary) != binary_id['sha256'] or raw['identities']['binary_sha256'] != binary_id['sha256']:
            raise ValueError('Fallback executable identity differs')
        inputs = manifest['identities']['inputs']
        canonical = safe_artifact(owner, inputs['input_evidence'])
        if digest(canonical) != inputs['input_sha256']:
            raise ValueError('Fallback canonical input differs')
        import hashlib
        selected = b''.join(canonical.read_bytes().splitlines(keepends=True)[:18000])
        expected = {'rom_sha256': inputs['rom_sha256'], 'movie_sha256': inputs['movie_sha256'],
                    'input_sha256': inputs['input_sha256'], 'selected_input_sha256': hashlib.sha256(selected).hexdigest()}
        if any(raw['identities'].get(key) != value for key, value in expected.items()):
            raise ValueError('Fallback replay input identity differs')
        process_path = owner / 'logs/fallback-only-replay.json'
        process = json.loads(process_path.read_text())
        if process['status'] != 'pass' or process['complete'] is not True or process['returncode'] != 0:
            raise ValueError('Fallback process did not complete')
        command = process['command']
        if process not in manifest.get('commands', []):
            raise ValueError('Fallback process report lacks recorded invocation origin')
        original_owner = Path(manifest['parity_build_inputs']['fallback-only']['target_directory']).parent.parent
        original_binary = original_owner / binary_id['evidence']
        if ('--fallback-only' not in command or '--frames' not in command or command[command.index('--frames') + 1] != '18000'
                or command[0] != str(original_binary)):
            raise ValueError('Fallback process mode/scope/binary differs')
        for flag, expected in (('--rom', str(Path(process['cwd'])/'rom/sm.sfc')),
                               ('--out', str(original_owner/'diagnostics/fallback-only-frame-parity'))):
            if flag not in command or command[command.index(flag)+1] != expected:
                raise ValueError('Fallback invocation does not bind expected ROM/output')
        if command[1] != str(Path(process['cwd'])/'movies/super-metroid-any-3653.lsmv'):
            raise ValueError('Fallback invocation does not bind expected movie')
        for key in required:
            if report.get(key) != raw.get(key):
                raise ValueError('Fallback summary differs from underlying cold evidence')
        for item in ('game', 'steps'):
            identity = manifest['identities']['fallback-only-generated-' + item]
            path = safe_artifact(owner, identity['evidence'])
            if digest(path) != identity['sha256']:
                raise ValueError('Fallback empty compilation artifact differs')
            if item == 'steps' and path.read_text() != 'pub static STEP_TABLE: &[runtime::StepEntry] = &[];\npub const ROM_SHA256: Option<&str> = None;\n':
                raise ValueError('Fallback compilation contains generated entries')
            artifacts.append(path)
        artifacts.extend((raw_path, binary, canonical, process_path))
    elif name == 'synchronization':
        from readiness_sync import compare_captures
        original = owner / 'generated/original.csv'
        if not original.exists():
            record = next(item for item in manifest['reused_evidence'] if item['kind'] == 'original-oracle')
            original_owner = Path(record['path'])
            if read_verdict(original_owner).get('status') != 'pass':
                raise ValueError('Original oracle lacks successful completed origin')
            original_manifest = json.loads((original_owner/'manifest.json').read_text())
            if original_manifest['run_id'] != record['run_id']:
                raise ValueError('Original oracle run identity differs')
            original = original_owner / 'generated/original.csv'
        candidate = owner / 'generated/candidate.csv'
        compared = compare_captures(original, candidate, manifest['identities']['inputs'])
        if (compared['status'] != 'pass' or compared['complete'] is not True or report.get('expected_frames') != 18000
                or report.get('completed_frames') != 18000 or report.get('original_sha256') != digest(original)
                or report.get('candidate_sha256') != digest(candidate) or report.get('events') != compared['events']):
            raise ValueError('Independent synchronization evidence differs or incomplete')
        artifacts.extend((original, original.with_suffix('.capture.json'), candidate, candidate.with_suffix('.capture.json')))
        for capture in (original, candidate):
            metadata = json.loads(capture.with_suffix('.capture.json').read_text())
            capture_owner = capture.parent.parent
            def linked(value):
                value = Path(value)
                return value if value.is_absolute() else capture_owner / value
            artifacts.append(linked(metadata['observer']['artifact']))
            for key, value in metadata['provenance']['artifacts'].items():
                if key == 'source_root':
                    continue
                artifacts.append(linked(value))
            for entry in json.loads(linked(metadata['provenance']['artifacts']['source_inventory']).read_text())['files']:
                artifacts.append(linked(metadata['provenance']['artifacts']['source_root']) / entry['path'])
            for entry in metadata['provenance'].get('build_dependencies', []) + metadata['provenance'].get('packages', []):
                artifacts.append(linked(entry['path']))
            for entry in metadata.get('command_files', []):
                artifacts.append(linked(entry['artifact']))
            artifacts.append(linked(metadata['capture_report']))
    return {'report': report, 'artifacts': artifacts}


def safe_artifact(owner, relative):
    relative = Path(relative)
    if relative.is_absolute() or '..' in relative.parts:
        raise ValueError('Unsafe evidence artifact path')
    return Path(owner) / relative


def verify_accepted(owner, require_pointer=False):
    """Verify a complete accepted origin, independently of caller-chosen mandatory lists."""
    owner = Path(owner)
    verdict = read_verdict(owner)
    manifest = json.loads((owner / 'manifest.json').read_text())
    if verdict['status'] != 'pass' or verdict.get('acceptance_eligible') is not True or manifest['type'] not in ('readiness', 'iteration'):
        raise ValueError('Origin is not a complete accepted readiness run')
    if any(report.get('status') != 'pass' or report.get('complete') is not True for report in verdict.get('checks', {}).values()):
        raise ValueError('Accepted origin includes failed/incomplete check outcomes')
    if set(verdict.get('mandatory_checks', [])) != set(MANDATORY_CHECKS):
        raise ValueError('Accepted origin does not contain the complete mandatory registry')
    if any(manifest.get('scope', {}).get(name) != value for name, value in DOMAIN.items()):
        raise ValueError('Accepted origin domain/scope differs')
    inputs = manifest['identities']['inputs']
    if inputs.get('frames') != 129712 or inputs.get('seed') != 0 or inputs.get('profile') != 'lsnes-profile' or inputs.get('region') != 'NTSC':
        raise ValueError('Accepted origin input/profile scope differs')
    inventory_path = owner / 'source/inventory.json'
    inventory = json.loads(inventory_path.read_text())
    if inventory['sha256'] != inventory_digest(inventory['files']) or inventory['sha256'] != manifest['identities']['source']['sha256']:
        raise ValueError('Accepted source inventory checksum differs')
    guarded = [inventory_path, owner / 'manifest.json', owner / 'verdict.json']
    for entry in inventory['files']:
        path = safe_artifact(owner / 'source/files', entry['path'])
        if digest(path) != entry['sha256'] or (path.stat().st_mode & 0o777) != entry['mode']:
            raise ValueError('Accepted source snapshot checksum/mode differs')
        guarded.append(path)
    for name in MANDATORY_CHECKS:
        check = verdict.get('checks', {}).get(name, {})
        if check.get('status') != 'pass' or check.get('complete') is not True:
            raise ValueError('Accepted mandatory check failed or incomplete: ' + name)
        validate_gate_scope(name, check)
        evidence = manifest.get('check_evidence', {}).get(name)
        if not evidence:
            raise ValueError('Accepted mandatory check lacks digest-bound evidence: ' + name)
        path = safe_artifact(owner, evidence['path'])
        if digest(path) != evidence['sha256']:
            raise ValueError('Accepted check evidence checksum differs: ' + name)
        if json.loads(path.read_text()) != check:
            raise ValueError('Accepted check report differs from terminal evidence: ' + name)
        guarded.append(path)
    guarded.extend(verify_owned_evidence(owner, manifest))
    verified = {'manifest': manifest, 'verdict': verdict, 'inventory': inventory, 'guarded': guarded}
    if require_pointer:
        pointer = verify_accepted_pointer(owner, verified)
        verified['guarded'].append(pointer)
    return verified


def seal_gate_evidence(run):
    """Bind every mandatory report before terminal acceptance is considered."""
    run.manifest['contract_version'] = CONTRACT_VERSION
    run.manifest['check_evidence'] = {}
    run.manifest['process_evidence'] = {}
    for name in MANDATORY_CHECKS:
        if name not in run.checks:
            raise ValueError('Mandatory report missing: ' + name)
        path = run.path / 'checks' / (name + '.json')
        if json.loads(path.read_text()) != run.checks[name]:
            raise ValueError('Mandatory report differs from live outcome: ' + name)
        if name in ('check', 'lint') and not run.checks[name].get('reused_from'):
            process = run.checks[name]
            run.manifest['process_evidence'][name] = {field: digest(safe_artifact(run.path, process[field])) for field in ('stdout', 'stderr')}
        run.manifest['check_evidence'][name] = {'path': 'checks/' + name + '.json', 'sha256': digest(path)}
    seal_owned_evidence(run)
    run.save_manifest()


def apply_reuse(run, predecessor, name):
    """Reverify before importing one check, then guard every linked artifact to run end."""
    if name not in REUSABLE_CHECKS:
        raise ValueError('Candidate-specific checks cannot reuse prior evidence')
    plan = plan_reuse(run, predecessor)
    decision = plan[name]
    if decision['action'] != 'reuse':
        raise ValueError(decision['reason'])
    accepted = verify_accepted(predecessor, require_pointer=True)
    verified = verify_check(predecessor, name, accepted)
    import copy
    report = copy.deepcopy(verified['report'])
    report['reused_from'] = {'path': str(Path(predecessor).resolve()), 'run_id': accepted['manifest']['run_id'],
                             'check': name, 'dependency_sha256': decision['dependency_sha256']}
    for path in accepted['guarded'] + [Path(predecessor) / 'checks/dependency-policy.json'] + [Path(item) for item in verified['artifacts']]:
        run.guard_input(path)
    run.manifest['reused_evidence'].append({'kind': 'accepted-check', 'check': name,
                                          'run_id': accepted['manifest']['run_id'], 'path': str(Path(predecessor).resolve()),
                                          'dependency_sha256': decision['dependency_sha256'],
                                          'verified_dependencies': verify_policy(predecessor, accepted)['checks'][name],
                                          'artifacts': [{'path': str(path), 'sha256': digest(path)} for path in verified['artifacts']]})
    run.save_manifest()
    run.check_sources()
    run.record_check(name, report)
    return report


def accepted_pointer(owner, accepted):
    owner = Path(owner).resolve()
    manifest = accepted['manifest']
    return {'schema_version': 1, 'contract_version': CONTRACT_VERSION, 'run_id': manifest['run_id'],
            'run_path': str(owner), 'domain': DOMAIN,
            'source_inventory': 'source/inventory.json', 'source_sha256': manifest['identities']['source']['sha256'],
            'source_path': 'source/files', 'verdict_sha256': digest(owner / 'verdict.json'),
            'accepted_parent': manifest.get('accepted_parent'), 'accepted_utc': manifest['finished_utc']}


def verify_accepted_pointer(owner, accepted):
    owner = Path(owner).resolve()
    # Accepted runs live below out/runs; current and retained history are the
    # explicit lineage. A standalone passing run cannot impersonate acceptance.
    output = owner.parent.parent
    expected = accepted_pointer(owner, accepted)
    candidates = [output / 'accepted.json'] + sorted((output / 'accepted-history').glob('*.json'))
    for path in candidates:
        try:
            pointer = json.loads(path.read_text())
            if all(pointer.get(key) == value for key, value in expected.items() if key != 'accepted_utc') and pointer.get('accepted_utc'):
                return path
        except (OSError, ValueError):
            continue
    raise ValueError('Accepted run has no verified current/history pointer lineage')


def cache_reuse_decision(cache_path, identities):
    """Require exact complete candidate identities before invoking the paired API.

    This precheck never restores/relabels state or replaces prepare_cache's actual
    deserialization and cold-validation checks. Changed candidates rebuild cold.
    """
    cache_path = Path(cache_path)
    try:
        envelope = json.loads((cache_path / 'cache.json').read_text())
        pair = json.loads((cache_path / 'pair.json').read_text())
        validation = json.loads((cache_path / 'validation.json').read_text())
        if digest(cache_path / 'pair.json') != envelope['pair_file_sha256'] or digest(cache_path / 'validation.json') != envelope['validation_file_sha256']:
            raise ValueError('Checkpoint artifact checksum differs')
        if pair['identities'] != identities:
            raise ValueError('Candidate checkpoint identities differ; equivalent history not proven')
        if (validation.get('status') != 'pass' or validation.get('complete') is not True
                or validation.get('completed_frames') != 1 or validation.get('observed_rollovers') != 2
                or validation.get('pair_sha256') != pair['pair_sha256']):
            raise ValueError('Checkpoint lacks complete cold trace/state validation')
        return {'action': 'validate-with-paired-api', 'reason': 'Exact candidate identities; restore still requires paired API verification',
                'pair_sha256': pair['pair_sha256'], 'history_sha256': pair['history_sha256']}
    except (OSError, ValueError, KeyError, TypeError) as error:
        return {'action': 'rebuild-cold', 'reason': str(error)}


def owned_evidence_files(owner):
    owner=Path(owner)
    for directory in ('source','generated','checks','logs','diagnostics'):
        for path in sorted((owner/directory).rglob('*')):
            relative=path.relative_to(owner)
            if relative.parts[:2] in (('generated','original-build'),('generated','reference-target')) or any(part in ('target','build','__pycache__') for part in relative.parts):
                continue
            if path.is_symlink():
                raise ValueError('Owned evidence contains unsupported symlink: '+str(relative))
            if path.is_file() and path.name != 'evidence-inventory.json':
                yield path
    if (owner/'coverage.json').is_file():
        yield owner/'coverage.json'


def seal_owned_evidence(run):
    files=[{'path':path.relative_to(run.path).as_posix(),'sha256':digest(path),'mode':path.stat().st_mode&0o777}
           for path in owned_evidence_files(run.path)]
    path=run.path/'checks/evidence-inventory.json'
    write_json(path,{'schema_version':1,'sha256':inventory_digest(files),'files':files})
    run.manifest['evidence_inventory']={'path':'checks/evidence-inventory.json','sha256':digest(path)}


def verify_owned_evidence(owner,manifest):
    identity=manifest.get('evidence_inventory')
    if not identity:
        raise ValueError('Accepted origin lacks sealed owned evidence inventory')
    path=safe_artifact(owner,identity['path'])
    if digest(path)!=identity['sha256']:
        raise ValueError('Owned evidence inventory checksum differs')
    record=json.loads(path.read_text())
    if record.get('schema_version')!=1 or record.get('sha256')!=inventory_digest(record['files']):
        raise ValueError('Owned evidence inventory invalid')
    files=list(owned_evidence_files(owner))
    if [str(item.relative_to(owner)) for item in files]!=[item['path'] for item in record['files']]:
        raise ValueError('Owned evidence file set changed')
    for entry in record['files']:
        artifact=safe_artifact(owner,entry['path'])
        if digest(artifact)!=entry['sha256'] or artifact.stat().st_mode&0o777!=entry['mode']:
            raise ValueError('Owned evidence artifact checksum/mode differs: '+entry['path'])
    return [path]+files


def main(argv=None):
    import argparse
    from readiness import Run,preflight
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--predecessor',type=Path)
    args=parser.parse_args(argv)
    run=Run.create(args.root,'reuse-plan',DOMAIN,args.predecessor.name if args.predecessor else None)
    print('Run: '+str(run.path),flush=True)
    print('Brief: '+str(run.path/'brief.md'),flush=True)
    try:
        failures=preflight(run)
        if failures:
            return run.finalize('error','preflight','; '.join(failures))
        record_dependency_policy(run)
        predecessor=args.predecessor
        if predecessor is None and (run.root/'out/accepted.json').is_file():
            predecessor=Path(json.loads((run.root/'out/accepted.json').read_text())['run_path'])
        decisions=plan_reuse(run,predecessor)
        run.record_check('reuse-plan',{'status':'pass','complete':True,'decisions':decisions})
        return run.finalize('pass','reuse-plan','Reuse decisions recorded; candidate validation remains pending',['preflight','reuse-plan'])
    except Exception as error:
        return run.finalize('error','reuse-plan',str(error))


if __name__=='__main__':
    import sys
    sys.exit(main())
