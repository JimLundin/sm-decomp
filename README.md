# sm-recomp

A static recompilation of Super Metroid (SNES) to Rust — the 65816 CPU code is
translated to Rust ahead of time; the PPU, audio and DMA stay emulated.
"Recompile the CPU, emulate the silicon."

**Status: skeleton.** The recompiler, runtime and test harnesses build and pass
their tests on a built-in demo program. Nothing runs Super Metroid yet.

You need your own ROM. Nothing derived from it (the ROM, generated code, traces)
is ever committed — see `.gitignore`.

## Layout

| Crate | What it does | Status |
|---|---|---|
| `runtime` | CPU registers, `Bus` trait, call dispatch, ALU ops, addressing modes, fallback hook, miss log | working |
| `recomp` | ROM → decode → control-flow graph (tracks M/X widths) → Rust source | working, needs real-ROM shakedown |
| `game` | Generated code + hand-written overrides + hardware glue; builds a demo program when no ROM output exists | demo only |
| `diff` | Lockstep frame-by-frame comparison against a reference emulator | harness done, consoles not wired |
| `trace` | Grows `config/functions.toml` from miss logs; fills names from a symbol map | working |
| `movie` | BizHawk `.bk2` → plain per-frame joypad list | working; `.lsmv` TODO |
| `fuzz` | Turns SingleStepTests 65816 JSON into Rust tests through the real emitter | working |

## Quick start

```sh
cargo test --workspace          # includes an end-to-end demo: bytes → Rust → run → check RAM
cargo run -p game               # runs the demo program

# with your ROM at rom/sm.sfc and some functions in config/functions.toml:
cargo run -p recomp             # writes crates/game/generated/generated.rs
cargo build -p game             # picks up the generated code automatically

# instruction-level check against the 65816 test corpus:
cargo run -p fuzz -- gen path/to/65816/v1 20 && cargo test -p fuzz
```

See [docs/PIPELINE.md](docs/PIPELINE.md) for the loop and the milestones.

## Built on / borrowed from

- [snesrev/sm](https://github.com/snesrev/sm) and [sm_rewrite](https://github.com/testyourmine/sm_rewrite) — function names and boundaries, and the frame-compare idea.
- [Bank logs](https://patrickjohnston.org/bank/) — labels, jump tables, RAM map.
- [snesrecomp](https://github.com/mstan/snesrecomp) — prior art for SNES static recompilation (Python → C).
- [RustySNES](https://github.com/doublegate/RustySNES) — planned hardware backend, fallback CPU and reference emulator.
- [MesenCE](https://github.com/nesdev-org/MesenCE) — debugger/tracer and second reference.
- SingleStepTests 65816 — per-instruction test vectors.
