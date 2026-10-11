#!/usr/bin/env python3
"""Actual-C full-WRAM shared enemy projectile contact fixtures."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SLOTS = range(0, 36, 2)
DEFINITIONS = [0x9c29, 0x9c37, 0xcb4b, 0xc18c]


def case(name, values, a=0, b=0):
    return dict(name=name, a=a, b=b, fill=165,
                writes=[[p, [v & 255, (v >> 8) & 255]] for p, v in values])


def state():
    values = [(0x1997 + k, 0) for k in SLOTS]
    values += [(0xc18 + j, 0) for j in range(0, 10, 2)]
    values += [(0x18a8, 0), (0xa6e, 0), (0xa78, 0), (0x9a2, 0),
               (0x9c2, 10000), (0xaf6, 128), (0xafa, 128), (0xafe, 8),
               (0xb00, 16), (0xcce, 1), (0x643, 0), (0x645, 0), (0x646, 0),
               (0x648, 0), (0x5f5, 0), (0x998, 8), (0x592, 0)]
    return values


def enemy(k, definition=0xcb4b, properties=0x8014, x=128, y=128, radius=0x0404, flags=0):
    return [(0x1997 + k, definition), (0x1bd7 + k, properties),
            (0x1a4b + k, x), (0x1a93 + k, y), (0x1bb3 + k, radius),
            (0xf380 + k, flags)]


def save(name, symbol, cases, args=0):
    rom_path = ROOT / 'rom/sm.sfc'
    manifest = dict(family=name, bank='sm_a0.c',
        rust_module='crates/port/src/enemy_projectile_contacts.rs',
        c_support=['sm_91.c', 'sm_80.c', 'sm_b4.c', 'sm_rtl.c'],
        rom=dict(file=str(rom_path), sha256=hashlib.sha256(rom_path.read_bytes()).hexdigest()),
        domain='Even enemy projectile slots 0..34; even player projectile slots 0..8; '
               'valid bank86 definition pointers and sound ring cursors 0..15; '
               'actual unmodified ROM. Compare full WRAM and raw returns, including '
               'the intentional projectile-position aliases beyond nominal ten slots.',
        routines=[dict(symbol=symbol, args=args, **{'return': 'void'}, cases=cases)])
    (ROOT / f'tools/port/{name}.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
    print(f'{name}: {len(cases)} cases')


def generate():
    direct, detection = [], []
    damages = [0, 1, 299, 300, 301, 600, 1200, 4095]
    for k in SLOTS:
        for n in range(96):
            amount = damages[n % 8]
            health = [0, 1, 299, 300, 301, 10000, 0x7fff, 0x8000, 0xffff][(n // 8) % 9]
            properties = amount | [0, 0x4000, 0xe000][(n // 32) % 3]
            definition = DEFINITIONS[(n // 4) % 4]
            values = state() + enemy(k, definition, properties, [127, 128, 129, 0x8080][n % 4])
            values += [(0x9a2, [0, 1, 0x20, 0x21][(n // 8) % 4]),
                       (0x9c2, health), (0xa78, 1 if n % 11 == 0 else 0)]
            direct.append(case(f'hit-samus-{k}-{n}', values, k))
        for n in range(128):
            dx = [0, 7, 8, 11, 12, 13, 0x7fff, 0x8000, 0x8001, 0xffff, 0xfff4, 0xfff3,
                  0xfff5, 0x7ffe, 0x8002, 1][n % 16]
            radius = [0, 0x0404, 0x0004, 0x0400, 0xffff, 1, 0x100][n // 16 % 7]
            values = state() + enemy(k, radius=radius,
                properties=0x4014 if n % 7 else 0x2014,
                x=(128 + dx) & 65535 if n % 2 == 0 else 128,
                y=(128 + dx) & 65535 if n % 2 else 128)
            values += [(0xafe, [0, 8, 65535, 32768][n // 32]),
                       (0xb00, [16, 0, 65535, 32768][n // 32])]
            if n % 13 == 0:
                values.append((0x18a8, 1))
            if n % 17 == 0:
                values.append((0xa6e, 1))
            detection.append(case(f'detect-samus-{k}-{n}', values))
    for mask in [0, 1, 2, 0x2aaaa, 0x15555, 0x3ffff]:
        values = state()
        for k in SLOTS:
            if mask & (1 << (k // 2)):
                values += enemy(k, properties=0x4010 + k, x=127 if k % 4 == 0 else 129)
        detection.append(case(f'descending-samus-{mask}', values))
    save('enemy-projectile-hit-samus', 'HandleEprojCollWithSamus', direct, 1)
    save('enemy-projectile-detect-samus', 'EprojSamusCollDetect', detection)

    direct, detection = [], []
    types = [0, 1, 8, 0x101, 0x108, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700, 0x800, 0xffff]
    for k in SLOTS:
        for j in range(0, 10, 2):
            for n in range(20):
                flags = [0, 1, 2, 0xffff][n % 4]
                values = state() + enemy(k, DEFINITIONS[n % 4], 0xfabc, flags=flags)
                values += [(0xc18 + j, types[n % len(types)]), (0xc04 + j, 0x8123),
                           (0xb64 + j, 777), (0xb78 + j, 888)]
                # Write these last: flag1 uses enemy k to select the alias.
                values += [(0xb64 + k, 0x1111 + k), (0xb78 + k, 0x2222 + k)]
                for sprite in range(0, 64, 2):
                    values.append((0xef78 + sprite, 0 if sprite == (n % 3) * 2 else 0x8000))
                if n % 5 == 0:
                    values += [(0xef78 + sprite, 0x8000) for sprite in range(0, 64, 2)]
                cursor = [0, 15, 4, 10][n % 4]
                values += [(0x646, cursor), (0x643, (cursor - n % 8) & 15)]
                if n % 7 == 0:
                    values.append((0x5f5, 1))
                if n % 11 == 0:
                    values.append((0x998, 40))
                direct.append(case(f'hit-projectile-{k}-{j}-{n}', values, k, j))
        for n in range(112):
            flags = [0, 1, 2, 3][n // 28]
            delta = [0, 31, 32, 65535, 1, 0x8000, 0x7fff][n % 7]
            x = [0, 31, 32, 65535][n // 7 % 4]
            values = state() + enemy(k, x=x, y=x, flags=flags,
                properties=0x8014 if n % 9 else 0x14)
            values += [(0xb64, (x + delta) & 65535), (0xb78, x),
                       (0xc18, types[n % len(types)])]
            if n % 19 == 0:
                values.append((0xcce, 0))
            detection.append(case(f'detect-projectiles-{k}-{n}', values))
    for flags in [0, 1, 2, 3]:
        for count in range(6):
            values = state() + enemy(34, flags=flags) + enemy(2, flags=flags)
            values += [(0xb64 + j, 128) for j in range(0, 10, 2)]
            values += [(0xb78 + j, 128) for j in range(0, 10, 2)]
            for j in range(count):
                values += [(0xc18 + j * 2, 0x100 + j * 0x100)]
            values += [(0xef78 + s, 0) for s in range(0, 64, 2)]
            detection.append(case(f'ascending-projectiles-{flags}-{count}', values))
    for k in [0, 20, 34]:
        for write in [0, 1, 14, 15]:
            for queued in [0, 5, 6, 15]:
                for gate in range(4):
                    values = state() + enemy(k, flags=1)
                    values += [(0xc18 + 8, 0x101), (0xb64 + k, 0x8001),
                               (0xb78 + k, 0xffff), (0x646, write),
                               (0x643, (write - queued) & 15), (0x656 + write, 0x80)]
                    if gate == 1:
                        values.append((0x5f5, 1))
                    elif gate == 2:
                        values.append((0x998, 40))
                    elif gate == 3:
                        values.append((0x592, 0x8000))
                    direct.append(case(f'dud-sound-{k}-{write}-{queued}-{gate}', values, k, 8))
    save('enemy-projectile-hit-projectile', 'HandleEprojCollWithProj', direct, 2)
    save('enemy-projectile-detect-projectiles', 'EprojProjCollDet', detection)


if __name__ == '__main__':
    generate()
