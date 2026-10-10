# Project layout

The project recompiles Super Metroid's 65816 CPU code ahead of time and retains
emulated SNES hardware. One Cargo workspace owns all Rust crates, dependency
versions, and build output. `rust-toolchain.toml` pins `nightly-2026-09-29` so
local builds and CI use the same compiler; bump the date deliberately and rerun
formatting, Clippy, tests, and both emulator smoke checks. The workspace uses
edition 2024, resolver 3, and disables publishing for project crates.

## Crate responsibilities

| Crate | Owns | Dependencies within the project |
|---|---|---|
| `runtime` | CPU state, instruction semantics, addressing, dispatch, miss accounting; `Bus` and `Fallback` interfaces | None |
| `recomp` | LoROM reads, decoding, control-flow discovery, config parsing, Rust emission | None; emitted source calls `runtime` |
| `game` | Generated function table, handwritten overrides, construction of the recompiled machine; demo RAM bus | `runtime`; `recomp` at build time for the ROM-free demo |
| `snes` | Deterministic reference execution, shared SNES hardware bus, interpreter fallback, mixed generated/interpreted subject | Pinned RustySNES core; `runtime`; `movie` for ROM identity |
| `diff` | Input-driven lockstep CPU/clock/WRAM/video comparison, first-divergence reports, discovery misses | `movie`; `snes`; `game` table; comparison library is console-independent |
| `movie` | Movie import and per-frame controller input serialization | None |
| `trace` | Merge fallback misses into function config; apply symbol names | `recomp` |
| `fuzz` | Generate instruction tests through the real emitter | `recomp`; `runtime` in tests |

`runtime::Bus` and `runtime::Fallback` are the seams between compiled CPU code
and the emulated hardware. `diff::Console` is the seam between the comparison
harness and each runnable console. Hardware knowledge belongs in `snes`, CPU
semantics in `runtime`, and ROM-specific fixes in `game::overrides` or config.
The reference's interface exposes frame execution, WRAM, and framebuffer reads;
callers do not need to manage RustySNES's bus, cartridge, and reset ordering.

`SnesBus` owns a shared hardware handle. Generated routines and
`InterpreterFallback` use the same memory map, DMA, clocks, and interrupt
hooks. The subroutine fallback copies registers into RustySNES's CPU and
executes until the caller's return address is consumed. It supports native
mode and conventional balanced RTS/RTL returns; stack manipulation requires
overrides.

`Subject::fallback_only` runs the full interpreter through `SnesBus`, including
boot in emulation mode, and records native JSR/JSL targets and width variants.
`Subject::with_steps` accepts `game::steps::STEP_TABLE` and executes one compiled
instruction at a time, preserving hardware bus fetches and cycles. It polls
interrupts before compiled execution and delegates interrupt entry to the
interpreter without consuming NMI twice. Unsupported instructions stay
interpreted. The resume point is the next CPU instruction, so calls, returns,
wait loops, and frame yields retain the actual hardware stack and state.

The compiled whitelist lives in `recomp::step`; bodies use the existing
`emit_straight` semantics. `runtime::TimedBus` charges exact access timing.
The whole-subroutine table is separate and remains untimed. `diff --generated`
requires a nonempty instruction table, matching ROM identity, and actual
compiled execution, then compares registers, CPU/master clocks, WRAM, and
framebuffer each frame. `--fallback-only` checks the empty table path.

## Files and artifacts

```text
Cargo.toml / Cargo.lock       shared workspace and pinned dependency graph
rust-toolchain.toml           dated nightly + rustfmt + Clippy
rustfmt.toml                  shared Rust formatting
justfile                      setup, checks, generation, smoke runs
crates/                       maintained Rust implementations and their tests
config/                       reviewed function entries, widths, jump targets
docs/                         architecture, setup, pipeline milestones
tools/                        download/setup helpers, label conversion, Mesen wrapper/Lua
rom/sm.sfc                    local ROM input (ignored)
movies/                       local TAS movies and converted inputs (ignored)
deps/                         downloaded reference material, corpus, emulator runtime (ignored)
out/generated/game.rs         ROM-derived Rust consumed by game/build.rs (ignored)
out/generated/steps.rs        ROM-derived instruction table consumed by game (ignored)
out/generated/fuzz.rs         corpus-derived tests consumed by fuzz/build.rs (ignored)
out/                          symbols, trace logs, diff reports, probe snapshots (ignored)
target/                       Cargo build products and OUT_DIR copies (ignored)
```

Build scripts resolve generated inputs relative to their crate manifests and
watch the exact file path. Creating, changing, or removing an input triggers a
rebuild. With no generated game input, `game` builds its miniature demo; with no
generated fuzz input, `fuzz` only checks the harness. Generators are invoked
from the repository root. `just` ensures that working directory automatically.

RustySNES's base is a commit-pinned Cargo git dependency. A root Cargo patch
selects its verified local source plus the documented HDMA fix, clocked APU bus, and explicit lsnes recording profile.
Run `just setup` or `python3 tools/prepare_rustysnes.py` before Cargo in a clean
checkout; CI runs the latter. It still builds without a commercial ROM or
movie data. Setup downloads the other material used during investigation and
validation. Never commit the ROM,
ROM-derived code, traces, or downloaded upstream trees.

## Checks

`just check` runs formatting and workspace tests. `just lint` runs Clippy with
warnings treated as failures. CI runs both using the pinned nightly and locked
dependencies, without commercial inputs. Tests live beside the implementation
or in each crate's `tests/`; the `fuzz` crate owns the external instruction
corpus tests. `just reference-smoke` and `just mesen-smoke` require the local
ROM and verify that each reference advances frames. Full TAS synchronization
and frame equivalence remain the acceptance checks for the recompilation loop.
