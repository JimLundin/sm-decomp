#!/usr/bin/env python3
"""Build the paired movement live contract from its offline case matrix."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/projectile-paired-motion.json').read_text())
    manifest['family'] = 'projectile-paired-live-leaves'
    for routine in manifest['routines']:
        routine['live_eligible'] = routine['symbol'] != 'EprojInstruction'
    (ROOT / 'tools/port/projectile-paired-live-leaves.json').write_text(
        json.dumps(manifest, separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
