#!/usr/bin/env python3
"""Native-ROM randomized projectile sprite caller fixtures."""
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/projectile-script-leaves.json').read_text())
    manifest['family'] = 'projectile-sprite-spawns'
    manifest['c_support'] = [p.name for p in sorted((ROOT / 'deps/snesrev-sm/src').glob('sm_*.c'))
                             if p.name != 'sm_86.c']
    rom = (ROOT / 'rom/sm.sfc').read_bytes()
    # Find actual native instruction operands rather than inventing a ROM view.
    routines = []
    for symbol, opcode in (('EprojInstr_ECE3', 0xece3), ('EprojInstr_ED17', 0xed17)):
        operands = []
        # Native instruction streams can be byte-aligned.
        for offset in range(0x30000, 0x37ffc):
            command, sprite = struct.unpack_from('<HH', rom, offset)
            if command == opcode and sprite <= 61:
                operands.append(0x868000 + offset - 0x30000 + 2)
        if not operands:
            raise ValueError(f'no native operands for {symbol}')
        cases = []
        for pointer in operands:
            for slot in (0, 16, 34):
                for seed in (0, 1, 0x1234, 0xffff):
                    for free in (-1, 0, 31):
                        values = [(0x5e5, seed), (0x1a4b + slot, 0), (0x1a93 + slot, 0xffff)]
                        values += [(0xef78 + i * 2, 0 if i == free else 0xbe5a) for i in range(32)]
                        cases.append(dict(name=f'{symbol}-{pointer}-{slot}-{seed}-{free}', a=slot, b=pointer,
                                          writes=[[p, [v & 255, v >> 8]] for p, v in values]))
        routines.append(dict(symbol=symbol, arg_types=['uint16', 'rom_ptr'],
                             **{'return': 'rom_ptr'}, pointer_bank=134, cases=cases))
    manifest['routines'] = routines
    (ROOT / 'tools/port/projectile-sprite-spawns.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
