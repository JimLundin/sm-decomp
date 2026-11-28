import argparse
import sys
from pathlib import Path

from ordered import FrozenList

from sm_tools.ci.core.check_replay_transition import (
    DESCRIPTION,
    LEDGER,
    Outcome,
    checked,
    failed,
)

ROOT = Path(__file__).resolve().parents[5]


class Arguments(argparse.Namespace):
    root: Path = ROOT


def outcome(root: Path) -> Outcome:
    directory = root / "scenarios/replay"
    try:
        sources = FrozenList(
            (directory / name).read_bytes().decode() for name in LEDGER
        )
    except (OSError, ValueError) as error:
        return failed(error)
    return checked(sources)


def check(root: Path) -> int:
    result = outcome(root)
    print(
        result.line,
        file=sys.stderr if result.status else sys.stdout,
        flush=True,
    )
    return result.status


def main() -> int:
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    _ = parser.add_argument("--root", type=Path, default=ROOT)
    return check(parser.parse_args(namespace=Arguments()).root)
