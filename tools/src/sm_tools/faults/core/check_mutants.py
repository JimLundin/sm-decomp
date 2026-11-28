import hashlib
import json
import re
import tomllib
from collections.abc import Container, Mapping, Sequence
from dataclasses import dataclass
from pathlib import PurePath, PurePosixPath
from typing import TypeIs, cast

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

PLAYED = "played "
PLAYED_LOCK = "scenarios/locks/lsnes.toml"
PLAYED_RUNNER = "crates/verify/src/replay/played.rs"
PLAYED_RUNNER_OWNERS = PurePosixPath(PLAYED_RUNNER).with_suffix("")
PRODUCTION = PurePosixPath("crates/game/src/production")
ADAPTERS = FrozenOrderedSet((
    PurePosixPath("crates/game/src/production.rs"),
    PurePosixPath(PLAYED_RUNNER),
))
OWNER_FOLDERS = FrozenList((PRODUCTION, PLAYED_RUNNER_OWNERS))
OUTSIDE = "fault source is outside production adapters and the played runner"
NOT_RUST = "fault source must be Rust"
MANIFEST = "tools/mutants.toml"
SELECTABLE = "target/selectable-faults"
INDEX = "scenarios/replay/index.toml"
BASELINE = "scenarios/replay/baseline.toml"
DOMAINS = "docs/architecture/domains.toml"
BEHAVIOURS = "docs/behaviours"
PRODUCTION_ROOM_CATALOGUES = FrozenList((
    "world-intro-rooms",
    "world-shaft-entry",
))
FAULT_FIELDS = FrozenList((
    "name",
    "responsibility",
    "file",
    "before",
    "after",
    "case",
))
FAULT_KEYS = FrozenList((*FAULT_FIELDS, "rule"))
DETECTIONS = FrozenList(("difference", "data-failure", "checkpoint-failure"))
RUST_PANIC = 101
REPORTED_EXITS = FrozenList((0, 1))
CHANGED_INPUTS = "worktree inputs changed; coordinator must reconcile"
CATALOGUE_ENTRY = re.compile(r"###\s*([^\W\d_]\w*)")
PLAYED_STATUS = re.compile(r"(?: stop=At\(|^tools: replay played: )(\w+) ")
WORD_START = re.compile(r"(?<!^)(?=[A-Z])")
SECONDS = r" seconds=\d+\.\d{6}\n"
INPUT_CHECK = rf"(?:TIMING input-check{SECONDS})?"
COMPILATION = rf"(?:TIMING compilation{SECONDS})?"


@dataclass(frozen=True, slots=True)
class Fault:
    name: str
    responsibility: str
    file: str
    before: str
    after: str
    case: str
    rule: str | None = None


@dataclass(frozen=True, slots=True)
class Manifest:
    faults: FrozenList[Fault]
    callers: FrozenOrderedDict[str, FrozenList[str]]


@dataclass(frozen=True, slots=True)
class ReplayCase:
    responsibilities: FrozenList[str]
    rules: FrozenList[str]


@dataclass(frozen=True, slots=True)
class Coverage:
    cases: FrozenOrderedDict[str, ReplayCase]
    supported: FrozenOrderedSet[str]
    rules: FrozenList[str]


@dataclass(frozen=True, slots=True)
class Replay:
    status: int
    output: str
    seconds: float = 0.0


@dataclass(frozen=True, slots=True)
class Receipt:
    canonical: str
    status: str | None
    passed: bool

    @property
    def digest(self) -> str:
        return hashlib.sha256(self.canonical.encode()).hexdigest()


@dataclass(frozen=True, slots=True)
class Logged[V]:
    value: V
    lines: FrozenList[str]


def is_table(value: object) -> TypeIs[Mapping[str, object]]:
    return isinstance(value, Mapping)


def is_list(value: object) -> TypeIs[list[object]]:
    return isinstance(value, list)


def parse_toml(text: str) -> Mapping[str, object]:
    return tomllib.loads(text)


def tables(value: object) -> FrozenList[Mapping[str, object]]:
    if not is_list(value) or not all(is_table(item) for item in value):
        message = "expected an array of tables"
        raise ValueError(message)
    return FrozenList(item for item in value if is_table(item))


def strings(value: object) -> FrozenList[str]:
    if not is_list(value) or not all(isinstance(item, str) for item in value):
        message = "expected an array of strings"
        raise ValueError(message)
    return FrozenList(item for item in value if isinstance(item, str))


def text(row: Mapping[str, object], key: str) -> str:
    value = row.get(key)
    if not isinstance(value, str):
        message = f"expected a string {key}"
        raise TypeError(message)
    return value


def timing(subject: str, seconds: float) -> str:
    return f"TIMING {subject} seconds={seconds:.6f}"


