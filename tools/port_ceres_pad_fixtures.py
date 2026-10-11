#!/usr/bin/env python3
"""Exercise Ceres pad timer, signed height and Samus release boundaries."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/ceres-platform.json').read_text())
    cases = []
    for slot in range(0, 36, 2):
        for timer in (0, 1, 2, 60, 0x7fff, 0x8000, 0x8001, 0xffff):
            for height in (0, 71, 72, 73, 0x7fff, 0x8000, 0x8047, 0x8048, 0xffff):
                values = [(0xafa, height), (0x1aff + slot, timer),
                          (0x1a93 + slot, 0xface), (0x1b47 + slot, 0xabcd),
                          (0x1b8f + slot, 0xbeef), (0xa42, 0x1234), (0xa44, 0x5678)]
                cases.append(dict(name=f'pad-{slot}-{timer}-{height}', a=slot,
                                  writes=[[p, [v & 255, v >> 8]] for p, v in values]))
    manifest['family'] = 'ceres-pad'
    manifest['routines'] = [dict(symbol='EprojPreInstr_CeresElevatorPad',
                                 args=1, **{'return': 'void'}, cases=cases,
                                 live_eligible=True)]
    (ROOT / 'tools/port/ceres-pad.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
