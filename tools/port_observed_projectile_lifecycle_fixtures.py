#!/usr/bin/env python3
"""Controlled native-ROM allocation/frame chains, with inherited eight-word host."""

import itertools
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SLOTS = range(0, 36, 2)
HOST = [0xA55A, 0x5AA5, 11, 22, 33, 44, 55, 66]


def writes(values):
    return [[p, [v & 255, (v >> 8) & 255]] for p, v in values]


def native_slot(slot, definition):
    rom = (ROOT / 'rom/sm.sfc').read_bytes()
    offset = 0x30000 + definition - 0x8000
    word = lambda n: int.from_bytes(rom[offset + n:offset + n + 2], 'little')
    return [(0x1997 + slot, definition), (0x1A03 + slot, word(2)),
            (0x1B47 + slot, word(4)), (0x1BB3 + slot, word(6)),
            (0x1BD7 + slot, word(8)), (0x1B8F + slot, 1)]


def generate_boundaries(template):
    manifest = dict(template, family='observed-projectile-frame-boundaries')
    manifest.pop('case_order')
    cases = []
    combinations = itertools.product((0, 34), (0x9642, 0x966C, 0x9688),
                                     (0x8000, 0xFF00, 0, 0xFF, 0x100, 0x7FFF),
                                     (256, 512, 0x7FFF, 0xFFF0), (0, 1, 0x7FFF, 0xFFFF))
    for i, (slot, projectile, velocity, position, fraction) in enumerate(combinations):
        values = native_slot(slot, projectile)
        values += [(0x198D, 0x8000), (0x7A5, 16),
                   (0x1A4B + slot, position), (0x1A93 + slot, position),
                   (0x1A27 + slot, fraction), (0x1A6F + slot, fraction),
                   (0x1AB7 + slot, velocity), (0x1ADB + slot, velocity),
                   (0x1AFF + slot, 3), (0x1B23 + slot, projectile if projectile != 0x9642 else 1)]
        cases.append(dict(name=f'boundary-{i}', host_words=[11, 22, 33, 44, 55, 66, 77, 88],
                          writes=writes(values)))
    manifest['routines'] = [template['routines'][0],
                            dict(symbol='EprojRunAll', args=0, **{'return': 'void'},
                                 live_eligible=False, cases=cases)]
    (ROOT / 'tools/port/observed-projectile-frame-boundaries.json').write_text(
        json.dumps(manifest, separators=(',', ':')) + '\n')


def main():
    manifest = json.loads((ROOT / 'tools/port/projectile-host-context.json').read_text())
    manifest.update(family='observed-projectile-ownership',
                    rust_module='crates/port/src/observed_projectile_ownership.rs')
    manifest['routines'][0]['live_eligible'] = False
    allocations, with_gfx, frames = [], [], []
    for axis in ('horizontal', 'vertical'):
        for start in (0, 16, 34):
            for suppression in (0, 1):
                prefix = f'{axis}-{start}-{suppression}'
                values = [(0x198D, 0x8000), (0x7A5, 16), (0x7A7, 16),
                          (0xF7A, 103), (0xF7E, 171), (0x1995, suppression),
                          (0x7832, 0x300 if axis == 'horizontal' else 0),
                          (0x7834, 0x300 if axis == 'vertical' else 0),
                          (0x79F, 2 if start == 16 else 5 if start == 34 else 0)]
                # Fill higher slots with native stationary explosions; they
                # vacate while lower newborn slots run in descending order.
                for slot in SLOTS:
                    if slot > start:
                        values += native_slot(slot, 0x9650)
                        values += [(0x1A4B + slot, 128), (0x1A93 + slot, 128),
                                   (0x1B47 + slot, 0x95B8)]
                # Real solid terrain words, perpendicular to travel. Script
                # bytes, instruction lists and spritemaps remain native ROM.
                for row in range(16):
                    for col in range(16):
                        if (axis == 'horizontal' and col == 10) or (axis == 'vertical' and row == 10):
                            values.append((0x10002 + 2 * (row * 16 + col), 0x8000))
                name = prefix + '-allocate'
                allocations.append(dict(name=name, a=0x9642, b=1,
                                        writes=writes(values), host_words=HOST))
                previous = name
                for frame in range(110):
                    name = f'{prefix}-frame-{frame}'
                    config = dict(name=name, continue_from=previous)
                    if frame == 80:
                        # Reuse the latest impact hostpoint after full native
                        # lifecycle frames, without resetting the host context.
                        config.update(a=0x965E if axis == 'horizontal' else 0x9650, b=0)
                        allocations.append(config)
                    else:
                        frames.append(config)
                    previous = name
    # Ceres common initialization and descending release transitions from
    # actual native scripts. Height updates are explicit controlled inputs.
    for height in (0, 71, 72, 73, 0x7FFF, 0x8000, 0xFFFF):
        previous = f'ceres-{height}-pad'
        with_gfx.append(dict(name=previous, a=0, b=0x7C0 | (0xA387 << 16), host_words=HOST,
                             writes=writes([(0x198D, 0x8000), (0xAF6, 128), (0xAFA, height),
                                            (0xE54, 0x40), (0xF96 + 0x7C0, 0x321),
                                            (0xF98 + 0x7C0, 0x1400)])))
        name = f'ceres-{height}-platform'
        with_gfx.append(dict(name=name, a=0, b=0x40 | (0xA395 << 16), continue_from=previous,
                             writes=writes([(0xF96 + 0x40, 0x456), (0xF98 + 0x40, 0xA00)])))
        previous = name
        for frame in range(70):
            name = f'ceres-{height}-frame-{frame}'
            frames.append(dict(name=name, continue_from=previous))
            previous = name
    for enable in (0, 1, 0x7FFF):
        frames.append(dict(name=f'disabled-{enable}', host_words=HOST,
                           writes=writes([(0x198D, enable), (0x1997, 0xFFFF),
                                          (0x1A03, 0xFFFF), (0x1B47, 0xFFFF)])))
    # Cases are ordered globally: a continuation can switch routine. The
    # verifier executes the manifest's explicit order below.
    by_name = {case['name']: (symbol, case) for symbol, cases in
               [('SpawnEprojWithRoomGfx', allocations), ('SpawnEprojWithGfx', with_gfx), ('EprojRunAll', frames)] for case in cases}
    ordered, visited = [], set()

    def visit(name):
        if name in visited:
            return
        symbol, case = by_name[name]
        if 'continue_from' in case:
            visit(case['continue_from'])
        visited.add(name)
        ordered.append(name)

    for name in by_name:
        visit(name)
    manifest['case_order'] = ordered + ['control']
    manifest['routines'] = [manifest['routines'][0],
                            dict(symbol='SpawnEprojWithRoomGfx', args=2, **{'return': 'void'},
                                 live_eligible=True, cases=allocations),
                            dict(symbol='SpawnEprojWithGfx', arg_types=['uint16', 'uint32'],
                                 packed_args='eproj-with-gfx-v1', **{'return': 'void'},
                                 live_eligible=True, cases=with_gfx),
                            dict(symbol='EprojRunAll', args=0, **{'return': 'void'},
                                 live_eligible=True, cases=frames)]
    (ROOT / 'tools/port/observed-projectile-ownership.json').write_text(
        json.dumps(manifest, separators=(',', ':')) + '\n')
    generate_boundaries(manifest)


if __name__ == '__main__':
    main()
