from dataclasses import dataclass, field
lazy from collections.abc import Mapping

from ordered import FrozenOrderedDict, FrozenOrderedSet

WORK_RAM_WORDS = FrozenOrderedDict({
    "energy": 0x09C2,
    "max_energy": 0x09C4,
    "reserve_energy": 0x09D6,
    "max_reserve_energy": 0x09D4,
    "reserve_mode": 0x09C0,
    "missiles": 0x09C6,
    "max_missiles": 0x09C8,
    "super_missiles": 0x09CA,
    "max_super_missiles": 0x09CC,
    "power_bombs": 0x09CE,
    "max_power_bombs": 0x09D0,
    "equipped_items": 0x09A2,
    "collected_items": 0x09A4,
    "equipped_beams": 0x09A6,
    "collected_beams": 0x09A8,
    "selected_item": 0x09D2,
    "x": 0x0AF6,
    "y": 0x0AFA,
})
ITEM_BITS = FrozenOrderedDict({
    "varia": 0x0001,
    "spring-ball": 0x0002,
    "morph-ball": 0x0004,
    "screw-attack": 0x0008,
    "gravity": 0x0020,
    "hi-jump": 0x0100,
    "space-jump": 0x0200,
    "bombs": 0x1000,
    "speed-booster": 0x2000,
    "grapple": 0x4000,
    "x-ray": 0x8000,
})
BEAM_BITS = FrozenOrderedDict({
    "wave": 0x0001,
    "ice": 0x0002,
    "spazer": 0x0004,
    "plasma": 0x0008,
    "charge": 0x1000,
})
HUD_SELECTION_ORDER = FrozenOrderedDict({
    "none": 0,
    "missiles": 1,
    "super-missiles": 2,
    "power-bombs": 3,
    "grapple": 4,
    "x-ray": 5,
})
CHECKED_BY_THE_SEED_LOADER = FrozenOrderedSet((
    0x0998,
    0x079B,
    0x05B6,
    0x05D1,
    *range(0x0087, 0x008B),
))
WORK_RAM_END = 0x1FFFF
WORD_MAX = 0xFFFF


@dataclass(frozen=True, slots=True)
class Scenario:
    base: str
    edits: Mapping[str, int | str] = field(
        default_factory=FrozenOrderedDict[str, int | str]
    )
    work_ram_words: Mapping[int, int] = field(
        default_factory=FrozenOrderedDict[int, int]
    )
    note: str = ""


def special_beam(beam: str) -> Scenario:
    return Scenario(
        base="100-input-phase:305",
        edits=FrozenOrderedDict({
            "equipped_beams": f"charge+{beam}",
            "power_bombs": 10,
            "selected_item": "power-bombs",
        }),
        note=(
            f"{beam} special beam attack at the landing site: the charge beam"
            " and one other beam equipped and the power bombs selected, so a"
            " released charge fires that beam's special attack"
        ),
    )


