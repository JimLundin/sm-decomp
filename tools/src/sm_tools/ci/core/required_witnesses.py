import json
import re
from dataclasses import dataclass
from typing import cast

from ordered import FrozenList, FrozenOrderedSet

from sm_tools.ci.core.decoded import field, table, text, toml

DESCRIPTION = (
    "Run the required ignored witnesses or one exact verification library test."
)
TEST_HELP = "exact verify library test name"
NAME_ERROR = "expected a fully qualified verify library test name"
NAME = re.compile(r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)+")
SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; \d+ filtered out;.*$",
    re.MULTILINE,
)
LIST_SUMMARY = re.compile(r"^(\d+) tests?, \d+ benchmarks?$", re.MULTILINE)
BUILD = FrozenList((
    "cargo",
    "test",
    "--locked",
    "--profile",
    "verify",
    "-p",
    "verify",
    "--lib",
    "--no-run",
    "--message-format=json-render-diagnostics",
))
LISTED = ": test"


@dataclass(frozen=True, slots=True)
class Verdict:
    status: int
    summary: str
    counts: FrozenList[str]
    label: str


VERDICTS = FrozenList((
    Verdict(0, "ok", FrozenList(("1", "0", "0", "0")), "PASS"),
    Verdict(101, "FAILED", FrozenList(("0", "1", "0", "0")), "TEST FAILURE"),
))


@dataclass(frozen=True, slots=True)
class Completion:
    passed: bool
    lines: FrozenList[str]


def required(manifest: str) -> FrozenList[str]:
    groups = toml(manifest)
    if not groups or any(
        not isinstance(group, list) or not group for group in groups.values()
    ):
        message = "required witness groups must be nonempty arrays"
        raise ValueError(message)
    names = FrozenList(
        name
        for group in groups.values()
        for name in cast("list[object]", group)
    )
    if any(
        not isinstance(name, str) or not NAME.fullmatch(name) for name in names
    ):
        message = "required witnesses must be exact libtest names"
        raise ValueError(message)
    if len(names) != len(FrozenOrderedSet(names)):
        message = "duplicate required witness"
        raise ValueError(message)
    return FrozenList(text(name) for name in names)


def inclusion(*, selected: bool) -> str:
    return "--include-ignored" if selected else "--ignored"


def build_failure(status: int) -> str:
    return f"BUILD FAILURE (cargo exit {status})"


def is_verify_test(message: object) -> bool:
    return (
        field(message, "reason") == "compiler-artifact"
        and field(field(message, "target"), "name") == "verify"
        and bool(field(field(message, "profile"), "test"))
        and bool(field(message, "executable"))
    )


def executable(cargo_messages: str) -> str:
    executables = FrozenList(
        text(field(message, "executable"))
        for message in (
            table(cast("object", json.loads(line)))
            for line in cargo_messages.splitlines()
        )
        if is_verify_test(message)
    )
    if len(executables) != 1:
        message = "cargo did not report exactly one verify test executable"
        raise ValueError(message)
    return executables[0]


def discovery_failure(status: int) -> str:
    return f"libtest discovery failed (exit {status})"


def available(listing: str) -> FrozenOrderedSet[str]:
    tests = FrozenOrderedSet(
        line.removesuffix(LISTED)
        for line in listing.splitlines()
        if line.endswith(LISTED)
    )
    if LIST_SUMMARY.findall(listing) != [str(len(tests))]:
        message = "libtest did not report a complete test list"
        raise ValueError(message)
    return tests


def missing(
    names: FrozenList[str], tests: FrozenOrderedSet[str], *, selected: bool
) -> FrozenList[str]:
    label = "MISSING TEST" if selected else "MISSING REQUIRED WITNESS"
    return FrozenList(f"{label} {name}" for name in names if name not in tests)


def single_test(name: str, included: str) -> FrozenList[str]:
    return FrozenList((name, "--exact", included, "--color", "never"))


def verdict_without_output(name: str, status: int, stdout: str) -> Completion:
    summary = SUMMARY.findall(stdout)
    for verdict in VERDICTS:
        if (
            status == verdict.status
            and summary == [(verdict.summary, *verdict.counts)]
            and f"test {name} ... {verdict.summary}" in stdout.splitlines()
        ):
            reproduce = FrozenList((
                f"Reproduce with uv run --locked --project tools required-witnesses {name}",
            ))
            return Completion(
                passed=status == 0,
                lines=FrozenList((
                    f"{verdict.label} {name}",
                    *(reproduce if status else ()),
                )),
            )
    message = (
        f"{name}: libtest did not complete the required test (exit {status})"
    )
    raise ValueError(message)


def passed(results: FrozenList[bool], *, selected: bool) -> str:
    label = "Selected tests" if selected else "Required witnesses"
    return f"{label} passed: {sum(results)}/{len(results)}"
