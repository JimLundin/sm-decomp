"""Recorded replay differences must reject temporary production bridge faults.

Run while the coordinator has frozen the worktree, Cargo environment and external
ROM/cache/native inputs. Restore exact source bytes after every mutation, then
require a fresh Pass from an immutable vanilla executable. Print safe identities.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

WORKER_COUNT = 4

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

def source_path(root: Path, source: str) -> Path:
    relative = Path(source)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("fault source must be relative without traversal")
    path = (root / relative).resolve()
    production = (root / "crates/game/src/production").resolve()
    if path != (root / "crates/game/src/production.rs").resolve() and not path.is_relative_to(production):
        raise ValueError("fault source is outside production adapters")
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

    @classmethod
    def capture(cls, root: Path) -> "FrozenInputs":
        return cls({path: input_bytes(path) for path in input_paths(root)})

    def check(self, root: Path) -> None:
        started = time.perf_counter()
        if input_paths(root) != self.files.keys() or any(
            input_bytes(path) != original for path, original in self.files.items()
        ):
            raise ValueError("worktree inputs changed; coordinator must reconcile")
        print(f"TIMING input-check seconds={time.perf_counter() - started:.6f}", flush=True)

def build_executable(root: Path) -> Path:
    """Use Cargo's artifact path, including custom target directories and triples."""
    started = time.perf_counter()
    result = subprocess.run([os.environ.get("CARGO", "cargo"), "build", "--locked",
                             "--profile", "verify", "-p", "tools", "--bin", "tools",
                             "--message-format=json"], cwd=root, capture_output=True,
                            text=True, encoding="utf-8", errors="replace")
    print(f"TIMING compilation seconds={time.perf_counter() - started:.6f}", flush=True)
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

def run_case(root: Path, case: str, executable: Path | None = None) -> tuple[int, str]:
    if executable is None:
        try:
            executable = build_executable(root)
        except subprocess.CalledProcessError as error:
            return error.returncode, error.stderr
    command = [str(executable), "replay", "run",
               "scenarios/replay/index.toml", "--case", case]
    for flag, variable in [("--cache", "SM_CACHE"), ("--rom", "SM_ROM"), ("--native", "SM_AUDIO_NATIVE")]:
        if value := os.environ.get(variable):
            command.extend([flag, value])
    started = time.perf_counter()
    result = subprocess.run(command, cwd=root, capture_output=True, text=True,
                            encoding="utf-8", errors="replace")
    print(f"TIMING replay case={case} seconds={time.perf_counter() - started:.6f}", flush=True)
    return result.returncode, result.stdout if result.returncode == 0 else result.stderr

def receipt(output: str, case: str) -> dict[str, Any] | None:
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

def passed(result: tuple[int, str], case: str) -> bool:
    code, output = result
    observed = receipt(output, case)
    return code == 0 and observed is not None and observed.get("outcome") == {"status": "pass"}

def check_fault(root: Path, fault: Fault, baseline: dict[str, Any],
                vanilla: Path | None = None, inputs: FrozenInputs | None = None) -> bool:
    if inputs is not None:
        inputs.check(root)
    path = source_path(root, fault.file)
    original = path.read_bytes()
    before, after = fault.before.encode(), fault.after.encode()
    if not before or before == after or original.count(before) != 1:
        raise ValueError("fault must replace one unique changed fragment")
    mutated = original.replace(before, after, 1)
    try:
        path.write_bytes(mutated)
        code, output = run_case(root, fault.case)
        observed = receipt(output, fault.case)
        killed = code == 1 and observed is not None and observed.get("outcome") == {
            "status": "difference", "cut": fault.cut, "field": fault.field}
        print(f"{'KILLED' if killed else 'INVALID OR SURVIVED'} {fault.name}: {fault.case} cut{fault.cut} {fault.field}", flush=True)
    finally:
        started = time.perf_counter()
        if path.read_bytes() != mutated:
            raise ValueError("fault source changed concurrently; coordinator must reconcile")
        path.write_bytes(original)
        if path.read_bytes() != original:
            raise ValueError("source was not restored byte-for-byte")
        print(f"TIMING restoration fault={fault.name} seconds={time.perf_counter() - started:.6f}", flush=True)
    if inputs is not None:
        inputs.check(root)
    restored = run_case(root, fault.case, vanilla)
    if inputs is not None:
        inputs.check(root)
    if not passed(restored, fault.case) or receipt(restored[1], fault.case) != baseline:
        print(f"RESTORED BASELINE FAILED {fault.case}", flush=True)
        return False
    print(f"RESTORED PASS {fault.case}", flush=True)
    return killed

