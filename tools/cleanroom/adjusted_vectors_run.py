"""Require each Parameter entry's adjusted vector to depend on the value it adjusts.

--missing reads the vector file alone, so it can say only that a vector names an entry and
carries rules. The rest runs the domain's own vector test (MODEL §1 decision 16): an
adjusted vector proves its entry can be adjusted when the same case, run at the reference
rules, no longer gives the vector's answer. For each candidate vector it replaces that case
with two probes, one at a time, and runs the test on each:

- the control, a copy of the vector under another id: the test must still pass, so a failure
  of the other probe is the model's answer and not the copy's;
- the reference probe, the same copy without its owned overrides: the test must fail.

A rules table that is empty, repeats the reference values or changes something the case
doesn't reach leaves the reference probe passing, and the entry stays unproven. So a model
that ignores an adjusted value fails that entry's vector in the domain's test.

The vector test reads its vector file when it runs, through crates/vector_file, which reads the
file SM_VECTOR_FILE names in its place. The runner builds every domain's test once and runs it
whole on the committed file. Each probe is then written to a file of its own, and the same
executable runs on it. The build takes one cargo run per kind of test target, over all domains
together, and the domains' tests run one process per CPU.

tools/cleanroom/adjusted_rule_owners/DOMAIN.toml maps each vector-format rule subtree to
its reviewed Parameter entry owners: [owners], rule = ["Entry"]. Ownership means the
entry's declared Parameter, not a parameter consumed by a downstream behavior. A probe
removes only that entry's overrides, keeping every unrelated override fixed. Missing or
unknown ownership refuses a probe. If multiple owners of an overridden rule co-occur in
a candidate's entries, that witness is ambiguous and is refused. Separate witnesses can
prove entries that share a parameter. This proves entry adjustability, not each knob.

When linked cases inherit ordinary vectors, adjusted_probe_tests.toml names the reviewed
exact direct-vector test. The committed complete suite and selected test must both pass
before any mutation. Both probes then run only that exact test: dependent assertions
cannot credit an override ignored by its ordinary vector. A selected run must complete
exactly one nonignored test with the declared name. Linked files lacking selection refuse
probing. No dependent reference, expectation or override is edited.

Usage: adjusted_vectors_run.py [--owners | --missing] [DOMAIN...], from the repository root;
with no DOMAIN, every spec with a vector file and a clean crate. --owners checks only the
ownership and test selection a probe needs, and runs no test: the gate's form. --missing lists
each Parameter entry no vector adjusts, and needs a DOMAIN; the gate runs it on each domain whose
spec or vectors a branch changes. Exit 1 when an entry has adjusted vectors and none depends on
its rules, or under --missing when an entry has none; 2 when a domain can't be probed (its test
doesn't build, fails as committed or fails the control); else 0. An entry no vector adjusts is
listed and left to --missing, which gates it at the domain's spec round.
"""

import functools
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from collections.abc import Callable
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Any

SWITCH = re.compile(r"^- switch: *\**parameter", re.IGNORECASE)
PROBE_SUFFIX = "-probe"
VECTOR_FILE = "SM_VECTOR_FILE"


class CannotProbe(Exception):
    """The domain's test can't answer: nothing is proven or refuted."""


@dataclass(frozen=True)
class Candidate:
    """A vector that carries a rules table with something in it."""

    index: int
    id: str
    entries: frozenset[str]
    rules: frozenset[str]


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


@functools.lru_cache(maxsize=2)
def parsed(text: str, what: str) -> dict[str, Any]:
    """TOML text as a table, which callers don't change; a syntax error refuses, naming WHAT."""
    try:
        return tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise CannotProbe(f"{what} is not TOML: {error}") from None


def vectors(text: str) -> list[dict[str, Any]]:
    rows = parsed(text, "the vector file").get("vector", [])
    if not isinstance(rows, list):
        raise CannotProbe("`vector` is not an array of tables")
    return rows


