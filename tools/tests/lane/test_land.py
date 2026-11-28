import json
import os
import sys
from dataclasses import dataclass
lazy from collections.abc import Sequence
lazy from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedDict

from sm_tools.lane.core.land import (
    PR_TRIES,
    LandingError,
    RunListingError,
    ci_state,
)
from sm_tools.lane.shell import land as lane_land
from sm_tools.lane.shell.land import Landing, git, pick
from sm_tools.shell.process import git_out

GATE_OK = FrozenList((sys.executable, "-c", "print('GATE OK x y')"))
FAILING_GATE = FrozenList((sys.executable, "-c", "raise SystemExit(1)"))
ENVIRONMENT = FrozenOrderedDict({
    "GIT_AUTHOR_NAME": "fixture",
    "GIT_AUTHOR_EMAIL": "fixture@invalid",
    "GIT_COMMITTER_NAME": "fixture",
    "GIT_COMMITTER_EMAIL": "fixture@invalid",
    "GIT_CONFIG_GLOBAL": os.devnull,
    "GIT_CONFIG_NOSYSTEM": "1",
})
STALE_HEAD = "0" * 40


class FakeGitHub:
    def __init__(
        self,
        repo: Path,
        *,
        creates_after: int = 0,
        state: str = "success",
        stale: int = 0,
        failures: int = 0,
    ) -> None:
        self.repo: Path = repo
        self.creates_after: int = creates_after
        self.state: str = state
        self.stale: int = stale
        self.failures: int = failures
        self.listing: str | None = None
        self.prs: dict[str, int] = {}
        self.comments: list[int] = []

    def open_pr(self, branch: str) -> int | None:
        return self.prs.get(branch)

    def create_pr(self, branch: str, title: str, body_file: Path) -> None:
        assert title
        assert body_file.name
        if self.creates_after:
            self.creates_after -= 1
            return
        self.prs[branch] = len(self.prs) + 1

    def pr_head(self, pr: int) -> str:
        if self.failures:
            self.failures -= 1
            message = "gh pr view: connection reset"
            raise LandingError(message)
        if self.stale:
            self.stale -= 1
            return STALE_HEAD
        branch = next(b for b, n in self.prs.items() if n == pr)
        return git(self.repo, "rev-parse", f"origin/{branch}")

    def ci(self, branch: str, head: str) -> str:
        if self.listing is not None:
            return ci_state(self.listing, head)
        pushed = git(self.repo, "rev-parse", f"origin/{branch}")
        return self.state if pushed == head else "pending"

    def comment(self, pr: int, body: str) -> None:
        assert body
        self.comments.append(pr)


@dataclass(frozen=True, slots=True)
class Options:
    after: str | None = None
    dry_run: bool = False
    gate: Sequence[str] = GATE_OK


def commit(path: Path, name: str) -> str:
    _ = (path / name).write_text(name + "\n", encoding="utf-8")
    _ = git(path, "add", name)
    _ = git(path, "commit", "-q", "-m", name)
    return git(path, "rev-parse", "HEAD")


@dataclass(frozen=True, slots=True)
class Fixture:
    root: Path
    origin: Path
    repo: Path
    capsys: pytest.CaptureFixture[str]

    def lane(self, branch: str, start: str, *files: str) -> Path:
        path = self.root / branch.replace("/", "-")
        _ = git(
            self.repo, "worktree", "add", "-q", "-b", branch, str(path), start
        )
        for name in files:
            _ = commit(path, name)
        _ = git(path, "push", "-q", "origin", branch)
        return path

    def run_land(
        self,
        name: str,
        lanes: Sequence[str],
        github: FakeGitHub,
        options: Options | None = None,
    ) -> tuple[int, str]:
        chosen = options or Options()
        title = None if chosen.dry_run else "Title"
        landing = Landing(
            self.repo,
            name,
            FrozenList(lanes),
            chosen.after,
            title,
            self.root / "body.md",
        )
        try:
            lane_land.land(landing, github, chosen.gate, lambda: None)
        except LandingError as error:
            return error.code, f"{self.capsys.readouterr().out}{error}\n"
        return 0, self.capsys.readouterr().out

    def main_tip(self) -> str:
        return git(self.origin, "rev-parse", "main")


