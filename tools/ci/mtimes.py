import os
import re
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path

USAGE = """usage: mtimes.py record | restore

Run at the root of a checkout whose Cargo build is in target/.

record: writes HEAD's commit to target/.built-from, before the build is cached.
restore: after a cached build is restored, dates every tracked file 2000-01-01 and touches those that
differ between the recorded commit and HEAD, so Cargo rebuilds only what they feed. A shallow checkout
fetches the recorded commit. Every tracked file is touched, so everything rebuilds, when there is no
record, when its commit can't be found within a minute, or when a build configuration file such as a
Cargo.toml, Cargo.lock, build.rs or .cargo/config.toml changed since it: Cargo doesn't date those, and
a unit they had left out of the recorded build would otherwise come back with its old artifact."""
OLD = 946684800
RECORD = Path("target/.built-from")
COMMIT = re.compile(r"[0-9a-f]{40}")
FETCH_TIMEOUT = 60.0
BUILD_CONFIG = frozenset(
    {
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "rust-toolchain",
        "rust-toolchain.toml",
        "clippy.toml",
        ".clippy.toml",
    }
)
CARGO_CONFIG = frozenset({"config", "config.toml"})


def git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args], cwd=root, capture_output=True, text=True, check=False
    )


def tracked(root: Path, *args: str) -> list[Path]:
    listed = subprocess.run(
        ["git", *args[:1], "-z", *args[1:]],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return [root / name for name in listed.split("\0") if name]


def configures_build(file: Path) -> bool:
    return file.name in BUILD_CONFIG or (
        file.parent.name == ".cargo" and file.name in CARGO_CONFIG
    )


def date_since(root: Path, built_from: str) -> int:
    changed = tracked(root, "diff", "--name-only", built_from, "HEAD", "--")
    for file in tracked(root, "ls-files"):
        if file.is_symlink() or file.exists():
            os.utime(file, (OLD, OLD), follow_symlinks=False)
    for file in changed:
        if file.exists():
            os.utime(file)
    return len(changed)


def touch_all(root: Path) -> None:
    for file in tracked(root, "ls-files"):
        if file.is_symlink() or file.exists():
            os.utime(file, follow_symlinks=False)


def has_commit(root: Path, commit: str) -> bool:
    return git(root, "cat-file", "-e", f"{commit}^{{commit}}").returncode == 0


def fetched(root: Path, commit: str, timeout: float) -> bool:
    try:
        subprocess.run(
            ["git", "fetch", "-q", "--no-tags", "--depth=1", "origin", commit],
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
    commit = (
        record.read_text(encoding="utf-8").strip() if record.is_file() else ""
    )
    if not COMMIT.fullmatch(commit):
        return None
    if has_commit(root, commit) or fetched(root, commit, timeout):
        return commit
    return None


def restore(root: Path) -> int:
    commit = built_from(root)
    if commit is None:
        touch_all(root)
        print(
            "mtimes: the build records no commit this checkout can reach; touched every tracked file"
        )
        return 0
    changed = tracked(root, "diff", "--name-only", commit, "HEAD", "--")
    config = next(filter(configures_build, changed), None)
    if config is not None:
        touch_all(root)
        print(
            f"mtimes: {config.relative_to(root)} changed since the build's {commit[:12]}; touched every tracked file"
        )
        return 0
    print(
        f"mtimes: the build comes from {commit[:12]}; touched the {date_since(root, commit)} files changed since"
    )
    return 0


def record(root: Path) -> int:
    commit = git(root, "rev-parse", "--verify", "HEAD").stdout.strip()
    (root / RECORD).parent.mkdir(parents=True, exist_ok=True)
    (root / RECORD).write_text(commit + "\n", encoding="utf-8")
    print(f"mtimes: recorded that the build comes from {commit[:12]}")
    return 0


def main(arguments: Sequence[str]) -> int:
    match arguments:
        case ["record"]:
            return record(Path.cwd())
        case ["restore"]:
            return restore(Path.cwd())
        case _:
            print(USAGE, file=sys.stderr)
            return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
