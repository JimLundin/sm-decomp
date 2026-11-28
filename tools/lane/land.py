import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

HERE = Path(__file__).resolve().parent
PR_TRIES = 6
POLLS = 90
CACHES = {"__pycache__", ".mypy_cache", ".ruff_cache", ".venv", "uv.lock"}


class Failed(Exception):
    def __init__(self, message: str, code: int = 1) -> None:
        super().__init__(message)
        self.code = code


class GitHub(Protocol):
    def open_pr(self, branch: str) -> int | None: ...
    def create_pr(self, branch: str, title: str, body_file: Path) -> None: ...
    def pr_head(self, pr: int) -> str: ...
    def ci(self, branch: str, head: str) -> str: ...
    def comment(self, pr: int, body: str) -> None: ...


def gh(*args: str) -> str:
    try:
        done = subprocess.run(["gh", *args], capture_output=True, text=True, timeout=120, check=False)
    except subprocess.TimeoutExpired as error:
        raise Failed(f"gh {args[0]} {args[1]} timed out") from error
    if done.returncode != 0:
        raise Failed(f"gh {args[0]} {args[1]}: {done.stderr.strip()}")
    return done.stdout


def ci_state(runs: str, head: str) -> str:
    for run in json.loads(runs):
        if run["headSha"] == head and run["event"] == "pull_request":
            return str(run["conclusion"]) if run["status"] == "completed" else "pending"
    return "pending"


class Gh:
    def open_pr(self, branch: str) -> int | None:
        number = gh("pr", "list", "--head", branch, "--json", "number", "--jq", ".[0].number").strip()
        return int(number) if number else None

    def create_pr(self, branch: str, title: str, body_file: Path) -> None:
        try:
            gh("pr", "create", "--base", "main", "--head", branch, "--title", title, "--body-file", str(body_file))
        except Failed as error:
            print(error, flush=True)

    def pr_head(self, pr: int) -> str:
        return gh("pr", "view", str(pr), "--json", "headRefOid", "--jq", ".headRefOid").strip()

    def ci(self, branch: str, head: str) -> str:
        fields = "headSha,status,conclusion,event"
        return ci_state(gh("run", "list", "--branch", branch, "--workflow", "ci.yml", "--json", fields), head)

    def comment(self, pr: int, body: str) -> None:
        gh("pr", "comment", str(pr), "--body", body)


def git(repo: Path, *args: str) -> str:
    done = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, check=False)
    if done.returncode != 0:
        raise Failed(f"git {' '.join(args)}: {done.stderr.strip()}")
    return done.stdout.strip()


def holds(repo: Path, *args: str) -> bool:
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, check=False).returncode == 0


def tip(repo: Path, ref: str) -> str | None:
    done = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "-q", "--verify", f"{ref}^{{commit}}"],
        capture_output=True,
        text=True,
        check=False,
    )
    return done.stdout.strip() if done.returncode == 0 else None


def on(repo: Path, commit: str, target: str) -> bool:
    return not any(line.startswith("+") for line in git(repo, "cherry", target, commit).splitlines())


@dataclass(frozen=True)
class Landing:
    repo: Path
    name: str
    lanes: tuple[str, ...]
    after: str | None
    title: str | None = None
    body_file: Path | None = None

    @property
    def branch(self) -> str:
        return f"land/{self.name}"

    @property
    def worktree(self) -> Path:
        return Path(os.environ.get("TEMP") or os.environ.get("TMPDIR") or "/tmp") / "collab" / f"land-{self.name}"


def base(landing: Landing) -> str:
    return git(landing.repo, "rev-parse", f"{landing.after or 'origin/main'}^{{commit}}")


def identity(repo: Path, revisions: str) -> set[str]:
    return set(git(repo, "log", "--date=raw", "--format=%an%x00%ae%x00%ad%x00%s", revisions).splitlines())


