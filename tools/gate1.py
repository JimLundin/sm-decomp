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
    # A lock records the faithful oracle, the real game (REVIEW condition 8(i)).
    oracle = ["--oracle", "faithful"] if mode == "lock" else []
    result = run(VERIFY + ["--segment", row, "--mode", mode, "--lock", str(lock)] + oracle)
    print(f"{row}: verifier {mode}: exit {result.returncode}: "
          + " | ".join(line for line in result.stdout.splitlines()
                       if line.startswith(row) or line.startswith("FAIL")), flush=True)
    if result.returncode:
        # The verifier reports its own errors on stderr; names and reasons only.
        tail = [line for line in result.stderr.splitlines() if line.startswith(("verify:", "FAIL", "error", "usage:")) or ": error:" in line]
        print(f"{row}: verifier {mode} stderr: " + " | ".join(tail[-5:]), flush=True)
    return result.returncode


def ranges(left: bytes, right: bytes) -> str:
    """The first differing byte ranges of two dumps, named by region."""
    regions = (("wram", 0, 0x20000), ("vram", 0x20000, 0x30000), ("cgram", 0x30000, 0x30200),
               ("oam", 0x30200, 0x30400))
    parts = []
    for name, first, end in regions:
        found: list[list[int]] = []
        for at in range(first, min(end, len(left), len(right))):
            if left[at] != right[at]:
                if found and at - found[-1][1] <= 2:
                    found[-1][1] = at
                else:
                    found.append([at, at])
        total = sum(end - start + 1 for start, end in found)
        shown = ", ".join(f"{start - first:05x}-{end - first:05x}" for start, end in found[:40])
        parts.append(f"{name}: {total} bytes in {len(found)} ranges: {shown}")
    return "\n".join(parts)


def diagnose(row: str, entries: pathlib.Path, content: pathlib.Path, sealed: pathlib.Path,
             inputs: pathlib.Path, ticks: int, work: pathlib.Path) -> None:
    """Where the slice and the verifier's own engine run part."""
    logs = {}
    for side in ("verifier", "slice"):
        log = work / f"{row}-{side}.log"
        log.unlink(missing_ok=True)
        logs[side] = log
    def verifier(extra: dict[str, str]) -> None:
        run(VERIFY + ["--segment", row, "--mode", "check-lock", "--lock", str(entries)],
            env=dict(os.environ, **extra))
    def slice_run(extra: dict[str, str]) -> None:
        run(["target/release/sm_replay", "one", str(content), str(sealed), str(inputs), str(ticks)],
            env=dict(os.environ, **extra))
    verifier({"SM_DIGEST_LOG": str(logs["verifier"])})
    slice_run({"SM_DIGEST_LOG": str(logs["slice"])})
    left = logs["verifier"].read_text().splitlines()
    right = logs["slice"].read_text().splitlines()
    print(f"{row}: logs {len(left)} and {len(right)} ticks", flush=True)
    names = ("wram", "vram", "cgram", "oam", "high oam")
    first = {}
    for a, b in zip(left, right):
        for name, x, y in zip(names, a.split()[1:], b.split()[1:]):
            if x != y and name not in first:
                first[name] = int(a.split()[0])
    print(f"{row}: first differing tick per region: {first}", flush=True)
    if not first:
        return
    tick = min(first.values())
    dumps = {}
    for side, call in (("verifier", verifier), ("slice", slice_run)):
        path = work / f"{row}-{side}.dump"
        call({"SM_DUMP_TICK": str(tick), "SM_DUMP_OUT": str(path)})
        dumps[side] = path.read_bytes()
    print(f"{row}: at tick {tick}:\n{ranges(dumps['verifier'], dumps['slice'])}", flush=True)


def windowed(row: str, content: pathlib.Path, sealed: pathlib.Path, inputs: pathlib.Path,
             ticks: int, work: pathlib.Path) -> bool:
    """The windowed app on the same row, headless, against sm_replay's run of it,
    compared Tick by Tick on every machine stream's digest."""
    logs = {side: work / f"{row}-{side}-digests.txt" for side in ("window", "replay")}
    for path in logs.values():
        path.unlink(missing_ok=True)
    command = ["xvfb-run", "-a", "target/release/sm_slice", str(content), str(sealed),
               "--inputs", str(inputs), "--ticks", str(ticks), "--digests-out", str(logs["window"]),
               "--save", str(work / f"{row}-window.srm")]
    environment = dict(os.environ, SDL_AUDIODRIVER="dummy")
    shown = run(command, env=environment)
    print(f"{row}: sm_slice exit {shown.returncode}: {shown.stdout.strip()[-400:]}"
          f" {shown.stderr.strip()[-400:]}", flush=True)
    if shown.returncode:
        return False
    run(["target/release/sm_replay", "one", str(content), str(sealed), str(inputs), str(ticks),
         str(logs["replay"])])
    left, right = (logs[side].read_text().splitlines() if logs[side].exists() else []
                   for side in ("window", "replay"))
    first = next((a.split()[0] for a, b in zip(left, right) if a != b), None)
    if first is None and len(left) == len(right) == ticks:
        print(f"{row}: sm_slice MATCHES sm_replay on all {len(MACHINE)} machine streams at every Tick"
              f" ({ticks} ticks)", flush=True)
        return True
    print(f"{row}: sm_slice DIFFERS from sm_replay: digest logs {len(left)} and {len(right)}"
          f" of {ticks} ticks; first differing tick: {first}", flush=True)
    return False


def check(row: str, locked: dict, run_entry: dict) -> list[str]:
    differing = [name for name in MACHINE if locked["digests"][name] != run_entry["digests"][name]]
    if locked["ticks"] != run_entry["ticks"]:
        differing.append("ticks")
    return differing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rows", nargs="+", help="row ids, or `all` for every locked row")
    parser.add_argument("--shard", default="1/1", metavar="I/N",
                        help="run only the I-th of N equal parts of the rows")
    parser.add_argument("--slice", action="append", default=[], metavar="ROW",
                        help="also run the windowed sm_slice on ROW against sm_replay")
    args = parser.parse_args()
    root = pathlib.Path(".")
    lock = scenario.read_lock(root)
    failed = diagnosed = window_failed = 0
    work = pathlib.Path(tempfile.mkdtemp(prefix="gate1-"))
    rows = [row for row in args.rows if row != "all"]
    if "all" in args.rows:
        # Every locked corpus row; the room-chain entries are left to the corpus jobs.
        with (root / "scenarios/corpus.toml").open("rb") as file:
            corpus = [entry["id"] for entry in tomllib.load(file)["scenario"]]
        rows += [row for row in corpus if row in lock.entries and row not in rows]
    index, count = (int(part) for part in args.shard.split("/"))
    rows = rows[index - 1::count]
    print(f"Gate 1 shard {args.shard}: {len(rows)} rows", flush=True)
    for row in rows:
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
        if row in args.slice and not windowed(row, content, sealed, inputs, locked["ticks"], work):
            window_failed += 1
        if differing and diagnosed < 3:
            diagnosed += 1
            diagnose(row, entries, content, sealed, inputs, locked["ticks"], work)
    print(f"Gate 1 rows: {len(rows) - failed}/{len(rows)} match", flush=True)
    if args.slice:
        print(f"sm_slice rows: {len(args.slice) - window_failed}/{len(args.slice)} match sm_replay", flush=True)
    return 1 if failed or window_failed else 0


if __name__ == "__main__":
    sys.exit(main())