def validate_coverage(root: Path, faults: Sequence[Fault]) -> None:
    """Every responsibility with a reviewed Pass needs its own production fault."""
    with (root / "scenarios/replay/index.toml").open("rb") as stream:
        cases = {case["id"]: case for case in tomllib.load(stream)["case"]}
    with (root / "scenarios/replay/baseline.toml").open("rb") as stream:
        supported = {case["id"] for case in tomllib.load(stream)["case"]
                     if case["expected"] == {"status": "pass"}}
    required = {owner for case in supported for owner in cases[case]["responsibilities"]}
    for fault in faults:
        if fault.case not in supported or fault.responsibility not in cases[fault.case]["responsibilities"]:
            raise ValueError("fault must witness its responsibility in a reviewed Pass case")
        source_path(root, fault.file)
    if {fault.responsibility for fault in faults} != required:
        raise ValueError("every supported replay responsibility needs a production fault")

def partition_faults(faults: tuple[Fault, ...]) -> tuple[tuple[Fault, ...], ...]:
    """Assign each manifest row to one worker in manifest order."""
    worker_count = min(WORKER_COUNT, len(faults))
    if worker_count == 0:
        raise ValueError("cannot partition an empty fault manifest")
    return tuple(faults[worker::worker_count] for worker in range(worker_count))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    commands = parser.add_subparsers(dest="command")
    commands.add_parser("plan", help="print the CI worker matrix")
    worker = commands.add_parser("worker", help="run one partition of the fault manifest")
    worker.add_argument("index", choices=tuple(map(str, range(WORKER_COUNT))))
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[1]
    manifest = root / "tools/mutants.toml"
    raw_manifest = manifest.read_bytes()
    roster = tuple(Fault(**row) for row in tomllib.loads(raw_manifest.decode())["fault"])
    if not roster or any(not fault.name for fault in roster) or len({fault.name for fault in roster}) != len(roster):
        raise ValueError("fault names must be nonempty and unique")
    validate_coverage(root, roster)
    if args.command:
        partitions = partition_faults(roster)
        if args.command == "plan":
            print(json.dumps({"worker": list(range(len(partitions)))}))
            return 0
        index = int(args.index)
        if index >= len(partitions):
            parser.error(f"worker {index} is absent from this fault manifest")
        faults = partitions[index]
    else:
        faults = roster
    inputs = FrozenInputs.capture(root)
    if inputs.files.get(manifest) != raw_manifest:
        raise ValueError("fault manifest changed while planning")
    # Cargo's output is overwritten by mutant builds. The saved binary is private;
    # every baseline/restored receipt still comes from a newly executed process.
    with tempfile.TemporaryDirectory(prefix="sm-mutants-") as directory:
        try:
            built = build_executable(root)
        except subprocess.CalledProcessError:
            print("BASELINE BUILD FAILED", flush=True)
            return 1
        inputs.check(root)
        vanilla = Path(directory) / built.name
        shutil.copy2(built, vanilla)
        baselines = {}
        for case in dict.fromkeys(fault.case for fault in faults):
            inputs.check(root)
            result = run_case(root, case, vanilla)
            inputs.check(root)
            observed = receipt(result[1], case)
            if not passed(result, case) or observed is None:
                print(f"BASELINE FAILED {case}", flush=True)
                return 1
            baselines[case] = observed
            print(f"BASELINE PASS {case}", flush=True)
        results = [check_fault(root, fault, baselines[fault.case], vanilla, inputs) for fault in faults[:-1]]
        # Direct replay scripts use Cargo's normal artifact. Build the last restored
        # replay through Cargo so both that artifact and its fingerprints are vanilla.
        last = faults[-1]
        results.append(check_fault(root, last, baselines[last.case], inputs=inputs))
    return 0 if all(results) else 1

if __name__ == "__main__":
    sys.exit(main())
