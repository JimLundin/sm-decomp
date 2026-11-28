import argparse
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol
lazy from collections.abc import Callable, Sequence

from ordered import FrozenList, FrozenOrderedSet

from sm_tools.lane.core.land import (
    PENDING,
    POLL_SECONDS,
    POLLS,
    PR_TRIES,
    SUCCESS,
    LandingError,
    RunListingError,
    ci_state,
    conflict,
    gate_line,
    held,
    landed_comment,
    on,
    worktrees,
)
from sm_tools.lane.core.worktree import GATE
from sm_tools.lane.shell.scratch import scratch
from sm_tools.lane.shell.worktree import add, seed_dir
from sm_tools.shell.process import git as run_git

GH_SECONDS = 120


class GitHub(Protocol):
    def open_pr(self, branch: str) -> int | None: ...
    def create_pr(self, branch: str, title: str, body_file: Path) -> None: ...
    def pr_head(self, pr: int) -> str: ...
    def ci(self, branch: str, head: str) -> str: ...
    def comment(self, pr: int, body: str) -> None: ...


def gh(*arguments: str) -> str:
    try:
        done = subprocess.run(
            ("gh", *arguments),
            capture_output=True,
            text=True,
            timeout=GH_SECONDS,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        message = f"gh {arguments[0]} {arguments[1]} timed out"
        raise LandingError(message) from error
    if done.returncode != 0:
        message = f"gh {arguments[0]} {arguments[1]}: {done.stderr.strip()}"
        raise LandingError(message)
    return done.stdout


class Gh:
    @staticmethod
    def open_pr(branch: str) -> int | None:
        number = gh(
            "pr",
            "list",
            "--head",
            branch,
            "--json",
            "number",
            "--jq",
            ".[0].number",
        ).strip()
        return int(number) if number else None

    @staticmethod
    def create_pr(branch: str, title: str, body_file: Path) -> None:
        try:
            _ = gh(
                "pr",
                "create",
                "--base",
                "main",
                "--head",
                branch,
                "--title",
                title,
                "--body-file",
                str(body_file),
            )
        except LandingError as error:
            print(error, flush=True)

    @staticmethod
    def pr_head(pr: int) -> str:
        return gh(
            "pr", "view", str(pr), "--json", "headRefOid", "--jq", ".headRefOid"
        ).strip()

    @staticmethod
    def ci(branch: str, head: str) -> str:
        fields = "headSha,status,conclusion,event"
        listing = gh(
            "run",
            "list",
            "--branch",
            branch,
            "--workflow",
            "ci.yml",
            "--json",
            fields,
        )
        return ci_state(listing, head)

    @staticmethod
    def comment(pr: int, body: str) -> None:
        _ = gh("pr", "comment", str(pr), "--body", body)


def git(repo: Path, *arguments: str) -> str:
    done = run_git("-C", str(repo), *arguments)
    if done.returncode != 0:
        message = f"git {' '.join(arguments)}: {done.stderr.strip()}"
        raise LandingError(message)
    return done.stdout.strip()


def holds(repo: Path, *arguments: str) -> bool:
    return run_git("-C", str(repo), *arguments).returncode == 0


def tip(repo: Path, ref: str) -> str | None:
    done = run_git(
        "-C", str(repo), "rev-parse", "-q", "--verify", f"{ref}^{{commit}}"
    )
    return done.stdout.strip() if done.returncode == 0 else None


@dataclass(frozen=True, slots=True)
class Landing:
    repo: Path
    name: str
    lanes: FrozenList[str]
    after: str | None
    title: str | None = None
    body_file: Path | None = None

    @property
    def branch(self) -> str:
        return f"land/{self.name}"

    @property
    def worktree(self) -> Path:
        return scratch() / "collab" / f"land-{self.name}"


def base(landing: Landing) -> str:
    return git(
        landing.repo,
        "rev-parse",
        f"{landing.after or 'origin/main'}^{{commit}}",
    )


def identity(repo: Path, revisions: str) -> FrozenOrderedSet[str]:
    return FrozenOrderedSet(
        git(
            repo,
            "log",
            "--date=raw",
            "--format=%an%x00%ae%x00%ad%x00%s",
            revisions,
        ).splitlines()
    )


def start(landing: Landing) -> None:
    worktree = landing.worktree
    if not worktree.exists():
        try:
            status = add(
                landing.branch,
                worktree,
                base(landing),
                seed_dir(),
                landing.repo,
            )
        except (OSError, subprocess.CalledProcessError) as error:
            message = f"could not add the landing worktree {worktree}: {error}"
            raise LandingError(message) from error
        if status:
            message = f"could not add the landing worktree {worktree}"
            raise LandingError(message)
    elif tip(worktree, "CHERRY_PICK_HEAD"):
        message = f"finish or abort the cherry-pick in {worktree}, then run this again"
        raise LandingError(message, 2)


def pick_one(worktree: Path, commit: str, lane: str) -> None:
    if holds(worktree, "cherry-pick", "--ff", commit):
        print(f"picked {commit[:9]} {lane}", flush=True)
        return
    files = git(worktree, "diff", "--name-only", "--diff-filter=U").split()
    if (
        not files
        and tip(worktree, "CHERRY_PICK_HEAD")
        and holds(worktree, "diff", "--cached", "--quiet")
    ):
        _ = git(worktree, "cherry-pick", "--skip")
        print(
            f"skipped {commit[:9]} {lane}: main already has its change",
            flush=True,
        )
        return
    raise conflict(commit, lane, files, worktree)


def pick(landing: Landing) -> None:
    start(landing)
    worktree = landing.worktree
    picked = identity(worktree, f"{base(landing)}..HEAD")
    for lane in landing.lanes:
        picked |= identity(worktree, f"origin/{lane}..origin/main")
        revisions = git(
            worktree,
            "rev-list",
            "--reverse",
            "--right-only",
            "--cherry-pick",
            f"HEAD...origin/{lane}",
        )
        for commit in revisions.split():
            if identity(worktree, f"{commit}^!") <= picked:
                continue
            pick_one(worktree, commit, lane)


def gate(landing: Landing, command: Sequence[str]) -> str:
    environment = {**os.environ, "SM_GATE_BASE": base(landing)}
    done = subprocess.run(
        command,
        cwd=landing.worktree,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    print(gate_line(done.returncode, done.stdout), flush=True)
    return git(landing.worktree, "rev-parse", "HEAD")


def open_pr(
    github: GitHub, landing: Landing, body_file: Path, wait: Callable[[], None]
) -> int:
    for _ in range(PR_TRIES):
        pr = github.open_pr(landing.branch)
        if pr is not None:
            return pr
        github.create_pr(landing.branch, landing.title or "", body_file)
        wait()
    message = f"no PR for {landing.branch} after {PR_TRIES} tries"
    raise LandingError(message)


def ci_now(landing: Landing, github: GitHub, pr: int, head: str) -> str:
    _ = git(landing.repo, "fetch", "-q", "origin")
    if landing.after and not holds(
        landing.repo,
        "merge-base",
        "--is-ancestor",
        landing.after,
        "origin/main",
    ):
        print(f"#{pr} waits for {landing.after[:9]} to land", flush=True)
        return PENDING
    if github.pr_head(pr) != head:
        print(f"#{pr} waits for GitHub to show {head[:9]}", flush=True)
        return PENDING
    state = github.ci(landing.branch, head)
    print(f"#{pr} {head[:9]} ci={state}", flush=True)
    return state


def wait_for_ci(
    landing: Landing,
    github: GitHub,
    pr: int,
    head: str,
    wait: Callable[[], None],
) -> None:
    for _ in range(POLLS):
        try:
            state = ci_now(landing, github, pr, head)
        except RunListingError:
            raise
        except LandingError as error:
            print(f"#{pr} is not ready: {error}", flush=True)
            state = PENDING
        if state == SUCCESS:
            return
        if state != PENDING:
            message = f"the CI run on {head[:9]} ended {state}; read its log"
            raise LandingError(message, 4)
        wait()
    message = f"#{pr} timed out"
    raise LandingError(message, 5)


def remove_worktree(repo: Path, path: Path) -> None:
    shutil.rmtree(path / "target", ignore_errors=True)
    _ = git(repo, "worktree", "remove", "--force", str(path))
    _ = git(repo, "worktree", "prune")


def clean_local(
    repo: Path, branch: str, path: Path | None, target: str, *, dry_run: bool
) -> bool:
    verb = "would remove" if dry_run else "removed"
    local = tip(repo, f"refs/heads/{branch}")
    if not local:
        return True
    if not on(git(repo, "cherry", target, local)):
        print(f"kept {branch}: {local[:9]} is not on main", flush=True)
        return True
    if path:
        reason = held(
            git(
                path,
                "status",
                "--porcelain",
                "--ignored=matching",
                "--untracked-files=all",
            )
        )
        if reason:
            print(f"kept {path}: {reason}", flush=True)
            return False
        if not dry_run:
            remove_worktree(repo, path)
        print(f"{verb} worktree {path}", flush=True)
    if not dry_run:
        _ = git(repo, "update-ref", "-d", f"refs/heads/{branch}", local)
    print(f"{verb} {branch} at {local[:9]}", flush=True)
    return True


def clean_remote(
    repo: Path, branch: str, target: str, *, dry_run: bool
) -> None:
    verb = "would remove" if dry_run else "removed"
    remote = tip(repo, f"refs/remotes/origin/{branch}")
    if not remote:
        return
    if not on(git(repo, "cherry", target, remote)):
        print(f"kept origin/{branch}: {remote[:9]} is not on main", flush=True)
        return
    if not dry_run:
        _ = git(
            repo,
            "push",
            "-q",
            f"--force-with-lease=refs/heads/{branch}:{remote}",
            "origin",
            "--delete",
            branch,
        )
    print(f"{verb} origin/{branch} at {remote[:9]}", flush=True)


def clean(landing: Landing, target: str, *, dry_run: bool) -> None:
    repo = landing.repo
    checkouts = worktrees(git(repo, "worktree", "list", "--porcelain"))
    for branch in (*landing.lanes, landing.branch):
        path = checkouts.get(branch)
        if clean_local(
            repo, branch, Path(path) if path else None, target, dry_run=dry_run
        ):
            clean_remote(repo, branch, target, dry_run=dry_run)


def merge(
    landing: Landing,
    github: GitHub,
    head: str,
    body_file: Path,
    wait: Callable[[], None],
) -> None:
    _ = git(landing.worktree, "push", "-q", "-u", "origin", landing.branch)
    pr = open_pr(github, landing, body_file, wait)
    print(f"#{pr} opened for {landing.branch}", flush=True)
    wait_for_ci(landing, github, pr, head, wait)
    if not holds(
        landing.repo, "merge-base", "--is-ancestor", "origin/main", head
    ):
        message = "main moved; land again under a new name on the new main"
        raise LandingError(message, 3)
    _ = git(landing.repo, "push", "-q", "origin", f"{head}:refs/heads/main")
    print(f"landed #{pr} at {head[:9]}", flush=True)
    try:
        github.comment(pr, landed_comment(head))
    except LandingError as error:
        print(f"could not comment on #{pr}: {error}", flush=True)


def land(
    landing: Landing,
    github: GitHub,
    gate_command: Sequence[str],
    wait: Callable[[], None],
) -> None:
    _ = git(landing.repo, "fetch", "-q", "origin")
    pushed = tip(landing.repo, f"refs/remotes/origin/{landing.branch}")
    if pushed and holds(
        landing.repo, "merge-base", "--is-ancestor", pushed, "origin/main"
    ):
        print(f"{landing.branch} at {pushed[:9]} is on main", flush=True)
        clean(landing, "origin/main", dry_run=landing.title is None)
        return
    pick(landing)
    head = gate(landing, gate_command)
    if landing.title is None or landing.body_file is None:
        print(
            f"dry run: would push {landing.branch} at {head[:9]}, open its PR and land it",
            flush=True,
        )
        clean(landing, head, dry_run=True)
        return
    merge(landing, github, head, landing.body_file, wait)
    clean(landing, head, dry_run=False)


def parser() -> argparse.ArgumentParser:
    parsing = argparse.ArgumentParser(
        prog="land", description="Land reviewed lanes on main."
    )
    _ = parsing.add_argument("name", help="the landing branch is land/NAME")
    _ = parsing.add_argument(
        "lanes",
        nargs="+",
        metavar="lane",
        help="a reviewed branch on origin, in landing order",
    )
    _ = parsing.add_argument(
        "--after",
        help="build on this landing head and land only once it is on main",
    )
    _ = parsing.add_argument("--title")
    _ = parsing.add_argument("--body-file", type=Path)
    _ = parsing.add_argument(
        "--dry-run",
        action="store_true",
        help="pick and gate; push, open and land nothing",
    )
    return parsing


def wait_a_poll() -> None:
    time.sleep(POLL_SECONDS)


class Arguments(argparse.Namespace):
    name: str = ""
    lanes: Sequence[str] = ()
    after: str | None = None
    title: str | None = None
    body_file: Path | None = None
    dry_run: bool = False


def run(arguments: Sequence[str]) -> int:
    parsing = parser()
    parsed = parsing.parse_args(arguments, namespace=Arguments())
    if not parsed.dry_run and not (parsed.title and parsed.body_file):
        parsing.error("--title and --body-file are required unless --dry-run")
    repo = Path(
        git(
            Path.cwd(),
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        )
    ).parent
    title = None if parsed.dry_run else parsed.title
    landing = Landing(
        repo,
        parsed.name,
        FrozenList(parsed.lanes),
        parsed.after,
        title,
        parsed.body_file,
    )
    try:
        land(landing, Gh(), GATE, wait_a_poll)
    except LandingError as error:
        print(error, file=sys.stderr)
        return error.code
    return 0


def main() -> int:
    return run(sys.argv[1:])
