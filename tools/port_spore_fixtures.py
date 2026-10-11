#!/usr/bin/env python3
"""Rebuild original-ROM Spore Spawn projectile fixtures."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def writes(v):
    return [[p,[n&255,(n>>8)&255]] for p,n in v]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text());m['family']='spore-projectile-motion'
    spawn,pre,frames=[],[],[]
    for definition,count in [(0xde6c,5),(0xde88,4)]:
        for i in range(80):
            k=[0,2,16,34][i%4]
            v=[(0x1997+s,1) for s in range(0,36,2)]+[(0x1997+k,0),(0xf7a,i*997),(0xf7e,i*733)]
            spawn.append(dict(name=f'spawn-{definition:x}-{i}',a=definition,b=i%count,writes=writes(v)))
    for address in [0xdcee,0xdd46]:
        for i in range(256):
            k=[0,2,16,34][i%4]
            v=[(0x1997+k,0xde7a),(0x1a03+k,address),(0x1b47+k,0x9754),(0x1b8f+k,2),
               (0x1aff+k,0xa500|i),(0x1b23+k,[0,1,2,0x80,0xffff][i%5]),
               (0x1a4b+k,i*997),(0x1a93+k,[767,768,769,0xffff,0x8000,520][i%6]),
               (0x9000,i%2),(0x5e5,i*257)]
            pre.append(dict(name=f'pre-{address:x}-{i}',a=k,b=address,writes=writes(v)))
            frames.append(dict(name=f'frame-{address:x}-{i}',writes=writes(v+[(0x198d,0x8000)])))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)])]
    for name,cases,args in [('SpawnEprojWithRoomGfx',spawn,2),('EprojPreinstruction',pre,2),('EprojRunAll',frames,0)]:
        m['routines'].append(dict(symbol=name,args=args,**{'return':'void'},cases=cases))
    (ROOT/'tools/port/spore-projectile-motion.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
