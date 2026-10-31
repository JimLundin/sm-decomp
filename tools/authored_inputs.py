#!/usr/bin/env python3
"""Write the authored controller inputs of the corpus rows that use them.

A recipe is a short script in the input language below. It holds only
controller words chosen here, nothing from the ROM or a movie, except that a
seeded row's first words must be the movie words latched at its seed; --prefix
takes them from that seed's imported movie inputs. The output stays local and
private like every other corpus input; its row in movies/segments.toml pins
its SHA-256.

Input language (statements end at a newline or ';', '#' starts a comment):

    BUTTONS N            hold BUTTONS for exactly N frames ('_' releases;
                         join buttons with '+': Right+A 20)
    press BUTTONS [W]    hold 2 frames, then release W frames (default 12)
    hold BUTTONS [W]     hold 20 frames, then release W frames; menus see
                         Start, L and R only once held past their input delay
    repeat N { ... }     repeat the block N times
    MOVE ARGS...         a named move from MOVES, e.g. walljump Left
    $name                a recipe parameter; a sweep sets several values

Menus see a d-pad or face-button press on its first frame. Named moves are
starting points whose timing must be checked against the replay of each use.

    python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv target/seed-303.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0303-9A44-wram.bin --frames 1
    python tools/authored_inputs.py write pause-menu-tour movies/authored-pause-menu-tour.inputs --prefix target/seed-303.inputs
    python tools/authored_inputs.py write options-tour movies/authored-options-tour.inputs
    python tools/authored_inputs.py sweep draygon-release movies --prefix target/seed-low-129.inputs
    python tools/authored_inputs.py check

A sweep writes one input per parameter set and prints its segments.toml rows.
"""
import argparse
import csv
import hashlib
import itertools
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

BUTTONS = {"B": 0x8000, "Y": 0x4000, "Select": 0x2000, "Start": 0x1000, "Up": 0x0800,
           "Down": 0x0400, "Left": 0x0200, "Right": 0x0100, "A": 0x0080, "X": 0x0040,
           "L": 0x0020, "R": 0x0010}
OPPOSITE = {"Left": "Right", "Right": "Left", "Up": "Down", "Down": "Up"}
# Default controls: A jumps, B runs, X shoots.
JUMP, RUN, SHOOT = "A", "B", "X"


def _walljump(toward, wait="10"):
    away = OPPOSITE[toward]
    return f"{toward}+{JUMP} 12; {toward} 3; {away} 1; {away}+{JUMP} {wait}"


def _mockball(direction, run="40"):
    move = f"{direction}+{RUN}"
    return (f"{move} {run}; {move}+{JUMP} 6; {move} 8; {move}+Down 2; {move} 2; "
            f"{move}+Down 2; {direction} 30")


def _short_charge(direction, taps="4"):
    return f"repeat {taps} {{ {direction} 6; _ 2 }}; {direction}+{RUN} 60"


def _charge_shot(direction="_", frames="70"):
    held = SHOOT if direction == "_" else f"{direction}+{SHOOT}"
    return f"{held} {frames}; _ 1"


def _morph():
    return "press Down 4; press Down 4"


def _toggle_twice():
    return "press A; press A; press Down"


# Named moves: each takes its arguments as words and returns language text.
MOVES = {"walljump": _walljump, "mockball": _mockball, "short-charge": _short_charge,
         "charge-shot": _charge_shot, "morph": _morph, "toggle-twice": _toggle_twice}


def tokens(text):
    for line in text.splitlines():
        line = line.split("#", 1)[0]
        yield from re.findall(r"[{};]|[^\s{};]+", line)
        yield ";"


def statements(stream):
    """Group tokens into statements; a block is a nested list of statements."""
    current = []
    for token in stream:
        if token == ";":
            if current:
                yield current
            current = []
        elif token == "{":
            current.append(list(statements(stream)))
        elif token == "}":
            if current:
                yield current
            return
        else:
            current.append(token)
    if current:
        yield current


def word_of(buttons):
    if buttons == "_":
        return 0
    try:
        return sum(BUTTONS[name] for name in buttons.split("+"))
    except KeyError as error:
        raise ValueError(f"unknown button {error.args[0]!r} in {buttons!r}") from None


