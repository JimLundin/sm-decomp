import re
from dataclasses import dataclass
lazy from collections.abc import Sequence

from ordered import FrozenList

USAGE = """usage: record-caches ROWS BUNDLE ROM CACHE [JOBS]

Records the lockstep record cache (docs/ORACLE.md section 6) of the rows in ROWS, one row id a
line, `#` starting a comment, `-` for standard input. BUNDLE is the decrypted relock bundle's
relock-v2-lsnes/ directory, ROM the unheadered ROM, and each row is written to CACHE/<row>/.
JOBS rows record at once, 8 by default, each one lsnes process.

Each row is `oracle corpus ROM LOCK BUNDLE --cache CACHE --only ROW`. A row whose cache the reader
accepts, recorded by this lsnes and server with every requested watch, is kept. Each row reports
cached, recorded or failed, with the corpus output in CACHE/.logs/<row>.log.

Environment: ORACLE (default target/verify/oracle), SM_TOOLS (the cache reader, default
target/verify/tools, built first), SM_LOCK (default scenarios/locks/lsnes.toml), SM_LSNES (the
lsnes the oracle runs), WATCH_PCS (comma-separated addresses for --watch-pcs, so each row also
gets its hits.txt)."""
ROW = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]*")
POSITIVE = re.compile(r"[1-9][0-9]*")
DEFAULT_JOBS = "8"
JOBS_ARGUMENT = 4
STANDARD_INPUT = "-"
JOBS_REFUSAL = "jobs must be a positive integer"
BUNDLE_PARTS = FrozenList(("pads", "seeds"))


def rows_of(text: str) -> FrozenList[str]:
    rows = (
        "".join(line.split("#", 1)[0].split()) for line in text.splitlines()
    )
    return FrozenList(dict.fromkeys(row for row in rows if row))


def jobs_of(arguments: Sequence[str]) -> str:
    if len(arguments) > JOBS_ARGUMENT:
        return arguments[JOBS_ARGUMENT]
    return DEFAULT_JOBS


def usable_arguments(arguments: Sequence[str]) -> bool:
    return JOBS_ARGUMENT <= len(arguments) <= JOBS_ARGUMENT + 1


def missing_bundle_part(directory: str) -> str:
    return (
        f"{directory}: missing; BUNDLE is the relock bundle's relock-v2-lsnes/"
    )


def missing_rom(rom: str) -> str:
    return f"{rom}: missing"


def build_command(manifest: str, artifacts: str) -> FrozenList[str]:
    return FrozenList((
        "cargo",
        "build",
        "--quiet",
        "--locked",
        "--profile",
        "verify",
        "--manifest-path",
        manifest,
        "--target-dir",
        artifacts,
        "-p",
        "tools",
    ))


def reader_unavailable(checker: str) -> str:
    return f"cache reader unavailable: {checker}"


def lists_lock_rows(status: int, output: str) -> bool:
    return status == 0 and POSITIVE.fullmatch(output.strip()) is not None


def no_lock_rows(checker: str) -> str:
    return f"cache reader returned no lock rows: {checker}"


def lock_rows_command(checker: str, lock: str) -> FrozenList[str]:
    return FrozenList((checker, "cache-check", lock))


@dataclass(frozen=True, slots=True)
class Recording:
    lock: str
    bundle: str
    rom: str


def cache_check_command(
    checker: str,
    recording: Recording,
    cache: str,
    row: str,
    watches: Sequence[str],
) -> FrozenList[str]:
    return FrozenList((
        checker,
        "cache-check",
        recording.lock,
        recording.bundle,
        cache,
        row,
        *watches,
    ))


def corpus_command(
    oracle: Sequence[str],
    recording: Recording,
    cache: str,
    row: str,
    watches: str,
) -> FrozenList[str]:
    watch = ("--watch-pcs", watches) if watches else ()
    return FrozenList((
        *oracle,
        "corpus",
        recording.rom,
        recording.lock,
        recording.bundle,
        "--cache",
        cache,
        "--only",
        row,
        *watch,
    ))


def invalid_row(row: str) -> str:
    return f"invalid cache row id: {row}"


def waits(row: str) -> str:
    return f"{row} waits for another recorder"


def no_pads(row: str) -> str:
    return f"{row} failed: no {row}.pads in the bundle"


def cached(row: str, frames: str) -> str:
    return f"{row} cached {frames} frames"


def oracle_exit(status: int) -> str:
    return f"oracle exit {status}\n"


def recorded(row: str, frames: str, seconds: int) -> str:
    return f"{row} recorded {frames} frames in {seconds} s"


def failed(row: str, seconds: int, log: str) -> str:
    return f"{row} failed after {seconds} s, see {log}"
