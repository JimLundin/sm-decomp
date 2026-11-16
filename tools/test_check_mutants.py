"""Bridge fault credit requires an exact recorded difference and restored Pass."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from check_mutants import Fault, check_fault, source_path

def outcome(status: str, case: str = "audio/example", cut: int = 3, field: str = "audio.reply0") -> str:
    text = f'format = 1\n[[cases]]\nid = "{case}"\n[cases.outcome]\nstatus = "{status}"\n'
    if status == "difference":
        text += f'cut = {cut}\nfield = "{field}"\n'
    return text

class MutantCheck(unittest.TestCase):
    def exercise(self, result: tuple[int, str], restored: tuple[int, str] | None = None) -> bool:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/game/src/production.rs"
            path.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            path.write_bytes(original)
            fault = Fault("zero", str(path.relative_to(root)), "measured", "zero", "audio/example", 3, "audio.reply0")
            with patch("check_mutants.run_case", side_effect=[result, restored or (0, outcome("pass"))]):
                killed = check_fault(root, fault)
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

    def test_source_restores_after_runner_exception(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/game/src/production.rs"
            path.parent.mkdir(parents=True)
            original = b"let elapsed = measured;\n"
            path.write_bytes(original)
            fault = Fault("zero", str(path.relative_to(root)), "measured", "zero", "audio/example", 3, "audio.reply0")
            with patch("check_mutants.run_case", side_effect=OSError):
                with self.assertRaises(OSError):
                    check_fault(root, fault)
            self.assertEqual(path.read_bytes(), original)

if __name__ == "__main__":
    unittest.main()
