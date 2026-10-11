#!/usr/bin/env python3
"""Animated tile allocation/control against actual bank-87 ROM headers."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/sprite-objects.json').read_text())
    manifest.update(family='animated-tiles', bank='sm_87.c',
                    rust_module='crates/port/src/animated_tiles.rs')
    routines = []
    for name in ('EnableAnimtiles', 'DisableAnimtiles', 'ClearAnimtiles'):
        cases = [dict(name=f'{name}-{flag}', fill=165,
                      writes=[[0x1ef1, [flag & 255, flag >> 8]]])
                 for flag in (0, 1, 0x7fff, 0x8000, 0xffff)]
        routines.append(dict(symbol=name, args=0, **{'return': 'void'}, cases=cases))
    cases = []
    for free in range(-1, 6):
        for header in (0x82ab, 0x82c9, 0x82e7, 0x82fd):
            values = [(0x1ef5 + i * 2, 0 if i == free else 0x82ab) for i in range(6)]
            cases.append(dict(name=f'spawn-{free}-{header}', a=header, fill=165,
                              writes=[[p, [v & 255, v >> 8]] for p, v in values]))
    routines.append(dict(symbol='SpawnAnimtiles', args=1, **{'return': 'void'}, cases=cases))
    manifest['routines'] = routines
    (ROOT / 'tools/port/animated-tiles.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
