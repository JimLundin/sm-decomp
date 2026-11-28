from collections import Counter
from dataclasses import dataclass, field

from ordered import FrozenList, FrozenOrderedDict

USAGE = """usage: run-tests [folder]

Runs every test pytest collects in the folder, the tools project by default, with each test module
in its own process and as many processes at once as this process may use cores. Prints each
module's result, the output of each module that failed, the count of tests run against the count
collected and the count skipped. Fails naming each failed test, each module whose process exited
non-zero, and each module that ran a different count of tests than collection found in it."""


@dataclass(frozen=True, slots=True)
class Results:
    ran: int = 0
    skipped: int = 0
    failed: FrozenList[str] = field(default_factory=FrozenList[str])


@dataclass(frozen=True, slots=True)
class Run:
    status: int
    results: Results
    output: str
    seconds: float


@dataclass(frozen=True, slots=True)
class Outcome:
    module: str
    run: Run
    failed: FrozenList[str]


def collected(listing: str) -> FrozenOrderedDict[str, int]:
    modules = Counter(
        line.partition("::")[0]
        for line in listing.splitlines()
        if "::" in line and not line.startswith(" ")
    )
    return FrozenOrderedDict(sorted(modules.items()))


def results(text: str) -> Results:
    match text.splitlines():
        case [ran, skipped, *failed]:
            return Results(int(ran), int(skipped), FrozenList(failed))
        case _:
            return Results()


def written(ran: int, skipped: int, failed: FrozenList[str]) -> str:
    return "".join(f"{line}\n" for line in (str(ran), str(skipped), *failed))


def outcome(module: str, discovered: int, run: Run) -> Outcome:
    found = run.results
    exited = f"{module} (its process exited {run.status})"
    short = f"{module} (ran {found.ran} of {discovered} tests)"
    failed = FrozenList((
        *found.failed,
        *((exited,) if run.status else ()),
        *((short,) if found.ran != discovered else ()),
    ))
    return Outcome(module, run, failed)


def module_line(done: Outcome) -> str:
    verdict = "FAILED" if done.failed else "ok"
    ran = done.run.results.ran
    return f"{done.module}: {ran} tests {verdict} in {done.run.seconds:.1f}s"


def summary(
    outcomes: FrozenList[Outcome], discovered: int, workers: int, seconds: float
) -> FrozenList[str]:
    ran = sum(done.run.results.ran for done in outcomes)
    skipped = sum(done.run.results.skipped for done in outcomes)
    failed = sorted(test for done in outcomes for test in done.failed)
    counts = f"ran {ran} of {discovered} tests, {skipped} skipped"
    spread = f"in {len(outcomes)} modules with {workers} processes"
    return FrozenList((
        f"{counts}, {spread} in {seconds:.1f}s",
        *(f"FAILED {test}" for test in failed),
    ))
