#!/usr/bin/env python3
"""Reproduce actual-ROM animated tile frame fixtures for both oracle harnesses."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/animated-tile-events.json').read_text())
    manifest['family'] = 'animated-tile-frames'
    manifest['c_support'] = ['sm_rtl.c', 'sm_80.c', 'sm_8d.c', 'sm_86.c']
    manifest['rust_imports'] = ['audio', 'system', 'enemy_projectiles',
                                'enemy_projectile_instructions', 'enemy_projectile_collision', 'enemy']
    routines = []
    for symbol in ('ProcessAnimtilesObject', 'AnimtilesHandler'):
        cases = []
        for slot in range(0, 12, 2):
            # Preserve historical case labels while using the corrected script pointers.
            for label, pointer in ((0x82b1, 0x8293), (0x82cf, 0x82b1),
                                   (0x82ed, 0x82cf), (0x8303, 0x82ed)):
                for timer in (0, 1, 2, 0xffff):
                    values = [(0x1ef3, slot), (0x1ef1, 0x8000),
                              (0x1ef5 + slot, 0x82ab), (0x1f0d + slot, pointer),
                              (0x1f19 + slot, timer)]
                    cases.append(dict(name=f'{symbol}-{slot}-{label}-{timer}',
                                      writes=[[p, [v & 255, v >> 8]] for p, v in values]))
        routines.append(dict(symbol=symbol, args=0, **{'return': 'void'}, cases=cases))
    manifest['routines'] = routines
    (ROOT / 'tools/port/animated-tile-frames.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
    manifest['family'] = 'animated-tile-frame-live'
    manifest['c_support'] = [p.name for p in sorted((ROOT / 'deps/snesrev-sm/src').glob('sm_*.c'))
                             if p.name != 'sm_87.c']
    (ROOT / 'tools/port/animated-tile-frame-live.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