def candidates(text: str) -> list[Candidate]:
    """The vectors whose rules table isn't empty."""
    return [
        Candidate(index, str(row.get("id", "")), frozenset(row.get("entries", [])),
                  frozenset(row["rules"]))
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
    original = parsed(text, "the vector file")
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


def check_owners(text: str, entries: list[str], owners: dict[str, frozenset[str]]) -> None:
    """Each override of each adjusted vector has one reviewed owner among the vector's entries,
    and every owner is a Parameter entry of the spec."""
    for candidate in candidates(text):
        for rule in candidate.rules:
            if rule not in owners or not owners[rule] or not owners[rule].issubset(entries):
                raise CannotProbe(f"{candidate.id}: missing or unknown owner for {rule}")
            if len(owners[rule].intersection(candidate.entries)) > 1:
                raise CannotProbe(f"{candidate.id}: ambiguous owners for {rule}")


def owned(candidate: Candidate, entry: str, owners: dict[str, frozenset[str]]) -> frozenset[str]:
    return frozenset(rule for rule in candidate.rules if entry in owners[rule])


def prove(text: str, entries: list[str], owners: dict[str, frozenset[str]],
          run: Callable[[str], bool]) -> dict[str, Verdict]:
    """Each Parameter entry's verdict, from runs of the test on probes of the vectors that
    adjust it."""
    if not run(text):
        raise CannotProbe("the probed test fails on the committed vectors")
    adjusting_candidates = candidates(text)
    check_owners(text, entries, owners)
    depending: dict[tuple[str, str], bool] = {}

    def depends(candidate: Candidate, entry: str) -> bool:
        key = (candidate.id, entry)
        removed = owned(candidate, entry, owners)
        if not removed:
            return False
        if key not in depending:
            fails_at_reference = not run(probe(text, candidate, keep_rules=False,
                                               remove_rules=removed))
            if fails_at_reference and not run(probe(text, candidate, keep_rules=True)):
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

    return {entry: verdict(entry) for entry in entries}


SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in .+$", re.MULTILINE)
RUNNING = re.compile(r"^running (\d+) tests?$", re.MULTILINE)


def completed_test(result: subprocess.CompletedProcess[bytes], failure_code: int,
                   selected: str | None = None) -> bool:
    """Only a completed nonempty libtest suite can pass or refute a vector.

    Build errors, tool failures, crashes and empty/ignored suites are not model answers.
    Exactly one selected test target must complete and agree with its process status.
    """
    output = result.stdout.decode("utf-8", errors="replace")
    summaries = SUMMARY.findall(output)
    running = RUNNING.findall(output)
    if len(summaries) != 1 or len(running) != 1:
        raise CannotProbe("the runner did not complete exactly one test suite")
    status, *counts = summaries[0]
    passed, failed, ignored, measured, _filtered = map(int, counts)
    if passed + failed == 0 or int(running[0]) != passed + failed + ignored + measured:
        raise CannotProbe("the runner did not execute a nonempty consistent test suite")
    if selected is not None:
        names = re.findall(r"^test (\S+) \.\.\. (ok|FAILED|ignored).*$", output, re.MULTILINE)
        if passed + failed != 1 or ignored != 0 or measured != 0 or names != [
                (selected, "ok" if status == "ok" else "FAILED")]:
            raise CannotProbe("the exact selected direct-vector test did not execute alone")
    if status == "ok" and failed == 0 and result.returncode == 0:
        return True
    if status == "FAILED" and failed > 0 and result.returncode == failure_code:
        return False
    raise CannotProbe("test completion and process status disagree")


def test_target(crate: str, root: Path) -> tuple[str, ...]:
    tests = root / "crates/clean" / crate / "tests"
    dedicated = (tests / "vectors.rs").is_file() or (tests / "vectors/main.rs").is_file()
    return ("--test", "vectors") if dedicated else ("--lib",)


@dataclass(frozen=True)
class Artifact:
    """An executable cargo built, and the manifest of its package."""

    manifest: Path
    executable: Path


def artifacts(output: bytes) -> list[Artifact]:
    """The executables in cargo's JSON messages."""
    found: list[Artifact] = []
    for line in output.decode("utf-8").splitlines():
        message = json.loads(line)
        if (isinstance(message, dict) and message.get("reason") == "compiler-artifact"
                and isinstance(message.get("manifest_path"), str)
                and isinstance(message.get("executable"), str)):
            found.append(Artifact(Path(message["manifest_path"]), Path(message["executable"])))
    return found


def build(root: Path, crates: list[str]) -> dict[str, Path]:
    """Each crate's vector test executable, from one cargo run per kind of target."""
    groups: dict[tuple[str, ...], list[str]] = {}
    for crate in crates:
        groups.setdefault(test_target(crate, root), []).append(crate)
    built: dict[str, Path] = {}
    for target, members in groups.items():
        command = ["cargo", "test", "--locked", "--no-run",
                   "--message-format=json-render-diagnostics",
                   *(word for crate in members for word in ("-p", crate)), *target]
        try:
            result = subprocess.run(command, cwd=root, capture_output=True, check=False)
        except OSError as error:
            raise CannotProbe("cargo could not be launched") from error
        if result.returncode != 0:
            sys.stderr.buffer.write(result.stderr)
            raise CannotProbe("the vector tests don't build")
        built.update({a.manifest.parent.name: a.executable for a in artifacts(result.stdout)})
    missing = [crate for crate in crates if crate not in built]
    if missing:
        raise CannotProbe(f"cargo built no vector test for {', '.join(missing)}")
    return {crate: built[crate] for crate in crates}


def passes(executable: Path, cwd: Path, selection: str | None,
           vector_file: Path | None = None) -> bool:
    command = [str(executable), *((selection, "--exact") if selection is not None else ())]
    env = {key: value for key, value in os.environ.items() if key != VECTOR_FILE}
    if vector_file is not None:
        env[VECTOR_FILE] = str(vector_file)
    try:
        result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, check=False)
    except OSError as error:
        raise CannotProbe("the test runner could not be launched") from error
    return completed_test(result, failure_code=101, selected=selection)


