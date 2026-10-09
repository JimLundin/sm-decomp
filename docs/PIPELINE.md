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

## The loop

1. `recomp` regenerates Rust from `config/functions.toml`.
2. `diff` plays every movie through the reference and the subject in lockstep.
3. At the first differing frame it writes a report (frame, region, bytes, misses).
4. Triage:
   - **missing function or width variant** → `trace merge` adds it from `misses.csv`
   - **`assume_mx` panic** → a routine returns with different widths than it was
     called with; add the variant or write an override
   - **unknown indirect target** → add to `config/jump_tables.toml`
   - **emitter/ops bug** → fix it, add a fuzz case that reproduces it
   - **timing** (raster effects, HUD IRQ) → scheduler work or an override
5. Repeat.

Progress metric: share of calls served by generated code
(`Machine::generated_calls` vs `Machine::misses`). Done when every movie plays
to the end with zero divergences and zero fallback calls; then the fallback CPU
can be removed from release builds.

## Milestones

1. **Reference emulator.** Wrap RustySNES as a `diff::Console`. Play a full
   any% TAS to the end. Cross-check WRAM hashes against MesenCE at sample frames.
2. **Harness with fallback only.** Implement `game::hw::SnesBus` on RustySNES's
   `Bus` and its 65816 core as `runtime::Fallback`. With an empty function table
   the subject *is* the reference, so diff must report zero divergences. This
   proves the harness before any generated code is trusted.
3. **Discovery.** Run all movies with the fallback, collect `misses.csv`,
   `trace merge` into `functions.toml`, `trace names` from a symbol map built
   from snesrev/sm and the bank logs.
4. **First bank.** Generate bank $80 only (common routines), diff, fix.
5. **Bank by bank.** Expand; fallback share falls toward zero.
6. **Overrides.** NMI wait loop → scheduler yield; raster-timed code; anything
   that manipulates the stack to change its return address.

## Known hard parts

- **Widths.** Each (entry, M, X) is its own function. `cfg` tracks REP/SEP
  statically; after PLP and after calls it assumes widths are restored and the
  emitted code checks that at runtime (`assume_mx`).
- **Stack tricks.** Code that pops its own return address or pushes a fake one
  to "return" into a jump table. Detected as diffs; handled with overrides.
- **Timing.** Generated code doesn't count cycles. `Bus::tick` exists for
  per-block estimates; mid-frame IRQs (HUD split) likely need overrides or
  interrupt checks at block boundaries.
- **Decimal mode V flag** in `runtime::ops` is unverified until the fuzz corpus passes.
- **Movie formats.** Many Super Metroid TASes are lsnes `.lsmv`; `movie` only
  reads BizHawk `.bk2` so far. Also check that a movie syncs on the reference
  emulator before using it — a desync there is not your bug.
