#!/usr/bin/env python3
"""Native-ROM enemy projectile OAM passes and direct rendering leaf coverage."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def patches(values):
    return [[offset, [value & 255, (value >> 8) & 255]] for offset, value in values]


def generate():
    rom_path = ROOT / 'rom/sm.sfc'
    rom = rom_path.read_bytes()
    base = dict(bank='sm_86.c', rust_module='crates/port/src/enemy_projectile_rendering.rs',
                c_support=['sm_81.c', 'sm_rtl.c'],
                rom=dict(file=str(rom_path), sha256=hashlib.sha256(rom).hexdigest()),
                domain='Actual unmodified ROM; complete WRAM and raw returns. '
                       'Priority passes traverse eighteen even projectile slots; direct '
                       'slot calls additionally cover odd k floored by C. Native bank8D '
                       'spritemaps, all OAM cursor alignments and metadata/WRAM aliases; '
                       'camera-relative X/Y span signed visibility and wrapping boundaries. '
                       'Earthquake types include 0..36 and signed/wrapped table-index cases.')
    maps = [0x8000, 0x8007, 0x8023, 0x802a, 0x8040, 0x8056, 0x806c]
    for pointer in maps:
        count = int.from_bytes(rom[0x68000 + (pointer & 0x7fff):0x68002 + (pointer & 0x7fff)], 'little')
        if count > 64:
            raise ValueError(f'fixture pointer {pointer:04x} is not a bounded native spritemap')

    def case(name, values, a=0, b=0, fill=165, **extra):
        return dict(name=name, a=a, b=b, fill=fill, writes=patches(values), **extra)

    def save(family, routines):
        manifest = dict(base, family=family, routines=routines)
        if family.endswith('-direct'):
            manifest['projectile_rendering'] = True
        count = sum(len(r['cases']) for r in routines)
        if count > 3000:
            raise ValueError('each oracle run must remain at most 3000 cases')
        (ROOT / f'tools/port/{family}.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
        print(f'{family}: {count} cases')

    def pool(active=(), low=False, x=100, y=100, cursor=0):
        values = [(0x590, cursor), (0x183e, 0), (0x1840, 0), (0xa78, 0), (0x911, 0), (0x915, 0)]
        for k in range(0, 36, 2):
            values += [(0x1997+k, 1 if k in active else 0), (0x1bd7+k, 0x1000 if low else 0),
                       (0x1b6b+k, maps[(k//2) % len(maps)]), (0x19bb+k, 0x2400 + k),
                       (0x1a4b+k, x), (0x1a93+k, y)]
        return values

    points = [(x,100) for x in (0xff7f,0xff80,0xffff,0,1,255,256,383,384,0x7fff,0x8000)]
    points += [(100,y) for y in (0xff7f,0xff80,0xfff8,0xffff,0,1,127,128,248,255,256,383,384,0x7fff,0x8000)]
    passes = {True: [], False: []}
    for low, cases in passes.items():
        for k in range(0,36,2):
            for n,(x,y) in enumerate(points):
                camera = (0,1,0xffff)[n % 3]
                values = pool([k],low,x=(x+camera)&65535,y=(y+camera)&65535,
                              cursor=(0,4,28,252,508,509,510,511,512,543)[n%10])
                values += [(0x911,camera),(0x915,camera),(0x1b6b+k,maps[(n+k//2)%len(maps)]),
                           (0x19bb+k,(0,0xff,0x0100,0x7fff,0xffff)[n%5])]
                cases.append(case(f'{"low" if low else "high"}-slot-{k}-{n}',values))
        for kind in range(37):
            for timer in (0,1,2,3,0x8000,0xffff):
                values = pool(range(0,36,2),low,x=128,y=128,cursor=508)
                values += [(0x183e,kind),(0x1840,timer),(0xa78,0)]
                cases.append(case(f'{"low" if low else "high"}-quake-{kind}-{timer}',values,fill=255))
        for n in range(32):
            values = pool(range(0,36,2),low,cursor=(0,508,511,512,543,0xffff)[n%6])
            for k in range(0,36,2):
                values += [(0x1bd7+k,0x1000 if (k//2+n)%2 else 0),
                           (0x1a4b+k,(k*15+n-128)&65535),(0x1a93+k,(k*11+n-128)&65535)]
            values += [(0x183e,n%36),(0x1840,1+(n%3)),(0xa78,(0,1,0xffff)[n%3])]
            cases.append(case(f'{"low" if low else "high"}-mixed-{n}',values,fill=(0,165,255)[n%3]))
        # Empty and mismatched priorities never read an invalid map. Clipped
        # slots also return before dereferencing it. Freeze inhibits ROM quake.
        cases.append(case(f'{"low" if low else "high"}-empty',pool([],low)))
        values=pool(range(0,36,2),not low)
        for k in range(0,36,2):
            values += [(0x1b6b+k,0)]
        cases.append(case(f'{"low" if low else "high"}-other-priority',values))
    save('enemy-projectile-rendering',[
        dict(symbol='DrawLowPriorityEprojs',args=0,**{'return':'void'},cases=passes[True]),
        dict(symbol='DrawHighPriorityEprojs',args=0,**{'return':'void'},cases=passes[False])])

    shakes=[]
    for kind in [*range(37),0x7fff,0x8000,0x8023,0x8024,0xc000,0xffff]:
        for timer in (0,1,2,3,0x8000,0xffff):
            for frozen in (0,1,0xffff):
                x=y=0
                if timer and not frozen and ((kind-36)&65535) >= 0x8000:
                    offset=0x3046b+((kind*4)&65535)
                    x=int.from_bytes(rom[offset:offset+2],'little')
                    y=int.from_bytes(rom[offset+2:offset+4],'little')
                    if timer & 2:
                        x,y=(-x)&65535,(-y)&65535
                shakes.append(case(f'shake-{kind}-{timer}-{frozen}',[(0x183e,kind),(0x1840,timer),(0xa78,frozen)],
                                   expected_return=x|(y<<16)))
    direct=[]
    for k in range(36):
        for n,(x,y) in enumerate(points):
            shake=(0,0xffff,0x8000)[n%3]
            values=pool([],x=x,y=y,cursor=(0,508,511,512)[n%4])
            index=k&~1
            values += [(0x1a4b+index,(x-shake)&65535),(0x1a93+index,(y-shake)&65535),
                       (0x1b6b+index,maps[n%len(maps)]),(0x19bb+index,(0,0xff,0x7fff,0xffff)[n%4])]
            direct.append(case(f'direct-{k}-{n}',values,k,shake|(shake<<16)))
    # Both clipping branches must short-circuit before invalid ROM access.
    direct += [case('direct-invalid-map-horizontal-clip',pool([],x=384)+[(0x1b6b,0)]),
               case('direct-invalid-map-vertical-clip',pool([],y=384)+[(0x1b6b,0)])]
    leaves={False:[],True:[]}
    for offscreen,cases in leaves.items():
        for pointer in maps:
            for y in (0,1,3,127,128,248,251,252,255):
                for cursor in (0,508,511,512,543,0xffff):
                    gfx=(0,0xff,0x7fff,0xffff)[(y+cursor)%4]
                    x=(0xff80,0xffff,0,255,256,383)[cursor%6]
                    cases.append(case(f'leaf-{int(offscreen)}-{pointer}-{y}-{cursor}',[(0x590,cursor)],
                                      x|(y<<16),pointer|(gfx<<16),fill=(0,165,255)[y%3]))
    save('enemy-projectile-rendering-direct',[
        dict(symbol='PortScreenShake',args=0,**{'return':'uint32'},cases=shakes),
        dict(symbol='PortDrawEprojs',arg_types=['uint16','uint32'],**{'return':'void'},cases=direct),
        dict(symbol='PortDrawEprojSpritemap',arg_types=['uint32','uint32'],**{'return':'void'},cases=leaves[False]),
        dict(symbol='PortDrawEprojSpritemapOffscreen',arg_types=['uint32','uint32'],**{'return':'void'},cases=leaves[True])])


if __name__ == '__main__':
    generate()
