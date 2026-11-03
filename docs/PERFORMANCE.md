# Engine performance plan (2026-11-03)

Jim (2026-11-03): the engine must run far above real time. A playable engine needs a comfortable 60
ticks per second, and a game this old should run headless at hundreds. Replays currently run at
about 20 ticks per second on a CI runner, which is far too slow.

## Targets

| Mode | Now | Target |
| --- | --- | --- |
| Headless engine, no verification (sm_tools replay of inputs) | unmeasured | 600+ ticks/s (10x real time) on a CI runner |
| Engine with our renderer (sm_app) | unmeasured | a steady 60 fps with headroom on a laptop |
| Verified replay, digest lock (PR gate) | ~20 ticks/s | 100+ ticks/s |
| Verified replay, lockstep against the oracle | ~15 ticks/s | bounded by the oracle |

## Where the time goes (profiles from 2026-11-02, to be refreshed)

Self-time from CI `perf` profiles of a lock-mode replay:
- `WorkRam::copy_in` and the `note_*_write` epoch bookkeeping: 30-45%. Every RAM write scans or
  tables owner ranges.
- The projection codec (`codec::projected_byte`, `session_encoding::projected_byte` and its
  linear owner chain, `fx_encoding::control_graph`/`control_address`): 20-35%.
- Whole-Session encode and decode at lease boundaries (`session_encoding::visit/encode/decode`):
  5-10%.
- The emulated PPU (`ppu_get_pixel`, `ppu_line_config`, `ppu_run_line`): about 10%.
- The engine's own game logic is a small share.

The cost is the compatibility bridge between the RAM image and the typed Session. It is not the
game.

## Plan, in order

1. **Measure.** Add a headless engine-only ticks/s measurement for a fixed set of rows: no oracle,
   no RAM compare. Run a `perf` profile of the current main in CI. Record leases per tick, encode
   and decode bytes per tick, and the time per phase. These numbers become the baseline and are
   recorded in PROGRESS_REPORT.
2. **Session as the authority for the whole tick.** The address-API layer is nearly retired, so
   engine code reaches state through typed Session fields.
   - Keep one retained Session through each tick.
   - Stop the per-lease RAM decode/encode, and export the RAM image only at the points the oracle
     compares: once per tick, or never in headless mode.
   - The remaining `Game`-hosted paths (menus, Kraid's unpause coroutine, `foreign` calls) are the
     blockers. Move them onto the Session, or give them a single import/export per tick.
3. **Table-driven projection.** Where projection remains (verification export and the compat
   layer), replace per-byte owner chains with precomputed address-to-owner tables, and encode in
   bulk.
4. **Rendering.** Headless runs render nothing. Verified runs render only at compared frames.
   The desktop renders through `sm_runtime` rather than the emulated PPU (REVIEW item 2).
5. **Verification overhead.** The PR gate uses the digest lock (one hash per tick) in place of
   full-RAM normalisation and compare. Keep lockstep for localisation and the nightly run.
6. **Ratchet the speed.** Add a CI-reported ticks/s measure per mode, so that regressions show
   up, like the RAM ratchet. Make it strict once the numbers stabilise.

Each step lands behind the existing replay gate. A speedup that changes behaviour is a bug.
