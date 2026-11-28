import os
import subprocess
import sys
from pathlib import Path

from ordered import FrozenList

from sm_tools.ci.core.mtimes import (
    FETCH_TIMEOUT,
    NO_RECORD,
    OLD,
    RECORD,
    USAGE,
    configures_build,
    names,
    reconfigured,
    recorded,
    recorded_line,
    restored,
)
from sm_tools.shell.process import git, git_out


def tracked(root: Path, *arguments: str) -> FrozenList[str]:
    return names(git_out(*arguments[:1], "-z", *arguments[1:], cwd=root))


def date_since(root: Path, built_from: str) -> str:
    changed = tracked(root, "diff", "--name-only", built_from, "HEAD", "--")
    config = next(filter(configures_build, changed), None)
    if config is not None:
        touch_all(root)
        return reconfigured(built_from, config)
    for name in tracked(root, "ls-files"):
        file = root / name
        if file.is_symlink() or file.exists():
            os.utime(file, (OLD, OLD), follow_symlinks=False)
    for name in changed:
        if (root / name).exists():
            os.utime(root / name)
    return restored(built_from, len(changed))


def touch_all(root: Path) -> None:
    for name in tracked(root, "ls-files"):
        file = root / name
        if file.is_symlink() or file.exists():
            os.utime(file, follow_symlinks=False)


def has_commit(root: Path, commit: str) -> bool:
    return (
        git("cat-file", "-e", f"{commit}^{{commit}}", cwd=root).returncode == 0
    )


def fetched(root: Path, commit: str, timeout: float) -> bool:
    try:
        _ = subprocess.run(
            ("git", "fetch", "-q", "--no-tags", "--depth=1", "origin", commit),
            cwd=root,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return False
    return has_commit(root, commit)


def built_from(root: Path, timeout: float = FETCH_TIMEOUT) -> str | None:
    record = root / RECORD
    commit = recorded(
        record.read_text(encoding="utf-8") if record.is_file() else ""
    )
    if commit is None:
        return None
    if has_commit(root, commit) or fetched(root, commit, timeout):
        return commit
    return None


def restore(root: Path) -> int:
    commit = built_from(root)
    if commit is None:
        touch_all(root)
        print(NO_RECORD)
        return 0
    print(date_since(root, commit))
    return 0


def record(root: Path) -> int:
    commit = git("rev-parse", "--verify", "HEAD", cwd=root).stdout.strip()
    (root / RECORD).parent.mkdir(parents=True, exist_ok=True)
    _ = (root / RECORD).write_text(commit + "\n", encoding="utf-8")
    print(recorded_line(commit))
    return 0


def main() -> int:
    match FrozenList(sys.argv[1:]):
        case ["record"]:
            return record(Path.cwd())
        case ["restore"]:
            return restore(Path.cwd())
        case _:
            print(USAGE, file=sys.stderr)
            return 2
