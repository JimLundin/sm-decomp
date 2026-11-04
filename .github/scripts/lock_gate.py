#!/usr/bin/env python3
"""The engine-only gate against the oracle digest lock.

Every pull request and push runs the engine alone over every scenario that
scenarios/lock.toml locks and compares its running digests with the faithful
oracle's (docs/SCENARIO_HARNESS.md, Decision 4; ci.yml Verify). The planner
(`sm_pipeline plan`) first checks the lock as `tools/scenario.py lock --check`
does: it fails on a stale lock (an entry whose start, inputs or budget no longer
match its scenario, an entry for no scenario, or a lockable scenario neither
locked nor `unlocked`); it never re-records.

  record-oracle    re-record SEGMENTS privately and compare their committed entries

Recording (faithful oracle; docs/SCENARIO_HARNESS.md, "Oracle digest lock"):
  python tools/scenario.py lock --record
"""
import hashlib, os, pathlib, subprocess, sys, tempfile
from collections.abc import Mapping
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import scenario  # noqa: E402


def compare_oracle(locked: Mapping[str, dict[str, Any]], recorded: Mapping[str, dict[str, Any]],
                   selected: list[str]) -> list[str]:
    """Exact entries include identity, format, ticks, every locked stream and every mark.

    A recorded stream the locked entry lacks (a new stream, such as the Session
    streams) is not a failure: `new_streams` names it for the lock to take up."""
    failures: list[str] = []
    if set(recorded) != set(selected):
        failures.append("recorded scenario ids differ from the selected ids")
    for name in selected:
        expected, actual = locked.get(name), recorded.get(name)
        if expected is None or actual is None:
            failures.append(f"{name}: missing oracle entry")
            continue
        for field in sorted(expected.keys() | actual.keys()):
            if expected.get(field) != actual.get(field):
                if field in ("digests", "marks"):
                    before, after = expected.get(field, {}), actual.get(field, {})
                    keys = before.keys() if field == "digests" else before.keys() | after.keys()
                    changed = sorted(key for key in keys if before.get(key) != after.get(key))
                    if not changed:
                        continue
                    failures.append(f"{name}: {field} differ in {', '.join(changed)}")
                else:
                    failures.append(f"{name}: {field} differs")
    return failures


def new_streams(locked: Mapping[str, dict[str, Any]], recorded: Mapping[str, dict[str, Any]],
                selected: list[str]) -> list[str]:
    """`STREAM <id> <stream> <digest>` for each recorded stream the locked entry lacks:
    names and hashes only, which `tools/scenario.py lock --add-streams` takes up."""
    lines: list[str] = []
    for name in selected:
        before = locked.get(name, {}).get("digests", {})
        after = recorded.get(name, {}).get("digests", {})
        lines.extend(f"STREAM {name} {stream} {after[stream]}"
                     for stream in sorted(after.keys() - before.keys()))
    return lines


def record_oracle() -> None:
    """Re-record into a private temporary lock; never write the committed lock."""
    selected = os.environ["SEGMENTS"].split()
    locked = scenario.check(ROOT)
    if not selected or len(selected) != len(set(selected)) or set(selected) - locked:
        raise SystemExit("oracle sample must name distinct already locked scenarios")
    committed = ROOT / scenario.LOCK
    before = hashlib.sha256(committed.read_bytes()).digest()
    expected = scenario.read_lock(ROOT).entries
    with tempfile.TemporaryDirectory(prefix="oracle-sample-") as directory:
        recorded = pathlib.Path(directory, "lock.toml")
        # Raw verifier output stays private on the runner, including on failure.
        with pathlib.Path(directory, "replay.log").open("w") as log:
            result = subprocess.run(
                [sys.executable, str(ROOT / "tools/verify_dll.py"), str(ROOT / "rom/sm.smc"),
                 "--corpus", str(ROOT / "scenarios"), "--no-build", "--direct",
                 "--mode", "lock", "--oracle", "faithful", "--lock", str(recorded),
                 *[part for name in selected for part in ("--segment", name)]],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        if hashlib.sha256(committed.read_bytes()).digest() != before:
            raise SystemExit("FAIL: committed oracle lock changed during recording")
        if result.returncode:
            raise SystemExit(f"FAIL: faithful oracle recording exited {result.returncode}")
        entries = scenario.read_lock(ROOT, recorded).entries
        failures = compare_oracle(expected, entries, selected)
        added = [] if failures else new_streams(expected, entries, selected)
    for line in added:
        print(line, flush=True)
    for failure in failures:
        print(f"FAIL {failure}", flush=True)
    if failures:
        raise SystemExit(1)
    for name in selected:
        entry = expected[name]
        print(f"PASS {name}: oracle lock matches; {entry['ticks']} ticks; "
              f"domains {', '.join(sorted(entry['digests']))}; hash {entry['combined']}", flush=True)


if __name__ == "__main__":
    {"record-oracle": record_oracle}[sys.argv[1]]()
