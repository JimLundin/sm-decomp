#!/usr/bin/env python3
"""Rebuild actual-ROM spark initialization, collision, trails and lifecycle cases."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HOST = [0x1234, 0xabcd, 1, 2, 3, 4, 5, 6]
SLOTS = range(0, 36, 2)


def writes(values):
    return [[p, [v & 255, (v >> 8) & 255]] for p, v in values]


def case(name, values=(), **kwargs):
    return dict(name=name, fill=0, writes=writes(values), host_words=HOST, **kwargs)


def routine(symbol, cases, args=0):
    return dict(symbol=symbol, args=args, **{'return': 'void'}, cases=cases)


def save(name, routines):
    rom = ROOT / 'rom/sm.sfc'
    control = dict(symbol='EprojInstruction', arg_types=['uint16', 'rom_ptr'],
                   **{'return': 'rom_ptr'}, cases=[case('native-control', a=0, b=0x86f38f)])
    manifest = dict(family=name, bank='sm_86.c', c_support=['sm_rtl.c'],
                    rust_module='crates/port/src/enemy_projectiles.rs', projectile_host=True,
                    rom=dict(file=str(rom), sha256=hashlib.sha256(rom.read_bytes()).hexdigest()),
                    domain='Even projectile slots 0..34; aligned enemy slots <=0x7c0; '
                           'actual unchanged ROM including velocity pair at F3F0; bounded '
                           '64-block-wide terrain; valid sprite pool. Full WRAM, returns and '
                           'unchanged external host words compared; controlled offline frames.',
                    routines=[control, *routines])
    (ROOT / f'tools/port/{name}.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
    print(f'{name}: {sum(len(r["cases"]) for r in manifest["routines"])} cases')


def initial(enemy, seed):
    return [(0xe54, enemy), (0xf7a+enemy, 0xffff), (0xf7c+enemy, 0xffa5),
            (0xf7e+enemy, 0xfffc), (0xf80+enemy, 0x7f5a), (0x5e5, seed)]


def state(k, downward=True, solid=False, nmi=0, free=-1):
    values = [(0x1991, k), (0x1997+k, 0xf498), (0x1a03+k, 0xf3f0),
              (0x1b47+k, 0xf353), (0x1b8f+k, 2), (0x1bb3+k, 0x1010),
              (0x1a4b+k, 128), (0x1a27+k, 0xfe37), (0x1a93+k, 128),
              (0x1a6f+k, 0xfe73), (0x1ab7+k, 0x8000),
              (0x1adb+k, 0 if downward else 0xffff), (0x1b23+k, 0xffff),
              (0x1aff+k, 0xb800), (0x19df+k, 0xa6b5), (0x19bb+k, 0xa5b6), (0x7a5, 64), (0x5b4, nmi << 8)]
    values += [(0xef78+s, 0 if s == free else 0xbddc) for s in range(0, 64, 2)]
    if solid:
        # A complete solid row catches the swept vertical probe over the radius.
        values += [(0x10002+(8*64+x)*2, 0x8000) for x in range(64)]
    return values


def rebuild():
    allocate, initialize = [], []
    for free in range(-1, 18):
        for seed in range(8):
            enemy = [0, 0x40, 0x400, 0x7c0][seed % 4]
            values = [(0x1997+s, 0 if s // 2 == free else 0xf498) for s in SLOTS]
            values += initial(enemy, seed)
            allocate.append(case(f'allocate-{free}-{seed}', values, a=0xf498, b=0xffff))
    for k in SLOTS:
        for seed in [*range(32), 0xff, 0x100, 0xff00, 0xffff]:
            enemy = [0, 0x40, 0x400, 0x7c0][(seed+k) % 4]
            values = initial(enemy, seed) + [(0x1aff+k, 0xdead), (0x1b23+k, 0xbeef),
                (0x1ab7+k, 0x7fff), (0x1adb+k, 0x8000), (0x1b47+k, 0x9754)]
            initialize.append(case(f'initialize-{k}-{seed}', values, a=k, b=0xf391))
    save('sparks-allocation', [routine('SpawnEprojWithRoomGfx', allocate, 2),
                              routine('EprojInitializer', initialize, 2)])

    update, direct = [], []
    # Signed upward phases skip terrain and preserve all full-word fractions.
    for k in SLOTS:
        for i in range(32):
            values = state(k, downward=False, nmi=i % 4, free=[-1, 0, 2, 62][i % 4])
            values += [(0x1a4b+k, [0, 0xffff, 0x7fff, 0x8000][i % 4]),
                       (0x1a93+k, [0, 0xffff, 0x7fff, 0x8000][(i // 4) % 4]),
                       (0x1ab7+k, [0, 1, 0x7fff, 0x8000, 0xbfff, 0xc000, 0xfffe, 0xffff][i % 8]),
                       (0x1adb+k, [0xffff, 0xfffe, 0x8000, 0x8001][(i // 8) % 4]),
                       (0x1b23+k, [0, 1, 0xffff, 0x8000][i % 4]),
                       (0x1aff+k, [0, 1, 0x8000, 0xffff][(i // 4) % 4])]
            update.append(case(f'upward-{k}-{i}', values, a=k, b=0xf3f0))
        # The terrain helper uses the high vertical velocity as signed 8.8,
        # independently of the subsequent full16.16 gravity addition.
        for high in [0, 1, 3, 4, 255, 256, 0x7fff]:
            for low in [0, 0xbfff, 0xc000, 0xffff]:
                values = state(k, nmi=low % 4, free=62) + [(0x1adb+k, high), (0x1ab7+k, low)]
                update.append(case(f'downward-{k}-{high}-{low}', values, a=k, b=0xf3f0))
        for high, low in [(0xffff, 0xffff), (0xffff, 0xb800), (0, 0x4000),
                          (0x7fff, 0xffff), (0x8000, 0x8000), (0xdbbd, 0x301a)]:
            for nmi in range(4):
                values = state(k, solid=True, nmi=nmi, free=62)
                values += [(0x1b23+k, high), (0x1aff+k, low)]
                update.append(case(f'bounce-{k}-{high}-{low}-{nmi}', values, a=k, b=0xf3f0))
                if nmi == 0:
                    direct.append(case(f'native-bounce-{k}-{high}-{low}', values, a=k))
    # Slopes and extension chains exercise the shared collision seam from this
    # caller, including the helper's final alignment before the native bounce.
    for k in [0, 2, 16, 34]:
        for bts in [0, 1, 2, 3, 4, 5, 18, 31, 0x40, 0x45, 0x80, 0xc5]:
            values = state(k, nmi=0, free=62)
            for block in range(64):
                values += [(0x10002+(8*64+block)*2, 0x1000)]
                values += [(0x16402+8*64+block, bts)]
            # BTS is a byte array: patch exactly one byte, avoiding overlap
            # with neighbouring table values as a word writer would do.
            c = case(f'slope-{k}-{bts}', values, a=k, b=0xf3f0)
            c['writes'] = [[p, v[:1] if p >= 0x16402 else v] for p, v in c['writes']]
            update.append(c)
        for direction in [-1, 1]:
            values = state(k, nmi=0, free=62)
            values += [(0x10002+(8*64+7)*2, 0x5000),
                       (0x10002+(8*64+8)*2, 0x5000),
                       (0x10002+(8*64+7+direction)*2, 0x8000),
                       (0x10002+(8*64+8+direction)*2, 0x8000)]
            c = case(f'extension-{k}-{direction}', values, a=k, b=0xf3f0)
            c['writes'] += [[0x16402+8*64+block, [direction & 255]] for block in [7, 8]]
            update.append(c)
    save('sparks-update', [routine('EprojPreinstruction', update, 2),
                          routine('EprojPreInstr_Sparks', direct, 1)])

    frames = []
    for k in SLOTS:
        for i in range(32):
            values = state(k, downward=i % 2 == 0, solid=i % 4 == 0,
                           nmi=(i // 2) % 4, free=[-1, 0, 2, 62][(i // 8) % 4])
            values += [(0x198d, 0x8000), (0x1b8f+k, [0, 1, 2, 0xffff][(i // 8) % 4])]
            frames.append(case(f'frame-{k}-{i}', values))
    save('sparks-frames', [routine('SpawnEprojWithRoomGfx', [case('sequence-allocation-control',
        initial(0, 0), a=0xf498, b=0)], 2), routine('EprojRunAll', frames)])


def sequence(source_run):
    """Follow actual C initialization through gravity, landing and native deletion."""
    source_run = source_run.resolve()
    output = ROOT / 'out/port/sparks-sequence-inputs'
    output.mkdir(parents=True, exist_ok=True)
    seed = bytearray(0x20000)
    values = [(0x198d, 0x8000), (0xe54, 0), (0xf7a, 128), (0xf7c, 0xfe37),
              (0xf7e, 120), (0xf80, 0xfe73), (0x5e5, 0), (0x7a5, 64)]
    # Ground at y=160; all other state flows from the preceding actual C output.
    values += [(0x10002+(10*64+x)*2, 0x8000) for x in range(64)]
    for offset, data in writes(values):
        seed[offset:offset+2] = bytes(data)
    host = output / 'host.bin'
    host.write_bytes(b''.join(v.to_bytes(2, 'little') for v in HOST))
    cases = []

    def capture(name, symbol, state, a=0, b=0):
        before, after = output / f'{name}-before.wram', output / f'{name}-after.wram'
        before.write_bytes(state)
        result = subprocess.run([str(source_run / 'c'), symbol, str(a), str(b), str(before), str(after),
                                 str(source_run / 'rom.bin'), str(host), str(output / 'host-after.bin')],
                                check=True, capture_output=True, timeout=30)
        c = dict(name=name, before_file=str(before), host_words=HOST,
                 before_sha256=hashlib.sha256(before.read_bytes()).hexdigest(),
                 expected_after_file=str(after), expected_after_sha256=hashlib.sha256(after.read_bytes()).hexdigest(),
                 expected_return=int(result.stdout), a=a, b=b)
        return c, bytearray(after.read_bytes())

    allocated, state = capture('sequence-allocate', 'SpawnEprojWithRoomGfx', seed, 0xf498, 0)
    for frame in range(96):
        state[0x5b5] = frame & 255
        c, state = capture(f'sequence-{frame:03d}', 'EprojRunAll', state)
        cases.append(c)
    save('sparks-sequence', [routine('SpawnEprojWithRoomGfx', [allocated], 2),
                            routine('EprojRunAll', cases)])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sequence-source', type=Path)
    args = parser.parse_args()
    sequence(args.sequence_source) if args.sequence_source else rebuild()
