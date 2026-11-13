"""A compiler failure, absent test or surviving fault cannot count as a kill."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from check_mutants import Fault, check_fault


class MutantCheck(unittest.TestCase):
    def exercise(self, result: tuple[int, str]) -> bool:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/clean/model/src/lib.rs"
            path.parent.mkdir(parents=True)
            original = b"let speed = gravity;\n"
            path.write_bytes(original)
            fault = Fault("reversed", "model", str(path.relative_to(root)),
                          "speed = gravity", "speed = -gravity", "model::floor")
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
