import os
import shutil
import subprocess
import sys
from pathlib import Path

USAGE = """usage: worktree.py add <branch> <path> <base> | seed

add: a lane worktree on a new branch, its target/ taking the build of the seed, one worktree at
$SM_SEED_DIR (default $TEMP/sm-seed) built at main the way lanes build. The build is hard-linked
(copied where links fail or with SM_SEED_COPY=1), and the dep-info files cargo rewrites in place
are copied. The lane's tracked files are dated 2000-01-01 and those that differ from the seed's
commit are touched, so only what they feed rebuilds.

seed: builds the one seed at origin/main (the root, off-hours)."""
OLD = 946684800
LOCKS = (".cargo-lock", ".cargo-build-lock", ".cargo-artifact-lock")


def git(*args: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=False)


def tracked(path: Path, *args: str) -> list[Path]:
    listed = git(*args[:1], "-z", *args[1:], cwd=path).stdout
    return [path / name for name in listed.split("\0") if name]


def link_debug(seed: Path, target: Path) -> bool:
    built = seed / "target/debug"
    try:
        shutil.copytree(built, target / "debug", symlinks=True, copy_function=os.link)
    except OSError:
        shutil.rmtree(target / "debug", ignore_errors=True)
        return False
    for source in built.rglob("*"):
        if source.is_file() and not source.is_symlink() and (".fingerprint" in source.parts or source.suffix == ".d"):
            copy = target / "debug" / source.relative_to(built)
            copy.unlink()
            shutil.copy2(source, copy)
    return True


def add(branch: str, path: Path, base: str, seed: Path) -> int:
    added = git("worktree", "add", "-q", "-b", branch, str(path), base)
    if added.returncode != 0:
        print(added.stderr, end="", file=sys.stderr)
        return added.returncode
    seeded = git("rev-parse", "-q", "--verify", "HEAD", cwd=seed) if seed.is_dir() else None
    if seeded is None or seeded.returncode != 0 or not (seed / "target/.seeded").is_file():
        print(f"worktree: no seed at {seed}; the first build is cold")
        return 0
    target = path / "target"
    target.mkdir()
    how = "linked"
    if (seed / "target/debug").is_dir() and (os.environ.get("SM_SEED_COPY") or not link_debug(seed, target)):
        shutil.copytree(seed / "target/debug", target / "debug", symlinks=True)
        how = "copied"
    for lock in target.rglob("*"):
        if lock.name in LOCKS and lock.is_file():
            lock.unlink()
    for file in tracked(path, "ls-files"):
        if file.is_symlink() or file.exists():
            os.utime(file, (OLD, OLD), follow_symlinks=False)
    for file in tracked(path, "diff", "--name-only", seeded.stdout.strip(), "HEAD", "--"):
        if file.exists():
            os.utime(file)
    print(f"worktree: {how} the build of seed {git('rev-parse', '--short', 'HEAD', cwd=seed).stdout.strip()}")
    return 0


def build_seed(seed: Path) -> int:
    subprocess.run(["git", "fetch", "-q", "origin"], check=True)
    if seed.is_dir():
        subprocess.run(["git", "-C", str(seed), "checkout", "-q", "--detach", "origin/main"], check=True)
    else:
        subprocess.run(["git", "worktree", "add", "-q", "--detach", str(seed), "origin/main"], check=True)
    (seed / "target/.seeded").unlink(missing_ok=True)
    subprocess.run([sys.executable, "tools/lane/gate.py"], cwd=seed, check=True)
    subprocess.run(["cargo", "test", "-p", "game", "-p", "verify", "--no-run"], cwd=seed, check=True,
                   env=os.environ | {"CARGO_PROFILE_TEST_OPT_LEVEL": "3"})
    (seed / "target/.seeded").touch()
    size = subprocess.run(["du", "-sh", str(seed / "target")], capture_output=True, text=True, check=True)
    print(f"worktree: seed at {git('rev-parse', '--short', 'HEAD', cwd=seed).stdout.strip()}, "
          f"{size.stdout.split()[0]}")
    return 0


def main(arguments: list[str]) -> int:
    scratch = Path(os.environ.get("TEMP") or os.environ.get("TMPDIR") or "/tmp")
    seed = Path(os.environ.get("SM_SEED_DIR") or scratch / "sm-seed")
    match arguments:
        case ["add", branch, path, base]:
            return add(branch, Path(path), base, seed)
        case ["seed"]:
            return build_seed(seed)
        case _:
            print(USAGE, file=sys.stderr)
            return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
