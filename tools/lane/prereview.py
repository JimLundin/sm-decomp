import re
import subprocess
import sys
from collections.abc import Callable, Iterable, Iterator
from pathlib import Path

DASHES = ("—", "–")
MAX_COLUMNS = 100
DOCS = ("docs", "AGENTS.md", "README.md", ".agents")
EXTENSIONS = {".md", ".rs", ".py", ".sh", ".toml", ".yml", ".yaml", ".txt", ".awk", ".lua", ".csv", ".lock"}
LINK = r"(?:\[[^\]]*\]\([^)\s]*\)|<[a-z]+://[^>\s]+>|[a-z]+://\S+|\[[^\]]+\]:\s*\S+)"
LINK_ONLY = re.compile(rf"\s*(?:[-*+]\s+|\d+\.\s+)?{LINK}[.,;:]?\s*")
NOT_A_PATH = re.compile(r"[$*?<>{}\[\]~]|://")
HUNK = re.compile(r"@@ -\S+ \+(\d+)")
FENCES = ("```", "~~~")
SPAN = re.compile(r"`([^`]+)`(\s*\(planned\))?")
HINT = ", or write (planned) after the span when a later task makes it"
TRAILING = re.compile(r"(?:'s|[.,;)'\"])+$")


def git(root: Path, *args: str, stdin: str | None = None) -> str:
    return subprocess.run(["git", *args], cwd=root, input=stdin, capture_output=True, text=True, check=True).stdout


def message_findings(commit: str, message: str) -> list[str]:
    return [f"commit {commit}: a dash in its message"] if any(dash in message for dash in DASHES) else []


def line_findings(where: str, line: str, prose: bool = True) -> list[str]:
    findings = []
    if any(dash in line for dash in DASHES):
        findings.append(f"{where}: a dash")
    if prose and len(line) > MAX_COLUMNS and not line.lstrip().startswith("|") and not LINK_ONLY.fullmatch(line):
        findings.append(f"{where}: {len(line)} columns, over {MAX_COLUMNS}")
    return findings


def added_lines(diff: str) -> Iterator[tuple[str, int, str]]:
    path, number = "", 0
    for line in diff.splitlines():
        if line.startswith("+++ "):
            path = line.removeprefix("+++ b/")
        elif hunk := HUNK.match(line):
            number = int(hunk.group(1))
        elif line.startswith("+"):
            yield path, number, line[1:]
            number += 1


# The numbered lines outside YAML front matter and fenced code blocks.
def prose_lines(text: str) -> Iterator[tuple[int, str]]:
    lines = text.splitlines()
    start = 0
    if lines and lines[0] == "---":
        start = next((end + 1 for end in range(1, len(lines)) if lines[end] in ("---", "...")), 0)
    fence = ""
    for number, line in enumerate(lines[start:], start + 1):
        marker = line.lstrip()[:3]
        if marker in FENCES and fence in ("", marker):
            fence = "" if fence else marker
        elif not fence:
            yield number, line


# The repository paths in code spans. A span followed by `(planned)` names a file a later task makes.
def backticked_paths(text: str, top: set[str]) -> Iterator[tuple[int, str]]:
    for number, line in prose_lines(text):
        for span in SPAN.finditer(line):
            if span.group(2):
                continue
            for token in span.group(1).split():
                path = re.split(r"[:#]", TRAILING.sub("", token.lstrip("'\"")), maxsplit=1)[0].rstrip("/")
                if NOT_A_PATH.search(token) or path.split("/")[0] not in top:
                    continue
                if "/" in path or Path(path).suffix in EXTENSIONS:
                    yield number, path


def path_findings(doc: str, text: str, top: set[str], missing: Callable[[Iterable[str]], set[str]]) -> list[str]:
    found = list(backticked_paths(text, top))
    absent = missing({path for _, path in found})
    return [f"{doc}:{number}: `{path}` does not exist{HINT}" for number, path in found if path in absent]


def missing_paths(root: Path) -> Callable[[Iterable[str]], set[str]]:
    def missing(paths: Iterable[str]) -> set[str]:
        candidates = sorted(path for path in paths if not (root / path).exists())
        if not candidates:
            return set()
        ignored = subprocess.run(["git", "check-ignore", "--no-index", "--stdin"], cwd=root, capture_output=True,
                                 text=True, input="\n".join(candidates) + "\n", check=False).stdout.split()
        return set(candidates) - set(ignored)
    return missing


def findings(root: Path, base: str) -> list[str]:
    left = git(root, "merge-base", base, "HEAD").strip()
    out = []
    log = git(root, "log", "--format=%h%x00%B%x00", f"{left}..HEAD").split("\0")
    for commit, message in zip(log[0::2], log[1::2]):
        out += message_findings(commit.strip(), message)
    # Rename detection keeps a moved file's unchanged lines out of the added ones.
    diff = git(root, "diff", "-U0", "--find-renames", "--no-ext-diff", "--no-color", left, "--", "*.md")
    changed: dict[str, set[int]] = {}
    for path, number, _ in added_lines(diff):
        changed.setdefault(path, set()).add(number)
    for path in git(root, "ls-files", "-o", "--exclude-standard", "--", "*.md").splitlines():
        changed[path] = set(range(1, len((root / path).read_text().splitlines()) + 1))
    for path, numbers in changed.items():
        text = (root / path).read_text()
        prose = {number for number, _ in prose_lines(text)}
        for number, line in enumerate(text.splitlines(), 1):
            if number in numbers:
                out += line_findings(f"{path}:{number}", line, number in prose)
    top = {path.split("/")[0] for path in git(root, "ls-files").splitlines()}
    top |= set(git(root, "ls-tree", "--name-only", left).splitlines())
    missing = missing_paths(root)
    for doc in git(root, "ls-files", "-co", "--exclude-standard", "--", *DOCS).splitlines():
        if doc.endswith(".md") and (root / doc).is_file():
            out += path_findings(doc, (root / doc).read_text(), top, missing)
    return out


def main(argv: list[str]) -> int:
    base = argv[1] if len(argv) > 1 else "origin/main"
    try:
        root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel").strip())
        found = findings(root, base)
    except subprocess.CalledProcessError as error:
        print(f"prereview: {' '.join(error.cmd)} failed: {error.stderr.strip()}", file=sys.stderr)
        return 2
    for finding in found:
        print(f"prereview: {finding}")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
