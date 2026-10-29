#!/usr/bin/env python3
"""Inventory ROM table reads in game/ for the sm_data migration (REVIEW phase 2a).

Each 24-bit LoROM literal read through a ROM accessor is grouped into a table.
Per table the report lists read sites, files, whether every reader is a legacy
bank, the bytes up to the next referenced address in the same bank (an upper
bound on the extent, not a proof) and any overlap with runtime-patched ROM
bytes. Runtime patches live in rom_patches.rs and cpu_infra.rs; a static
sm_data copy of an overlapping range would diverge from the patched image.

Run from the repository root: python3 tools/rom_tables.py [--json] [--top N]
"""
import json
import os
import re
import sys
from collections import defaultdict

SRC = "sm/src"
READ = re.compile(
    r"\b(rom_table_word!|lorom_offset|rom_words?|rom_bytes?|table_word|RomPtr\w*|table)\s*\(")
LITERAL = re.compile(r"\b0x([89a-dA-D][0-9a-fA-F])([89a-fA-F][0-9a-fA-F]{3})\b")
PATCH_TUPLE = re.compile(r"\(\s*0x([0-9a-fA-F]{6}),\s*&\[([^\]]*)\]", re.S)
LEGACY = re.compile(r"game/sm_[0-9a-f]{2}\.rs$")


def lorom(address):
    return ((address >> 16) & 0x7F) << 15 | (address & 0x7FFF)


def patched_bytes():
    """LoROM offsets of every ROM byte the runtime may rewrite."""
    out = set()
    with open(os.path.join(SRC, "rom_patches.rs"), encoding="utf-8") as f:
        text = f.read()
    for m in PATCH_TUPLE.finditer(text):
        body = m.group(2).strip()
        repeat = re.fullmatch(r"[^;]+;\s*(\d+)", body)
        length = int(repeat.group(1)) if repeat else len([b for b in body.split(",") if b.strip()])
        start = lorom(int(m.group(1), 16))
        out.update(range(start, start + length))
    # Carry fixups and bug hooks rewrite single code bytes at listed addresses.
    with open(os.path.join(SRC, "cpu_infra.rs"), encoding="utf-8") as f:
        text = f.read()
    for m in LITERAL.finditer(text):
        out.add(lorom(int(m.group(1) + m.group(2), 16)))
    return out


def sites():
    for root, _, names in os.walk(os.path.join(SRC, "game")):
        for name in sorted(names):
            if not name.endswith(".rs"):
                continue
            path = os.path.join(root, name)
            rel = os.path.relpath(path, SRC).replace(os.sep, "/")
            with open(path, encoding="utf-8", errors="replace") as f:
                for number, line in enumerate(f, 1):
                    code = line.split("//", 1)[0]
                    kind = READ.search(code)
                    if not kind:
                        continue
                    for m in LITERAL.finditer(code[kind.start():]):
                        yield rel, number, kind.group(1), int(m.group(1) + m.group(2), 16)


def main():
    tables = defaultdict(lambda: {"sites": 0, "files": set(), "kinds": set()})
    for rel, _, kind, address in sites():
        t = tables[address]
        t["sites"] += 1
        t["files"].add(rel)
        t["kinds"].add(kind)
    patched = patched_bytes()
    ordered = sorted(tables)
    rows = []
    for i, address in enumerate(ordered):
        t = tables[address]
        following = next((a for a in ordered[i + 1:] if a >> 16 == address >> 16), None)
        bound = (following - address) if following else 0x10000 - (address & 0xFFFF)
        start = lorom(address)
        overlap = sorted(o for o in patched if start <= o < start + bound)
        rows.append({
            "address": f"{address:06x}",
            "sites": t["sites"],
            "files": sorted(t["files"]),
            "kinds": sorted(t["kinds"]),
            "legacy_only": all(LEGACY.search(f) for f in t["files"]),
            "bound_bytes": bound,
            "patched_within_bound": len(overlap),
        })
    rows.sort(key=lambda r: (-r["sites"], -len(r["files"]), r["address"]))
    summary = {
        "tables": len(rows),
        "sites": sum(r["sites"] for r in rows),
        "non_legacy_tables": sum(1 for r in rows if not r["legacy_only"]),
        "non_legacy_sites": sum(r["sites"] for r in rows if not r["legacy_only"]),
        "tables_with_patch_in_bound": sum(1 for r in rows if r["patched_within_bound"]),
    }
    if "--json" in sys.argv:
        print(json.dumps({"summary": summary, "tables": rows}, indent=1))
        return
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 30
    print(f"{summary['tables']} ROM tables, {summary['sites']} read sites")
    print(f"outside legacy banks: {summary['non_legacy_tables']} tables, {summary['non_legacy_sites']} sites")
    print(f"tables whose neighbour bound contains patched bytes: {summary['tables_with_patch_in_bound']}")
    for r in rows[:top]:
        flag = " PATCHED-IN-BOUND" if r["patched_within_bound"] else ""
        scope = "legacy" if r["legacy_only"] else f"{len(r['files'])} files"
        print(f"  ${r['address']}  {r['sites']:3} sites  {scope:9} <= {r['bound_bytes']:5} bytes{flag}")


if __name__ == "__main__":
    main()