def source_path(source: str) -> PurePosixPath:
    relative = PurePosixPath(source)
    if relative.is_absolute() or ".." in relative.parts:
        message = "fault source must be relative without traversal"
        raise ValueError(message)
    return owned_source(relative, ADAPTERS, OWNER_FOLDERS)


def owned_source[P: PurePath](
    path: P, adapters: Container[PurePath], folders: Sequence[PurePath]
) -> P:
    if path not in adapters and not any(
        path.is_relative_to(folder) for folder in folders
    ):
        raise ValueError(OUTSIDE)
    if path.suffix != ".rs":
        raise ValueError(NOT_RUST)
    return path


def fault(row: Mapping[str, object]) -> Fault:
    unknown = FrozenList(key for key in row if key not in FAULT_KEYS)
    if unknown:
        message = f"fault rows have unknown fields {list(unknown)}"
        raise ValueError(message)
    rule = row.get("rule")
    if rule is not None and not isinstance(rule, str):
        message = "a fault's rule must be a string"
        raise ValueError(message)
    name, responsibility, file, before, after, case = (
        text(row, key) for key in FAULT_FIELDS
    )
    return Fault(name, responsibility, file, before, after, case, rule)


def runtime_callers(raw: object) -> FrozenOrderedDict[str, FrozenList[str]]:
    if not is_table(raw):
        message = "runtime_callers must be a table"
        raise ValueError(message)
    for file, names in raw.items():
        if (
            not is_list(names)
            or not names
            or any(not isinstance(name, str) or not name for name in names)
            or len(names) != len(FrozenOrderedSet(names))
        ):
            message = "runtime_callers must list distinct function names per Rust file"
            raise ValueError(message)
        _ = source_path(file)
    return FrozenOrderedDict(
        (file, strings(names)) for file, names in raw.items()
    )


def manifest(source: str) -> Manifest:
    data = parse_toml(source)
    roster = FrozenList(fault(row) for row in tables(data.get("fault", [])))
    names = FrozenOrderedSet(entry.name for entry in roster)
    if not roster or "" in names or len(names) != len(roster):
        message = "fault names must be nonempty and unique"
        raise ValueError(message)
    return Manifest(roster, runtime_callers(data.get("runtime_callers", {})))


def catalogue_domains(domains: str) -> FrozenList[str]:
    models = tables(parse_toml(domains).get("model"))
    return FrozenList((
        *(text(model, "name") for model in models),
        *PRODUCTION_ROOM_CATALOGUES,
    ))


def catalogue_rules(specs: Mapping[str, str]) -> FrozenList[str]:
    rules = FrozenOrderedSet[str]()
    for path, spec in specs.items():
        fenced = False
        for line in (line.strip() for line in spec.splitlines()):
            if line.startswith("```"):
                fenced = not fenced
            elif not fenced and (entry := CATALOGUE_ENTRY.fullmatch(line)):
                rule = f"{PurePosixPath(path).stem}/{entry[1]}"
                if rule in rules:
                    message = f"{path}: duplicate catalogue entry {rule}"
                    raise ValueError(message)
                rules |= FrozenOrderedSet((rule,))
    return FrozenList(sorted(rules))


def coverage(index: str, baseline: str, rules: FrozenList[str]) -> Coverage:
    cases = FrozenOrderedDict(
        (
            text(case, "id"),
            ReplayCase(
                strings(case.get("responsibilities")),
                strings(case.get("rules")),
            ),
        )
        for case in tables(parse_toml(index).get("case"))
    )
    supported = FrozenOrderedSet(
        text(case, "id")
        for case in tables(parse_toml(baseline).get("case"))
        if expected(case) == {"status": "pass"}
    )
    return Coverage(cases, supported, rules)


def expected(case: Mapping[str, object]) -> object:
    if "expected" not in case:
        message = f"baseline case {case.get('id')!r} has no expected outcome"
        raise ValueError(message)
    return case["expected"]


def required_owners(known: Coverage) -> FrozenOrderedSet[str]:
    return FrozenOrderedSet(
        owner
        for case in known.supported
        for owner in known.cases[case].responsibilities
    ) | FrozenOrderedSet(("played",))


def validate_witness(known: Coverage, entry: Fault) -> None:
    if entry.case.startswith(PLAYED):
        if entry.responsibility != "played":
            message = "a played fault must belong to the played responsibility"
            raise ValueError(message)
    elif (
        entry.case not in known.supported
        or entry.responsibility not in known.cases[entry.case].responsibilities
    ):
        message = (
            "fault must witness its responsibility in a reviewed Pass case"
        )
        raise ValueError(message)
    _ = source_path(entry.file)


