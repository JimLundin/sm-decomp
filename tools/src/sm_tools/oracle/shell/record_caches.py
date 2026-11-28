import ctypes
import fcntl
import functools
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
lazy from collections.abc import Mapping, Sequence
lazy from typing import TextIO

from ordered import FrozenList

from sm_tools.oracle.core.record_caches import (
    BUNDLE_PARTS,
    JOBS_REFUSAL,
    POSITIVE,
    ROW,
    STANDARD_INPUT,
    USAGE,
    Recording,
    build_command,
    cache_check_command,
    cached,
    corpus_command,
    failed,
    invalid_row,
    jobs_of,
    lists_lock_rows,
    lock_rows_command,
    missing_bundle_part,
    missing_rom,
    no_lock_rows,
    no_pads,
    oracle_exit,
    reader_unavailable,
    recorded,
    rows_of,
    usable_arguments,
    waits,
)

ROOT = Path(__file__).resolve().parents[5]
RENAME_EXCHANGE = 2
AT_FDCWD = -100
AMBIENT = os.environ


@dataclass(frozen=True, slots=True)
class Batch:
    bundle: Path
    rom: Path
    cache: Path
    lock: Path
    oracle: FrozenList[str]
    checker: str
    environment: Mapping[str, str]

    def recording(self) -> Recording:
        return Recording(str(self.lock), str(self.bundle), str(self.rom))

    def cache_check(
        self, cache: Path, row: str, *watches: str, log: Path | None = None
    ) -> str | None:
        command = cache_check_command(
            self.checker, self.recording(), str(cache), row, watches
        )
        with (
            log.open("a", encoding="utf-8")
            if log
            else Path(os.devnull).open("w", encoding="utf-8")
        ) as errors:
            done = subprocess.run(
                command,
                stdout=subprocess.PIPE,
                stderr=errors,
                text=True,
                check=False,
                env=self.environment,
            )
        return done.stdout.strip() if done.returncode == 0 else None


def exchange(source: Path, target: Path) -> None:
    rename = ctypes.CDLL(None, use_errno=True).renameat2
    rename.argtypes = (
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    )
    if rename(
        AT_FDCWD,
        os.fsencode(source),
        AT_FDCWD,
        os.fsencode(target),
        RENAME_EXCHANGE,
    ):
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), str(target))


def publish(source: Path, target: Path) -> None:
    if target.exists() or target.is_symlink():
        exchange(source, target)
    else:
        _ = source.rename(target)


def record(batch: Batch, row: str) -> bool:
    if not ROW.fullmatch(row):
        print(invalid_row(row), file=sys.stderr, flush=True)
        return False
    with (batch.cache / ".locks" / row).open("a", encoding="utf-8") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print(waits(row), flush=True)
            fcntl.flock(lock, fcntl.LOCK_EX)
        return record_locked(batch, row)


def record_locked(batch: Batch, row: str) -> bool:
    target = batch.cache / row
    log = batch.cache / ".logs" / f"{row}.log"
    if not (batch.bundle / "pads" / f"{row}.pads").is_file():
        print(no_pads(row), flush=True)
        return False
    requested = batch.environment.get("WATCH_PCS", "")
    if (target / "lean.txt").is_file():
        if (
            frames := batch.cache_check(batch.cache, row, requested)
        ) is not None:
            print(cached(row, frames), flush=True)
            return True
        combined = batch.cache_check(
            batch.cache, row, "--carry-watches", requested
        )
        if combined is not None:
            requested = combined
    started = time.monotonic()
    with tempfile.TemporaryDirectory(
        prefix=".record.", dir=batch.cache
    ) as stage:
        staged = Path(stage) / "cache"
        command = corpus_command(
            batch.oracle, batch.recording(), str(staged), row, requested
        )
        with log.open("w", encoding="utf-8") as output:
            status = subprocess.run(
                command,
                stdout=output,
                stderr=subprocess.STDOUT,
                check=False,
                env=batch.environment,
            ).returncode
            if status != 0:
                _ = output.write(oracle_exit(status))
        seconds = round(time.monotonic() - started)
        if (
            status == 0
            and (frames := batch.cache_check(staged, row, requested, log=log))
            is not None
        ):
            publish(staged / row, target)
            print(recorded(row, frames, seconds), flush=True)
            return True
    print(failed(row, seconds, str(log)), flush=True)
    return False


def cache_reader(environment: Mapping[str, str]) -> str:
    artifacts = Path(environment.get("CARGO_TARGET_DIR") or ROOT / "target")
    if checker := environment.get("SM_TOOLS"):
        return checker
    _ = subprocess.run(
        build_command(str(ROOT / "Cargo.toml"), str(artifacts)),
        check=True,
        env=environment,
    )
    return str(artifacts / "verify/tools")


def missing_input(bundle: Path, rom: Path) -> str | None:
    for part in BUNDLE_PARTS:
        if not (bundle / part).is_dir():
            return missing_bundle_part(str(bundle / part))
    return None if rom.is_file() else missing_rom(str(rom))


def reader_refusal(
    checker: str, lock: Path, environment: Mapping[str, str]
) -> str | None:
    if not shutil.which(checker, path=environment.get("PATH")):
        return reader_unavailable(checker)
    listed = subprocess.run(
        lock_rows_command(checker, str(lock)),
        stdout=subprocess.PIPE,
        text=True,
        check=False,
        env=environment,
    )
    if not lists_lock_rows(listed.returncode, listed.stdout):
        return no_lock_rows(checker)
    return None


def record_rows(
    arguments: Sequence[str],
    rows: Sequence[str],
    environment: Mapping[str, str],
) -> int:
    bundle, rom, cache = (Path(argument) for argument in arguments[1:4])
    lock = Path(
        environment.get("SM_LOCK") or ROOT / "scenarios/locks/lsnes.toml"
    )
    checker = cache_reader(environment)
    if refusal := reader_refusal(checker, lock, environment):
        print(refusal, file=sys.stderr)
        return 2
    oracle = FrozenList(
        shlex.split(
            environment.get("ORACLE") or str(ROOT / "target/verify/oracle")
        )
    )
    work = Batch(bundle, rom, cache, lock, oracle, checker, environment)
    with ThreadPoolExecutor(max_workers=int(jobs_of(arguments))) as workers:
        done = FrozenList(workers.map(functools.partial(record, work), rows))
    return 0 if all(done) else 1


def batch(
    arguments: Sequence[str], stdin: TextIO, environment: Mapping[str, str]
) -> int:
    if not usable_arguments(arguments):
        print(USAGE, file=sys.stderr)
        return 2
    jobs = jobs_of(arguments)
    if not POSITIVE.fullmatch(jobs):
        print(JOBS_REFUSAL, file=sys.stderr)
        return 2
    rows_file, bundle = arguments[0], Path(arguments[1])
    rom, cache = Path(arguments[2]), Path(arguments[3])
    if missing := missing_input(bundle, rom):
        print(missing, file=sys.stderr)
        return 1
    (cache / ".logs").mkdir(parents=True, exist_ok=True)
    (cache / ".locks").mkdir(exist_ok=True)
    source = (
        stdin.read()
        if rows_file == STANDARD_INPUT
        else Path(rows_file).read_text(encoding="utf-8")
    )
    rows = rows_of(source)
    if not rows:
        return 0
    return record_rows(arguments, rows, environment)


def main() -> int:
    return batch(sys.argv[1:], sys.stdin, AMBIENT)
