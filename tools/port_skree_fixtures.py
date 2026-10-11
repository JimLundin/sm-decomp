#!/usr/bin/env python3
"""Rebuild Skree particle allocation, motion and visibility oracle cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text());m['family']='skree-particles'
    spawn,pre,frames=[],[],[]
    for definition in [0x8bc2,0x8bd0,0x8bde,0x8bec]:
        for i in range(48):
            k=[0,2,16,34][i%4];enemy=(i%32)*64
            v=[(0x1997+s,1) for s in range(0,36,2)]+[(0x1997+k,0),(0xe54,enemy),
                (0xf7a+enemy,i*1777),(0xf7e+enemy,i*733)]
            spawn.append(dict(name=f'spawn-{definition:x}-{i}',a=definition,b=i,writes=writes(v)))
    for address in [0x8b5d,0xe4fe]:
        for i in range(128):
            k=[0,2,16,34][i%4];camera=[0,0xff80,0x7fff][i%3]
            v=[(0x1997+k,1),(0x1a03+k,address),(0x1b47+k,0x9754),(0x1b8f+k,2),
               (0x911,camera),(0x915,camera),(0x1a4b+k,camera+[-1,0,1,128,255,256,257,0x8000][i%8]),
               (0x1a93+k,camera+[0,255,256,257][i%4]),(0x1a27+k,0xffa5),(0x1a6f+k,0xff5a),
               (0x1ab7+k,[0,1,0x100,0xffff,0xff00,0x7fff,0x8000,320][(i//4)%8]),
               (0x1adb+k,[0,1,0xff00,0xfbff][(i//32)%4])]
            pre.append(dict(name=f'pre-{address:x}-{i}',a=k,b=address,writes=writes(v)))
            frames.append(dict(name=f'frame-{address:x}-{i}',writes=writes(v+[(0x198d,0x8000)])))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)])]
    for name,cases,args in [('SpawnEprojWithRoomGfx',spawn,2),('EprojPreinstruction',pre,2),('EprojRunAll',frames,0)]:
        m['routines'].append(dict(symbol=name,args=args,**{'return':'void'},cases=cases))
    (ROOT/'tools/port/skree-particles.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
