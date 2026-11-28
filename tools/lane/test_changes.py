import contextlib
import io
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import changes

KINDS = {
    "docs only": (["docs/STATUS.md", "docs/LEGACY.md", ".agents/skills/lane/SKILL.md"], set()),
    "lane tools": (["tools/lane/land.py", "tools/lane/test_land.py"], {"python"}),
    "CI tools": (["tools/ci/replay.py"], {"python", "rom"}),
    "planted faults": (["tools/mutants.toml"], {"python", "rom"}),
    "a clean model": (["crates/clean/samus_movement/src/lib.rs"], {"rust", "python", "rom", "adjusted", "engine"}),
    "a behaviour spec": (["docs/behaviours/samus-movement.md"], {"rust", "python", "adjusted"}),
    "the engine": (["crates/runtime/src/lib.rs"], {"rust", "python", "rom", "engine"}),
    "the app": (["crates/app/src/main.rs"], {"rust", "python", "engine"}),
    "the vector reader": (["crates/vector_file/src/lib.rs"], {"rust", "python"}),
    "the engine size report": (["crates/tools/src/engine_size.rs"], {"rust", "python", "rom", "engine"}),
    "the lock file": (["Cargo.lock"], set(changes.GROUPS)),
    "the workflow": ([".github/workflows/ci.yml"], set(changes.GROUPS)),
    "an action": ([".github/actions/cargo-restore/action.yml"], set(changes.GROUPS)),
    "this table": (["tools/lane/changes.py"], set(changes.GROUPS)),
}


def git(repo: Path, *args: str) -> None:
    subprocess.run(["git", "-C", str(repo), *args], capture_output=True, check=True)


class Groups(unittest.TestCase):
    def test_each_kind_of_change_starts_its_groups(self) -> None:
        for kind, (paths, expected) in KINDS.items():
            with self.subTest(kind):
                self.assertEqual({name for name, changed in changes.groups(paths).items() if changed}, expected)

    def test_a_prefix_of_a_folder_name_is_not_that_folder(self) -> None:
        self.assertFalse(changes.groups(["cratesmith.md", "docs/behaviours-old.md"])["rust"])

    def test_without_a_base_every_group_runs(self) -> None:
        self.assertEqual(changes.groups(None), dict.fromkeys(changes.GROUPS, True))


class ChangedFiles(unittest.TestCase):
    def test_committed_staged_and_untracked_files_count(self) -> None:
        repo = Path(self.enterContext(tempfile.TemporaryDirectory(dir=os.environ.get("TEMP"))))
        git(repo, "init", "-q", "-b", "main")
        for name in ("docs/a.md", "tools/ci/b.py", "crates/app/c.rs"):
            (repo / name).parent.mkdir(parents=True, exist_ok=True)
            (repo / name).write_text("before\n")
        git(repo, "add", ".")
        git(repo, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "base")
        git(repo, "branch", "base")
        (repo / "docs/a.md").write_text("after\n")
        git(repo, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-am", "docs")
        (repo / "tools/ci/b.py").write_text("after\n")
        (repo / "crates/app/new.rs").write_text("new\n")
        out = io.StringIO()
        with contextlib.chdir(repo), contextlib.redirect_stdout(out):
            self.assertEqual(sorted(changes.changed_files("base")), ["crates/app/new.rs", "docs/a.md", "tools/ci/b.py"])
            self.assertEqual(changes.main(["base"]), 0)
        self.assertEqual(out.getvalue().split(), ["rust=true", "python=true", "rom=true", "adjusted=false",
                                                  "engine=true"])


if __name__ == "__main__":
    unittest.main()
