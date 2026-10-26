# sm-decomp

Super Metroid (SNES) in Rust. The whole game runs as Rust code on emulated
SNES hardware, and every frame can be checked against the original ROM
running on the built-in 65816 emulator.

The code was transpiled once from [snesrev/sm](https://github.com/snesrev/sm)
with c2rust and is now being cleaned up into idiomatic Rust, one subsystem at
a time, with that per-frame comparison as the test. See
[docs/PLAN.md](docs/PLAN.md).

![Engine migration progress: required checks, legacy, unsafe and production lines, and production Rust by area](https://jimlundin.github.io/sm-decomp/treemap.svg)

Drawn by `tools/treemap.py` from `main` on every push.

The accepted destination is our own purpose-built Rust game and engine. The
[architecture](docs/target-architecture/README.md) describes its contracts and
staged migration policy. Windows workers can keep the
current emulated backend and oracle for development comparisons while replacing
responsibilities. Existing run and verification commands continue to apply.

## Run

The maintained Rust code uses edition 2024. The requested latest-nightly switch
is pending local Windows administrator authorization; the usable default remains
stable. See [the toolchain status](docs/PLAN.md).

Needs SDL2 (`apt install libsdl2-dev` on Debian/Ubuntu), rustup and the
[pinned LakeSnes CPU, PPU and DSP libraries](docs/CPU_DEPENDENCY.md),
[SPC interpreter and native music libraries](docs/SPC_DEPENDENCY.md),
[configuration/utility](docs/CONFIG_DEPENDENCY.md),
[GLSL shader](docs/SHADER_DEPENDENCY.md),
[desktop host](docs/DESKTOP_DEPENDENCY.md) and
[stb_image PNG libraries](docs/PNG_DEPENDENCY.md). Build these static libraries
first and include their directory in `LIBRARY_PATH` when building or running.
Put your own unheadered dump of Super Metroid (Japan, USA) (En,Ja) at
`rom/sm.smc` (SHA-1 `da957f0d63d14cb441d215462904c4fa8519c613`; the `rom/`
folder is gitignored, never commit it), then from the repository root:

```sh
cargo run --release -- rom/sm.smc
```

Settings and key bindings are in `sm.ini`. By default arrows move, X/Z/S/A
are SNES A/B/X/Y, C/V are L/R, Enter is Start and right Shift is Select. Lines reading `Verify failure!` mean the
Rust code disagreed with the original ROM on that frame; please report them.

## Verify

Run the checkpoint corpus with the headless per-frame verifier using the
[movie instructions](movies/README.md). Its rows cover areas, bosses, item
pickups, transitions and credits. The verifier reports mismatches and required
observations, then exits nonzero on failure. A passing row checks the behavior
it exercises; it does not establish fidelity for every branch of the game.

## Credits and licence

Game logic, runtime and SNES emulation come from snesrev/sm (MIT, see
[its license](third_party/snesrev/LICENSE.txt)), translated to Rust with c2rust.
The hardware adapters link unmodified LakeSnes code under its
[CPU license](third_party/lakesnes/LICENSE.txt) and
[PPU/DSP license](third_party/lakesnes/LICENSE-2022.txt) and
[SPC license](third_party/lakesnes/LICENSE-SPC-2021.txt); include these with
distributed binaries. PNG decoding uses stb_image under its
[MIT license](third_party/stb_image/LICENSE.txt), which must also accompany distributed binaries.
Native music, configuration, utilities, shaders and the desktop host use selected Zelda3 routines with maintained SM
compatibility changes under the [Zelda3 MIT license](third_party/zelda3/LICENSE.txt);
include that notice with distributed binaries too.
Names and labels also draw on Patrick Johnston's
[bank logs](https://patrickjohnston.org/bank/). Original contributions in this
repository are MIT ([LICENSE](LICENSE)). Super Metroid is a trademark of
Nintendo; no ROM or game data is included.

Earlier approaches (static recompilation, the mixed C/Rust `restart`
workspace, hand-ported subsystems) are preserved in the branch
`archive/pre-transpile-2026-10-17`.
