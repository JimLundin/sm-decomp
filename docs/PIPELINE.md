# The pipeline

```
 TAS movies ──movie──▶ .inputs ─────────────────────────────┐
                                                             ▼
 ROM ──recomp(config)──▶ generated.rs ──game──▶ subject ──diff──▶ first divergence
                ▲                                  │            report
                │                                  ▼               │
          functions.toml ◀──trace merge── misses.csv               │
                ▲                                                  │
                └───────────── fix config / emitter / override ◀───┘
```

## Start iterating

```sh
just iterate                                      # regenerate, build, compare any% first 9000 frames
just diff-fallback movies/super-metroid-any-3653.lsmv 9000 out/fallback
just diff-generated movies/super-metroid-low-3273.lsmv 9000 out/generated-low
just sync-smoke                                   # original-core early boot comparison
just sync-check                                   # original-core 12000-frame acceptance gate
just sync-full                                    # full any% recording
just sync-replay out/sync/lsnes.csv                 # reuse a completed baseline after a hardware edit
just check
just lint
```

The selected slice starts at `WaitForNMI` in `config/functions.toml`.
`recompile-steps` emits both the original subroutine table and a resumable
instruction table in `out/generated/`. The frame runner uses the instruction
table: it executes supported instructions as compiled Rust, preserving opcode
fetches, bus-cycle timing, register state, and interrupt polling. Other
instructions run on the interpreter, including calls and returns. It stops at
each hardware frame and compares CPU registers/cycles, master clock, WRAM, and
framebuffer. Compiled input bytes and the ROM SHA-256 are checked before use.
A run with no compiled instructions executed fails instead of reporting a
misleading success.

Each output directory contains `match.txt` or `divergence.txt`, `coverage.txt`,
and `misses.csv`. Divergence frame indices are zero-based. Generated/interpreted
instruction counts are the current progress metric. Call-target logs still
feed function discovery; they record interpreted JSR/JSL targets, including
calls to partly compiled functions, so they are not a generated-call miss rate.

Add one function or instruction timing pattern at a time, run the same prefix,
then extend the movies and frame limits. Start with the shortest prefix that
covers the changed code. `just iterate` never expands the function config
implicitly. The whole-subroutine table remains available for routine tests;
its untimed bodies are not used in the frame runner.

## The loop

1. `recompile-steps` regenerates Rust from `config/functions.toml` and builds `diff`.
2. `diff-generated` plays a selected movie through the reference and the compiled/interpreted subject in lockstep.
3. At the first differing frame it writes a report (frame, region, bytes, misses).
4. Triage:
   - **missing function or width variant** → `trace merge` adds it from `misses.csv`
   - **`assume_mx` panic** → a routine returns with different widths than it was
     called with; add the variant or write an override
   - **unknown indirect target** → add to `config/jump_tables.toml`
   - **emitter/ops bug** → fix it, add a fuzz case that reproduces it
   - **timing** (raster effects, HUD IRQ) → scheduler work or an override
5. Repeat.

The final acceptance criterion is all movies completing with zero divergences
and zero interpreted instructions, plus synchronization with the original
recording core. Until then, retain the interpreter and
report the tested prefix and compiled instruction count.

To inspect discovery before changing the reviewed config:

```sh
cp config/functions.toml out/discovery.functions.toml
cargo run -p trace --locked --offline -- merge out/discovery.functions.toml out/diff-generated/misses.csv
cargo run -p trace --locked --offline -- names out/discovery.functions.toml out/symbols.toml
cargo run -p recomp --locked --offline -- --functions out/discovery.functions.toml --out out/discovery-generated.rs
```

The initial any% 3,600-frame and low%/100% 600-frame logs yielded 410 entries
and 416 width variants; 402 entries matched the downloaded labels. Emitting
those entries produced Rust that compiled in an isolated local crate, with
nine unsupported BRK stops. Those stops may reflect incomplete width or
control-flow assumptions and need investigation. This verifies the plumbing;
it does not validate those generated routines against the ROM. The reviewed
`config/functions.toml` now contains only the initial `WaitForNMI` slice.

## Milestones

Current status: input import, hardware adapters, instruction-sized generated
execution, discovery logs, and frame comparison are implemented. A 19,000-frame
mixed execution prefix passes with the HDMA timing patch. The independent
lsnes baseline revealed a frame-label error, missing per-channel HDMA clocks,
and APU phase drift during the Ceres sound upload. Setup maintains the HDMA
correction, clocked APU bus, and explicit lsnes rate profile. All 10,472 strict
checkpoints through 12,000 any% frames now pass; subsequent DMA/HDMA and
interrupt corrections extend strict agreement through 18,000 frames (15,184
events). The fresh 36,000-frame run still first differs at frame 18,195.
`sync-check` defaults to the strict 12,000-frame gate. `just probe-hdma` provides
a fast independent diagnostic for the remaining implied-cycle/HDMA timing
failure; branch-only probes pass while some NOP phases fail by a few clocks.
Full TAS completion remains an acceptance milestone. See
[LSNES_BASELINE.md](LSNES_BASELINE.md).

1. **Reference emulator.** Resolve the strict original-core checkpoint failure,
   then play a full any% TAS to the end. Compare initialized semantic fields
   with lsnes; whole WRAM hashes differ across cores' power-on initialization.
   `just reference-replay MOVIE FRAMES` records WRAM/video fingerprints and
   game milestones; `just mesen-replay INPUTS FRAMES` checks independent
   progression with controller injection validated.
2. **Harness with fallback only.** Implement the SNES bus adapter in `crates/snes` on RustySNES's
   `Bus` and its 65816 core as `runtime::Fallback`. With an empty function table
   the subject *is* the reference, so diff must report zero divergences. This
   proves the harness before any generated code is trusted.
   `just diff-fallback MOVIE FRAMES OUT_DIR` writes CPU/clock/WRAM/video verdicts and misses.
3. **Discovery.** Run all movies with the fallback, collect `misses.csv`,
   `trace merge` into `functions.toml`, `trace names` from a symbol map built
   from snesrev/sm and the bank logs.
4. **First bank.** Generate bank $80 only (common routines), diff, fix.
5. **Bank by bank.** Expand; fallback share falls toward zero.
6. **Overrides.** NMI wait loop → scheduler yield; raster-timed code; anything
   that manipulates the stack to change its return address.

Crate responsibilities and generated artifact paths are documented in
[ARCHITECTURE.md](ARCHITECTURE.md).

## Known hard parts

- **Widths.** Each (entry, M, X) is its own function. `cfg` tracks REP/SEP
  statically; after PLP and after calls it assumes widths are restored and the
  emitted code checks that at runtime (`assume_mx`).
- **Stack tricks.** Code that pops its own return address or pushes a fake one
  to "return" into a jump table. Detected as diffs; handled with overrides.
- **Timing.** The instruction table currently supports immediate LDA/LDX,
  REP/SEP, absolute STA, NOP, and DEX. These patterns have hardware tests.
  New patterns need exact access ordering and interrupt/frame tests before
  entering the whitelist in `recomp::step`. Whole-subroutine output is untimed.
- **Decimal mode V flag** in `runtime::ops` is unverified until the fuzz corpus passes.
- **Movie formats.** The importer supports clean-start P1 gamepad records
  with one sample per frame in lsnes `.lsmv`, plus BizHawk `.bk2`. Reset,
  subframe, and stateful input needs a richer runner. Check that a movie syncs
  on the reference emulator before using it.
