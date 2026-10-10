"""Automatic diagnosis and controls through the public evidence interfaces."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ControlsTests(unittest.TestCase):
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
                               {"status": "pass", "complete": True})
        self.assertEqual(result["route"], "increment")
        self.assertNotIn("root_cause", result)
        self.assertIn("supports", result["conclusion"])

    def test_sync_failure_requests_independent_observations(self):
        from readiness_controls import route_failure
        result = route_failure({"status": "divergence", "stage": "synchronization"},
                               {"status": "pass", "complete": True}, {"status": "pass", "complete": True})
        self.assertEqual(result["route"], "independent hardware/input observation")
