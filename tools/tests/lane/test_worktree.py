from dataclasses import dataclass
lazy from collections.abc import Mapping
lazy from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedDict

from sm_tools.lane.shell.worktree import run
from sm_tools.shell.process import git_out

IDENTITY = FrozenOrderedDict({
    "GIT_AUTHOR_NAME": "worktree-check",
    "GIT_AUTHOR_EMAIL": "worktree-check@localhost",
    "GIT_COMMITTER_NAME": "worktree-check",
    "GIT_COMMITTER_EMAIL": "worktree-check@localhost",
})
LOCKS = FrozenList((".cargo-lock", ".cargo-build-lock", ".cargo-artifact-lock"))
DEP_INFO = FrozenList(("deps/x.d", ".fingerprint/x-1/dep-lib-x"))
SOURCES = FrozenOrderedDict({
    ".gitignore": "target/",
    "same.rs": "same",
    "changed.rs": "old",
    "deleted.rs": "gone",
})


@dataclass(frozen=True, slots=True)
class Fixture:
    tmp: Path
    repo: Path
    seed: Path
    monkeypatch: pytest.MonkeyPatch
    capsys: pytest.CaptureFixture[str]

    @property
    def built(self) -> Path:
        return self.seed / "target/debug"

    def git(self, *arguments: str, cwd: Path | None = None) -> str:
        return git_out(*arguments, cwd=cwd or self.repo).strip()

    def add(self, lane: str, base: str, environment: Mapping[str, str]) -> str:
        for name, value in environment.items():
            self.monkeypatch.setenv(name, value)
        status = run(("add", lane, str(self.tmp / lane), base))
        out = self.capsys.readouterr().out
        assert status == 0, out
        return out


def linked(a: Path, b: Path) -> bool:
    return a.stat().st_ino == b.stat().st_ino


def newer(a: Path, b: Path) -> bool:
    return a.stat().st_mtime_ns > b.stat().st_mtime_ns


def build_seed(seed: Path) -> None:
    built = seed / "target/debug"
    (built / "deps").mkdir(parents=True)
    (built / ".fingerprint/x-1").mkdir(parents=True)
    (seed / "target/release").mkdir()
    _ = (built / "deps/x.rlib").write_text("built\n")
    for info in DEP_INFO:
        _ = (built / info).write_text("info\n")
    _ = (seed / "target/release/out").write_text("shipped\n")
    for lock in LOCKS:
        (built / lock).touch()


@pytest.fixture
def fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> Fixture:
    repo = tmp_path / "repo"
    repo.mkdir()
    seed = tmp_path / "seed"
    for name, value in IDENTITY.items():
        monkeypatch.setenv(name, value)
    monkeypatch.setenv("SM_SEED_DIR", str(seed))
    monkeypatch.chdir(repo)
    made = Fixture(tmp_path, repo, seed, monkeypatch, capsys)
    _ = made.git("init", "-q", "-b", "main", ".")
    _ = made.git("config", "commit.gpgsign", "false")
    for name, text in SOURCES.items():
        _ = (repo / name).write_text(text + "\n")
    _ = made.git("add", "-A")
    _ = made.git("commit", "-q", "-m", "seed")
    _ = made.git("worktree", "add", "-q", "--detach", str(seed), "HEAD")
    build_seed(seed)
    _ = (repo / "changed.rs").write_text("new\n")
    _ = made.git("rm", "-q", "deleted.rs")
    _ = (repo / "added.rs").write_text("added\n")
    _ = made.git("add", "-A")
    _ = made.git("commit", "-q", "-m", "base")
    return made


def check_lane(
    fixture: Fixture, lane: str, environment: Mapping[str, str]
) -> None:
    (fixture.seed / "target/.seeded").touch()
    assert f"{lane} the build" in fixture.add(lane, "main", environment)
    path = fixture.tmp / lane
    built = path / "target/debug"
    seed = fixture.built
    assert fixture.git("rev-parse", "HEAD", cwd=path) == fixture.git(
        "rev-parse", "main"
    )
    assert (built / "deps/x.rlib").read_text() == "built\n"
    assert not (path / "target/release").exists()
    assert linked(built / "deps/x.rlib", seed / "deps/x.rlib") == (
        lane == "linked"
    )
    for lock in LOCKS:
        assert (seed / lock).is_file(), f"the seed's {lock} is missing"
        (built / lock).touch()
        assert not linked(built / lock, seed / lock), f"{lock} is shared"
    for info in DEP_INFO:
        assert (built / info).read_text() == "info\n"
        assert not linked(built / info, seed / info), f"{info} is shared"
        _ = (built / info).write_text("rewritten\n")
        assert (seed / info).read_text() == "info\n"
    rlib = built / "deps/x.rlib"
    assert newer(path / "changed.rs", rlib), "a changed source looks built"
    assert newer(path / "added.rs", rlib), "an added source looks built"
    assert not newer(path / "same.rs", rlib), (
        "an unchanged source looks changed"
    )
    assert not fixture.git("status", "--porcelain", cwd=path)


def test_an_unbuilt_seed_starts_cold(fixture: Fixture) -> None:
    assert "first build is cold" in fixture.add("cold", "HEAD", {})
    assert not (fixture.tmp / "cold/target").exists()


def test_a_lane_links_the_seed_build(fixture: Fixture) -> None:
    check_lane(fixture, "linked", {})


def test_sm_seed_copy_copies_the_seed_build(fixture: Fixture) -> None:
    check_lane(fixture, "copied", {"SM_SEED_COPY": "1"})


def test_a_changed_build_configuration_rebuilds_everything(
    fixture: Fixture,
) -> None:
    (fixture.seed / "target/.seeded").touch()
    _ = (fixture.repo / "Cargo.toml").write_text("[workspace]\n")
    _ = fixture.git("add", "Cargo.toml")
    _ = fixture.git("commit", "-q", "-m", "configure")
    out = fixture.add("configured", "main", {})
    assert "Cargo.toml changed since" in out
    path = fixture.tmp / "configured"
    rlib = path / "target/debug/deps/x.rlib"
    assert newer(path / "same.rs", rlib), "an unchanged source looks built"
