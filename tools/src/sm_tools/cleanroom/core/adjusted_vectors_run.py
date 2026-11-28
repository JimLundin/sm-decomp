import functools
import json
import re
import tomllib
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import PurePosixPath
from typing import TypeIs, cast

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

USAGE = """Require each Parameter entry's adjusted vector to depend on the value it adjusts.

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

Usage: adjusted-vectors-run [--owners | --missing] [DOMAIN...], from the repository root;
with no DOMAIN, every spec with a vector file and a clean crate. --owners checks only the
ownership and test selection a probe needs, and runs no test: the gate's form. --missing lists
each Parameter entry no vector adjusts, and needs a DOMAIN; the gate runs it on each domain whose
spec or vectors a branch changes. Exit 1 when an entry has adjusted vectors and none depends on
its rules, or under --missing when an entry has none; 2 when a domain can't be probed (its test
doesn't build, fails as committed or fails the control); else 0. An entry no vector adjusts is
listed and left to --missing, which gates it at the domain's spec round."""

NAME = "adjusted_vectors_run"
HELP = FrozenList((FrozenList(("-h",)), FrozenList(("--help",))))
SWITCH = re.compile(r"^- switch: *\**parameter", re.IGNORECASE)
PROBE_SUFFIX = "-probe"
VECTOR_FILE = "SM_VECTOR_FILE"
OWNERS_FOLDER = "tools/cleanroom/adjusted_rule_owners"
PROBE_TESTS = "tools/cleanroom/adjusted_probe_tests.toml"
SPECS = "docs/behaviours"
VECTORS = "scenarios/vectors"
CLEAN_CRATES = "crates/clean"
LIBTEST_FAILED = 101
DEDICATED_TARGET = FrozenList(("--test", "vectors"))
LIBRARY_TARGET = FrozenList(("--lib",))
SELECTION = re.compile(r"[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*")
SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in .+$",
    re.MULTILINE,
)
RUNNING = re.compile(r"^running (\d+) tests?$", re.MULTILINE)
TEST_LINE = re.compile(
    r"^test (\S+) \.\.\. (ok|FAILED|ignored).*$", re.MULTILINE
)
RULES_TABLE = "[vector.rules]"
VECTOR_TABLE = "[[vector]]"
INLINE_RULES = re.compile(r"rules\s*=")
VECTOR_ID = re.compile(r'^(id\s*=\s*)"[^"]*"')

type Table = Mapping[str, object]
type Owners = Mapping[str, FrozenOrderedSet[str]]


class CannotProbeError(Exception):
    pass


@dataclass(frozen=True, slots=True)
class Candidate:
    index: int
    id: str
    entries: FrozenOrderedSet[str]
    rules: FrozenOrderedSet[str]


@dataclass(frozen=True, slots=True)
class Proven:
    vector: str
    shared_with: FrozenList[str]


@dataclass(frozen=True, slots=True)
class Unproven:
    pass


@dataclass(frozen=True, slots=True)
class Unadjusted:
    pass


type Verdict = Proven | Unproven | Unadjusted
type Verdicts = FrozenOrderedDict[str, Verdict]


@dataclass(frozen=True, slots=True)
class RunProbe:
    text: str


@dataclass(frozen=True, slots=True)
class Artifact:
    manifest: PurePosixPath
    executable: PurePosixPath


@dataclass(frozen=True, slots=True)
class Domain:
    name: str
    entries: FrozenList[str]
    text: str
    owners: Owners
    selected: str | None

    @property
    def crate(self) -> str:
        return crate_of(self.name)


@dataclass(frozen=True, slots=True)
class Report:
    lines: FrozenList[str]
    unproven: int


class UnansweredProbeError(Exception):
    def __init__(self, text: str) -> None:
        super().__init__(text)
        self.text: str = text


def is_table(value: object) -> TypeIs[Table]:
    return isinstance(value, Mapping)


def is_list(value: object) -> TypeIs[list[object]]:
    return isinstance(value, list)


def crate_of(domain: str) -> str:
    return domain.replace("-", "_")