def file_runs(executable: Path, cwd: Path, selection: str | None,
              scratch: Path) -> Callable[[str], bool]:
    vector_file = scratch / "vectors.toml"

    def run(text: str) -> bool:
        vector_file.write_text(text, encoding="utf-8")
        return passes(executable, cwd, selection, vector_file)

    return run


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
    raw_owners = parsed(ownership.read_text(encoding="utf-8"), f"{domain}: {ownership.name}").get("owners")
    if not isinstance(raw_owners, dict) or not all(
            isinstance(k, str) and isinstance(v, list) and all(isinstance(e, str) for e in v)
            for k, v in raw_owners.items()):
        raise CannotProbe(f"{domain}: owners must map rules to arrays of Parameter entries")
    return {rule: frozenset(names) for rule, names in raw_owners.items()}


def probe_test(root: Path, domain: str, text: str) -> str | None:
    """Reviewed exact tests execute ordinary vectors without dependent-case assertions."""
    path = root / "tools/cleanroom/adjusted_probe_tests.toml"
    try:
        tests = tomllib.loads(path.read_text(encoding="utf-8")).get("tests", {}) if path.is_file() else {}
        if not isinstance(tests, dict) or not all(isinstance(k, str) and isinstance(v, str)
                and re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*", v)
                for k, v in tests.items()):
            raise ValueError()
    except ValueError as error:
        raise CannotProbe("invalid direct-vector test selection metadata") from error
    selected: str | None = tests.get(domain)
    # Linked cases can refute inherited rules even when the ordinary candidate ignores
    # them. They require reviewed isolation rather than whole-suite probe answers.
    linked = any(isinstance(rows, list) and any(isinstance(row, dict) and "base" in row
                 for row in rows) for rows in parsed(text, f"{domain}: the vector file").values())
    if linked and selected is None:
        raise CannotProbe(f"{domain}: linked vectors need a reviewed direct-vector test")
    return selected


@dataclass(frozen=True)
class Domain:
    """What a domain's proof reads: its spec's Parameter entries, its vectors, their rule
    owners and its direct-vector test."""

    name: str
    entries: list[str]
    text: str
    owners: dict[str, frozenset[str]]
    selected: str | None

    @property
    def crate(self) -> str:
        return self.name.replace("-", "_")


def load(root: Path, name: str) -> Domain:
    spec = (root / "docs/behaviours" / f"{name}.md").read_text(encoding="utf-8")
    text = (root / "scenarios/vectors" / f"{name}.toml").read_text(encoding="utf-8")
    selected = probe_test(root, name, text)
    owners = rule_owners(root, name, bool(candidates(text)))
    return Domain(name, parameter_entries(spec), text, owners, selected)


