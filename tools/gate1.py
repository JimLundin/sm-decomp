#!/usr/bin/env python3
"""Gate 1 evidence: the slice's headless replay against the oracle, row by row.

For each row: restore its bundle, record the oracle's digests where the lock has
none, prepare its start with the oracle-side importer, replay it with sm_replay
(no CPU emulator or native code), and compare each machine stream.

Prints names and verdicts only. The ROM and the row data stay on the runner.
"""
import argparse
import os
import pathlib
import subprocess
import sys
import tempfile
import tomllib
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import scenario  # noqa: E402

MACHINE = ("wram", "sram", "vram", "cgram", "oam", "ppu", "video", "apu")
RELEASES = "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/"
VERIFY = [sys.executable, "tools/verify_dll.py", "rom/sm.smc", "--corpus", "scenarios",
          "--no-build", "--direct"]


def run(command: list[str], **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(command, capture_output=True, text=True, **kwargs)


def restore(row: str, pins: dict, bundles: dict, work: pathlib.Path) -> None:
    asset = pins[row]["bundle"]
    path = work / asset
    if not path.exists():
        urllib.request.urlretrieve(RELEASES + asset, path)
    result = run([sys.executable, "tools/ci_seed_data.py", "restore", str(path), "--sha256",
                  bundles[asset], "--runtime-only", "--segment", row])
    if result.returncode:
        raise RuntimeError(f"{row}: restore failed: {result.stderr[-300:]}")


def verifier_entry(row: str, lock: pathlib.Path, mode: str) -> int:
    result = run(VERIFY + ["--segment", row, "--mode", mode, "--lock", str(lock)])
    print(f"{row}: verifier {mode}: exit {result.returncode}: "
          + " | ".join(line for line in result.stdout.splitlines()
                       if line.startswith(row) or line.startswith("FAIL")), flush=True)
    return result.returncode


def check(row: str, locked: dict, run_entry: dict) -> list[str]:
    differing = [name for name in MACHINE if locked["digests"][name] != run_entry["digests"][name]]
    if locked["ticks"] != run_entry["ticks"]:
        differing.append("ticks")
    return differing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rows", nargs="+")
    args = parser.parse_args()
    root = pathlib.Path(".")
    lock = scenario.read_lock(root)
    failed = 0
    work = pathlib.Path(tempfile.mkdtemp(prefix="gate1-"))
    for row in args.rows:
        restore(row, lock.pins, lock.bundles, work)
        entries = pathlib.Path("scenarios/lock.toml")
        locked = lock.entries.get(row)
        if locked is None:
            # A row the lock has no entry for: record the oracle's now, on this runner.
            fresh = work / f"{row}-lock.toml"
            fresh.write_bytes(entries.read_bytes())
            if verifier_entry(row, fresh, "lock"):
                failed += 1
                continue
            locked = scenario.read_lock(root, fresh).entries.get(row)
            entries = fresh
        start = work / f"{row}.start"
        environment = dict(os.environ, SM_EXPORT_START=str(start))
        run(VERIFY + ["--segment", row, "--mode", "check-lock", "--lock", str(entries)],
            env=environment)
        if not start.exists():
            print(f"{row}: no prepared start", flush=True)
            failed += 1
            continue
        verifier_entry(row, entries, "check-lock")
        inputs = pathlib.Path("movies") / lock.pins[row]["inputs"]
        out = work / f"{row}-entry.toml"
        content, sealed = work / f"{row}.content", work / f"{row}.sealed"
        prepared = run(["target/release/sm_replay", "prepare", "rom/sm.smc", str(start),
                        str(content), str(sealed)])
        if prepared.returncode:
            print(f"{row}: prepare failed: {prepared.stderr[-300:]}", flush=True)
            failed += 1
            continue
        replay = run(["target/release/sm_replay", "gate", str(content), str(sealed), str(inputs),
                      str(locked["ticks"]), str(out)])
        print(f"{row}: replay exit {replay.returncode}\n{replay.stdout.strip()[-1200:]}\n"
              f"{replay.stderr.strip()[-600:]}", flush=True)
        if replay.returncode:
            failed += 1
            continue
        with out.open("rb") as file:
            entry = tomllib.load(file)
        differing = check(row, locked, entry)
        print(f"{row}: {'MATCHES the oracle on ' + str(len(MACHINE)) + ' streams' if not differing else 'DIFFERS in ' + ', '.join(differing)}"
              f" ({entry['ticks']} ticks)", flush=True)
        failed += bool(differing)
    print(f"Gate 1 rows: {len(args.rows) - failed}/{len(args.rows)} match", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
