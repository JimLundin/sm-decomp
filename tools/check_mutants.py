"""Recorded replay differences must reject temporary production bridge faults.

Run while the coordinator has frozen the named sources. Restore their exact bytes
after every mutation, then require a fresh Pass. Print only safe result identities.
"""
import os
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

@dataclass(frozen=True)
class Fault:
    name: str
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

def run_case(root: Path, case: str) -> tuple[int, str]:
    command = [os.environ.get("CARGO", "cargo"), "run", "--locked", "--profile", "verify",
               "-p", "tools", "--bin", "tools", "--", "replay", "run",
               "scenarios/replay/index.toml", "--case", case]
    for flag, variable in [("--cache", "SM_CACHE"), ("--rom", "SM_ROM"), ("--native", "SM_AUDIO_NATIVE")]:
        if value := os.environ.get(variable):
            command.extend([flag, value])
    result = subprocess.run(command, cwd=root, capture_output=True, text=True,
                            encoding="utf-8", errors="replace")
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

def check_fault(root: Path, fault: Fault) -> bool:
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
        if path.read_bytes() != mutated:
            raise ValueError("fault source changed concurrently; coordinator must reconcile")
        path.write_bytes(original)
    if path.read_bytes() != original:
        raise ValueError("source was not restored byte-for-byte")
    if not passed(run_case(root, fault.case), fault.case):
        print(f"RESTORED BASELINE FAILED {fault.case}", flush=True)
        return False
    print(f"RESTORED PASS {fault.case}", flush=True)
    return killed

def main() -> int:
    root = Path(__file__).resolve().parents[1]
    with (root / "tools/mutants.toml").open("rb") as stream:
        faults = [Fault(**row) for row in tomllib.load(stream)["fault"]]
    if not faults or len({fault.name for fault in faults}) != len(faults):
        raise ValueError("fault names must be nonempty and unique")
    for case in dict.fromkeys(fault.case for fault in faults):
        if not passed(run_case(root, case), case):
            print(f"BASELINE FAILED {case}", flush=True)
            return 1
    return 0 if all(check_fault(root, fault) for fault in faults) else 1

if __name__ == "__main__":
    sys.exit(main())