SCENARIOS = FrozenOrderedDict({
    "coverage-metroid-ice": Scenario(
        base="100-input-phase:317",
        edits=FrozenOrderedDict({
            "equipped_items": "varia+morph-ball+gravity+hi-jump+bombs",
            "equipped_beams": "ice+plasma+charge",
            "selected_item": "none",
            "energy": 699,
            "max_energy": 699,
            "reserve_energy": 0,
            "missiles": 50,
            "max_missiles": 50,
        }),
        note=(
            "Retail Metroid ice freeze followed by ordinary missiles;"
            " no screw attack"
        ),
    ),
    "coverage-metroid": Scenario(
        base="100-input-phase:317",
        edits=FrozenOrderedDict({
            "equipped_items": (
                "varia+spring-ball+morph-ball+gravity+hi-jump+space-jump"
                "+bombs+speed-booster+grapple+x-ray"
            ),
            "equipped_beams": 0,
            "energy": 699,
            "max_energy": 699,
            "reserve_energy": 0,
        }),
        note="Metroid latch and drain with screw attack unequipped",
    ),
    "coverage-botwoon": Scenario(
        base="low-input-phase:122",
        edits=FrozenOrderedDict({
            "energy": 699,
            "max_energy": 699,
            "reserve_energy": 0,
        }),
        note=(
            "Botwoon spit patterns with enough energy to survive released"
            " controls"
        ),
    ),
    "draygon-repel": Scenario(
        base="100-boundary:259",
        edits=FrozenOrderedDict({
            "energy": 1099,
            "reserve_energy": 300,
            "selected_item": "grapple",
        }),
        note=(
            "Draygon grab with the grapple ready: the 100% movie's Draygon"
            " fight with full energy and reserves, so Samus survives his"
            " whole grab. The grapple-repel recipe reselects the grapple"
            " before firing it, since the movie words change the item"
        ),
    ),
    "one-energy-crocomire": Scenario(
        base="100-input-phase:142",
        edits=FrozenOrderedDict({"energy": 1, "reserve_energy": 0}),
        note=(
            "death at the first hit in Crocomire's room: the movie words now"
            " play out as on hardware, where Samus is never killed, so the"
            " death row takes its first hit as fatal"
        ),
    ),
    "crystal-flash": Scenario(
        base="100-input-phase:305",
        edits=FrozenOrderedDict({
            "energy": 40,
            "reserve_energy": 0,
            "power_bombs": 11,
        }),
        note=(
            "crystal flash conditions at the landing site late in the game:"
            " under 51 energy, empty reserves and at least ten of each kind"
            " of ammunition once the bomb is laid"
        ),
    ),
    "oum-roll": Scenario(
        base="100-input-phase:256",
        edits=FrozenOrderedDict({"x": 540, "y": 590}),
        note="Samus beside the first Oum, which the route never approaches",
    ),
    "tatori-touch": Scenario(
        base="100-input-phase:221",
        edits=FrozenOrderedDict({"energy": 999}),
        note=(
            "Tatori room with full energy, where the movie arrives with one,"
            " so Samus survives the turtles' touch"
        ),
    ),
    "ridley-swoop": Scenario(
        base="100-boundary:118",
        edits=FrozenOrderedDict({"energy": 699, "reserve_energy": 200}),
        work_ram_words=FrozenOrderedDict({0x0F8C: 9000}),
        note=(
            "Ridley fight at half health with full energy: Ridley's health,"
            " enemy slot 0, down to 9000, where he swoops, which the movie's"
            " quick fight never sees"
        ),
    ),
    "wrecked-ship-hand": Scenario(
        base="100-input-phase:211",
        edits=FrozenOrderedDict({"x": 1192, "y": 347}),
        note=(
            "Samus on the Wrecked Ship Chozo hand in the Bowling Alley after"
            " Phantoon, which the movie passes"
        ),
    ),
    "kihunter-wings": Scenario(
        base="100-input-phase:184",
        edits=FrozenOrderedDict({"equipped_beams": 0, "x": 232, "y": 187}),
        note=(
            "Samus beside a kihunter with the power beam: in the top corridor"
            " of the Crateria kihunter room beside the left kihunter, so her"
            " shots wound it without killing it and its wings fall"
        ),
    ),
    "etecoon-tunnel-jump": Scenario(
        base="100-input-phase:43",
        edits=FrozenOrderedDict({"x": 550, "y": 1979}),
        work_ram_words=FrozenOrderedDict({
            0x11FA: 560,
            0x11FE: 1993,
            0x1212: 0xE854,
            0x1214: 1,
            0x122C: 2,
            0x1230: 11,
            0x1232: 0xEE3E,
        }),
        note=(
            "Samus beside the waiting etecoon at the top of the shaft: Green"
            " Brinstar's main shaft entered beside the etecoons, the etecoon"
            " in enemy slot 10 on route 2 moved ahead of her as its last hop"
            " leaves it, with its X ($11FA), Y ($11FE), instruction list"
            " ($1212, $A7:E854 hopping facing left) and timer ($1214), run"
            " speed ($122C), timer ($1230) and function ($1232, $A7:EE3E hop"
            " until Samus is near). The route never climbs the shaft, so the"
            " etecoon never shows her the tunnel jump"
        ),
    ),
    "lower-norfair-hand": Scenario(
        base="100-input-phase:133",
        edits=FrozenOrderedDict({
            "collected_items": (
                "varia+morph-ball+screw-attack+hi-jump+space-jump+bombs"
                "+speed-booster+x-ray"
            ),
            "x": 104,
            "y": 155,
        }),
        note=(
            "Samus beside the Lower Norfair Chozo hand with the space jump,"
            " on the power bomb blocks that cover the acid statue's hand,"
            " which the route passes before it has the space jump"
        ),
    ),
    **{
        f"special-beam-{beam}": special_beam(beam)
        for beam in ("wave", "ice", "spazer", "plasma")
    },
    "dachora": Scenario(
        base="100-input-phase:44",
        edits=FrozenOrderedDict({"x": 170, "y": 1691}),
        note=(
            "Samus beside the dachora, on the floor of its room entered at"
            " the top before the route has the speed booster"
        ),
    ),
    "one-energy-save": Scenario(
        base="any-boundary:110",
        edits=FrozenOrderedDict({"energy": 1, "reserve_energy": 0}),
        note=(
            "save, then death at the first hit: Samus saves in the any%"
            " movie's Norfair save room (B167), and the first hit outside"
            " kills her, so the game over menu reloads the save"
        ),
    ),
})


def resolve(name: str, value: int | str) -> int:
    if isinstance(value, int):
        return value
    if name == "selected_item":
        return HUD_SELECTION_ORDER[value]
    if name.endswith("_items"):
        return sum(ITEM_BITS[item] for item in value.split("+"))
    if name.endswith("_beams"):
        return sum(BEAM_BITS[beam] for beam in value.split("+"))
    message = f"{name} takes a number"
    raise ValueError(message)


def planned_words(scenario: Scenario) -> FrozenOrderedDict[int, int]:
    words: dict[int, int] = {}
    for name, value in scenario.edits.items():
        if name not in WORK_RAM_WORDS:
            message = f"unknown field {name}"
            raise ValueError(message)
        words[WORK_RAM_WORDS[name]] = resolve(name, value)
    words.update(scenario.work_ram_words)
    for offset, value in words.items():
        if not 0 <= offset < WORK_RAM_END or not 0 <= value <= WORD_MAX:
            message = f"edit ${offset:05X} = {value:#x} is out of range"
            raise ValueError(message)
        if (
            offset in CHECKED_BY_THE_SEED_LOADER
            or offset + 1 in CHECKED_BY_THE_SEED_LOADER
        ):
            message = (
                f"${offset:04X} is checked by the seed loader and cannot change"
            )
            raise ValueError(message)
    return FrozenOrderedDict(words)
