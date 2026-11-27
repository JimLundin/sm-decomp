#!/usr/bin/env python3
import argparse
import hashlib
from pathlib import Path
from collections.abc import Sequence
import zipfile

BUTTON_BITS = dict(zip(
    ('B', 'Y', 'Select', 'Start', 'Up', 'Down', 'Left', 'Right', 'A', 'X', 'L', 'R'),
    (0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010),
))


def load(path: Path, rom: Path) -> list[int]:
    with zipfile.ZipFile(path) as movie:
        names = movie.namelist()
        if 'input' in names:
            for name in names:
                if name in ('savestate', 'savestate.anchor', 'movie.bin') or name.startswith(
                    ('moviesram.', 'ramcontent.', 'setting.')
                ) or (name.startswith('port') and name[4:].isdigit()):
                    raise ValueError(f'unsupported LSMV initialization: {name}')
            if movie.read('gametype').strip() != b'snes_ntsc' or movie.read('controlsversion').strip() != b'0':
                raise ValueError('unsupported LSMV game or controls version')
            expected = movie.read('rom.sha256').decode().strip().lower()
            if hashlib.sha256(rom.read_bytes()).hexdigest() != expected:
                raise ValueError('movie ROM SHA-256 does not match')
            frames = parse_lsmv(movie.read('input').decode())
        else:
            if {'Core.bin', 'SaveRAM.bin'} & set(names):
                raise ValueError('stateful BK2 initialization is unsupported')
            frames = parse_bk2(movie.read('Input Log.txt').decode())
    if not frames:
        raise ValueError('empty movie')
    return frames


def parse_lsmv(text: str) -> list[int]:
    buttons = 'BYsSudlrAXLR'
    frames: list[int] = []
    for number, line in enumerate(text.splitlines(), 1):
        header, pad = line.split('|', 1)
        if header not in ('F.', 'F. 0 0') or len(pad) != 12:
            raise ValueError(f'LSMV line {number}: unsupported reset, subframe or controller')
        word = 0
        for index, (value, button) in enumerate(zip(pad, buttons)):
            if value not in ('.', ' ', button):
                raise ValueError(f'LSMV line {number}: invalid button {value!r}')
            if value == button:
                word |= 0x8000 >> index
        frames.append(word)
    return frames


def parse_bk2(text: str) -> list[int]:
    key = next(line[len('LogKey:'):] for line in text.splitlines() if line.startswith('LogKey:'))
    groups = [[name for name in group.split('|') if name] for group in key.split('#') if group]
    allowed = {'Reset', 'Power'} | {f'P{port} {name}' for port in (1, 2) for name in BUTTON_BITS}
    if any(name not in allowed for group in groups for name in group):
        raise ValueError('unsupported BK2 control')
    frames: list[int] = []
    for line in text.splitlines():
        if not line.startswith('|'):
            continue
        fields = line[1:-1].split('|')
        if not line.endswith('|') or len(fields) != len(groups):
            raise ValueError('invalid BK2 frame groups')
        word = 0
        for group, field in zip(groups, fields):
            if len(group) != len(field):
                raise ValueError('invalid BK2 button group')
            for name, value in zip(group, field):
                if value in ('.', ' '):
                    continue
                if not name.startswith('P1 '):
                    raise ValueError(f'active reset, power or extra controller: {name}')
                word |= BUTTON_BITS[name[3:]]
        frames.append(word)
    return frames


def main(argv: Sequence[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        description='Import a clean-start LSMV or BK2 movie as hexadecimal SNES controller words.')
    parser.add_argument('movie', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--rom', type=Path, required=True)
    args = parser.parse_args(argv)
    frames = load(args.movie, args.rom)
    args.output.write_text(''.join(f'{word:04X}\n' for word in frames),
                           encoding='ascii', newline='\r\n')
    print(f'Imported {len(frames)} controller frames')


if __name__ == '__main__':
    main()
