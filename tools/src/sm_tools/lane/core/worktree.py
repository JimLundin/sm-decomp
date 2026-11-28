lazy from collections.abc import Sequence

from ordered import FrozenList

USAGE = """usage: worktree add <branch> <path> <base> | seed

add: a lane worktree on a new branch, its target/ taking the build of the seed, one worktree at
$SM_SEED_DIR (default $TEMP/sm-seed) built at main the way lanes build. The build is hard-linked
(copied where links fail or with SM_SEED_COPY=1), and the dep-info files cargo rewrites in place
are copied. The lane's tracked files are dated 2000-01-01 and those that differ from the seed's
commit are touched, so only what they feed rebuilds. When a build configuration file differs, such
as a Cargo.toml, Cargo.lock or build.rs, every tracked file is touched, as mtimes restore does.

seed: builds the one seed at origin/main (the root, off-hours)."""
LOCKS = FrozenList((".cargo-lock", ".cargo-build-lock", ".cargo-artifact-lock"))
SEEDED = "target/.seeded"
SEED_FOLDER = "sm-seed"
FINGERPRINTS = ".fingerprint"
DEP_INFO = ".d"
GATE = FrozenList(("uv", "run", "--locked", "--project", "tools", "gate"))
SEED_BUILD = FrozenList((
    "cargo",
    "test",
    "-p",
    "game",
    "-p",
    "verify",
    "--no-run",
))


def rewritten_in_place(parts: Sequence[str], suffix: str) -> bool:
    return FINGERPRINTS in parts or suffix == DEP_INFO


def cold(seed: str) -> str:
    return f"worktree: no seed at {seed}; the first build is cold"


def seeded(how: str, commit: str) -> str:
    return f"worktree: {how} the build of seed {commit}"


def seed_built(commit: str, size: str) -> str:
    return f"worktree: seed at {commit}, {size.split(maxsplit=1)[0]}"
