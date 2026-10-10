#!/usr/bin/env python3
"""Convert the downloaded snesrev label map to trace's TOML format."""

import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent
symbols = {}
for line in (root / "deps/snesrev-sm/assets/names.txt").read_text().splitlines():
    address, name = line.split(maxsplit=1)
    value = int(address, 16)
    if value >= 0x808000 and value & 0xFFFF >= 0x8000:
        symbols[value] = name
output = root / "out/symbols.toml"
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text("".join('{} = {}\n'.format(json.dumps("{:06X}".format(address)), json.dumps(name))
                          for address, name in sorted(symbols.items())))
print("symbols: {} ROM labels -> {}".format(len(symbols), output.relative_to(root)))
