"""Invented metadata proves the DLL runner accounts for every requested package."""

import contextlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import threading
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

    def wrapper_status(self, outcomes, selected=None, jobs="4"):
        """The wrapper's status when cargo builds every selected target and the children return
        `outcomes` after the build's."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = selected or [("sample", "--lib", "", root)]
            built = {target: root / f"harness{index}.exe" for index, target in enumerate(selected)}
            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "sample"]), \
                 patch.dict(test_dll.os.environ, {"SM_TEST_JOBS": jobs}), \
                 patch.object(test_dll, "targets", return_value=selected), \
                 patch.object(test_dll, "artifacts", return_value=built), \
                 patch.object(test_dll, "place"), \
                 patch.object(test_dll.subprocess, "run", side_effect=outcomes), \
                 contextlib.redirect_stdout(io.StringIO()):
                return test_dll.main()

    def test_child_failure_preserves_stage_and_infrastructure_precedence(self):
        result = lambda status: subprocess.CompletedProcess([], status)
        self.assertEqual(self.wrapper_status([result(0), result(0)]), 0)
        self.assertEqual(self.wrapper_status([result(0), result(101)]), 1)
        for outcomes in [[result(126), result(0)], [result(1), result(0)], [OSError()],
                         [result(0), result(126)], [result(0), result(-9)], [result(0), result(1)],
                         [result(0), OSError()]]:
            self.assertEqual(self.wrapper_status(outcomes), 2)
        selected = [("sample", "--lib", "", Path(".")), ("sample", "--test", "other", Path("."))]
        for jobs in ["1", "4"]:
            self.assertEqual(self.wrapper_status([result(0), result(101), result(1)], selected, jobs), 2)
            self.assertEqual(self.wrapper_status([result(0), result(101), result(0)], selected, jobs), 1)

    def test_a_target_cargo_did_not_build_is_an_infrastructure_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = [("sample", "--lib", "", root), ("sample", "--test", "other", root)]
            runs = []

            def run(command, **kwargs):
                runs.append(command)
                return subprocess.CompletedProcess(command, 0)

            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "sample"]), \
                 patch.object(test_dll, "targets", return_value=selected), \
                 patch.object(test_dll, "artifacts", return_value={selected[0]: root / "lib.exe"}), \
                 patch.object(test_dll, "place"), \
                 patch.object(test_dll.subprocess, "run", side_effect=run), \
                 contextlib.redirect_stdout(io.StringIO()) as captured:
                self.assertEqual(test_dll.main(), 2)
            self.assertEqual(len(runs), 2)  # The build, then the one suite that was built.
            self.assertIn("sample --test other (build)", captured.getvalue())

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

            selected = [("sample", "--lib", "", root)]
            captured = io.StringIO()
            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "--suite", "lib", "sample"]), patch.object(test_dll, "targets", return_value=selected), patch.object(test_dll, "artifacts", return_value={selected[0]: root / "lib.exe"}), patch.object(test_dll, "place"), patch.object(test_dll.subprocess, "run", side_effect=run), contextlib.redirect_stdout(captured):
                self.assertEqual(test_dll.main(), 2)
            self.assertEqual(calls, 2)
            self.assertIn("test result: FAILED.", captured.getvalue())

    def test_one_cargo_invocation_builds_every_requested_target(self):
        root = Path("/invented")
        selected = [("a", "--lib", "", root / "a"), ("a", "--test", "t", root / "a"),
                    ("b", "--bin", "tool", root / "b")]
        command = test_dll.build_command("cargo", selected, None)
        self.assertEqual(command[:3], ["cargo", "test", "--no-run"])
        self.assertEqual(command[command.index("--target") + 1], "x86_64-pc-windows-gnu")
        self.assertEqual([command[i + 1] for i, arg in enumerate(command) if arg == "-p"], ["a", "b"])
        self.assertEqual(command[-1], "--tests")
        self.assertEqual(test_dll.build_command("cargo", selected[:1], "lib")[-1], "--lib")
        self.assertEqual(test_dll.build_command("cargo", selected[1:2], "test:t")[-2:], ["--test", "t"])

    def test_harnesses_link_as_dlls_exporting_main_and_keep_existing_flags(self):
        definition = Path("/invented dir/exports.def")
        tail = ["-C", "link-arg=-shared", "-C", "link-arg=/invented dir/exports.def"]
        for env, expected in [({}, tail), ({"RUSTFLAGS": "-C debuginfo=0"}, ["-C", "debuginfo=0", *tail]),
                              ({"RUSTFLAGS": "-W x", "CARGO_ENCODED_RUSTFLAGS": "-C\x1fopt-level=1"},
                               ["-C", "opt-level=1", *tail])]:
            with patch.dict(test_dll.os.environ, env, clear=True):
                flags = test_dll.build_env(definition)["CARGO_ENCODED_RUSTFLAGS"]
            self.assertEqual(flags.split("\x1f"), expected)

    def test_artifacts_map_each_harness_to_its_package_and_target(self):
        a, b = Path("/invented/a"), Path("/invented/b")
        selected = [("a", "--lib", "", a), ("a", "--test", "vectors", a), ("b", "--test", "vectors", b),
                    ("b", "--bin", "tool", b)]

        def message(root, kind, name, test, executable):
            return json.dumps({"reason": "compiler-artifact", "manifest_path": str(root / "Cargo.toml"),
                               "target": {"kind": [kind], "name": name}, "profile": {"test": test},
                               "executable": executable})

        stdout = "\n".join([
            message(a, "lib", "a", True, "/t/a-1.exe"),
            message(a, "lib", "a", False, None),
            message(a, "test", "vectors", True, "/t/vectors-2.exe"),
            message(b, "test", "vectors", True, "/t/vectors-3.exe"),
            message(b, "bin", "tool", False, "/t/tool.exe"),
            message(b, "custom-build", "build-script-build", False, "/t/build.exe"),
            json.dumps({"reason": "build-finished", "success": True}),
        ])
        built = test_dll.artifacts(stdout, selected)
        self.assertEqual(built, {selected[0]: Path("/t/a-1.exe"), selected[1]: Path("/t/vectors-2.exe"),
                                 selected[2]: Path("/t/vectors-3.exe")})

    def test_place_links_the_harness_once_and_replaces_a_stale_dll(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            harness, dll = root / "harness.exe", root / "out.dll"
            harness.write_bytes(b"new")
            dll.write_bytes(b"stale")
            test_dll.place(harness, dll)
            self.assertEqual(dll.read_bytes(), b"new")
            self.assertTrue(os.path.samefile(harness, dll))
            test_dll.place(harness, dll)
            self.assertTrue(os.path.samefile(harness, dll))

    def test_suites_run_concurrently_up_to_the_job_limit_with_whole_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = [("sample", "--test", f"t{index}", root) for index in range(3)]
            built = {target: root / f"{target[2]}.exe" for target in selected}
            for jobs, together in [("3", True), ("2", False), ("1", False)]:
                barrier = threading.Barrier(3, timeout=2)
                met = []

                def run(command, **kwargs):
                    if command[0] == "cargo":
                        return subprocess.CompletedProcess(command, 0)
                    try:
                        barrier.wait()
                        met.append(True)
                    except threading.BrokenBarrierError:
                        met.append(False)
                    name = Path(command[4]).stem
                    return subprocess.CompletedProcess(command, 0, f"{name} first\n{name} last\n".encode())

                with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "sample"]), \
                     patch.dict(test_dll.os.environ, {"SM_TEST_JOBS": jobs}), \
                     patch.object(test_dll, "targets", return_value=selected), \
                     patch.object(test_dll, "artifacts", return_value=built), \
                     patch.object(test_dll, "place"), \
                     patch.object(test_dll.subprocess, "run", side_effect=run), \
                     contextlib.redirect_stdout(io.StringIO()) as captured:
                    self.assertEqual(test_dll.main(), 0)
                self.assertEqual(all(met), together, jobs)
                for index in range(3):
                    self.assertIn(f"(sample-t{index}.dll)\nsample-t{index} first\nsample-t{index} last\n",
                                  captured.getvalue())

    def test_every_suite_exports_only_main_and_forwards_libtest_arguments(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = [("sample", kind, name, root) for kind, name in
                        [("--lib", ""), ("--bin", "tool"), ("--test", "integration")]]
            builds = []
            runs = []

            def run(command, **kwargs):
                if command[0] == "cargo":
                    flags = kwargs["env"]["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
                    self.assertIn("link-arg=-shared", flags)
                    definitions = [Path(arg.removeprefix("link-arg=")) for arg in flags
                                   if arg.startswith("link-arg=") and arg.endswith(".def")]
                    self.assertEqual(len(definitions), 1)
                    self.assertEqual(definitions[0].read_bytes(), b"EXPORTS\nmain\n")
                    builds.append(command)
                else:
                    self.assertEqual(command[-3:], ["--exact", "invented test", "--nocapture"])
                    self.assertEqual(kwargs["cwd"], root)
                    runs.append(command)
                return subprocess.CompletedProcess(command, 0, b"")

            built = {target: root / f"harness{index}.exe" for index, target in enumerate(selected)}
            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "sample",
                                           "--", "--exact", "invented test", "--nocapture"]), \
                 patch.object(test_dll, "targets", return_value=selected), \
                 patch.object(test_dll, "artifacts", return_value=built), \
                 patch.object(test_dll, "place"), \
                 patch.object(test_dll.subprocess, "run", side_effect=run), \
                 contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(test_dll.main(), 0)
            self.assertEqual(len(builds), 1)
            self.assertEqual(len(runs), 3)

    def test_failed_export_preparation_does_not_build_or_run(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(sys, "argv", ["test_dll.py", "cargo", str(root / "out"), "sample"]), \
                 patch.object(test_dll, "targets", return_value=[("sample", "--lib", "", root)]), \
                 patch.object(Path, "write_text", side_effect=OSError()), \
                 patch.object(test_dll.subprocess, "run") as run, \
                 contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(test_dll.main(), 2)
                run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
