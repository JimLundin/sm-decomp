#!/usr/bin/env python3
"""Build real DC77 spore allocation and descending frame movement cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def main():
    m=json.loads((ROOT/'tools/port/projectile-host-context.json').read_text())
    rom=(ROOT/'rom/sm.sfc').read_bytes();assert rom[0x35c0e:0x35c10]==bytes.fromhex('77dc')
    direct,frames,pre=[],[],[]
    for i in range(256):
        k=[0,2,16,34][i%4];x=[0,127,128,255,0x7fff,0x8000,0xffff,i*997&65535][i%8]
        y=[0,766,767,768,0x7fff,0x8000,0xffff,520][(i//8)%8]
        values=[(0x1997+k,0xde88),(0x1a03+k,0xdd46),(0x1b23+k,5),(0x1b47+k,0xdc0e),(0x1b8f+k,1),
                (0x1a4b+k,x),(0x1a93+k,y),(0x5e5,i*257)]
        if i&4:
            for s in range(0,36,2):
                if s!=k:values +=[(0x1997+s,1),(0x1a03+s,0x8170),(0x1b47+s,0x9754),(0x1b8f+s,2)]
            if i&8:values.append((0x1997+(0 if k else 2),0))
        w=[[p,[v&255,v>>8]] for p,v in values];h=[1,2,3,4,5,6,7,8]
        direct.append(dict(name=f'spawn-{i}',a=k,b=0x86dc0e,writes=w,host_words=h))
        frames.append(dict(name=f'frame-{i}',writes=w+[[0x198d,[0,128]]],host_words=h))
        vals=[(0x1997+k,0xde7a),(0x1a03+k,0xdcee),(0x1aff+k,i),(0x1b23+k,x),(0x1a4b+k,x),(0x1a93+k,y)]
        pre.append(dict(name=f'move-{i}',a=k,b=0xdcee,writes=[[p,[v&255,v>>8]] for p,v in vals],host_words=h))
    m['family']='host-spore-spawning'
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=direct),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames),dict(symbol='EprojPreinstruction',args=2,**{'return':'void'},cases=pre)]
    (ROOT/'tools/port/host-spore-spawning.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':main()
