import sys
from pathlib import Path
lazy from collections.abc import Sequence

from ordered import FrozenList, FrozenOrderedSet

from sm_tools.lane.core.prereview import (
    DOCS,
    backticked_paths,
    changed_docs,
    commit_findings,
    doc_findings,
    is_doc,
    path_findings,
    reported,
    top_names,
)
from sm_tools.shell.process import git


class GitError(Exception):
    def __init__(self, arguments: Sequence[str], stderr: str) -> None:
        super().__init__(f"git {' '.join(arguments)} failed: {stderr.strip()}")


def read(*arguments: str, cwd: Path) -> str:
    done = git(*arguments, cwd=cwd)
    if done.returncode != 0:
        raise GitError(arguments, done.stderr)
    return done.stdout


def missing(root: Path, paths: Sequence[str]) -> FrozenList[str]:
    candidates = sorted(
        path for path in FrozenOrderedSet(paths) if not (root / path).exists()
    )
    if not candidates:
        return FrozenList()
    listed = "\n".join(candidates) + "\n"
    ignored = git(
        "check-ignore", "--no-index", "--stdin", cwd=root, stdin=listed
    ).stdout.split()
    return FrozenList(path for path in candidates if path not in ignored)


def mentions(root: Path, doc: str, top: Sequence[str]) -> FrozenList[str]:
    found = backticked_paths((root / doc).read_text(), top)
    absent = missing(root, FrozenList(mention.path for mention in found))
    return path_findings(doc, found, absent)


def findings(root: Path, base: str) -> FrozenList[str]:
    left = read("merge-base", base, "HEAD", cwd=root).strip()
    log = read("log", "--format=%h%x00%B%x00", f"{left}..HEAD", cwd=root)
    diff = read(
        "diff",
        "-U0",
        "--find-renames",
        "--no-ext-diff",
        "--no-color",
        left,
        "--",
        "*.md",
        cwd=root,
    )
    untracked = read(
        "ls-files", "-o", "--exclude-standard", "--", "*.md", cwd=root
    ).splitlines()
    changed = changed_docs(
        diff, {path: (root / path).read_text() for path in untracked}
    )
    top = FrozenList(
        top_names(
            read("ls-files", cwd=root),
            read("ls-tree", "--name-only", left, cwd=root),
        )
    )
    docs = read(
        "ls-files", "-co", "--exclude-standard", "--", *DOCS, cwd=root
    ).splitlines()
    return FrozenList((
        *commit_findings(log),
        *(
            finding
            for path, numbers in changed.items()
            for finding in doc_findings(
                path, (root / path).read_text(), numbers
            )
        ),
        *(
            finding
            for doc in docs
            if is_doc(doc) and (root / doc).is_file()
            for finding in mentions(root, doc, top)
        ),
    ))


def run(arguments: Sequence[str]) -> int:
    base = arguments[0] if arguments else "origin/main"
    try:
        root = Path(
            read("rev-parse", "--show-toplevel", cwd=Path.cwd()).strip()
        )
        found = findings(root, base)
    except GitError as error:
        print(f"prereview: {error}", file=sys.stderr)
        return 2
    for line in reported(found):
        print(line)
    return 1 if found else 0


def main() -> int:
    return run(sys.argv[1:])
