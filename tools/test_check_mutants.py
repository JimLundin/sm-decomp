"""Bridge fault credit requires an exact recorded difference and restored Pass."""
import tempfile
from contextlib import redirect_stdout
from io import StringIO
import json
import subprocess
import tomllib
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch
import check_mutants
from check_mutants import (PLAYED_RUNNER, Fault, FrozenInputs, build_executable, detected, receipt,
                           replay_selectable, scratch_files, source_path, validate_coverage)

def outcome(status: str, case: str = "audio/example", cut: int = 3, field: str = "audio.reply0") -> str:
    text = f'format = 1\n[[cases]]\nid = "{case}"\nobservation_sha256 = "original"\n[cases.outcome]\nstatus = "{status}"\n'
    if status == "difference":
        text += f'cut = {cut}\nfield = "{field}"\n'
    return text + '[cases.counts]\ncuts = 3\nrestorations = 2\n'

class MutantCheck(unittest.TestCase):
    def test_rule_inventory_includes_entries_without_a_fault_or_case(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            specs = root / "docs/behaviours"
            specs.mkdir(parents=True)
            (root / "docs/architecture").mkdir()
            (root / "docs/architecture/domains.toml").write_text(
                '[[model]]\nname = "world-blocks"\n', encoding="utf-8")
            for domain in ("world-intro-rooms", "world-shaft-entry"):
                (specs / f"{domain}.md").touch()
            (specs / "uncatalogued.md").write_text("### FutureRule\n", encoding="utf-8")
            (specs / "world-blocks.md").write_text(
                "### Closure\n### 2.1 A heading\n```\n### Example\n```\n### Untested\n", encoding="utf-8")
            (specs / "world-blocks.decode.md").write_text("### Notes\n", encoding="utf-8")
            self.assertEqual(check_mutants.catalogue_rules(root), ["world-blocks/Closure", "world-blocks/Untested"])
            with (specs / "world-blocks.md").open("a", encoding="utf-8") as spec:
                spec.write("### Untested\n")
            with self.assertRaises(ValueError):
                check_mutants.catalogue_rules(root)

    def test_rule_credit_requires_its_assigned_fault_and_complete_receipt(self) -> None:
        fault = Fault("contact", "played", PLAYED_RUNNER, "old", "new",
                      "played steam 0 200", 137, "steam.hit", "PuffTouches")
        placement = replace(fault, name="placement", cut=100, field="steam.vents",
                            rule="VentPlaced")
        unassigned = replace(fault, name="unassigned", rule=None)
        def block(entry: Fault) -> list[str]:
            replay = f"TIMING replay case={entry.case} seconds=0.100000"
            return [
                replay,
                f"KILLED {entry.name}: {entry.case} cut{entry.cut} {entry.field}",
                f"TIMING restoration fault={entry.name} seconds=0.100000",
                "TIMING input-check seconds=0.100000",
                "TIMING compilation seconds=0.100000",
                replay,
                "TIMING input-check seconds=0.100000",
                f"RESTORED PASS {entry.case}",
            ]

        def report(logs: list[str]) -> list[str]:
            output = StringIO()
            with redirect_stdout(output):
                check_mutants.report_rule_coverage(
                    ["PuffTouches", "PuffReach", "VentPlaced"],
                    [fault, placement, unassigned], logs)
            return output.getvalue().splitlines()

        contact = block(fault)
        other = block(replace(placement, case="played other 0 200"))
        expected = ["PuffTouches: unknown", "PuffReach: unknown",
                    "VentPlaced: unknown", "Unassigned faults: 1"]
        for label, lines in {
            "swapped restoration and resumed case": contact[:2] + [other[2], other[5], contact[-1]],
            "wrong restoration fault": contact[:2] + [other[2]] + contact[3:],
            "wrong resumed case": contact[:5] + [other[5]] + contact[6:],
            "wrong kill cut": block(replace(fault, cut=138)),
            "unassigned": block(unassigned),
            "orphaned pass": [contact[-1]],
            "missing restoration": contact[:2] + contact[3:],
            "missing resumed replay": contact[:5] + contact[6:],
            "duplicate kill": contact[:2] + contact[1:],
            "duplicate restoration": contact[:3] + contact[2:],
            "duplicate resumed replay": contact[:6] + contact[5:],
            "duplicate pass": contact + contact[-1:],
        }.items():
            with self.subTest(label=label):
                self.assertEqual(report(["\n".join(lines)]), expected)
        self.assertEqual(report(["\n".join(contact[:-1]), contact[-1]]), expected)
        self.assertEqual(report(["\n".join(contact + block(unassigned))]),
                         ["PuffTouches: killed through contact", *expected[1:]])

    def test_full_manifest_uses_the_selectable_runner(self) -> None:
        root = Path(__file__).resolve().parents[1]
        manifest = root / "tools/mutants.toml"
        raw = manifest.read_bytes()
        data = tomllib.loads(raw.decode())
        roster = tuple(Fault(**row) for row in data["fault"])
        callers = check_mutants.runtime_callers(root, data["runtime_callers"])
        frozen = FrozenInputs({manifest: raw})
        for args in ([], ["selectable"]):
            with (
                self.subTest(args=args),
                patch.object(FrozenInputs, "capture", return_value=frozen),
                patch.object(check_mutants, "build_executable", side_effect=AssertionError),
                patch.object(check_mutants, "run_selectable", return_value=1) as run,
            ):
                self.assertEqual(check_mutants.main(args), 1)
                run.assert_called_once_with(root, frozen, roster, callers)

    def test_build_failure_cannot_execute_a_stale_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stale = root / "tools"
            stale.write_bytes(b"stale executable")
            artifact = json.dumps({"reason": "compiler-artifact", "target": {
                "name": "tools", "kind": ["bin"]}, "executable": str(stale)})
            failed = subprocess.CompletedProcess([], 101, artifact, "")
            with patch("check_mutants.subprocess.run", return_value=failed):
                with self.assertRaises(subprocess.CalledProcessError):
                    build_executable(root, target=root / "build")

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
                        build_executable(root, target=root / "build")
            result = subprocess.CompletedProcess([], 0, json.dumps(artifact), "")
            with patch("check_mutants.subprocess.run", return_value=result):
                self.assertEqual(build_executable(root, target=root / "build"), executable.resolve())

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

    def exercise(self, result: tuple[int, str], restored: tuple[int, str] | None = None) -> bool:
        fault = Fault("zero", "audio", "source.rs", "before", "after", "audio/example", 3, "audio.reply0")
        healthy = (0, outcome("pass"))
        with tempfile.TemporaryDirectory() as directory:
            tree = Path(directory)
            frozen = FrozenInputs.capture(tree, scratch_files)
            with patch("check_mutants.run_case", side_effect=[healthy, result, restored or healthy]):
                return replay_selectable(tree, (fault,), tree / "tools", frozen, {})

    def test_only_production_adapters_are_allowed(self) -> None:
        root = Path(__file__).resolve().parents[1]
        for name in ["crates/game/src/production.rs", "crates/game/src/production/movement.rs",
                     PLAYED_RUNNER, "crates/verify/src/replay/played/platform.rs"]:
            self.assertEqual(source_path(root, name), (root / name).resolve())
        for name in ["crates/clean/types/src/lib.rs", "crates/game/src/tick.rs",
                     "crates/verify/src/replay/hud.rs",
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
                        replace(faults[0], case="blocks/missing-requests"),
                        replace(faults[0], rule="samus-movement/DoesNotExist"),
                        replace(faults[0], rule="game-random/AdvanceRandom")]:
            with self.assertRaises(ValueError):
                validate_coverage(root, [changed, *faults[1:]])

    def test_every_anchor_names_one_place_in_its_source(self) -> None:
        root = Path(__file__).resolve().parents[1]
        with (root / "tools/mutants.toml").open("rb") as stream:
            faults = [Fault(**row) for row in tomllib.load(stream)["fault"]]
        for fault in faults:
            with self.subTest(fault=fault.name, file=fault.file):
                self.assertEqual(source_path(root, fault.file).read_bytes().count(fault.before.encode()), 1)

    def test_manifest_validation_checks_anchors_without_building(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            source = root / "crates/game/src/production.rs"
            source.parent.mkdir(parents=True)
            config = root / "tools/mutants.toml"
            config.parent.mkdir()
            config.write_text('''[[fault]]
name = "zero"
responsibility = "audio"
file = "crates/game/src/production.rs"
before = "measured"
after = "zero"
case = "audio/example"
cut = 3
field = "audio.reply0"
''', encoding="utf-8", newline="\n")
            for text, unique in [(b"let elapsed = measured;\n", True), (b"let elapsed = sampled;\n", False),
                                 (b"let a = measured;\nlet b = measured;\n", False)]:
                source.write_bytes(text)
                with self.subTest(source=text), \
                     patch.object(check_mutants, "__file__", str(root / "tools/check_mutants.py")), \
                     patch("check_mutants.validate_coverage"), \
                     patch("check_mutants.build_executable") as build, redirect_stdout(StringIO()):
                    if unique:
                        self.assertEqual(check_mutants.main(["validate"]), 0)
                    else:
                        with self.assertRaises(ValueError):
                            check_mutants.main(["validate"])
                    build.assert_not_called()

    def test_a_played_run_kills_only_at_its_recorded_stop(self) -> None:
        case = "played row-1 0 9"
        def report(stop: str, row: str = "row-1") -> str:
            return f'PLAYED_ALONE row={row} start=0 held=4 stop={stop} seed_held=["energy"]\n'
        stopped = report('At(Difference { cut: 5, field: "camera.x" })')
        ended = receipt(report("End"), case)
        assert ended is not None
        self.assertEqual(ended["outcome"], {"status": "pass"})
        self.assertEqual(receipt(stopped, case), {"id": case, "report": stopped.strip(), "outcome": {
            "status": "difference", "cut": 5, "field": "camera.x"}})
        for output in [report("End", row="row-2"), report("End") * 2, ""]:
            self.assertIsNone(receipt(output, case))
        fault = Fault("stale", "played", PLAYED_RUNNER, "controller.held", "stale.held", case, 5, "camera.x")
        for result, killed in [((0, stopped), True), ((1, stopped), False),
                               ((0, report('At(Difference { cut: 6, field: "camera.x" })')), False),
                               ((0, report('At(DataFailure { cut: Some(5), source: "pad" })')), False),
                               ((0, report('At(DataFailure { cut: Some(5), source: "camera.x" })')), True),
                               ((0, report('At(Unsupported { cut: Some(5), dependency: "camera.x" })')), False),
                               ((0, report("End")), False)]:
            with self.subTest(result=result):
                self.assertEqual(detected(fault, result) is not None, killed)

if __name__ == "__main__":
    unittest.main()
