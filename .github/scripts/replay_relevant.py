#!/usr/bin/env python3
"""Tell a pull request's Verify run to skip replays when no changed file can affect them.

A pull request skips when every file it changes, against its base or against the
last head that passed Verify on the same branch, matches IRRELEVANT. Anything else,
including a file this script cannot classify or a comparison it cannot fetch,
runs the full gate. Scheduled and dispatched runs never skip here.
"""
import fnmatch
import json
import os
import subprocess

# Files that cannot change what a replay executes or compares.
IRRELEVANT = [
    "*.md",
    "docs/*",
    ".agents/*",
    "archive/*",
    "tools/lane/*",
    "tools/ram_ratchet.py",
    "tools/cycle_report.py",
    "tools/progress.sh",
    "tools/progress_accounting.py",
    ".github/scripts/replay_relevant.py",
    # Measured row seconds only decide how rows pack into jobs, not what any row replays.
    ".github/scripts/row_seconds.*",
    # Tests the Quality jobs already run: target-package integration tests (cargo
    # test) and the mutation lists, whose anchors Quality checks.
    # sm/src/verification/*.rs is the verifier itself and stays relevant.
    # ci.yml itself is relevant: it defines the Verify jobs.
    "crates/*/tests/*",
    "sm/src/verification/*mutations.py",
    ".github/scripts/test_summary.py",
]


def irrelevant(path: str) -> bool:
    return any(fnmatch.fnmatch(path, pattern) for pattern in IRRELEVANT)


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def changed(base: str, head: str) -> list[str] | None:
    """The files changed between two commits, or None when either cannot be fetched."""
    try:
        git("fetch", "--quiet", "--depth=1", "origin", base, head)
        return [line for line in git("diff", "--name-only", base, head).splitlines() if line]
    except subprocess.CalledProcessError:
        return None


def passed_heads(branch: str) -> list[str]:
    """Head commits of recent successful pull request Verify runs on this branch."""
    path = (f"repos/{os.environ['GITHUB_REPOSITORY']}/actions/workflows/{os.environ['WORKFLOW']}/runs"
            f"?event=pull_request&status=success&branch={branch}&per_page=5")
    try:
        runs: list[dict[str, str]] = json.loads(subprocess.check_output(["gh", "api", path]))["workflow_runs"]
    except (subprocess.CalledProcessError, KeyError, ValueError):
        return []
    return [run["head_sha"] for run in runs]


def main() -> None:
    skip, reason = False, "not a pull request"
    if os.environ["GITHUB_EVENT_NAME"] == "pull_request":
        with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as source:
            event = json.load(source)["pull_request"]
        head = event["head"]["sha"]
        reason = "replay-relevant files changed"
        for base in [event["base"]["sha"], *passed_heads(event["head"]["ref"])]:
            files = changed(base, head)
            if files is not None and all(irrelevant(path) for path in files):
                skip = True
                reason = f"only non-replay files changed since {base[:12]}: {files}"
                break
    with open(os.environ["GITHUB_OUTPUT"], "a") as out:
        out.write(f"skip={'true' if skip else 'false'}\n")
    print(f"skip={skip}: {reason}")


if __name__ == "__main__":
    main()
