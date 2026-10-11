#!/usr/bin/env python3
"""Rebuild enemy-linked decoration and Nuclear Waffle allocation fixtures."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text())
    m['family']='botwoon-body-allocation'
    cases=[]
    for definition in [0xeba0]:
        for i in range(192):
            enemy=(i%16)*64
            target=enemy+((i//16)%8)*64
            free=[0,2,16,34,None][i%5]
            vals=[(0x1997+s,1) for s in range(0,36,2)]
            if free is not None:
                vals += [(0x1997+free,0)]
            vals += [(0xe54,enemy),(0x8014+enemy,target-enemy),
                     (0x8022+enemy,i*997),(0x8024+enemy,i*733),
                     (0x7800+target,0xdead),(0xfa8,target),(0xf7a,i*997),(0xf7e,i*733),(0x7820+target,0xbeef)]
            vals += [(0xf78+enemy+d,i*1777+d*93) for d in [2,4,6,8]]
            vals += [(0xf380+s,0x1234+s) for s in range(0,36,2)]
            cases.append(dict(name=f'spawn-{definition:x}-{i}',a=definition,b=i%3,writes=writes(vals)))
    pre,frames=[],[]
    for state in [0xea98,0xeaf4,0xeb04,0xeb1f,0xeb93]:
        for i in range(64):
            k=[0,10,16,34][i%4]
            v=[(0x1997+k,1),(0x1a03+k,0xea80),(0x1ab7+k,state),
               (0x1aff+k,([0,16,48,63][i%4] if state==0xea98 else [0,254,255,256][i%4])),
               (0x1b23+k,0xe8f3),(0x1b47+k,0x9754),(0x1b8f+k,2),
               (0x1a93+k,[0,199,200,201][i%4]),(0x1a6f+k,0xffa5),
               (0x1adb+k,(i%64)*256),(0x19bb+k,0x100),(0x8020,(i//4)%2),
               (0xf9c,(i//8)%2),(0xe44,(i//16)%4)]
            pre.append(dict(name=f'pre-{state:x}-{i}',a=k,b=0xea80,writes=writes(v)))
            frames.append(dict(name=f'frame-{state:x}-{i}',writes=writes(v+[(0x198d,0x8000)])))
    m['family']='botwoon-body-lifecycle'
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)]),
                   dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=cases)]
    for name,cases,args in [('EprojPreinstruction',pre,2),('EprojRunAll',frames,0)]:
        m['routines'].append(dict(symbol=name,args=args,**{'return':'void'},cases=cases))
    (ROOT/'tools/port/botwoon-body-lifecycle.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
