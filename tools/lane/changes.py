import subprocess
import sys
from collections.abc import Iterable, Sequence
from fnmatch import fnmatchcase

USAGE = """usage: changes.py [BASE]

Prints GROUP=true or GROUP=false for each path group of CI's jobs: true when a path of the group
changed since BASE, committed or in the working tree. Without BASE every group is true. A pattern
is a path, the folder it names, or a glob; one starting with ! leaves its paths out of the group.
The workflow, its actions and this file count for every group."""

EVERY = (".github", "tools/lane/changes.py")
BUILD = ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")
RUST = (
    "crates", *BUILD, ".cargo", ".config", "deny.toml", "clippy.toml", "README.md", "scenarios", "movies",
    "docs/behaviours", "docs/architecture", "docs/cleanroom", "tools/cleanroom", "tools/oracle",
    "tools/lsnes_lag_log.lua", "tools/scenario_seeds.py", ".agents/skills/implementer", ".agents/skills/blind-reviewer",
)
GROUPS: dict[str, tuple[str, ...]] = {
    "rust": RUST,
    "python": (*RUST, "tools", "pyproject.toml"),
    "rom": (
        "crates", "!crates/app", "!crates/vector_file", *BUILD, ".cargo", "scenarios", "movies", "tools/ci",
        "tools/check_mutants.py", "tools/selectable_faults.py", "tools/mutants.toml", "tools/fault-requirements.txt",
        "docs/architecture/domains.toml", "docs/behaviours", "tools/check_replay_transition.py",
    ),
    "adjusted": (
        "docs/behaviours", "scenarios/vectors", "crates/clean", "crates/vector_file", *BUILD,
        "tools/cleanroom/adjusted_*",
    ),
    "engine": (
        "crates/app", "crates/game", "crates/runtime", "crates/data", "crates/clean", "crates/canonical",
        "crates/canonical_derive", *BUILD,
        "crates/tools/src/engine_size*", "scenarios/engine-size.toml",
    ),
}


def changed_since(base: str, paths: Sequence[str]) -> bool:
    committed = subprocess.run(["git", "diff", "--quiet", "--no-renames", f"{base}...HEAD", "--", *paths],
                               capture_output=True, check=False)
    if committed.returncode != 0:
        return True
    status = subprocess.run(["git", "status", "--porcelain", "--no-renames", "--untracked-files=all", "--", *paths],
                            capture_output=True, text=True, check=False)
    return status.returncode != 0 or bool(status.stdout.strip())


def changed_files(base: str) -> list[str]:
    committed = subprocess.run(["git", "diff", "-z", "--name-only", "--no-renames", f"{base}...HEAD"],
                               capture_output=True, text=True, check=True).stdout
    status = subprocess.run(["git", "status", "-z", "--porcelain", "--no-renames", "--untracked-files=all"],
                            capture_output=True, text=True, check=True).stdout
    return [name for name in committed.split("\0") if name] + [entry[3:] for entry in status.split("\0") if entry]


def covers(pattern: str, path: str) -> bool:
    return fnmatchcase(path, pattern) or path.startswith(pattern + "/")


def touched(patterns: Sequence[str], paths: Iterable[str]) -> bool:
    kept = [*EVERY, *(pattern for pattern in patterns if not pattern.startswith("!"))]
    left_out = [pattern[1:] for pattern in patterns if pattern.startswith("!")]
    return any(any(covers(pattern, path) for pattern in kept) and not any(covers(pattern, path) for pattern in left_out)
               for path in paths)


def groups(paths: Sequence[str] | None) -> dict[str, bool]:
    return {name: paths is None or touched(patterns, paths) for name, patterns in GROUPS.items()}


def main(arguments: list[str]) -> int:
    if len(arguments) > 1:
        print(USAGE, file=sys.stderr)
        return 2
    for name, changed in groups(changed_files(arguments[0]) if arguments else None).items():
        print(f"{name}={str(changed).lower()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
