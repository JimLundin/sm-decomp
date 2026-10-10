"""Automatic diagnosis and controls through the public evidence interfaces."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ControlsTests(unittest.TestCase):
    def test_predecessor_material_linking_preserves_restored_rom_directory(self):
        from readiness_controls import link_control_material
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)/'root';checkout=Path(directory)/'checkout'
            (root/'rom').mkdir(parents=True);(checkout/'rom').mkdir(parents=True)
            (root/'rom/sm.sfc').write_bytes(b'private fixture ROM')
            (checkout/'rom/README.md').write_text('restored tracked documentation')
            link_control_material(checkout,root)
            self.assertEqual((checkout/'rom/README.md').read_text(),'restored tracked documentation')
            self.assertTrue((checkout/'rom/sm.sfc').is_symlink())
            self.assertEqual((checkout/'rom/sm.sfc').read_bytes(),b'private fixture ROM')

    def test_discovery_keeps_process_provenance_separate_from_result(self):
        from readiness import Run
        from readiness_diagnostics import discover_intervals
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);subprocess.run(['git','init','-q',str(root)],check=True)
            run=Run.create(root,'diagnostic',{'frames':18000});run.capture_source()
            inputs={'rom_sha256':'a'*64,'movie_sha256':'b'*64,'input_sha256':'c'*64}
            run.manifest['identities']['inputs']=inputs
            generated=run.path/'generated/generated';generated.mkdir()
            for name in ('game','steps'):
                path=generated/(name+'.rs');path.write_text('// preserved fixture\n')
                run.record_artifact('generated-generated-'+name,path)
            candidate=generated/'diff';candidate.write_bytes(b'prepared parity executable')
            run.record_artifact('generated-binary',candidate)
            binary=run.path/'build/generated/release/focused-replay';binary.parent.mkdir(parents=True)
            # A real subprocess emits a deterministic scoped result. This test
            # checks evidence lifecycle; the feedback probe tests real hardware.
            binary.write_text('#!/usr/bin/env python3\nimport sys, json, hashlib\nfrom pathlib import Path\n'
                'output=Path(sys.argv[sys.argv.index("--out")+1]); output.mkdir()\n'
                'keys=json.loads(Path(sys.argv[sys.argv.index("--discover-keys")+1]).read_text())\n'
                'report={"schema_version":1,"status":"pass","complete":True,"expected_frames":18000,"completed_frames":0,"keys":keys,"intervals":[],"unexecuted":[],"identities":'+repr(inputs)+'}\n'
                'report["binary_sha256"]=hashlib.sha256(Path(sys.argv[0]).read_bytes()).hexdigest()\n'
                '(output/"discovery.json").write_text(json.dumps(report))\n')
            binary.chmod(0o755);run.record_artifact('focused-prepared-binary',binary,'generated/focused-prepared')
            result=discover_intervals(run,[{'address':0x8000,'mx':3}])
            self.assertEqual(result['status'],'pass')
            self.assertEqual(run.checks['instruction-discovery-process']['returncode'],0)
            self.assertEqual(run.checks['instruction-discovery'],result)
            self.assertNotIn('command',result)

    def test_three_nondiscriminating_experiments_preserve_candidate_and_request_decision(self):
        from readiness_controls import record_experiment
        from readiness import Run
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            run = Run.create(root, "diagnostic", {"frames": 18000}, "preserved-parent")
            for number in range(3):
                report = record_experiment(run, "Timing hypothesis", "An independent IRQ probe differs",
                                           {"status": "pass", "command": ["probe", str(number)]},
                                           discriminating=False, evidence="Identical outcome")
            self.assertTrue(report["decision_required"])
            self.assertEqual(report["candidate_run_id"], run.manifest["run_id"])
            self.assertEqual(report["predecessor_run_id"], "preserved-parent")
            self.assertGreaterEqual(len(report["choices"]), 2)
            self.assertEqual(len(list((run.path / "diagnostics/experiments").glob("*.json"))), 3)
            self.assertFalse((root / "out/accepted.json").exists())

    def test_prepare_candidate_never_claims_short_parity_validation(self):
        from readiness_parity import prepare_candidate
        from readiness import Run
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            run = Run.create(root, "diagnostic", {"instructions": 8})
            run.capture_source()
            result = prepare_candidate(run, "fallback-only")
            self.assertEqual(result["status"], "error")
            self.assertNotIn("fallback-only-parity", run.checks)
            self.assertNotIn("changed-entry-coverage", run.checks)

    def test_passing_controls_support_increment_attribution_without_root_cause(self):
        from readiness_controls import route_failure
        result = route_failure({"status": "divergence"},
                               {"status": "pass", "complete": True},
                               {"status": "pass", "complete": True},
                               candidate={"status":"divergence","complete":True,"first_mismatch":{"instruction":3}})
        self.assertEqual(result["route"], "increment")
        self.assertNotIn("root_cause", result)
        self.assertIn("supports", result["conclusion"])

    def test_passing_early_candidate_focus_does_not_attribute_later_cold_failure(self):
        from readiness_controls import route_failure
        passing={"status":"pass","complete":True}
        result=route_failure({"status":"divergence","stage":"generated-parity",
            "first_divergence":{"frame":7}},passing,passing,candidate=passing)
        self.assertEqual(result["route"],"failure not reproduced")
        self.assertEqual(result["next_operation"]["start_input"],7)
        self.assertFalse(result["new_discriminating_evidence"])

    def test_failed_occurrence_drives_cold_localization_and_incomplete_candidate_cannot_attribute(self):
        from readiness_controls import failure_intervals,route_failure
        failure={'status':'divergence','stage':'generated-parity','first_divergence':{'frame':9}}
        self.assertEqual(failure_intervals(failure),[{'start_input':9,'start_instruction':0,'instructions':2000000,'until_frame_end':True}])
        passing={'status':'pass','complete':True}
        result=route_failure(failure,passing,passing,candidate={'status':'error','complete':False})
        self.assertEqual(result['route'],'failure not reproduced')

    def test_cold_frame_command_report_requires_a_rollover_and_preserves_identity(self):
        from readiness import Run
        from readiness_diagnostics import diagnose
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);subprocess.run(['git','init','-q',str(root)],check=True)
            run=Run.create(root,'diagnostic',{'frames':1});run.capture_source()
            inputs={'rom_sha256':'a'*64,'movie_sha256':'b'*64,'input_sha256':'c'*64}
            run.manifest['identities']['inputs']=inputs
            generated=run.path/'generated/generated';generated.mkdir()
            for name in ('game','steps'):(generated/(name+'.rs')).write_text('// fixture')
            binary=run.path/'build/generated/release/focused-replay';binary.parent.mkdir(parents=True)
            binary.write_text('#!/usr/bin/env python3\nimport sys,json,hashlib\nfrom pathlib import Path\n'
                'assert "--until-frame-end" in sys.argv and "--cache" not in sys.argv\n'
                'output=Path(sys.argv[sys.argv.index("--out")+1]);output.mkdir()\n'
                'report={"schema_version":1,"status":"pass","complete":True,"scope":"until-frame-end","instruction_budget":2000000,"observed_rollovers":1,"expected_instructions":None,"completed_instructions":123,"start_input_index":9,"start_instruction":0,"replay":"cold","disabled_keys":[],"identities":'+repr(inputs)+'}\n'
                'report["binary_sha256"]=hashlib.sha256(Path(sys.argv[0]).read_bytes()).hexdigest()\n'
                '(output/"instruction.json").write_text(json.dumps(report))\n')
            binary.chmod(0o755);run.record_artifact('focused-prepared-binary',binary,'generated/focused-prepared')
            report=diagnose(run,start_input=9,until_frame_end=True)
            self.assertEqual(report['status'],'pass')
            self.assertEqual(report['observed_rollovers'],1)

    def test_sync_failure_requests_independent_observations(self):
        from readiness_controls import route_failure
        result = route_failure({"status": "divergence", "stage": "synchronization"},
                               {"status": "pass", "complete": True}, {"status": "pass", "complete": True})
        self.assertEqual(result["route"], "independent hardware/input observation")
