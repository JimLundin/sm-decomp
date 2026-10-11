#!/usr/bin/env python3
"""Generate native-ROM palette frame oracle and live admission fixtures."""
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/palette-effect-allocation.json').read_text())
    manifest['family'] = 'palette-effect-frames'
    manifest['c_support'] = [p.name for p in sorted((ROOT / 'deps/snesrev-sm/src').glob('sm_*.c'))
                             if p.name != 'sm_8d.c']
    manifest['rust_imports'] = ['audio', 'system', 'enemy_projectiles',
                              'enemy_projectile_instructions', 'enemy_projectile_collision', 'enemy']
    rom = (ROOT / 'rom/sm.sfc').read_bytes()
    routines = []
    for symbol in ('PalFx_ProcessOne', 'PaletteFxHandler'):
        cases = []
        for slot in range(0, 16, 2):
            for header in (0xe194, 0xe1a0, 0xe1a4, 0xe1bc, 0xf761, 0xf779):
                pointer = struct.unpack_from('<H', rom, 0x68000 + (header & 0x7fff) + 2)[0]
                for timer in (0, 1, 2, 0xffff):
                    values = [(0x1e7b, slot), (0x1e79, 0x8000),
                              (0x1e7d + slot, header), (0x1ead + slot, 0xc526),
                              (0x1ebd + slot, pointer), (0x1ecd + slot, timer),
                              (0x1e8d + slot, 0), (0x79f, 0), (0xd828, 0)]
                    case = dict(name=f'{symbol}-{slot}-{header}-{timer}',
                                writes=[[p, [v & 255, v >> 8]] for p, v in values])
                    if symbol == 'PalFx_ProcessOne':
                        case['a'] = slot
                    cases.append(case)
        routines.append(dict(symbol=symbol, args=int(symbol == 'PalFx_ProcessOne'),
                             **{'return': 'void'}, cases=cases))
    for symbol in ('EnablePaletteFx', 'DisablePaletteFx', 'ClearPaletteFXObjects'):
        routines.append(dict(symbol=symbol, args=0, **{'return': 'void'}, cases=[
            dict(name=f'{symbol}-{flag}', writes=[[0x1e79, [flag & 255, flag >> 8]]])
            for flag in (0, 1, 0x8000, 0xffff)]))
    manifest['routines'] = routines
    (ROOT / 'tools/port/palette-effect-frames.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
