"""Invented metadata proves the DLL runner accounts for every requested package."""

import importlib.util
import json
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("test_dll", Path(__file__).with_name("test_dll.py"))
assert spec is not None and spec.loader is not None
test_dll = importlib.util.module_from_spec(spec)
spec.loader.exec_module(test_dll)


class CoverageTests(unittest.TestCase):
    def select(self, packages, requested):
        result = SimpleNamespace(stdout=json.dumps({"packages": packages}))
        with patch.object(test_dll.subprocess, "run", return_value=result):
            return test_dll.targets("cargo", requested)

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
