#!/usr/bin/env python3
"""Actual-ROM sprite scheduler cases across every sprite slot."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p, [v & 255, v >> 8]] for p, v in values]


def main():
    manifest = json.loads((ROOT / 'tools/port/ceres-platform.json').read_text())
    manifest.update(family='sprite-objects', bank='sm_b4.c',
                    rust_module='crates/port/src/sprite_objects.rs')
    routines = []
    for symbol in ('SpriteObject_Instr_RepeatLast', 'SpriteObject_Instr_Terminate',
                   'SpriteObject_Instr_Goto', 'HandleSpriteObjects', 'ClearSpriteObjects'):
        cases = []
        if symbol == 'ClearSpriteObjects':
            cases = [dict(name=f'clear-{fill}', fill=fill) for fill in (0, 0xa5, 255)]
        for slot in range(0, 64, 2):
            if symbol.startswith('SpriteObject_Instr_'):
                pointers = (0xbf70, 0xc1a8, 0xc300) if symbol.endswith('Goto') else (0, 2, 0xbe5a, 0xfffe)
                for pointer in pointers:
                    cases.append(dict(name=f'{symbol}-{slot}-{pointer}', fill=0xa5,
                                      writes=writes([(0x1844, slot), (0xef78 + slot, pointer)])))
            elif symbol == 'HandleSpriteObjects':
                for pointer, timer in ((0xbe5a, 0), (0xbe5a, 1), (0xbe5a, 2),
                                       (0xbf0c, 1), (0xbe54, 1), (0xbf6c, 1),
                                       (0xbe58, 0xbcf0), (0xbf10, 0xbd07), (0xbf70, 0xbd12)):
                    for disabled in (0, 1, 2):
                        cases.append(dict(name=f'frame-{slot}-{pointer}-{timer}-{disabled}',
                                          writes=writes([(0xef78 + slot, pointer),
                                                         (0xeff8 + slot, timer),
                                                         (0xf2f8 + slot, disabled)])))
        if symbol == 'HandleSpriteObjects':
            for address in (0x185e, 0xa78):
                cases.append(dict(name=f'frozen-{address}', fill=0xa5,
                                  writes=writes([(address, 1)])))
        routines.append(dict(symbol=symbol, args=0, **{'return': 'void'}, cases=cases))
    manifest['routines'] = routines
    (ROOT / 'tools/port/sprite-objects.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
