#!/usr/bin/env python3
"""Build explicit external projectile host-state allocation oracle cases."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def main():
    m=json.loads((ROOT/'tools/port/enemy-drop-selection.json').read_text())
    m.update(family='projectile-host-context',rust_module='crates/port/src/enemy_projectiles.rs',projectile_host=True)
    rom=(ROOT/'rom/sm.sfc').read_bytes()
    cases=[]
    for init in [0x947f,0xe468,0xef29]:
        definition=rom.index(init.to_bytes(2,'little'),0x30000,0x38000)-0x30000+0x8000
        for i in range(256):
            k=[0,2,16,34][i%4]
            writes=[[0x1997+s,[1,0]] for s in range(0,36,2)]
            if i%16!=15:writes.append([0x1997+k,[0,0]])
            writes += [[0x5e5,[i,i]], [0x9c2,[i,0]], [0x9c4,[99,0]]]
            cases.append(dict(name=f'host-{init:04x}-{i}',a=definition,b=i%30 if init==0xe468 else i*257,
                              writes=writes,host_words=[[0,1,0x7fff,0x8000,0xffff,i*997&65535][i%6],
                              [0,1,0x7fff,0x8000,0xffff,i*733&65535][(i//6)%6],11,22,33,44,0xa55a,
                              [0,0xd37f,0xd3bf,0xf593][i%4]]))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='control',a=0,b=0x869754,host_words=[0]*8)]),
                   dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=cases)]
    (ROOT/'tools/port/projectile-host-context.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':main()
