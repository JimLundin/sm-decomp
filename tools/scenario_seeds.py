#!/usr/bin/env python3
"""Write scenario seeds: recorded room seeds with declared state edits.

A scenario names a recorded original-emulator room seed and the work-RAM
state to change in it: Samus's energy, ammunition, items and beams, her
position, or any other RAM word (a boss phase, for example). The tool copies
the seed's regions and registers into target/seeds/scenario-NAME/, applies the
edits to its WRAM, and records the new region hashes in that capture's
rooms.csv. The verifier then loads the scenario seed into both engines like
any other seed, so the original game decides what follows from the edited
state; a corpus row uses it as `seed = "scenario-NAME:INDEX"` with the
seed_sha256 this tool prints.

Edits stay within the seed's room: the room, its level data and its video
memory come from the recording. The gameplay state, room and NMI words and
the latched controller words the seed loader checks cannot be edited.
Scenario seeds are ROM-derived like their sources; they stay local and are
published only in encrypted bundles.

    python tools/scenario_seeds.py write draygon-repel
    python tools/scenario_seeds.py show draygon-repel
    python tools/scenario_seeds.py check
"""
import argparse
import csv
import hashlib
import re
import shutil
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGIONS = (("wram", 131072), ("sram", 8192), ("vram", 65536), ("cgram", 512),
           ("oam", 544), ("apuram", 65536))

# Named work-RAM words ($7E bank offsets).
FIELDS = {
    "energy": 0x09C2, "max_energy": 0x09C4,
    "reserve_energy": 0x09D6, "max_reserve_energy": 0x09D4, "reserve_mode": 0x09C0,
    "missiles": 0x09C6, "max_missiles": 0x09C8,
    "super_missiles": 0x09CA, "max_super_missiles": 0x09CC,
    "power_bombs": 0x09CE, "max_power_bombs": 0x09D0,
    "equipped_items": 0x09A2, "collected_items": 0x09A4,
    "equipped_beams": 0x09A6, "collected_beams": 0x09A8,
    "selected_item": 0x09D2,
    "x": 0x0AF6, "y": 0x0AFA,
}
# Item and beam bits, so recipes can name equipment.
ITEMS = {"varia": 0x0001, "spring-ball": 0x0002, "morph-ball": 0x0004, "screw-attack": 0x0008,
         "gravity": 0x0020, "hi-jump": 0x0100, "space-jump": 0x0200, "bombs": 0x1000,
         "speed-booster": 0x2000, "grapple": 0x4000, "x-ray": 0x8000}
BEAMS = {"wave": 0x0001, "ice": 0x0002, "spazer": 0x0004, "plasma": 0x0008, "charge": 0x1000}
# The HUD's selectable items, in selection order.
SELECTABLE = {"none": 0, "missiles": 1, "super-missiles": 2, "power-bombs": 3,
              "grapple": 4, "x-ray": 5}
# Words the seed loader checks against the recorded registers or input.
PROTECTED = {0x0998, 0x079B, 0x05B6, 0x05D1, *range(0x0087, 0x008B)}


@dataclass
class Scenario:
    base: str
    edits: dict = field(default_factory=dict)
    # Raw work-RAM words: {offset: value}.
    words: dict = field(default_factory=dict)
    note: str = ""