def validate_rule(known: Coverage, entry: Fault) -> None:
    if entry.rule is None:
        return
    if entry.rule not in known.rules:
        message = (
            f"fault {entry.name} names unknown catalogue rule {entry.rule}"
        )
        raise ValueError(message)
    case = known.cases.get(entry.case)
    if case is not None and entry.rule not in case.rules:
        message = f"case {entry.case} does not witness {entry.rule} for fault {entry.name}"
        raise ValueError(message)


def validate_coverage(known: Coverage, faults: Sequence[Fault]) -> None:
    for entry in faults:
        validate_witness(known, entry)
    if FrozenOrderedSet(entry.responsibility for entry in faults) != (
        required_owners(known)
    ):
        message = (
            "every supported replay responsibility needs a production fault"
        )
        raise ValueError(message)
    for case_id, case in known.cases.items():
        unknown = FrozenList(
            rule for rule in case.rules if rule not in known.rules
        )
        if unknown:
            message = (
                f"case {case_id} names unknown catalogue rule {unknown[0]}"
            )
            raise ValueError(message)
    for entry in faults:
        validate_rule(known, entry)


def require_unique_anchor(entry: Fault, source: bytes) -> int:
    before = entry.before.encode()
    start = source.find(before)
    if (
        not before
        or before == entry.after.encode()
        or start < 0
        or source.find(before, start + 1) >= 0
    ):
        message = f"fault {entry.name} must replace one unique changed fragment of {entry.file}"
        raise ValueError(message)
    return start


def validate_anchors(
    faults: Sequence[Fault], sources: Mapping[str, bytes]
) -> None:
    for entry in faults:
        _ = require_unique_anchor(entry, sources[entry.file])


def check_inputs[P](
    frozen: Mapping[P, bytes | None],
    listed: FrozenOrderedSet[P],
    current: Mapping[P, bytes | None],
) -> None:
    if listed != frozen.keys() or any(
        current.get(path) != original for path, original in frozen.items()
    ):
        raise ValueError(CHANGED_INPUTS)


def json_rows(output: str) -> FrozenList[Mapping[str, object]]:
    decoded = FrozenList(
        cast("object", json.loads(line)) for line in output.splitlines()
    )
    return FrozenList(row for row in decoded if is_table(row))


def build_errors(output: str) -> FrozenList[str]:
    messages = FrozenList(row.get("message") for row in json_rows(output))
    return FrozenList(
        str(message.get("message"))
        for message in messages
        if is_table(message) and message.get("level") == "error"
    )


def is_tools_binary(artifact: Mapping[str, object]) -> bool:
    target = artifact.get("target")
    return (
        artifact.get("reason") == "compiler-artifact"
        and is_table(target)
        and target.get("name") == "tools"
        and target.get("kind") == ["bin"]
    )


def reported_executable(output: str) -> str:
    executables = FrozenList(
        executable
        for artifact in json_rows(output)
        if is_tools_binary(artifact)
        and isinstance(executable := artifact.get("executable"), str)
    )
    if len(executables) != 1:
        message = "build must report exactly one tools executable"
        raise ValueError(message)
    return executables[0]


def replay_command(
    case: str, executable: str, environment: Mapping[str, str]
) -> FrozenList[str]:
    if case.startswith(PLAYED):
        command = (executable, "replay", *case.split(), "--lock", PLAYED_LOCK)
        sources = (("--cache", "SM_CACHE"), ("--rom", "SM_ROM"))
    else:
        command = (executable, "replay", "run", INDEX, "--case", case)
        sources = (
            ("--cache", "SM_CACHE"),
            ("--rom", "SM_ROM"),
            ("--native", "SM_AUDIO_NATIVE"),
        )
    return FrozenList((
        *command,
        *(
            argument
            for flag, variable in sources
            if (value := environment.get(variable))
            for argument in (flag, value)
        ),
    ))


def receipt_of(observed: Mapping[str, object]) -> Receipt:
    outcome = observed.get("outcome")
    status = outcome.get("status") if is_table(outcome) else None
    return Receipt(
        json.dumps(observed, sort_keys=True, separators=(",", ":")),
        status if isinstance(status, str) else None,
        is_table(outcome) and outcome == {"status": "pass"},
    )


def played_receipt(output: str, case: str) -> Receipt | None:
    _, row, start, _ = case.split()
    lines = FrozenList(
        line
        for line in output.splitlines()
        if line.startswith((
            f"PLAYED_ALONE row={row} start={start} ",
            "tools: replay played: ",
        ))
    )
    if len(lines) != 1:
        return None
    [line] = lines
    status = "other"
    if " stop=End " in line:
        status = "pass"
    elif found := PLAYED_STATUS.search(line):
        status = WORD_START.sub("-", found[1]).lower()
    return receipt_of({
        "id": case,
        "report": line,
        "outcome": {"status": status},
    })


