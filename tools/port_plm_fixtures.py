#!/usr/bin/env python3
"""Generate PLM fixtures with synthetic scripts and pinned room-effect tables."""

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
    for seed in range(16):
        slot = [0, 2, 38, 78][seed % 4]
        pc = 0x9700 + seed * 8
        write(0x84, pc, le(0x8B17) + le(0x86BC))
        writes = [[0x1C27, le(slot)], [0x1C87 + slot, le(66)],
                  [0x1E17 + slot, le(1 | ((seed % 4) << 10))],
                  [0x7A5, le([0, 32, 256, 288][seed % 4])],
                  [0x330, le(0)], [0x1C25, le(0)]]
        cases.append({"name": f"draw-block-{seed}", "a": slot,
                      "b": 0x840000 | pc, "writes": writes, "expected_return": 0})
    effect_definitions = [0xB4B1, 0xA993, 0xB743, 0xB751, 0xCEFC, 0xD904, 0xE517]
    original = (ROOT / "rom/sm.sfc").read_bytes()
    for start, size in [(0xA7AB, 32), (0xCE41, 32), (0xE42C, 60), (0xE47E, 40)]:
        offset = 0x30000 + (start & 0x7FFF)
        write(0x86, start, original[offset:offset + size])
    for definition in effect_definitions:
        offset = 0x30000 + (definition & 0x7FFF)
        write(0x86, definition, original[offset:offset + 14])
    effect_case_index = 0
    for opcode in [0xBBE1, 0xD357, 0xD52C, 0xD77A, 0xD790, 0xD79F, 0xD7B6, 0xD30B]:
        for seed in range(8):
            slot = [0, 2, 38, 78][seed % 4]
            pc = 0x8000 + effect_case_index * 16
            effect_case_index += 1
            operand = le(0xB4B1 if opcode == 0xBBE1 else seed % 3 * 2)
            if opcode == 0xD30B:
                operand = le(0) + le(2) + le(4) + le(0)
            write(0x84, pc, le(opcode) + operand)
            writes = [[0x1997, [0] * 36], [0x1C27, le(slot)],
                      [0x1C87 + slot, le(0x208)], [0x7A5, le(32)],
                      [0x1DC7 + slot, le(0x1234)], [0x5E5, le(seed * 193)]]
            if seed % 2:
                writes.append([0x1997, le(0xB4B1) * 17 + le(0)])
            cases.append({"name": f"spawn-op-{opcode:04x}-{seed}", "a": slot,
                          "b": 0x840000 | pc, "writes": writes})
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
    # Real frame dispatch: wakeups execute the link in the same frame;
    # deletion does not suppress instruction timer decrement or script execution.
    preinstructions = [0xBD0F, 0xBD26, 0xBD50, 0xBD88, 0xBDB2, 0xBE1C,
                       0xBDD4, 0xBDE3, 0xBDF2, 0xBE01, 0xBE1F, 0xBE30, 0xB7DD,
                       0x8469, 0x84E6, 0x853D, 0x86D0, 0x8AA6, 0x8AE0,
                       0xBAFA, 0xBB6A, 0xBBA3, 0xD356, *range(0xDAD5, 0xDADE), 0xDFA8]
    preinstructions += [0xAC89, 0xB7EF, 0xB82A, 0xB846, 0xB8B0, 0xB948,
                        0xBB52, 0xBB6B, 0xBBA4, 0xD15C, 0xD1E6, 0xD33B,
                        0xD4BF, 0xD753, 0xDADE, 0xDAEE, 0xDAFE, 0xDB0E,
                        0xDB64, 0xDF89, 0xDFE6]
    for pre in preinstructions:
        for seed in range(12):
            slot = [0, 2, 38, 78][seed % 4]
            pc, link = 0xA800, 0xA810
            write(0x84, pc, le(0x86BC) + le(0x874E) + [5] + le(0x86BC))
            write(0x84, link, le(0x874E) + [7] + le(0x86BC))
            writes = [
                [0x1C23, le(0x8000)],
                [0x1C37 + slot, le(0xB703)],
                [0x1CD7 + slot, le(pre)],
                [0x1D27 + slot, le(pc)],
                [0xDEBC + slot, le(link)],
                [0xDE1C + slot, le([0, 1, 2][seed % 3])],
                [0x1D77 + slot, le([0, 1, 0x100, 0x200, 0x300, 0x400][seed % 6])],
                [0x79F, le(seed % 8)],
                [0xD828 + seed % 8, [7 if seed % 2 else 0]],
                [0xE50, le(seed)], [0xE52, le(6)],
                [0x1E6D, le(0x8000 if seed % 2 else 0)],
                [0xD821, [0x80 if seed % 2 else 0]],
                [0x5F5, le(1)],
                [0x7A5, le(32)], [0x1C87 + slot, le(2 * (32 * 8 + 4))],
                [0x1DC7 + slot, le(seed % 8)],
                [0x9A4, le([0, 0x1000, 0x2000, 0x3000][seed % 4])],
                [0xAF6, le([0, 240, 241, 0x348, 0x349, 0xAE0][seed % 6])],
                [0xAFA, le(1345 if seed % 2 else 128)],
                [0x197A, le(0x8000 if seed % 2 else 0x200)],
                [0x1978, le(0x400)], [0x592, le(seed % 2)],
                [0x8F, le(0x8000 if seed % 2 else 0)],
                [0xD8B0, [0xFF if seed % 2 else 0]],
                [0x10202 + 2 * (32 * 8 + 4), le(255 if seed % 2 else 254)],
            ]
            if pre == 0xB846:
                writes.append([0x1D77 + slot, le([0, 6, 12, 18][seed % 4])])
            frames.append({"name": f"frame-pre-{pre:04x}-{seed}",
                           "writes": writes, "expected_return": 0})
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
    render = []
    # Cross camera edges, quadrant flips, buffer limits and screen splitting.
    for vertical in [False, True]:
        for count in [1, 2, 4, 16]:
            for seed in range(32):
                pc = 0xB800
                data = le(count | (0x8000 if vertical else 0))
                for i in range(count):
                    data += le(((i + seed) % 8) | ((seed % 4) << 10))
                # A second segment tests offset relative to the original origin.
                data += le(0x0101) + le(1) + le(3 | ((seed % 4) << 10)) + le(0)
                # Each case needs its own ROM address because seed affects tiles.
                pc += len(render) * 64
                write(0x84, pc, data)
                slot = [0, 2, 38, 78][seed % 4]
                camera_x, camera_y = 16 * (seed % 3), 16 * (seed % 5)
                x = [0, 1, 15, 16, 17, 31, 32, 48][seed % 8]
                y = [0, 1, 14, 15, 16, 17, 20, 32][(seed // 4) % 8]
                tiles = []
                for tile in range(8):
                    for quadrant in range(4):
                        tiles += le(0x1000 + tile * 16 + quadrant)
                writes = [[0xDE6C + slot, le(pc)], [0x1C29, le(x)],
                          [0x1C2B, le(y)], [0x911, le(camera_x)],
                          [0x915, le(camera_y)], [0x91D, le(0x100 if seed % 2 else 0)],
                          [0x1C25, le([0, 8, 480, 504][seed % 4])],
                          [0x330, le([0, 14, 224, 240, 476, 480][seed % 6])],
                          [0xA000, tiles]]
                if seed % 2:
                    writes += [[0x1000, data], [0xDE6C + slot, le(0x1000)]]
                render.append({"name": f"render-{int(vertical)}-{count}-{seed}",
                               "a": slot, "writes": writes, "expected_return": 0})
    drawn_frames = []
    for seed in range(32):
        slot = [0, 2, 38, 78][seed % 4]
        pc, data_pc = 0xA900 + seed * 16, 0xAB00 + seed * 32
        write(0x84, pc, le(3) + le(data_pc) + le(0x86BC))
        data = le(2 | (0x8000 if seed % 2 else 0))
        data += le(1 | ((seed % 4) << 10)) + le(2) + le(0)
        write(0x84, data_pc, data)
        tiles = []
        for tile in range(8):
            for quadrant in range(4):
                tiles += le(0x2000 + tile * 16 + quadrant)
        writes = [[0x1C23, le(0x8000)], [0x1C37 + slot, le(0xB703)],
                  [0x1CD7 + slot, le(0x8469)], [0x1D27 + slot, le(pc)],
                  [0xDE1C + slot, le(1)], [0x1C87 + slot, le(2 * (32 + seed))],
                  [0x7A5, le(32)], [0xA000, tiles],
                  [0x911, le(seed % 3 * 16)], [0x915, le(0)],
                  [0x91D, le(0x100 if seed % 2 else 0)]]
        drawn_frames.append({"name": f"drawn-frame-{seed}", "writes": writes,
                             "expected_return": 0})
    for seed in range(8):
        writes = [[0x1C23, le(0x8000)], [0x7A5, le(32)], [0xA000, tiles]]
        for slot in [0, 2, 38, 78]:
            pc, data_pc = 0xAE00 + slot * 8, 0xB100 + slot * 8
            write(0x84, pc, le(2) + le(data_pc))
            write(0x84, data_pc, le(1) + le(1 + slot % 4) + le(0))
            writes += [[0x1C37 + slot, le(0xB703)], [0x1CD7 + slot, le(0x8469)],
                       [0x1D27 + slot, le(pc)], [0xDE1C + slot, le(1)],
                       [0x1C87 + slot, le(2 * (32 + 1 + seed))]]
        drawn_frames.append({"name": f"drawn-multi-frame-{seed}", "writes": writes,
                             "expected_return": 0})
    for seed in range(12):
        slot = [0, 2, 38, 78][seed % 4]
        pc = 0x8400 + seed * 8
        write(0x84, pc, le(0x86BC) + le(0x86BC))
        writes = [[0x1C23, le(0x8000)], [0x1C37 + slot, le(0xB703)],
                  [0x1CD7 + slot, le(0xB927)], [0x1D27 + slot, le(pc)],
                  [0xDE1C + slot, le(2)], [0x1997, [0] * 36],
                  [0xAF6, le(241 if seed % 2 else 240)],
                  [0xAFA, le(2081 if seed % 3 else 2080)]]
        drawn_frames.append({"name": f"escape-spawn-frame-{seed}", "writes": writes,
                             "expected_return": 0})
    coordinates = []
    for width in [0, 1, 16, 31, 255, 256, 272, 65535]:
        for block_index in [0, 2, 30, 32, 1022, 65534]:
            coordinates.append({"name": f"coords-{width}-{block_index}", "a": 78,
                                "writes": [[0x7A5, le(width)],
                                           [0x1C87 + 78, le(block_index)]],
                                "expected_return": 0})
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
                "symbol": "PlmFrame",
                "args": 0,
                "return": "uint32",
                "cases": drawn_frames,
            },
            {
                "symbol": "CalculatePlmBlockCoords",
                "args": 1,
                "return": "void",
                "cases": coordinates,
            },
            {
                "symbol": "DrawPlm",
                "args": 1,
                "return": "void",
                "cases": render,
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
