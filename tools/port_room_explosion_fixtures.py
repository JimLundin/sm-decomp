#!/usr/bin/env python3
"""Reproduce room explosion gates, native selectors and allocation cases."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/room.json').read_text())
    manifest['family'] = 'room-random-explosions'
    manifest['bank'] = 'sm_8f.c'
    manifest['c_support'] = [p.name for p in sorted((ROOT / 'deps/snesrev-sm/src').glob('sm_*.c'))
                             if p.name != 'sm_8f.c']
    routines = []
    for symbol in ('RoomCode_GenRandomExplodes', 'RoomCode_GenRandomExplodes_Nonblank',
                   'RoomCode_GenRandomExplodes_4th', 'RoomCode_ExplodeShakes'):
        cases = []
        for seed in range(16):
            for frozen, frame in ((0, 0), (0, 1), (0, 2), (1, 0)):
                for blank in (0, 255):
                    for full in (False, True):
                        values = [(0x5e5, seed), (0xa78, frozen), (0x5b6, frame),
                                  (0x911, 0), (0x915, 0), (0x7a5, 16), (0x1840, 3)]
                        if full:
                            values += [(0xef78 + i * 2, 0xbe5a) for i in range(32)]
                        # With these seeds the first random Y byte is one.
                        values += [(0x10002 + i * 2, blank) for i in range(16)]
                        case = dict(name=f'{symbol}-{seed}-{frozen}-{frame}-{blank}-{full}',
                                    writes=[[p, [v & 255, v >> 8]] for p, v in values])
                        if symbol == 'RoomCode_GenRandomExplodes':
                            case.update(a=0xffff, b=0x8000)
                        cases.append(case)
        routines.append(dict(symbol=symbol, args=2 if symbol == 'RoomCode_GenRandomExplodes' else 0,
                             **{'return': 'void'}, cases=cases))
    manifest['routines'] = routines
    (ROOT / 'tools/port/room-random-explosions.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
