#!/usr/bin/env python3
"""Write the authored controller inputs of the corpus rows that use them.

A recipe is a short script in the input language below. It holds only
controller words chosen here, nothing from the ROM or a movie, except that a
seeded row's first words must be the movie words latched at its seed; --prefix
takes them from that seed's imported movie inputs. The output stays local and
private like every other corpus input; scenarios/lock.toml pins its SHA-256
for its scenario in scenarios/corpus.toml.

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

A sweep writes one input per parameter set and prints its scenarios and the
tools/scenario.py lock command that pins their inputs.
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


def _mockball(direction, run="40", air="19"):
    """Run, spin jump and, just before landing, morph without forward: Down
    aims down, a frame of jump alone keeps that pose (any other button would
    fall back and drop the extra run speed) and a fresh Down morphs. The ball
    lands still morphing and rolls on with the jump's momentum."""
    move = f"{direction}+{RUN}"
    return (f"{move} {run}; {move}+{JUMP} 6; {move} {air}; {RUN}+Down 1; {JUMP} 1; "
            f"{RUN}+Down 1; {direction} 40")


def _short_charge(direction, taps="4"):
    return f"repeat {taps} {{ {direction} 6; _ 2 }}; {direction}+{RUN} 60"


def _charge_shot(direction="_", frames="70"):
    held = SHOOT if direction == "_" else f"{direction}+{SHOOT}"
    return f"{held} {frames}; _ 1"


def _morph():
    return "press Down 4; press Down 4"


def _spark(direction):
    """Release a stored shinespark: straight up, or from a jump sideways or
    diagonally (holding the aim-up button R)."""
    turn = "Left 3; _ 10; " if "left" in direction else ""
    side = "Left" if "left" in direction else "Right"
    if direction == "up":
        return "A 10"
    aim = "+R" if direction.startswith("up-") else ""
    return f"{turn}A 4; _ 8; {side}{aim}+A 10"


def _toggle_twice():
    return "press A; press A; press Down"


