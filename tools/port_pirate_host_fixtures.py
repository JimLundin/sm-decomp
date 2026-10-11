#!/usr/bin/env python3
"""Build pirate point/r22 and rectangle allocation followed by real scripts."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def main():
    m=json.loads((ROOT/'tools/port/projectile-host-context.json').read_text());rom=(ROOT/'rom/sm.sfc').read_bytes()
    assert rom[0x3217b:0x3217d]==bytes.fromhex('09a0');assert rom[0x32189:0x3218b]==bytes.fromhex('98a0')
    spawn,frames=[],[]
    for init,definition in [(0xa009,0xa17b),(0xa098,0xa189)]:
        for i in range(128):
            k=[0,2,16,34][i%4];enemy=(i%32)*64;parameter=[0,1,0xffff][i%3]
            x=[0,127,128,255,0x7fff,0x8000,0xffff,128][i%8];y=[0,16,239,255][(i//8)%4]
            h=[x,y,x,y,[0,1,0xffff,0x8000][i%4],[0,1,0xffff,0x8000][(i//4)%4],[0,1,0xffff][i%3],77]
            vals=[(0x1997+s,1) for s in range(0,36,2)]+[(0x1997+k,0),(0xe54,enemy),(0xf78+enemy,0xd37f),
                  (0xf78+enemy+0x3c,[0,0x8000,0xffff][i%3]),(0x998,40 if i%16==0 else 8),(0x5f5,int(i%16==1))]
            w=[[p,[v&255,v>>8]] for p,v in vals]
            spawn.append(dict(name=f'spawn-{init:04x}-{i}',a=definition,b=parameter,writes=w,host_words=h))
            script=(0x9f7d if h[6] else 0x9f41) if init==0xa009 else (0x9fe1 if parameter else 0x9fb9)
            xx=x if init==0xa009 else (h[2]+h[4])&65535;yy=y if init==0xa009 else(h[3]+h[5])&65535
            values=[(0x1997+k,definition),(0x1a03+k,0xa05b),(0x1b47+k,script),(0x1b8f+k,1),
                    (0x1a4b+k,xx),(0x1a93+k,yy),(0x1aff+k,0x8000 if init==0xa009 else 2048),(0x1b23+k,0 if init==0xa009 else 1)]
            frames.append(dict(name=f'frame-{init:04x}-{i}',writes=[[p,[v&255,v>>8]]for p,v in values]+[[0x198d,[0,128]]],host_words=h))
    flight_host=[1,2,3,4,5,6,7,8]
    for i in range(32):
        k=[0,34][i%2];pre=[0xa05c,0xa07a,0xa0d1,0xa124][i%4]
        values=[(0x1997+k,0xa189 if pre>=0xa0d1 else 0xa17b),(0x1a03+k,pre),(0x1b47+k,0x9754),(0x1b8f+k,2),
                (0x1a4b+k,[0,128,255,0xffff][i%4]),(0x1a93+k,32),(0x1aff+k,[0,32,2048,0x8000][i%4]),
                (0x1b23+k,i%2)]
        frames.append(dict(name=f'flight-{i}',writes=[[p,[v&255,v>>8]]for p,v in values]+[[0x198d,[0,128]]],host_words=flight_host))
    for i,(write,read) in enumerate([(0,0),(6,0),(15,0),(0,1)]):
        values=[(0xe54,0),(0xf78,0xd37f),(0x998,8)]
        w=[[p,[v&255,v>>8]]for p,v in values]+[[0x647,[write]],[0x644,[read]],[0x654,[0xa5]],[0x666+write,[0xff]]]
        spawn.append(dict(name=f'quota-{i}',a=0xa17b,b=0,writes=w,host_words=[100,32,3,4,5,6,0,8]))
    m['family']='pirate-host-context'
    m['routines']=[m['routines'][0],dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=spawn),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames)]
    (ROOT/'tools/port/pirate-host-context.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':main()
