#!/usr/bin/env python3
"""Rebuild native Maridia tube allocation, motion, script and PLM callers."""
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
    return dict(name=name, fill=165, writes=writes(values), **kwargs)


def routine(symbol, cases, args=0, pointer=False):
    return dict(symbol=symbol, cases=cases, **(
        dict(arg_types=['uint16', 'rom_ptr'], **{'return': 'rom_ptr'}) if pointer
        else dict(args=args, **{'return': 'void'})))


def write_manifest(name, routines, host=True):
    manifest = json.loads((ROOT / 'tools/port/mother-brain-tube-motion.json').read_text())
    manifest['family'] = name
    manifest['routines'] = routines
    manifest['rust_module'] = 'crates/port/src/enemy_projectiles.rs' if host else 'crates/port/src/plm_instructions.rs'
    if host:
        manifest['projectile_host'] = True
        for r in routines:
            for c in r['cases']:
                c['host_words'] = HOST
    else:
        manifest.pop('projectile_host', None)
        manifest['bank'] = 'sm_84.c'
    (ROOT / f'tools/port/{name}.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


def rebuild():
    allocation, initializers, pre, scripts, frames = [], [], [], [], []
    for definition, init, count in [(0xd904, 0xd6a5, 1), (0xd912, 0xd6c9, 20), (0xd920, 0xd774, 12)]:
        for parameter in range(count):
            for k in [0, 16, 34]:
                for width, block in [(1, 0), (16, 0x7ffe), (255, 0xffff)]:
                    plm = [0, 38, 78][k // 16]
                    values = [(0x1997 + s, 0xd904) for s in SLOTS]
                    values += [(0x1997 + k, 0), (0x1c27, plm), (0x7a5, width),
                               (0x1c87 + plm, block), (0x1c29, 0xdead), (0x1c2b, 0xbeef)]
                    allocation.append(case(f'allocate-{definition:x}-{parameter}-{k}-{width}', values,
                                           a=definition, b=parameter))
                    initializers.append(case(f'initialize-{init:x}-{parameter}-{k}-{width}',
                        values + [(0x1993, parameter), (0x1aff + k, 0x1234)], a=k, b=init))
        allocation.append(case(f'saturated-{definition:x}', [(0x1997 + s, 1) for s in SLOTS],
                               a=definition, b=0xffff))
    velocities = [0, 1, 255, 256, 0xffff, 0xff00, 0x8000, 0x7fff]
    for address in [0xd7bf, 0xd7de, 0xd7fd, 0xd83d, 0xd89f, 0xd8df]:
        for i in range(128 if address in [0xd83d, 0xd89f] else 64):
            k = [0, 2, 16, 34][i % 4]
            active = [34, 16, 2, 0][i % 4] if address in [0xd83d, 0xd89f] else k
            y, camera = [(287, 0), (288, 0), (289, 0), (0xffff, 0),
                         (0x8000, 0), (0x7fff, 0), (127, 128), (384, 128)][i % 8]
            values = [(0x1991, active), (0x5b6, i), (0x915, camera)]
            for s in {k, active}:
                values += [(0x1997 + s, 0xd912), (0x1a4b + s, [0xee00, 0xffff, 0, 256][i % 4]),
                           (0x1b23 + s, [0, 0xffff, 0x8000, 255][i % 4]),
                           (0x1aff + s, [0, 1, 0xff00, 0xffff][i % 4]),
                           (0x1a93 + s, y), (0x1a6f + s, [0, 1, 0xff00, 0xffff][i % 4]),
                           (0x1ab7 + s, (i * 2) if address in [0xd83d, 0xd89f] else velocities[i % 8]),
                           (0x1adb + s, velocities[(i // 8) % 8])]
            # Distinct phase source and destination velocity make global aliasing observable.
            if active != k:
                values.append((0x1ab7 + active, 0xfffe))
            pre.append(case(f'callback-{address:x}-{i}', values, a=k, b=address))
    for pc in [0xd51b, 0xd501, 0xd5a1, 0xd666]:
        for i in range(64):
            k = [0, 2, 16, 34][i % 4]
            values = [(0x5e5, [0, 1, 0xffff, 0x8000, 0x7fff, 0xff00, 0x00ff, 0x1234][i % 8]),
                      (0x5e7, (i * 997) & 0xffff), (0x5b6, i),
                      (0x1b23 + k, [0, 0xffff, 0x8000, 256][(i // 4) % 4])]
            scripts.append(case(f'opcode-{pc:x}-{i}', values, a=k, b=0x860000 | pc))
    for pc, callback in [(0xd4c5, 0xd7fd), (0xd501, 0xd7fd), (0xd51b, 0xd7fd),
                         (0xd5a1, 0xd7fd), (0xd652, 0xd8df), (0xd666, 0xd8df),
                         (0xd670, 0xd89f)]:
        for i in range(16):
            k = [0, 2, 16, 34][i % 4]
            values = [(0x1997 + s, 0) for s in SLOTS]
            values += [(0x198d, 0x8000), (0x1997 + k, 0xd912), (0x1a03 + k, callback),
                       (0x1b47 + k, pc), (0x1b8f + k, 1 if i % 2 else 2),
                       (0x1a4b + k, 128), (0x1b23 + k, 128), (0x1aff + k, 0xff00),
                       (0x1a93 + k, 96), (0x1a6f + k, 0xff00), (0x1ab7 + k, 0xff00),
                       (0x1adb + k, 192), (0x915, 0), (0x5b6, i), (0x5e5, i)]
            frames.append(case(f'scheduled-{pc:x}-{i}', values))
    write_manifest('n00b-tube', [routine('SpawnEprojWithRoomGfx', allocation, 2),
                               routine('EprojInitializer', initializers, 2),
                               routine('EprojPreinstruction', pre, 2),
                               routine('EprojInstruction', scripts, pointer=True),
                               routine('EprojRunAll', frames)])
    callers = []
    for pc in [0xd4f6, 0xd507, 0xd509, 0xd513]:
        for free in range(19):
            for plm in [0, 78]:
                values = [(0x1997 + s, 0 if s // 2 < free else 0xd904) for s in SLOTS]
                values += [(0x1c27, plm), (0x1c87 + plm, 0x1234), (0x7a5, 32),
                           (0x183e, 0xffff), (0x1840, 0xffff), (0x197e, 0xffff)]
                callers.append(case(f'caller-{pc:x}-{free}-{plm}', values, a=78-plm, b=0x840000 | pc))
    write_manifest('n00b-tube-callers', [routine('PlmInstruction', callers, pointer=True)], host=False)


def sequence(source_run, caller_run):
    """Capture an actual C sixteen-effect PLM spawn through 96 scheduled frames."""
    source_run, caller_run = source_run.resolve(), caller_run.resolve()
    initial = caller_run / 'caller-d507-18-0/c-after.wram'
    output = ROOT / 'out/port/n00b-tube-sequence-inputs'
    output.mkdir(parents=True, exist_ok=True)
    state = bytearray(initial.read_bytes())
    state[0x198d:0x198f] = (0x8000).to_bytes(2, 'little')
    state[0x915:0x917] = (1024).to_bytes(2, 'little')
    cases = []
    host = output / 'host.bin'
    host.write_bytes(b''.join(v.to_bytes(2, 'little') for v in HOST))
    for frame in range(96):
        state[0x5b6:0x5b8] = frame.to_bytes(2, 'little')
        before, after = output / f'{frame:03d}-before.wram', output / f'{frame:03d}-after.wram'
        before.write_bytes(state)
        result = subprocess.run([str(source_run / 'c'), 'EprojRunAll', '0', '0', str(before),
                                 str(after), str(source_run / 'rom.bin'), str(host), str(output / 'host-after.bin')],
                                check=True, capture_output=True, timeout=30)
        cases.append(dict(name=f'sequence-{frame:03d}', before_file=str(before),
                          before_sha256=hashlib.sha256(before.read_bytes()).hexdigest(),
                          expected_after_file=str(after), expected_after_sha256=hashlib.sha256(after.read_bytes()).hexdigest(),
                          expected_return=int(result.stdout)))
        state = bytearray(after.read_bytes())
    write_manifest('n00b-tube-sequence', [routine('EprojInstruction', [dict(name='sequence-native-control',
        fill=0, a=0, b=0x86d51b)], pointer=True), routine('EprojRunAll', cases)])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sequence-source', type=Path)
    parser.add_argument('--caller-source', type=Path)
    args = parser.parse_args()
    if args.sequence_source:
        if not args.caller_source:
            parser.error('--sequence-source requires --caller-source')
        sequence(args.sequence_source, args.caller_source)
    else:
        rebuild()