SCENARIOS = {
    # The 100% movie's Draygon fight with full energy and reserves, so Samus
    # survives his whole grab. The grapple-repel recipe reselects the
    # grapple before firing it, since the movie words change the item.
    "draygon-repel": Scenario(
        base="100-boundary:259",
        edits={"energy": 1099, "reserve_energy": 300, "selected_item": "grapple"},
        note="Draygon grab with the grapple ready"),
    # The 100% movie's landing site late in the game, with energy and
    # ammunition set for a crystal flash: under 51 energy, empty reserves and
    # at least ten of each kind of ammunition once the bomb is laid.
    # The 100% movie's Crocomire room with one energy and no reserves: the
    # movie words now play out as on hardware, where Samus is never killed, so
    # the death row takes its first hit as fatal.
    "one-energy-crocomire": Scenario(
        base="100-input-phase:142",
        edits={"energy": 1, "reserve_energy": 0},
        note="death at the first hit in Crocomire's room"),
    "crystal-flash": Scenario(
        base="100-input-phase:305",
        edits={"energy": 40, "reserve_energy": 0, "power_bombs": 11},
        note="crystal flash conditions at the landing site"),
    # The 100% movie's Oum room with Samus moved beside the first Oum, which
    # the route never approaches.
    "oum-roll": Scenario(
        base="100-input-phase:256",
        edits={"x": 540, "y": 590},
        note="Samus beside the first Oum"),
    # The 100% movie's Tatori room, where the movie arrives with one energy,
    # with full energy so Samus survives the turtles' touch.
    "tatori-touch": Scenario(
        base="100-input-phase:221",
        edits={"energy": 999},
        note="Tatori room with full energy"),
    # The 100% movie's Ridley fight with full energy and reserves and Ridley's
    # health (enemy slot 0) down to 9000, where he swoops, which the movie's
    # quick fight never sees.
    "ridley-swoop": Scenario(
        base="100-boundary:118",
        edits={"energy": 699, "reserve_energy": 200},
        words={0x0F8C: 9000},
        note="Ridley fight at half health with full energy"),
    # The 100% movie's Bowling Alley after Phantoon, with Samus moved onto the
    # Chozo statue's hand, which the movie passes.
    "wrecked-ship-hand": Scenario(
        base="100-input-phase:211",
        edits={"x": 1192, "y": 347},
        note="Samus on the Wrecked Ship Chozo hand"),
    # The 100% movie's Crateria kihunter room with only the power beam, Samus
    # moved to the top corridor beside the left kihunter, so her shots wound
    # it without killing it and its wings fall.
    "kihunter-wings": Scenario(
        base="100-input-phase:184",
        edits={"equipped_beams": 0, "x": 232, "y": 187},
        note="Samus beside a kihunter with the power beam"),
    # Green Brinstar's main shaft entered beside the etecoons (seed 43), with
    # Samus moved to the top of their shaft and the etecoon that waits for her
    # there (enemy slot 10, route 2) moved ahead of her, as its last hop
    # leaves it: its X ($11FA), Y ($11FE), instruction list ($1212, $A7:E854
    # hopping facing left) and timer ($1214), run speed ($122C), timer ($1230)
    # and function ($1232, $A7:EE3E hop until Samus is near). The route never
    # climbs the shaft, so the etecoon never shows her the tunnel jump.
    "etecoon-tunnel-jump": Scenario(
        base="100-input-phase:43",
        edits={"x": 550, "y": 1979},
        words={0x11FA: 560, 0x11FE: 1993, 0x1212: 0xE854, 0x1214: 1, 0x122C: 2,
               0x1230: 11, 0x1232: 0xEE3E},
        note="Samus beside the waiting etecoon at the top of the shaft"),
    # The landing site with the charge beam and one other beam equipped and
    # the power bombs selected, so a released charge fires that beam's
    # special attack.
    **{f"special-beam-{beam}": Scenario(
        base="100-input-phase:305",
        edits={"equipped_beams": f"charge+{beam}", "power_bombs": 10,
               "selected_item": "power-bombs"},
        note=f"{beam} special beam attack at the landing site")
       for beam in ("wave", "ice", "spazer", "plasma")},
    # The 100% movie's Dachora room, entered at its top before the route has
    # the speed booster, with Samus moved to the floor beside the dachora,
    # which the route never approaches.
    "dachora": Scenario(
        base="100-input-phase:44",
        edits={"x": 170, "y": 1691},
        note="Samus beside the dachora"),
}


def resolve(name, value):
    """A field's value as a word: numbers, or names of items, beams or the HUD item."""
    if isinstance(value, int):
        return value
    table = {"selected_item": SELECTABLE}.get(name)
    if table is None and name.endswith("_items"):
        table = ITEMS
    if table is None and name.endswith("_beams"):
        table = BEAMS
    if table is None:
        raise ValueError(f"{name} takes a number")
    if name == "selected_item":
        return table[value]
    return sum(table[item] for item in value.split("+"))


def planned_words(scenario):
    words = {}
    for name, value in scenario.edits.items():
        if name not in FIELDS:
            raise ValueError(f"unknown field {name}")
        words[FIELDS[name]] = resolve(name, value)
    words.update(scenario.words)
    for offset, value in words.items():
        if not 0 <= offset < 0x1FFFF or not 0 <= value <= 0xFFFF:
            raise ValueError(f"edit ${offset:05X} = {value:#x} is out of range")
        if offset in PROTECTED or offset + 1 in PROTECTED:
            raise ValueError(f"${offset:04X} is checked by the seed loader and cannot change")
    return words


