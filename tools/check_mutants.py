import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

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
    cut: int
    field: str
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
        self.check(root)

    def check(self, root: Path) -> None:
        started = time.perf_counter()
        if (self.listing or input_paths)(root) != self.files.keys() or any(
            input_bytes(path) != original for path, original in self.files.items()
        ):
            raise ValueError("worktree inputs changed; coordinator must reconcile")
        print(f"TIMING input-check seconds={time.perf_counter() - started:.6f}", flush=True)

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

def run_case(root: Path, case: str, executable: Path, *, env: Mapping[str, str]) -> tuple[int, str]:
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
    print(f"TIMING replay case={case} seconds={time.perf_counter() - started:.6f}", flush=True)
    return result.returncode, result.stdout if result.returncode == 0 else result.stderr

def played_receipt(output: str, case: str) -> dict[str, Any] | None:
    """The run's one report line; its stop is the Debug form of the runner's `Stop`."""
    _, row, start, _ = case.split()
    lines = [line for line in output.splitlines()
             if line.startswith(f"PLAYED_ALONE row={row} start={start} ")]
    if len(lines) != 1:
        return None
    stop = lines[0].partition(" stop=")[2]
    outcome: dict[str, Any] = {"status": "other", "stop": stop.partition(" seed_held=")[0]}
    if stop.startswith("End "):
        outcome = {"status": "pass"}
    elif found := re.match(r'At\(Difference \{ cut: (\d+), field: "([^"]+)" \}\) ', stop):
        outcome = {"status": "difference", "cut": int(found[1]), "field": found[2]}
    elif found := re.match(r'At\(DataFailure \{ cut: Some\((\d+)\), source: "([^"]+)" \}\) ', stop):
        outcome = {"status": "data-failure", "cut": int(found[1]), "field": found[2]}
    return {"id": case, "report": lines[0], "outcome": outcome}

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

def detected(fault: Fault, result: tuple[int, str]) -> dict[str, Any] | None:
    code, output = result
    observed = receipt(output, fault.case)
    played = fault.case.startswith(PLAYED)
    stops = ("difference", "data-failure") if played else ("difference",)
    if code == (0 if played else 1) and observed is not None and any(
        observed.get("outcome") == {"status": status, "cut": fault.cut, "field": fault.field} for status in stops
    ):
        print(f"KILLED {fault.name}: {fault.case} cut{fault.cut} {fault.field}", flush=True)
        return observed
    print(f"INVALID OR SURVIVED {fault.name}: {fault.case} expected cut{fault.cut} {fault.field} exit={code}", flush=True)
    return None


