# sm-decomp

Super Metroid (SNES) in Rust. Gameplay currently runs as Rust code on
emulated SNES hardware. A built-in 65816 emulator runs the original ROM
as a reference, allowing the Rust implementation to be checked frame by frame.

The code was transpiled once from [snesrev/sm](https://github.com/snesrev/sm)
with c2rust and is being developed into safe, idiomatic Rust while preserving
the original game's behavior and quirks.

![Codebase progress: Rust ownership and unsafe counts, with 24-hour unsafe-block trends by area](https://jimlundin.github.io/sm-decomp/treemap.svg)

The goal is a purpose-built Rust engine with owned gameplay state, its own
renderer and audio, and no SNES emulation dependency in the finished runtime.
The planned [architecture](docs/target-architecture/README.md) also supports
Map Rando integration through custom sprites and palettes, configurable room
and door connections, and opt-in re-spin and quality-of-life mechanics.
These are the destination; engine ownership and backend independence remain
work in progress.

## Run

See the [setup and build guide](docs/ITERATION.md) for prerequisites, ROM
requirements, launch commands and controls. You need your own copy of the game.

## Verify

The [verification guide](movies/README.md) explains the headless per-frame
ROM comparisons and replay corpus, which covers areas, bosses, item pickups,
transitions and credits. Passing replays establish fidelity for the behavior
they exercise; other paths remain to be checked.

## Credits and licence

Game logic, runtime and SNES emulation come from snesrev/sm (MIT, see
[its license](third_party/snesrev/LICENSE.txt)), translated to Rust with c2rust.
The hardware adapters link unmodified LakeSnes code under its
[CPU license](third_party/lakesnes/LICENSE.txt) and
[PPU/DSP license](third_party/lakesnes/LICENSE-2022.txt) and
[SPC license](third_party/lakesnes/LICENSE-SPC-2021.txt). PNG decoding uses
stb_image under its [MIT license](third_party/stb_image/LICENSE.txt).
Native music, configuration, utilities, shaders and the desktop host use
selected Zelda3 routines with maintained SM compatibility changes under the
[Zelda3 MIT license](third_party/zelda3/LICENSE.txt).
Names and labels also draw on Patrick Johnston's
[bank logs](https://patrickjohnston.org/bank/). Original contributions in this
repository are MIT ([LICENSE](LICENSE)). Super Metroid is a trademark of
Nintendo; no ROM or game data is included.
