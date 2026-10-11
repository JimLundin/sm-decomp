#!/usr/bin/env python3
"""Sprite allocation cases covering slots and native caller list indices."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/sprite-objects.json').read_text())
    cases = []
    for free in range(-1, 32):
        for list_id in (0, 1, 3, 7, 8, 9, 15, 21, 24, 29, 43, 44, 45, 46, 61):
            for x, y, palette in ((0, 0, 0), (0xffff, 0x8000, 0xffff), (123, 456, 0xe05)):
                values = [(0xef78 + i * 2, 0 if i == free else 0xbe5a) for i in range(32)]
                cases.append(dict(name=f'alloc-{free}-{list_id}-{x}',
                                  a=x | (y << 16), b=list_id | (palette << 16), fill=165,
                                  writes=[[p, [v & 255, v >> 8]] for p, v in values]))
    manifest['family'] = 'sprite-allocation'
    manifest['routines'] = [dict(symbol='CreateSpriteAtPos', arg_types=['uint32', 'uint32'],
                                 packed_u16_args=4, **{'return': 'uint16'}, cases=cases)]
    (ROOT / 'tools/port/sprite-allocation.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
