"""Require each Parameter entry's adjusted vector to depend on the value it adjusts.

adjusted_vectors.sh reads the vector file alone, so it can say only that a vector names an
entry and carries rules. This runs the domain's own vector test (MODEL §1 decision 16): an
adjusted vector proves its entry can be adjusted when the same case, run at the reference
rules, no longer gives the vector's answer. For each candidate vector it replaces that case
with two probes, one at a time, and runs the test on each:

- the control, a copy of the vector under another id: the test must still pass, so a failure
  of the other probe is the model's answer and not the copy's;
- the reference probe, the same copy without its owned overrides: the test must fail.

A rules table that is empty, repeats the reference values or changes something the case
doesn't reach leaves the reference probe passing, and the entry stays unproven. So a model
that ignores an adjusted value fails that entry's vector in the domain's test. The vector file
is restored whatever happens.

tools/cleanroom/adjusted_rule_owners/DOMAIN.toml maps each vector-format rule subtree to
its reviewed Parameter entry owners: [owners], rule = ["Entry"]. Ownership means the
entry's declared Parameter, not a parameter consumed by a downstream behavior. A probe
removes only that entry's overrides, keeping every unrelated override fixed. Missing or
unknown ownership refuses a probe. If multiple owners of an overridden rule co-occur in
a candidate's entries, that witness is ambiguous and is refused. Separate witnesses can
prove entries that share a parameter. This proves entry adjustability, not each knob.


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


def block_bounds(text: str, index: int) -> tuple[int, int]:
    """The line range of the index-th `[[vector]]`, up to the next array's table."""
    lines = text.splitlines()
    starts = [n for n, line in enumerate(lines) if line.startswith("[[vector]]")]
    start = starts[index]
    end = next((n for n in range(start + 1, len(lines)) if lines[n].startswith("[[")),
               len(lines))
    return start, end


def without_rules(lines: list[str]) -> list[str]:
    """The block less its rules, inline or as a `[vector.rules]` table."""
    kept: list[str] = []
    table = ""
    for line in lines:
        if line.startswith("["):
            table = line.strip()
        in_rules_table = (table == "[vector.rules]" or table.startswith("[vector.rules.")
                          or table.startswith("[[vector.rules."))
        inline_rules = table == "[[vector]]" and re.match(r"rules\s*=", line)
        if in_rules_table or inline_rules:
            continue
        kept.append(line)
    return kept


def inline_toml(value: Any) -> str:
    """Encode parsed rule values as TOML; unsupported values refuse rather than drift."""
    import json

    if isinstance(value, dict):
        return "{ " + ", ".join(json.dumps(k) + " = " + inline_toml(v)
                                for k, v in value.items()) + " }"
    if isinstance(value, list):
        return "[" + ", ".join(inline_toml(v) for v in value) + "]"
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return repr(value)
    raise CannotProbe("unsupported rule value")


def probe(text: str, candidate: Candidate, keep_rules: bool,
          remove_rules: frozenset[str] | None = None) -> str:
    """The file with the candidate replaced under a unique probe id.

    The entire parsed file is checked against the expected replacement, so other vectors,
    globals and arrays cannot change as a side effect of the text edit.
    """
    original = tomllib.loads(text)
    original_rows = vectors(text)
    probe_id = candidate.id + PROBE_SUFFIX
    ids = [row.get("id") for row in original_rows]
    if len(set(ids)) != len(ids) or probe_id in ids:
        raise CannotProbe(f"{candidate.id}: vector ids must stay unique")
    if original_rows[candidate.index].get("id") != candidate.id:
        raise CannotProbe(f"{candidate.id}: candidate doesn't identify its vector")
    start, end = block_bounds(text, candidate.index)
    all_lines = text.splitlines()
    lines = all_lines[start:end]
    replacement_rules = dict(original_rows[candidate.index].get("rules", {}))
    if not keep_rules:
        for rule in remove_rules if remove_rules is not None else tuple(replacement_rules):
            if rule not in replacement_rules:
                raise CannotProbe(f"{candidate.id}: unknown override {rule}")
            del replacement_rules[rule]
        lines = without_rules(lines)
        # Insert an inline table immediately after the array heading. Values are copied
        # from the parsed original; a complete semantic comparison below checks the edit.
        if replacement_rules:
            lines.insert(1, "rules = " + inline_toml(replacement_rules))
    renamed: list[str] = []
    table = ""
    for line in lines:
        if line.startswith("["):
            table = line.strip()
        if table == "[[vector]]":
            line = re.sub(r'^(id\s*=\s*)"[^"]*"',
                          lambda m: f'{m.group(1)}"{probe_id}"', line)
        renamed.append(line)
    probed = "\n".join(all_lines[:start] + renamed + all_lines[end:]) + "\n"
    expected = dict(original_rows[candidate.index], id=probe_id)
    if not keep_rules:
        if replacement_rules:
            expected["rules"] = replacement_rules
        else:
            del expected["rules"]
    expected_rows = original_rows.copy()
    expected_rows[candidate.index] = expected
    expected_file = dict(original, vector=expected_rows)
    if tomllib.loads(probed) != expected_file:
        raise CannotProbe(f"{candidate.id}: replacement changes the case or unrelated content")
    return probed


