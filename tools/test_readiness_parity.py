"""Parity command evidence at the user-approved command and run seams."""
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class ParityCommandTests(unittest.TestCase):
    def test_setup_failure_has_a_new_terminal_run_and_no_binary_identity(self):
        root = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as directory:
            checkout = Path(directory)
            subprocess.run(["git", "init", "-q", str(checkout)], check=True)
            result = subprocess.run([sys.executable, str(root / "tools/readiness_parity.py"),
                                     "--root", str(checkout), "--mode", "fallback-only", "--frames", "18000"],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            runs = list((checkout / "out/runs").iterdir())
            self.assertEqual(len(runs), 1)
            verdict = json.loads((runs[0] / "verdict.json").read_text())
            self.assertEqual(verdict["status"], "error")
            self.assertFalse(verdict["acceptance_eligible"])
            manifest = json.loads((runs[0] / "manifest.json").read_text())
            self.assertNotIn("fallback-only-binary", manifest["identities"])
            self.assertEqual(manifest["scope"]["frames"], 18000)

    @unittest.skipUnless(os.environ.get("SM_READINESS_INTEGRATION_TESTS") == "1", "explicit real replay integration check")
    def test_real_fallback_prefix_has_isolated_binary_and_exact_scope(self):
        root = Path(__file__).resolve().parent.parent
        result = subprocess.run([sys.executable, str(root / "tools/readiness_parity.py"),
                                 "--root", str(root), "--mode", "fallback-only", "--frames", "18000"],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        run = Path(next(line[5:] for line in result.stdout.splitlines() if line.startswith("Run: ")))
        verdict = json.loads((run / "verdict.json").read_text())
        self.assertEqual(verdict["checks"]["fallback-only-parity"]["completed_frames"], 18000)
        self.assertFalse(verdict["acceptance_eligible"])
        manifest = json.loads((run / "manifest.json").read_text())
        self.assertTrue((run / manifest["identities"]["fallback-only-binary"]["evidence"]).is_file())
        self.assertEqual(manifest["identities"]["inputs"]["frames"], 129712)

    def test_stage_maps_validation_and_infrastructure_exit_codes(self):
        from readiness import Run
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            run = Run.create(root, "parity", {"frames": 18000})
            run.capture_source()
            failed = run.run_stage("validation", [sys.executable, "-c", "raise SystemExit(1)"],
                                   exit_statuses={1: "divergence", 2: "error"})
            broken = run.run_stage("infrastructure", [sys.executable, "-c", "raise SystemExit(2)"],
                                   exit_statuses={1: "divergence", 2: "error"})
            self.assertEqual(failed["status"], "divergence")
            self.assertEqual(broken["status"], "error")
            self.assertEqual(broken["returncode"], 2)

    @unittest.skipUnless(os.environ.get("SM_READINESS_INTEGRATION_TESTS") == "1", "explicit real replay integration check")
    def test_real_generated_domain_executes_the_fresh_timed_table(self):
        root = Path(__file__).resolve().parent.parent
        result = subprocess.run([sys.executable, str(root / "tools/readiness_parity.py"),
                                 "--root", str(root), "--mode", "generated", "--frames", "18000"],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        run = Path(next(line[5:] for line in result.stdout.splitlines() if line.startswith("Run: ")))
        verdict = json.loads((run / "verdict.json").read_text())
        parity = verdict["checks"]["generated-parity"]
        self.assertEqual(parity["completed_frames"], 18000)
        self.assertGreater(parity["generated_steps"], 0)
        self.assertGreater(parity["compiled_table_entries"], 0)
        self.assertFalse(verdict["acceptance_eligible"])
