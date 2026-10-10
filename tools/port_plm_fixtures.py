#!/usr/bin/env python3
"""Generate synthetic PLM bytecode/WRAM fixtures; no original-ROM bytes."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def generate(output_dir: Path, manifest_path: Path | None = None) -> Path:
    """Write reproducible assets and a manifest to the selected output directory."""
    rom = bytearray(0x300000)

    def write(bank, pc, data):
        rom[
            (bank & 0x7F) * 0x8000 + (pc & 0x7FFF) : (bank & 0x7F) * 0x8000
            + (pc & 0x7FFF)
            + len(data)
        ] = bytes(data)

    def le(w):
        return [w & 255, w >> 8]

    opcodes = [
        0x86B4,
        0x86BC,
        0x86C1,
        0x86CA,
        0x8724,
        0x873F,
        0x874E,
        0x87E5,
        0x880E,
        0x882D,
        0x883E,
        0x8848,
        0x8865,
        0x887C,
        0x8899,
        0x8A24,
        0x8A2E,
        0x8A3A,
        0x8A72,
        0x8A91,
        0x8ACD,
        0x8AF1,
        0x8B55,
        0x8B93,
        0x8BD1,
        0x8BDD,
        0x8D41,
    ] + list(range(0x8C07, 0x8C86, 9))
    cases = []
    for oi, op in enumerate(opcodes):
        pc = 0x9000 + oi * 32
        for seed in range(8):
            slot = [0, 2, 38, 78][seed % 4]
            argument = [0, 1, 7, 8, 63, 0x8001, 0xFFFF, 3][seed]
            operand = le(0x9800)
            if op == 0x874E:
                operand = [9]
            if op == 0x87E5:
                operand = le(4) + le(0x2000) + [0x7E] + le(0x3000)
            if op == 0x880E:
                operand = [2] + le(0x9800)
            if op in (0x882D, 0x883E):
                operand = le(7) + le(0x9800)
            if op in (0x8A91, 0x8ACD):
                operand = [4] + le(0x9800)
            if op in (0x8AF1, 0x8BD1, 0x8BDD) or 0x8C07 <= op <= 0x8C85:
                operand = [6]
            if op == 0x8D41:
                operand = [3, 2] + le(0x9800)
            write(0x84, pc, le(op) + operand)

            def word(off, value):
                return [off, le(value)]

            writes = [
                word(0x1C37 + slot, 0xB703),
                word(0x1C87 + slot, 0x200),
                word(0x1CD7 + slot, 0x853D),
                word(0x1D27 + slot, pc),
                word(0x1D77 + slot, [1, 2, 0, 0x1201][seed % 4]),
                word(0x1DC7 + slot, argument),
                word(0xDEBC + slot, 0x9800),
                word(0xDF0C + slot, [0, 2, 3, 0xFF, 0xFFFF, 4, 5, 0x100][seed]),
                word(0x7A5, 16),
                word(0xAF6, 0x20),
                word(0xAFA, 0x100),
                word(0x5E7, 0x55AA),
                word(0x330, seed * 7),
                word(0x998, 8),
                word(0x79F, seed % 8),
                [0xD828 + seed % 8, [2 if seed % 2 else 0]],
                [0xD820, [0x80 if seed % 2 else 0]],
                [0xD830, [0xFF if seed % 2 else 0]],
                [0xD870, [0xFF if seed % 2 else 0]],
                [0xD8B0, [0xFF if seed % 2 else 0]],
                word(0x10002 + 0x200, 0x8123),
            ]
            if op in (0x8B55, 0x8B93):
                writes.append(word(0x1DC7 + slot, 0x8800))
                write(0x8F, 0x8800, [0, 1, 3, 2, 0x80])
            cases.append(
                {
                    "name": f"op-{op:04x}-{seed}",
                    "a": slot,
                    "b": 0x840000 | pc,
                    "writes": writes,
                }
            )
    frames = []
    for seed in range(12):
        pc = 0xA000 + seed * 64
        # Odd-byte timer loop, single-register subroutine, then delete or sleep.
        script = (
            le(0x874E)
            + [3]
            + le(0x873F)
            + le(pc + 3)
            + le(0x8A2E)
            + le(pc + 13)
            + le(0x86BC if seed % 2 == 0 else 0x86B4)
            + le(0x8A3A)
        )
        write(0x84, pc, script)
        slot = [0, 2, 38, 78][seed % 4]
        writes = [
            [0x1C23, le(0x8000)],
            [0x1C37 + slot, le(0xB703)],
            [0x1CD7 + slot, le(0x853D)],
            [0x1D27 + slot, le(pc)],
            [0xDE1C + slot, le([1, 2, 0][seed % 3])],
        ]
        frames.append({"name": f"frame-{seed}", "writes": writes, "expected_return": 0})
    draw = []
    for seed in range(12):
        pc = 0xB000 + seed * 32
        data = (
            le(3)
            + le(0x1111)
            + le(0x2222)
            + le(0x3333)
            + le(0x01FF)
            + le(0x8002)
            + le(0x4444)
            + le(0x5555)
            + le(0)
        )
        write(0x84, pc, data)
        slot = [0, 2, 38, 78][seed % 4]
        writes = [[0x1C87 + slot, le(0x400)], [0xDE6C + slot, le(pc)], [0x7A5, le(16)]]
        if seed % 2:
            writes.append([0x1000, list(data)])
            writes[1] = [0xDE6C + slot, le(0x1000)]
        draw.append(
            {"name": f"draw-{seed}", "a": slot, "writes": writes, "expected_return": 0}
        )
    output_dir = output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / "plm-instructions-synthetic.sfc"
    path.write_bytes(rom)
    manifest_path = (manifest_path or output_dir / "plm-instructions.json").resolve()
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest = {
        "family": "plm-instructions",
        "bank": "sm_84.c",
        "c_support": ["sm_80.c", "sm_rtl.c"],
        "rust_module": "crates/port/src/plm_instructions.rs",
        "rust_imports": ["audio"],
        "rom": {"file": str(path), "sha256": hashlib.sha256(rom).hexdigest()},
        "routines": [
            {
                "symbol": "PlmInstruction",
                "arg_types": ["uint16", "rom_ptr"],
                "return": "rom_ptr",
                "cases": cases,
            },
            {
                "symbol": "PlmControlFrame",
                "args": 0,
                "return": "uint32",
                "cases": frames,
            },
            {
                "symbol": "ProcessPlmDrawInstruction",
                "args": 1,
                "return": "void",
                "cases": draw,
            },
        ],
    }
    metadata = {k: v for k, v in manifest.items() if k != "routines"}
    rows = []
    for routine in manifest["routines"]:
        fields = {k: v for k, v in routine.items() if k != "cases"}
        row = json.dumps(fields, separators=(",", ":"))[:-1] + ',"cases":[\n'
        row += (
            ",\n".join(
                "    " + json.dumps(c, separators=(",", ":")) for c in routine["cases"]
            )
            + "\n  ]}"
        )
        rows.append(row)
    manifest_path.write_text(
        json.dumps(metadata, separators=(",", ":"))[:-1]
        + ',"routines":[\n'
        + ",\n".join(rows)
        + "\n]}\n"
    )
    return manifest_path


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output-dir", type=Path, default=ROOT / "out/port/plm-fixtures"
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help="Optional manifest destination; defaults inside output directory.",
    )
    args = parser.parse_args(argv)
    path = generate(args.output_dir, args.manifest)
    manifest = json.loads(path.read_text())
    print(
        json.dumps(
            {
                "manifest": str(path),
                "rom": manifest["rom"]["file"],
                "rom_sha256": manifest["rom"]["sha256"],
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