def parameter_entries(spec: str) -> FrozenList[str]:
    found: list[str] = []
    entry = ""
    for line in spec.splitlines():
        if line.startswith("### "):
            entry = line.split()[1]
        elif entry and SWITCH.match(line):
            found.append(entry)
            entry = ""
    return FrozenList(found)


@functools.lru_cache(maxsize=2)
def parsed(text: str, what: str) -> Table:
    try:
        return tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        message = f"{what} is not TOML: {error}"
        raise CannotProbeError(message) from None


def vectors(text: str) -> FrozenList[Table]:
    rows = parsed(text, "the vector file").get("vector", [])
    if not is_list(rows) or not all(is_table(row) for row in rows):
        message = "`vector` is not an array of tables"
        raise CannotProbeError(message)
    return FrozenList(row for row in rows if is_table(row))


def strings(value: object) -> FrozenOrderedSet[str]:
    items = value if is_list(value) else []
    return FrozenOrderedSet(item for item in items if isinstance(item, str))


def rule_names(row: Table) -> FrozenOrderedSet[str]:
    rules = row["rules"]
    if not is_table(rules):
        message = f"{row.get('id', '')}: rules is not a table"
        raise CannotProbeError(message)
    return FrozenOrderedSet(rules)


def candidates(text: str) -> FrozenList[Candidate]:
    return FrozenList(
        Candidate(
            index,
            str(row.get("id", "")),
            strings(row.get("entries", [])),
            rule_names(row),
        )
        for index, row in enumerate(vectors(text))
        if row.get("rules")
    )


def block_bounds(text: str, index: int) -> tuple[int, int]:
    lines = text.splitlines()
    starts = [
        n for n, line in enumerate(lines) if line.startswith(VECTOR_TABLE)
    ]
    start = starts[index]
    end = next(
        (n for n in range(start + 1, len(lines)) if lines[n].startswith("[[")),
        len(lines),
    )
    return start, end


def in_rules_table(table: str) -> bool:
    return table == RULES_TABLE or table.startswith((
        "[vector.rules.",
        "[[vector.rules.",
    ))


def without_rules(lines: Sequence[str]) -> FrozenList[str]:
    kept: list[str] = []
    table = ""
    for line in lines:
        if line.startswith("["):
            table = line.strip()
        inline_rules = table == VECTOR_TABLE and INLINE_RULES.match(line)
        if in_rules_table(table) or inline_rules:
            continue
        kept.append(line)
    return FrozenList(kept)


def inline_toml(value: object) -> str:
    if is_table(value):
        members = ", ".join(
            json.dumps(key) + " = " + inline_toml(item)
            for key, item in value.items()
        )
        return "{ " + members + " }"
    if is_list(value):
        return "[" + ", ".join(inline_toml(item) for item in value) + "]"
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int | float):
        return repr(value)
    message = "unsupported rule value"
    raise CannotProbeError(message)


def renamed(lines: Sequence[str], probe_id: str) -> FrozenList[str]:
    out: list[str] = []
    table = ""
    for line in lines:
        if line.startswith("["):
            table = line.strip()
        out.append(
            VECTOR_ID.sub(lambda m: f'{m.group(1)}"{probe_id}"', line)
            if table == VECTOR_TABLE
            else line
        )
    return FrozenList(out)


def rules_of(row: Table) -> dict[str, object]:
    rules = row.get("rules", {})
    return dict(rules) if is_table(rules) else {}


def remaining_rules(
    candidate: Candidate, row: Table, removed: FrozenOrderedSet[str] | None
) -> dict[str, object]:
    rules = rules_of(row)
    for rule in removed if removed is not None else FrozenList(rules):
        if rule not in rules:
            message = f"{candidate.id}: unknown override {rule}"
            raise CannotProbeError(message)
        del rules[rule]
    return rules