@pytest.fixture
def fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> Fixture:
    monkeypatch.setenv("TEMP", str(tmp_path))
    monkeypatch.setenv("SM_SEED_DIR", str(tmp_path / "no-seed"))
    for name, value in ENVIRONMENT.items():
        monkeypatch.setenv(name, value)
    origin = tmp_path / "origin.git"
    repo = tmp_path / "repo"
    _ = git_out("init", "-q", "--bare", "-b", "main", str(origin))
    _ = git_out("init", "-q", "-b", "main", str(repo))
    made = Fixture(tmp_path, origin, repo, capsys)
    _ = commit(repo, "base.txt")
    _ = git(repo, "remote", "add", "origin", str(origin))
    _ = git(repo, "push", "-q", "-u", "origin", "main")
    _ = git(repo, "fetch", "-q", "origin")
    return made


def runs(*rows: tuple[str, str, str, str]) -> str:
    keys = ("headSha", "status", "conclusion", "event")
    return json.dumps([dict(zip(keys, row, strict=True)) for row in rows])


def test_lands_lanes_and_removes_only_landed_ones(fixture: Fixture) -> None:
    first = fixture.lane("lane/a", "main", "a1.txt", "a2.txt")
    stacked = fixture.lane("lane/b", "lane/a", "b.txt")
    dirty = fixture.lane("lane/c", "main", "c.txt")
    _ = (dirty / "notes.txt").write_text("unsaved\n", encoding="utf-8")
    moved = fixture.lane("lane/d", "main", "d.txt")
    github = FakeGitHub(fixture.repo)
    _ = commit(moved, "unpushed.txt")
    code, out = fixture.run_land(
        "batch", ("lane/a", "lane/b", "lane/c", "lane/d"), github
    )
    assert code == 0, out
    landed = git(fixture.origin, "log", "--format=%s", "main").split()
    assert landed == ["d.txt", "c.txt", "b.txt", "a2.txt", "a1.txt", "base.txt"]
    assert github.comments == [1]
    assert not (first.exists())
    assert not (stacked.exists())
    assert dirty.exists()
    assert moved.exists()
    assert f"kept {dirty}: it has uncommitted changes" in out
    assert "kept lane/d" in out
    branches = git(
        fixture.repo,
        "for-each-ref",
        "--format=%(refname)",
        "refs/heads",
        "refs/remotes",
    )
    assert sorted(
        ref for ref in branches.split() if not ref.endswith("/HEAD")
    ) == [
        "refs/heads/lane/c",
        "refs/heads/lane/d",
        "refs/heads/main",
        "refs/remotes/origin/lane/c",
        "refs/remotes/origin/main",
    ]


def test_a_conflict_stops_and_a_rerun_continues_after_its_resolution(
    fixture: Fixture,
) -> None:
    one = fixture.lane("lane/one", "main", "same.txt")
    _ = (one / "same.txt").write_text("one\n", encoding="utf-8")
    _ = git(one, "commit", "-qam", "one edits")
    _ = git(one, "push", "-q", "origin", "lane/one")
    two = fixture.lane("lane/two", "main")
    _ = (two / "same.txt").write_text("two\n", encoding="utf-8")
    _ = git(two, "add", "same.txt")
    _ = git(two, "commit", "-qm", "two adds")
    _ = commit(two, "after.txt")
    _ = git(two, "push", "-q", "origin", "lane/two")
    github = FakeGitHub(fixture.repo)
    code, out = fixture.run_land("conflict", ("lane/one", "lane/two"), github)
    assert code == 2, out
    assert "same.txt" in out
    worktree = fixture.root / "collab" / "land-conflict"
    _ = (worktree / "same.txt").write_text("both\n", encoding="utf-8")
    _ = git(worktree, "add", "same.txt")
    _ = git(worktree, "-c", "core.editor=true", "cherry-pick", "--continue")
    code, out = fixture.run_land("conflict", ("lane/one", "lane/two"), github)
    assert code == 0, out
    subjects = git(fixture.origin, "log", "--format=%s", "main").splitlines()
    assert subjects == [
        "after.txt",
        "two adds",
        "one edits",
        "same.txt",
        "base.txt",
    ]
    assert "kept lane/two" in out


def test_retries_the_pr_and_refuses_when_none_opens(fixture: Fixture) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    code, out = fixture.run_land(
        "late", ("lane/a",), FakeGitHub(fixture.repo, creates_after=2)
    )
    assert code == 0, out
    before = fixture.main_tip()
    _ = fixture.lane("lane/b", "main", "b.txt")
    code, out = fixture.run_land(
        "never", ("lane/b",), FakeGitHub(fixture.repo, creates_after=PR_TRIES)
    )
    assert code == 1, out
    assert "no PR" in out
    assert fixture.main_tip() == before