@dataclass(frozen=True)
class Proven:
    """A vector that fails when this entry alone uses its reference rules."""

    vector: str
    shared_with: tuple[str, ...]


@dataclass(frozen=True)
class Unproven:
    """No named vector depends on overrides owned by this entry."""


@dataclass(frozen=True)
class Unadjusted:
    """No vector that names the entry carries a rule."""


Verdict = Proven | Unproven | Unadjusted


def prove(path: Path, entries: list[str], run: Callable[[], bool],
          owners: dict[str, frozenset[str]]) -> dict[str, Verdict]:
    """Each Parameter entry's verdict, from probes of the vectors that adjust it."""
    original = path.read_bytes()
    text = original.decode("utf-8")
    if not run():
        raise CannotProbe("the vector test fails on the committed vectors")
    adjusting_candidates = candidates(text)
    for candidate in adjusting_candidates:
        for rule in vectors(text)[candidate.index]["rules"]:
            if rule not in owners or not owners[rule] or not owners[rule].issubset(entries):
                raise CannotProbe(f"{candidate.id}: missing or unknown owner for {rule}")
            if len(owners[rule].intersection(candidate.entries)) > 1:
                raise CannotProbe(f"{candidate.id}: ambiguous owners for {rule}")
    depending: dict[tuple[str, str], bool] = {}

    def depends(candidate: Candidate, entry: str) -> bool:
        key = (candidate.id, entry)
        removed = frozenset(rule for rule in vectors(text)[candidate.index]["rules"]
                            if entry in owners[rule])
        if not removed:
            return False
        if key not in depending:
            path.write_bytes(probe(text, candidate, keep_rules=False,
                                   remove_rules=removed).encode("utf-8"))
            fails_at_reference = not run()
            if fails_at_reference:
                path.write_bytes(probe(text, candidate, keep_rules=True).encode("utf-8"))
                if not run():
                    raise CannotProbe(f"{candidate.id}: the test refuses a copy of it")
            depending[key] = fails_at_reference
        return depending[key]

    def others(candidate: Candidate, entry: str) -> tuple[str, ...]:
        return tuple(e for e in entries if e != entry and e in candidate.entries)

    def verdict(entry: str) -> Verdict:
        adjusting = sorted((c for c in adjusting_candidates if entry in c.entries),
                           key=lambda c: len(others(c, entry)))
        if not adjusting:
            return Unadjusted()
        return next((Proven(c.id, others(c, entry)) for c in adjusting if depends(c, entry)),
                    Unproven())

    try:
        return {entry: verdict(entry) for entry in entries}
    finally:
        path.write_bytes(original)


SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in .+$", re.MULTILINE)
RUNNING = re.compile(r"^running (\d+) tests?$", re.MULTILINE)