def pick(landing: Landing) -> None:
    worktree = landing.worktree
    if not worktree.exists():
        add = [sys.executable, str(HERE / "worktree.py"), "add", landing.branch, str(worktree), base(landing)]
        if subprocess.run(add, cwd=landing.repo, check=False).returncode != 0:
            raise Failed(f"could not add the landing worktree {worktree}")
    elif tip(worktree, "CHERRY_PICK_HEAD"):
        raise Failed(f"finish or abort the cherry-pick in {worktree}, then run this again", 2)
    picked = identity(worktree, f"{base(landing)}..HEAD")
    for lane in landing.lanes:
        picked |= identity(worktree, f"origin/{lane}..origin/main")
        revisions = git(worktree, "rev-list", "--reverse", "--right-only", "--cherry-pick", f"HEAD...origin/{lane}")
        for commit in revisions.split():
            if identity(worktree, f"{commit}^!") <= picked:
                continue
            if not holds(worktree, "cherry-pick", "--ff", commit):
                files = git(worktree, "diff", "--name-only", "--diff-filter=U").split()
                if not files and tip(worktree, "CHERRY_PICK_HEAD") and holds(worktree, "diff", "--cached", "--quiet"):
                    git(worktree, "cherry-pick", "--skip")
                    print(f"skipped {commit[:9]} {lane}: main already has its change", flush=True)
                    continue
                raise Failed(
                    f"CONFLICT {commit[:9]} from {lane}: {' '.join(files)}; resolve it in {worktree}, "
                    "run git cherry-pick --continue, then run this again",
                    2,
                )
            print(f"picked {commit[:9]} {lane}", flush=True)


def gate(landing: Landing, command: Sequence[str]) -> str:
    env = {**os.environ, "SM_GATE_BASE": base(landing)}
    done = subprocess.run(command, cwd=landing.worktree, env=env, capture_output=True, text=True, check=False)
    lines = done.stdout.strip().splitlines()
    if done.returncode != 0 or not lines or not lines[-1].startswith("GATE OK"):
        raise Failed("the gate failed:\n" + "\n".join(lines[-20:]))
    print(lines[-1], flush=True)
    return git(landing.worktree, "rev-parse", "HEAD")


def open_pr(github: GitHub, branch: str, title: str, body_file: Path, wait: Callable[[], None]) -> int:
    for _ in range(PR_TRIES):
        pr = github.open_pr(branch)
        if pr is not None:
            return pr
        github.create_pr(branch, title, body_file)
        wait()
    raise Failed(f"no PR for {branch} after {PR_TRIES} tries")


def ci_now(landing: Landing, github: GitHub, pr: int, head: str) -> str:
    git(landing.repo, "fetch", "-q", "origin")
    if landing.after and not holds(landing.repo, "merge-base", "--is-ancestor", landing.after, "origin/main"):
        print(f"#{pr} waits for {landing.after[:9]} to land", flush=True)
        return "pending"
    if github.pr_head(pr) != head:
        print(f"#{pr} waits for GitHub to show {head[:9]}", flush=True)
        return "pending"
    state = github.ci(landing.branch, head)
    print(f"#{pr} {head[:9]} ci={state}", flush=True)
    return state


def wait_for_ci(landing: Landing, github: GitHub, pr: int, head: str, wait: Callable[[], None]) -> None:
    for _ in range(POLLS):
        try:
            state = ci_now(landing, github, pr, head)
        except Failed as error:
            print(f"#{pr} is not ready: {error}", flush=True)
            state = "pending"
        if state == "success":
            return
        if state != "pending":
            raise Failed(f"the CI run on {head[:9]} ended {state}; read its log", 4)
        wait()
    raise Failed(f"#{pr} timed out", 5)


def held(path: Path) -> str | None:
    lines = git(path, "status", "--porcelain", "--ignored=matching", "--untracked-files=all").splitlines()
    if any(not line.startswith("!! ") for line in lines):
        return "it has uncommitted changes"
    ignored = [
        line[3:] for line in lines if not line[3:].startswith("target/") and not CACHES & set(line[3:].split("/"))
    ]
    return f"it has ignored files outside target/: {' '.join(ignored)}" if ignored else None


def remove_worktree(repo: Path, path: Path) -> None:
    shutil.rmtree(path / "target", ignore_errors=True)
    git(repo, "worktree", "remove", "--force", str(path))
    git(repo, "worktree", "prune")


def worktrees(repo: Path) -> dict[str, Path]:
    found: dict[str, Path] = {}
    path = None
    for line in git(repo, "worktree", "list", "--porcelain").splitlines():
        if line.startswith("worktree "):
            path = Path(line.removeprefix("worktree "))
        elif line.startswith("branch refs/heads/") and path:
            found[line.removeprefix("branch refs/heads/")] = path
    return found


