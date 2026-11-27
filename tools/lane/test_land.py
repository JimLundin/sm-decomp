import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import land

GATE_OK = [sys.executable, "-c", "print('GATE OK x y')"]


class FakeGitHub:
    def __init__(
        self, repo: Path, creates_after: int = 0, ci: str = "success", stale: int = 0, failures: int = 0
    ) -> None:
        self.repo = repo
        self.creates_after = creates_after
        self.state = ci
        self.stale = stale
        self.failures = failures
        self.prs: dict[str, int] = {}
        self.comments: list[int] = []

    def open_pr(self, branch: str) -> int | None:
        return self.prs.get(branch)

    def create_pr(self, branch: str, title: str, body_file: Path) -> None:
        if self.creates_after:
            self.creates_after -= 1
            return
        self.prs[branch] = len(self.prs) + 1

    def pr_head(self, pr: int) -> str:
        if self.failures:
            self.failures -= 1
            raise land.Failed("gh pr view: connection reset")
        if self.stale:
            self.stale -= 1
            return "0" * 40
        branch = next(b for b, n in self.prs.items() if n == pr)
        return land.git(self.repo, "rev-parse", f"origin/{branch}")

    def ci(self, branch: str, head: str) -> str:
        return self.state if land.git(self.repo, "rev-parse", f"origin/{branch}") == head else "pending"

    def comment(self, pr: int, body: str) -> None:
        self.comments.append(pr)


