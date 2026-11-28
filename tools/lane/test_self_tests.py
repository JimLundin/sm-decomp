import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
PYTHON_SUITES = {name: f"tools/lane/test_{name}.py" for name in ("worktree", "gate", "self_tests")} | {
    "adjusted_vectors": "tools/cleanroom/test_adjusted_vectors_run.py"}
EVERY_SUITE = sorted(PYTHON_SUITES)
PYTHON_SUITE = """import os
import pathlib
import unittest


class Ran(unittest.TestCase):
    def test_ran(self) -> None:
        pathlib.Path(os.environ["RAN"], "{name}").touch()


unittest.main()
"""


class SelfTestSelection(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.repo = self.scratch / "repo"
        (self.repo / "tools/lane").mkdir(parents=True)
        (self.repo / "tools/cleanroom").mkdir()
        for name in ("changes.sh", "self_tests.sh"):
            shutil.copy(HERE / name, self.repo / "tools/lane" / name)
        (self.repo / ".gitignore").write_text("__pycache__/\n")
        for name, suite in PYTHON_SUITES.items():
            (self.repo / suite).write_text(PYTHON_SUITE.format(name=name))
        for path in ("tools/lane/gate.sh", "tools/lane/land.py", "tools/cleanroom/adjusted_vectors_run.py",
                     "tools/cleanroom/box_relay.py"):
            (self.repo / path).touch()
        self.git("init", "-q")
        for setting in ("user.name=Fixture", "user.email=fixture@example.invalid", "commit.gpgsign=false",
                        "status.showUntrackedFiles=no"):
            self.git("config", *setting.split("="))
        self.git("add", "-A")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD")

    def git(self, *args: str) -> str:
        return subprocess.run(["git", *args], cwd=self.repo, check=True, capture_output=True, text=True).stdout.strip()

    def append(self, path: str) -> None:
        file = self.repo / path
        file.parent.mkdir(parents=True, exist_ok=True)
        with file.open("a") as text:
            text.write("edited\n")

    def run_selector(self, *base: str) -> list[str]:
        ran = Path(tempfile.mkdtemp(dir=self.scratch))
        result = subprocess.run(["bash", "tools/lane/self_tests.sh", *base], cwd=self.repo,
                                env=os.environ | {"RAN": str(ran)}, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return sorted(record.name for record in ran.iterdir())

    def test_an_unchanged_branch_runs_nothing(self) -> None:
        self.assertEqual(self.run_selector("HEAD"), [])

    def test_a_subject_edit_runs_its_suite_staged_unstaged_or_committed(self) -> None:
        self.append("tools/lane/gate.sh")
        self.assertEqual(self.run_selector("HEAD"), ["gate"])
        self.git("add", "tools/lane/gate.sh")
        self.assertEqual(self.run_selector("HEAD"), ["gate"])
        (self.repo / "tools/lane/gate.sh").write_text(self.git("show", "HEAD:tools/lane/gate.sh"))
        self.assertEqual(self.run_selector("HEAD"), ["gate"])
        self.git("restore", "--staged", "--worktree", "tools/lane/gate.sh")
        self.append("tools/lane/gate.sh")
        self.git("commit", "-qam", "Edit the gate")
        self.assertEqual(self.run_selector(self.base), ["gate"])

    def test_a_tool_with_its_own_tests_runs_no_suite(self) -> None:
        for path in ("tools/lane/land.py", "tools/cleanroom/box_relay.py"):
            with self.subTest(path=path):
                self.append(path)
                self.assertEqual(self.run_selector("HEAD"), [])
                self.git("restore", "--", path)

    def test_a_package_marker_runs_no_suite(self) -> None:
        self.append("tools/lane/__init__.py")
        self.append("tools/cleanroom/__init__.py")
        self.assertEqual(self.run_selector("HEAD"), [])

    def test_an_adjusted_vector_input_runs_its_suite(self) -> None:
        self.append("tools/cleanroom/adjusted_vectors_run.py")
        self.assertEqual(self.run_selector("HEAD"), ["adjusted_vectors"])
        self.append("tools/lane/gate.sh")
        self.git("commit", "-qam", "Edit the gate and the adjusted-vector prover")
        self.assertEqual(self.run_selector(self.base), ["adjusted_vectors", "gate"])

    def test_an_unknown_lane_file_runs_every_suite(self) -> None:
        self.git("mv", "tools/lane/gate.sh", "tools/lane/renamed")
        self.assertEqual(self.run_selector("HEAD"), EVERY_SUITE)

    def test_a_subject_moved_out_of_its_folder_still_runs_its_suite(self) -> None:
        self.git("mv", "tools/lane/gate.sh", "removed-gate.sh")
        self.assertEqual(self.run_selector("HEAD"), ["gate"])

    def test_no_base_runs_every_suite(self) -> None:
        self.assertEqual(self.run_selector(), EVERY_SUITE)

    def test_a_base_that_does_not_resolve_fails(self) -> None:
        result = subprocess.run(["bash", "tools/lane/self_tests.sh", "missing-branch"], cwd=self.repo,
                                capture_output=True, text=True, check=False)
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