@functools.lru_cache(maxsize=256)
def probe(
    text: str,
    candidate: Candidate,
    *,
    keep_rules: bool,
    remove_rules: FrozenOrderedSet[str] | None = None,
) -> str:
    original = parsed(text, "the vector file")
    rows = vectors(text)
    probe_id = candidate.id + PROBE_SUFFIX
    ids = FrozenList(row.get("id") for row in rows)
    if len(FrozenOrderedSet(ids)) != len(ids) or probe_id in ids:
        message = f"{candidate.id}: vector ids must stay unique"
        raise CannotProbeError(message)
    row = rows[candidate.index]
    if row.get("id") != candidate.id:
        message = f"{candidate.id}: candidate doesn't identify its vector"
        raise CannotProbeError(message)
    start, end = block_bounds(text, candidate.index)
    every_line = text.splitlines()
    lines = FrozenList(every_line[start:end])
    rules = rules_of(row)
    if not keep_rules:
        rules = remaining_rules(candidate, row, remove_rules)
        stripped = without_rules(lines)
        lines = (
            FrozenList((
                stripped[0],
                f"rules = {inline_toml(rules)}",
                *stripped[1:],
            ))
            if rules
            else stripped
        )
    probed = (
        "\n".join((
            *every_line[:start],
            *renamed(lines, probe_id),
            *every_line[end:],
        ))
        + "\n"
    )
    expected = {**row, "id": probe_id}
    if not keep_rules:
        if rules:
            expected["rules"] = rules
        else:
            del expected["rules"]
    expected_rows = [*rows]
    expected_rows[candidate.index] = expected
    if tomllib.loads(probed) != {**original, "vector": expected_rows}:
        message = (
            f"{candidate.id}: replacement changes the case or unrelated content"
        )
        raise CannotProbeError(message)
    return probed


def check_owners(text: str, entries: Sequence[str], owners: Owners) -> None:
    for candidate in candidates(text):
        for rule in candidate.rules:
            owner = owners.get(rule)
            if not owner or not owner <= FrozenOrderedSet(entries):
                message = f"{candidate.id}: missing or unknown owner for {rule}"
                raise CannotProbeError(message)
            if len(owner & candidate.entries) > 1:
                message = f"{candidate.id}: ambiguous owners for {rule}"
                raise CannotProbeError(message)


def owned(
    candidate: Candidate, entry: str, owners: Owners
) -> FrozenOrderedSet[str]:
    return FrozenOrderedSet(
        rule for rule in candidate.rules if entry in owners[rule]
    )


class Proof:
    def __init__(
        self,
        text: str,
        entries: Sequence[str],
        owners: Owners,
        answers: Sequence[bool],
    ) -> None:
        self.text: str = text
        self.entries: FrozenList[str] = FrozenList(entries)
        self.owners: Owners = owners
        self.answers: FrozenList[bool] = FrozenList(answers)
        self.asked: int = 0
        self.depending: dict[tuple[str, str], bool] = {}

    def run(self, probed: str) -> bool:
        if self.asked == len(self.answers):
            raise UnansweredProbeError(probed)
        self.asked += 1
        return self.answers[self.asked - 1]

    def depends(self, candidate: Candidate, entry: str) -> bool:
        key = (candidate.id, entry)
        removed = owned(candidate, entry, self.owners)
        if not removed:
            return False
        if key not in self.depending:
            reference = probe(
                self.text, candidate, keep_rules=False, remove_rules=removed
            )
            fails_at_reference = not self.run(reference)
            if fails_at_reference and not self.run(
                probe(self.text, candidate, keep_rules=True)
            ):
                message = f"{candidate.id}: the test refuses a copy of it"
                raise CannotProbeError(message)
            self.depending[key] = fails_at_reference
        return self.depending[key]

    def others(self, candidate: Candidate, entry: str) -> FrozenList[str]:
        return FrozenList(
            other
            for other in self.entries
            if other != entry and other in candidate.entries
        )

    def verdict(
        self, entry: str, adjusting_candidates: Sequence[Candidate]
    ) -> Verdict:
        adjusting = sorted(
            (c for c in adjusting_candidates if entry in c.entries),
            key=lambda c: len(self.others(c, entry)),
        )
        if not adjusting:
            return Unadjusted()
        return next(
            (
                Proven(c.id, self.others(c, entry))
                for c in adjusting
                if self.depends(c, entry)
            ),
            Unproven(),
        )

    def verdicts(self) -> Verdicts:
        if not self.run(self.text):
            message = "the probed test fails on the committed vectors"
            raise CannotProbeError(message)
        adjusting = candidates(self.text)
        check_owners(self.text, self.entries, self.owners)
        return FrozenOrderedDict(
            (entry, self.verdict(entry, adjusting)) for entry in self.entries
        )


