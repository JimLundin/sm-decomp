# sm-decomp

Super Metroid (SNES) in Rust. Gameplay currently runs as Rust code on
emulated SNES hardware. A built-in 65816 emulator runs the original ROM
as a reference, allowing the Rust implementation to be checked frame by frame.

The code was transpiled once from [snesrev/sm](https://github.com/snesrev/sm)
with c2rust and is being developed into safe, idiomatic Rust while preserving
the original game's behavior and quirks.

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

Load prepared room content with `sm.exe --room-pack PATH ROM`. On Windows, the selected ROM path must be ASCII; a relative ROM path also requires an ASCII working directory unless `--config` is supplied. Room-pack paths support Unicode.

## Verify

The [verification guide](movies/README.md) explains the headless per-frame
ROM comparisons and replay corpus, which covers areas, bosses, item pickups,
transitions and credits. Passing replays establish fidelity for the behavior
they exercise; other paths remain to be checked.

## Credits and licence

The following projects supply translated or linked code; their applicable
copyright and license notices are retained in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

| Project | Code used here |
| --- | --- |
| [snesrev/sm](https://github.com/snesrev/sm/tree/578f90b3cc49557bb70060ad033bb90b8cf8ac50) | One-time Rust translation of gameplay, runtime and SNES emulation; [translation record](tools/transpile/README.md). |
| LakeSnes: [CPU](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965), [PPU/DSP](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18), [SPC700](https://github.com/elzo-d/LakeSnes/tree/1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29) | Pinned native hardware implementations, fetched by [the installer](tools/install_lakesnes.py) and linked into builds. |
| [snesrev/zelda3](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf) | Selected music, configuration, utility, shader and desktop routines with maintained SM compatibility changes. |
| [stb_image 2.26](https://crates.io/crates/stb-sys/0.6.0) | PNG decoder implementation; [source pins and build instructions](docs/PNG_DEPENDENCY.md). |

Tools and reference material: [c2rust](https://github.com/immunant/c2rust)
performed the original translation; Patrick Johnston's
[bank logs](https://patrickjohnston.org/bank/) inform names and labels;
[lsnes](https://tasvideos.org/EmulatorResources/Lsnes) and
[Snes9x](https://github.com/snes9xgit/snes9x) provide independent replay and
snapshot observations. These credits do not imply their implementations are
bundled with this repository.

Original contributions are MIT ([LICENSE](LICENSE)). Super Metroid is a
trademark of Nintendo; no ROM or game data is included.
