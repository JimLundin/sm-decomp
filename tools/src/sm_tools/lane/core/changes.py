from fnmatch import fnmatchcase
lazy from collections.abc import Sequence

from ordered import FrozenList, FrozenOrderedDict

USAGE = """usage: changes [BASE]

Prints GROUP=true or GROUP=false for each path group of CI's jobs: true when a path of the group
changed since BASE, committed or in the working tree. Without BASE every group is true. A pattern
is a path, the folder it names, or a glob; one starting with ! leaves its paths out of the group.
The workflow, its actions and this tool count for every group."""

LANE = "tools/src/sm_tools/lane"
EVERY = FrozenList((
    ".github",
    f"{LANE}/core/changes.py",
    f"{LANE}/shell/changes.py",
    "tools/src/sm_tools/shell/process.py",
))
BUILD = FrozenList(("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"))
RUST = FrozenList((
    "crates",
    *BUILD,
    ".cargo",
    ".config",
    "deny.toml",
    "clippy.toml",
    "README.md",
    "scenarios",
    "movies",
    "docs/behaviours",
    "docs/architecture",
    "docs/cleanroom",
    "tools/cleanroom",
    "tools/oracle",
    "tools/lsnes_lag_log.lua",
    "tools/scenario_seeds.py",
    ".agents/skills/implementer",
    ".agents/skills/blind-reviewer",
))
GROUPS = FrozenOrderedDict({
    "rust": RUST,
    "python": FrozenList((*RUST, "tools", "pyproject.toml")),
    "rom": FrozenList((
        "crates",
        "!crates/app",
        "!crates/vector_file",
        *BUILD,
        ".cargo",
        "scenarios",
        "movies",
        "tools/ci",
        "tools/src/sm_tools/ci",
        "tools/check_mutants.py",
        "tools/selectable_faults.py",
        "tools/mutants.toml",
        "tools/fault-requirements.txt",
        "tools/pyproject.toml",
        "tools/uv.lock",
        "docs/architecture/domains.toml",
        "docs/behaviours",
    )),
    "adjusted": FrozenList((
        "docs/behaviours",
        "scenarios/vectors",
        "crates/clean",
        "crates/vector_file",
        *BUILD,
        "tools/cleanroom/adjusted_*",
    )),
    "engine": FrozenList((
        "crates/app",
        "crates/game",
        "crates/runtime",
        "crates/data",
        "crates/clean",
        "crates/canonical",
        "crates/canonical_derive",
        *BUILD,
        "crates/tools/src/engine_size*",
        "scenarios/engine-size.toml",
    )),
})
UNCOMMITTED = FrozenList((
    "status",
    "-z",
    "--porcelain",
    "--no-renames",
    "--untracked-files=all",
))
STATUS_PREFIX = 3
MOST_ARGUMENTS = 1


def committed(base: str) -> FrozenList[str]:
    return FrozenList((
        "diff",
        "-z",
        "--name-only",
        "--no-renames",
        f"{base}...HEAD",
    ))


def committed_under(base: str, paths: Sequence[str]) -> FrozenList[str]:
    return FrozenList((
        "diff",
        "--quiet",
        "--no-renames",
        f"{base}...HEAD",
        "--",
        *paths,
    ))


def uncommitted_under(paths: Sequence[str]) -> FrozenList[str]:
    return FrozenList((*UNCOMMITTED, "--", *paths))


def listed_changes(status: int, listing: str) -> bool:
    return status != 0 or bool(listing.strip())


def listed(diff: str, status: str) -> FrozenList[str]:
    return FrozenList((
        *(name for name in diff.split("\0") if name),
        *(entry[STATUS_PREFIX:] for entry in status.split("\0") if entry),
    ))


def covers(pattern: str, path: str) -> bool:
    return fnmatchcase(path, pattern) or path.startswith(pattern + "/")


def touched(patterns: Sequence[str], paths: Sequence[str]) -> bool:
    kept = FrozenList((
        *EVERY,
        *(pattern for pattern in patterns if not pattern.startswith("!")),
    ))
    left_out = FrozenList(
        pattern[1:] for pattern in patterns if pattern.startswith("!")
    )
    return any(
        any(covers(pattern, path) for pattern in kept)
        and not any(covers(pattern, path) for pattern in left_out)
        for path in paths
    )


def groups(paths: Sequence[str] | None) -> FrozenOrderedDict[str, bool]:
    return FrozenOrderedDict(
        (name, paths is None or touched(patterns, paths))
        for name, patterns in GROUPS.items()
    )


def outputs(changed: FrozenOrderedDict[str, bool]) -> FrozenList[str]:
    return FrozenList(
        f"{name}={str(value).lower()}" for name, value in changed.items()
    )
