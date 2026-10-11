#!/usr/bin/env python3
"""Build real enemy-drop instruction and scheduler transition cases."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def main():
    m = json.loads((ROOT / 'tools/port/enemy-drop-selection.json').read_text())
    direct, frames = [], []
    source = m['routines'][1]['cases']
    for i, case in enumerate(source):
        k = case['a']
        values = [(0x1997+k, 1), (0x1a03+k, 0x8170), (0x1b47+k, 0xecc1),
                  (0x1b8f+k, 1), (0x1bd7+k, [0xffff, 0x4000, 0x1234, 0][i % 4]),
                  (0x1aff+k, 0xa55a), (0x1b23+k, 0x5aa5)]
        w = case['writes'] + [[p, [v & 255, v >> 8]] for p, v in values]
        direct.append({'name': case['name'], 'a': k, 'b': 0x86ecc1, 'writes': w})
        if i % 4 == 0 or i < 256:
            frames.append({'name': 'frame-' + case['name'], 'writes': w + [[0x198d, [0, 128]]]})
    m['family'] = 'enemy-drop-transition'
    m['routines'] = [{'symbol': 'EprojInstruction', 'arg_types': ['uint16', 'rom_ptr'], 'return': 'rom_ptr', 'cases': direct},
                     {'symbol': 'EprojRunAll', 'args': 0, 'return': 'void', 'cases': frames}]
    (ROOT / 'tools/port/enemy-drop-transition.json').write_text(json.dumps(m, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
