"""Require the recorded floor and ratchet checks to reject each planted model fault.

Run with SM_CACHE and SM_ROM set, as for the harness tests. Faults are applied one
at a time and restored even when a test, build or source check fails. Only libtest
results and safe domain floor summaries are printed: the captured diagnostics may
contain oracle observations.
"""

import os
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path


@dataclass(frozen=True)
class Fault:
    name: str
    domain: str
    file: str
    before: str
    after: str
    test: str
    extra_tests: list[str] = field(default_factory=list)


def test_command(test: str) -> list[str]:
    if os.name == "nt":
        return ["bash", "tools/lane/run_tests.sh", "verify", "--", test,
                "--exact", "--nocapture"]
    return ["cargo", "test", "--locked", "--profile", "verify", "-p", "verify",
            "--lib", test, "--", "--exact", "--nocapture"]


def run_test(root: Path, test: str) -> tuple[int, str]:
    result = subprocess.run(test_command(test), cwd=root, capture_output=True,
                            text=True, encoding="utf-8", errors="replace")
    return result.returncode, result.stdout + result.stderr


def reported(output: str, test: str, status: str) -> bool:
    return re.search(r"^test " + re.escape(test) + r" \.\.\. " + status + r"$",
                     output, re.MULTILINE) is not None


def check_fault(root: Path, fault: Fault) -> bool:
    path = (root / fault.file).resolve()
    clean = (root / "crates/clean").resolve()
    if not path.is_relative_to(clean):
        raise ValueError(f"{fault.name}: source is outside the clean crates")
    original = path.read_bytes()
    before, after = fault.before.encode(), fault.after.encode()
    if before == after or not before or original.count(before) != 1:
        raise ValueError(f"{fault.name}: replacement must change one unique source fragment")
    try:
        path.write_bytes(original.replace(before, after, 1))
        killed = True
        for test in [fault.test, *fault.extra_tests]:
            code, output = run_test(root, test)
            scoped = show_scope(output, fault.domain)
            regression = floor_regressed(output, fault.domain)
            if test.startswith("harness::") and (not scoped or
                    (code != 0 and not regression)):
                code, output = 1, ""
            if code != 0 and reported(output, test, "FAILED"):
                print(f"KILLED {fault.domain} {fault.name}: {test}", flush=True)
            else:
                reason = "SURVIVED" if code == 0 and reported(output, test, "ok") else "INVALID RUN"
                print(f"{reason} {fault.domain} {fault.name}: {test}", flush=True)
                killed = False
    finally:
        path.write_bytes(original)
    if path.read_bytes() != original:
        raise ValueError(f"{fault.name}: source was not restored byte-for-byte")
    return killed


def floor_regressed(output: str, domain: str) -> bool:
    """An accounting assertion alone cannot prove a domain floor rejected the fault."""
    pattern = (r"^FLOOR (?:harness|free) " + re.escape(domain)
               + r" [A-Za-z0-9_-]+ [0-9]+ [0-9]+ FAILED$")
    failures = re.findall(pattern, output, re.MULTILINE)
    for line in failures:
        print(line, flush=True)
    return bool(failures)


def show_scope(output: str, domain: str) -> bool:
    """Print only counts for M0; a kill on another row is not its proof."""
    pattern = (r"^FLOOR (harness|free) " + re.escape(domain)
               + r" chain-any-0003-DFD7 [0-9]+ [0-9]+ (ok|FAILED)$")
    found = False
    for line in output.splitlines():
        if re.fullmatch(pattern, line):
            print(line, flush=True)
            found = True
    if not found:
        print(f"M0 {domain}: no domain floor result in this test", flush=True)
    return found



def main() -> int:
    root = Path(__file__).resolve().parents[1]
    for variable, directory in [("SM_CACHE", True), ("SM_ROM", False)]:
        value = os.environ.get(variable)
        path = Path(value) if value else None
        if path is None or not (path.is_dir() if directory else path.is_file()):
            raise ValueError(f"{variable} must name an existing {'directory' if directory else 'file'}")
    with (root / "tools/mutants.toml").open("rb") as manifest:
        faults = [Fault(**row) for row in tomllib.load(manifest)["fault"]]
    if not faults or len({fault.name for fault in faults}) != len(faults):
        raise ValueError("fault names must be nonempty and unique")
    dirty = subprocess.run(["git", "status", "--porcelain", "--", "crates/clean"],
                           cwd=root, capture_output=True, text=True, check=True).stdout
    if dirty:
        raise ValueError("clean crate changes must be committed before planting faults")
    for test in dict.fromkeys(test for fault in faults
                              for test in [fault.test, *fault.extra_tests]):
        code, output = run_test(root, test)
        if code != 0 or not reported(output, test, "ok"):
            print(f"BASELINE FAILED: {test}", flush=True)
            return 1
        print(f"BASELINE OK: {test}", flush=True)
        domains = list(dict.fromkeys(fault.domain for fault in faults
                                     if test in [fault.test, *fault.extra_tests]))
        for domain in domains:
            if not show_scope(output, domain) and test.startswith("harness::"):
                print(f"BASELINE INVALID: M0 floor missing: {test}", flush=True)
                return 1
    killed = {fault.name: check_fault(root, fault) for fault in faults}
    for domain in dict.fromkeys(fault.domain for fault in faults):
        members = [fault for fault in faults if fault.domain == domain]
        count = sum(killed[fault.name] for fault in members)
        print(f"mutants {domain}: {count}/{len(members)} killed", flush=True)
    return 0 if all(killed.values()) else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        sys.exit(f"mutant check: {error}")