def base_entries(base):
    capture, index = base.split(":")
    directory = ROOT / "target/seeds" / capture
    with (directory / "rooms.csv").open(newline="") as stream:
        rows = [row for row in csv.DictReader(stream) if row["room_index"] == index]
    if len(rows) != len(REGIONS):
        raise ValueError(f"{base}: the capture has no complete seed")
    return directory, rows


def seed_digest(directory, prefix):
    digest = hashlib.sha256()
    for region, size in REGIONS:
        data = (directory / f"{prefix}-{region}.bin").read_bytes()
        if len(data) != size:
            raise ValueError(f"{region} has {len(data)} bytes, not {size}")
        digest.update(data)
    digest.update((directory / f"{prefix}-registers.csv").read_bytes())
    return digest.hexdigest()


def write(name):
    scenario = SCENARIOS[name]
    words = planned_words(scenario)
    source, rows = base_entries(scenario.base)
    prefix = rows[0]["file"].removesuffix("-wram.bin")
    target = ROOT / "target/seeds" / f"scenario-{name}"
    if target.exists():
        shutil.rmtree(target)
    target.mkdir(parents=True)
    for region, _ in REGIONS:
        shutil.copyfile(source / f"{prefix}-{region}.bin", target / f"{prefix}-{region}.bin")
    shutil.copyfile(source / f"{prefix}-registers.csv", target / f"{prefix}-registers.csv")
    wram = bytearray((target / f"{prefix}-wram.bin").read_bytes())
    for offset, value in sorted(words.items()):
        wram[offset:offset + 2] = value.to_bytes(2, "little")
    (target / f"{prefix}-wram.bin").write_bytes(wram)
    with (target / "rooms.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator="\n")
        writer.writeheader()
        for row in rows:
            data = (target / row["file"]).read_bytes()
            writer.writerow({**row, "sha256": hashlib.sha256(data).hexdigest()})
    index = int(rows[0]["room_index"])
    print(f'seed = "scenario-{name}:{index}"')
    print(f'seed_sha256 = "{seed_digest(target, prefix)}"')
    print(f"# {len(words)} edited words; wram: {target / (prefix + '-wram.bin')}")


def show(name):
    scenario = SCENARIOS[name]
    source, rows = base_entries(scenario.base)
    wram = (source / rows[0]["file"]).read_bytes()
    names = {offset: field for field, offset in FIELDS.items()}
    for offset, value in sorted(planned_words(scenario).items()):
        before = int.from_bytes(wram[offset:offset + 2], "little")
        print(f"${offset:04X} {names.get(offset, 'word'):20} {before:#06x} -> {value:#06x}")


def check():
    for name, scenario in SCENARIOS.items():
        if not re.fullmatch(r"[a-z0-9-]+", name):
            raise SystemExit(f"{name}: scenario names are lowercase words joined by '-'")
        if not re.fullmatch(r"[A-Za-z0-9-]+:[0-9]+", scenario.base):
            raise SystemExit(f"{name}: base must be CAPTURE:INDEX")
        try:
            planned_words(scenario)
        except (ValueError, KeyError) as error:
            raise SystemExit(f"{name}: {error}")
    for bad in ({"energy": 1, "x": "left"}, {"health": 1}):
        try:
            planned_words(Scenario(base="x:1", edits=bad))
        except (ValueError, KeyError):
            continue
        raise SystemExit(f"accepted an invalid edit: {bad}")
    try:
        planned_words(Scenario(base="x:1", words={0x0998: 8}))
        raise SystemExit("accepted an edit to the checked gameplay state")
    except ValueError:
        pass
    print(f"{len(SCENARIOS)} scenarios check")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    for command in ("write", "show"):
        commands.add_parser(command).add_argument("scenario", choices=sorted(SCENARIOS))
    commands.add_parser("check")
    args = parser.parse_args()
    try:
        if args.command == "check":
            check()
        elif args.command == "write":
            write(args.scenario)
        else:
            show(args.scenario)
    except (ValueError, OSError) as error:
        sys.exit(f"scenario_seeds: {error}")


if __name__ == "__main__":
    main()
