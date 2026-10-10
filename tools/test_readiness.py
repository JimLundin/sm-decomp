"""Behavior checks at the readiness command and immutable evidence seam."""

import json
import hashlib
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

from readiness import Run, read_verdict, verify_movie_inputs


class ReadinessEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        (self.root / "Cargo.toml").write_text("[workspace]\n")
        (self.root / "crates/demo/src").mkdir(parents=True)
        (self.root / "crates/demo/src/lib.rs").write_text("// candidate\n")

    def test_unique_runs_are_visibly_incomplete_until_finalized(self):
        first = Run.create(self.root, "preflight", {"frames": 18000})
        second = Run.create(self.root, "preflight", {"frames": 18000})
        self.assertNotEqual(first.path, second.path)
        self.assertEqual(read_verdict(first.path)["status"], "incomplete")
        self.assertFalse(read_verdict(first.path)["acceptance_eligible"])
        manifest = json.loads((first.path / "manifest.json").read_text())
        self.assertEqual(manifest["schema_version"], 1)
        self.assertEqual(manifest["state"], "running")
        self.assertEqual(first.finalize("pass", "preflight", "setup verified"), 0)
        terminal = (first.path / "verdict.json").read_bytes()
        self.assertFalse(read_verdict(first.path)["acceptance_eligible"])
        with self.assertRaises(RuntimeError):
            first.finalize("pass", "preflight", "overwrite")
        self.assertEqual((first.path / "verdict.json").read_bytes(), terminal)
        self.assertTrue((first.path / "brief.md").is_file())

    def test_snapshot_restores_dirty_and_untracked_source_without_private_material(self):
        subprocess.run(["git", "add", "Cargo.toml", "crates"], cwd=self.root, check=True)
        source = self.root / "crates/demo/src/lib.rs"
        source.write_text("// dirty candidate\n")
        (source.parent / "new.rs").write_text("// untracked maintained source\n")
        for relative in ("rom/sm.sfc", "out/generated/steps.rs", "crates/demo/.env",
                         "crates/demo/private.pem", "personal.txt"):
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("excluded")
        run = Run.create(self.root, "preflight", {})
        run.capture_source()
        inventory = json.loads((run.path / "source/inventory.json").read_text())
        self.assertEqual([entry["path"] for entry in inventory["files"]],
                         ["Cargo.toml", "crates/demo/src/lib.rs", "crates/demo/src/new.rs"])
        self.assertEqual((run.path / "source/files/crates/demo/src/lib.rs").read_text(),
                         "// dirty candidate\n")
        restored = self.root / "restored"
        run.restore_source(restored)
        self.assertEqual((restored / "crates/demo/src/new.rs").read_text(),
                         "// untracked maintained source\n")
        with self.assertRaises(RuntimeError):
            run.restore_source(self.root)
        source.write_text("// edited during check\n")
        with self.assertRaisesRegex(RuntimeError, "changed"):
            run.check_sources()

    def test_user_interrupt_logs_current_process_and_prevents_later_stages(self):
        run=Run.create(self.root,'readiness',{});run.capture_source()
        with self.assertRaises(KeyboardInterrupt):
            run.run_stage('user-stop',[sys.executable,'-c',
                'import os,signal,time; os.kill(os.getppid(),signal.SIGINT); time.sleep(5)'])
        self.assertEqual(run.checks['user-stop']['status'],'incomplete')
        self.assertEqual(run.checks['user-stop']['reason'],'Stage interrupted')
        with self.assertRaises(KeyboardInterrupt):run.run_stage('never-started',[sys.executable,'-c','print("unexpected")'])
        self.assertNotIn('never-started',run.checks)

    def test_failed_and_interrupted_stages_retain_logs_and_never_invent_artifacts(self):
        run = Run.create(self.root, "readiness", {})
        run.capture_source()
        result = run.run_stage("build", [sys.executable, "-c",
            "import sys; print('compiler output'); print('compiler failure', file=sys.stderr); sys.exit(7)"])
        self.assertEqual(result["returncode"], 7)
        self.assertEqual((run.path / result["stdout"]).read_text(), "compiler output\n")
        self.assertEqual((run.path / result["stderr"]).read_text(), "compiler failure\n")
        self.assertGreaterEqual(result["elapsed_seconds"], 0)
        self.assertNotIn("binary", run.manifest["identities"])
        timed = run.run_stage("timeout", [sys.executable, "-c", "import time; time.sleep(5)"], timeout=0.02)
        self.assertEqual(timed["status"], "incomplete")
        self.assertEqual(run.finalize("error", "build", "Build failed", ["build"]), 2)
        self.assertFalse(read_verdict(run.path)["acceptance_eligible"])

    def test_mutating_stage_cannot_record_a_pass(self):
        run = Run.create(self.root, "readiness", {})
        run.capture_source()
        result = run.run_stage("bad-check", [sys.executable, "-c",
            "from pathlib import Path; Path('crates/demo/src/lib.rs').write_text('changed')"])
        self.assertEqual(result["status"], "error")
        self.assertIn("changed", result["reason"])
        self.assertEqual(run.finalize("pass", mandatory_checks=["bad-check"]), 2)
        self.assertEqual(read_verdict(run.path)["status"], "error")

    def test_movie_inputs_bind_container_manifest_converted_inputs_and_compiled_rom(self):
        (self.root / "rom").mkdir()
        rom = self.root / "rom/sm.sfc"
        rom.write_bytes(b"fixture ROM")
        rom_hash = hashlib.sha256(rom.read_bytes()).hexdigest()
        (self.root / "movies").mkdir()
        movie = self.root / "movies/fixture.lsmv"
        with zipfile.ZipFile(movie, "w") as archive:
            for name, value in {"gametype": "snes_ntsc\n", "controlsversion": "0\n",
                                "coreversion": "bsnes v085 (Compatibility core)\n",
                                "rom.sha256": rom_hash + "\n",
                                "input": "F. 0 0|...S........\nF.|B.......A...\n"}.items():
                archive.writestr(name, value)
        movie.with_suffix(".inputs").write_text("1000\n8080\n")
        generated = self.root / "out/generated"
        generated.mkdir(parents=True)
        (generated / "steps.rs").write_text('pub const ROM_SHA256: Option<&str> = Some("' + rom_hash + '");\n')
        entry = {"file": "fixture.lsmv", "sha256": hashlib.sha256(movie.read_bytes()).hexdigest(),
                 "rom_sha256": rom_hash, "frames": 2}
        run = Run.create(self.root, "preflight", {})
        run.capture_source()
        identity = verify_movie_inputs(run, entry)
        self.assertEqual(identity["frames"], 2)
        self.assertEqual((run.path / "generated/movie.inputs").read_text(), "1000\n8080\n")
        self.assertEqual(identity["seed"], 0)
        self.assertEqual(identity["profile"], "lsnes-profile")
        self.assertEqual(identity["compiled_table_rom"], "verified")
        movie.with_suffix(".inputs").write_text("0000\n8080\n")
        with self.assertRaisesRegex(RuntimeError, "input"):
            verify_movie_inputs(Run.create(self.root, "preflight", {}), entry)

    def test_preflight_command_preserves_setup_failure_and_names_exact_user_step(self):
        (self.root / "movies").mkdir()
        (self.root / "movies/manifest.json").write_text(json.dumps([
            {"file": "super-metroid-any-3653.lsmv", "rom_sha256": "0" * 64,
             "sha256": "1" * 64, "frames": 18000}]))
        command = [sys.executable, str(Path(__file__).with_name("readiness.py")),
                   "preflight", "--root", str(self.root)]
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertIn("Run:", result.stdout)
        self.assertIn("Brief:", result.stdout)
        runs = list((self.root / "out/runs").iterdir())
        self.assertEqual(len(runs), 1)
        verdict = read_verdict(runs[0])
        self.assertEqual(verdict["status"], "error")
        self.assertFalse(verdict["acceptance_eligible"])
        self.assertIn("rom/sm.sfc", (runs[0] / "brief.md").read_text())
        self.assertTrue((runs[0] / "logs/doctor.json").is_file())

    def test_signals_and_stale_verdicts_remain_incomplete_and_failures_cannot_be_omitted(self):
        run = Run.create(self.root, "readiness", {})
        run.capture_source()
        report = run.run_stage("interrupted", [sys.executable, "-c",
                              "import os, signal; os.kill(os.getpid(), signal.SIGTERM)"])
        self.assertEqual(report["signal"], 15)
        self.assertEqual(report["status"], "incomplete")
        run.record_check("apparently-good", {"status": "pass", "complete": True})
        self.assertEqual(run.finalize("pass", mandatory_checks=["apparently-good"]), 2)
        self.assertFalse(read_verdict(run.path)["acceptance_eligible"])
        other = Run.create(self.root, "readiness", {})
        (other.path / "verdict.json").write_bytes((run.path / "verdict.json").read_bytes())
        self.assertEqual(read_verdict(other.path)["status"], "incomplete")

    def test_success_artifact_is_preserved_and_identity_cannot_be_replaced(self):
        run = Run.create(self.root, "readiness", {})
        binary = self.root / "candidate-binary"
        binary.write_bytes(b"candidate executable")
        identity = run.record_artifact("binary", binary, "generated/candidate-binary")
        self.assertEqual(identity["bytes"], 20)
        self.assertEqual((run.path / "generated/candidate-binary").read_bytes(), b"candidate executable")
        with self.assertRaises(RuntimeError):
            run.record_artifact("binary", binary)
        with self.assertRaises(OSError):
            run.record_artifact("missing-binary", self.root / "missing")
        self.assertNotIn("missing-binary", run.manifest["identities"])


if __name__ == "__main__":
    unittest.main()
