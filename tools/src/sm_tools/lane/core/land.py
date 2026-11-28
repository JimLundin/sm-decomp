import json
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import PurePath
from typing import cast

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

PR_TRIES = 6
POLL_SECONDS = 15
POLLS = 3 * 3600 // POLL_SECONDS
CACHES = FrozenOrderedSet((
    "__pycache__",
    ".mypy_cache",
    ".ruff_cache",
    ".venv",
    "uv.lock",
))
PENDING = "pending"
SUCCESS = "success"
HEAD_PREFIX = "branch refs/heads/"
WORKTREE_PREFIX = "worktree "
IGNORED = "!! "
RUN_FIELDS = FrozenList(("headSha", "status", "conclusion", "event"))


class LandingError(Exception):
    def __init__(self, message: str, code: int = 1) -> None:
        super().__init__(message)
        self.code: int = code


class RunListingError(LandingError):
    pass


@dataclass(frozen=True, slots=True)
class WorkflowRun:
    head: str
    status: str
    conclusion: str
    event: str


def workflow_run(row: object) -> WorkflowRun:
    fields: Mapping[str, object] = (
        cast("Mapping[str, object]", row) if isinstance(row, Mapping) else {}
    )
    values = FrozenList(fields.get(name) for name in RUN_FIELDS)
    texts = FrozenList(value for value in values if isinstance(value, str))
    if len(texts) != len(RUN_FIELDS):
        message = f"gh run list printed a run without {', '.join(RUN_FIELDS)}: {row!r}"
        raise RunListingError(message)
    return WorkflowRun(*texts)


def workflow_runs(text: str) -> FrozenList[WorkflowRun]:
    try:
        decoded = cast("object", json.loads(text))
    except json.JSONDecodeError as error:
        message = f"gh run list printed no JSON: {error}"
        raise RunListingError(message) from error
    if not isinstance(decoded, list):
        message = f"gh run list printed no list of runs: {text[:200]!r}"
        raise RunListingError(message)
    return FrozenList(
        workflow_run(row) for row in cast("list[object]", decoded)
    )


def ci_state(runs: str, head: str) -> str:
    for run in workflow_runs(runs):
        if run.head == head and run.event == "pull_request":
            return run.conclusion if run.status == "completed" else PENDING
    return PENDING


def held(status: str) -> str | None:
    lines = status.splitlines()
    if any(not line.startswith(IGNORED) for line in lines):
        return "it has uncommitted changes"
    ignored = FrozenList(
        line[3:]
        for line in lines
        if not line[3:].startswith("target/")
        and not CACHES & FrozenOrderedSet(line[3:].split("/"))
    )
    return (
        f"it has ignored files outside target/: {' '.join(ignored)}"
        if ignored
        else None
    )


def worktrees(listing: str) -> FrozenOrderedDict[str, PurePath]:
    found: dict[str, PurePath] = {}
    path = None
    for line in listing.splitlines():
        if line.startswith(WORKTREE_PREFIX):
            path = PurePath(line.removeprefix(WORKTREE_PREFIX))
        elif line.startswith(HEAD_PREFIX) and path:
            found[line.removeprefix(HEAD_PREFIX)] = path
    return FrozenOrderedDict(found)


def on(cherry: str) -> bool:
    return not any(line.startswith("+") for line in cherry.splitlines())


def conflict(
    commit: str, lane: str, files: Sequence[str], worktree: PurePath
) -> LandingError:
    return LandingError(
        f"CONFLICT {commit[:9]} from {lane}: {' '.join(files)}; resolve it in {worktree}, run git cherry-pick --continue, then run this again",
        2,
    )


def gate_line(status: int, stdout: str) -> str:
    lines = stdout.strip().splitlines()
    if status != 0 or not lines or not lines[-1].startswith("GATE OK"):
        raise LandingError("the gate failed:\n" + "\n".join(lines[-20:]))
    return lines[-1]


def landed_comment(head: str) -> str:
    return (
        f"Landed on main as a fast-forward to `{head[:9]}`; CI passed on this"
        " head."
    )
