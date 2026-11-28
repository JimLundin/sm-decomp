import re
from pathlib import PurePosixPath

from ordered import FrozenList, FrozenOrderedSet

USAGE = """usage: mtimes record | restore

Run at the root of a checkout whose Cargo build is in target/.

record: writes HEAD's commit to target/.built-from, before the build is cached.
restore: after a cached build is restored, dates every tracked file 2000-01-01 and touches those that
differ between the recorded commit and HEAD, so Cargo rebuilds only what they feed. A shallow checkout
fetches the recorded commit. Every tracked file is touched, so everything rebuilds, when there is no
record, when its commit can't be found within a minute, or when a build configuration file such as a
Cargo.toml, Cargo.lock, build.rs or .cargo/config.toml changed since it: Cargo doesn't date those, and
a unit they had left out of the recorded build would otherwise come back with its old artifact."""
OLD = 946684800
RECORD = "target/.built-from"
COMMIT = re.compile(r"[0-9a-f]{40}")
FETCH_TIMEOUT = 60.0
BUILD_CONFIG = FrozenOrderedSet((
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "rust-toolchain",
    "rust-toolchain.toml",
    "clippy.toml",
    ".clippy.toml",
))
CARGO_CONFIG = FrozenOrderedSet(("config", "config.toml"))
NO_RECORD = (
    "mtimes: the build records no commit this checkout can reach;"
    " touched every tracked file"
)


def names(listing: str) -> FrozenList[str]:
    return FrozenList(name for name in listing.split("\0") if name)


def configures_build(name: str) -> bool:
    path = PurePosixPath(name)
    return path.name in BUILD_CONFIG or (
        path.parent.name == ".cargo" and path.name in CARGO_CONFIG
    )


def recorded(text: str) -> str | None:
    commit = text.strip()
    return commit if COMMIT.fullmatch(commit) else None


def reconfigured(commit: str, config: str) -> str:
    return (
        f"mtimes: {config} changed since the build's {commit[:12]};"
        " touched every tracked file"
    )


def restored(commit: str, changed: int) -> str:
    return (
        f"mtimes: the build comes from {commit[:12]};"
        f" touched the {changed} files changed since"
    )


def recorded_line(commit: str) -> str:
    return f"mtimes: recorded that the build comes from {commit[:12]}"
