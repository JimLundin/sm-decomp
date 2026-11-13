"""Require each Parameter entry's adjusted vector to depend on the value it adjusts.

adjusted_vectors.sh reads the vector file alone, so it can say only that a vector names an
entry and carries rules. This runs the domain's own vector test (MODEL §1 decision 16): an
adjusted vector proves its entry can be adjusted when the same case, run at the reference
rules, no longer gives the vector's answer. For each candidate vector it appends two probes to
the vector file, one at a time, and runs the test on each:

- the control, a copy of the vector under another id: the test must still pass, so a failure
  of the other probe is the model's answer and not the copy's;
- the reference probe, the same copy without its rules: the test must fail.

A rules table that is empty, repeats the reference values or changes something the case
doesn't reach leaves the reference probe passing, and the entry stays unproven. So a model
that ignores an adjusted value fails that entry's vector in the domain's test. The vector file
is restored whatever happens.

It can't tell which of a vector's entries its rules belong to: a vector proven to depend on
its rules counts for every Parameter entry it names. So an entry is tried first on the vectors
that name no other Parameter entry, and a proof by a vector that names others says which, for
a reader to judge.

Usage: adjusted_vectors_run.py [DOMAIN...], from the repository root; with no DOMAIN, every
spec with a vector file and a clean crate. Exit 1 when an entry has adjusted vectors and none
depends on its rules; 2 when a domain can't be probed (its test fails as committed, or fails
the control); else 0. An entry no vector adjusts is listed and left to adjusted_vectors.sh,
which gates it at the domain's spec round.
"""

import os
import re
import subprocess
import sys
import tomllib
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

SWITCH = re.compile(r"^- switch: *\**parameter", re.IGNORECASE)
PROBE_SUFFIX = "-probe"


class CannotProbe(Exception):
    """The domain's test can't answer: nothing is proven or refuted."""


@dataclass(frozen=True)
class Candidate:
    """A vector that carries a rules table with something in it."""

    index: int
    id: str
    entries: frozenset[str]


def parameter_entries(spec: str) -> list[str]:
    """The spec's entries whose switch is Parameter, in its order."""
    found: list[str] = []
    entry = ""
    for line in spec.splitlines():
        if line.startswith("### "):
            entry = line.split()[1]
        elif entry and SWITCH.match(line):
            found.append(entry)
            entry = ""
    return found


def vectors(text: str) -> list[dict[str, Any]]:
    rows = tomllib.loads(text).get("vector", [])
    if not isinstance(rows, list):
        raise CannotProbe("`vector` is not an array of tables")
    return rows


def candidates(text: str) -> list[Candidate]:
    """The vectors whose rules table isn't empty."""
    return [
        Candidate(index, str(row.get("id", "")), frozenset(row.get("entries", [])))
        for index, row in enumerate(vectors(text))
        if row.get("rules")
    ]


def block(text: str, index: int) -> list[str]:
    """The lines of the index-th `[[vector]]`, up to the next array's table."""
    lines = text.splitlines()
    starts = [n for n, line in enumerate(lines) if line.startswith("[[vector]]")]
    start = starts[index]
    end = next((n for n in range(start + 1, len(lines)) if lines[n].startswith("[[")),
               len(lines))
    return lines[start:end]


def without_rules(lines: list[str]) -> list[str]:
    """The block less its rules, inline or as a `[vector.rules]` table."""
    kept: list[str] = []
    table = ""
    for line in lines:
        if line.startswith("["):
            table = line.strip()
        if table == "[vector.rules]" or (table == "[[vector]]"
                                         and re.match(r"rules\s*=", line)):
            continue
        kept.append(line)
    return kept


def probe(text: str, candidate: Candidate, keep_rules: bool) -> str:
    """The file with a copy of the candidate appended under a probe id.

    The copy is checked as TOML against the vector it copies, so the text edit can't pass for
    a different case.
    """
    lines = block(text, candidate.index)
    if not keep_rules:
        lines = without_rules(lines)
    probe_id = candidate.id + PROBE_SUFFIX
    renamed = [re.sub(r'^(id\s*=\s*)"[^"]*"', lambda m: f'{m.group(1)}"{probe_id}"', line)
               for line in lines]
    probed = text.rstrip("\n") + "\n\n" + "\n".join(renamed) + "\n"
    expected = dict(vectors(text)[candidate.index], id=probe_id)
    if not keep_rules:
        del expected["rules"]
    rows = vectors(probed)
    if len(rows) != len(vectors(text)) + 1 or rows[-1] != expected:
        raise CannotProbe(f"{candidate.id}: its copy doesn't read as the vector it copies")
    return probed