# Named moves: each takes its arguments as words and returns language text.
MOVES = {"walljump": _walljump, "mockball": _mockball, "short-charge": _short_charge,
         "charge-shot": _charge_shot, "morph": _morph, "toggle-twice": _toggle_twice,
         "spark": _spark}


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
    # Landing site (seed 100-input-phase:305): run right until the speed
    # booster charges, crouch to store the charge, then release it.
    "shinespark": Recipe("Right+B 150; Right+B+Down 2; _ 30; spark $direction; _ 200",
                         {"direction": "up"}),
    # Landing site with crystal flash conditions (scenario crystal-flash):
    # morph, select the power bombs, lay one and hold L+R+Down+shoot until it
    # ends.
    "crystal-flash": Recipe("""
        _ 40; Down 2; _ 6; Down 2; _ 6; repeat 3 { Select 2; _ 6 }; _ 20
        X 2; Down+L+R+X 500; _ 300
    """),
    # Landing site: walk to the left wall, spin jump into it for $hold
    # frames, then kick off to the right.
    "walljump": Recipe("_ 20; Left 8; Left+A $hold; Right 2; Right+A 10; _ 100", {"hold": 22}),
    # Select the X-ray scope, hold it and sweep the beam up and down.
    "xray-scan": Recipe("""
        _ 40; repeat 5 { Select 2; _ 6 }; _ 10
        B 30; Up+B 30; Down+B 30; B 20; _ 60
    """),
    # Landing site: walk right, jump onto the gunship, walk to its hatch and
    # board; once the ship has refilled Samus, keep confirming its save prompt.
    "gunship-save": Recipe("""
        _ 10; Right 340; Right+A 25; Right 10; _ 20; Right 32; _ 30; Down 2
        _ 500; repeat 8 { A 2; _ 58 }; _ 300
    """),
    # Landing site: spin jump right and keep space jumping every 28 frames.
    "space-jump-chain": Recipe("_ 20; Right 8; Right+A 10; repeat 8 { Right 24; Right+A 4 }; _ 120"),
    # Seed 100-input-phase:306: spin back and forth with the screw attack
    # through the room's enemies.
    "screw-attack": Recipe("_ 20; repeat 12 { Left 6; Left+A 30; Right 6; Right+A 30 }; _ 60"),
    # Landing site: mockball to the right (the ball keeps the extra run speed).
    "mockball": Recipe("mockball Right; _ 20"),
    # Landing site with one beam, charge and the power bombs (scenario
    # special-beam-*): keep shooting held past a full charge, which fires the
    # beam's special attack, then watch it run.
    "special-beam": Recipe("_ 20; X 150; _ 300"),
    # Oum room with Samus beside the first Oum (scenario oum-roll): walk into
    # it, so it rolls and pushes her back into the pit.
    "oum-roll": Recipe("_ 30; Right 60; _ 300"),
    # Tatori room with full energy (scenario tatori-touch): hop right across
    # the sleeping turtle and its young.
    "tatori-touch": Recipe("_ 10; repeat 10 { Right 30; Right+A 15 }; _ 300"),
    # After the death row's 1,800 movie words (--prefix-frames 1800): on the
    # game over menu choose to quit, skip the title, then in the file select
    # copy the saved file to another slot and clear a file.
    "file-copy-clear": Recipe("""
        press Down 20; press A 300; press Start 230; press A 120
        repeat 3 { press Down 30 }; press A 90; press Up 30
        press A 60; press A 60; press A 120; press A 120
        repeat 4 { press Down 30 }; press A 90; press A 60; press A 60; press A 200
    """),
    # Pause, wait for the pause screen, unpause and watch the room resume.
    "pause-resume": Recipe("press Start 150; hold Start 200; _ $tail", {"tail": 240}),
    # Ceres seed 100-input-phase:5: run right, open the door to Ridley's room
    # with a beam shot and keep running into the transition.
    "ceres-ridley-door": Recipe("Right+B 70; Right+X 2; Right+B 80; _ 220"),
    # Red Tower's bottom (seed 100-input-phase:176): walk to the left door,
    # select missiles, open it with two and walk into the refill room.
    "red-tower-refill-door": Recipe(
        "Left 70; Select 2; _ 6; X 2; _ 30; X 2; _ 30; Left 120; _ 150"),
    # Colosseum (seed 100-input-phase:257): select the grapple, drop off the
    # entry ledge, turn and grapple the wall's grapple blocks, reel in until the
    # swing stops against the wall (the wallgrab), then let go and jump off it.
    "grapple-walljump": Recipe("""
        repeat 4 { Select 2; _ 4 }; Right 14; _ 4; Left 6; _ 1
        X+Left 4; X 10; X+Up 2; X 40; _ 4; A 4; _ 60
    """),
    # After the boss-draygon words to 1,480 (seed 100-boundary:259): stay
    # released until Draygon grabs Samus, select the grapple, then fire it
    # while struggling in his grip, straight, up and down, toward him and away.
    "draygon-grapple-held": Recipe("""
        _ 1120; Select 2; _ 4
        Right 10; Right+X 4; Right 20; Right+Up 4; Right+Up+X 4; Right+Up 20
        Right+Down 4; Right+Down+X 4; Right+Down 20; Left 10; Left+X 4; Left 20
        Left+Up 4; Left+Up+X 4; Left+Up 20; _ 100
    """),
    # Bowling Alley with Samus on the Chozo hand (scenario wrecked-ship-hand):
    # morph in its palm, then ride the statue across the spikes until it sets
    # her down.
    "wrecked-ship-hand": Recipe("_ 10; morph; _ 2100"),
    # Crateria's kihunter room with the power beam (scenario kihunter-wings):
    # turn to the left kihunter and keep shooting as it swoops, so it loses
    # its wings, which drift down.
    "kihunter-wings": Recipe("_ 10; Left 2; repeat 40 { X 2; _ 6 }; _ 120"),
    # The Lower Norfair acid statue room with the space jump (scenario
    # lower-norfair-hand): morph, select the power bombs and lay one to clear
    # the blocks over the statue's hand, stand, jump onto the hand, morph
    # facing right in its palm and wait while the acid lowers.
    "lower-norfair-hand": Recipe("""
        _ 4; morph; repeat 3 { Select 2; _ 4 }; X 2; _ 200
        Up 2; _ 6; Up 2; _ 20; A 4; Left+A 16; A 10; _ 30
        morph; Right 2; _ 1050
    """),
    # Green Brinstar's main shaft entered beside the etecoons (seed 43): turn
    # right to face them and wait, so the one that lands beside her runs left.
    "etecoon-face-right": Recipe("Right 12; _ 500"),
    # Bomb Torizo's room (seed 100-input-phase:27): walk to the bombs, wait out
    # their message, then dodge and keep shooting as Bomb Torizo wakes. With
    # its health at 100 (scenario bomb-torizo-fight) the shots finish it.
    "bomb-torizo-fight": Recipe("""
        _ 10; Right+X 70; _ 700; A 1; _ 120; Left+X 100; _ 5; Left+A+X 70; _ 125
        A+X 20; Left+A+X 60; _ 160; Right 1; X 70; B+Right+A+X 65; Left+X 1
        X 230; B+Left+A+X 4; repeat 60 { X 1; _ 9 }; _ 130
    """),
    # Bomb Torizo's room (seed 100-input-phase:27): take the bombs, wait out
    # their message, morph and lay two, the second of which bomb jumps her.
    "bomb-jump": Recipe("_ 10; Right+X 70; _ 700; A 1; _ 60; morph; X 2; _ 100; X 2; _ 100"),
    # Norfair's save room B167 (seed any-boundary:110): walk to the station's
    # step, back off, hop onto the station, step onto its centre and confirm
    # the save prompt and the saved message. Samus waits at rest before each
    # move, so both oracles' landings line up.
    "norfair-save": Recipe("""
        _ 19; Left 30; _ 10; Right 10; _ 20; Left 6; Left+A 6; _ 40; Left 3; _ 60
        A 2; _ 180; A 2; _ 220
    """),
    # The same save with one energy (scenario one-energy-save): leave the pit
    # for the Business Center, where the first hit kills her, then confirm the
    # game over menu's continue and the file's load.
    "save-death-continue": Recipe("""
        _ 19; Left 30; _ 10; Right 10; _ 20; Left 6; Left+A 6; _ 40; Left 3; _ 60
        A 2; _ 180; A 2; _ 160
        Left 60; _ 10; Right 14; _ 10; Left 2; Left+A 30; Left 140; Left+A 40; _ 300
        repeat 5 { Right+A 30; Left+A 30 }; _ 300; repeat 3 { A 2; _ 300 }; _ 360
    """),
    # Double Chamber ADAD after the 100% movie's 127 words (seed
    # 100-input-phase:103), which open the shot gate: below its trigger,
    # shoot straight up until a shot triggers it again and it closes.
    "gate-reclose": Recipe("_ 80; repeat 4 { Up 10; Up+X 2; Up 20; _ 10 }; _ 60"),
}