def completed_test(result: subprocess.CompletedProcess[bytes], failure_code: int) -> bool:
    """Only a completed nonempty libtest suite can pass or refute a vector.

    Build errors, tool failures, crashes and empty/ignored suites are not model answers.
    Exactly one selected test target must complete and agree with its process status.
    """
    output = result.stdout.decode("utf-8", errors="replace").replace("\r\n", "\n")
    summaries = SUMMARY.findall(output)
    running = RUNNING.findall(output)
    if len(summaries) != 1 or len(running) != 1:
        raise CannotProbe("the runner did not complete exactly one test suite")
    status, *counts = summaries[0]
    passed, failed, ignored, measured, _filtered = map(int, counts)
    if passed + failed == 0 or int(running[0]) != passed + failed + ignored + measured:
        raise CannotProbe("the runner did not execute a nonempty consistent test suite")
    if status == "ok" and failed == 0 and result.returncode == 0:
        return True
    if status == "FAILED" and failed > 0 and result.returncode == failure_code:
        return False
    raise CannotProbe("test completion and process status disagree")


def test_command(crate: str, root: Path) -> list[str]:
    dedicated = (root / "crates/clean" / crate / "tests/vectors.rs").is_file()
    if os.name == "nt":
        target = "test:vectors" if dedicated else "lib"
        return ["bash", "tools/lane/run_tests.sh", "--suite", target, crate]
    command = ["cargo", "test", "--locked", "-p", crate]
    return command + (["--test", "vectors"] if dedicated else ["--lib"])


def rule_owners(root: Path, domain: str, required: bool) -> dict[str, frozenset[str]]:
    """Reviewed ownership uses vector-format top-level rule subtrees, not model names.

    An entry with no adjusted candidates stays advisory. An existing candidate may never
    fall back to removing unrelated rules because its ownership metadata is missing.
    """
    ownership = root / "tools/cleanroom/adjusted_rule_owners" / f"{domain}.toml"
    if not ownership.is_file():
        if not required:
            return {}
        raise CannotProbe(f"{domain}: missing reviewed rule ownership")
    raw_owners = tomllib.loads(ownership.read_text(encoding="utf-8")).get("owners")
    if not isinstance(raw_owners, dict) or not all(
            isinstance(k, str) and isinstance(v, list) and all(isinstance(e, str) for e in v)
            for k, v in raw_owners.items()):
        raise CannotProbe(f"{domain}: owners must map rules to arrays of Parameter entries")
    return {rule: frozenset(names) for rule, names in raw_owners.items()}


def check_domain(root: Path, domain: str) -> int:
    """Print the domain's verdicts; the number of entries whose adjusted vectors prove nothing."""
    spec = (root / "docs/behaviours" / f"{domain}.md").read_text(encoding="utf-8")
    crate = domain.replace("-", "_")

    def run() -> bool:
        try:
            result = subprocess.run(test_command(crate, root), cwd=root, capture_output=True,
                                    check=False)
        except OSError as error:
            raise CannotProbe("the test runner could not be launched") from error
        return completed_test(result, failure_code=1 if os.name == "nt" else 101)

    entries = parameter_entries(spec)
    vector_path = root / "scenarios/vectors" / f"{domain}.toml"
    owners = rule_owners(root, domain, bool(candidates(vector_path.read_text(encoding="utf-8"))))
    verdicts = prove(vector_path, entries, run, owners)
    for entry, verdict in verdicts.items():
        match verdict:
            case Unadjusted():
                print(f"adjusted: {domain}: {entry} has no adjusted vector")
            case Unproven():
                print(f"adjusted: {domain}: no named vector depends on {entry}'s "
                      "owned rule overrides")
            case Proven(vector, ()):
                print(f"adjusted: {domain}: {entry} depends on its rules in {vector}")
            case Proven(vector, shared_with):
                print(f"adjusted: {domain}: {entry} depends on its rules in {vector}, which "
                      f"also names {', '.join(shared_with)}")
    proven = sum(1 for verdict in verdicts.values() if isinstance(verdict, Proven))
    print(f"adjusted: {domain}: {proven} of {len(entries)} Parameter entries have a vector "
          "that fails when its owned overrides are removed")
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
