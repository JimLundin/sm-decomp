#!/usr/bin/env python3
"""Pinned B2 pirate caller scripts, current-enemy context and allocator refusal."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def main():
    m = json.loads((ROOT / 'tools/port/pirate-host-context.json').read_text())
    rom = (ROOT / 'rom/sm.sfc').read_bytes()
    routines = [m['routines'][0]]
    for name, opcode, pointers in [
        ('SpacePirates_Instr_FireLaserL', 0xef2a, [0xecce]),
        ('SpacePirates_Instr_FireLaserR', 0xef5d, [0xed8e]),
        ('SpacePirates_Instr_15', 0xf564, [0xf17a, 0xf1aa, 0xf1e2, 0xf368, 0xf398, 0xf3d0]),
    ]:
        for pc in pointers:
            p = 0x190000 + (pc & 0x7fff)
            assert rom[p-2:p] == opcode.to_bytes(2, 'little')
            if opcode == 0xf564:
                assert int.from_bytes(rom[p:p+2], 'little') in (0, 1)
        cases = []
        for i in range(192):
            enemy = i % 32 * 64
            k = [0, 2, 16, 34][i % 4]
            full = i % 6 == 0
            values = [(0x1997+s, 0xaaaa) for s in range(0, 36, 2)]
            if not full:
                values.append((0x1997+k, 0))
            values += [(0xe54, enemy), (0xf78+enemy, 0xd37f),
                       (0xf78+enemy+2, [0, 23, 24, 0x7fff, 0x8000, 0xffff][i % 6]),
                       (0xf78+enemy+6, [0, 15, 16, 0xffff][i % 4]),
                       (0xf78+enemy+0x1e, [0, 0x8000, 0x200][i % 3]),
                       (0xf78+enemy+0x20, [0, 0x1ff, 0xffff][i % 3]),
                       (0xf78+enemy+0x3c, [0, 0x8000, 0xffff][i % 3]),
                       (0x1993, 0xface), (0x998, 40 if i % 16 == 0 else 8),
                       (0x5f5, int(i % 16 == 1))]
            writes = [[p, [v & 255, v >> 8]] for p, v in values]
            writes += [[0x647, [[0, 6, 15][i % 3]]], [0x644, [int(i % 4 == 0)]], [0x654, [0xa5]]]
            cases.append(dict(name=f'{opcode:04x}-{i}', a=(enemy+64)&0x7c0,
                              b=0xb20000+pointers[i % len(pointers)], writes=writes,
                              host_words=[0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0xffff, 0x1234]))
        routines.append(dict(symbol=name, arg_types=['uint16', 'rom_ptr'],
                             pointer_bank=0xb2, **{'return': 'rom_ptr'}, cases=cases))
    m['family'] = 'pirate-host-callers'
    m['routines'] = routines
    (ROOT / 'tools/port/pirate-host-callers.json').write_text(json.dumps(m, separators=(',', ':'))+'\n')


if __name__ == '__main__':
    main()
