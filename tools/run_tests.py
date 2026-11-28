import os
import subprocess
import sys
import tempfile
import time
import unittest
from collections.abc import Iterator
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path

USAGE = """usage: run_tests.py [folder]

Runs every test unittest discovery finds under the folder, tools/ by default, with each test module
in its own process and as many processes at once as this process may use cores. Prints each
module's result, the output of each module that failed, and the count of tests run against the count
discovered. Fails naming each failed test, and when the counts differ."""


@dataclass(frozen=True)
class Outcome:
    module: str
    ran: int
    failed: list[str]
    output: str
    seconds: float


def cases(suite: unittest.TestSuite) -> Iterator[unittest.TestCase]:
    for test in suite:
        if isinstance(test, unittest.TestSuite):
            yield from cases(test)
        else:
            assert isinstance(test, unittest.TestCase)
            yield test


def report_module(folder: Path, module: str, results: Path) -> None:
    sys.path.insert(0, str(folder))
    result = unittest.TextTestRunner(verbosity=1).run(unittest.defaultTestLoader.loadTestsFromName(module))
    failed = [test.id() for test, _ in (*result.failures, *result.errors)]
    failed += [test.id() for test in result.unexpectedSuccesses]
    results.write_text("\n".join([str(result.testsRun), *failed]) + "\n")


def run_module(folder: Path, module: str) -> Outcome:
    started = time.monotonic()
    with tempfile.TemporaryDirectory() as scratch:
        results = Path(scratch, "results")
        run = subprocess.run([sys.executable, __file__, "--module", str(folder), module, str(results)],
                             stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                             check=False)
        lines = results.read_text().splitlines() if results.is_file() else []
    ran, failed = (int(lines[0]), lines[1:]) if lines else (0, [f"{module} (its process exited {run.returncode})"])
    return Outcome(module, ran, failed, run.stdout, time.monotonic() - started)


def main(folder: Path) -> int:
    loader = unittest.TestLoader()
    found = loader.discover(str(folder), top_level_dir=str(folder))
    if loader.errors:
        print(*loader.errors, sep="\n")
        print("run_tests: discovery failed")
        return 1
    counts: dict[str, int] = {}
    for case in cases(found):
        counts[type(case).__module__] = counts.get(type(case).__module__, 0) + 1
    discovered = sum(counts.values())
    ran = 0
    failed: list[str] = []
    workers = max(1, min(len(os.sched_getaffinity(0)), len(counts)))
    started = time.monotonic()
    with ThreadPoolExecutor(workers) as pool:
        for done in as_completed([pool.submit(run_module, folder, module) for module in counts]):
            outcome = done.result()
            ran += outcome.ran
            failed += outcome.failed
            verdict = "FAILED" if outcome.failed else "ok"
            print(f"{outcome.module}: {outcome.ran} tests {verdict} in {outcome.seconds:.1f}s", flush=True)
            if outcome.failed:
                print(outcome.output, flush=True)
    print(f"ran {ran} of {discovered} tests in {len(counts)} modules with {workers} processes "
          f"in {time.monotonic() - started:.1f}s")
    for test in sorted(failed):
        print(f"FAILED {test}")
    if ran != discovered:
        print(f"run_tests: ran {ran} tests but discovered {discovered}")
    return 1 if failed or ran != discovered else 0


if __name__ == "__main__":
    if sys.argv[1:2] == ["--module"] and len(sys.argv) == 5:
        report_module(Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4]))
    elif len(sys.argv) > 2 or sys.argv[1:2] in (["-h"], ["--help"]):
        print(USAGE)
        raise SystemExit(len(sys.argv) > 2)
    else:
        raise SystemExit(main(Path(sys.argv[1] if len(sys.argv) > 1 else Path(__file__).parent).resolve()))