def clean(landing: Landing, target: str, dry_run: bool) -> None:
    repo = landing.repo
    checkouts = worktrees(repo)
    verb = "would remove" if dry_run else "removed"
    for branch in (*landing.lanes, landing.branch):
        local, remote = tip(repo, f"refs/heads/{branch}"), tip(repo, f"refs/remotes/origin/{branch}")
        path = checkouts.get(branch)
        if local and on(repo, local, target):
            reason = held(path) if path else None
            if reason:
                print(f"kept {path}: {reason}", flush=True)
                continue
            if path:
                if not dry_run:
                    remove_worktree(repo, path)
                print(f"{verb} worktree {path}", flush=True)
            if not dry_run:
                git(repo, "update-ref", "-d", f"refs/heads/{branch}", local)
            print(f"{verb} {branch} at {local[:9]}", flush=True)
        elif local:
            print(f"kept {branch}: {local[:9]} is not on main", flush=True)
        if remote and on(repo, remote, target):
            if not dry_run:
                git(
                    repo, "push", "-q", f"--force-with-lease=refs/heads/{branch}:{remote}", "origin", "--delete", branch
                )
            print(f"{verb} origin/{branch} at {remote[:9]}", flush=True)
        elif remote:
            print(f"kept origin/{branch}: {remote[:9]} is not on main", flush=True)


def land(landing: Landing, github: GitHub, gate_command: Sequence[str], wait: Callable[[], None]) -> None:
    git(landing.repo, "fetch", "-q", "origin")
    pushed = tip(landing.repo, f"refs/remotes/origin/{landing.branch}")
    if pushed and holds(landing.repo, "merge-base", "--is-ancestor", pushed, "origin/main"):
        print(f"{landing.branch} at {pushed[:9]} is on main", flush=True)
        clean(landing, "origin/main", dry_run=landing.title is None)
        return
    pick(landing)
    head = gate(landing, gate_command)
    if landing.title is None or landing.body_file is None:
        print(f"dry run: would push {landing.branch} at {head[:9]}, open its PR and land it", flush=True)
        clean(landing, head, dry_run=True)
        return
    git(landing.worktree, "push", "-q", "-u", "origin", landing.branch)
    pr = open_pr(github, landing.branch, landing.title, landing.body_file, wait)
    print(f"#{pr} opened for {landing.branch}", flush=True)
    wait_for_ci(landing, github, pr, head, wait)
    if not holds(landing.repo, "merge-base", "--is-ancestor", "origin/main", head):
        raise Failed("main moved; land again under a new name on the new main", 3)
    git(landing.repo, "push", "-q", "origin", f"{head}:refs/heads/main")
    print(f"landed #{pr} at {head[:9]}", flush=True)
    try:
        github.comment(pr, f"Landed on main as a fast-forward to `{head[:9]}`; CI passed on this head.")
    except Failed as error:
        print(f"could not comment on #{pr}: {error}", flush=True)
    clean(landing, head, dry_run=False)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="land.py", description="Land reviewed lanes on main.")
    parser.add_argument("name", help="the landing branch is land/NAME")
    parser.add_argument("lanes", nargs="+", metavar="lane", help="a reviewed branch on origin, in landing order")
    parser.add_argument("--after", help="build on this landing head and land only once it is on main")
    parser.add_argument("--title")
    parser.add_argument("--body-file", type=Path)
    parser.add_argument("--dry-run", action="store_true", help="pick and gate; push, open and land nothing")
    args = parser.parse_args(argv)
    if not args.dry_run and not (args.title and args.body_file):
        parser.error("--title and --body-file are required unless --dry-run")
    repo = Path(git(Path.cwd(), "rev-parse", "--path-format=absolute", "--git-common-dir")).parent
    title = None if args.dry_run else args.title
    landing = Landing(repo, args.name, tuple(args.lanes), args.after, title, args.body_file)
    try:
        land(landing, Gh(), [sys.executable, "tools/lane/gate.py"], lambda: time.sleep(120))
    except Failed as error:
        print(error, file=sys.stderr)
        return error.code
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
