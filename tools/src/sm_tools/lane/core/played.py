lazy from collections.abc import Sequence

from ordered import FrozenList

USAGE = """usage: played [<row> | <row>/<start>-<end>]...

Plays the reviewed runs of scenarios/replay/played.toml, every run without names, in one build
and one test process. Each run prints its held cuts and stop beside the file's, and the exit is
nonzero when any differs. Needs SM_CACHE and SM_ROM; SM_LOCK defaults to the lsnes lock."""
TEST = "replay::played::tests::reviewed_runs_hold"
REQUIRED = FrozenList(("SM_CACHE", "SM_ROM"))
LOCK = "scenarios/locks/lsnes.toml"
COMMAND = FrozenList((
    "cargo",
    "test",
    "-p",
    "verify",
    "--lib",
    "--",
    "--ignored",
    "--exact",
    TEST,
    "--nocapture",
))
RAN_ALONE = "test result: ok. 1 passed; 0 failed; 0 ignored"
NOT_ALONE = f"played: {TEST} did not run as the one test"
NOT_ALONE_STATUS = 4


def refused(arguments: Sequence[str]) -> bool:
    return any(argument.startswith("-") for argument in arguments)


def ran_alone(lines: Sequence[str]) -> bool:
    return f"test {TEST} ... ok" in lines and any(
        line.startswith(RAN_ALONE) for line in lines
    )