def frames_of(text):
    if not text.isdigit():
        raise ValueError(f"frame count {text!r} is not a whole number")
    return int(text)


def run(statement_list, params, depth=0):
    if depth > 16:
        raise ValueError("moves nest too deeply")
    for statement in statement_list:
        statement = [token if isinstance(token, list) else substitute(token, params)
                     for token in statement]
        head, *rest = statement
        if head == "repeat":
            if len(rest) != 2 or not isinstance(rest[1], list):
                raise ValueError("repeat takes a count and a { block }")
            for _ in range(frames_of(rest[0])):
                yield from run(rest[1], params, depth + 1)
        elif head in ("press", "hold"):
            if not 1 <= len(rest) <= 2:
                raise ValueError(f"{head} takes buttons and an optional release count")
            held = 2 if head == "press" else 20
            wait = frames_of(rest[1]) if len(rest) == 2 else 12
            yield from [word_of(rest[0])] * held + [0] * wait
        elif head in MOVES:
            yield from run(statements(tokens(MOVES[head](*rest))), params, depth + 1)
        elif len(rest) == 1:
            yield from [word_of(head)] * frames_of(rest[0])
        else:
            raise ValueError(f"cannot read statement: {' '.join(map(str, statement))}")


def substitute(token, params):
    def value(match):
        if match[1] not in params:
            raise ValueError(f"recipe parameter ${match[1]} has no value")
        return str(params[match[1]])
    return re.sub(r"\$([a-z_]+)", value, token)


def compile_recipe(text, params=None):
    return list(run(list(statements(tokens(text))), params or {}))


@dataclass
class Recipe:
    text: str
    params: dict = field(default_factory=dict)


@dataclass
class Sweep:
    """Variants of one recipe and the corpus rows that replay them."""
    recipe: str
    row: str
    seed: str
    values: dict
    expect: list
    # Probes each variant must reach, keyed by its row suffix.
    hits: dict = field(default_factory=dict)


RECIPES = {
    # Pause in a room with reserve energy, scroll the map, open the equipment
    # screen, set the reserve tanks to manual and transfer their energy,
    # restore automatic mode, toggle each beam, suit, item and boots twice,
    # return to the map and unpause.
    "pause-menu-tour": Recipe("""
        _ 30; press Start 150
        Right 40; Left 40; Down 40; Up 40; _ 10
        hold R 60
        press A 15; press Down 15; press A 90; press A 15
        press Down 15; repeat 5 { toggle-twice }
        press Right 15; repeat 9 { toggle-twice }
        press Left 15; press Up 15
        hold L 60; Down 20; _ 10
        hold Start 150
    """),
    # From power-on: skip to the title, leave it and choose the first (empty)
    # file. In the option mode, remap two controls, scroll to reset and end,
    # toggle both special settings, switch the text language twice, go back to
    # the file select, choose the file again and start the game. Leaving a
    # submenu or switching the language selects the first entry again.
    "options-tour": Recipe("""
        _ 200; press Start 150; press Start 200; press A 200
        press Down; press Down; press Down; press A 60
        press B; press Down; press Y; repeat 6 { press Down 30 }
        press Down 30; press A; press Up 30; press A 60
        press Down; press Down; press Down; press Down; press A 60
        press A; press Right; press Down; press A; press A
        press Down; press A 60
        press Down; press A 30; press Down; press Down; press A 30
        press B 200; press A 200; press A 300
    """),
    # Low% seed 85 (Kronic Boost Room, entered at its top): run and jump right
    # through the door to the Volcano Room, then stand while its lavaquake runs.
    "volcano-room": Recipe("repeat 8 { Right 10; Right+A 20 }; _ 600"),
    # Release the controller after the movie prefix (--prefix-frames). From
    # the low% Draygon fight, Samus then stays caught instead of grappling.
    "release": Recipe("_ $frames", {"frames": 1600}),
    # The same from the 100% Draygon fight (seed 100-boundary:259) after 1,480
    # movie words: with the gravity suit Samus outlasts the spiral, and
    # Draygon reaches the final tail whips.
    "long-release": Recipe("_ 3000"),
    # After the boss-draygon words to 1,480 (scenario seed draygon-repel):
    # cancel the item, select the grapple and fire it four frames before
    # Draygon reaches Samus, so the grapple repels his grab.
    "grapple-repel": Recipe("""
        _ 820; press Y 6; repeat 4 { press Select 6 }; _ 82
        X 60; _ 1700
    """),
    # Pause, wait for the pause screen, unpause and watch the room resume.
    "pause-resume": Recipe("press Start 150; hold Start 200; _ $tail", {"tail": 240}),
}

