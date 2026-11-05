#!/usr/bin/env python3
"""Compare a headless replay's digests with a row's oracle lock entry (Gate 1).

gate1_check.py <row> <entry.toml>: each machine stream that differs, or none.
"""
import sys
import tomllib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import scenario  # noqa: E402

MACHINE = ("wram", "sram", "vram", "cgram", "oam", "ppu", "video", "apu")


def main() -> int:
    row, entry = sys.argv[1], sys.argv[2]
    locked = scenario.read_lock(Path(".")).entries[row]
    with open(entry, "rb") as file:
        run = tomllib.load(file)
    if locked["ticks"] != run["ticks"]:
        print(f"{row}: {run['ticks']} ticks, the lock has {locked['ticks']}")
        return 1
    differing = [name for name in MACHINE if locked["digests"][name] != run["digests"][name]]
    for name in MACHINE:
        state = "DIFFERS" if name in differing else "matches"
        print(f"{row} {name}: {state}")
    return 1 if differing else 0


if __name__ == "__main__":
    sys.exit(main())