def report(domain: str, verdicts: dict[str, Verdict]) -> int:
    """Print the domain's verdicts; the number of entries whose adjusted vectors prove nothing."""
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
    print(f"adjusted: {domain}: {proven} of {len(verdicts)} Parameter entries have a vector "
          "that fails when its owned overrides are removed")
    return sum(1 for verdict in verdicts.values() if isinstance(verdict, Unproven))


def verdicts_of(domain: Domain, cwd: Path, executable: Path) -> dict[str, Verdict]:
    with tempfile.TemporaryDirectory() as scratch:
        run = file_runs(executable, cwd, domain.selected, Path(scratch))
        try:
            return prove(domain.text, domain.entries, domain.owners, run)
        except CannotProbe as error:
            raise CannotProbe(f"{domain.name}: {error}") from None


def check_domains(root: Path, names: list[str]) -> int:
    """Prove each domain, one process per CPU; the number of entries whose adjusted vectors
    prove nothing."""
    loaded = [load(root, name) for name in names]
    cwds = [root / "crates/clean" / domain.crate for domain in loaded]
    executables = build(root, [domain.crate for domain in loaded])
    with ProcessPoolExecutor() as pool:
        baselines = pool.map(passes, [executables[domain.crate] for domain in loaded], cwds,
                             [None] * len(loaded))
        for domain, passed in zip(loaded, baselines, strict=True):
            if not passed:
                raise CannotProbe(f"{domain.name}: the vector test fails on the committed vectors")
        for domain in loaded:
            check_owners(domain.text, domain.entries, domain.owners)
        proofs = [pool.submit(verdicts_of, domain, cwd, executables[domain.crate])
                  for domain, cwd in zip(loaded, cwds, strict=True)]
        return sum(report(domain.name, proof.result())
                   for domain, proof in zip(loaded, proofs, strict=True))


def domains(root: Path) -> list[str]:
    return sorted(
        spec.stem for spec in (root / "docs/behaviours").glob("*.md")
        if "." not in spec.stem
        and (root / "scenarios/vectors" / f"{spec.stem}.toml").is_file()
        and (root / "crates/clean" / spec.stem.replace("-", "_")).is_dir()
    )


def check_domain_owners(root: Path, name: str) -> None:
    """The ownership and test selection a probe of the domain needs, checked running no test."""
    domain = load(root, name)
    try:
        check_owners(domain.text, domain.entries, domain.owners)
    except CannotProbe as error:
        # A vector's id alone doesn't say which domain's owners lack the rule.
        raise CannotProbe(f"{name}: {error}") from None
    print(f"adjusted: {name}: every adjusted vector's overrides have a reviewed owner")


def missing(root: Path, name: str) -> int:
    spec = root / "docs/behaviours" / f"{name}.md"
    if not spec.is_file():
        raise CannotProbe(f"no spec {spec}")
    entries = parameter_entries(spec.read_text(encoding="utf-8"))
    vectors_file = root / "scenarios/vectors" / f"{name}.toml"
    text = vectors_file.read_text(encoding="utf-8") if vectors_file.is_file() else ""
    adjusted = {entry for candidate in candidates(text) for entry in candidate.entries}
    without = [entry for entry in entries if entry not in adjusted]
    for entry in without:
        print(f"adjusted: {name}: {entry} has no adjusted vector")
    print(f"adjusted: {name}: {len(entries) - len(without)} of {len(entries)} Parameter entries have an "
          "adjusted vector")
    return len(without)


def main(arguments: list[str]) -> int:
    root = Path.cwd()
    if arguments[:1] == ["--missing"]:
        if len(arguments) == 1:
            print("adjusted_vectors_run: --missing needs a DOMAIN", file=sys.stderr)
            return 2
        try:
            return 1 if sum(missing(root, name) for name in arguments[1:]) else 0
        except CannotProbe as error:
            print(f"adjusted_vectors_run: {error}", file=sys.stderr)
            return 2
    owners_only = arguments[:1] == ["--owners"]
    if owners_only:
        arguments = arguments[1:]
    unproven = 0
    try:
        if owners_only:
            for domain in arguments or domains(root):
                check_domain_owners(root, domain)
        else:
            unproven = check_domains(root, arguments or domains(root))
    except CannotProbe as error:
        print(f"adjusted_vectors_run: {error}", file=sys.stderr)
        return 2
    return 1 if unproven else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