@dataclass(frozen=True)
class Proven:
    """A vector that fails at the reference rules, and the other Parameter entries it names."""

    vector: str
    shared_with: tuple[str, ...]


@dataclass(frozen=True)
class Unproven:
    """Vectors adjust the entry, and each still passes at the reference rules."""


@dataclass(frozen=True)
class Unadjusted:
    """No vector that names the entry carries a rule."""


Verdict = Proven | Unproven | Unadjusted


def prove(path: Path, entries: list[str], run: Callable[[], bool]) -> dict[str, Verdict]:
    """Each Parameter entry's verdict, from probes of the vectors that adjust it."""
    original = path.read_bytes()
    text = original.decode("utf-8")
    if not run():
        raise CannotProbe("the vector test fails on the committed vectors")
    depending: dict[str, bool] = {}

    def depends(candidate: Candidate) -> bool:
        if candidate.id not in depending:
            path.write_bytes(probe(text, candidate, keep_rules=False).encode("utf-8"))
            fails_at_reference = not run()
            if fails_at_reference:
                path.write_bytes(probe(text, candidate, keep_rules=True).encode("utf-8"))
                if not run():
                    raise CannotProbe(f"{candidate.id}: the test refuses a copy of it")
            depending[candidate.id] = fails_at_reference
        return depending[candidate.id]

    def others(candidate: Candidate, entry: str) -> tuple[str, ...]:
        return tuple(e for e in entries if e != entry and e in candidate.entries)

    def verdict(entry: str) -> Verdict:
        adjusting = sorted((c for c in candidates(text) if entry in c.entries),
                           key=lambda c: len(others(c, entry)))
        if not adjusting:
            return Unadjusted()
        return next((Proven(c.id, others(c, entry)) for c in adjusting if depends(c)),
                    Unproven())

    try:
        return {entry: verdict(entry) for entry in entries}
    finally:
        path.write_bytes(original)


def test_command(crate: str) -> list[str]:
    if os.name == "nt":
        return ["bash", "tools/lane/run_tests.sh", crate]
    return ["cargo", "test", "--locked", "-p", crate, "--test", "vectors"]


def check_domain(root: Path, domain: str) -> int:
    """Print the domain's verdicts; the number of entries whose adjusted vectors prove nothing."""
    spec = (root / "docs/behaviours" / f"{domain}.md").read_text(encoding="utf-8")
    crate = domain.replace("-", "_")

    def run() -> bool:
        return subprocess.run(test_command(crate), cwd=root, capture_output=True,
                              check=False).returncode == 0

    entries = parameter_entries(spec)
    verdicts = prove(root / "scenarios/vectors" / f"{domain}.toml", entries, run)
    for entry, verdict in verdicts.items():
        match verdict:
            case Unadjusted():
                print(f"adjusted: {domain}: {entry} has no adjusted vector")
            case Unproven():
                print(f"adjusted: {domain}: {entry}'s adjusted vectors all pass at the "
                      "reference rules")
            case Proven(vector, ()):
                print(f"adjusted: {domain}: {entry} depends on its rules in {vector}")
            case Proven(vector, shared_with):
                print(f"adjusted: {domain}: {entry} depends on its rules in {vector}, which "
                      f"also names {', '.join(shared_with)}")
    proven = sum(1 for verdict in verdicts.values() if isinstance(verdict, Proven))
    print(f"adjusted: {domain}: {proven} of {len(entries)} Parameter entries have a vector "
          "that fails at the reference rules")
    return sum(1 for verdict in verdicts.values() if isinstance(verdict, Unproven))


def domains(root: Path) -> list[str]:
    return sorted(
        spec.stem for spec in (root / "docs/behaviours").glob("*.md")
        if "." not in spec.stem
        and (root / "scenarios/vectors" / f"{spec.stem}.toml").is_file()
        and (root / "crates/clean" / spec.stem.replace("-", "_")).is_dir()
    )


def main(arguments: list[str]) -> int:
    root = Path.cwd()
    unproven = 0
    try:
        for domain in arguments or domains(root):
            unproven += check_domain(root, domain)
    except CannotProbe as error:
        print(f"adjusted_vectors_run: {error}", file=sys.stderr)
        return 2
    return 1 if unproven else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
