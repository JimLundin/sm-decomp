#!/usr/bin/env python3
"""Actual-ROM sprite rendering cases at signed clipping and OAM boundaries."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/sprite-objects.json').read_text())
    cases = []
    for slot in (0, 30, 62):
        for x in (0xffef, 0xfff0, 0xffff, 0, 255, 271, 272, 0x7fff, 0x8000):
            for y in (0xffff, 0, 255, 271, 272, 0x8000):
                for cursor in (0, 28, 508):
                    values = [(0xef78 + slot, 0xbe6c), (0xf0f8 + slot, x),
                              (0xf1f8 + slot, y), (0xf078 + slot, 0xffff),
                              (0x590, cursor), (0x570, 0x5555)]
                    cases.append(dict(name=f'draw-{slot}-{x}-{y}-{cursor}',
                                      writes=[[p, [v & 255, v >> 8]] for p, v in values]))
    manifest['family'] = 'sprite-rendering'
    manifest['c_support'] = list(dict.fromkeys(manifest['c_support'] + ['sm_81.c']))
    manifest['routines'] = [dict(symbol='DrawSpriteObjects', args=0,
                                 **{'return': 'void'}, cases=cases)]
    (ROOT / 'tools/port/sprite-rendering.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
