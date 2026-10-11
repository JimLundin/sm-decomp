#!/usr/bin/env python3
"""Build host-aware Ridley terrain impact and descending scheduler cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def main():
    m=json.loads((ROOT/'tools/port/projectile-host-context.json').read_text())
    pre,frames=[],[]
    for i in range(256):
        k=[0,2,16,34][i%4]
        axis=i%3
        values=[(0x1997+k,0x9642),(0x1a03+k,0x940e),(0x1b23+k,1 if i%8==7 else 0),
                (0x1a4b+k,29),(0x1a93+k,29),(0x1a27+k,0xffa5),(0x1a6f+k,0xff5a),(0x1bb3+k,0x0202),
                (0x1ab7+k,0x300),(0x1adb+k,0x300),(0x7a5,16),(0x1b47+k,0x9754),(0x1b8f+k,2),
                (0x198d,0x8000),(0x998,40 if i%16==0 else 8),(0x5f5,int(i%16==1))]
        if axis==0:values.append((0x10002+18*2,0x8000))
        elif axis==1:values.append((0x10002+33*2,0x8000))
        if i&4:
            for s in range(0,36,2):
                if s!=k:values +=[(0x1997+s,1),(0x1a03+s,0x8170),(0x1b47+s,0x9754),(0x1b8f+s,2)]
        w=[[p,[v&255,v>>8]] for p,v in values]
        h=[0xa55a,0x5aa5,11,22,33,44,55,66]
        pre.append(dict(name=f'impact-{i}',a=k,b=0x940e,writes=w,host_words=h))
        frames.append(dict(name=f'frame-{i}',writes=w,host_words=h))
    for axis in [0,1,2]:
        for k in [0,34]:
            for gate in [0,1,2,3,4]:
                values=[(0x1997+k,0x9642),(0x1a03+k,0x940e),(0x1b23+k,0),(0x1a4b+k,29),(0x1a93+k,29),
                        (0x1bb3+k,0x0202),(0x1ab7+k,0x300),(0x1adb+k,0x300),(0x7a5,16),
                        (0x1b47+k,0x9754),(0x1b8f+k,2),(0x198d,0x8000),(0x998,40 if gate==1 else 8),
                        (0x5f5,int(gate==2)),(0x592,0x8000 if gate==3 else 0)]
                if axis in [0,2]:values.append((0x10002+18*2,0x8000))
                if axis in [1,2]:values.append((0x10002+33*2,0x8000))
                w=[[p,[v&255,v>>8]] for p,v in values]+[[0x647,[6 if gate==4 else 0]],[0x644,[0]]]
                name=f'priority-{axis}-{k}-{gate}'
                pre.append(dict(name=name,a=k,b=0x940e,writes=w,host_words=h))
                frames.append(dict(name='frame-'+name,writes=w,host_words=h))
    for k in [0,34]:
        for vx,vy in [(0xfd00,0),(0,0xfd00),(0xfd00,0xfd00)]:
            values=[(0x1997+k,0x9642),(0x1a03+k,0x940e),(0x1a4b+k,17),(0x1a93+k,17),
                    (0x1bb3+k,0x0202),(0x1ab7+k,vx),(0x1adb+k,vy),(0x7a5,16),(0x10002,0x8000),
                    (0x1b47+k,0x9754),(0x1b8f+k,2),(0x198d,0x8000)]
            w=[[p,[v&255,v>>8]] for p,v in values]
            name=f'negative-{k}-{vx}-{vy}'
            pre.append(dict(name=name,a=k,b=0x940e,writes=w,host_words=h))
            frames.append(dict(name='frame-'+name,writes=w,host_words=h))
    m['family']='ridley-fireball-host-impact'
    m['routines']=[m['routines'][0],dict(symbol='EprojPreinstruction',args=2,**{'return':'void'},cases=pre),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames)]
    (ROOT/'tools/port/ridley-fireball-host-impact.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':main()