def step(
    text: str, entries: Sequence[str], owners: Owners, answers: Sequence[bool]
) -> RunProbe | Verdicts:
    try:
        return Proof(text, entries, owners, answers).verdicts()
    except UnansweredProbeError as pending:
        return RunProbe(pending.text)


def completed_test(
    output: bytes,
    returncode: int,
    failure_code: int,
    selected: str | None = None,
) -> bool:
    printed = output.decode("utf-8", errors="replace")
    summaries = FrozenList(SUMMARY.finditer(printed))
    running = FrozenList(RUNNING.finditer(printed))
    if len(summaries) != 1 or len(running) != 1:
        message = "the runner did not complete exactly one test suite"
        raise CannotProbeError(message)
    summary = summaries[0]
    status = summary.group(1)
    passed, failed, ignored, measured = (
        int(summary.group(group)) for group in range(2, 6)
    )
    if passed + failed == 0 or int(running[0].group(1)) != (
        passed + failed + ignored + measured
    ):
        message = "the runner did not execute a nonempty consistent test suite"
        raise CannotProbeError(message)
    if selected is not None:
        names = FrozenList(
            (found.group(1), found.group(2))
            for found in TEST_LINE.finditer(printed)
        )
        alone = FrozenList(((selected, "ok" if status == "ok" else "FAILED"),))
        if passed + failed != 1 or ignored or measured or names != alone:
            message = (
                "the exact selected direct-vector test did not execute alone"
            )
            raise CannotProbeError(message)
    if status == "ok" and failed == 0 and returncode == 0:
        return True
    if status == "FAILED" and failed > 0 and returncode == failure_code:
        return False
    message = "test completion and process status disagree"
    raise CannotProbeError(message)


def test_target(*, dedicated: bool) -> FrozenList[str]:
    return DEDICATED_TARGET if dedicated else LIBRARY_TARGET


def decoded(line: str) -> object:
    return cast("object", json.loads(line))


def artifacts(output: bytes) -> FrozenList[Artifact]:
    found: list[Artifact] = []
    for line in output.decode("utf-8").splitlines():
        message = decoded(line)
        if (
            not is_table(message)
            or message.get("reason") != "compiler-artifact"
        ):
            continue
        manifest = message.get("manifest_path")
        executable = message.get("executable")
        if isinstance(manifest, str) and isinstance(executable, str):
            found.append(
                Artifact(PurePosixPath(manifest), PurePosixPath(executable))
            )
    return FrozenList(found)


def build_groups(
    targets: Sequence[tuple[str, FrozenList[str]]],
) -> FrozenOrderedDict[FrozenList[str], FrozenList[str]]:
    groups: dict[FrozenList[str], list[str]] = {}
    for crate, target in targets:
        groups.setdefault(target, []).append(crate)
    return FrozenOrderedDict(
        (target, FrozenList(members)) for target, members in groups.items()
    )


def build_command(
    members: Sequence[str], target: Sequence[str]
) -> FrozenList[str]:
    return FrozenList((
        "cargo",
        "test",
        "--locked",
        "--no-run",
        "--message-format=json-render-diagnostics",
        *(word for crate in members for word in ("-p", crate)),
        *target,
    ))


def built_crates(
    artifacts_found: Sequence[Artifact],
) -> FrozenOrderedDict[str, PurePosixPath]:
    return FrozenOrderedDict(
        (found.manifest.parent.name, found.executable)
        for found in artifacts_found
    )


def unbuilt(crates: Sequence[str], built: Mapping[str, object]) -> str | None:
    missing = FrozenList(crate for crate in crates if crate not in built)
    return (
        f"cargo built no vector test for {', '.join(missing)}"
        if missing
        else None
    )


def test_command(executable: str, selection: str | None) -> FrozenList[str]:
    exact = (selection, "--exact") if selection is not None else ()
    return FrozenList((executable, *exact))


def without_vector_file(
    environment: Mapping[str, str],
) -> FrozenOrderedDict[str, str]:
    return FrozenOrderedDict(
        (key, value) for key, value in environment.items() if key != VECTOR_FILE
    )


