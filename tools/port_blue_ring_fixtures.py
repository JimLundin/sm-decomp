#!/usr/bin/env python3
"""Reproduce actual-ROM Mother Brain blue-ring allocation and lifecycle cases."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def patches(values):
    return [[offset, [value & 255, value >> 8]] for offset, value in values]


def generate():
    rom_path = ROOT / 'rom/sm.sfc'
    rom = rom_path.read_bytes()
    base = dict(bank='sm_86.c', rust_module='crates/port/src/enemy_projectiles.rs',
                c_support=['sm_rtl.c'], projectile_host=True,
                rom=dict(file=str(rom_path), sha256=hashlib.sha256(rom).hexdigest()),
                domain='Even projectile slots 0..34; aligned linked enemy slots 0..0x7c0; '
                       'valid sound cursors; real unmodified ROM. Full WRAM, raw returns '
                       'and all eight external projectile host words compared.')
    control = dict(symbol='EprojInstruction', arg_types=['uint16', 'rom_ptr'],
                   **{'return': 'rom_ptr'}, cases=[dict(name='dispatcher-control', a=0,
                                                       b=0x869754, host_words=[0]*8)])
    host = [11, 22, 33, 44, 55, 66, 77, 88]

    def case(name, values, a=0, b=0, fill=165):
        return dict(name=name, a=a, b=b, fill=fill, writes=patches(values), host_words=host)

    def save(family, routines):
        manifest = dict(base, family=family, routines=[control, *routines])
        (ROOT / f'tools/port/{family}.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
        print(f'{family}: {sum(len(r["cases"]) for r in manifest["routines"])} cases')

    allocation = []
    for free in (0, 2, 16, 34):
        for angle in range(256):
            x, y = ((0, 0), (0xffff, 0xffff), (128, 96), (0x7fff, 0x8000))[angle % 4]
            values = [(0x1997 + k, 0 if k == free else 0xcb4b) for k in range(0, 36, 2)]
            values += [(0xfba, x), (0xfbe, y)]
            # Include nonzero high argument bits: native sine lookup truncates angle.
            allocation.append(case(f'alloc-{free}-{angle}', values, 0xcb4b, angle | 0xa500))
    for free in range(-1, 18):
        values = [(0x1997 + k*2, 0 if k == free else 0xcb4b) for k in range(18)]
        allocation.append(case(f'pool-{free}', values + [(0xfba, 0xffff), (0xfbe, 0xffff)], 0xcb4b, 0xffff))
    save('mother-brain-blue-ring-allocation', [dict(symbol='SpawnEprojWithRoomGfx', args=2,
                                                  **{'return': 'void'}, cases=allocation)])

    updates, room, enemy, tracking, direct_updates, explosions, quakes, clears = [], [], [], [], [], [], [], []

    def state(k, x=128, y=128, linked=0, health=100):
        values = [(0x1997 + s, 0xcb4b) for s in range(0, 36, 2)]
        values += [(0x1991, k), (0x1aff+k, 0), (0x1a4b+k, x), (0x1a93+k, y),
                   (0x1a27+k, 0xff37), (0x1a6f+k, 0x0037), (0x1ab7+k, 0), (0x1adb+k, 0),
                   (0x1bb3+k, 0x0606), (0x1b47+k, 0xc432), (0x1b8f+k, 2),
                   (0x7854, linked), (0x7828, 0xffff), (0xfba, 100), (0xfbe, 100),
                   (0x911, 0), (0x915, 0), (0xaf6, 1000), (0xafa, 1000),
                   (0xafe, 8), (0xb00, 16), (0x9c2, health), (0x9a2, 0), (0xa78, 0),
                   (0x643, 0), (0x645, 0), (0x646, 0), (0x648, 0),
                   (0x5f5, 0), (0x998, 8), (0x592, 0)]
        if linked:
            values += [(0xf7a+linked, x), (0xf7e+linked, y), (0xf82+linked, 8),
                       (0xf84+linked, 8), (0xf8c+linked, health)]
        return values

    for k in range(0, 36, 2):
        for delay in (1, 2, 8, 0xffff):
            values = state(k) + [(0x1aff+k, delay), (0xfba, 0xffff), (0xfbe, 0xfff8)]
            updates.append(case(f'delay-{k}-{delay}', values, k, 0xc335))
        for x, y in ((0,0), (0xffff,0xffff), (0x7fff,0x8000)):
            tracking.append(case(f'track-{k}-{x}', [(0xfba,x), (0xfbe,y)], k))
        clears.append(case(f'clear-{k}', [], k))
        for vx, vy, x, y in ((0,0,128,128), (0x0180,0xff80,128,128),
                              (0x7fff,0x8000,128,128), (0xffff,0xffff,0,32),
                              (0x8000,0x7fff,0xffff,0xffff)):
            values = state(k,x,y) + [(0x1ab7+k,vx), (0x1adb+k,vy)]
            updates.append(case(f'move-{k}-{vx}', values, k, 0xc335))
        for x in (0,247,248,0x7fff,0x8000,0xffff):
            for y in (31,32,215,216,0x8000):
                room.append(case(f'room-{k}-{x}-{y}', state(k,x,y), k))
                updates.append(case(f'room-update-{k}-{x}-{y}', state(k,x,y), k, 0xc335))
        for linked in (64,128,0x7c0):
            for health in (0,1,79,80,81,0x8000,0xffff):
                values = state(k,linked=linked,health=health)
                enemy.append(case(f'enemy-{k}-{linked}-{health}', values, k))
                updates.append(case(f'enemy-update-{k}-{linked}-{health}', values, k, 0xc335))
        for delta in (0,13,14,0x7fff,0x8000,0xffff):
            values = state(k,linked=128) + [(0xf7a+128,(128+delta)&65535), (0xf7e+128,128)]
            enemy.append(case(f'extent-{k}-{delta}', values, k))
        for items in (0,1,32,33):
            for frozen in (0,1):
                values = state(k,health=19) + [(0xaf6,128), (0xafa,128), (0x9a2,items), (0xa78,frozen)]
                updates.append(case(f'samus-{k}-{items}-{frozen}', values, k, 0xc335))
        for distance in (0,13,14,0x7fff,0x8000,0xffff):
            for health in (0,19,20,39,40,79,80,81,0x8000,0xffff):
                values = state(k,health=health) + [(0xaf6,(128+distance)&65535), (0xafa,128)]
                updates.append(case(f'samus-boundary-{k}-{distance}-{health}', values, k, 0xc335))
        for camera in (0,128,0x7fff,0x8000,0xffff):
            for relative in (0,247,248,0xffff):
                values = state(k,x=(camera+relative)&65535) + [(0x911,camera)]
                room.append(case(f'camera-{k}-{camera}-{relative}',values,k))
                updates.append(case(f'camera-update-{k}-{camera}-{relative}',values,k,0xc335))
        # Explosion slot reuse, another higher free slot, and sound quota/gates.
        for free in (-1,0,34):
            for sound_gate in (0,1,6):
                values = state(k,x=248) + [(0x645,0), (0x648,sound_gate)]
                if free >= 0:
                    values.append((0x1997+free,0))
                updates.append(case(f'explosion-{k}-{free}-{sound_gate}', values, k, 0xc335))
                explosions.append(case(f'direct-explosion-{k}-{free}-{sound_gate}', values,k))
                quakes.append(case(f'direct-quake-{k}-{free}-{sound_gate}',values,k))
        for delay in (0,1,8):
            direct_updates.append(case(f'direct-update-{k}-{delay}',state(k)+[(0x1aff+k,delay)],k))
    save('mother-brain-blue-ring-lifecycle', [
        dict(symbol='EprojPreinstruction', args=2, **{'return':'void'}, cases=updates),
        dict(symbol='CheckForBlueRingCollisionWithRoom', args=1, **{'return':'uint8'}, cases=room),
        dict(symbol='CheckForCollisionWithShitroid_DoubleRet', args=1, **{'return':'uint8'}, cases=enemy),
        dict(symbol='sub_86C320', args=1, **{'return':'void'}, cases=tracking),
        dict(symbol='sub_86C42E', args=1, **{'return':'void'}, cases=clears),
        dict(symbol='Eproj_MoveToBlueRingSpawnPosition',args=1,**{'return':'void'},cases=direct_updates),
        dict(symbol='BlueRingContactExplosion',args=1,**{'return':'void'},cases=explosions),
        dict(symbol='Eproj_Earthqhake5',args=1,**{'return':'void'},cases=quakes)])

    frames = []
    for k in range(0,36,2):
        for delay in (0,1,8):
            for timer in (0,1,2,0xffff):
                values = state(k) + [(0x1997+s,0 if s != k else 0xcb4b) for s in range(0,36,2)]
                values += [(0x198d,0x8000), (0x1a03+k,0xc335), (0x1aff+k,delay), (0x1b8f+k,timer)]
                frames.append(case(f'frame-{k}-{delay}-{timer}', values, fill=0))
        for x in (128,248):
            values = state(k,x=x) + [(0x1997+s,0 if s != k else 0xcb4b) for s in range(0,36,2)]
            values += [(0x198d,0x8000), (0x1a03+k,0xc335), (0x1b8f+k,1), (0x5e5,1)]
            frames.append(case(f'frame-contact-{k}-{x}', values, fill=0))
    save('mother-brain-blue-ring-frames', [dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames)])


if __name__ == '__main__':
    generate()
