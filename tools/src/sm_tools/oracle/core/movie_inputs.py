import hashlib
lazy from collections.abc import Sequence

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

DESCRIPTION = (
    "Import a clean-start LSMV or BK2 movie as hexadecimal SNES controller"
    " words."
)
BUTTON_BITS = FrozenOrderedDict({
    "B": 0x8000,
    "Y": 0x4000,
    "Select": 0x2000,
    "Start": 0x1000,
    "Up": 0x0800,
    "Down": 0x0400,
    "Left": 0x0200,
    "Right": 0x0100,
    "A": 0x0080,
    "X": 0x0040,
    "L": 0x0020,
    "R": 0x0010,
})
LSMV_BUTTONS = "BYsSudlrAXLR"
LSMV_INPUT = "input"
LSMV_STATE_MEMBERS = FrozenOrderedSet((
    "savestate",
    "savestate.anchor",
    "movie.bin",
))
LSMV_STATE_PREFIXES = ("moviesram.", "ramcontent.", "setting.")
LSMV_HEADERS = FrozenOrderedSet(("F.", "F. 0 0"))
BK2_STATE_MEMBERS = FrozenOrderedSet(("Core.bin", "SaveRAM.bin"))
BK2_INPUT = "Input Log.txt"
BK2_SYSTEM = FrozenOrderedSet(("Reset", "Power"))
RELEASED = FrozenOrderedSet((".", " "))


def is_lsmv(names: Sequence[str]) -> bool:
    return LSMV_INPUT in names


def check_lsmv_members(names: Sequence[str]) -> None:
    for name in names:
        if (
            name in LSMV_STATE_MEMBERS
            or name.startswith(LSMV_STATE_PREFIXES)
            or (name.startswith("port") and name[4:].isdigit())
        ):
            message = f"unsupported LSMV initialization: {name}"
            raise ValueError(message)


def check_lsmv_game(gametype: bytes, controls: bytes) -> None:
    if gametype.strip() != b"snes_ntsc" or controls.strip() != b"0":
        message = "unsupported LSMV game or controls version"
        raise ValueError(message)


def check_rom(expected: bytes, rom: bytes) -> None:
    if hashlib.sha256(rom).hexdigest() != expected.decode().strip().lower():
        message = "movie ROM SHA-256 does not match"
        raise ValueError(message)


def check_bk2_members(names: Sequence[str]) -> None:
    if BK2_STATE_MEMBERS & FrozenOrderedSet(names):
        message = "stateful BK2 initialization is unsupported"
        raise ValueError(message)


def check_frames(frames: Sequence[int]) -> None:
    if not frames:
        message = "empty movie"
        raise ValueError(message)


def parse_lsmv(text: str) -> FrozenList[int]:
    frames: list[int] = []
    for number, line in enumerate(text.splitlines(), 1):
        header, pad = line.split("|", 1)
        if header not in LSMV_HEADERS or len(pad) != len(LSMV_BUTTONS):
            message = (
                f"LSMV line {number}: unsupported reset, subframe or controller"
            )
            raise ValueError(message)
        word = 0
        for index, (value, button) in enumerate(
            zip(pad, LSMV_BUTTONS, strict=True)
        ):
            if value not in RELEASED and value != button:
                message = f"LSMV line {number}: invalid button {value!r}"
                raise ValueError(message)
            if value == button:
                word |= 0x8000 >> index
        frames.append(word)
    return FrozenList(frames)


def bk2_word(groups: Sequence[Sequence[str]], fields: Sequence[str]) -> int:
    word = 0
    for group, field in zip(groups, fields, strict=True):
        if len(group) != len(field):
            message = "invalid BK2 button group"
            raise ValueError(message)
        for name, value in zip(group, field, strict=True):
            if value in RELEASED:
                continue
            if not name.startswith("P1 "):
                message = f"active reset, power or extra controller: {name}"
                raise ValueError(message)
            word |= BUTTON_BITS[name[3:]]
    return word


def parse_bk2(text: str) -> FrozenList[int]:
    key = next(
        line[len("LogKey:") :]
        for line in text.splitlines()
        if line.startswith("LogKey:")
    )
    groups = FrozenList(
        FrozenList(name for name in group.split("|") if name)
        for group in key.split("#")
        if group
    )
    allowed = BK2_SYSTEM | FrozenOrderedSet(
        f"P{port} {name}" for port in (1, 2) for name in BUTTON_BITS
    )
    if any(name not in allowed for group in groups for name in group):
        message = "unsupported BK2 control"
        raise ValueError(message)
    frames: list[int] = []
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        fields = line[1:-1].split("|")
        if not line.endswith("|") or len(fields) != len(groups):
            message = "invalid BK2 frame groups"
            raise ValueError(message)
        frames.append(bk2_word(groups, fields))
    return FrozenList(frames)


def inputs_text(frames: Sequence[int]) -> str:
    return "".join(f"{word:04X}\n" for word in frames)


def imported(frames: Sequence[int]) -> str:
    return f"Imported {len(frames)} controller frames"
