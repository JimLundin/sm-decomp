#!/usr/bin/env python3
"""Rebuild Shaktool attack direction, collision and linked-circle oracle cases."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text())
    m['family']='shaktool-attack-circles'
    pre,frames,spawn=[],[],[]
    for definition in [0xbe25,0xbe33,0xbe41]:
        for angle in range(256):
            k=[0,2,16,34][angle%4]
            enemy=(angle%32)*64
            vals=[(0x1997+s,1) for s in range(0,36,2)]
            vals += [(0x1997+k,0),(0xe54,enemy),(0xf78+enemy+0x36,0xa500|angle),
                     (0xf78+enemy+2,(angle*997)&65535),(0xf78+enemy+6,(angle*733)&65535)]
            spawn.append(dict(name=f'spawn-{definition:x}-{angle}',a=definition,b=[0,3,16,35][angle%4],writes=writes(vals)))
    for address in [0xbe03,0xbe12]:
        for i in range(96):
            k=[0,2,16,34][i%4]
            linked=[0,3,16,35][(i//4)%4]
            vx=[0,0x100,0xff00,0x400,0xfc00,0x7f,0xff81,0x200][i%8]
            vy=[0x200,0xfe00,0x100,0xff00][(i//8)%4]
            vals=[(0x7a5,16),(0x1991,k),(0x1997+(linked&~1),i%2),
                  (0x1997+k,0xbe25),(0x1aff+k,linked),(0x1a03+k,address),
                  (0x1b47+k,0x9754),(0x1b8f+k,2),(0x1bb3+k,0x404),
                  (0x1a4b+k,29),(0x1a93+k,29),(0x1a27+k,0xffa5),(0x1a6f+k,0xff5a),
                  (0x1ab7+k,vx),(0x1adb+k,vy)]
            # Optional solid right wall and floor, with blank room elsewhere.
            if (i//16)%3 != 0:
                vals += [(0x10002+2*(row*16+2),0x8000) for row in range(16)]
            if (i//16)%3 == 2:
                vals += [(0x10002+2*(2*16+col),0x8000) for col in range(16)]
            pre.append(dict(name=f'pre-{address:x}-{i}',a=k,b=address,writes=writes(vals)))
            other=linked&~1
            if other!=k:
                vals += [(0x1a03+other,0x8170),(0x1b47+other,0x9754),(0x1b8f+other,2)]
            frames.append(dict(name=f'frame-{address:x}-{i}',writes=writes(vals+[(0x198d,0x8000)])))
    for script in [0xbd68,0xbd78,0xbd8c]:
        for timer in [1,2]:
            vals=[(0x7a5,16),(0x198d,0x8000),(0x1997,0xbe33),
                  (0x1a03,0x84fb),(0x1b47,script),(0x1b8f,timer),
                  (0x1a4b,64),(0x1a93,64),(0x1bb3,0x404)]
            frames.append(dict(name=f'real-script-{script:x}-{timer}',writes=writes(vals)))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)]),
                   dict(symbol='EprojPreinstruction',args=2,**{'return':'void'},cases=pre),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames),
                   dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=spawn)]
    (ROOT/'tools/port/shaktool-attack-circles.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