def receipt(output: str, case: str) -> Receipt | None:
    if case.startswith(PLAYED):
        return played_receipt(output, case)
    for prefix in ("format = 1", "tools: format = 1"):
        start = output.find(prefix)
        if start < 0:
            continue
        try:
            parsed = parse_toml(output[start:].removeprefix("tools: "))
        except tomllib.TOMLDecodeError:
            continue
        cases = parsed.get("cases")
        if parsed.get("format") == 1 and is_list(cases) and len(cases) == 1:
            observed = cases[0]
            if is_table(observed) and observed.get("id") == case:
                return receipt_of(observed)
    return None


def replay_timing(case: str, replay: Replay) -> str:
    return timing(f"replay case={case}", replay.seconds)


def crash_receipt(case: str, status: int) -> Receipt:
    return receipt_of({
        "id": case,
        "outcome": {"status": "crash", "exit": status},
    })


def detected(
    entry: Fault, replay: Replay, crashes: Sequence[int]
) -> Logged[Receipt | None]:
    observed = receipt(replay.output, entry.case)
    status = observed.status if observed is not None else None
    code = replay.status
    if code in crashes:
        return Logged(
            crash_receipt(entry.case, code),
            FrozenList((
                f"KILLED {entry.name}: {entry.case} crash exit={code}",
            )),
        )
    if code in REPORTED_EXITS and status in DETECTIONS:
        return Logged(
            observed,
            FrozenList((f"KILLED {entry.name}: {entry.case} {status}",)),
        )
    verdict = "SURVIVED" if status == "pass" else "INVALID"
    return Logged(
        None,
        FrozenList((
            f"{verdict} {entry.name}: {entry.case} status={status} exit={code}",
        )),
    )


def restored_pass(
    entry: Fault, replay: Replay, baseline: Receipt, faulty: Receipt | None
) -> Logged[bool]:
    restored = receipt(replay.output, entry.case)
    if (
        replay.status
        or restored is None
        or restored != baseline
        or not restored.passed
    ):
        return Logged(
            value=False,
            lines=FrozenList((f"RESTORED BASELINE FAILED {entry.case}",)),
        )
    passed = f"RESTORED PASS {entry.case}"
    if faulty is None:
        return Logged(value=False, lines=FrozenList((passed,)))
    digests = f"{baseline.digest} {faulty.digest} {restored.digest}"
    return Logged(
        value=True,
        lines=FrozenList((passed, f"RECEIPTS {entry.name} {digests}")),
    )


def baseline_verdict(case: str, replay: Replay) -> Logged[Receipt | None]:
    baseline = receipt(replay.output, case)
    started = replay_timing(case, replay)
    if replay.status or baseline is None or not baseline.passed:
        return Logged(None, FrozenList((started, f"BASELINE FAILED {case}")))
    return Logged(baseline, FrozenList((started, f"BASELINE PASS {case}")))


def fault_verdict(
    entry: Fault,
    faulty: Replay,
    restored: Replay,
    baseline: Receipt,
    crashes: Sequence[int],
) -> Logged[bool]:
    detection = detected(entry, faulty, crashes)
    restoration = restored_pass(entry, restored, baseline, detection.value)
    return Logged(
        restoration.value,
        FrozenList((
            replay_timing(entry.case, faulty),
            *detection.lines,
            f"SELECTOR OFF fault={entry.name}",
            replay_timing(entry.case, restored),
            *restoration.lines,
        )),
    )


def kill_and_restoration(entry: Fault) -> re.Pattern[str]:
    name, case = re.escape(entry.name), re.escape(entry.case)
    replay = rf"TIMING replay case={case}{SECONDS}"
    kill = rf"^{replay}KILLED {name}: {case} [^\n]+\n"
    rebuilt = (
        rf"TIMING restoration fault={name}{SECONDS}{INPUT_CHECK}{COMPILATION}"
    )
    selector_off = rf"SELECTOR OFF fault={name}\n{INPUT_CHECK}"
    resumed = rf"{replay}{INPUT_CHECK}RESTORED PASS {case}$(?!\nRESTORED PASS )"
    return re.compile(
        f"{kill}(?:{rebuilt}|{selector_off}){resumed}", re.MULTILINE
    )


def rule_coverage(
    rules: Sequence[str], faults: Sequence[Fault], logs: Sequence[str]
) -> FrozenList[str]:
    def status(rule: str) -> str:
        confirmed = FrozenList(
            entry.name
            for entry in faults
            if entry.rule == rule
            and any(kill_and_restoration(entry).search(log) for log in logs)
        )
        if confirmed:
            return f"killed through {', '.join(confirmed)}"
        return "unknown"

    unassigned = sum(entry.rule is None for entry in faults)
    return FrozenList((
        *(f"{rule}: {status(rule)}" for rule in rules),
        f"Unassigned faults: {unassigned}",
    ))
