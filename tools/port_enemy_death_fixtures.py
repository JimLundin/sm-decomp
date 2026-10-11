#!/usr/bin/env python3
"""Rebuild enemy-death explosion allocation fixtures against pinned bank86."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/botwoon-body-allocation.json').read_text())
    manifest['family'] = 'enemy-death-explosion'
    cases = []
    for animation in range(5):
        for i in range(192):
            enemy = (i % 32) * 64
            free = [0, 2, 16, 34, None][i % 5]
            values = [(0x1997 + s, 1) for s in range(0, 36, 2)]
            if free is not None:
                values += [(0x1997 + free, 0)]
            values += [(0xe54, enemy), (0xf7a + enemy, i * 997), (0xf7e + enemy, i * 733),
                       (0xf78 + enemy, 0x8000 + i * 17),
                       (0xf86 + enemy, [0, 0x4000, 0xffff, 0xbfff][(i // 32) % 4]),
                       (0x1c27, 0xffff)]
            values += [(base + s, 0xa55a) for s in range(0, 36, 2) for base in [0xf3c8, 0xf410, 0x19bb]]
            cases.append(dict(name=f'animation-{animation}-{i}', a=0xf345, b=animation,
                              writes=[[p, [v & 255, (v >> 8) & 255]] for p, v in values]))
    manifest['routines'][1]['cases'] = cases
    (ROOT / 'tools/port/enemy-death-explosion.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
