#!/usr/bin/env python3
"""Build Mother Brain turret screen/destruction/rotation/bullet host cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def main():
    m=json.loads((ROOT/'tools/port/projectile-host-context.json').read_text());pre,frames=[],[]
    rom=(ROOT/'rom/sm.sfc').read_bytes()
    assert rom[0x36509:0x3650b]==bytes.fromhex('68e4')
    assert rom[0x3418c:0x3418e]==bytes.fromhex('59bf')
    for i in range(256):
        k=[0,2,16,34][i%4];camera=[0,0xff80,0x7fff][i%3]
        x=camera+[-5,-4,0,259,260,261,0x8000,128][i%8]
        y=camera+[-17,-16,0,239,240,241,0x8000,32][(i//8)%8]
        vals=[(0x1997+k,0xc17e),(0x1a03+k,0xbfdf),(0x1b47+k,0x9754),(0x1b8f+k,2),
              (0x1a4b+k,x&65535),(0x1a93+k,y&65535),(0x911,camera),(0x915,camera),(0x783a,i%2),
              (0x1a6f+k,0x100|(i%8)),(0x1a27+k,0xbec1),(0x1ab7+k,[0,1,2,0xffff][i%4]),
              (0x1adb+k,[1,2,0,0xffff][(i//4)%4]),(0x5e5,i*257),(0x7a5,16)]
        if i&16:
            for s in range(0,36,2):
                if s!=k:vals +=[(0x1997+s,1),(0x1a03+s,0x8170),(0x1b47+s,0x9754),(0x1b8f+s,2)]
        w=[[p,[v&255,(v>>8)&255]] for p,v in vals];h=[1,2,3,4,5,6,7,8]
        pre.append(dict(name=f'turret-{i}',a=k,b=0xbfdf,writes=w,host_words=h))
        frames.append(dict(name=f'frame-{i}',writes=w+[[0x198d,[0,128]]],host_words=h))
    for i in range(32):
        k=[0,34][i%2]
        vals=[(0x1997+k,0xc17e),(0x1a03+k,0xbfdf),(0x1b47+k,0x9754),(0x1b8f+k,2),
              (0x1a4b+k,100),(0x1a93+k,32),(0x783a,(i//2)%2),(0x1a6f+k,0x100|i%8),
              (0x1a27+k,0xbec1),(0x1ab7+k,1),(0x1adb+k,1),(0x7a5,16)]
        w=[[p,[v&255,v>>8]] for p,v in vals]
        pre.append(dict(name=f'active-{i}',a=k,b=0xbfdf,writes=w,host_words=h))
        frames.append(dict(name=f'frame-active-{i}',writes=w+[[0x198d,[0,128]]],host_words=h))
    m['family']='mother-brain-turret-host-lifecycle'
    m['routines']=[m['routines'][0],dict(symbol='EprojPreinstruction',args=2,**{'return':'void'},cases=pre),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames)]
    (ROOT/'tools/port/mother-brain-turret-host-lifecycle.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':main()