def receipt_digest(observed: Mapping[str, Any]) -> str:
    return hashlib.sha256(json.dumps(observed, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def restored_pass(fault: Fault, result: tuple[int, str], baseline: Mapping[str, Any],
                  faulty: Mapping[str, Any] | None) -> bool:
    restored = receipt(result[1], fault.case)
    if result[0] or restored != baseline or restored is None or restored.get("outcome") != {"status": "pass"}:
        print(f"RESTORED BASELINE FAILED {fault.case}", flush=True)
        return False
    print(f"RESTORED PASS {fault.case}", flush=True)
    if faulty is None:
        return False
    print(f"RECEIPTS {fault.name} {receipt_digest(baseline)} {receipt_digest(faulty)} {receipt_digest(restored)}", flush=True)
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
            _, _, start, end = fault.case.split()
            if fault.responsibility != "played" or not int(start) < fault.cut <= int(end):
                raise ValueError("a played fault must stop its run at a cut it plays")
        elif fault.case not in supported or fault.responsibility not in cases[fault.case]["responsibilities"]:
            raise ValueError("fault must witness its responsibility in a reviewed Pass case")
        source_path(root, fault.file)
    if {fault.responsibility for fault in faults} != required:
        raise ValueError("every supported replay responsibility needs a production fault")
    rules = catalogue_rules(root)
    for fault in faults:
        if fault.rule is not None and fault.rule not in rules:
            raise ValueError(f"fault {fault.name} names unknown catalogue rule {fault.rule}")
        if fault.rule is not None and fault.case in cases and fault.rule not in cases[fault.case]["rules"]:
            raise ValueError(f"case {fault.case} does not witness {fault.rule} for fault {fault.name}")
    return rules

def catalogue_rules(root: Path) -> list[str]:
    """Read the published catalogue IDs, using spec_units' '### Id' entry convention."""
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
                         logs: Sequence[str]) -> list[str]:
    """Credit complete kill and restoration blocks from the selected run logs."""
    seconds = r" seconds=\d+\.\d{6}\n"
    input_check = rf"(?:TIMING input-check{seconds})?"
    compilation = rf"(?:TIMING compilation{seconds})?"
    detections: list[str] = []
    for rule in rules:
        confirmed = []
        for fault in faults:
            if fault.rule != rule:
                continue
            name, case, field = map(re.escape, (fault.name, fault.case, fault.field))
            replay = rf"TIMING replay case={case}{seconds}"
            block = re.compile(
                rf"^{replay}KILLED {name}: {case} cut{fault.cut} {field}\n"
                rf"(?:TIMING restoration fault={name}{seconds}{input_check}{compilation}|SELECTOR OFF fault={name}\n{input_check})"
                rf"{replay}{input_check}RESTORED PASS {case}$(?!\nRESTORED PASS )",
                re.MULTILINE,
            )
            if any(block.search(log) for log in logs):
                confirmed.append(fault.name)
        detections.extend(confirmed)
        status = f"killed through {', '.join(confirmed)}" if confirmed else "unknown"
        print(f"{rule}: {status}")
    unassigned = sum(fault.rule is None for fault in faults)
    print(f"Unassigned faults: {unassigned}")
    return detections

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
    return {path for path in tree.rglob("*") if path.is_file() or path.is_symlink()}


def replay_selectable(tree: Path, faults: Sequence[Fault], executable: Path,
                      frozen: FrozenInputs, env: Mapping[str, str]) -> bool:
    baselines = {}
    for case in dict.fromkeys(fault.case for fault in faults):
        clean = run_case(tree, case, executable, env=env)
        frozen.check(tree)
        baseline = receipt(clean[1], case)
        if clean[0] or baseline is None or baseline.get("outcome") != {"status": "pass"}:
            print(f"BASELINE FAILED {case}", flush=True)
            return False
        baselines[case] = baseline
        print(f"BASELINE PASS {case}", flush=True)
    success = True
    for ordinal, fault in enumerate(faults, 1):
        selected = {**env, "GAMMA_ACTIVE": str(ordinal)}
        faulty = detected(fault, run_case(tree, fault.case, executable, env=selected))
        print(f"SELECTOR OFF fault={fault.name}", flush=True)
        frozen.check(tree)
        restored = run_case(tree, fault.case, executable, env=env)
        frozen.check(tree)
        if not restored_pass(fault, restored, baselines[fault.case], faulty):
            success = False
    return success


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
        frozen.check(tree)
        success = replay_selectable(tree, faults, executable, frozen, env)
        inputs.check(root)
    print(f"TIMING selectable faults={len(faults)} seconds={time.perf_counter() - started:.6f}", flush=True)
    return 0 if success else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check replay fault detection and restoration.", allow_abbrev=False)
    commands = parser.add_subparsers(dest="command")
    commands.add_parser("selectable", help="run the complete manifest in one selectable build")
    commands.add_parser("validate", help="validate the manifest without building or running replay")
    coverage = commands.add_parser(
        "rule-coverage", help="report every catalogue rule using logs from reviewed runs of this roster")
    coverage.add_argument("--write", type=Path, metavar="RECEIPT", help="save detections for tools report")
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
        detected = report_rule_coverage(rules, roster, [log.read_text(encoding="utf-8") for log in args.logs])
        if args.write is not None:
            names = json.dumps(detected, ensure_ascii=False).replace("\x7f", "\\u007f")
            args.write.write_text(
                f'format = 1\nmanifest_sha256 = "{hashlib.sha256(raw_manifest).hexdigest()}"\n'
                f'detected = {names}\n', encoding="utf-8", newline="\n")
        return 0
    if args.command == "validate":
        return 0
    inputs = FrozenInputs.capture(root)
    if inputs.files.get(manifest) != raw_manifest:
        raise ValueError("fault manifest changed during validation")
    return run_selectable(root, inputs, roster, callers)

if __name__ == "__main__":
    sys.exit(main())
