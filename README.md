# sm-recomp

A static recompilation of Super Metroid (SNES) to Rust — the 65816 CPU code is
translated to Rust ahead of time; the PPU, audio and DMA stay emulated.
"Recompile the CPU, emulate the silicon."

**Status: recompilation loop ready for incremental validation.** `just iterate`
generates an instruction table, builds, and compares mixed compiled/interpreted
execution against the reference. Timing support starts with a small whitelist;
full TAS synchronization remains an independent acceptance milestone.
The original lsnes baseline is installed; **10,472 strict checkpoints pass
through frame 12,000** after frame-label, HDMA, and APU recording-profile fixes.
A further HDMA correction fixes the door-transition failure at frame 17,976.
Strict validation has passed through frame 18,000; later timing differences
remain under investigation. A fresh 36,000-frame replay first differs in
checkpoint label at frame 18,195 and position at frame 35,203. IRQ mask
sampling is corrected; `just probe-hdma` reproduces a remaining HDMA timing
disagreement with synthetic cartridges. See [docs/LSNES_BASELINE.md](docs/LSNES_BASELINE.md).

You need your own ROM. Nothing derived from it (the ROM, generated code, traces)
is ever committed — see `.gitignore`.

## Layout

| Crate | What it does | Status |
|---|---|---|
| `runtime` | CPU registers, `Bus` trait, call dispatch, ALU ops, addressing modes, fallback hook, miss log | working |
| `recomp` | ROM → decode → control-flow graph (tracks M/X widths) → Rust source | working, needs real-ROM shakedown |
| `game` | Generated code + hand-written overrides; builds a demo program when no ROM output exists | resumable instruction table and subroutine output |
| `snes` | SNES hardware, interpreter fallback, reference and subject runners | compiled/interpreted frames, hardware timing and interrupt tests work |
| `diff` | Lockstep frame-by-frame comparison against a reference emulator | CPU/clock/WRAM/video comparison and discovery reports work |
| `trace` | Grows `config/functions.toml` from miss logs; fills names from a symbol map | working |
| `movie` | lsnes `.lsmv` / BizHawk `.bk2` → per-frame joypad list | clean-start P1 movies supported; unsupported records rejected |
| `fuzz` | Turns SingleStepTests 65816 JSON into Rust tests through the real emitter | working |

## Quick start

The workspace uses pinned **nightly Rust** (`nightly-2026-09-29`), edition 2024.
For dependency setup on Linux x64, run `just setup`, then `just doctor`.
See [docs/SETUP.md](docs/SETUP.md) for the pinned downloads, emulator smoke checks,
and commands for the recompilation loop.

```sh
cargo test --workspace          # includes an end-to-end demo: bytes → Rust → run → check RAM
cargo run -p game               # runs the demo program

just fetch-movies               # verified any%, low%, and 100% TAS recordings
just import-movies              # convert the corpus to .inputs
just iterate                   # generate, build, compare any% first 9000 frames
just setup-lsnes                # build the original TAS recording core locally
just sync-smoke                 # passing early boot prefix
just sync-check                 # 12000-frame TAS gate
just diff-fallback movies/super-metroid-any-3653.lsmv 3600
just reference-replay movies/super-metroid-any-3653.lsmv 9000
just mesen-replay movies/super-metroid-any-3653.inputs 9000

# with your ROM at rom/sm.sfc and some functions in config/functions.toml:
cargo run -p recomp             # writes out/generated/game.rs
cargo build -p game             # picks up the generated code automatically

# instruction-level check against the 65816 test corpus:
cargo run -p fuzz -- gen path/to/65816/v1 20 && cargo test -p fuzz
```

See [docs/PIPELINE.md](docs/PIPELINE.md) for the loop and the milestones.
See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for crate responsibilities,
dependency direction, and artifact locations.

## Built on / borrowed from

- [snesrev/sm](https://github.com/snesrev/sm) and [sm_rewrite](https://github.com/testyourmine/sm_rewrite) — function names and boundaries, and the frame-compare idea.
- [Bank logs](https://patrickjohnston.org/bank/) — labels, jump tables, RAM map.
- [snesrecomp](https://github.com/mstan/snesrecomp) — prior art for SNES static recompilation (Python → C).
- [RustySNES](https://github.com/doublegate/RustySNES) — hardware backend, fallback CPU and reference emulator.
- [MesenCE](https://github.com/nesdev-org/MesenCE) — debugger/tracer and second reference.
- [lsnes rr2-β23](https://tasvideos.org/EmulatorResources/Lsnes) — original TAS recording core and synchronization baseline.
- SingleStepTests 65816 — per-instruction test vectors.
