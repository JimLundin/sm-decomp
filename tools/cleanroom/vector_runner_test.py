"""The vector runner's wiring on code_check_fixtures; no cargo, no test runs."""

import contextlib
import io
import pathlib
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import vector_runner  # noqa: E402

FIXTURES = HERE / "code_check_fixtures"
# The fixtures of the two rules the runner checks, named TAG__variant, and the clean one.
SHAPES = sorted(p for p in FIXTURES.iterdir() if p.name.split("__")[0] in ("argument-object", "layering"))


class Runner(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        # A lane directory: the scan scripts and the awk pinned beside the runner.
        self.lane = pathlib.Path(directory.name)
        for name in vector_runner.SHAPE_SCAN:
            shutil.copyfile(HERE / name, self.lane / name)
        awk = shutil.which("awk")
        assert awk is not None, "the test needs an awk on PATH"
        self.pin(str(pathlib.Path(awk).resolve()))
        patcher = patch.object(vector_runner, "HERE", self.lane)
        patcher.start()
        self.addCleanup(patcher.stop)

    def pin(self, awk: str) -> None:
        (self.lane / vector_runner.AWK_PIN).write_text(awk + "\n", encoding="utf-8", newline="\n")

    def run_in(self, workspace: pathlib.Path) -> tuple[int, str, list[str]]:
        """The runner's status, output and the targets it ran, each target passing."""
        crates = sorted(p for p in (workspace / "crates/clean").iterdir() if p.is_dir())
        targets = [(crate.name, "--lib", "", crate) for crate in crates]
        ran: list[str] = []

        def run_target(_cargo: str, _workspace: pathlib.Path,
                       target: tuple[str, str, str, pathlib.Path]) -> None:
            ran.append(target[0])

        output = io.StringIO()
        with patch.object(vector_runner, "clean_targets", return_value=targets), \
                patch.object(vector_runner, "run_target", side_effect=run_target), \
                contextlib.redirect_stdout(output):
            status = vector_runner.run("cargo", workspace)
        return status, output.getvalue(), ran

    def test_a_shape_finding_fails_the_run_and_the_tests_still_run(self) -> None:
        status, output, ran = self.run_in(FIXTURES / "clean")
        self.assertEqual(status, 0, output)
        self.assertEqual(ran, ["demo", "types"])
        self.assertTrue(SHAPES)
        for fixture in SHAPES:
            with self.subTest(fixture=fixture.name):
                status, output, ran = self.run_in(fixture)
                self.assertEqual(status, 1, output)
                self.assertIn(f": {fixture.name.split('__')[0]}: ", output)
                self.assertTrue(ran)

    def test_an_awk_not_pinned_outside_the_workspace_refuses_before_any_test(self) -> None:
        workspace = FIXTURES / "clean"
        # The awk a session could plant: a file in the export's working directory.
        planted = workspace / "crates/clean/demo/src/lib.rs"
        for pin in (None, "", "awk", str(self.lane / "no-such-awk"), str(planted)):
            with self.subTest(pin=pin):
                if pin is None:
                    (self.lane / vector_runner.AWK_PIN).unlink(missing_ok=True)
                else:
                    self.pin(pin)
                with patch.object(vector_runner, "clean_targets") as targets, \
                        patch.object(vector_runner, "run_target") as run_target, \
                        self.assertRaises(SystemExit) as refused:
                    vector_runner.run("cargo", workspace)
                self.assertNotIn(refused.exception.code, (0, None))
                targets.assert_not_called()
                run_target.assert_not_called()


if __name__ == "__main__":
    unittest.main()
