import os
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
lazy from collections.abc import Iterator
lazy from concurrent.futures import Future

from ordered import FrozenList

from sm_tools.ci.core.run_tests import (
    USAGE,
    Outcome,
    Results,
    Run,
    collected,
    module_line,
    outcome,
    results,
    summary,
)

PROJECT = Path(__file__).resolve().parents[4]
PYTEST = FrozenList((
    sys.executable,
    "-m",
    "pytest",
    "-p",
    "no:cacheprovider",
    "--rootdir=.",
))
PLUGIN = "sm_tools.ci.shell.results_plugin"
HELP = FrozenList((FrozenList(("-h",)), FrozenList(("--help",))))


def run_module(folder: Path, module: str, discovered: int) -> Outcome:
    started = time.monotonic()
    with tempfile.TemporaryDirectory() as scratch:
        written = Path(scratch, "results")
        run = subprocess.run(
            (*PYTEST, "-q", "-p", PLUGIN, f"--results={written}", module),
            cwd=folder,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
        )
        found = (
            results(written.read_text(encoding="utf-8"))
            if written.is_file()
            else Results()
        )
    seconds = time.monotonic() - started
    return outcome(
        module, discovered, Run(run.returncode, found, run.stdout, seconds)
    )


def reported(pending: FrozenList[Future[Outcome]]) -> Iterator[Outcome]:
    for finished in as_completed(pending):
        done = finished.result()
        print(module_line(done), flush=True)
        if done.failed:
            print(done.run.output, flush=True)
        yield done


def run(folder: Path) -> int:
    listing = subprocess.run(
        (
            *PYTEST,
            "--collect-only",
            "-q",
            *(() if folder == PROJECT else (".",)),
        ),
        cwd=folder,
        capture_output=True,
        text=True,
        check=False,
    )
    if listing.returncode != 0:
        print(listing.stdout + listing.stderr)
        print("run-tests: collection failed")
        return 1
    counts = collected(listing.stdout)
    workers = max(1, min(len(os.sched_getaffinity(0)), len(counts)))
    started = time.monotonic()
    with ThreadPoolExecutor(workers) as pool:
        pending = FrozenList(
            pool.submit(run_module, folder, module, count)
            for module, count in counts.items()
        )
        outcomes = FrozenList(reported(pending))
    lines = summary(
        outcomes, sum(counts.values()), workers, time.monotonic() - started
    )
    print("\n".join(lines))
    return 1 if any(done.failed for done in outcomes) else 0


def main() -> int:
    arguments = FrozenList(sys.argv[1:])
    if arguments in HELP or len(arguments) > 1:
        print(USAGE)
        return int(len(arguments) > 1)
    return run(Path(arguments[0]).resolve() if arguments else PROJECT)
