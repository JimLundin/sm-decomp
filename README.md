# sm-decomp

Super Metroid (SNES) in Rust. The whole game runs as Rust code on emulated
SNES hardware, and every frame can be checked against the original ROM
running on the built-in 65816 emulator.

The code was transpiled once from [snesrev/sm](https://github.com/snesrev/sm)
with c2rust and is now being cleaned up into idiomatic Rust, one subsystem at
a time, with that per-frame comparison as the test. See
[docs/PLAN.md](docs/PLAN.md).

The accepted destination is our own purpose-built Rust game and engine. The
[architecture handoff](docs/target-architecture/README.md) includes a standalone
interactive report and a staged migration plan. Windows workers can keep the
current emulated backend and oracle for development comparisons while replacing
responsibilities. Existing run and verification commands continue to apply.

The earlier [interactive architecture atlas](docs/architecture.html) records
current construction, migration experiments, and observed frame execution order.
Both reports work offline without a build step.

## Run

Needs SDL2 (`apt install libsdl2-dev` on Debian/Ubuntu), rustup and the
[pinned LakeSnes CPU, PPU and DSP libraries](docs/CPU_DEPENDENCY.md),
[SPC interpreter and native music libraries](docs/SPC_DEPENDENCY.md),
[configuration/utility](docs/CONFIG_DEPENDENCY.md),
[GLSL shader](docs/SHADER_DEPENDENCY.md),
[desktop host](docs/DESKTOP_DEPENDENCY.md) and
[stb_image PNG libraries](docs/PNG_DEPENDENCY.md). Build these static libraries
first and include their directory in `LIBRARY_PATH` when building or running.
Put your own ROM at `rom/sm.smc` (see [rom/README.md](rom/README.md)), then
from the repository root:

```sh
cargo run --release -- rom/sm.smc
```

Settings and key bindings are in `sm.ini`. By default arrows move, X/Z/S/A
are SNES A/B/X/Y, C/V are L/R, Enter is Start and right Shift is Select. Lines reading `Verify failure!` mean the
Rust code disagreed with the original ROM on that frame; please report them.

## Verify

Import a TAS movie and run the headless per-frame comparison using the
[movie instructions](movies/README.md). The verifier reports mismatches and
gameplay coverage, then exits nonzero on failure. The imported any% movie
currently visits only six rooms and does not reach credits; a passing replay
checks the behavior it exercises, not the entire game.

## Credits and licence

Game logic, runtime and SNES emulation come from snesrev/sm (MIT, see
[LICENSE-snesrev](LICENSE-snesrev)), translated to Rust with c2rust.
The hardware adapters link unmodified LakeSnes code under its
[CPU license](third_party/lakesnes/LICENSE.txt) and
[PPU/DSP license](third_party/lakesnes/LICENSE-2022.txt) and
[SPC license](third_party/lakesnes/LICENSE-SPC-2021.txt); include these with
distributed binaries. PNG decoding uses stb_image under its
[MIT license](LICENSE-stb_image), which must also accompany distributed binaries.
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
