#!/usr/bin/env python3
"""Actual-ROM Mother Brain bomb allocation, bounce, collision and frame fixtures."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def patches(values):
    return [[offset, [value & 255, value >> 8]] for offset, value in values]


def generate():
    rom_path = ROOT / 'rom/sm.sfc'
    base = dict(bank='sm_86.c', rust_module='crates/port/src/enemy_projectiles.rs',
                c_support=['sm_rtl.c'], projectile_host=True,
                rom=dict(file=str(rom_path), sha256=hashlib.sha256(rom_path.read_bytes()).hexdigest()),
                domain='Even projectile slots 0..34; bomb acceleration F>>1 below ten; '
                       'valid sound cursors and inventory; actual unmodified ROM. '
                       'Complete WRAM, raw returns and eight external host words compared.')
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

    def state(k, x=128, y=128, stage=0):
        values = [(0x1997 + s, 0xcb59) for s in range(0, 36, 2)]
        values += [(0x1991, k), (0x1a4b+k, x), (0x1a93+k, y),
                   (0x1a27+k, 0xfe37), (0x1a6f+k, 0xfe73), (0x1ab7+k, 224),
                   (0x1adb+k, 256), (0x1aff+k, 112), (0x1b23+k, stage),
                   (0x1bb3+k, 0x0606), (0x1b47+k, 0xc76e), (0x1b8f+k, 2),
                   (0x784a, 1), (0xcd2, 0), (0x643, 0), (0x645, 0), (0x646, 0),
                   (0x648, 0), (0x5f5, 0), (0x998, 8), (0x592, 0), (0x5e5, 1),
                   (0x5b4, 0x1234), (0x5b6, 0), (0x9c2, 100), (0x9c4, 100),
                   (0x9c6, 5), (0x9c8, 5), (0x9ca, 5), (0x9cc, 5),
                   (0x9ce, 5), (0x9d0, 5), (0x9d4, 0), (0x9d6, 0)]
        for bomb in range(10, 20, 2):
            values += [(0xc18+bomb, 0), (0xc7c+bomb, 0), (0xb64+bomb, x),
                       (0xb78+bomb, y), (0xbb4+bomb, 8), (0xbc8+bomb, 8)]
        return values

    allocation, initializers = [], []
    for free in range(-1, 18):
        for parameter in (0, 1, 255, 256, 0xffff):
            for count in (0, 1, 0xffff):
                values = [(0x1997 + s*2, 0 if s == free else 0xcb59) for s in range(18)]
                values += [(0xfba, 0xfff8), (0xfbe, 0xfff0), (0x784a, count)]
                allocation.append(case(f'allocate-{free}-{parameter}-{count}', values, 0xcb59, parameter))
    for k in range(0, 36, 2):
        for parameter in (0, 1, 255, 256, 0xffff):
            for scratch in (0, 0xa555, 0xffff):
                values = [(0x1993, parameter), (0x1a27+k, scratch),
                          (0xfba, 0xfff8), (0xfbe, 0xfff0), (0x784a, 0xffff)]
                initializers.append(case(f'initialize-{k}-{parameter}-{scratch}', values, k, 0xc482))
    save('mother-brain-bomb-allocation', [dict(symbol='SpawnEprojWithRoomGfx', args=2,
                                            **{'return': 'void'}, cases=allocation),
                                        dict(symbol='EprojInitializer', args=2,
                                             **{'return': 'void'}, cases=initializers)])

    movement, updates, collisions, direct = [], [], [], []
    for k in range(0, 36, 2):
        for x, y in ((0, 0), (239, 207), (240, 208), (241, 209), (0x7fff, 0x8000),
                     (0x80ef, 0x80cf), (0x80f0, 0x80d0), (0xffff, 0xffff)):
            for vx, vy, acceleration in ((0, 0, 7), (224, 256, 16), (0xff20, 0xfe00, 0x170),
                                         (0x7fff, 0x8000, 0xffff), (0x8000, 0x7fff, 1)):
                values = state(k, x, y) + [(0x1ab7+k, vx), (0x1adb+k, vy)]
                movement.append(case(f'move-{k}-{x}-{y}-{vx}', values, k, acceleration))
        for stage in range(20):
            for y in (100, 207, 208):
                values = state(k, 239, y, stage)
                updates.append(case(f'stage-{k}-{stage}-{y}', values, k, 0xc4c8))
        for vx in (0, 1, 2, 3, 224, 0x7fff, 0x8000, 0x8001, 0xfffd, 0xfffe, 0xffff):
            values = state(k) + [(0x1ab7+k, vx)]
            updates.append(case(f'damping-{k}-{vx}', values, k, 0xc4c8))
        # Zero-width rectangles, inclusive center and exclusive far-edge checks.
        for radius in (0, 1, 0x0606, 0xffff):
            for delta in (0, 7, 8, 13, 14, 0x7fff, 0x8000, 0xffff):
                for axis in (0, 1):
                    values = state(k) + [(0xcd2, 1), (0xc18+10, 0xa500),
                                       (0x1bb3+k, radius),
                                       ((0xb64 if axis == 0 else 0xb78)+10, (128+delta)&65535)]
                    collisions.append(case(f'extent-{k}-{radius}-{delta}-{axis}', values, k))
                    updates.append(case(f'collision-update-{k}-{radius}-{delta}-{axis}', values, k, 0xc4c8))
        for bomb in range(10, 20, 2):
            for bomb_type, variable, counter in ((0x500, 0, 1), (0x500, 1, 1),
                                                  (0x400, 0, 1), (0x1500, 0, 1),
                                                  (0x500, 0, 0), (0x500, 0, 0xffff)):
                values = state(k) + [(0xcd2, counter), (0xc18+bomb, bomb_type),
                                    (0xc7c+bomb, variable)]
                collisions.append(case(f'bomb-{k}-{bomb}-{bomb_type}-{variable}-{counter}', values, k))
        for free in (-1, 0, 2, 34):
            for count in (0, 1, 0xffff):
                for sound_gate in (0, 1, 6):
                    values = state(k, stage=18) + [(0x784a, count), (0x648, sound_gate)]
                    if free >= 0:
                        values.append((0x1997+free, 0))
                    updates.append(case(f'final-{k}-{free}-{count}-{sound_gate}', values, k, 0xc4c8))
                    values += [(0xcd2, 1), (0xc18+10, 0x500)]
                    collisions.append(case(f'drop-{k}-{free}-{count}-{sound_gate}', values, k))
        for stage in (0, 1, 2, 16, 18, 19):
            direct.append(case(f'native-update-{k}-{stage}', state(k, 128, 208, stage), k))
    save('mother-brain-bomb-movement', [
        dict(symbol='MoveMotherBrainBomb', args=2, **{'return': 'uint8'}, cases=movement)])
    save('mother-brain-bomb-collision', [
        dict(symbol='MotherBrainBomb_Bomb_CollDetect_DoubleRet', args=1, **{'return': 'uint8'}, cases=collisions)])
    save('mother-brain-bomb-update', [
        dict(symbol='EprojPreinstruction', args=2, **{'return': 'void'}, cases=updates),
        dict(symbol='EprojPreInstr_MotherBrainBomb', args=1, **{'return': 'void'}, cases=direct)])

    frames = []
    for k in range(0, 36, 2):
        for stage in (0, 2, 16, 18):
            for timer in (0, 1, 2, 0xffff):
                values = state(k, 239, 208, stage)
                values += [(0x1997+s, 0 if s != k else 0xcb59) for s in range(0, 36, 2)]
                values += [(0x198d, 0x8000), (0x1a03+k, 0xc4c8), (0x1b8f+k, timer)]
                frames.append(case(f'frame-{k}-{stage}-{timer}', values, fill=0))
        for bomb in (10, 18):
            values = state(k) + [(0x1997+s, 0 if s != k else 0xcb59) for s in range(0, 36, 2)]
            values += [(0x198d, 0x8000), (0x1a03+k, 0xc4c8), (0x1b8f+k, 1),
                       (0xcd2, 1), (0xc18+bomb, 0x500)]
            frames.append(case(f'frame-bomb-{k}-{bomb}', values, fill=0))
    save('mother-brain-bomb-frames', [dict(symbol='EprojRunAll', args=0,
                                        **{'return': 'void'}, cases=frames)])


if __name__ == '__main__':
    generate()
