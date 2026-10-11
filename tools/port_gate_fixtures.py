#!/usr/bin/env python3
"""Rebuild spawned gate and save station electricity oracle cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text());m['family']='shot-gates-electricity'
    rom=(ROOT/'rom/sm.sfc').read_bytes();spawn,pre,frames=[],[],[]
    for init in [0xe5d0,0xe5d5,0xe5da,0xe6ad]:
        p=rom.index(init.to_bytes(2,'little'),0x36500,0x36700)-0x30000+0x8000
        for i in range(64):
            k=[0,2,16,34][i%4];plm=(i%40)*2
            v=[(0x1997+s,1) for s in range(0,36,2)]+[(0x1997+k,0),(0x1c27,plm),
                (0x7a5,[1,16,32,64][i%4]),(0x1c87+plm,i*14),(0x1c29,0xdead),(0x1c2b,0xbeef)]
            spawn.append(dict(name=f'spawn-{init:x}-{i}',a=p,b=0,writes=writes(v)))
    for i in range(128):
        k=[0,2,16,34][i%4]
        v=[(0x1997+k,1),(0x1a03+k,0xe605),(0x1b47+k,0x9752),(0x1b8f+k,2),
           (0x19df+k,[0,0xfff,0x1000,0xffff][i%4]),
           (0x1adb+k,[0,1,255,256,0xffff,0xff00,0x8000,0x7fff][(i//4)%8]),
           (0x1a93+k,i*997),(0x1a6f+k,0xffa5)]
        pre.append(dict(name=f'move-{i}',a=k,b=0xe605,writes=writes(v)))
        frames.append(dict(name=f'frame-{i}',writes=writes(v+[(0x198d,0x8000)])))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)])]
    for name,cases,args in [('SpawnEprojWithRoomGfx',spawn,2),('EprojPreinstruction',pre,2),('EprojRunAll',frames,0)]:
        m['routines'].append(dict(symbol=name,args=args,**{'return':'void'},cases=cases))
    (ROOT/'tools/port/shot-gates-electricity.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