def test_a_red_run_or_a_moved_main_lands_nothing(fixture: Fixture) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    before = fixture.main_tip()
    code, out = fixture.run_land(
        "red", ("lane/a",), FakeGitHub(fixture.repo, state="failure")
    )
    assert (code, fixture.main_tip()) == (4, before), out
    _ = fixture.lane("lane/b", "main", "b.txt")
    github = FakeGitHub(fixture.repo)
    _ = git(fixture.repo, "fetch", "-q", "origin")
    pick(Landing(fixture.repo, "moved", FrozenList(("lane/b",)), None))
    moved = fixture.lane("lane/other", "main", "other.txt")
    _ = git(moved, "push", "-q", "origin", "HEAD:main")
    code, out = fixture.run_land("moved", ("lane/b",), github)
    assert code == 3, out


def test_after_builds_on_an_earlier_landing_and_waits_for_it(
    fixture: Fixture,
) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    _ = fixture.lane("lane/b", "lane/a", "b.txt")
    _ = git(fixture.repo, "fetch", "-q", "origin")
    earlier = git(fixture.repo, "rev-parse", "origin/lane/a")
    before = fixture.main_tip()
    code, out = fixture.run_land(
        "stacked", ("lane/b",), FakeGitHub(fixture.repo), Options(after=earlier)
    )
    assert (code, fixture.main_tip()) == (5, before), out
    assert f"waits for {earlier[:9]}" in out
    _ = git(fixture.repo, "push", "-q", "origin", f"{earlier}:main")
    code, out = fixture.run_land(
        "stacked", ("lane/b",), FakeGitHub(fixture.repo), Options(after=earlier)
    )
    assert code == 0, out
    assert git(fixture.origin, "log", "-1", "--format=%s", "main") == "b.txt"


def test_a_dry_run_pushes_nothing_and_names_what_landing_would_remove(
    fixture: Fixture,
) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    before = git(fixture.origin, "for-each-ref")
    code, out = fixture.run_land(
        "dry", ("lane/a",), FakeGitHub(fixture.repo), Options(dry_run=True)
    )
    assert code == 0, out
    assert git(fixture.origin, "for-each-ref") == before
    assert "would remove origin/lane/a" in out
    assert (fixture.root / "lane-a").exists()


def test_ci_state_reads_only_the_pull_request_run_on_the_head() -> None:
    assert ci_state(runs(), "h") == "pending"
    old = runs(("old", "completed", "success", "pull_request"))
    assert ci_state(old, "h") == "pending"
    pushed = runs(("h", "completed", "success", "push"))
    assert ci_state(pushed, "h") == "pending"
    running = runs(("h", "in_progress", "", "pull_request"))
    assert ci_state(running, "h") == "pending"
    newest = runs(
        ("h", "completed", "cancelled", "pull_request"),
        ("h", "completed", "success", "pull_request"),
    )
    assert ci_state(newest, "h") == "cancelled"


@pytest.mark.parametrize(
    ("listing", "reason"),
    [
        ("<html>rate limited</html>", "printed no JSON"),
        ('{"runs": []}', "printed no list of runs"),
        ('[{"headSha": "h"}]', "printed a run without"),
        (
            '[{"headSha": "h", "status": null, "conclusion": "", "event": "push"}]',
            "printed a run without",
        ),
    ],
)
def test_malformed_run_listings_are_refused(listing: str, reason: str) -> None:
    with pytest.raises(RunListingError, match=reason):
        _ = ci_state(listing, "h")


def test_a_malformed_run_listing_stops_the_wait_at_once(
    fixture: Fixture,
) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    before = fixture.main_tip()
    github = FakeGitHub(fixture.repo)
    github.listing = "garbled"
    code, out = fixture.run_land("garbled", ("lane/a",), github)
    assert (code, fixture.main_tip()) == (1, before), out
    assert "gh run list printed no JSON" in out
    assert "is not ready" not in out


def test_a_landing_worktree_git_refuses_stops_with_its_reason(
    fixture: Fixture,
) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    _ = git(fixture.repo, "branch", "land/taken", "main")
    code, out = fixture.run_land("taken", ("lane/a",), FakeGitHub(fixture.repo))
    assert code == 1, out
    assert "could not add the landing worktree" in out.splitlines()[-1]


