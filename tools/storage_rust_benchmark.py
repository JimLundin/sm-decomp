#!/usr/bin/env python3
"""Measure the Rust evidence crate on a lossless, matched capture scope.

Scopes contain hardware.log, fixtures/*.wram, and frame-*.bin as temporary
hardlinks. Originals are never modified. Build the release binary first.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def measure(command, stdout=None):
    start = time.perf_counter()
    result = subprocess.run(command, stdout=stdout or subprocess.PIPE,
                            stderr=subprocess.PIPE, text=stdout is None)
    if result.returncode:
        raise RuntimeError(f"{command} failed: {result.stderr[-4000:]}")
    return {"seconds": time.perf_counter() - start,
            "stdout": result.stdout if stdout is None else None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--scope-root", type=Path, required=True)
    parser.add_argument("--store", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("sources", type=Path, nargs="+")
    args = parser.parse_args()
    args.scope_root.mkdir(parents=True, exist_ok=False)
    args.store.mkdir(parents=True, exist_ok=False)
    report = {"binary": str(args.binary.resolve()), "store": str(args.store.resolve()),
              "binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(), "captures": []}
    try:
        for i, source in enumerate(args.sources):
            scope = args.scope_root / str(i)
            (scope / "fixtures").mkdir(parents=True)
            members = [source / "hardware.log", *sorted(source.glob("frame-*.bin")),
                       *sorted((source / "fixtures").glob("*.wram"))]
            for member in members:
                dest = scope / (Path("fixtures") / member.name if member.parent.name == "fixtures" else member.name)
                os.link(member, dest)
            entry = {"source": str(source), "files": len(members),
                     "logical_bytes": sum(p.stat().st_size for p in members)}
            print(f"ingesting {source}", flush=True)
            entry["ingest"] = measure([str(args.binary), "ingest", str(args.store), str(scope)])
            roots = json.loads(entry["ingest"]["stdout"])
            entry.update(roots)
            print(f"verifying {source}", flush=True)
            entry["verify"] = measure([str(args.binary), "verify", str(args.store), roots["run"], str(scope)])
            entry["stats"] = json.loads(measure([str(args.binary), "stats", str(args.store)])["stdout"])
            # Cold-process reads include loading all pack indexes. Discard only stdout.
            with open(os.devnull, "wb") as sink:
                entry["frame_60_cold"] = measure([str(args.binary), "frame", str(args.store), roots["run"], "hardware.log", "60"], sink)
                entry["snapshot_cold"] = measure([str(args.binary), "cat", str(args.store), roots["run"], sorted(source.glob("frame-*.bin"))[0].name], sink)
            report["captures"].append(entry)
            args.report.write_text(json.dumps(report, indent=2) + "\n")
    finally:
        for path in sorted(args.scope_root.rglob("*"), key=lambda p: len(p.parts), reverse=True):
            if path.is_dir():
                path.rmdir()
            else:
                path.unlink()
        args.scope_root.rmdir()
    report["pack_sha256"] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in args.store.glob("*.pack")}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
