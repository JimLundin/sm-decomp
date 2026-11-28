import re
from dataclasses import dataclass
from pathlib import PurePosixPath
lazy from collections.abc import Mapping, Sequence

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

DASHES = FrozenList(("\u2014", "\u2013"))
MAX_COLUMNS = 100
DOCS = FrozenList(("docs", "AGENTS.md", "README.md", ".agents"))
EXTENSIONS = FrozenOrderedSet((
    ".md",
    ".rs",
    ".py",
    ".sh",
    ".toml",
    ".yml",
    ".yaml",
    ".txt",
    ".awk",
    ".lua",
    ".csv",
    ".lock",
))
LINK = r"(?:\[[^\]]*\]\([^)\s]*\)|<[a-z]+://[^>\s]+>|[a-z]+://\S+|\[[^\]]+\]:\s*\S+)"
LINK_ONLY = re.compile(rf"\s*(?:[-*+]\s+|\d+\.\s+)?{LINK}[.,;:]?\s*")
NOT_A_PATH = re.compile(r"[$*?<>{}\[\]~]|://")
HUNK = re.compile(r"@@ -\S+ \+(\d+)")
FENCES = FrozenList(("```", "~~~"))
FRONT_MATTER = "---"
FRONT_MATTER_ENDS = FrozenList(("---", "..."))
SPAN = re.compile(r"`([^`]+)`(\s*\(planned\))?")
HINT = ", or write (planned) after the span when a later task makes it"
TRAILING = re.compile(r"(?:'s|[.,;)'\"])+$")
ANCHOR = re.compile(r"[:#]")
FENCE_WIDTH = 3


@dataclass(frozen=True, slots=True)
class Added:
    path: str
    number: int
    line: str


@dataclass(frozen=True, slots=True)
class Mention:
    number: int
    path: str


def message_findings(commit: str, message: str) -> FrozenList[str]:
    return FrozenList(
        (f"commit {commit}: a dash in its message",)
        if any(dash in message for dash in DASHES)
        else ()
    )


def line_findings(
    where: str, line: str, *, prose: bool = True
) -> FrozenList[str]:
    dash = any(dash in line for dash in DASHES)
    long = (
        prose
        and len(line) > MAX_COLUMNS
        and not line.lstrip().startswith("|")
        and not LINK_ONLY.fullmatch(line)
    )
    return FrozenList((
        *((f"{where}: a dash",) if dash else ()),
        *(
            (f"{where}: {len(line)} columns, over {MAX_COLUMNS}",)
            if long
            else ()
        ),
    ))


def added_lines(diff: str) -> FrozenList[Added]:
    added: list[Added] = []
    path, number = "", 0
    for line in diff.splitlines():
        if line.startswith("+++ "):
            path = line.removeprefix("+++ b/")
        elif hunk := HUNK.match(line):
            number = int(hunk.group(1))
        elif line.startswith("+"):
            added.append(Added(path, number, line[1:]))
            number += 1
    return FrozenList(added)


def body_start(lines: Sequence[str]) -> int:
    if not lines or lines[0] != FRONT_MATTER:
        return 0
    return next(
        (
            end + 1
            for end in range(1, len(lines))
            if lines[end] in FRONT_MATTER_ENDS
        ),
        0,
    )


def prose_lines(text: str) -> FrozenOrderedDict[int, str]:
    lines = text.splitlines()
    start = body_start(lines)
    prose: dict[int, str] = {}
    fence = ""
    for number, line in enumerate(lines[start:], start + 1):
        marker = line.lstrip()[:FENCE_WIDTH]
        if marker in FENCES and (not fence or fence == marker):
            fence = "" if fence else marker
        elif not fence:
            prose[number] = line
    return FrozenOrderedDict(prose)


def span_path(token: str, top: Sequence[str]) -> str | None:
    stripped = TRAILING.sub("", token.lstrip("'\""))
    path = ANCHOR.split(stripped, maxsplit=1)[0].rstrip("/")
    if NOT_A_PATH.search(token) or path.split("/")[0] not in top:
        return None
    if "/" in path or PurePosixPath(path).suffix in EXTENSIONS:
        return path
    return None


def backticked_paths(text: str, top: Sequence[str]) -> FrozenList[Mention]:
    return FrozenList(
        Mention(number, path)
        for number, line in prose_lines(text).items()
        for span in SPAN.finditer(line)
        if not span.group(2)
        for token in span.group(1).split()
        if (path := span_path(token, top)) is not None
    )


def path_findings(
    doc: str, found: Sequence[Mention], absent: Sequence[str]
) -> FrozenList[str]:
    return FrozenList(
        f"{doc}:{mention.number}: `{mention.path}` does not exist{HINT}"
        for mention in found
        if mention.path in absent
    )


def changed_numbers(diff: str) -> FrozenOrderedDict[str, FrozenOrderedSet[int]]:
    changed: dict[str, list[int]] = {}
    for added in added_lines(diff):
        changed.setdefault(added.path, []).append(added.number)
    return FrozenOrderedDict(
        (path, FrozenOrderedSet(numbers)) for path, numbers in changed.items()
    )


def doc_findings(
    path: str, text: str, numbers: FrozenOrderedSet[int]
) -> FrozenList[str]:
    prose = prose_lines(text)
    return FrozenList(
        finding
        for number, line in enumerate(text.splitlines(), 1)
        if number in numbers
        for finding in line_findings(
            f"{path}:{number}", line, prose=number in prose
        )
    )


def commit_findings(log: str) -> FrozenList[str]:
    fields = log.split("\0")
    return FrozenList(
        finding
        for commit, message in zip(fields[0::2], fields[1::2], strict=False)
        for finding in message_findings(commit.strip(), message)
    )


def all_lines(text: str) -> FrozenOrderedSet[int]:
    return FrozenOrderedSet(range(1, len(text.splitlines()) + 1))


def top_names(tracked: str, at_base: str) -> FrozenOrderedSet[str]:
    return FrozenOrderedSet((
        *(path.split("/")[0] for path in tracked.splitlines()),
        *at_base.splitlines(),
    ))


def is_doc(path: str) -> bool:
    return path.endswith(".md")


def reported(found: Sequence[str]) -> FrozenList[str]:
    return FrozenList(f"prereview: {finding}" for finding in found)


def changed_docs(
    diff: str, untracked: Mapping[str, str]
) -> FrozenOrderedDict[str, FrozenOrderedSet[int]]:
    return FrozenOrderedDict({
        **changed_numbers(diff),
        **{path: all_lines(text) for path, text in untracked.items()},
    })
