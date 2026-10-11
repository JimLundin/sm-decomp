#!/usr/bin/env python3
"""Rebuild original-ROM Mother Brain room turret projectile oracle fixtures."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p,[v&255,(v>>8)&255]] for p,v in values]


def main():
    m=json.loads((ROOT/'tools/port/tourian-statue-effects.json').read_text())
    m['family']='mother-brain-turret-projectiles'
    spawn,pre,frames,rotation,rx,ry,ops=[],[],[],[],[],[],[]
    for i in range(144):
        k=[0,2,16,34][i%4]
        vals=[(0x1997+s,1) for s in range(0,36,2)]+[(0x1997+k,0),(0x5e5,i*997)]
        spawn.append(dict(name=f'turret-{i}',a=0xc17e,b=i%12,writes=writes(vals)))
    for i in range(128):
        k=[0,2,16,34][i%4]; source=[0,2,16,34][(i//4)%4]
        vals=[(0x1997+s,1) for s in range(0,36,2)]
        vals += [(0x1997+k,0),(0x1a6f+source,0xff00|i%8),
                 (0x1a4b+source,i*997),(0x1a93+source,i*733)]
        spawn.append(dict(name=f'bullet-{i}',a=0xc18c,b=source+(i%2),writes=writes(vals)))
    for i in range(192):
        k=[0,2,16,34][i%4]; active=[0,2,16,34][(i//4)%4]
        vx=[0,0x100,0xff00,0x2c0,0xfd40,0x7fff,0x8000,0xffff][i%8]
        vy=[0,0x100,0xff00,0x2c0][(i//8)%4]
        vals=[(0x7a5,16),(0x1991,active),(0x1997+k,0xc18c),
              (0x1997+active,0xc18c),(0x1a03+k,0xc0e0),(0x1b47+k,0x9754),(0x1b8f+k,2),
              (0x1a4b+k,512),(0x1a93+k,512),(0x1a27+k,0xffa5),(0x1a6f+k,0xff5a),
              (0x1ab7+k,vx),(0x1adb+k,vy),(0x1bd7+k,i*127)]
        # All non-air tile types count as solid here, including slopes and extensions.
        if i%3:
            vals += [(0x10002+2*(row*16+col),((i%15)+1)<<12)
                     for row in range(23,42) for col in range(23,42)]
        pre.append(dict(name=f'move-{i}',a=k,b=0xc0e0,writes=writes(vals)))
        if active!=k:
            vals += [(0x1a03+active,0x8170),(0x1b47+active,0x9754),(0x1b8f+active,2)]
        frames.append(dict(name=f'frame-{i}',writes=writes(vals+[(0x198d,0x8000)])))
    rom=(ROOT/'rom/sm.sfc').read_bytes()
    for i in range(96):
        ptr=int.from_bytes(rom[0x33ec9+(i%12)*2:0x33ecb+(i%12)*2],'little')
        rotation.append(dict(name=f'rotate-{i}',a=[0,2,16,34][i%4],writes=writes([
            (0x1a27+[0,2,16,34][i%4],ptr),(0x1a6f+[0,2,16,34][i%4],(0x100 if i%2 else 0xff00)|(i%8))])))
    for i in range(256):
        c=dict(name=f'random-{i}',a=[0,2,16,34][i%4],writes=writes([(0x5e5,i*257)]))
        rx.append(dict(c,name=f'random-x-{i}'));ry.append(dict(c,name=f'random-y-{i}'))
    for opcode in [0xc173,0xc1b4]:
        p=rom.index(opcode.to_bytes(2,'little'),0x33f00,0x34200)-0x30000+0x8000
        for i in range(8):
            ops.append(dict(name=f'op-{opcode:x}-{i}',a=0,b=0x860000+p,writes=writes([(0x1aff,i*2),(0x19bb,1024)])))
    m['routines']=[dict(symbol='EprojInstruction',arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},cases=[dict(name='sleep-control',a=0,b=0x869754)]+ops)]
    for name,cases,args in [('SpawnEprojWithRoomGfx',spawn,2),('EprojPreinstruction',pre,2),('EprojRunAll',frames,0),
                            ('Eproj_MotherBrainRoomTurretBullets_Func2',rotation,1),('Eproj_SetXvelRandom',rx,1),('Eproj_SetYvelRandom',ry,1)]:
        m['routines'].append(dict(symbol=name,args=args,**{'return':'void'},cases=cases))
    (ROOT/'tools/port/mother-brain-turret-projectiles.json').write_text(json.dumps(m,separators=(',',':'))+'\n')


if __name__=='__main__':
    main()
