#!/usr/bin/env python3
"""Rebuild paired projectile movement and fixed extent oracle cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text());m['family']='projectile-paired-motion'
    motion,extent=[],[]
    for i in range(256):
        k=[0,2,16,34][i%4]
        v=[(0x97dc+k,0xa500|i),(0x1a4b+k,i*997),(0x1a27+k,0xffa5),
           (0x1a93+k,i*733),(0x1a6f+k,0xff5a),(0x1ab7+k,i*127),
           (0x1aff+k,i*1777),(0x1adb+k,i*233),(0x1b23+k,i*3331)]
        motion.append(dict(name=f'motion-{i}',a=k,writes=writes(v)))
    for i,x in enumerate([0,1,511,512,513,0x7fff,0x8000,0xffff]):
        for j,y in enumerate([0,1,511,512,513,0x7fff,0x8000,0xffff]):
            k=[0,2,16,34][i%4]
            extent.append(dict(name=f'extent-{i}-{j}',a=k,writes=writes([(0x1a4b+k,x),(0x1a93+k,y)])))
    pre,frames=[],[]
    for i in range(256):
        k=[0,2,16,34][i%4];camera=[0,0xff80,0x7fff][i%3]
        v=[(0x1997+k,1),(0x1a03+k,0xec05),(0x1b47+k,0x9754),(0x1b8f+k,2),
           (0x97dc+k,i),(0x911,camera),(0x915,camera),
           (0x1a4b+k,camera+[-1,0,1,255,256,257,128,0x8000][i%8]),
           (0x1a93+k,camera+[0,255,256,257][i%4]),(0x1a27+k,0xffa5),
           (0x1a6f+k,0xff5a),(0x1ab7+k,1),(0x1aff+k,0x8000),
           (0x1adb+k,2),(0x1b23+k,0x4000)]
        pre.append(dict(name=f'pre-{i}',a=k,b=0xec05,writes=writes(v)))
        frames.append(dict(name=f'frame-{i}',writes=writes(v+[(0x198d,0x8000)])))
    m['family']='botwoon-spit-scheduling'
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)]),
                   dict(symbol='Eproj_FuncE73E_MoveXY',args=1,**{'return':'void'},cases=motion),
                   dict(symbol='Eproj_FuncE722',args=1,**{'return':'uint16'},cases=extent)]
    for name,cases,args in [('EprojPreinstruction',pre,2),('EprojRunAll',frames,0)]:
        m['routines'].append(dict(symbol=name,args=args,**{'return':'void'},cases=cases))
    (ROOT/'tools/port/botwoon-spit-scheduling.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
