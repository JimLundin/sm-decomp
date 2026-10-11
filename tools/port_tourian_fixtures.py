#!/usr/bin/env python3
"""Rebuild original-ROM Tourian scheduling fixtures for the dispatcher oracle."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def writes(values):
    return [[p, [v & 255, (v >> 8) & 255]] for p, v in values]


def main():
    template = json.loads((ROOT / 'tools/port/tourian-statue-effects.json').read_text())
    template['family'] = 'tourian-statue-scheduling'
    pre, frames, spawn, instructions = [], [], [], []
    for address in [0xb977, 0xb982, 0xb9fd, 0xba37, 0xba42]:
        for i in range(72):
            k = [0, 2, 16, 34][i % 4]
            y = [0, 1, 255, 256, 511, 512, 767, 768, 0xffff][i % 9]
            velocity = [0, 1, 255, 256, 0xff00, 0xffff, 0x7fff, 0x8000][i // 9]
            values = [(0x1997+k, 0xba78), (0x1a03+k, address),
                      (0x1b47+k, 0x9754), (0x1b8f+k, 2),
                      (0x1a4b+k, 0xffff-i), (0x1a27+k, 0xffa5),
                      (0x1a93+k, y), (0x1a6f+k, 0xff5a),
                      (0x1ab7+k, velocity ^ 0xffff), (0x1adb+k, velocity),
                      (0x195e, y + (i % 3) - 1), (0x1aff+k, i*997),
                      (0x1b23+k, i*733), (0x915, 0xffff-i),
                      (0x9e00, i*1231), (0x1e6d, 0x1234), (0x1e6f, i % 2)]
            pre.append(dict(name=f'pre-{address:04x}-{i}', a=k, b=address, writes=writes(values)))
            if i < 18:
                frames.append(dict(name=f'frame-{address:04x}-{i}',
                                   writes=writes(values+[(0x198d, 0x8000)])))
    # Crossing allocation: self reuse, lower/higher destination, exhausted pool.
    for target in [0, 16, 34, None]:
        for frame in [False, True]:
            k = 16
            values = []
            for s in range(0, 36, 2):
                values += [(0x1997+s, 1), (0x1a03+s, 0x8170),
                           (0x1b47+s, 0x9754), (0x1b8f+s, 2)]
            if target is not None:
                values += [(0x1997+target, 0)]
            values += [(0x1a03+k, 0xb982), (0x1a93+k, 99),
                       (0x1adb+k, 0x200), (0x195e, 100), (0x198d, 0x8000)]
            # Self reuse is legal for direct preinstruction, but inactive in run_all.
            if frame and target == k:
                continue
            case = dict(name=f'cross-{target}-{frame}', writes=writes(values))
            if frame:
                frames.append(case)
            else:
                case.update(a=k, b=0xb982)
                pre.append(case)
    rom = (ROOT / 'rom/sm.sfc').read_bytes()
    defs = [p for p in range(0xba5c, 0xbb1e, 14)
            if int.from_bytes(rom[0x30000+(p&0x7fff):0x30002+(p&0x7fff)], 'little')
            in [0xb87a, 0xb93e, 0xb951, 0xb964]]
    for definition in defs:
        for i in range(18):
            k = i*2
            spawn.append(dict(name=f'spawn-{definition:04x}-{i}', a=definition, b=k,
                              writes=writes([(0x1a4b+k, i*1777), (0x195e, i*4093)])))
    # Find real ROM opcode locations, passing the opcode pointer expected by the dispatcher.
    for opcode in [0xb7ea, 0xb818]:
        p = rom.index(opcode.to_bytes(2, 'little'), 0x33700, 0x33900)-0x30000+0x8000
        for k in [0, 2, 16, 34]:
            instructions.append(dict(name=f'op-{opcode:04x}-{k}', a=k, b=0x860000+p,
                                     writes=writes([(0x1a4b+k, 100), (0x1a93+k, 90), (0x5e5, 0xface)])))
    template['routines'] = [
        dict(symbol='EprojInstruction', arg_types=['uint16', 'rom_ptr'], return_='rom_ptr',
             cases=[dict(name='sleep-control', a=0, b=0x869754)]+instructions),
        dict(symbol='EprojPreinstruction', args=2, return_='void', cases=pre),
        dict(symbol='EprojRunAll', args=0, return_='void', cases=frames),
        dict(symbol='SpawnEprojWithRoomGfx', args=2, return_='void', cases=spawn)]
    for routine in template['routines']:
        routine['return'] = routine.pop('return_')
    (ROOT / 'tools/port/tourian-statue-scheduling.json').write_text(json.dumps(template, indent=2)+'\n')


if __name__ == '__main__':
    main()
