# Environment setup

Run commands from the repository root. The Rust toolchain is pinned to
`nightly-2026-09-29` in `rust-toolchain.toml`, including rustfmt and Clippy.
All crates use edition 2024 and Cargo resolver 3. The workspace needs Rust and a C linker; the
headless RustySNES core does not need SDL, a display, or audio devices.

```sh
just setup                       # download, verify SHA-256, extract, build
just doctor                      # check tools, ROM identity, and local inputs
just check                       # formatting and workspace tests, offline
just lint                        # Clippy for all workspace targets, warnings fail
just reference-smoke             # load rom/sm.sfc and advance 120 frames
just mesen-smoke                  # independently advance 120 frames in MesenCE
just setup-lsnes                  # build the original TAS recording core locally
just sync-smoke                   # verify its capture path on the passing boot prefix
```

The Rust bootstrap requires `rustup`, `cargo`, `cc`, `curl`, `tar`,
`unzip`, `patch`, and `dpkg-deb`; `just` provides the shortcuts. These tools were already
installed on this host. Without `just`, use the Rust `tools/bootstrap` commands
`setup` and `doctor`. `setup --offline` reuses cached
downloads and Cargo packages. `--skip-mesen` prepares the Rust tools and corpus
on hosts other than Linux x64.

## Local dependencies

Versions, URLs, and archive checksums live in `tools/dependencies.toml`.
External archives and reference material are in gitignored `deps/`; Cargo
packages and git dependencies use Cargo's normal cache. `rustysnes-core` is
pinned by git revision in the root `Cargo.toml` and the shared `Cargo.lock`.
The root Cargo patch uses the verified `deps/rustysnes` source with the
HDMA, clocked APU, DMA alignment, interrupt timing, and lsnes recording-profile patches. Before running Cargo in a fresh checkout,
run `just setup` or
`cargo run --manifest-path tools/bootstrap/Cargo.toml --locked -- prepare-rustysnes`.
The independent bootstrap package resolves before the root workspace. CI uses it too.
The base revision stays pinned in `Cargo.toml` and `tools/dependencies.toml`;
the maintained difference is [the timing patch](../tools/patches/README.md).
Setup does not modify system libraries or install global packages.

| Dependency | Local path | Purpose |
|---|---|---|
| RustySNES, commit `98f8d398` plus hardware timing patches | `deps/rustysnes` | Reference emulator and hardware/fallback backend |
| snesrev/sm, commit `578f90b3` | `deps/snesrev-sm` | Function labels and reference material |
| SingleStepTests/65816, commit `db6b1040` | `deps/65816/v1` | All 256 native-mode opcode files, 10,000 cases each |
| MesenCE 2.2.1, Linux x64 | `deps/mesen/Mesen` | Independent debugger and frame oracle |
| Private glibc 2.35, SDL2, Wayland libraries, patchelf | `deps/mesen-runtime`, `deps/patchelf` | Run MesenCE on this Ubuntu 20.04 host |
| lsnes rr2-β23 / bsnes v085 Compatibility | `deps/lsnes/source` | Original TAS synchronization baseline |
| Private Boost 1.71 and Lua 5.2 development/runtime packages, curl headers | `deps/lsnes-build-deps` | Build the recording core without global installs |

The corpus archive also contains emulation-mode tests; setup extracts only the
native-mode files consumed by `fuzz`. Downloads and extracted sources occupy
about 2.5 GB, plus build outputs.

MesenCE's release requires newer glibc than Ubuntu 20.04's 2.31. Setup patches
the downloaded binary's interpreter to the private loader, and `tools/mesen`
selects private library and configuration directories. Always launch through
that wrapper. Rerun setup after moving the repository: the patched interpreter
uses an absolute path. MesenCE can restart itself during first-run setup; the
patched interpreter keeps that restart on the same runtime. Headless test mode
uses `--testRunner`, a Lua script, and a ROM. GUI mode requires a working display.

`just setup-lsnes` downloads and verifies the original source archive and the
private development packages recorded in `docs/lsnes-dependencies.toml`,
applies build compatibility fixes, and builds the headless recording core.
This recipe targets Ubuntu 20.04 x86_64 and requires host `g++`, `make`,
`pkg-config`, libarchive, zlib headers, and the libcurl OpenSSL runtime.
Use `cargo run --manifest-path tools/bootstrap/Cargo.toml --locked --offline -- setup-lsnes --offline` for cached rebuilds. The build
log is `deps/lsnes/build.log`. Rerun after moving the repository to refresh
private include/library paths. See [LSNES_BASELINE.md](LSNES_BASELINE.md).

## ROM and labels

`rom/sm.sfc` is the unheadered 3 MiB Super Metroid (Japan, USA) (En,Ja) image.
Its expected SHA-1 is `da957f0d63d14cb441d215462904c4fa8519c613`; `doctor`
checks it. Setup never downloads the ROM.

`just symbols` converts snesrev's ROM labels to `out/symbols.toml`, which
`just trace-names` can consume. This contains data labels as well as function
labels; it names discovered entries and does not infer function boundaries.

## Recompilation commands

