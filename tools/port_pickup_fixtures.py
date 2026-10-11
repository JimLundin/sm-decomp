#!/usr/bin/env python3
"""Build pickup expiry, grapple, collision, restore and sound-refusal cases."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def main():
    m = json.loads((ROOT/'tools/port/enemy-drop-selection.json').read_text())
    pre, frames = [], []
    for kind in range(1,6):
        for i in range(256):
            k=[0,2,16,34][i%4]
            lifetime=[1,0,384,385,400,2,0x8000,0xffff][i%8]
            dx=[0,15,16,19,20,21,0x8000,0xffff][(i//8)%8]
            dy=[0,15,16,21][(i//32)%4]
            value=[0,98,99,100,0x7fff,0x8000,0xfffe,0xffff][i%8]
            vals=[(0x1aff+k,kind*2),(0x1b23+k,lifetime),(0x1a03+k,0xefe0),(0x1b47+k,0x9754),(0x1b8f+k,2),(0x1997+k,1),
                  (0x1a4b+k,100),(0x1a93+k,100),(0x1bb3+k,0x0404),(0xaf6,100+dx),(0xafa,100+dy),(0xafe,16),(0xb00,16),
                  (0xd08,100+dx),(0xd0c,100+dy),(0xd32,0xc4f0 if i&1 else 0),(0x9c2,value),(0x9c4,99),(0x9d6,value),(0x9d4,100),
                  (0x9c6,value),(0x9c8,[10,99,100,0x8000][i%4]),(0x9d8,value),(0x9ca,value),(0x9cc,10),(0x9ce,value),(0x9d0,10),
                  (0x9c0,[0,1,2,0xffff][i%4]),(0x998,40 if i%16==0 else 8),(0x5f5,int(i%16==1)),(0x592,0x8000 if i%16==2 else 0)]
            w=[[p,[v&255,(v>>8)&255]] for p,v in vals]+[[0x647,[i%16]],[0x644,[(i//16)%16]],[0x654,[0xa5]]]
            pre.append({'name':f'pickup-{kind}-{i}','a':k,'b':0xefe0,'writes':w})
            frames.append({'name':f'frame-{kind}-{i}','writes':w+[[0x198d,[0,128]]]})
    def targeted(name, changes):
        base = pre[0]['writes']
        values = {p: bytes(v) for p,v in base}
        for p,v in changes:
            values[p] = bytes([v&255,(v>>8)&255])
        w = [[p,list(v)] for p,v in values.items()]
        pre.append({'name':name,'a':0,'b':0xefe0,'writes':w})
    for life in [385,384]:
        for axis in [0,1]:
            for distance in [15,16,0x8000]:
                targeted(f'grapple-{life}-{axis}-{distance}',[(0x1b23,life),(0xd32,0),(0xaf6,1000),(0xafa,1000),
                         (0xd08,100+(distance if axis==0 else 0)),(0xd0c,100+(distance if axis==1 else 0))])
    for axis in [0,1]:
        for distance in [19,20,0x8000]:
            targeted(f'collision-{axis}-{distance}',[(0x1b23,400),(0xd32,0xc4f0),
                     (0xaf6,100+(distance if axis==0 else 0)),(0xafa,100+(distance if axis==1 else 0))])
    for kind in range(1,6):
        for maximum in [98,99,100]:
            for delta in [-1,0,1]:
                for gate in [0,1,2,3,4]:
                    targeted(f'restore-{kind}-{maximum}-{delta}-{gate}',[(0x1aff,kind*2),(0x1b23,400),
                             (0xaf6,100),(0xafa,100),(0x9c2,maximum+delta),(0x9c4,maximum),
                             (0x9c6,maximum+delta),(0x9c8,maximum),(0x9ca,maximum+delta),(0x9cc,maximum),
                             (0x9ce,maximum+delta),(0x9d0,maximum),(0x998,40 if gate==1 else 8),
                             (0x5f5,int(gate==2)),(0x592,0x8000 if gate==3 else 0)])
                    pre[-1]['writes'] += [[0x647,[1 if gate==4 else 0]],[0x644,[0]]]
    m['family']='projectile-pickup-collection'
    m['rust_dependencies']=list(set(m.get('rust_dependencies',[])+['crates/port/src/audio.rs']))
    m['routines']=[m['routines'][0],{'symbol':'EprojPreinstruction','args':2,'return':'void','cases':pre},
                   {'symbol':'EprojRunAll','args':0,'return':'void','cases':frames}]
    (ROOT/'tools/port/projectile-pickup-collection.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
