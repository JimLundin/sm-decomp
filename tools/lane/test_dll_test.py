"""Invented metadata proves the DLL runner accounts for every requested package."""

import contextlib
import importlib.util
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("test_dll", Path(__file__).with_name("test_dll.py"))
assert spec is not None and spec.loader is not None
test_dll = importlib.util.module_from_spec(spec)
spec.loader.exec_module(test_dll)


class CoverageTests(unittest.TestCase):
    def select(self, packages, requested, suite=None):
        result = SimpleNamespace(stdout=json.dumps({"packages": packages}))
        with patch.object(test_dll.subprocess, "run", return_value=result):
            return test_dll.targets("cargo", requested, suite)

    def package(self, name, kinds):
        return {
            "name": name,
            "manifest_path": f"/invented/{name}/Cargo.toml",
            "targets": [{"name": kind, "kind": [kind], "test": enabled} for kind, enabled in kinds],
        }

    def test_enabled_unit_and_integration_targets(self):
        package = self.package("sample", [("lib", True), ("bin", True), ("proc-macro", True), ("test", True), ("example", True), ("bin", False)])
        selected = self.select([package], ["sample"])
        self.assertEqual([(kind, name) for _, kind, name, _ in selected], [("--lib", ""), ("--bin", "bin"), ("--lib", ""), ("--test", "test")])

    def test_empty_package_fails_alone_and_in_mixed_request(self):
        empty = self.package("empty", [("bin", False)])
        covered = self.package("covered", [("lib", True)])
        for requested in [["empty"], ["covered", "empty"]]:
            with self.assertRaises(SystemExit):
                self.select([empty, covered], requested)

    def test_suite_selector_runs_one_enabled_existing_target(self):
        package = self.package("sample", [("lib", True), ("test", True), ("bin", True)])
        self.assertEqual([(kind, name) for _, kind, name, _ in self.select([package], ["sample"], "lib")], [("--lib", "")])
        self.assertEqual([(kind, name) for _, kind, name, _ in self.select([package], ["sample"], "test:test")], [("--test", "test")])
        for selector in ["test:absent", "test:", "bin", "x86_64-pc-windows-gnu"]:
            with self.assertRaises(SystemExit):
                self.select([package], ["sample"], selector)
        disabled = self.package("sample", [("lib", True), ("test", False)])
        with self.assertRaises(SystemExit):
            self.select([disabled], ["sample"], "test:test")
        with self.assertRaises(SystemExit):
            self.select([package], ["sample", "sample"], "lib")

    def wrapper_status(self, outcomes, selected=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = selected or [("sample", "--lib", "", root)]
            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "sample"]), patch.object(test_dll, "targets", return_value=selected), patch.object(test_dll.subprocess, "run", side_effect=outcomes), contextlib.redirect_stdout(io.StringIO()):
                return test_dll.main()

    def test_child_failure_preserves_stage_and_infrastructure_precedence(self):
        result = lambda status: subprocess.CompletedProcess([], status)
        self.assertEqual(self.wrapper_status([result(0), result(0)]), 0)
        self.assertEqual(self.wrapper_status([result(0), result(101)]), 1)
        for outcomes in [[result(126)], [result(1)], [OSError()],
                         [result(0), result(126)], [result(0), result(-9)],
                         [result(0), OSError()]]:
            self.assertEqual(self.wrapper_status(outcomes), 2)
        selected = [("sample", "--lib", "", Path(".")), ("sample", "--test", "other", Path("."))]
        self.assertEqual(self.wrapper_status([result(0), result(101), result(1)], selected), 2)

    def test_target_discovery_failure_is_infrastructure(self):
        with tempfile.TemporaryDirectory() as directory:
            for error in [OSError(), subprocess.CalledProcessError(1, []), ValueError(), SystemExit(1)]:
                with patch.object(sys, "argv", ["test_dll.py", "cargo", directory, "sample"]), patch.object(test_dll, "targets", side_effect=error), contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(test_dll.main(), 2)

    def test_real_child126_after_failed_summary_is_an_infrastructure_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = root / "child.py"
            child.write_text("import sys\n"
                             "sys.stdout.buffer.write(b'running 1 test\\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\\n')\n"
                             "sys.exit(126)\n", encoding="utf-8", newline="\n")
            real_run = subprocess.run
            calls = 0

            def run(command, **kwargs):
                nonlocal calls
                calls += 1
                if calls == 1:
                    return subprocess.CompletedProcess(command, 0)
                result = real_run([sys.executable, str(child)], capture_output=True)
                print(result.stdout.decode(), end="")
                return result

            captured = io.StringIO()
            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "--suite", "lib", "sample"]), patch.object(test_dll, "targets", return_value=[("sample", "--lib", "", root)]), patch.object(test_dll.subprocess, "run", side_effect=run), contextlib.redirect_stdout(captured):
                self.assertEqual(test_dll.main(), 2)
            self.assertEqual(calls, 2)
            self.assertIn("test result: FAILED.", captured.getvalue())

    def test_bin_command_compiles_a_test_harness(self):
        command = test_dll.build_command("cargo", "sample", "--bin", "sample", Path("sample.dll"))
        self.assertEqual(command[command.index("--profile") + 1], "test")
        self.assertEqual(command[command.index("--bin") + 1], "sample")
        self.assertIn("link-arg=-shared", command)
        for kind, name in [("--lib", ""), ("--test", "integration")]:
            command = test_dll.build_command("cargo", "sample", kind, name, Path("sample.dll"))
            self.assertNotIn("--test", command[command.index("--") + 1:])


if __name__ == "__main__":
    unittest.main()