SWEEPS = {
    # Where the low% Draygon fight is released decides whether Draygon
    # grabs Samus (1902) or misses her and swoops again (1880).
    "draygon-release": Sweep(
        recipe="release", row="boss-draygon-release", seed="low-input-phase:129",
        values={"prefix_frames": [1880, 1902], "frames": [1600]}, expect=["room:DA60"]),
    # Release a stored shinespark in each direction.
    "shinespark": Sweep(
        recipe="shinespark", row="samus-shinespark", seed="100-input-phase:305",
        values={"direction": ["up", "right", "left", "up-right", "up-left"]},
        expect=["room:91F8"],
        hits={"up": ["shinespark.store", "shinespark.vertical"],
              "right": ["shinespark.store", "shinespark.horizontal"],
              "left": ["shinespark.store", "shinespark.horizontal"],
              "up-right": ["shinespark.store", "shinespark.diagonal"],
              "up-left": ["shinespark.store", "shinespark.diagonal"]}),
    # Wall jumps after spinning into the wall for several lengths of time.
    "walljump": Sweep(
        recipe="walljump", row="samus-walljump", seed="100-input-phase:305",
        values={"hold": [16, 22, 28]}, expect=["room:91F8"],
        hits={str(hold): ["samus.jump.wall"] for hold in (16, 22, 28)}),
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
    "shinespark": "ab22c119c7303e4d4e6c99e68c7fd21af4bab44336a2888d0b87fd04a9901ef1",
    "crystal-flash": "d82453f56f8f0f9a579385961533f2415fe86e47e53e171559334fd8f5e199b6",
    "walljump": "51c1ddb44ada90dc6af96e0dbc0b1bdbed90566eaf1c625e4b9ca8705cc435a1",
    "xray-scan": "5419367a7705c5a3a6ba91985ff47f4f13ddd073ffacab058acc1732807f55ee",
    "gunship-save": "1c59417aed908dbb468ef711a1955314b2b4cecc8bac14f0e1eec07c6c5493fe",
    "space-jump-chain": "fc3d13a804e7b28519f9142ac92f57fc4e48884c181e90690d947019f961345d",
    "screw-attack": "ce49ad2b34eb6e5256cb0a73deb28043b434660b096aaee3ba35974ce51570fe",
    "mockball": "24739893dc7f2f9fdf11b50d828e83e6a75107d6929e1cde14e626cb46c0d3f9",
    "special-beam": "52d64f88295f3755852b633a417da2b0525cc712979c8a9c78e7e66486f74f73",
    "oum-roll": "1ee33e3559e0d647c919181f1b70ad8d1bac9ac4ba331444264b04c5598d4441",
    "tatori-touch": "f557eb7d7d0ff9172bb4e0e2c865f139eb92d20b12ca62b4315dfe0581e1c7d1",
    "file-copy-clear": "a8d3e69d868ec3373c3edc178a75bf55bf06b68ba8db78af2c161a64307208fe",
    "pause-resume": "6982afc4c06ba2311042798e0c0c2ccdbe47a7dbe599fb50b927ba414c24d077",
    "ceres-ridley-door": "45991b53cf8828c9542d208c26bc8d30143eb6f8fa1e21f5e590bec33909cb59",
    "red-tower-refill-door": "73d7fc2e9b32e137867ca292defb27fcf7d9b87316fc4fa004ed442474f4d95d",
    "grapple-walljump": "7e33e488fdb27153a412c60cfdfd446474251343430506417d5dce9848d0ac09",
    "draygon-grapple-held": "25548fe6af9cc6dc0e3da4f127a498af703f31b4a2a8cc402e0abe0b5fa1ea20",
    "wrecked-ship-hand": "f968153478819d4364cde8e0f882b682a8cc97f9e52d25ac51928b8cabf2127f",
    "kihunter-wings": "a12900693ec38b1f1eea8979e825b04b4744e5dfefdc7b9f758f9e148d0e9bfc",
    "lower-norfair-hand": "c20e8f655f662e35059ebf591e6969ec47947bba45b58b5a653197213e6d0dab",
    "etecoon-face-right": "6973e5e54d5e1b35c7bd6123b9516eabb8dc88b5adb3e5adf98f9f9ff68c6678",
    "bomb-torizo-fight": "08149d61cf9c1d0b15a0a84aa04cbbf3ec403ab972de7068a5073a1cafad5837",
    "bomb-jump": "42c5d0cea0ef134fe24bb583d08d62520b1c83f73ec11767f43802e32e9da122",
    "norfair-save": "49a0ba0b75a67da03959d8c53776284b1db5f57ad7d10c6bc1e4226679f2ff3d",
    "save-death-continue": "07002600f6a8d22a803c4c3c7bf715e0a30b9bdf326d9d2aa94437fb4104706f",
    "gate-reclose": "8b753fa8a650f77de1b281f32cc9f846cf55afe634c49b04e668658c9b1648bd",
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
        facts = ", ".join(f'"{item}"' for item in [*plan.expect, *(f"probe:{hit}" for hit in plan.hits.get(suffix, []))])
        print(f'\n[[scenario]]\nid = "{plan.row}-{suffix}"\nstart = "seed:{seed_sha256(plan.seed)}"\n'
              f"ticks = {len(data)}\nfacts = [{facts}]\n"
              f"# python tools/scenario.py lock --pin {plan.row}-{suffix} seed={plan.seed} inputs={stem}.inputs")


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
        args = {"walljump": ["Left"], "mockball": ["Right"], "short-charge": ["Right"],
                "spark": ["up-left"]}
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