def rule_owners(text: str | None, domain: str, *, required: bool) -> Owners:
    if text is None:
        if not required:
            return FrozenOrderedDict()
        message = f"{domain}: missing reviewed rule ownership"
        raise CannotProbeError(message)
    raw = parsed(text, f"{domain}: {domain}.toml").get("owners")
    if not is_table(raw) or not all(
        is_list(names) and all(isinstance(name, str) for name in names)
        for names in raw.values()
    ):
        message = (
            f"{domain}: owners must map rules to arrays of Parameter entries"
        )
        raise CannotProbeError(message)
    return FrozenOrderedDict(
        (rule, strings(names)) for rule, names in raw.items()
    )


def selections(metadata: bytes | None) -> Mapping[str, str]:
    try:
        tests: object = (
            tomllib.loads(metadata.decode()).get("tests", {})
            if metadata is not None
            else FrozenOrderedDict[str, object]()
        )
    except ValueError:
        message = "invalid direct-vector test selection metadata"
        raise CannotProbeError(message) from None
    if not is_table(tests) or not all(
        isinstance(value, str) and SELECTION.fullmatch(value)
        for value in tests.values()
    ):
        message = "invalid direct-vector test selection metadata"
        raise CannotProbeError(message)
    return FrozenOrderedDict(
        (domain, value)
        for domain, value in tests.items()
        if isinstance(value, str)
    )


def linked(text: str, domain: str) -> bool:
    return any(
        is_list(rows) and any(is_table(row) and "base" in row for row in rows)
        for rows in parsed(text, f"{domain}: the vector file").values()
    )


def probe_test(metadata: bytes | None, domain: str, text: str) -> str | None:
    selected = selections(metadata).get(domain)
    if linked(text, domain) and selected is None:
        message = f"{domain}: linked vectors need a reviewed direct-vector test"
        raise CannotProbeError(message)
    return selected


def domain(
    name: str, spec: str, text: str, owners: str | None, metadata: bytes | None
) -> Domain:
    selected = probe_test(metadata, name, text)
    reviewed = rule_owners(owners, name, required=bool(candidates(text)))
    return Domain(name, parameter_entries(spec), text, reviewed, selected)


def report(domain_name: str, verdicts: Mapping[str, Verdict]) -> Report:
    lines: list[str] = []
    for entry, verdict in verdicts.items():
        match verdict:
            case Unadjusted():
                lines.append(
                    f"adjusted: {domain_name}: {entry} has no adjusted vector"
                )
            case Unproven():
                lines.append(
                    f"adjusted: {domain_name}: no named vector depends on {entry}'s owned rule overrides"
                )
            case Proven(vector, ()):
                lines.append(
                    f"adjusted: {domain_name}: {entry} depends on its rules in {vector}"
                )
            case Proven(vector, shared_with):
                lines.append(
                    f"adjusted: {domain_name}: {entry} depends on its rules in {vector}, which also names {', '.join(shared_with)}"
                )
    proven = sum(
        1 for verdict in verdicts.values() if isinstance(verdict, Proven)
    )
    lines.append(
        f"adjusted: {domain_name}: {proven} of {len(verdicts)} Parameter entries have a vector that fails when its owned overrides are removed"
    )
    unproven = sum(
        1 for verdict in verdicts.values() if isinstance(verdict, Unproven)
    )
    return Report(FrozenList(lines), unproven)


def owners_line(name: str) -> str:
    return f"adjusted: {name}: every adjusted vector's overrides have a reviewed owner"


def missing(name: str, spec: str, text: str) -> Report:
    entries = parameter_entries(spec)
    adjusted = FrozenOrderedSet(
        entry for candidate in candidates(text) for entry in candidate.entries
    )
    without = FrozenList(entry for entry in entries if entry not in adjusted)
    lines = FrozenList((
        *(
            f"adjusted: {name}: {entry} has no adjusted vector"
            for entry in without
        ),
        (
            f"adjusted: {name}: {len(entries) - len(without)} of {len(entries)} Parameter entries have an adjusted vector"
        ),
    ))
    return Report(lines, len(without))
