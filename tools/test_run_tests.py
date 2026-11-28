import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

RUNNER = Path(__file__).with_name("run_tests.py")
PASSING = """import unittest


class Passes(unittest.TestCase):
    def test_one(self) -> None:
        pass

    def test_two(self) -> None:
        pass
"""
FAILING = """import unittest


class Fails(unittest.TestCase):
    def test_breaks(self) -> None:
        self.fail("planted")
"""
DISCOVERED_ONLY = """import unittest


class Discovered(unittest.TestCase):
    def test_only_under_discovery(self) -> None:
        pass


def load_tests(loader: unittest.TestLoader, tests: unittest.TestSuite, pattern: str | None) -> unittest.TestSuite:
    return tests if pattern else unittest.TestSuite()
"""
DYING = """import os
import unittest


class Dies(unittest.TestCase):
    def test_exits(self) -> None:
        os._exit(3)
"""


class RunTests(unittest.TestCase):
    def run_folder(self, modules: dict[str, str]) -> subprocess.CompletedProcess[str]:
        folder = Path(self.enterContext(tempfile.TemporaryDirectory()))
        (folder / "nested").mkdir()
        (folder / "nested" / "__init__.py").touch()
        for name, source in modules.items():
            (folder / name).write_text(source)
        return subprocess.run([sys.executable, "-I", str(RUNNER), str(folder)], capture_output=True, text=True,
                              check=False)

    def test_every_module_runs_and_the_counts_match(self) -> None:
        run = self.run_folder({"test_first.py": PASSING, "nested/test_second.py": PASSING})
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn("ran 4 of 4 tests in 2 modules", run.stdout)

    def test_a_failure_names_its_test(self) -> None:
        run = self.run_folder({"test_first.py": PASSING, "nested/test_broken.py": FAILING})
        self.assertEqual(run.returncode, 1)
        self.assertIn("ran 3 of 3 tests", run.stdout)
        self.assertIn("FAILED nested.test_broken.Fails.test_breaks", run.stdout.splitlines())

    def test_tests_that_never_ran_fail_the_run(self) -> None:
        run = self.run_folder({"test_first.py": PASSING, "test_discovered.py": DISCOVERED_ONLY})
        self.assertEqual(run.returncode, 1)
        self.assertIn("ran 2 of 3 tests", run.stdout)

    def test_a_module_whose_process_dies_is_named(self) -> None:
        run = self.run_folder({"test_first.py": PASSING, "test_dying.py": DYING})
        self.assertEqual(run.returncode, 1)
        self.assertIn("ran 2 of 3 tests", run.stdout)
        self.assertIn("FAILED test_dying (its process exited 3)", run.stdout.splitlines())


if __name__ == "__main__":
    unittest.main()
