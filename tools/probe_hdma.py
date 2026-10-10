#!/usr/bin/env python3
"""Compare synthetic HDMA instruction timing with the unmodified recording core.

This diagnostic intentionally fails while uncovered timing patterns disagree.
It never modifies the commercial game, movie inputs, or emulator sources.
"""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent.parent
TRACE = re.compile(r"([0-9a-f]{6}) .* V:\s*(\d+) H:\s*(\d+)")


def cartridge(kind, loop, phase):
    rom = bytearray(65536)
    code = bytearray([0x18, 0xfb, 0xe2, 0x30])
    for reg, value in [(0x4370, 0x40 if kind == "indirect" else 0), (0x4371, 0),
                       (0x4372, 0), (0x4373, 0x90), (0x4374, 0), (0x4377, 0)]:
        code.extend([0xa9, value, 0x8d, reg & 255, reg >> 8])
    code.extend([0xad, 0x12, 0x42, 0x10, 0xfb, 0xa9, 0x80, 0x8d, 0x0c, 0x42])
    code.extend([0xea] * phase)
    code.extend([0xea, 0xea, 0x80, 0xfc] if loop == "nop" else [0x80, 0xfe])
    rom[:len(code)] = code
    table = [3, 0, 0x91, 0x83, 3, 0x91, 0, 0, 0, 0, 0] if kind == "indirect" else [3, 0x80, 0x83, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0]
    rom[0x1000:0x100b] = bytes(table)
    rom[0x1100:0x1106] = bytes([0x80] * 6)
    rom[0x7fc0:0x7fd5] = b"SM HDMA PROBE        "
    rom[0x7fd5] = 0x20
    rom[0x7fd9] = 1
    rom[0x7fdc:0x7fe0] = bytes([0xcb, 0xed, 0x34, 0x12])
    rom[0x7ffc:0x7ffe] = bytes([0, 0x80])
    return rom


def original_rows(path):
    frame, last = 0, 0
    for line in path.read_text().splitlines():
        match = TRACE.match(line)
        if not match:
            continue
        pc, v, h = match.groups()
        clock = int(v) * 1364 + int(h)
        if clock < last:
            frame += 1
        last = clock
        # Only two frames: the short NTSC line first occurs in frame three.
        yield pc, clock + frame * 357368


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["direct", "indirect"], default="indirect")
    parser.add_argument("--loop", choices=["nop", "branch"], default="nop")
    parser.add_argument("--phases", type=int, default=8)
    args = parser.parse_args()
    if not 1 <= args.phases <= 22:
        parser.error("--phases must be between 1 and 22")
    out = ROOT / "out/sync/hdma-probes"
    out.mkdir(parents=True, exist_ok=True)
    subprocess.run(["cargo", "build", "--release", "-p", "snes", "--bin", "timing-probe", "--locked", "--offline"], cwd=ROOT, check=True)
    reports = []
    for phase in range(args.phases):
        stem = out / f"{args.kind}-{args.loop}-{phase}"
        rom = cartridge(args.kind, args.loop, phase)
        stem.with_suffix(".sfc").write_bytes(rom)
        with zipfile.ZipFile(ROOT / "movies/super-metroid-any-3653.lsmv") as source, zipfile.ZipFile(stem.with_suffix(".lsmv"), "w") as target:
            for name in source.namelist():
                data = source.read(name)
                if name == "input":
                    data = b"F. 0 0|............\n" * 10
                if name == "rom.sha256":
                    data = (hashlib.sha256(rom).hexdigest() + "\n").encode()
                if name == "rom.hint":
                    data = b"SM HDMA PROBE\n"
                target.writestr(name, data)
        oracle = stem.with_suffix(".oracle.txt")
        lua = stem.with_suffix(".lua")
        lua.write_text(f"local out=assert(io.open('{oracle}','w'))\nlocal function trace(cpu,line,real) if real then out:write(line,'\\n') end end\nmemory.registertrace(0,trace)\n")
        env = dict(os.environ, XDG_CONFIG_HOME=str(stem.with_suffix(".config")))
        with stem.with_suffix(".log").open("w") as log:
            subprocess.run([str(ROOT / "deps/lsnes/source/src/util/lsnes-dumpavi.util"), "--dumper=INTERNAL-NULL", f"--rom-a={stem}.sfc", "--length=2", f"--lua={lua}", f"{stem}.lsmv"], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
        candidate = stem.with_suffix(".candidate.csv")
        subprocess.run([str(ROOT / "target/release/timing-probe"), str(stem.with_suffix(".sfc")), str(candidate), "2"], cwd=ROOT, check=True)
        reference = list(original_rows(oracle))
        with candidate.open() as file:
            actual = [(pc, int(clock)) for pc, clock in csv.reader(file)]
        if not reference or len(actual) < len(reference):
            raise RuntimeError(f"incomplete capture for {stem}")
        mismatch = next((i for i, (a, b) in enumerate(zip(reference, actual)) if a != b), None)
        report = {"phase": phase, "status": "PASS" if mismatch is None else "FAIL", "compared_instructions": len(reference) if mismatch is None else mismatch}
        if mismatch is not None:
            report.update(original=reference[mismatch], candidate=actual[mismatch])
        reports.append(report)
        print(json.dumps(report), flush=True)
    path = out / f"{args.kind}-{args.loop}.json"
    path.write_text(json.dumps(reports, indent=2) + "\n")
    return int(any(report["status"] != "PASS" for report in reports))


if __name__ == "__main__":
    raise SystemExit(main())
