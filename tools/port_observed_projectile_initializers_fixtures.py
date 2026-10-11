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
    manifest['family'] = 'observed-projectile-initializers'
    direct, common, allocation, noop, frames = [], [], [], [], []
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
    manifest['routines'] = [manifest['routines'][0]]
    for symbol, args, cases in [('EprojInitializer', 2, direct), ('sub_86A301', 1, common),
                                ('SpawnEprojWithRoomGfx', 2, allocation),
                                ('EprojPreinstruction', 2, noop), ('EprojRunAll', 0, frames)]:
        manifest['routines'].append(dict(symbol=symbol, args=args, **{'return': 'void'}, cases=cases))
    path = ROOT / 'tools/port/observed-projectile-initializers.json'
    path.write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
