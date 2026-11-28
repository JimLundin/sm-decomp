import os
import subprocess
import sys
import tempfile
from pathlib import Path

from ordered import FrozenList

from sm_tools.ci.core.replay import (
    HEAD,
    USAGE,
    case,
    merge,
    missing_input,
    part_report,
    parts,
    reproduce,
)
from sm_tools.ci.shell import check_replay_transition


def transition() -> int:
    return check_replay_transition.check(check_replay_transition.ROOT)


def run_case(name: str, extra: FrozenList[str]) -> int:
    if status := transition():
        return status
    head = subprocess.run(
        HEAD, capture_output=True, text=True, check=False
    ).stdout.strip()
    print(reproduce(head, name), flush=True)
    return subprocess.run(case(os.environ, name, extra), check=False).returncode


def reported(logs: Path, label: str, process: subprocess.Popen[bytes]) -> int:
    status = process.wait()
    log = (logs / f"{label}.log").read_text(encoding="utf-8", errors="replace")
    print(part_report(label, status, log), flush=True)
    return status


def run_baseline(receipt: str) -> int:
    if missing := missing_input(os.environ):
        print(missing, file=sys.stderr)
        return 2
    if status := transition():
        return status
    with tempfile.TemporaryDirectory() as directory:
        logs = Path(directory)
        running: list[tuple[str, subprocess.Popen[bytes]]] = []
        for label, command in parts(os.environ, logs).items():
            with (logs / f"{label}.log").open("w") as log:
                running.append((
                    label,
                    subprocess.Popen(
                        command, stdout=log, stderr=subprocess.STDOUT
                    ),
                ))
        statuses = FrozenList(
            reported(logs, label, process) for label, process in running
        )
        if any(statuses):
            return 1
        return subprocess.run(
            merge(os.environ, logs, receipt), check=False
        ).returncode


def main() -> int:
    match FrozenList(sys.argv[1:]):
        case ["baseline", receipt]:
            return run_baseline(receipt)
        case ["case", name, *extra]:
            return run_case(name, FrozenList(extra))
        case _:
            print(USAGE, file=sys.stderr)
            return 2
