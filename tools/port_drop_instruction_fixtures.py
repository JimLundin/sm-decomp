#!/usr/bin/env python3
"""Build host-writing drop instructions and same-frame allocation cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def main():
    m=json.loads((ROOT/'tools/port/projectile-host-context.json').read_text());rom=(ROOT/'rom/sm.sfc').read_bytes()
    direct,frames=[],[]
    for opcode in [0xab8a,0x9270,0x8c68,0xdc61]:
        pointer={0xab8a:0xab82,0x9270:0x901b,0x8c68:0x8c60,0xdc61:0xdc54}[opcode]
        assert rom[0x30000+(pointer&0x7fff):0x30002+(pointer&0x7fff)]==opcode.to_bytes(2,'little')
        for i in range(64):
            k=[0,2,16,34][i%4]
            vals=[(0x1997+k,1),(0x1a03+k,0x8170),(0x1b47+k,pointer),(0x1b8f+k,1),
                  (0x1a4b+k,i*997&65535),(0x1a93+k,i*733&65535),(0x79f,i%6),(0x5e5,i*257),
                  (0x9c2,[0,29,30,49,50,99][i%6]),(0x9c4,99),(0xf78+k+0x1e,i*113),(0xf78+k+0x20,i*137)]
            if i&4:
                vals += [(0x1997+s,1) for s in range(0,36,2)]
                if i&8:vals.append((0x1997+(0 if k!=0 else 2),0))
                vals += [(0x1a03+s,0x8170) for s in range(0,36,2) if s!=k]
                vals += [(0x1b47+s,0x9754) for s in range(0,36,2) if s!=k]
                vals += [(0x1b8f+s,2) for s in range(0,36,2) if s!=k]
            w=[[p,[v&255,v>>8]] for p,v in vals];h=[1,2,3,4,5,6,7,8]
            direct.append(dict(name=f'drop-{opcode:04x}-{i}',a=k,b=0x860000|pointer,writes=w,host_words=h))
            # Opcode occurrences below are actual script words, not coincidental code bytes.
            if opcode in [0x9270,0xab8a,0x8c68,0xdc61]:
                frames.append(dict(name=f'frame-{opcode:04x}-{i}',writes=w+[[0x198d,[0,128]]],host_words=h))
    for i in range(12):
        k=[0,2,34][i%3];pointer=0xab82
        values=[(0x1997+k,1),(0x1a03+k,0x8170),(0x1b47+k,pointer),(0x1b8f+k,1),
                (0x1a4b+k,[0x7fff,0x8000,0xffff][i%3]),(0x1a93+k,0xffff),
                (0x5e5,99),(0x79f,i%2),(0x9c2,99),(0x9c4,99),(0x9d6,100),(0x9d4,100),
                (0x9c6,10),(0x9c8,10),(0x9ca,10),(0x9cc,10),(0x9ce,10),(0x9d0,10)]
        w=[[p,[v&255,v>>8]] for p,v in values]
        direct.append(dict(name=f'edge-{i}',a=k,b=0x860000|pointer,writes=w,host_words=h))
        frames.append(dict(name=f'frame-edge-{i}',writes=w+[[0x198d,[0,128]]],host_words=h))
    m['family']='projectile-host-drop-instructions'
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=direct),
                   dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames)]
    (ROOT/'tools/port/projectile-host-drop-instructions.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':main()
