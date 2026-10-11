#!/usr/bin/env python3
"""Ceres platform equality boundary across every projectile slot."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def main():
    m = json.loads((ROOT / 'tools/port/projectile-paired-live-leaves.json').read_text())
    cases = []
    for k in range(0, 36, 2):
        for y in (0, 71, 72, 73, 0x7fff, 0x8000, 0xffff):
            for timer in (0, 1, 0xffff):
                values = [(0xafa, y), (0x1b8f+k, timer), (0x1b47+k, 0xabcd),
                          (0x1b47+(k+2)%36, 0xface), (0x1b8f+(k+2)%36, 0xbeef)]
                cases.append(dict(name=f'platform-{k}-{y}-{timer}', a=k,
                                  writes=[[p, [v&255, v>>8]] for p, v in values]))
    m['family'] = 'ceres-platform'
    m['routines'] = [dict(symbol='EprojPreInstr_CeresElevatorPlatform', args=1,
                          **{'return': 'void'}, cases=cases, live_eligible=True)]
    (ROOT / 'tools/port/ceres-platform.json').write_text(json.dumps(m, separators=(',', ':'))+'\n')


if __name__ == '__main__':
    main()