SWEEPS = {
    # Where the low% Draygon fight is released decides whether Draygon
    # grabs Samus (1902) or misses her and swoops again (1880).
    "draygon-release": Sweep(
        recipe="release", row="boss-draygon-release", seed="low-input-phase:129",
        values={"prefix_frames": [1880, 1902], "frames": [1600]}, expect=["room:DA60"]),
    # Pause the boss-kraid words while Kraid is alive (400), sinking (950),
    # dead in the fight (1030) and dead after the room reloads on Samus's
    # return from the Varia Suit room (1960); each unpause runs that hook.
    "kraid-pause": Sweep(
        recipe="pause-resume", row="boss-kraid-pause", seed="100-input-phase:73",
        values={"prefix_frames": [400, 950, 1030, 1960]}, expect=["room:A59F", "state:18"],
        hits={"400": ["kraid_pause.unpause_alive", "kraid_pause.transfer_top_half"],
              "950": ["kraid_pause.unpause_sinking", "kraid_pause.sinking_row_wait",
                      "kraid_pause.sinking_last_wait"],
              "1030": ["kraid_pause.unpause_dead", "kraid_pause.transfer_top_half"],
              "1960": ["kraid.dead_hook", "kraid_pause.unpause_dead",
                       "kraid_pause.transfer_top_half"]}),
}

# SHA-256 of each recipe's own words, before any prefix; `check` keeps them.
PINNED = {
    "pause-menu-tour": "b3fd5b2ec38a6cc00731f528244f120b7793ea0802dd4a5cd8e0b673b8efe5f0",
    "options-tour": "b06fcdb5eddf0784604c5e8e2c7d4a6f724ecffb62e762b82c799e0a75bc0128",
    "volcano-room": "e38317b3da5183e18af6c56257404ffee041db2d87583f8304549b1f5760d6d0",
    "release": "47a5f51e76c59f0f2b1f2b336b18e249cf9337d58fdb5fff6074b2ebea876071",
    "long-release": "5e93dfb35fefa55b112c3503c95292700424e13106b27eea5f248a0649e593a2",
    "grapple-repel": "875864d6999f51b0782c478dc115064e830ec019168817fbf2aefcee32c93cff",
    "pause-resume": "6982afc4c06ba2311042798e0c0c2ccdbe47a7dbe599fb50b927ba414c24d077",
}


def encode(data):
    return "".join(f"{word:04X}\n" for word in data).encode()


def read_prefix(path, frames):
    if frames == 0:
        return []
    if path is None:
        raise ValueError("a seeded recipe needs --prefix")
    prefix = [int(word, 16) for word in path.read_text().split()[:frames]]
    if len(prefix) != frames:
        raise ValueError("prefix is shorter than its frame count")
    return prefix


def write(path, data):
    with path.open("xb") as stream:
        stream.write(encode(data))


def seed_sha256(seed):
    capture, index = seed.split(":")
    with open(Path(__file__).resolve().parents[1] / f"movies/room-seeds-{capture}.csv",
              newline="") as stream:
        for row in csv.DictReader(stream):
            if row["room_index"] == index:
                return row["observation_sha256"]
    raise ValueError(f"no recorded seed {seed}")


