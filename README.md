# sm-decomp

Super Metroid (SNES) rebuilt in Rust. The goal is a compact engine built for
this one game: deterministic, owned game state, its own renderer and sound
engine, content loaded from data, and no SNES emulation in the finished game.
It should play the original faithfully, quirks included.

Map Rando is the first use case and the acceptance bar: custom sprites and
palettes, configurable room and door connections, and opt-in mechanics such as
re-spin, all supplied through `sm_data` and explicit profiles. The
[accepted architecture](docs/target-architecture/README.md) describes the
destination.

## Migration and crate roles

The code began as a one-time c2rust translation of
[snesrev/sm](https://github.com/snesrev/sm) ([translation record](tools/transpile/README.md))
and is maintained by hand. Target responsibilities are gameplay in `sm_game`,
content in `sm_data`, rendering/audio in `sm_runtime`, the device shell in
`sm_app`, and replay/import tooling in `sm_tools`. `sm` supplies retained compat
and oracle tooling while responsibilities migrate.

For measured crate size, implementation status and retained validation evidence,
read the dated [progress report](docs/PROGRESS_REPORT.md#readme-snapshot).
[Architecture migration](docs/target-architecture/README.md#responsibility-migration)
defines each responsibility's destination and removal condition.

### How code moves

A module moves into `sm_game` once it compiles without `sm`. Where it still
needs the emulated machine, it declares a narrow host trait, and `sm`
implements that trait for `Game` or for a Runtime session lease
(`sm/src/game/session_runtime.rs`). A re-export shim stays at the old `sm`
path so remaining callers keep compiling. State that `sm_game` owns lives in
`Session`; `session_encoding.rs` projects it back into the SNES RAM layout so
the oracle comparison and checkpoints still see the original bytes.
`Imported<T>` (`sm_game::imported`) ties an `sm_data` record to the cartridge
image it was imported from.

## Build and run

You need your own copy of the game: an unheadered Super Metroid (Japan, USA)
(En,Ja) ROM at `rom/sm.smc`, SHA-1 `da957f0d63d14cb441d215462904c4fa8519c613`.
No ROM or game data is included in this repository.

The target crates build and test with plain Cargo on any platform:

```sh
cargo test -p sm_data -p sm_game -p sm_runtime
```

The desktop game and the verifier currently build only for Windows x86-64 with
the GNU toolchain. They need pinned native libraries built with Zig, SDL2 and
Python 3.11+; the [build guide](docs/ITERATION.md) covers setup, controls and
options. Once those are installed:

```sh
cargo run --release -p sm_app -- rom/sm.smc
```

## Verification

The verifier replays recorded inputs in lockstep. Each frame starts from the
same state and input: the original ROM runs on the emulated 65816 ("theirs"),
the Rust game runs on the same emulated hardware ("mine"), and the results are
compared. Every frame it checks WRAM, SRAM, VRAM, CGRAM, OAM, PPU registers and
port state, and the rendered 256x240 image. Any difference fails the run. The
[verification guide](movies/README.md) explains how to run it locally.

Read the workflow definitions for their current triggers and selections:
[PR CI](.github/workflows/ci.yml), [corpus](.github/workflows/segments.yml),
[full route](.github/workflows/full-route.yml) and [coverage](.github/workflows/coverage.yml).
The [report](docs/PROGRESS_REPORT.md#workflow-inventory-snapshot) retains the old
inventory; selected-frame counts alone do not prove complete movie coverage.

The ROM for CI comes from repository secrets. The ROM import check confirms
that every `sm_data` record accepts the real cartridge; a record that rejected
it would leave the game on its old ROM reads and the replays would not exercise
it. A change lands after PR CI and a full corpus pass on its exact head; main
is then fast-forwarded.

What this does not cover:

- Only paths the inputs reach are checked. The coverage workflow lists the
  code the corpus never runs.
- The image comparison is not independent: both sides draw through the same
  emulated PPU, so a renderer fault they share goes unnoticed. Independent
  pixel checks exist only for selected cases.
- Audio is not compared by default. The SPC700/DSP state is compared only
  with `--audio-trace` or `--audio-trace-directory`.
- A fixed list of WRAM ranges is normalised before comparison
  (`normalize_snapshot_ram` in `sm/src/cpu_infra.rs`); those bytes are not
  checked by the comparison.
- The desktop build and the verifier run only on Windows.
- Self-consistency of the target crates is not fidelity: the target-engine
  gates in [VALIDATION](docs/target-architecture/VALIDATION.md) require their own
  evidence; consult the dated [report](docs/PROGRESS_REPORT.md#architecture-snapshot).

## Progress

Progress means behaviour moving into the target crates, and moved code no
longer reaching into the compat RAM image or hardware registers.
`python3 tools/cycle_report.py` prints both: line counts per target crate,
the coupling that ties `sm/src/game/` to the emulator, and
"sm_game files still on compat RAM/registers" with the number of sites per kind
of access. Code in `sm_game` that still reads the RAM image through those macros
is relocated, not yet owned. `sh tools/progress.sh` counts transpiler residue
(unsafe functions, C ABI exports, casts, legacy bank lines). CI prints both.

For source-bound measurements, read the [progress report](docs/PROGRESS_REPORT.md#source-count-snapshot).
Rerun the accounting tools for current numbers; #6 is the sole daily report.

## Documentation

| Read | For |
| --- | --- |
| [Build guide](docs/ITERATION.md) | Setup, build, run, direct replays and captures |
| [Verification guide](movies/README.md) | Corpus rows, replay data, CI checks |
| [Native dependencies](docs/NATIVE_DEPENDENCIES.md), [PNG decoder](docs/PNG_DEPENDENCY.md) | Pinned native libraries and how to build them |
| [Accepted architecture](docs/target-architecture/README.md) | Destination packages, contracts and validation gates |
| [State migration](docs/ENGINE_STATE_MIGRATION.md) | Rules for moving state out of RAM |
| [Plan](docs/PLAN.md), [Review](docs/REVIEW.md) | Current work order, milestones and process |
| [Archive](docs/archive/README.md) | Retired documents |

## Credits and licence

The following projects supply translated or linked code; their applicable
copyright and license notices are retained in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

| Project | Code used here |
| --- | --- |
| [snesrev/sm](https://github.com/snesrev/sm/tree/578f90b3cc49557bb70060ad033bb90b8cf8ac50) | One-time Rust translation of gameplay, runtime and SNES emulation; [translation record](tools/transpile/README.md). |
| LakeSnes: [CPU](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965), [PPU](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18), [SPC700](https://github.com/elzo-d/LakeSnes/tree/1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29) | The CPU and PPU are pinned native C, fetched by [the installer](tools/install_lakesnes.py) and linked into builds. The SPC700 and DSP now run as Rust in `sm/src/snes/`. |
| [snesrev/zelda3](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf) | Selected configuration, utility, shader and desktop routines, fetched and patched by the native installers, with bridges in `sm/src/native_*.c`. |
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
