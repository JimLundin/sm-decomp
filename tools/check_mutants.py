import argparse
import contextlib
import functools
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import tomllib
from collections.abc import Callable, Generator, Mapping, Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Any, TypeVar

# A case that starts with this is a lock row played alone, `played ROW START END` as
# `tools replay played` takes it; any other case is an indexed replay case.
PLAYED = "played "
PLAYED_LOCK = "scenarios/locks/lsnes.toml"
PLAYED_RUNNER = "crates/verify/src/replay/played.rs"

@dataclass(frozen=True)
class Fault:
    name: str
    responsibility: str
    file: str
    before: str
    after: str
    case: str
    rule: str | None = None

def source_path(root: Path, source: str) -> Path:
    relative = Path(source)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("fault source must be relative without traversal")
    path = (root / relative).resolve()
    production = (root / "crates/game/src/production").resolve()
    adapters = {(root / "crates/game/src/production.rs").resolve(), (root / PLAYED_RUNNER).resolve()}
    # The played runner's owners are its own modules, in the folder named after it.
    runner = (root / PLAYED_RUNNER).with_suffix("").resolve()
    if path not in adapters and not path.is_relative_to(production) and not path.is_relative_to(runner):
        raise ValueError("fault source is outside production adapters and the played runner")
    if path.suffix != ".rs":
        raise ValueError("fault source must be Rust")
    return path

def input_paths(root: Path) -> set[Path]:
    result = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
                            cwd=root, check=True, capture_output=True)
    return {root / os.fsdecode(name) for name in result.stdout.split(b"\0") if name}

def input_bytes(path: Path) -> bytes | None:
    return path.read_bytes() if path.exists() else None

@dataclass(frozen=True)
class FrozenInputs:
    """Include dirty and nonignored untracked inputs; preserve absent tracked files."""
    files: dict[Path, bytes | None]
    listing: Callable[[Path], set[Path]] | None = None

    @classmethod
    def capture(cls, root: Path, listing: Callable[[Path], set[Path]] | None = None) -> "FrozenInputs":
        paths = (listing or input_paths)(root)
        return cls({path: input_bytes(path) for path in paths}, listing)

    def copy_to(self, root: Path, tree: Path) -> None:
        for path, original in self.files.items():
            if original is not None:
                copied = tree / path.relative_to(root)
                copied.parent.mkdir(parents=True, exist_ok=True)
                copied.write_bytes(original)
                shutil.copymode(path, copied)
        print(self.check(root), flush=True)

    def check(self, root: Path) -> str:
        started = time.perf_counter()
        if (self.listing or input_paths)(root) != self.files.keys() or any(
            input_bytes(path) != original for path, original in self.files.items()
        ):
            raise ValueError("worktree inputs changed; coordinator must reconcile")
        return f"TIMING input-check seconds={time.perf_counter() - started:.6f}"

def build_executable(root: Path, *, target: Path) -> Path:
    """Use Cargo's artifact path, including custom target directories and triples."""
    started = time.perf_counter()
    result = subprocess.run([os.environ.get("CARGO", "cargo"), "build", "--locked",
                             "--profile", "verify", "-p", "tools", "--bin", "tools",
                             "--message-format=json", "--target-dir", str(target)], cwd=root, capture_output=True,
                            text=True, encoding="utf-8", errors="replace")
    print(f"TIMING compilation seconds={time.perf_counter() - started:.6f}", flush=True)
    if result.returncode:
        for line in result.stdout.splitlines():
            message = json.loads(line).get("message")
            if isinstance(message, dict) and message.get("level") == "error":
                print(message.get("message"), file=sys.stderr)
        result.check_returncode()
    executables = []
    for line in result.stdout.splitlines():
        artifact = json.loads(line)
        if (artifact.get("reason") == "compiler-artifact"
            and artifact.get("target", {}).get("name") == "tools"
            and artifact.get("target", {}).get("kind") == ["bin"]
            and isinstance(artifact.get("executable"), str)):
            executables.append(Path(artifact["executable"]))
    if len(executables) != 1:
        raise ValueError("build must report exactly one tools executable")
    executable = (root / executables[0]).resolve()
    if not executable.is_file():
        raise ValueError("reported tools executable is missing")
    return executable

