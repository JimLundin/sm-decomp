#!/usr/bin/env python3
"""Exercise actual-ROM observed projectile initializers and allocation aliases."""

import itertools
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SLOTS = tuple(range(0, 36, 2))
BOUNDARIES = (0, 1, 24, 25, 42, 43, 0x7FFF, 0x8000, 0xFFFF)
PARAMETERS = (0, 1, 2, 255, 256, 0x7FFF, 0x8000, 0xFFFF)
HOST = [0xA55A, 0x5AA5, 11, 22, 33, 44, 55, 66]


def writes(values):
    return [[p, [v & 255, (v >> 8) & 255]] for p, v in values]


def source_values(i):
    enemy = ((i % 31) + 1) * 64
    return [(0xE54, enemy), (0x79F, (0, 2, 5, 0xFFFF)[i % 4]),
            (0x1995, PARAMETERS[(i // 4) % len(PARAMETERS)]),
            (0xF7A, BOUNDARIES[i % len(BOUNDARIES)]),
            (0xF7E, BOUNDARIES[(i // 3) % len(BOUNDARIES)]),
            (0x7832, PARAMETERS[(i // 2) % len(PARAMETERS)]),
            (0x7834, PARAMETERS[(i // 5) % len(PARAMETERS)]),
            (0xF7A + enemy, 0x1234), (0xF7E + enemy, 0x5678),
            (0x7832 + enemy, 0xAAAA), (0x7834 + enemy, 0xBBBB),
            (0xAF6, BOUNDARIES[(i // 4) % len(BOUNDARIES)]),
            (0xAFA, BOUNDARIES[(i // 7) % len(BOUNDARIES)]),
            # These unrelated selectors must not reject the enemy-zero or
            # Samus initializers, nor be read as substitute source indices.
            (0x1C27, 0xFFFF), (0xFA8, 0xFFFF)]


def stale_slot(k, i):
    return [(p + k, (0xA55A + i * 997 + p) & 0xFFFF)
            for p in (0x19BB, 0x19DF, 0x1A03, 0x1A27, 0x1A4B, 0x1A6F,
                      0x1A93, 0x1AB7, 0x1ADB, 0x1AFF, 0x1B23, 0x1B47,
                      0x1B6B, 0x1B8F, 0x1BB3, 0x1BD7, 0x1BFB)]


def case(name, values, **args):
    return dict(name=name, writes=writes(values), host_words=HOST, **args)


def main():
    manifest = json.loads((ROOT / 'tools/port/projectile-host-context.json').read_text())
    manifest.update(family='observed-projectile-initializers',
                    rust_module='crates/port/src/observed_projectile_ownership.rs')
    direct, common, allocation, with_gfx, noop, frames = [], [], [], [], [], []
    for k in SLOTS:
        for i, (area, parameter) in enumerate(itertools.product((0, 2, 5, 0xFFFF), PARAMETERS)):
            values = source_values(i) + stale_slot(k, i)
            values += [(0x79F, area), (0x1993, parameter)]
            direct.append(case(f'ridley-init-{k}-{i}', values, a=k, b=0x93CA))
        for address in (0xA2EE, 0xA31B):
            for i, parameter in enumerate(PARAMETERS + (0,)):
                values = source_values(i * 7) + stale_slot(k, i) + [(0x1993, parameter)]
                direct.append(case(f'ceres-init-{address:x}-{k}-{i}', values, a=k, b=address))
        for i in range(9):
            common.append(case(f'ceres-common-{k}-{i}', source_values(i * 7) + stale_slot(k, i), a=k))
        for definition in (0x9642, 0xA387, 0xA395):
            for i, parameter in enumerate(PARAMETERS * 4):
                values = [(0x1997 + s, 1) for s in SLOTS]
                values += source_values(i) + stale_slot(k, i) + [(0x1997 + k, 0)]
                if i & 1 and k >= 2:
                    values += [(0x1997 + k - 2, 0)]
                allocation.append(case(f'allocate-{definition:x}-{k}-{i}', values, a=definition, b=parameter))
        for definition in (0x9650, 0x965E, 0x966C, 0x967A, 0x9688, 0x9696):
            for i in range(8):
                source = SLOTS[(SLOTS.index(k) + (0, 1, -1, 9)[i % 4]) % len(SLOTS)]
                parameter = source if definition >= 0x966C else PARAMETERS[i]
                values = [(0x1997 + s, 1) for s in SLOTS]
                values += source_values(i) + stale_slot(k, i)
                values += [(0x1A4B + source, BOUNDARIES[i]),
                           (0x1A93 + source, BOUNDARIES[i + 1]),
                           (0x1AFF + source, PARAMETERS[i]), (0x1997 + k, 0)]
                allocation.append(case(f'allocate-{definition:x}-{k}-{i}', values, a=definition, b=parameter))
        for i, timer in enumerate((0, 1, 2, 0xFFFF)):
            values = source_values(i) + stale_slot(k, i)
            noop.append(case(f'noop-950c-{k}-{i}', values, a=k, b=0x950C))
            values += [(0x198D, 0x8000), (0x1997 + k, 0x9650),
                       (0x1A03 + k, 0x950C), (0x1B47 + k, 0x95A0),
                       (0x1B8F + k, timer)]
            frames.append(case(f'noop-frame-{k}-{i}', values))
    for i, (definition, parameter) in enumerate(itertools.product(
            (0x9642, 0xA387, 0xA395, 0x9650, 0x965E, 0x966C, 0x967A, 0x9688, 0x9696, 0xFFFF), PARAMETERS)):
        values = [(0x1997 + s, 1) for s in SLOTS] + source_values(i)
        allocation.append(case(f'saturated-{definition:x}-{i}', values, a=definition, b=parameter))
    # Exercise the genuine native three-word caller through a packed Rust
    # seam. Current enemy, graphics source enemy and Ridley's enemy-zero
    # position/velocity source deliberately differ.
    for k in (*SLOTS, None):
        for enemy in (0, 0x40, 0x7C0):
            for definition in (0x9642, 0xA387, 0xA395):
                for i, parameter in enumerate(PARAMETERS):
                    current = 0x80 if enemy != 0x80 else 0xC0
                    values = [(0x1997 + slot, 1) for slot in SLOTS]
                    values += source_values(i) + [(0xE54, current)]
                    values += [(0xF7A, 0xFFF0), (0xF7E, 0xFFF8),
                               (0x7832, 0x8000), (0x7834, 0xFFFF),
                               (0xF7A + current, 0x1234), (0xF7E + current, 0x5678),
                               (0x7832 + current, 0x1111), (0x7834 + current, 0x2222)]
                    for source, tiles, palette in ((0, 0x123, 0xA00), (0x40, 0x456, 0x1400), (0x7C0, 0x789, 0x1C00)):
                        values += [(0xF96 + source, tiles), (0xF98 + source, palette)]
                    if k is not None:
                        values += stale_slot(k, i) + [(0x1997 + k, 0)]
                        if i & 1 and k >= 2:
                            values += [(0x1997 + k - 2, 0)]
                    name = f'with-gfx-{definition:x}-{enemy}-{k if k is not None else "full"}-{i}'
                    with_gfx.append(case(name, values, a=parameter, b=enemy | (definition << 16)))
    # Observed definitions override gfx during initialization. Actual native
    # B4B1 (Tourian fake wall) retains allocator gfx, making the source OR
    # observable through this same host-aware allocator seam. This bounded
    # offline control does not expand the live nine-definition support set.
    for k in SLOTS:
        for enemy in (0, 0x40, 0x7C0):
            for i, (tiles, palette) in enumerate(((0x123, 0xA00), (0xF0F0, 0x0FF0), (0xFFFF, 0))):
                values = [(0x1997 + slot, 1) for slot in SLOTS]
                values += stale_slot(k, i) + [(0x1997 + k, 0), (0xE54, 0x80)]
                for source in (0, 0x40, 0x7C0, 0x80):
                    values += [(0xF96 + source, 0x1111), (0xF98 + source, 0x2222)]
                values += [(0xF96 + enemy, tiles), (0xF98 + enemy, palette)]
                with_gfx.append(case(f'offline-gfx-or-b4b1-{enemy}-{k}-{i}', values,
                                     a=0xBEEF, b=enemy | (0xB4B1 << 16)))
    manifest['routines'] = [manifest['routines'][0]]
    for symbol, args, cases in [('EprojInitializer', 2, direct), ('sub_86A301', 1, common),
                                ('SpawnEprojWithRoomGfx', 2, allocation),
                                ('EprojPreinstruction', 2, noop), ('EprojRunAll', 0, frames)]:
        manifest['routines'].append(dict(symbol=symbol, args=args, **{'return': 'void'}, cases=cases))
    manifest['routines'].append(dict(symbol='SpawnEprojWithGfx', arg_types=['uint16', 'uint32'],
                                     packed_args='eproj-with-gfx-v1', **{'return': 'void'},
                                     live_eligible=False, cases=with_gfx))
    for routine in manifest['routines']:
        routine['live_eligible'] = False
    path = ROOT / 'tools/port/observed-projectile-initializers.json'
    path.write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
