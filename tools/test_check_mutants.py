"""Bridge fault credit requires an exact recorded difference and restored Pass."""
import tempfile
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
import json
import subprocess
import tomllib
import unittest
from dataclasses import replace
from pathlib import Path
from typing import Any
from unittest.mock import patch
import check_mutants
from check_mutants import (Fault, FrozenInputs, build_executable, check_fault, receipt,
                           run_case, source_path, validate_coverage)

def outcome(status: str, case: str = "audio/example", cut: int = 3, field: str = "audio.reply0") -> str:
    text = f'format = 1\n[[cases]]\nid = "{case}"\nobservation_sha256 = "original"\n[cases.outcome]\nstatus = "{status}"\n'
    if status == "difference":
        text += f'cut = {cut}\nfield = "{field}"\n'
    return text + '[cases.counts]\ncuts = 3\nrestorations = 2\n'

def pass_receipt() -> dict[str, Any]:
    observed = receipt(outcome("pass"), "audio/example")
    assert observed is not None
    return observed

class MutantCheck(unittest.TestCase):
    def test_success_leaves_normal_artifact_vanilla_for_direct_replay(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            source = root / "crates/game/src/production.rs"
            source.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            source.write_bytes(original)
            config = root / "tools/mutants.toml"
            config.parent.mkdir()
            config.write_text('''[[fault]]
name = "first"
responsibility = "audio"
file = "crates/game/src/production.rs"
before = "measured"
after = "zero"
case = "audio/example"
cut = 3
field = "audio.reply0"
[[fault]]
name = "last"
responsibility = "audio"
file = "crates/game/src/production.rs"
before = "measured"
after = "zero"
case = "audio/example"
cut = 3
field = "audio.reply0"
''', encoding="utf-8", newline="\n")
            target = root / "target/tools"
            target.parent.mkdir()
            inputs = FrozenInputs({source: original, config: config.read_bytes()})
            def compile_source(build_root: Path) -> Path:
                target.write_bytes(source.read_bytes())
                return target
            def execute(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                vanilla = Path(command[0]).read_bytes() == original
                text = outcome("pass" if vanilla else "difference")
                return subprocess.CompletedProcess(command, 0 if vanilla else 1,
                                                   text if vanilla else "", "" if vanilla else text)
            with patch.object(check_mutants, "__file__", str(root / "tools/check_mutants.py")), \
                 patch("check_mutants.validate_coverage"), \
                 patch("check_mutants.FrozenInputs.capture", return_value=inputs), \
                 patch("check_mutants.input_paths", return_value=set(inputs.files)), \
                 patch("check_mutants.build_executable", side_effect=compile_source), \
                 patch("check_mutants.subprocess.run", side_effect=execute):
                self.assertEqual(check_mutants.main([]), 0)
                self.assertEqual(source.read_bytes(), original)
                self.assertTrue(check_mutants.passed(run_case(root, "audio/example", target), "audio/example"))
                self.assertEqual(target.read_bytes(), original)

    def test_plan_and_worker_partition_exhaustive_rosters(self) -> None:
        template = Fault("fault", "audio", "crates/game/src/production.rs",
                         "measured", "zero", "audio/example", 3, "audio.reply0")
        for count in [1, 3, 4, 5, 9, 25]:
            roster = tuple(replace(template, name=f"fault-{index}") for index in range(count))
            workers = check_mutants.partition_faults(roster)
            self.assertEqual(len(workers), min(count, 4))
            self.assertTrue(all(workers))
            self.assertCountEqual([fault for worker in workers for fault in worker], roster)
        with self.assertRaises(ValueError):
            check_mutants.partition_faults(())

        output = StringIO()
        with redirect_stdout(output):
            self.assertEqual(check_mutants.main(["plan"]), 0)
        self.assertEqual(json.loads(output.getvalue()), {"worker": [0, 1, 2, 3]})

        output = StringIO()
        with patch("check_mutants.partition_faults", return_value=((template,),)), redirect_stdout(output):
            self.assertEqual(check_mutants.main(["plan"]), 0)
            with redirect_stderr(StringIO()), self.assertRaises(SystemExit) as error:
                check_mutants.main(["worker", "1"])
            self.assertEqual(error.exception.code, 2)
        self.assertEqual(json.loads(output.getvalue()), {"worker": [0]})

    def test_invalid_worker_commands_fail_before_build(self) -> None:
        for args in [["worker"], ["worker", "-1"], ["worker", "4"], ["worker", "01"],
                     ["worker", "0", "0"], ["worker", "0", "worker", "1"],
                     ["plan", "worker", "0"]]:
            with self.subTest(args=args), redirect_stderr(StringIO()), self.assertRaises(SystemExit) as error:
                check_mutants.main(args)
            self.assertEqual(error.exception.code, 2)

    def test_build_failure_cannot_execute_a_stale_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stale = root / "tools"
            stale.write_bytes(b"stale executable")
            artifact = json.dumps({"reason": "compiler-artifact", "target": {
                "name": "tools", "kind": ["bin"]}, "executable": str(stale)})
            failed = subprocess.CompletedProcess([], 101, artifact, "")
            with patch("check_mutants.subprocess.run", return_value=failed) as execute:
                self.assertEqual(run_case(root, "audio/example"), (101, ""))
                self.assertEqual(execute.call_count, 1)

    def test_cargo_reported_artifact_is_required_and_respects_its_location(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "custom-target" / "tools"
            executable.parent.mkdir()
            executable.write_bytes(b"executable")
            artifact = {"reason": "compiler-artifact", "target": {
                "name": "tools", "kind": ["bin"]}, "executable": str(executable)}
            for rows in [[], [artifact, artifact], [{**artifact, "executable": str(root / "missing")}]]:
                result = subprocess.CompletedProcess([], 0, "\n".join(json.dumps(row) for row in rows), "")
                with patch("check_mutants.subprocess.run", return_value=result):
                    with self.assertRaises(ValueError):
                        build_executable(root)
            result = subprocess.CompletedProcess([], 0, json.dumps(artifact), "")
            with patch("check_mutants.subprocess.run", return_value=result):
                self.assertEqual(build_executable(root), executable.resolve())

    def test_frozen_inputs_preserve_dirty_missing_and_untracked_file_states(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            tracked = root / "tracked.rs"
            absent = root / "absent.rs"
            tracked.write_bytes(b"initial\n")
            absent.write_bytes(b"initial\n")
            subprocess.run(["git", "add", "tracked.rs", "absent.rs"], cwd=root, check=True)
            tracked.write_bytes(b"dirty\r\n")
            absent.unlink()
            untracked = root / "config.toml"
            untracked.write_bytes(b"input\n")
            inputs = FrozenInputs.capture(root)
            inputs.check(root)
            for path, changed in [(tracked, b"initial\n"), (absent, b"appeared\n"),
                                  (untracked, b"changed\n"), (root / "new.rs", b"new\n")]:
                original = path.read_bytes() if path.exists() else None
                path.write_bytes(changed)
                with self.assertRaises(ValueError):
                    inputs.check(root)
                if original is None:
                    path.unlink()
                else:
                    path.write_bytes(original)
            untracked.unlink()
            with self.assertRaises(ValueError):
                inputs.check(root)

    def test_concurrent_neighbour_edits_prevent_retained_replay_credit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/game/src/production.rs"
            path.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            path.write_bytes(original)
            neighbour = root / "model.rs"
            neighbour.write_bytes(b"original\n")
            inputs = FrozenInputs({path: original, neighbour: b"original\n"})
            fault = Fault("zero", "audio", str(path.relative_to(root)), "measured", "zero", "audio/example", 3, "audio.reply0")
            for edit_during in [1, 2]:
                calls = 0
                neighbour.write_bytes(b"original\n")
                def replay(*args: object) -> tuple[int, str]:
                    nonlocal calls
                    calls += 1
                    if calls == edit_during:
                        neighbour.write_bytes(b"changed\n")
                    return (1, outcome("difference")) if calls == 1 else (0, outcome("pass"))
                with self.subTest(edit_during=edit_during), \
                     patch("check_mutants.input_paths", return_value={path, neighbour}), \
                     patch("check_mutants.run_case", side_effect=replay) as execute:
                    with self.assertRaises(ValueError):
                        check_fault(root, fault, pass_receipt(), root / "vanilla", inputs)
                    self.assertEqual(execute.call_count, edit_during)
                self.assertEqual(path.read_bytes(), original)

    def exercise(self, result: tuple[int, str], restored: tuple[int, str] | None = None) -> bool:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/game/src/production.rs"
            path.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            path.write_bytes(original)
            fault = Fault("zero", "audio", str(path.relative_to(root)), "measured", "zero", "audio/example", 3, "audio.reply0")
            with patch("check_mutants.run_case", side_effect=[result, restored or (0, outcome("pass"))]):
                killed = check_fault(root, fault, pass_receipt())
            self.assertEqual(path.read_bytes(), original)
            return killed

    def test_only_production_adapters_are_allowed(self) -> None:
        root = Path(__file__).resolve().parents[1]
        for name in ["crates/game/src/production.rs", "crates/game/src/production/movement.rs"]:
            self.assertEqual(source_path(root, name), (root / name).resolve())
        for name in ["crates/clean/types/src/lib.rs", "crates/game/src/tick.rs",
                     "crates/game/src/production_other/audio.rs", "crates/game/src/production/../production.rs"]:
            with self.assertRaises(ValueError):
                source_path(root, name)

    def test_build_failure_missing_receipt_and_wrong_identity_cannot_kill(self) -> None:
        for result in [(101, outcome("difference")), (1, ""), (1, outcome("unsupported")),
                       (1, outcome("difference", case="another/case")), (1, outcome("difference", cut=4)),
                       (1, outcome("difference", field="audio.reply1")), (0, outcome("pass"))]:
            self.assertFalse(self.exercise(result))
        self.assertTrue(self.exercise((1, "tools: " + outcome("difference"))))

    def test_restored_baseline_must_pass(self) -> None:
        self.assertFalse(self.exercise((1, outcome("difference")), (1, outcome("difference"))))

    def test_restored_pass_must_keep_receipt_identity(self) -> None:
        for before, after in [('cuts = 3', 'cuts = 4'), ('restorations = 2', 'restorations = 1'),
                              ('observation_sha256 = "original"', 'observation_sha256 = "changed"')]:
            restored = outcome("pass").replace(before, after)
            self.assertFalse(self.exercise((1, outcome("difference")), (0, restored)))

    def test_supported_responsibilities_cannot_lose_faults(self) -> None:
        root = Path(__file__).resolve().parents[1]
        with (root / "tools/mutants.toml").open("rb") as stream:
            faults = [Fault(**row) for row in tomllib.load(stream)["fault"]]
        validate_coverage(root, faults)
        for owner in {fault.responsibility for fault in faults}:
            with self.assertRaises(ValueError):
                validate_coverage(root, [fault for fault in faults if fault.responsibility != owner])
        for changed in [replace(faults[0], responsibility="contact"),
                        replace(faults[0], case="blocks/missing-requests")]:
            with self.assertRaises(ValueError):
                validate_coverage(root, [changed, *faults[1:]])

    def test_source_restores_after_runner_exception(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/game/src/production.rs"
            path.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            path.write_bytes(original)
            fault = Fault("zero", "audio", str(path.relative_to(root)), "measured", "zero", "audio/example", 3, "audio.reply0")
            with patch("check_mutants.run_case", side_effect=OSError):
                with self.assertRaises(OSError):
                    check_fault(root, fault, pass_receipt())
            self.assertEqual(path.read_bytes(), original)

if __name__ == "__main__":
    unittest.main()
