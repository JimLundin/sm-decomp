import os
import subprocess
import sys
from pathlib import Path
lazy from collections.abc import Mapping, Sequence
lazy from typing import IO

from ordered import FrozenList

from sm_tools.lane.core.played import (
    COMMAND,
    LOCK,
    NOT_ALONE,
    NOT_ALONE_STATUS,
    REQUIRED,
    USAGE,
    ran_alone,
    refused,
)
from sm_tools.shell.process import git_out


def play(
    top: Path, environment: Mapping[str, str]
) -> tuple[int, FrozenList[str]]:
    lines: list[str] = []
    with subprocess.Popen(
        COMMAND,
        cwd=top,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        errors="replace",
    ) as test:
        stdout: IO[str] | None = test.stdout
        for line in stdout or ():
            print(line, end="", flush=True)
            lines.append(line.rstrip("\n"))
    return test.returncode, FrozenList(lines)


def run(arguments: Sequence[str]) -> int:
    if refused(arguments):
        print(USAGE, file=sys.stderr)
        return 2
    for name in REQUIRED:
        if not os.environ.get(name):
            print(f"played: set {name}", file=sys.stderr)
            return 2
    top = Path(git_out("rev-parse", "--show-toplevel").strip())
    environment = os.environ | {
        "SM_LOCK": os.environ.get("SM_LOCK") or str(top / LOCK),
        "CARGO_PROFILE_TEST_OPT_LEVEL": os.environ.get(
            "CARGO_PROFILE_TEST_OPT_LEVEL"
        )
        or "3",
        "SM_PLAYED": " ".join(arguments),
    }
    status, lines = play(top, environment)
    if status != 0:
        return status
    if not ran_alone(lines):
        print(NOT_ALONE, file=sys.stderr)
        return NOT_ALONE_STATUS
    return 0


def main() -> int:
    return run(sys.argv[1:])