def sweep(name, out_dir, prefix):
    plan = SWEEPS[name]
    keys = sorted(plan.values)
    for combination in itertools.product(*(plan.values[key] for key in keys)):
        params = {**RECIPES[plan.recipe].params, **dict(zip(keys, combination))}
        frames = int(params.pop("prefix_frames", 1))
        data = read_prefix(prefix, frames) + compile_recipe(RECIPES[plan.recipe].text, params)
        suffix = "-".join(str(value) for key, value in zip(keys, combination)
                          if len(plan.values[key]) > 1)
        stem = f"authored-{plan.row}-{suffix}"
        write(out_dir / f"{stem}.inputs", data)
        expect = ", ".join(f'"{item}"' for item in plan.expect)
        print(f'\n[[segment]]\nid = "{plan.row}-{suffix}"\nseed = "{plan.seed}"\n'
              f'seed_sha256 = "{seed_sha256(plan.seed)}"\ninputs = "{stem}.inputs"\n'
              f'input_sha256 = "{hashlib.sha256(encode(data)).hexdigest()}"\n'
              f"start_frame = 0\nframes = {len(data)}\nexpect = [{expect}]")
        if suffix in plan.hits:
            print("require_hit = [" + ", ".join(f'"{hit}"' for hit in plan.hits[suffix]) + "]")


def check():
    for name, digest in PINNED.items():
        recipe = RECIPES[name]
        actual = hashlib.sha256(encode(compile_recipe(recipe.text, recipe.params))).hexdigest()
        if actual != digest:
            raise SystemExit(f"{name}: recipe words changed ({actual})")
    cases = {
        "_ 3; A 2": [0, 0, 0, 0x80, 0x80],
        "press Start 1": [0x1000, 0x1000, 0],
        "repeat 2 { Right+B 1; _ 1 }": [0x8100, 0, 0x8100, 0],
        "walljump Left 1": [0x280] * 12 + [0x200] * 3 + [0x100] + [0x180],
        "charge-shot _ 2": [0x40, 0x40, 0],
    }
    for text, expected in cases.items():
        if compile_recipe(text) != expected:
            raise SystemExit(f"language case failed: {text}")
    for text in ("Jump 2", "Right", "repeat 2 Right 1", "_ $missing"):
        try:
            compile_recipe(text)
        except ValueError:
            continue
        raise SystemExit(f"language accepted invalid text: {text}")
    for name in MOVES:
        args = {"walljump": ["Left"], "mockball": ["Right"], "short-charge": ["Right"]}
        compile_recipe(" ".join([name, *args.get(name, [])]))
    print(f"{len(PINNED)} pinned recipes and {len(cases)} language cases match")


def main():
    arguments = sys.argv[1:]
    if arguments and arguments[0] in RECIPES:
        arguments.insert(0, "write")
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    writer = commands.add_parser("write", help="write one recipe's input")
    writer.add_argument("recipe", choices=sorted(RECIPES))
    writer.add_argument("output", type=Path)
    writer.add_argument("--prefix", type=Path, metavar="INPUTS",
                        help="the seed's imported movie inputs")
    writer.add_argument("--prefix-frames", type=int, metavar="N",
                        help="movie words kept before the recipe (default 1 with --prefix)")
    writer.add_argument("--set", action="append", default=[], metavar="NAME=VALUE",
                        help="override a recipe parameter")
    sweeper = commands.add_parser("sweep", help="write a sweep's inputs and print its rows")
    sweeper.add_argument("sweep", choices=sorted(SWEEPS))
    sweeper.add_argument("out_dir", type=Path)
    sweeper.add_argument("--prefix", type=Path, metavar="INPUTS")
    commands.add_parser("check", help="check the pinned recipes and the language")
    args = parser.parse_args(arguments)
    try:
        if args.command == "check":
            check()
        elif args.command == "sweep":
            sweep(args.sweep, args.out_dir, args.prefix)
        else:
            recipe = RECIPES[args.recipe]
            params = dict(recipe.params)
            for item in args.set:
                key, _, value = item.partition("=")
                params[key] = value
            frames = args.prefix_frames if args.prefix_frames is not None else int(bool(args.prefix))
            data = read_prefix(args.prefix, frames) + compile_recipe(recipe.text, params)
            write(args.output, data)
            print(f"{len(data)} frames")
    except ValueError as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
