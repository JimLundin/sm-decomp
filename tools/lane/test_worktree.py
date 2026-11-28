import os
import subprocess
import tempfile
import unittest
from pathlib import Path

WORKTREE = Path(__file__).resolve().with_name("worktree.sh")
IDENTITY = {"GIT_AUTHOR_NAME": "worktree-check", "GIT_AUTHOR_EMAIL": "worktree-check@localhost",
            "GIT_COMMITTER_NAME": "worktree-check", "GIT_COMMITTER_EMAIL": "worktree-check@localhost"}
LOCKS = (".cargo-lock", ".cargo-build-lock", ".cargo-artifact-lock")
DEP_INFO = ("deps/x.d", ".fingerprint/x-1/dep-lib-x")


def linked(a: Path, b: Path) -> bool:
    return a.stat().st_ino == b.stat().st_ino


def newer(a: Path, b: Path) -> bool:
    return a.stat().st_mtime_ns > b.stat().st_mtime_ns


class WorktreeAdd(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(self.enterContext(tempfile.TemporaryDirectory(dir=os.environ.get("TEMP"))))
        self.repo = self.tmp / "repo"
        self.repo.mkdir()
        self.seed = self.tmp / "seed"
        self.env = os.environ | IDENTITY | {"SM_SEED_DIR": str(self.seed)}
        self.git("init", "-q", "-b", "main", ".")
        self.git("config", "commit.gpgsign", "false")
        for name, text in {".gitignore": "target/", "same.rs": "same", "changed.rs": "old", "deleted.rs": "gone"}.items():
            (self.repo / name).write_text(text + "\n")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "seed")
        self.git("worktree", "add", "-q", "--detach", str(self.seed), "HEAD")
        self.built = self.seed / "target/debug"
        (self.built / "deps").mkdir(parents=True)
        (self.built / ".fingerprint/x-1").mkdir(parents=True)
        (self.seed / "target/release").mkdir()
        (self.built / "deps/x.rlib").write_text("built\n")
        for info in DEP_INFO:
            (self.built / info).write_text("info\n")
        (self.seed / "target/release/out").write_text("shipped\n")
        for lock in LOCKS:
            (self.built / lock).touch()
        (self.repo / "changed.rs").write_text("new\n")
        self.git("rm", "-q", "deleted.rs")
        (self.repo / "added.rs").write_text("added\n")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")

    def git(self, *args: str, cwd: Path | None = None) -> str:
        return subprocess.run(["git", *args], cwd=cwd or self.repo, env=self.env, check=True,
                              capture_output=True, text=True).stdout.strip()

    def add(self, lane: str, base: str, **env: str) -> str:
        return subprocess.run(["bash", str(WORKTREE), "add", lane, str(self.tmp / lane), base], cwd=self.repo,
                              env=self.env | env, check=True, capture_output=True, text=True).stdout

    def test_an_unbuilt_seed_starts_cold(self) -> None:
        self.assertIn("first build is cold", self.add("cold", "HEAD"))
        self.assertFalse((self.tmp / "cold/target").exists())

    def test_a_lane_links_the_seed_build(self) -> None:
        self.check_lane("linked", {})

    def test_sm_seed_copy_copies_the_seed_build(self) -> None:
        self.check_lane("copied", {"SM_SEED_COPY": "1"})

    def check_lane(self, lane: str, env: dict[str, str]) -> None:
        (self.seed / "target/.seeded").touch()
        self.assertIn(f"{lane} the build", self.add(lane, "main", **env))
        path = self.tmp / lane
        built = path / "target/debug"
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=path), self.git("rev-parse", "main"))
        self.assertEqual((built / "deps/x.rlib").read_text(), "built\n")
        self.assertFalse((path / "target/release").exists())
        self.assertEqual(linked(built / "deps/x.rlib", self.built / "deps/x.rlib"), lane == "linked")
        for lock in LOCKS:
            self.assertTrue((self.built / lock).is_file(), f"the seed's {lock} is missing")
            (built / lock).touch()
            self.assertFalse(linked(built / lock, self.built / lock), f"{lock} is shared")
        for info in DEP_INFO:
            self.assertEqual((built / info).read_text(), "info\n")
            self.assertFalse(linked(built / info, self.built / info), f"{info} is shared")
            (built / info).write_text("rewritten\n")
            self.assertEqual((self.built / info).read_text(), "info\n")
        self.assertTrue(newer(path / "changed.rs", built / "deps/x.rlib"), "a changed source looks built")
        self.assertTrue(newer(path / "added.rs", built / "deps/x.rlib"), "an added source looks built")
        self.assertFalse(newer(path / "same.rs", built / "deps/x.rlib"), "an unchanged source looks changed")
        self.assertEqual(self.git("status", "--porcelain", cwd=path), "")


if __name__ == "__main__":
    unittest.main()
