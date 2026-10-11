#!/usr/bin/env python3
"""Rebuild enemy-linked decoration and Nuclear Waffle allocation fixtures."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text())
    m['family']='botwoon-spit-allocation'
    cases=[]
    for definition in [0xec48]:
        for i in range(256):
            enemy=(i%16)*64
            target=enemy+((i//16)%8)*64
            free=[0,2,16,34,None][i%5]
            vals=[(0x1997+s,1) for s in range(0,36,2)]
            if free is not None:
                vals += [(0x1997+free,0)]
            vals += [(0xe54,enemy),(0x8014+enemy,target-enemy),
                     (0x8022+enemy,i*997),(0x8024+enemy,i*733),
                     (0x7800+target,0xdead),(0xfa8,target),(0x60b,0xa500|i),(0xf7a,i*997),(0xf7e,i*733),(0x7820+target,0xbeef)]
            vals += [(0xf78+enemy+d,i*1777+d*93) for d in [2,4,6,8]]
            vals += [(0xf380+s,0x1234+s) for s in range(0,36,2)]
            cases.append(dict(name=f'spawn-{definition:x}-{i}',a=definition,b=[0,1,255,256,0x7fff,0x8000,0xffff][i%7],writes=writes(vals)))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)]),
                   dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=cases)]
    (ROOT/'tools/port/botwoon-spit-allocation.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
