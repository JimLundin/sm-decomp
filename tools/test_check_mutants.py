"""A compiler failure, absent test or surviving fault cannot count as a kill."""

import io
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from check_mutants import Fault, check_fault, show_scope


class MutantCheck(unittest.TestCase):
    def exercise(self, result: tuple[int, str], test: str = "model::floor") -> bool:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/clean/model/src/lib.rs"
            path.parent.mkdir(parents=True)
            original = b"let speed = gravity;\n"
            path.write_bytes(original)
            fault = Fault("reversed", "model", str(path.relative_to(root)),
                          "speed = gravity", "speed = -gravity", test)
            with patch("check_mutants.run_test", return_value=result):
                killed = check_fault(root, fault)
            self.assertEqual(path.read_bytes(), original)
            return killed

    def test_only_a_failed_named_test_kills(self) -> None:
        self.assertTrue(self.exercise((101, "test model::floor ... FAILED\n")))
        self.assertFalse(self.exercise((101, "")))
        self.assertFalse(self.exercise((0, "test result: ok. 0 passed\n")))
        self.assertFalse(self.exercise((0, "test model::floor ... ok\n")))
        self.assertFalse(self.exercise((101, "test other::floor ... FAILED\n")))

    def test_scope_reports_only_the_named_domains_m0_floor(self) -> None:
        self.assertTrue(show_scope("FLOOR free model chain-any-0003-DFD7 2 3 FAILED", "model"))
        self.assertTrue(show_scope("FLOOR free model chain-any-0003-DFD7 3 3 ok", "model"))
        self.assertFalse(show_scope("FLOOR free other chain-any-0003-DFD7 2 3 FAILED", "model"))
        self.assertFalse(show_scope("FLOOR free model another-row 2 3 FAILED", "model"))

    def test_harness_without_m0_evidence_is_invalid(self) -> None:
        with redirect_stdout(io.StringIO()) as output:
            self.assertFalse(self.exercise((101, "test harness::floor ... FAILED\n"), "harness::floor"))
        self.assertIn("INVALID RUN", output.getvalue())

    def test_accounting_only_failure_cannot_kill(self) -> None:
        result = (101, "FLOOR free model chain-any-0003-DFD7 3 3 ok\n"
                  "test harness::floor ... FAILED\n")
        self.assertFalse(self.exercise(result, "harness::floor"))

    def test_a_passing_harness_reports_a_surviving_fault(self) -> None:
        result = (0, "FLOOR free model chain-any-0003-DFD7 3 3 ok\n"
                  "test harness::floor ... ok\n")
        with redirect_stdout(io.StringIO()) as output:
            self.assertFalse(self.exercise(result, "harness::floor"))
        self.assertIn("SURVIVED model reversed: harness::floor", output.getvalue())
        self.assertNotIn("INVALID RUN", output.getvalue())

    def test_another_rows_failure_does_not_change_m0s_status(self) -> None:
        result = (101, "FLOOR free model chain-any-0003-DFD7 3 3 ok\n"
                  "FLOOR free model other-row 2 3 FAILED\n"
                  "test harness::floor ... FAILED\n")
        with redirect_stdout(io.StringIO()) as output:
            self.assertTrue(self.exercise(result, "harness::floor"))
        self.assertIn("chain-any-0003-DFD7 3 3 ok", output.getvalue())
        self.assertNotIn("chain-any-0003-DFD7 3 3 FAILED", output.getvalue())

    def test_every_named_check_must_reject_the_fault(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/clean/model/src/lib.rs"
            path.parent.mkdir(parents=True)
            original = b"let speed = gravity;\n"
            path.write_bytes(original)
            fault = Fault("reversed", "model", str(path.relative_to(root)),
                          "speed = gravity", "speed = -gravity", "model::floor",
                          ["model::free"])
            results = [(101, "test model::floor ... FAILED\n"),
                       (0, "test model::free ... ok\n")]
            with patch("check_mutants.run_test", side_effect=results) as runner:
                self.assertFalse(check_fault(root, fault))
                self.assertEqual(runner.call_count, 2)
            self.assertEqual(path.read_bytes(), original)
            with patch("check_mutants.run_test", side_effect=[results[0], OSError()]):
                with self.assertRaises(OSError):
                    check_fault(root, fault)
            self.assertEqual(path.read_bytes(), original)

    def test_source_is_restored_when_the_runner_raises(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/clean/model/src/lib.rs"
            path.parent.mkdir(parents=True)
            original = b"let speed = gravity;\n"
            path.write_bytes(original)
            fault = Fault("reversed", "model", str(path.relative_to(root)),
                          "speed = gravity", "speed = -gravity", "model::floor")
            with patch("check_mutants.run_test", side_effect=OSError):
                with self.assertRaises(OSError):
                    check_fault(root, fault)
            self.assertEqual(path.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