def run_case(root: Path, case: str, executable: Path, *, env: Mapping[str, str],
             log: list[str]) -> tuple[int, str]:
    if case.startswith(PLAYED):
        command = [str(executable), "replay", *case.split(), "--lock", PLAYED_LOCK]
        sources = [("--cache", "SM_CACHE"), ("--rom", "SM_ROM")]
    else:
        command = [str(executable), "replay", "run", "scenarios/replay/index.toml", "--case", case]
        sources = [("--cache", "SM_CACHE"), ("--rom", "SM_ROM"), ("--native", "SM_AUDIO_NATIVE")]
    for flag, variable in sources:
        if value := env.get(variable):
            command.extend([flag, value])
    started = time.perf_counter()
    result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True,
                            encoding="utf-8", errors="replace")
    log.append(f"TIMING replay case={case} seconds={time.perf_counter() - started:.6f}")
    return result.returncode, result.stdout if result.returncode == 0 else result.stderr

def played_receipt(output: str, case: str) -> dict[str, Any] | None:
    _, row, start, _ = case.split()
    lines = [line for line in output.splitlines()
             if line.startswith((f"PLAYED_ALONE row={row} start={start} ", "tools: replay played: "))]
    if len(lines) != 1:
        return None
    status = "other"
    if " stop=End " in lines[0]:
        status = "pass"
    elif found := re.search(r"(?: stop=At\(|^tools: replay played: )(\w+) ", lines[0]):
        status = re.sub(r"(?<!^)(?=[A-Z])", "-", found[1]).lower()
    return {"id": case, "report": lines[0], "outcome": {"status": status}}

def receipt(output: str, case: str) -> dict[str, Any] | None:
    if case.startswith(PLAYED):
        return played_receipt(output, case)
    for prefix in ("format = 1", "tools: format = 1"):
        start = output.find(prefix)
        if start < 0:
            continue
        try:
            parsed = tomllib.loads(output[start:].removeprefix("tools: "))
        except tomllib.TOMLDecodeError:
            continue
        cases = parsed.get("cases")
        if parsed.get("format") == 1 and isinstance(cases, list) and len(cases) == 1:
            observed = cases[0]
            if isinstance(observed, dict) and observed.get("id") == case:
                return observed
    return None

def require_unique_anchor(fault: Fault, source: bytes) -> int:
    """A refactor of the patched lines can drift an anchor; it must still name one place."""
    before = fault.before.encode()
    start = source.find(before)
    if not before or before == fault.after.encode() or start < 0 or source.find(before, start + 1) >= 0:
        raise ValueError(f"fault {fault.name} must replace one unique changed fragment of {fault.file}")
    return start

def validate_anchors(root: Path, faults: Sequence[Fault]) -> None:
    for fault in faults:
        require_unique_anchor(fault, source_path(root, fault.file).read_bytes())

DETECTIONS = ("difference", "data-failure", "checkpoint-failure")
# A Rust panic exits 101; these signals are the replay's own crash. SIGKILL is left out because the
# OOM killer or the runner sends it, which says nothing about the fault.
CRASHES = (101, *(-number for number in (signal.SIGSEGV, signal.SIGABRT, signal.SIGILL,
                                         signal.SIGBUS, signal.SIGFPE)))


def detected(fault: Fault, result: tuple[int, str], log: list[str]) -> dict[str, Any] | None:
    code, output = result
    observed = receipt(output, fault.case)
    status = observed["outcome"].get("status") if observed is not None else None
    if code in CRASHES:
        log.append(f"KILLED {fault.name}: {fault.case} crash exit={code}")
        return {"id": fault.case, "outcome": {"status": "crash", "exit": code}}
    if code in (0, 1) and status in DETECTIONS:
        log.append(f"KILLED {fault.name}: {fault.case} {status}")
        return observed
    verdict = "SURVIVED" if status == "pass" else "INVALID"
    log.append(f"{verdict} {fault.name}: {fault.case} status={status} exit={code}")
    return None


