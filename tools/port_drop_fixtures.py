#!/usr/bin/env python3
"""Build actual-ROM enemy-drop hysteresis and inventory oracle cases."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/botwoon-spit-scheduling.json').read_text())
    manifest['family'] = 'enemy-drop-selection'
    cases = []
    headers = [0, 0xd37f, 0xd3bf, 0xd3ff, 0xe0ff, 0xf593]
    for header in headers:
        for i in range(256):
            k = [0, 2, 16, 34][i % 4]
            health = [0, 29, 30, 49, 50, 99, 0xffff, 0xffe0][i % 8]
            values = [(0xf3c8 + k, header), (0x5e5, i * 257), (0xe1a, [0, 1, 0x100, 0x101][i % 4]),
                      (0x9c2, health), (0x9d6, [0, 1, 30, 100][(i // 8) % 4]),
                      (0x9c4, 99), (0x9d4, 100)]
            for j, (current, maximum) in enumerate([(0x9c6, 0x9c8), (0x9ca, 0x9cc), (0x9ce, 0x9d0)]):
                values += [(current, 10 if i & (1 << (j + 2)) else 9), (maximum, 10)]
            cases.append({'name': f'drop-{header:04x}-{i}', 'a': k,
                          'writes': [[p, [v & 255, v >> 8]] for p, v in values]})
    for seed in range(65536):
        low = (seed & 255) * 5
        carry = (low >> 8) + (((seed >> 8) * 5) & 255) + 1
        next_seed = (((low & 255) | ((carry & 255) << 8)) + (carry >> 8) + 17) & 65535
        if next_seed & 255 == 0:
            cases.append({'name': f'zero-retry-{seed}', 'a': 0,
                          'writes': [[0xf3c8, [0x7f, 0xd3]], [0x5e5, [seed & 255, seed >> 8]]]})
    manifest['routines'] = [manifest['routines'][0], {'symbol': 'RandomDropRoutine', 'args': 1, 'return': 'uint16', 'cases': cases}]
    (ROOT / 'tools/port/enemy-drop-selection.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
