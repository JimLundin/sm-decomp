"""Bridge fault credit requires an exact recorded difference and restored Pass."""
import tempfile
import tomllib
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch
from check_mutants import Fault, check_fault, receipt, source_path, validate_coverage

def outcome(status: str, case: str = "audio/example", cut: int = 3, field: str = "audio.reply0") -> str:
    text = f'format = 1\n[[cases]]\nid = "{case}"\nobservation_sha256 = "original"\n[cases.outcome]\nstatus = "{status}"\n'
    if status == "difference":
        text += f'cut = {cut}\nfield = "{field}"\n'
    return text + '[cases.counts]\ncuts = 3\nrestorations = 2\n'

class MutantCheck(unittest.TestCase):
    def exercise(self, result: tuple[int, str], restored: tuple[int, str] | None = None) -> bool:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/game/src/production.rs"
            path.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            path.write_bytes(original)
            fault = Fault("zero", "audio", str(path.relative_to(root)), "measured", "zero", "audio/example", 3, "audio.reply0")
            with patch("check_mutants.run_case", side_effect=[result, restored or (0, outcome("pass"))]):
                killed = check_fault(root, fault, receipt(outcome("pass"), "audio/example"))
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
                    check_fault(root, fault, receipt(outcome("pass"), "audio/example"))
            self.assertEqual(path.read_bytes(), original)

if __name__ == "__main__":
    unittest.main()
