import ctypes
import fcntl
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path

USAGE = """usage: record_caches.py ROWS BUNDLE ROM CACHE [JOBS]

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
ROOT = Path(__file__).resolve().parents[2]
ROW = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]*")
RENAME_EXCHANGE = 2
AT_FDCWD = -100


@dataclass(frozen=True)
class Batch:
    bundle: Path
    rom: Path
    cache: Path
    lock: Path
    oracle: list[str]
    checker: str

    def cache_check(self, cache: Path, row: str, *watches: str, log: Path | None = None) -> str | None:
        with open(log, "a") if log else open(os.devnull, "w") as errors:
            done = subprocess.run([self.checker, "cache-check", str(self.lock), str(self.bundle), str(cache), row,
                                   *watches], stdout=subprocess.PIPE, stderr=errors, text=True, check=False)
        return done.stdout.strip() if done.returncode == 0 else None


def exchange(source: Path, target: Path) -> None:
    rename = ctypes.CDLL(None, use_errno=True).renameat2
    rename.argtypes = (ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint)
    if rename(AT_FDCWD, os.fsencode(source), AT_FDCWD, os.fsencode(target), RENAME_EXCHANGE):
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), str(target))


def publish(source: Path, target: Path) -> None:
    if target.exists() or target.is_symlink():
        exchange(source, target)
    else:
        source.rename(target)


def record(batch: Batch, row: str) -> bool:
    if not ROW.fullmatch(row):
        print(f"invalid cache row id: {row}", file=sys.stderr, flush=True)
        return False
    with open(batch.cache / ".locks" / row, "a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print(f"{row} waits for another recorder", flush=True)
            fcntl.flock(lock, fcntl.LOCK_EX)
        return record_locked(batch, row)


def record_locked(batch: Batch, row: str) -> bool:
    target = batch.cache / row
    log = batch.cache / ".logs" / f"{row}.log"
    if not (batch.bundle / "pads" / f"{row}.pads").is_file():
        print(f"{row} failed: no {row}.pads in the bundle", flush=True)
        return False
    requested = os.environ.get("WATCH_PCS", "")
    if (target / "lean.txt").is_file():
        if (frames := batch.cache_check(batch.cache, row, requested)) is not None:
            print(f"{row} cached {frames} frames", flush=True)
            return True
        if (combined := batch.cache_check(batch.cache, row, "--carry-watches", requested)) is not None:
            requested = combined
    watch = ["--watch-pcs", requested] if requested else []
    started = time.monotonic()
    with tempfile.TemporaryDirectory(prefix=".record.", dir=batch.cache) as stage:
        staged = Path(stage) / "cache"
        with log.open("w") as output:
            status = subprocess.run([*batch.oracle, "corpus", str(batch.rom), str(batch.lock), str(batch.bundle),
                                     "--cache", str(staged), "--only", row, *watch],
                                    stdout=output, stderr=subprocess.STDOUT, check=False).returncode
            if status != 0:
                output.write(f"oracle exit {status}\n")
        seconds = round(time.monotonic() - started)
        if status == 0 and (frames := batch.cache_check(staged, row, requested, log=log)) is not None:
            publish(staged / row, target)
            print(f"{row} recorded {frames} frames in {seconds} s", flush=True)
            return True
    print(f"{row} failed after {seconds} s, see {log}", flush=True)
    return False


def rows_of(text: str) -> list[str]:
    rows = ("".join(line.split("#", 1)[0].split()) for line in text.splitlines())
    return list(dict.fromkeys(row for row in rows if row))


def main(arguments: list[str]) -> int:
    if not 4 <= len(arguments) <= 5:
        print(USAGE, file=sys.stderr)
        return 2
    rows_file, bundle, rom, cache = arguments[0], Path(arguments[1]), Path(arguments[2]), Path(arguments[3])
    jobs = arguments[4] if len(arguments) == 5 else "8"
    if not re.fullmatch(r"[1-9][0-9]*", jobs):
        print("jobs must be a positive integer", file=sys.stderr)
        return 2
    for directory in (bundle / "pads", bundle / "seeds"):
        if not directory.is_dir():
            print(f"{directory}: missing; BUNDLE is the relock bundle's relock-v2-lsnes/", file=sys.stderr)
            return 1
    if not rom.is_file():
        print(f"{rom}: missing", file=sys.stderr)
        return 1
    (cache / ".logs").mkdir(parents=True, exist_ok=True)
    (cache / ".locks").mkdir(exist_ok=True)
    rows = rows_of(sys.stdin.read() if rows_file == "-" else Path(rows_file).read_text())
    if not rows:
        return 0
    lock = Path(os.environ.get("SM_LOCK") or ROOT / "scenarios/locks/lsnes.toml")
    artifacts = Path(os.environ.get("CARGO_TARGET_DIR") or ROOT / "target")
    checker = os.environ.get("SM_TOOLS") or str(artifacts / "verify/tools")
    if not os.environ.get("SM_TOOLS"):
        subprocess.run(["cargo", "build", "--quiet", "--locked", "--profile", "verify", "--manifest-path",
                        str(ROOT / "Cargo.toml"), "--target-dir", str(artifacts), "-p", "tools"], check=True)
    if not shutil.which(checker):
        print(f"cache reader unavailable: {checker}", file=sys.stderr)
        return 2
    listed = subprocess.run([checker, "cache-check", str(lock)], stdout=subprocess.PIPE, text=True, check=False)
    if listed.returncode != 0 or not re.fullmatch(r"[1-9][0-9]*", listed.stdout.strip()):
        print(f"cache reader returned no lock rows: {checker}", file=sys.stderr)
        return 2
    oracle = shlex.split(os.environ.get("ORACLE") or str(ROOT / "target/verify/oracle"))
    batch = Batch(bundle, rom, cache, lock, oracle, checker)
    with ThreadPoolExecutor(max_workers=int(jobs)) as workers:
        recorded = list(workers.map(lambda row: record(batch, row), rows))
    return 0 if all(recorded) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
