import argparse
import zipfile
from pathlib import Path
lazy from collections.abc import Sequence

from ordered import FrozenList

from sm_tools.oracle.core.movie_inputs import (
    BK2_INPUT,
    DESCRIPTION,
    LSMV_INPUT,
    check_bk2_members,
    check_frames,
    check_lsmv_game,
    check_lsmv_members,
    check_rom,
    imported,
    inputs_text,
    is_lsmv,
    parse_bk2,
    parse_lsmv,
)


class Arguments(argparse.Namespace):
    movie: Path = Path()
    output: Path = Path()
    rom: Path = Path()


def load(path: Path, rom: Path) -> FrozenList[int]:
    with zipfile.ZipFile(path) as movie:
        names = FrozenList(movie.namelist())
        if is_lsmv(names):
            check_lsmv_members(names)
            check_lsmv_game(
                movie.read("gametype"), movie.read("controlsversion")
            )
            check_rom(movie.read("rom.sha256"), rom.read_bytes())
            frames = parse_lsmv(movie.read(LSMV_INPUT).decode())
        else:
            check_bk2_members(names)
            frames = parse_bk2(movie.read(BK2_INPUT).decode())
    check_frames(frames)
    return frames


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    _ = parser.add_argument("movie", type=Path)
    _ = parser.add_argument("output", type=Path)
    _ = parser.add_argument("--rom", type=Path, required=True)
    args = parser.parse_args(argv, namespace=Arguments())
    frames = load(args.movie, args.rom)
    _ = args.output.write_text(
        inputs_text(frames), encoding="ascii", newline="\r\n"
    )
    print(imported(frames))
    return 0
