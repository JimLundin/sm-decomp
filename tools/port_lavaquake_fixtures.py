#!/usr/bin/env python3
"""Rebuild original-ROM lavaquake rock allocation and arc oracle cases."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p, [v & 255, (v >> 8) & 255]] for p, v in values]


def main():
    m = json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text())
    m['family'] = 'lavaquake-rocks'
    pre, frames, spawn = [], [], []
    for state in [0xbc16, 0xbc8f]:
        for speed in [0, 1, 2, 3, 4, 16, 32, 62, 63, 64, 65, 90, 0xffff]:
            for i, edge in enumerate([-1, 0, 1, 128, 255, 256, 257]):
                k = [0, 2, 16, 34][i % 4]
                camera = [0, 0xff80, 0x7fff][i % 3]
                vals = [(0x1997+k, 0xbbc7), (0x1a03+k, 0xbc0f),
                        (0x1b47+k, 0x9754), (0x1b8f+k, 2),
                        (0x1aff+k, state), (0x1adb+k, speed),
                        (0x1ab7+k, [0, 1, 0xffff, 0x100, 0xff00, 0x8000, 0x7fff][i]),
                        (0x911, camera), (0x915, camera),
                        (0x1a4b+k, camera+edge), (0x1a93+k, camera+edge),
                        (0x1a27+k, 0xff5a), (0x1a6f+k, 0xffa5), (0x1b23+k, 0xdead)]
                pre.append(dict(name=f'arc-{state:x}-{speed}-{i}', a=k, b=0xbc0f, writes=writes(vals)))
                frames.append(dict(name=f'frame-{state:x}-{speed}-{i}', writes=writes(vals+[(0x198d,0x8000)])))
    rom=(ROOT/'rom/sm.sfc').read_bytes()
    definition=rom.index(bytes.fromhex('dbbb0fbc'),0x30000,0x38000)-0x30000+0x8000
    for i in range(72):
        k=[0,2,16,34][i%4]
        enemy=(i%32)*64
        vals=[(0x1997+s,1) for s in range(0,36,2)]
        vals += [(0x1997+k,0),(0xe54,enemy),(0x1995,(i*1377)&65535)]
        vals += [(0xf78+enemy+d,(i*1777+d*93)&65535) for d in [2,4,6,8]]
        spawn.append(dict(name=f'spawn-{i}',a=definition,b=[0,1,2,32,64,0xffff][i%6],writes=writes(vals)))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)]),
                   dict(symbol='EprojPreinstruction',args=2,**{'return':'void'},cases=pre),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames),
                   dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=spawn)]
    (ROOT/'tools/port/lavaquake-rocks.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__ == '__main__':
    main()
