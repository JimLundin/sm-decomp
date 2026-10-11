#!/usr/bin/env python3
"""Build actual-C Yapping Maw body allocation and wrapped-link fixtures."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/botwoon-body-allocation.json').read_text())
    manifest['family'] = 'yapping-maw-body'
    cases = []
    for i in range(384):
        enemy = (i % 32) * 64
        free = [0, 2, 16, 34, None][i % 5]
        body_index = [0, 1, 2, 3, 0x7fff, 0x8000, 0xfffe, 0xffff][(i // 5) % 8]
        target = (enemy + 2 * body_index) & 0xffff
        values = [(0x1997 + s, 1) for s in range(0, 36, 2)]
        if free is not None:
            values += [(0x1997 + free, 0)]
        values += [(0xe54, enemy), (0xf7a + enemy, i * 997),
                   (0xf7e + enemy, i * 733), (0xfb6 + enemy, [0, 1, 0x8000, 0xffff][(i // 32) % 4]),
                   (0x8800 + target, 0xdead), (0x8808 + enemy, body_index),
                   (0x1c27, 0xffff)]
        cases.append(dict(name=f'body-{i}', a=0xec95, b=i,
                          writes=[[p, [v & 255, (v >> 8) & 255]] for p, v in values]))
    manifest['routines'][1]['cases'] = cases
    (ROOT / 'tools/port/yapping-maw-body.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