def test_waits_through_a_stale_head_and_transient_gh_failures(
    fixture: Fixture,
) -> None:
    _ = fixture.lane("lane/a", "main", "a.txt")
    code, out = fixture.run_land(
        "slow", ("lane/a",), FakeGitHub(fixture.repo, stale=2, failures=2)
    )
    assert code == 0, out
    assert "waits for GitHub to show" in out
    assert "is not ready: gh pr view: connection reset" in out
    assert git(fixture.origin, "log", "-1", "--format=%s", "main") == "a.txt"


def interrupted(landing: Landing, target: str, *, dry_run: bool) -> None:
    message = f"interrupted {landing.name} at {target} dry={dry_run}"
    raise LandingError(message)


def test_a_rerun_after_landing_goes_straight_to_cleanup(
    fixture: Fixture, monkeypatch: pytest.MonkeyPatch
) -> None:
    path = fixture.lane("lane/a", "main", "a.txt")
    with monkeypatch.context() as patched:
        patched.setattr(lane_land, "clean", interrupted)
        code, out = fixture.run_land(
            "again", ("lane/a",), FakeGitHub(fixture.repo)
        )
    assert code == 1, out
    landed = fixture.main_tip()
    code, out = fixture.run_land(
        "again",
        ("lane/a",),
        FakeGitHub(fixture.repo),
        Options(gate=FAILING_GATE),
    )
    assert code == 0, out
    assert f"land/again at {landed[:9]} is on main" in out
    assert not path.exists()
    assert not (fixture.root / "collab" / "land-again").exists()
    assert not git(fixture.repo, "branch", "--list", "lane/a", "land/again")


def test_skips_a_pick_main_already_has(fixture: Fixture) -> None:
    resolved = fixture.lane("lane/z", "main", "z.txt")
    _ = git(fixture.repo, "cherry-pick", "lane/z")
    _ = (fixture.repo / "z.txt").write_text("resolved\n", encoding="utf-8")
    _ = git(fixture.repo, "commit", "-q", "--amend", "-a", "--no-edit")
    _ = git(fixture.repo, "push", "-q", "origin", "main")
    both, one = fixture.lane("lane/a", "main"), fixture.lane("lane/b", "main")
    for path, names in ((both, ("x.txt", "y.txt")), (one, ("x.txt",))):
        for name in names:
            _ = (path / name).write_text("same\n", encoding="utf-8")
        _ = git(path, "add", *names)
        _ = git(path, "commit", "-qm", " ".join(names))
        _ = git(path, "push", "-q", "origin", path.name.replace("-", "/"))
    code, out = fixture.run_land(
        "empty", ("lane/z", "lane/a", "lane/b"), FakeGitHub(fixture.repo)
    )
    assert code == 0, out
    picked = "".join(
        line for line in out.splitlines() if line.startswith("picked")
    )
    assert "lane/z" not in picked
    assert "lane/b: main already has its change" in out
    subjects = git(fixture.origin, "log", "--format=%s", "main").splitlines()
    assert subjects == ["x.txt y.txt", "z.txt", "base.txt"]
    assert resolved.exists()


def test_ignored_files_outside_target_keep_a_worktree(fixture: Fixture) -> None:
    kept = fixture.lane("lane/a", "main", "a.txt")
    removed = fixture.lane("lane/b", "main", "b.txt")
    _ = (fixture.repo / ".git" / "info" / "exclude").write_text(
        "*.sfc\n/target\n__pycache__/\n/.venv/\n/uv.lock\n", encoding="utf-8"
    )
    _ = (kept / "game.sfc").write_bytes(b"rom")
    for path in (
        removed / "target" / "debug" / "out",
        removed / "tools" / "__pycache__" / "x.pyc",
        removed / ".venv" / "bin" / "python",
        removed / "uv.lock",
    ):
        path.parent.mkdir(parents=True, exist_ok=True)
        _ = path.write_bytes(b"build")
    code, out = fixture.run_land(
        "ignored", ("lane/a", "lane/b"), FakeGitHub(fixture.repo)
    )
    assert code == 0, out
    assert f"kept {kept}: it has ignored files outside target/: game.sfc" in out
    assert (kept / "game.sfc").exists()
    assert not removed.exists()
