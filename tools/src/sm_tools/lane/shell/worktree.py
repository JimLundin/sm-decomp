import os
import shutil
import subprocess
import sys
from pathlib import Path
lazy from collections.abc import Sequence

from ordered import FrozenList

from sm_tools.lane.core.worktree import (
    GATE,
    LOCKS,
    OLD,
    SEED_BUILD,
    SEED_FOLDER,
    SEEDED,
    USAGE,
    cold,
    names,
    rewritten_in_place,
    seed_built,
    seeded,
)
from sm_tools.lane.shell.scratch import scratch
from sm_tools.shell.process import git, git_out


def tracked(path: Path, *arguments: str) -> FrozenList[Path]:
    listed = git(*arguments[:1], "-z", *arguments[1:], cwd=path).stdout
    return FrozenList(path / name for name in names(listed))


def link_debug(seed: Path, target: Path) -> bool:
    built = seed / "target/debug"
    try:
        _ = shutil.copytree(
            built, target / "debug", symlinks=True, copy_function=os.link
        )
    except OSError:
        shutil.rmtree(target / "debug", ignore_errors=True)
        return False
    for source in built.rglob("*"):
        if (
            source.is_file()
            and not source.is_symlink()
            and rewritten_in_place(source.parts, source.suffix)
        ):
            copy = target / "debug" / source.relative_to(built)
            copy.unlink()
            _ = shutil.copy2(source, copy)
    return True


def take_build(seed: Path, target: Path) -> str:
    target.mkdir()
    how = "linked"
    if (seed / "target/debug").is_dir() and (
        os.environ.get("SM_SEED_COPY") or not link_debug(seed, target)
    ):
        _ = shutil.copytree(
            seed / "target/debug", target / "debug", symlinks=True
        )
        how = "copied"
    for lock in target.rglob("*"):
        if lock.name in LOCKS and lock.is_file():
            lock.unlink()
    return how


def date_sources(path: Path, seed_commit: str) -> None:
    for file in tracked(path, "ls-files"):
        if file.is_symlink() or file.exists():
            os.utime(file, (OLD, OLD), follow_symlinks=False)
    for file in tracked(path, "diff", "--name-only", seed_commit, "HEAD", "--"):
        if file.exists():
            os.utime(file)


def add(branch: str, path: Path, base: str, seed: Path, repo: Path) -> int:
    added = git(
        "worktree", "add", "-q", "-b", branch, str(path), base, cwd=repo
    )
    if added.returncode != 0:
        print(added.stderr, end="", file=sys.stderr)
        return added.returncode
    commit = (
        git("rev-parse", "-q", "--verify", "HEAD", cwd=seed)
        if seed.is_dir()
        else None
    )
    if (
        commit is None
        or commit.returncode != 0
        or not (seed / SEEDED).is_file()
    ):
        print(cold(str(seed)))
        return 0
    how = take_build(seed, path / "target")
    date_sources(path, commit.stdout.strip())
    short = git("rev-parse", "--short", "HEAD", cwd=seed).stdout.strip()
    print(seeded(how, short))
    return 0


def build_seed(seed: Path) -> int:
    _ = git_out("fetch", "-q", "origin")
    if seed.is_dir():
        _ = git_out("checkout", "-q", "--detach", "origin/main", cwd=seed)
    else:
        _ = git_out(
            "worktree", "add", "-q", "--detach", str(seed), "origin/main"
        )
    (seed / SEEDED).unlink(missing_ok=True)
    _ = subprocess.run(GATE, cwd=seed, check=True)
    _ = subprocess.run(
        SEED_BUILD,
        cwd=seed,
        check=True,
        env=os.environ | {"CARGO_PROFILE_TEST_OPT_LEVEL": "3"},
    )
    (seed / SEEDED).touch()
    size = subprocess.run(
        ("du", "-sh", str(seed / "target")),
        capture_output=True,
        text=True,
        check=True,
    )
    short = git("rev-parse", "--short", "HEAD", cwd=seed).stdout.strip()
    print(seed_built(short, size.stdout))
    return 0


def seed_dir() -> Path:
    return Path(os.environ.get("SM_SEED_DIR") or scratch() / SEED_FOLDER)


def run(arguments: Sequence[str]) -> int:
    match tuple(arguments):
        case ("add", branch, path, base):
            return add(branch, Path(path), base, seed_dir(), Path.cwd())
        case ("seed",):
            return build_seed(seed_dir())
        case _:
            print(USAGE, file=sys.stderr)
            return 2


def main() -> int:
    return run(sys.argv[1:])
