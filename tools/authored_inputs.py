#!/usr/bin/env python3
"""Write the authored controller inputs of the corpus rows that use them.

A recipe holds only controller words chosen here, nothing from the ROM or a
movie, except that a seeded row's first word must be the movie word latched at
its seed; --prefix takes it from that seed's imported movie inputs. Each step
holds its buttons for a number of frames. Menus see a d-pad or face-button press
on its first frame, but Start, L and R only after they are held unchanged for
the menu's input delay, so those are held longer. The output stays local and
private like every other corpus input; its row in movies/segments.toml pins
its SHA-256. For pause-menu-tour (seed 100-input-phase:303):

    python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv target/seed-303.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0303-9A44-wram.bin --frames 1
    python tools/authored_inputs.py pause-menu-tour movies/authored-pause-menu-tour.inputs --prefix target/seed-303.inputs
    python tools/authored_inputs.py options-tour movies/authored-options-tour.inputs
"""
import argparse
from pathlib import Path

BUTTONS = {"B": 0x8000, "Y": 0x4000, "Select": 0x2000, "Start": 0x1000, "Up": 0x0800,
           "Down": 0x0400, "Left": 0x0200, "Right": 0x0100, "A": 0x0080, "X": 0x0040,
           "L": 0x0020, "R": 0x0010}


def press(buttons, wait=12):
    """Press buttons for two frames, then release them for `wait` frames."""
    return [(buttons, 2), ("", wait)]


def hold(buttons, wait):
    """Hold a menu Start, L or R past the timed-input delay, then release it."""
    return [(buttons, 20), ("", wait)]


def toggle_twice(times):
    """Toggle the selected item off and on again, then move down; `times` times."""
    return (press("A") + press("A") + press("Down")) * times


# Pause in a room with reserve energy, scroll the map, open the equipment
# screen, set the reserve tanks to manual and transfer their energy, restore
# automatic mode, toggle each beam, suit, item and boots twice, return to the
# map and unpause.
PAUSE_MENU_TOUR = [
    ("", 30), *press("Start", 150),
    ("Right", 40), ("Left", 40), ("Down", 40), ("Up", 40), ("", 10),
    *hold("R", 60),
    *press("A", 15), *press("Down", 15), *press("A", 90), *press("A", 15),
    *press("Down", 15), *toggle_twice(5),
    *press("Right", 15), *toggle_twice(9),
    *press("Left", 15), *press("Up", 15),
    *hold("L", 60), ("Down", 20), ("", 10),
    *hold("Start", 150),
]

# From power-on: skip to the title, leave it and choose the first (empty) file.
# In the option mode, remap two controls, scroll to reset and end, toggle both
# special settings, switch the text language twice, go back to the file
# select, choose the file again and start the game. Leaving a submenu or
# switching the language selects the first entry again.
OPTIONS_TOUR = [
    ("", 200), *press("Start", 150), *press("Start", 200), *press("A", 200),
    *press("Down"), *press("Down"), *press("Down"), *press("A", 60),
    *press("B"), *press("Down"), *press("Y"), *[step for _ in range(6) for step in press("Down", 30)],
    *press("Down", 30), *press("A"), *press("Up", 30), *press("A", 60),
    *press("Down"), *press("Down"), *press("Down"), *press("Down"), *press("A", 60),
    *press("A"), *press("Right"), *press("Down"), *press("A"), *press("A"),
    *press("Down"), *press("A", 60),
    *press("Down"), *press("A", 30), *press("Down"), *press("Down"), *press("A", 30),
    *press("B", 200), *press("A", 200), *press("A", 300),
]

RECIPES = {"pause-menu-tour": PAUSE_MENU_TOUR, "options-tour": OPTIONS_TOUR}


def words(steps):
    for buttons, frames in steps:
        word = sum(BUTTONS[name] for name in buttons.split("+") if name)
        yield from [word] * frames


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("recipe", choices=sorted(RECIPES))
    parser.add_argument("output", type=Path)
    parser.add_argument("--prefix", type=Path, metavar="INPUTS",
                        help="the seed's imported movie inputs")
    parser.add_argument("--prefix-frames", type=int, default=1, metavar="N",
                        help="movie words kept before the recipe (default 1, the latched word)")
    args = parser.parse_args()
    prefix = []
    if args.prefix:
        prefix = [int(word, 16) for word in args.prefix.read_text().split()[:args.prefix_frames]]
        if len(prefix) != args.prefix_frames:
            parser.error("prefix is shorter than --prefix-frames")
    data = prefix + list(words(RECIPES[args.recipe]))
    with args.output.open("x", newline="\n") as stream:
        stream.write("".join(f"{word:04X}\n" for word in data))
    print(f"{len(data)} frames")


if __name__ == "__main__":
    main()