```sh
just demo                        # built-in program before subroutine output is generated
just recompile                   # emit whole-subroutine Rust, build game
just iterate                     # generate resumable steps, build, compare first 9000 frames
just sync-check                  # strict original-core 12000-frame gate
just trace-merge out/run.miss.csv # add discovered function/width variants
just trace-names                  # name entries using out/symbols.toml
just movie movies/run.bk2 movies/run.inputs
just import-movies               # import all verified downloaded LSMV movies
just diff-fallback movies/super-metroid-any-3653.lsmv 3600 out/diff-any
just reference-replay movies/super-metroid-any-3653.lsmv 9000 out/reference-any.csv
just mesen-replay movies/super-metroid-any-3653.inputs 9000
just fuzz 1                      # one case per opcode as a quick semantics probe
just fuzz 20                     # larger instruction-level probe
```

`config/functions.toml` starts with the `WaitForNMI` slice.
Writing `out/generated/game.rs` switches `game` away from its demo and skips
the demo-specific tests. `out/generated/steps.rs` is the separate instruction
table consumed by `diff --generated`; its bodies preserve exact access timing
for a small tested whitelist. Unsupported instructions stay interpreted. To check generation without
switching modes, use:

```sh
cargo run -p recomp --locked --offline -- --out out/setup-generated.rs
```

`just fetch-movies` downloads three published Super Metroid TAS inputs matching
the local ROM: any%, low% (13%), and 100%. Their sources and checksums are in
`movies/manifest.toml`; details are in [TAS_MOVIES.md](TAS_MOVIES.md). These are
lsnes `.lsmv` files. `just import-movies` verifies each archive and its frame
count, then writes `.inputs`. `movie` also accepts BizHawk `.bk2`, extracted
logs, and `.inputs`. The frame runner supports clean starts with one P1
gamepad sample per frame; LSMV reset, subframe, savestate, and customized
initialization records are rejected. Replay directly from `.lsmv` to retain
the container's ROM SHA-256 check; `.inputs` carries button masks only.
Keep movies and converted inputs in `movies/` (gitignored).

`diff-fallback` and `diff-generated` compare CPU registers/cycles, master
clock, WRAM, and framebuffer after each frame and write
`misses.csv`, `coverage.txt`, plus `match.txt` or `divergence.txt` under the
output directory. Instruction coverage counts show actual compiled work;
call-target logs record interpreted JSR/JSL discovery targets.
Frame offsets in divergence reports are zero-based. It replaces prior verdict
files when reusing a directory. Reference replay writes periodic WRAM and
framebuffer SHA-256 fingerprints, game state, and room ID to CSV. Omit
`--frames` from the underlying Cargo commands to play a whole movie.

Mesen replay explicitly enables the P1 SNES controller and disables P2 via
command-line settings, without saving those overrides. It indexes the movie
by controller polls and checks that injection succeeds. Mesen's first
`endFrame` occurs before its first input poll; its printed frame counter is
therefore not an exact RustySNES comparison boundary. The helper stops after
the requested number of input polls, rather than dropping the last input.
These independent runs check game milestones, not byte-for-byte equivalence.

## Verified on this host

On 2026-10-09, `nightly-2026-09-29` built the unified workspace and RustySNES core. The workspace
baseline tests and built-in demo passed. RustySNES loaded the verified ROM and
advanced 120 frames with seed 0 and no controller input; its WRAM FNV-1a
fingerprint was `32db39b95b8bcf3d`. MesenCE also advanced 120 frames and exited
successfully through the Lua smoke script. These are environment checks, not
an accuracy comparison or proof that a full TAS synchronizes.

The instruction probe (`just fuzz 1`) generated 226 cases, skipping 30 control
flow/unsupported cases. 222 generated cases passed; four ADC cases failed
because the overflow flag differed (`6f`, `71`, `75`, `7f`). The extra harness
test passed. This agrees with the existing decimal-mode overflow TODO in
`runtime::ops`; fixing instruction semantics is subsequent implementation work.
The sampled generated cases are preserved locally in `out/fuzz-smoke-cases.rs`
and removed from `out/generated/fuzz.rs` so baseline checks still run.
Running `just fuzz` activates its cases, and `just check` then runs those too.

The workspace now also passes movie import, region-layout validation,
SNES memory-map routing, fallback-only frame execution, and compiled-to-fallback
subroutine tests. The generated any% prefix matched CPU/clocks, WRAM, and
framebuffer for 9,000 frames, executing 25,359 generated instructions, including
entry into Ceres gameplay; low% and 100%
each matched 600. These checks validate the adapter path against the same
RustySNES backend. The independent Mesen run reached the same sampled intro,
new-game, and Ceres gameplay milestones through frame 9,000. See
[TAS_MOVIES.md](TAS_MOVIES.md) for replay observations and remaining limits.

Generated frame dispatch now resumes per instruction and preserves interrupt
polling and bus timing. The initial whitelist is immediate LDA/LDX, REP/SEP,
absolute STA, NOP, and DEX. Broader instruction coverage and full TAS
synchronization remain subsequent work. The strict 12,000-frame lsnes gate
passes after checkpoint-label, HDMA, and APU profile corrections. A further
HDMA correction stops overhead after all enabled channels terminate; fresh
extended validation found frame-boundary differences after frame 18,000.
DMA alignment and interrupt timing corrections are maintained as pinned patches;
full TAS acceptance is still pending.
See [PIPELINE.md](PIPELINE.md) and [LSNES_BASELINE.md](LSNES_BASELINE.md).