def receipt_digest(observed: Mapping[str, Any]) -> str:
    return hashlib.sha256(json.dumps(observed, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def restored_pass(fault: Fault, result: tuple[int, str], baseline: Mapping[str, Any],
                  faulty: Mapping[str, Any] | None, log: list[str]) -> bool:
    restored = receipt(result[1], fault.case)
    if result[0] or restored != baseline or restored is None or restored.get("outcome") != {"status": "pass"}:
        log.append(f"RESTORED BASELINE FAILED {fault.case}")
        return False
    log.append(f"RESTORED PASS {fault.case}")
    if faulty is None:
        return False
    log.append(f"RECEIPTS {fault.name} {receipt_digest(baseline)} {receipt_digest(faulty)} {receipt_digest(restored)}")
    return True


def validate_coverage(root: Path, faults: Sequence[Fault]) -> list[str]:
    """Every responsibility with a reviewed Pass, and the played runner, needs its own fault."""
    with (root / "scenarios/replay/index.toml").open("rb") as stream:
        cases = {case["id"]: case for case in tomllib.load(stream)["case"]}
    with (root / "scenarios/replay/baseline.toml").open("rb") as stream:
        supported = {case["id"] for case in tomllib.load(stream)["case"]
                     if case["expected"] == {"status": "pass"}}
    required = {owner for case in supported for owner in cases[case]["responsibilities"]} | {"played"}
    for fault in faults:
        if fault.case.startswith(PLAYED):
            if fault.responsibility != "played":
                raise ValueError("a played fault must belong to the played responsibility")
        elif fault.case not in supported or fault.responsibility not in cases[fault.case]["responsibilities"]:
            raise ValueError("fault must witness its responsibility in a reviewed Pass case")
        source_path(root, fault.file)
    if {fault.responsibility for fault in faults} != required:
        raise ValueError("every supported replay responsibility needs a production fault")
    rules = catalogue_rules(root)
    for case_id, case in cases.items():
        for rule in case["rules"]:
            if rule not in rules:
                raise ValueError(f"case {case_id} names unknown catalogue rule {rule}")
    for fault in faults:
        if fault.rule is not None and fault.rule not in rules:
            raise ValueError(f"fault {fault.name} names unknown catalogue rule {fault.rule}")
        if fault.rule is not None and fault.case in cases and fault.rule not in cases[fault.case]["rules"]:
            raise ValueError(f"case {fault.case} does not witness {fault.rule} for fault {fault.name}")
    return rules

def catalogue_rules(root: Path) -> list[str]:
    """Read the published catalogue IDs, from the specs' '### Id' entries."""
    with (root / "docs/architecture/domains.toml").open("rb") as stream:
        models = [model["name"] for model in tomllib.load(stream)["model"]]
    # These two catalogues belong to production room loading, outside the clean model crates.
    domains = [*models, "world-intro-rooms", "world-shaft-entry"]
    rules: set[str] = set()
    for domain in domains:
        spec = root / "docs/behaviours" / f"{domain}.md"
        fenced = False
        for line in spec.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith("```"):
                fenced = not fenced
            elif not fenced and (entry := re.fullmatch(r"###\s*([^\W\d_]\w*)", line)):
                rule = f"{spec.stem}/{entry[1]}"
                if rule in rules:
                    raise ValueError(f"{spec}: duplicate catalogue entry {rule}")
                rules.add(rule)
    return sorted(rules)

def report_rule_coverage(rules: Sequence[str], faults: Sequence[Fault],
                         logs: Sequence[str]) -> None:
    """Credit complete kill and restoration blocks from the selected run logs."""
    seconds = r" seconds=\d+\.\d{6}\n"
    input_check = rf"(?:TIMING input-check{seconds})?"
    compilation = rf"(?:TIMING compilation{seconds})?"
    for rule in rules:
        confirmed = []
        for fault in faults:
            if fault.rule != rule:
                continue
            name, case = map(re.escape, (fault.name, fault.case))
            replay = rf"TIMING replay case={case}{seconds}"
            block = re.compile(
                rf"^{replay}KILLED {name}: {case} [^\n]+\n"
                rf"(?:TIMING restoration fault={name}{seconds}{input_check}{compilation}|SELECTOR OFF fault={name}\n{input_check})"
                rf"{replay}{input_check}RESTORED PASS {case}$(?!\nRESTORED PASS )",
                re.MULTILINE,
            )
            if any(block.search(log) for log in logs):
                confirmed.append(fault.name)
        status = f"killed through {', '.join(confirmed)}" if confirmed else "unknown"
        print(f"{rule}: {status}")
    unassigned = sum(fault.rule is None for fault in faults)
    print(f"Unassigned faults: {unassigned}")

def runtime_callers(root: Path, raw: object) -> dict[str, tuple[str, ...]]:
    if not isinstance(raw, dict):
        raise ValueError("runtime_callers must be a table")
    result = {}
    for file, names in raw.items():
        if (not isinstance(file, str) or not isinstance(names, list) or not names
            or any(not isinstance(name, str) or not name for name in names)
            or len(names) != len(set(names))):
            raise ValueError("runtime_callers must list distinct function names per Rust file")
        source_path(root, file)
        result[file] = tuple(names)
    return result


def replay_dependencies(tree: Path, *, locked: bool) -> set[str]:
    command = [os.environ.get("CARGO", "cargo"), "tree", "-p", "tools", "--edges", "normal,build",
               "--prefix", "none", "--format", "{p} [{f}]"]
    if locked:
        command.append("--locked")
    output = subprocess.check_output(command, cwd=tree, text=True)
    return {line.removesuffix(" (*)") for line in output.splitlines()}


def plant_selectors(tree: Path, faults: Sequence[Fault], callers: Mapping[str, Sequence[str]]) -> None:
    from selectable_faults import Replacement, instrument_file
    edits: dict[str, list[tuple[int, Fault]]] = {}
    for ordinal, fault in enumerate(faults, 1):
        edits.setdefault(fault.file, []).append((ordinal, fault))
    for file in dict.fromkeys((*edits, *callers)):
        source = source_path(tree, file)
        original = source.read_bytes()
        changes = [
            Replacement(ordinal, require_unique_anchor(fault, original), fault.before.encode(), fault.after.encode())
            for ordinal, fault in edits.get(file, ())
        ]
        source.write_bytes(instrument_file(original, changes, callers.get(file, ())))
    normal = replay_dependencies(tree, locked=True)
    if any(package.startswith("cargo-gamma-rt v") for package in normal):
        raise ValueError("normal replay already links the fault selector")
    version = "0.8.0"
    for crate in sorted({Path(file).parts[1] for file in edits}):
        with (tree / "crates" / crate / "Cargo.toml").open("a", encoding="utf-8", newline="\n") as manifest:
            manifest.write(f'\n[dependencies.cargo-gamma-rt]\nversion = "={version}"\n')
    if replay_dependencies(tree, locked=False) != normal | {f"cargo-gamma-rt v{version} []"}:
        raise ValueError("the selectable build changed another dependency or feature")


def scratch_files(tree: Path) -> set[Path]:
    return {Path(directory, name) for directory, _, names in os.walk(tree) for name in names}


Verdict = TypeVar("Verdict")


def in_order(jobs: Sequence[Callable[[list[str]], Verdict]]) -> Generator[Verdict, None, None]:
    logs: list[list[str]] = [[] for _ in jobs]
    with ThreadPoolExecutor(len(os.sched_getaffinity(0))) as pool:
        futures = [pool.submit(job, log) for job, log in zip(jobs, logs, strict=True)]
        try:
            for future, log in zip(futures, logs, strict=True):
                try:
                    result = future.result()
                finally:
                    for line in log:
                        print(line, flush=True)
                yield result
        finally:
            pool.shutdown(cancel_futures=True)


def check_baseline(tree: Path, case: str, executable: Path,
                   env: Mapping[str, str], log: list[str]) -> dict[str, Any] | None:
    clean = run_case(tree, case, executable, env=env, log=log)
    baseline = receipt(clean[1], case)
    if clean[0] or baseline is None or baseline.get("outcome") != {"status": "pass"}:
        log.append(f"BASELINE FAILED {case}")
        return None
    log.append(f"BASELINE PASS {case}")
    return baseline


def check_fault(tree: Path, ordinal: int, fault: Fault, executable: Path,
                env: Mapping[str, str], baseline: Mapping[str, Any], log: list[str]) -> bool:
    selected = {**env, "GAMMA_ACTIVE": str(ordinal)}
    faulty = detected(fault, run_case(tree, fault.case, executable, env=selected, log=log), log)
    log.append(f"SELECTOR OFF fault={fault.name}")
    restored = run_case(tree, fault.case, executable, env=env, log=log)
    return restored_pass(fault, restored, baseline, faulty, log)


def check_phase(tree: Path, frozen: FrozenInputs, phase: str) -> None:
    try:
        print(frozen.check(tree), flush=True)
    except ValueError as error:
        raise ValueError(f"{phase}: {error}") from error


def replay_selectable(tree: Path, faults: Sequence[Fault], executable: Path,
                      frozen: FrozenInputs, env: Mapping[str, str]) -> bool:
    cases = list(dict.fromkeys(fault.case for fault in faults))
    baselines = {}
    with contextlib.closing(in_order([
        functools.partial(check_baseline, tree, case, executable, env) for case in cases
    ])) as results:
        for case, baseline in zip(cases, results, strict=True):
            if baseline is None:
                break
            baselines[case] = baseline
    check_phase(tree, frozen, "baseline phase")
    if len(baselines) != len(cases):
        return False
    verdicts = list(in_order([
        functools.partial(check_fault, tree, ordinal, fault, executable, env, baselines[fault.case])
        for ordinal, fault in enumerate(faults, 1)
    ]))
    check_phase(tree, frozen, "fault phase")
    return all(verdicts)


def run_selectable(root: Path, inputs: FrozenInputs, faults: Sequence[Fault],
                   callers: Mapping[str, Sequence[str]]) -> int:
    started = time.perf_counter()
    env = dict(os.environ)
    env.pop("GAMMA_ACTIVE", None)
    env.pop("GAMMA_REACH", None)
    for name in ("SM_ROM", "SM_CACHE", "SM_AUDIO_NATIVE"):
        if value := env.get(name):
            env[name] = str((root / value).resolve())
    target = root / "target"
    target.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=target, prefix="selectable-faults-") as directory:
        scratch = Path(directory)
        tree = scratch / "source"
        inputs.copy_to(root, tree)
        plant_selectors(tree, faults, callers)
        frozen = FrozenInputs.capture(tree, scratch_files)
        executable = build_executable(tree, target=scratch / "build")
        print(frozen.check(tree), flush=True)
        success = replay_selectable(tree, faults, executable, frozen, env)
        print(inputs.check(root), flush=True)
    print(f"TIMING selectable faults={len(faults)} seconds={time.perf_counter() - started:.6f}", flush=True)
    return 0 if success else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check replay fault detection and restoration.", allow_abbrev=False)
    commands = parser.add_subparsers(dest="command")
    commands.add_parser("selectable", help="run the complete manifest in one selectable build")
    commands.add_parser("validate", help="validate the manifest without building or running replay")
    coverage = commands.add_parser(
        "rule-coverage", help="report every catalogue rule using logs from reviewed runs of this roster")
    coverage.add_argument("logs", nargs="*", type=Path, metavar="LOG")
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[1]
    manifest = root / "tools/mutants.toml"
    raw_manifest = manifest.read_bytes()
    data = tomllib.loads(raw_manifest.decode())
    roster = tuple(Fault(**row) for row in data["fault"])
    if not roster or any(not fault.name for fault in roster) or len({fault.name for fault in roster}) != len(roster):
        raise ValueError("fault names must be nonempty and unique")
    rules = validate_coverage(root, roster)
    callers = runtime_callers(root, data.get("runtime_callers", {}))
    validate_anchors(root, roster)
    if args.command == "rule-coverage":
        report_rule_coverage(rules, roster, [log.read_text(encoding="utf-8") for log in args.logs])
        return 0
    if args.command == "validate":
        return 0
    inputs = FrozenInputs.capture(root)
    if inputs.files.get(manifest) != raw_manifest:
        raise ValueError("fault manifest changed during validation")
    return run_selectable(root, inputs, roster, callers)

if __name__ == "__main__":
    sys.exit(main())
