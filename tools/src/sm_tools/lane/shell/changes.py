import sys
lazy from collections.abc import Sequence

lazy from ordered import FrozenList

from sm_tools.lane.core.changes import (
    MOST_ARGUMENTS,
    UNCOMMITTED,
    USAGE,
    committed,
    committed_under,
    groups,
    listed,
    listed_changes,
    outputs,
    uncommitted_under,
)
from sm_tools.shell.process import git, git_out


def changed_files(base: str) -> FrozenList[str]:
    return listed(git_out(*committed(base)), git_out(*UNCOMMITTED))


def changed_since(base: str, paths: Sequence[str]) -> bool:
    if git(*committed_under(base, paths)).returncode != 0:
        return True
    status = git(*uncommitted_under(paths))
    return listed_changes(status.returncode, status.stdout)


def run(arguments: Sequence[str]) -> int:
    if len(arguments) > MOST_ARGUMENTS:
        print(USAGE, file=sys.stderr)
        return 2
    paths = changed_files(arguments[0]) if arguments else None
    print("\n".join(outputs(groups(paths))))
    return 0


def main() -> int:
    return run(sys.argv[1:])