class LandTest(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        values = {
            "TEMP": str(self.root),
            "SM_SEED_DIR": str(self.root / "no-seed"),
            "GIT_AUTHOR_NAME": "fixture",
            "GIT_AUTHOR_EMAIL": "fixture@invalid",
            "GIT_COMMITTER_NAME": "fixture",
            "GIT_COMMITTER_EMAIL": "fixture@invalid",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
        }
        self.enterContext(mock.patch.dict(os.environ, values))
        self.origin = self.root / "origin.git"
        self.repo = self.root / "repo"
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(self.origin)], check=True)
        subprocess.run(["git", "init", "-q", "-b", "main", str(self.repo)], check=True)
        self.commit(self.repo, "base.txt")
        land.git(self.repo, "remote", "add", "origin", str(self.origin))
        land.git(self.repo, "push", "-q", "-u", "origin", "main")
        land.git(self.repo, "fetch", "-q", "origin")

    def commit(self, path: Path, name: str) -> str:
        (path / name).write_text(name + "\n", encoding="utf-8")
        land.git(path, "add", name)
        land.git(path, "commit", "-q", "-m", name)
        return land.git(path, "rev-parse", "HEAD")

    def lane(self, branch: str, start: str, *files: str) -> Path:
        path = self.root / branch.replace("/", "-")
        land.git(self.repo, "worktree", "add", "-q", "-b", branch, str(path), start)
        for name in files:
            self.commit(path, name)
        land.git(path, "push", "-q", "origin", branch)
        return path

    def run_land(
        self,
        name: str,
        lanes: tuple[str, ...],
        github: FakeGitHub,
        *,
        after: str | None = None,
        dry_run: bool = False,
        gate: list[str] = GATE_OK,
    ) -> tuple[int, str]:
        title = None if dry_run else "Title"
        landing = land.Landing(self.repo, name, lanes, after, title, self.root / "body.md")
        out = io.StringIO()
        code = 0
        with contextlib.redirect_stdout(out):
            try:
                land.land(landing, github, gate, lambda: None)
            except land.Failed as error:
                print(error)
                code = error.code
        return code, out.getvalue()

    def main_tip(self) -> str:
        return land.git(self.origin, "rev-parse", "main")

    def test_lands_lanes_and_removes_only_landed_ones(self) -> None:
        first = self.lane("lane/a", "main", "a1.txt", "a2.txt")
        stacked = self.lane("lane/b", "lane/a", "b.txt")
        dirty = self.lane("lane/c", "main", "c.txt")
        (dirty / "notes.txt").write_text("unsaved\n", encoding="utf-8")
        moved = self.lane("lane/d", "main", "d.txt")
        github = FakeGitHub(self.repo)
        self.commit(moved, "unpushed.txt")
        code, out = self.run_land("batch", ("lane/a", "lane/b", "lane/c", "lane/d"), github)
        self.assertEqual(code, 0, out)
        landed = land.git(self.origin, "log", "--format=%s", "main").split()
        self.assertEqual(landed, ["d.txt", "c.txt", "b.txt", "a2.txt", "a1.txt", "base.txt"])
        self.assertEqual(github.comments, [1])
        self.assertFalse(first.exists() or stacked.exists())
        self.assertTrue(dirty.exists() and moved.exists())
        self.assertIn(f"kept {dirty}: it has uncommitted changes", out)
        self.assertIn("kept lane/d", out)
        branches = land.git(self.repo, "for-each-ref", "--format=%(refname)", "refs/heads", "refs/remotes")
        self.assertEqual(
            sorted(branches.split()),
            [
                "refs/heads/lane/c",
                "refs/heads/lane/d",
                "refs/heads/main",
                "refs/remotes/origin/lane/c",
                "refs/remotes/origin/main",
            ],
        )

    def test_a_conflict_stops_and_a_rerun_continues_after_its_resolution(self) -> None:
        one = self.lane("lane/one", "main", "same.txt")
        (one / "same.txt").write_text("one\n", encoding="utf-8")
        land.git(one, "commit", "-qam", "one edits")
        land.git(one, "push", "-q", "origin", "lane/one")
        two = self.lane("lane/two", "main")
        (two / "same.txt").write_text("two\n", encoding="utf-8")
        land.git(two, "add", "same.txt")
        land.git(two, "commit", "-qm", "two adds")
        self.commit(two, "after.txt")
        land.git(two, "push", "-q", "origin", "lane/two")
        github = FakeGitHub(self.repo)
        code, out = self.run_land("conflict", ("lane/one", "lane/two"), github)
        self.assertEqual(code, 2, out)
        self.assertIn("same.txt", out)
        worktree = self.root / "collab" / "land-conflict"
        (worktree / "same.txt").write_text("both\n", encoding="utf-8")
        land.git(worktree, "add", "same.txt")
        land.git(worktree, "-c", "core.editor=true", "cherry-pick", "--continue")
        code, out = self.run_land("conflict", ("lane/one", "lane/two"), github)
        self.assertEqual(code, 0, out)
        subjects = land.git(self.origin, "log", "--format=%s", "main").splitlines()
        self.assertEqual(subjects, ["after.txt", "two adds", "one edits", "same.txt", "base.txt"])
        self.assertIn("kept lane/two", out)

    def test_retries_the_pr_and_refuses_when_none_opens(self) -> None:
        self.lane("lane/a", "main", "a.txt")
        code, out = self.run_land("late", ("lane/a",), FakeGitHub(self.repo, creates_after=2))
        self.assertEqual(code, 0, out)
        before = self.main_tip()
        self.lane("lane/b", "main", "b.txt")
        code, out = self.run_land("never", ("lane/b",), FakeGitHub(self.repo, creates_after=land.PR_TRIES))
        self.assertEqual(code, 1, out)
        self.assertIn("no PR", out)
        self.assertEqual(self.main_tip(), before)

    def test_a_red_run_or_a_moved_main_lands_nothing(self) -> None:
        self.lane("lane/a", "main", "a.txt")
        before = self.main_tip()
        code, out = self.run_land("red", ("lane/a",), FakeGitHub(self.repo, ci="failure"))
        self.assertEqual((code, self.main_tip()), (4, before), out)
        self.lane("lane/b", "main", "b.txt")
        github = FakeGitHub(self.repo)
        land.git(self.repo, "fetch", "-q", "origin")
        land.pick(land.Landing(self.repo, "moved", ("lane/b",), None))
        moved = self.lane("lane/other", "main", "other.txt")
        land.git(moved, "push", "-q", "origin", "HEAD:main")
        code, out = self.run_land("moved", ("lane/b",), github)
        self.assertEqual(code, 3, out)

    def test_after_builds_on_an_earlier_landing_and_waits_for_it(self) -> None:
        self.lane("lane/a", "main", "a.txt")
        self.lane("lane/b", "lane/a", "b.txt")
        land.git(self.repo, "fetch", "-q", "origin")
        earlier = land.git(self.repo, "rev-parse", "origin/lane/a")
        before = self.main_tip()
        code, out = self.run_land("stacked", ("lane/b",), FakeGitHub(self.repo), after=earlier)
        self.assertEqual((code, self.main_tip()), (5, before), out)
        self.assertIn(f"waits for {earlier[:9]}", out)
        land.git(self.repo, "push", "-q", "origin", f"{earlier}:main")
        code, out = self.run_land("stacked", ("lane/b",), FakeGitHub(self.repo), after=earlier)
        self.assertEqual(code, 0, out)
        self.assertEqual(land.git(self.origin, "log", "-1", "--format=%s", "main"), "b.txt")

    def test_a_dry_run_pushes_nothing_and_names_what_landing_would_remove(self) -> None:
        self.lane("lane/a", "main", "a.txt")
        before = land.git(self.origin, "for-each-ref")
        code, out = self.run_land("dry", ("lane/a",), FakeGitHub(self.repo), dry_run=True)
        self.assertEqual(code, 0, out)
        self.assertEqual(land.git(self.origin, "for-each-ref"), before)
        self.assertIn("would remove origin/lane/a", out)
        self.assertTrue((self.root / "lane-a").exists())

    def test_ci_state_reads_only_the_pull_request_run_on_the_head(self) -> None:
        def runs(*rows: tuple[str, str, str, str]) -> str:
            keys = ("headSha", "status", "conclusion", "event")
            return json.dumps([dict(zip(keys, row)) for row in rows])

        self.assertEqual(land.ci_state(runs(), "h"), "pending")
        self.assertEqual(land.ci_state(runs(("old", "completed", "success", "pull_request")), "h"), "pending")
        self.assertEqual(land.ci_state(runs(("h", "completed", "success", "push")), "h"), "pending")
        self.assertEqual(land.ci_state(runs(("h", "in_progress", "", "pull_request")), "h"), "pending")
        newest = runs(("h", "completed", "cancelled", "pull_request"), ("h", "completed", "success", "pull_request"))
        self.assertEqual(land.ci_state(newest, "h"), "cancelled")

    def test_waits_through_a_stale_head_and_transient_gh_failures(self) -> None:
        self.lane("lane/a", "main", "a.txt")
        code, out = self.run_land("slow", ("lane/a",), FakeGitHub(self.repo, stale=2, failures=2))
        self.assertEqual(code, 0, out)
        self.assertIn("waits for GitHub to show", out)
        self.assertIn("is not ready: gh pr view: connection reset", out)
        self.assertEqual(land.git(self.origin, "log", "-1", "--format=%s", "main"), "a.txt")

    def test_a_rerun_after_landing_goes_straight_to_cleanup(self) -> None:
        path = self.lane("lane/a", "main", "a.txt")
        with mock.patch.object(land, "clean", side_effect=land.Failed("interrupted")):
            code, out = self.run_land("again", ("lane/a",), FakeGitHub(self.repo))
        self.assertEqual(code, 1, out)
        landed = self.main_tip()
        failing_gate = [sys.executable, "-c", "raise SystemExit(1)"]
        code, out = self.run_land("again", ("lane/a",), FakeGitHub(self.repo), gate=failing_gate)
        self.assertEqual(code, 0, out)
        self.assertIn(f"land/again at {landed[:9]} is on main", out)
        self.assertFalse(path.exists() or (self.root / "collab" / "land-again").exists())
        self.assertEqual(land.git(self.repo, "branch", "--list", "lane/a", "land/again"), "")

    def test_skips_a_pick_main_already_has(self) -> None:
        resolved = self.lane("lane/z", "main", "z.txt")
        land.git(self.repo, "cherry-pick", "lane/z")
        (self.repo / "z.txt").write_text("resolved\n", encoding="utf-8")
        land.git(self.repo, "commit", "-q", "--amend", "-a", "--no-edit")
        land.git(self.repo, "push", "-q", "origin", "main")
        both, one = self.lane("lane/a", "main"), self.lane("lane/b", "main")
        for path, names in ((both, ("x.txt", "y.txt")), (one, ("x.txt",))):
            for name in names:
                (path / name).write_text("same\n", encoding="utf-8")
            land.git(path, "add", *names)
            land.git(path, "commit", "-qm", " ".join(names))
            land.git(path, "push", "-q", "origin", path.name.replace("-", "/"))
        code, out = self.run_land("empty", ("lane/z", "lane/a", "lane/b"), FakeGitHub(self.repo))
        self.assertEqual(code, 0, out)
        self.assertNotIn("lane/z", "".join(line for line in out.splitlines() if line.startswith("picked")))
        self.assertIn("lane/b: main already has its change", out)
        subjects = land.git(self.origin, "log", "--format=%s", "main").splitlines()
        self.assertEqual(subjects, ["x.txt y.txt", "z.txt", "base.txt"])
        self.assertTrue(resolved.exists())

    def test_ignored_files_outside_target_keep_a_worktree(self) -> None:
        kept = self.lane("lane/a", "main", "a.txt")
        removed = self.lane("lane/b", "main", "b.txt")
        (self.repo / ".git" / "info" / "exclude").write_text("*.sfc\n/target\n__pycache__/\n", encoding="utf-8")
        (kept / "game.sfc").write_bytes(b"rom")
        for path in (removed / "target" / "debug" / "out", removed / "tools" / "__pycache__" / "x.pyc"):
            path.parent.mkdir(parents=True)
            path.write_bytes(b"build")
        code, out = self.run_land("ignored", ("lane/a", "lane/b"), FakeGitHub(self.repo))
        self.assertEqual(code, 0, out)
        self.assertIn(f"kept {kept}: it has ignored files outside target/: game.sfc", out)
        self.assertTrue((kept / "game.sfc").exists())
        self.assertFalse(removed.exists())


if __name__ == "__main__":
    unittest.main()
