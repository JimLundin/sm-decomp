from contextlib import chdir
lazy from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.lane.core.changes import GROUPS, groups
from sm_tools.lane.shell.changes import changed_files, run
from sm_tools.shell.process import git_out

EVERY_GROUP = FrozenOrderedSet(GROUPS)
KINDS = FrozenOrderedDict({
    "docs only": (
        FrozenList((
            "docs/STATUS.md",
            "docs/LEGACY.md",
            ".agents/skills/lane/SKILL.md",
        )),
        FrozenOrderedSet[str](),
    ),
    "lane tools": (
        FrozenList((
            "tools/src/sm_tools/lane/shell/land.py",
            "tools/tests/lane/test_land.py",
        )),
        FrozenOrderedSet(("python",)),
    ),
    "CI tools": (
        FrozenList(("tools/src/sm_tools/ci/shell/replay.py",)),
        FrozenOrderedSet(("python", "rom")),
    ),
    "planted faults": (
        FrozenList(("tools/mutants.toml",)),
        FrozenOrderedSet(("python", "rom")),
    ),
    "a clean model": (
        FrozenList(("crates/clean/samus_movement/src/lib.rs",)),
        EVERY_GROUP,
    ),
    "a behaviour spec": (
        FrozenList(("docs/behaviours/samus-movement.md",)),
        FrozenOrderedSet(("rust", "python", "rom", "adjusted")),
    ),
    "the engine": (
        FrozenList(("crates/runtime/src/lib.rs",)),
        FrozenOrderedSet(("rust", "python", "rom", "engine")),
    ),
    "the app": (
        FrozenList(("crates/app/src/main.rs",)),
        FrozenOrderedSet(("rust", "python", "engine")),
    ),
    "the vector reader": (
        FrozenList(("crates/vector_file/src/lib.rs",)),
        FrozenOrderedSet(("rust", "python", "adjusted")),
    ),
    "the transition check": (
        FrozenList(("tools/src/sm_tools/ci/core/check_replay_transition.py",)),
        FrozenOrderedSet(("python", "rom")),
    ),
    "the codec": (
        FrozenList(("crates/canonical/src/lib.rs",)),
        FrozenOrderedSet(("rust", "python", "rom", "engine")),
    ),
    "the engine size report": (
        FrozenList(("crates/tools/src/engine_size.rs",)),
        FrozenOrderedSet(("rust", "python", "rom", "engine")),
    ),
    "the lock file": (FrozenList(("Cargo.lock",)), EVERY_GROUP),
    "the workflow": (FrozenList((".github/workflows/ci.yml",)), EVERY_GROUP),
    "an action": (
        FrozenList((".github/actions/cargo-restore/action.yml",)),
        EVERY_GROUP,
    ),
    "the tools lock": (FrozenList(("tools/uv.lock",)), EVERY_GROUP),
    "the tools project": (FrozenList(("tools/pyproject.toml",)), EVERY_GROUP),
    "the tools action": (
        FrozenList((".github/actions/tools/action.yml",)),
        EVERY_GROUP,
    ),
    "the build dating tool": (
        FrozenList(("tools/src/sm_tools/ci/shell/mtimes.py",)),
        EVERY_GROUP,
    ),
    "this table": (
        FrozenList(("tools/src/sm_tools/lane/core/changes.py",)),
        EVERY_GROUP,
    ),
})


def started(paths: FrozenList[str]) -> FrozenOrderedSet[str]:
    return FrozenOrderedSet(
        name for name, changed in groups(paths).items() if changed
    )


@pytest.mark.parametrize("kind", KINDS)
def test_each_kind_of_change_starts_its_groups(kind: str) -> None:
    paths, expected = KINDS[kind]
    assert started(paths) == expected


def test_a_prefix_of_a_folder_name_is_not_that_folder() -> None:
    assert not groups(FrozenList(("cratesmith.md", "docs/behaviours-old.md")))[
        "rust"
    ]


def test_without_a_base_every_group_runs() -> None:
    assert groups(None) == FrozenOrderedDict(dict.fromkeys(GROUPS, True))


def git(repo: Path, *arguments: str) -> None:
    _ = git_out(
        "-c", "user.name=t", "-c", "user.email=t@t", *arguments, cwd=repo
    )


def test_committed_staged_and_untracked_files_count(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    git(tmp_path, "init", "-q", "-b", "main")
    for name in ("docs/a.md", "tools/src/sm_tools/ci/b.py", "crates/app/c.rs"):
        (tmp_path / name).parent.mkdir(parents=True, exist_ok=True)
        _ = (tmp_path / name).write_text("before\n")
    git(tmp_path, "add", ".")
    git(tmp_path, "commit", "-q", "-m", "base")
    git(tmp_path, "branch", "base")
    _ = (tmp_path / "docs/a.md").write_text("after\n")
    git(tmp_path, "commit", "-q", "-am", "docs")
    _ = (tmp_path / "tools/src/sm_tools/ci/b.py").write_text("after\n")
    _ = (tmp_path / "crates/app/new.rs").write_text("new\n")
    with chdir(tmp_path):
        assert sorted(changed_files("base")) == [
            "crates/app/new.rs",
            "docs/a.md",
            "tools/src/sm_tools/ci/b.py",
        ]
        assert run(("base",)) == 0
    assert capsys.readouterr().out.split() == [
        "rust=true",
        "python=true",
        "rom=true",
        "adjusted=false",
        "engine=true",
    ]
