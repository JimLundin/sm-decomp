"""Verified evidence reuse at the approved readiness command/evidence seam."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from readiness import Run
from readiness_reuse import record_dependency_policy, plan_reuse, verify_accepted, accepted_pointer, seal_gate_evidence
from readiness_contract import MANDATORY_CHECKS


def gate(name):
    report = {'status':'pass','complete':True}
    if name in ('fallback-only-parity','generated-parity','synchronization'):
        report.update(expected_frames=18000,completed_frames=18000,matched_frames=18000,generated_steps=0 if name=='fallback-only-parity' else 1,compiled_table_entries=1)
    if name=='instruction': report.update(expected=70011,run=70011,failed=0,skipped=0)
    if name=='checkpoint-validation': report.update(completed_frames=1,observed_rollovers=2)
    if name=='instruction-diagnosis': report.update(expected_instructions=8,completed_instructions=8)

    if name=='instruction':
        report.update(passed=70011,opcodes={opcode:{'expected':10000+(11 if opcode=='a9' else 0),'run':10000+(11 if opcode=='a9' else 0),'passed':10000+(11 if opcode=='a9' else 0),'failed':0,'skipped':0} for opcode in ('a9','a2','c2','e2','8d','ea','ca')})
    if name=='timing':
        report.update(returncode=0,oracles={name:'a'*64 for name in ('crates/snes/tests/hdma_oracle.csv','crates/snes/tests/irq_mask_timing.rs','crates/snes/tests/interrupt_timing.rs','crates/snes/tests/dma_timing.rs')},sources=['independent'],conditions={'applicable':['bus']})
    if name=='synchronization': report.update(checkpoint_pc='828948',start_frame=272,end_frame=18000,candidate_frame_offset=0,events=15184,candidate_completed_frames=18000)
    if name=='changed-entry-coverage': report.update(generated_instructions=1,uncovered_keys=[],executed=[{'address':0x8000,'mx':3,'count':1}],validated_distinct_keys=1,required_keys=[],candidate_inventory_sha256='a'*64,profile_sha256='b'*64)
    if name=='checkpoint-validation': report.update(reference={'observed_rollovers':2,'instructions':1,'ordered_trace_sha256':'a'*64,'ending_sha256':'b'*64},subject={'observed_rollovers':2,'instructions':1,'ordered_trace_sha256':'a'*64,'ending_sha256':'b'*64})
    if name=='feedback-demonstrations':
        cases=('correct-execution','wrong-instruction','transient-effect','uncovered-key','empty-compilation','preflight-build-failure','interrupted-truncated','incompatible-stale','paired-cache')
        report.update(cases={case:{'status':'pass','complete':True} for case in cases},expected_cases=len(cases),completed_cases=len(cases))
    return report


class ReuseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        for name in ('Cargo.toml', 'Cargo.lock', 'crates/recomp/src/step.rs',
                     'crates/runtime/src/bus.rs', 'config/functions.toml', 'docs/guide.md', 'justfile'):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(name)
        (self.root/'justfile').write_text('check:\n    true\n\nlint:\n    true\n')
        subprocess.run(['git', 'add', '.'], cwd=self.root, check=True)

    def candidate(self):
        run = Run.create(self.root, 'readiness', {'domain': 'Japan/USA any%', 'frames': 18000})
        run.capture_source()
        run.manifest['identities'].update(inputs={'rom_sha256': '1'*64, 'movie_sha256': '2'*64,
            'input_sha256': '3'*64, 'frames': 129712, 'seed': 0, 'reset': 'clean power-on',
            'controller': 'P1 SNES gamepad', 'region': 'NTSC', 'profile': 'lsnes-profile'},
            hardware={'source_sha256': '4'*64, 'features': ['lsnes-profile']},
            compiler={'version': 'fixture compiler'})
        run.manifest['build_settings'] = {'profile': 'release', 'features': ['lsnes-profile']}
        run.save_manifest()
        record_dependency_policy(run)
        return run

    def test_cpu_only_policy_retains_shared_dependencies_and_requires_candidate_gates(self):
        old = self.candidate()
        (self.root / 'config/functions.toml').write_text('new compiled key')
        current = self.candidate()
        old_policy = json.loads((old.path / 'checks/dependency-policy.json').read_text())
        new_policy = json.loads((current.path / 'checks/dependency-policy.json').read_text())
        self.assertEqual(old_policy['checks']['synchronization'], new_policy['checks']['synchronization'])
        self.assertEqual(old_policy['checks']['fallback-only-parity'], new_policy['checks']['fallback-only-parity'])
        self.assertNotEqual(old_policy['checks']['check'], new_policy['checks']['check'])
        plan = plan_reuse(current, old.path)
        self.assertEqual(plan['generated-parity']['action'], 'fresh')
        self.assertEqual(plan['changed-entry-coverage']['action'], 'fresh')
        self.assertEqual(plan['instruction-diagnosis']['action'], 'fresh')
        self.assertEqual(plan['synchronization']['action'], 'fresh')
        self.assertIn('accepted', plan['synchronization']['reason'])

    def test_self_described_pass_with_one_gate_cannot_be_accepted_origin(self):
        run = self.candidate()
        run.record_check('preflight', {'status': 'pass', 'complete': True})
        run.finalize('pass', mandatory_checks=['preflight'])
        with self.assertRaisesRegex(ValueError, 'mandatory'):
            verify_accepted(run.path)

    def accepted(self):
        run = self.candidate()
        for name in MANDATORY_CHECKS:
            if name in ('check', 'lint'):
                run.run_stage(name, ['just',name])
            else:
                run.record_check(name, gate(name))
        run.manifest['check_evidence'] = {name: {'path': 'checks/' + name + '.json',
                                               'sha256': __import__('readiness').digest(run.path / 'checks' / (name + '.json'))}
                                           for name in MANDATORY_CHECKS}
        seal_gate_evidence(run)
        run.finalize('pass', mandatory_checks=MANDATORY_CHECKS)
        (self.root / 'out/accepted.json').write_text(json.dumps(accepted_pointer(run.path, verify_accepted(run.path))))
        return run

    def test_complete_check_files_bind_accepted_origin_and_detect_mutation(self):
        run = self.accepted()
        self.assertEqual(verify_accepted(run.path)['manifest']['run_id'], run.manifest['run_id'])
        (run.path / 'checks/check.json').write_text('{"status":"pass","complete":true,"forged":true}')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            verify_accepted(run.path)

    def test_documentation_retains_check_reuse_but_shared_source_requires_fresh(self):
        old = self.accepted()
        (self.root / 'docs/guide.md').write_text('clarified instructions')
        current = self.candidate()
        plan = plan_reuse(current, old.path)
        self.assertEqual(plan['check']['action'], 'reuse')
        self.assertEqual(plan['lint']['action'], 'reuse')
        (self.root / 'crates/runtime/src/bus.rs').write_text('changed timing')
        altered = self.candidate()
        plan = plan_reuse(altered, old.path)
        self.assertEqual(plan['fallback-only-parity']['action'], 'fresh')
        self.assertEqual(plan['synchronization']['action'], 'fresh')
        self.assertIn('dependencies', plan['synchronization']['reason'])

    def test_reused_check_requires_a_real_successful_process_report(self):
        old = self.accepted()
        current = self.candidate()
        (old.path / 'logs/check.json').unlink()
        plan = plan_reuse(current, old.path)
        self.assertEqual(plan['check']['action'], 'fresh')
        self.assertIn('evidence', plan['check']['reason'])

    def test_cpu_only_accepted_fallback_reuse_imports_exact_scope_and_guards_evidence(self):
        from readiness import digest, write_json
        from readiness_reuse import apply_reuse
        import hashlib
        run = self.candidate()
        inputs = run.path / 'generated/movie.inputs'
        inputs.write_text('0000\n' * 129712)
        run.manifest['identities']['inputs'].update(input_sha256=digest(inputs), input_evidence='generated/movie.inputs')
        generated = run.path / 'generated/fallback-only'
        generated.mkdir()
        binary = generated / 'diff'
        binary.write_bytes(b'compiled fallback binary')
        run.record_artifact('fallback-only-binary', binary)
        run.manifest['identities']['fallback-only-binary']['evidence'] = 'generated/fallback-only/diff'
        for name, contents in (('game', 'pub static TABLE: &[runtime::TableEntry] = &[];\n'),
                               ('steps', 'pub static STEP_TABLE: &[runtime::StepEntry] = &[];\npub const ROM_SHA256: Option<&str> = None;\n')):
            artifact = generated / (name + '.rs')
            artifact.write_text(contents)
            run.record_artifact('fallback-only-generated-' + name, artifact)
            run.manifest['identities']['fallback-only-generated-' + name]['evidence'] = str(artifact.relative_to(run.path))
        raw = {'schema_version': 1, 'mode': 'fallback-only', 'expected_frames': 18000,
               'completed_frames': 18000, 'matched_frames': 18000, 'available_frames': 129712,
               'generated_steps': 0, 'interpreted_steps': 100, 'seed': 0, 'controller': 'P1',
               'profile': 'lsnes-profile', 'regions': ['CPU+clocks', 'WRAM', 'framebuffer'],
               'status': 'pass', 'complete': True, 'identities': {'binary_sha256': digest(binary),
               'rom_sha256': '1'*64, 'movie_sha256': '2'*64, 'input_sha256': digest(inputs),
               'selected_input_sha256': hashlib.sha256(('0000\n'*18000).encode()).hexdigest()}}
        raw_path = run.path / 'diagnostics/parity.json'
        write_json(raw_path, raw)
        run.record_artifact('fallback-only-parity-report', raw_path)
        process = {'status': 'pass', 'complete': True, 'returncode': 0, 'cwd':str(self.root), 'command': [str(binary),str(self.root/'movies/super-metroid-any-3653.lsmv'), '--fallback-only', '--frames', '18000','--rom',str(self.root/'rom/sm.sfc'),'--out',str(run.path/'diagnostics/fallback-only-frame-parity')]}
        run.manifest['parity_build_inputs']={'fallback-only':{'target_directory':str(run.path/'build/fallback-only')}}
        write_json(run.path / 'logs/fallback-only-replay.json', process)
        run.manifest['commands'].append(process)
        # Replace the policy before it becomes completed evidence, after inputs
        # were materialized; retained initial policy no longer defines this run.
        policy_path = run.path / 'checks/dependency-policy.json'
        policy = json.loads(policy_path.read_text())
        for check in policy['checks'].values():
            check['settings']['inputs'] = {key:value for key,value in run.manifest['identities']['inputs'].items() if key != 'input_evidence'}
        write_json(policy_path, policy)
        run.manifest['identities']['dependency-policy']['sha256'] = digest(policy_path)
        for name in MANDATORY_CHECKS:
            if name in ('check', 'lint'):
                run.run_stage(name, ['just',name])
            else:
                report = dict(raw, evidence='diagnostics/parity.json') if name == 'fallback-only-parity' else gate(name)
                run.record_check(name, report)
        run.manifest['check_evidence'] = {name: {'path':'checks/'+name+'.json','sha256':digest(run.path/'checks'/(name+'.json'))} for name in MANDATORY_CHECKS}
        seal_gate_evidence(run)
        run.finalize('pass', mandatory_checks=MANDATORY_CHECKS)
        (self.root / 'out/accepted.json').write_text(json.dumps(accepted_pointer(run.path, verify_accepted(run.path))))
        (self.root / 'crates/recomp/src/step.rs').write_text('changed immediate LDA body')
        current = Run.create(self.root, 'readiness', {'domain':'Japan/USA any%','frames':18000})
        current.capture_source()
        current.manifest['identities'].update({key:value for key,value in run.manifest['identities'].items() if key in ('inputs','hardware','compiler')})
        current.manifest['build_settings'] = run.manifest['build_settings']
        current.save_manifest()
        record_dependency_policy(current)
        plan = plan_reuse(current, run.path)
        self.assertEqual(plan['fallback-only-parity']['action'], 'reuse')
        report = apply_reuse(current, run.path, 'fallback-only-parity')
        self.assertEqual(report['completed_frames'], 18000)
        self.assertEqual(current.manifest['reused_evidence'][0]['run_id'], run.manifest['run_id'])
        binary.write_bytes(b'changed binary')
        with self.assertRaisesRegex(RuntimeError, 'Input changed'):
            current.check_sources()

    def test_unknown_markdown_build_input_forces_fresh_checks(self):
        old = self.accepted()
        path = self.root / 'crates/runtime/src/embedded.md'
        path.write_text('new executable embedded resource')
        subprocess.run(['git','add',str(path)],cwd=self.root,check=True)
        current = self.candidate()
        plan = plan_reuse(current, old.path)
        self.assertEqual(plan['check']['action'],'fresh')
        self.assertEqual(plan['fallback-only-parity']['action'],'fresh')

    def test_completed_scope_without_accepted_pointer_cannot_reuse(self):
        old = self.accepted()
        (self.root / 'out/accepted.json').unlink()
        current = self.candidate()
        plan = plan_reuse(current, old.path)
        self.assertEqual(plan['check']['action'],'fresh')
        self.assertIn('lineage',plan['check']['reason'])

    def test_checkpoint_reuse_never_relabels_changed_candidate_history(self):
        from readiness_reuse import cache_reuse_decision
        from readiness import digest,write_json
        cache=self.root/'cache'
        cache.mkdir()
        identities={'source_sha256':'1'*64,'hardware_sha256':'2'*64,'inputs':'same'}
        pair={'identities':identities,'pair_sha256':'3'*64,'history_sha256':'4'*64}
        write_json(cache/'pair.json',pair)
        write_json(cache/'validation.json',{'status':'pass','complete':True,'completed_frames':1,
                                          'observed_rollovers':2,'pair_sha256':'3'*64})
        write_json(cache/'cache.json',{'pair_file_sha256':digest(cache/'pair.json'),
                                     'validation_file_sha256':digest(cache/'validation.json')})
        self.assertEqual(cache_reuse_decision(cache,identities)['action'],'validate-with-paired-api')
        changed=dict(identities,source_sha256='5'*64)
        self.assertEqual(cache_reuse_decision(cache,changed)['action'],'rebuild-cold')
        (cache/'pair.json').write_text('corrupt')
        self.assertEqual(cache_reuse_decision(cache,identities)['action'],'rebuild-cold')

    def test_document_referenced_by_build_is_an_executable_dependency(self):
        (self.root/'Cargo.toml').write_text('include documentation docs/guide.md')
        old=self.accepted()
        (self.root/'docs/guide.md').write_text('changed embedded data')
        current=self.candidate()
        plan=plan_reuse(current,old.path)
        self.assertEqual(plan['check']['action'],'fresh')
        self.assertEqual(plan['synchronization']['action'],'fresh')

    def test_rehashed_policy_cannot_remove_a_shared_dependency(self):
        from readiness import digest,write_json
        old=self.accepted()
        policy_path=old.path/'checks/dependency-policy.json'
        policy=json.loads(policy_path.read_text())
        policy['checks']['synchronization']['files']=[]
        write_json(policy_path,policy)
        manifest_path=old.path/'manifest.json'
        manifest=json.loads(manifest_path.read_text())
        manifest['identities']['dependency-policy']['sha256']=digest(policy_path)
        write_json(manifest_path,manifest)
        current=self.candidate()
        plan=plan_reuse(current,old.path)
        self.assertEqual(plan['synchronization']['action'],'fresh')
        self.assertIn('checksum',plan['synchronization']['reason'])


if __name__ == '__main__':
    unittest.main()
