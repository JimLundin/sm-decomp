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

## Baseline (measured 2026-11-03)

Step 1's numbers, on main `07244dfda`, from the temporary `collab/perf-baseline` instrumentation
(`SM_PERF_COUNTERS`, `SM_PERF_HEADLESS`) and CI runs 41641005122 and 41644716265. The
verifier was built as in CI, plus frame pointers. Ticks/s is the verifier's in-process loop,
which excludes about 1.5 s of process start and seed load per row. Mother Brain
(`chain-any-0190-DD58`) and `credits` ran their first 1500 ticks only.

Runner CPUs differ by up to 1.8x for the same row: Kraid in lock mode ran at 22.3 ticks/s on
an EPYC 9V45, 15.0 on a Xeon 8573C and 12.2 on an EPYC 9V74. Compare modes only on one runner.
A speed ratchet must normalise for the runner, for example against the oracle lock rate in the
same job.

### Ticks per second, all modes on one runner (Xeon 8573C)

| Row | Ticks | Oracle lock (record) | Lock (PR gate) | Headless | Lockstep |
| --- | ---: | ---: | ---: | ---: | ---: |
| `chain-any-0001-DF45` (Ceres, after the intro) | 412 | 224 | 19.8 | 21.0 | 18.0 |
| `chain-any-0012-91F8` (Landing Site) | 1031 | 155 | 15.5 | 15.8 | 13.8 |
| `chain-any-0063-A59F` (Kraid) | 1266 | 157 | 15.0 | 15.4 | 13.7 |
| `chain-any-0097-B32E` (Ridley) | 2672 | 140 | 11.5 | 11.9 | 10.8 |
| `chain-any-0190-DD58` (Mother Brain) | 1500 | 150 | 11.4 | 11.8 | 10.9 |
| `credits` | 1500 | 184 | 23.5 | 24.4 | 21.0 |

- Oracle lock: the faithful oracle only, recording the digest lock.
- Lock: the engine only, with typed observation and running digests checked against the lock.
- Headless: the engine only, with no observation, digests or verifier-only Session check. The
  frame settlement (`draw_frame_to_ppu`) still runs, because it runs the scanline interrupts;
  skipping it changes behaviour.
- Lockstep: oracle and engine paired, with the full RAM compare.

**Headless is only 3% faster than the lock gate.** Verification is not the bottleneck: the
engine itself runs at 11-24 ticks/s. The oracle runs 7-13x faster than the engine.

### Time per tick, Kraid (Xeon 8573C, microseconds)

| Phase | Lock | Headless | Lockstep |
| --- | ---: | ---: | ---: |
| Whole frame | 66,868 | 64,845 | 73,178 |
| Engine tick, leases included | 61,194 | 59,357 | 60,833 |
| - Session encode at lease release and `foreign` | 49,363 | 49,470 | 49,091 |
| - Session decode at lease open and after `foreign` | 5,795 | 5,839 | 5,811 |
| - Verifier-only decode check of the kept Session | 1,857 | 0 | 1,850 |
| - Game logic and the rest of the tick | 4,179 | 4,048 | 4,081 |
| Frame settlement and emulated PPU (`draw_frame_to_ppu`) | 5,374 | 5,403 | 10,657 (both sides) |
| Oracle CPU frame | 0 | 0 | 844 |
| Typed observation and digests | 219 | 0 | 31 |
| Paired RAM compare and normalisation | 0 | 0 | 30 |

The oracle's own frame is 0.8 ms; its `draw_frame_to_ppu` is 5.3 ms (83% of an oracle lock
tick), so the emulated PPU bounds the oracle at about 150-220 ticks/s.

### Counts per tick (deterministic, the same in every mode)

| Row | Leases | Fresh imports | Kept reuses | Decodes | Decoded KB | Encodes | Encoded KB | Spans written | `foreign` | `owned_call` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| DF45 | 16.7 | 7.6 | 9.1 | 16.5 | 1,887 | 25.6 | 2,923 | 1.69 M | 8.9 | 9.6 |
| 91F8 | 20.1 | 9.3 | 10.8 | 22.2 | 2,533 | 32.9 | 3,760 | 2.17 M | 12.9 | 8.6 |
| Kraid | 17.8 | 8.1 | 9.7 | 24.0 | 2,742 | 33.7 | 3,844 | 2.22 M | 15.9 | 14.8 |
| Ridley | 20.8 | 9.4 | 11.4 | 33.3 | 3,807 | 44.7 | 5,105 | 2.94 M | 23.9 | 11.3 |
| Mother Brain | 22.4 | 11.4 | 11.1 | 33.7 | 3,855 | 44.8 | 5,113 | 2.95 M | 22.4 | 13.2 |
| credits | 12.3 | 5.6 | 6.7 | 14.1 | 1,616 | 20.8 | 2,375 | 1.37 M | 8.5 | 5.0 |

- No lease used the retained persistent Session on these rows (0 per tick).
- Each `foreign` call exports and re-imports the whole Session. They are about half the
  encodes and two thirds of the decodes.
- Each encode writes about 117 KB in about 66,000 spans; the 12,800 terrain behaviour bytes
  are one span each. Each decode reads about 117 KB.
- On Kraid one encode costs about 1.5 ms and one decode about 0.24 ms (Xeon 8573C).

### Where the time goes (`perf record -g`, Kraid, EPYC 7763)

Self time, grouped (lock mode; headless is within one point of it in every bucket):

| Bucket | Self time | Main symbols |
| --- | ---: | --- |
| Projection codec | 40% | `fx_encoding::control_graph` 10%, `actors::codec::{write_,}projected_byte` 11%, `menu_window_encoding::lane` 3%, HDMA live-owner `BTreeMap` lookups 4%, `session_encoding::projected_byte` 4%, `fx_encoding::source_owner` 1.4% |
| RAM write bookkeeping | 24% | `WorkRam::note_source_publication` (a `BTreeMap` lookup per byte) 10%, `note_owned_write` 4%, `copy_in` inlined in the encode visitor 7.5% |
| `memmove` (libc) | 21% | Struct copies in the FX source projection per byte: `fx_encoding::session_source_byte` 10.5%, `projected_source_byte` 7.4% |
| Emulated PPU | 8% | `ppu_line_config` 3%, `ppu_get_pixel` 2.7%, `ppu_run_line` 1% |
| Session encode/decode traversal | 4% | `session_encoding::{encode,decode,visit}` |
| Verification compare/normalisation | under 0.5% | `observation::single_tick` 0.3% |
| Game logic, allocation and the rest | under 5% | Not one symbol above 0.25% |

Inclusive time:
- `encode_session` 75%. Of that, `fx_encoding::encode_sources` is 40%: for each of about 2,560
  FX source addresses, `projected_source_byte` resolves the physical source and rebuilds the
  whole `control_graph`. The `visit` writes are 29%.
- The callers that trigger the encodes: `actors::drawing::draw_scene` 42% (its sprite,
  spritemap and projectile hosts run through `foreign`), `Runtime::relinquish` 40% and
  `produce_player_scene` 20%.
- `import_session_image` (decode) 7.5%, the verifier's kept-Session check 2.3% and
  `draw_frame_to_ppu` 8.3%.

### The five biggest targets

Estimates are for Kraid headless on the Xeon (64.8 ms per tick, 15.4 ticks/s). Targets 2-4
overlap, because each removes encodes.

1. **Hoist the FX source projection out of the per-byte loop.** `encode_sources` rebuilds
   `control_graph` and copies the live HDMA structures for every source byte, about 2,560 times
   per encode and 34 encodes per tick. Compute the graph once per encode, as
   `encode_display_sources` already does, and copy nothing per byte. This is a local change in
   `sm_game::fx_encoding`. Estimate: save about 25 ms per tick, 1.6x (to about 25 ticks/s).
2. **Move the drawing hosts off `foreign`.** There are 16-24 `foreign` calls per tick, each a
   full export and import (about 1.7 ms). They come mostly from `draw_scene`
   (`draw_spritemap`, `submit_sprite`, `draw_projectiles`, `draw_flare`). Estimate: save
   about 27 ms per tick, 1.7x alone.
3. **One retained Session through the tick (PERFORMANCE step 2).** After target 2, the
   remaining 18-22 lease releases still encode, and 8-11 fresh leases decode. A Session kept
   for the tick, exported once, removes the rest. Estimate, with target 2: from 64.8 ms to
   about 10 ms per tick, about 6x (to about 100 ticks/s on this CPU, 150 on the fastest).
4. **Make RAM writes cheap.** Each byte written does a `BTreeMap` lookup in
   `note_source_publication` and a class fold in `note_owned_write`. Terrain behaviour goes out
   as 12,800 one-byte spans. Skip the publication lookup outside the tracked ranges, batch the
   terrain span, and fold classes per range. Estimate: save about 10-15 ms per tick (20%)
   while encodes remain. The same fix speeds up every RAM write the compatibility layer makes.
5. **Render no pixels when nothing compares them.** The frame settlement must keep its
   scanline interrupts, but its pixel work (`ppu_get_pixel`, `ppu_line_config`,
   `ppu_run_line`) is about 5 ms per tick. That is 8% of the tick now and about half of it once
   targets 2-3 land. It is also 83% of an oracle lock tick. Headless runs, and verified frames
   that compare no pixels, could stop at the line state. Estimate: 2x after target 3 (to about
   200 ticks/s), and 4-5x on oracle lock recording.

Smaller items:
- Precompute the address-to-owner tables for `actors::codec::projected_byte` and
  `menu_window_encoding::lane` (step 3, about 15% while projection remains).
- Drop the verifier-only kept-Session decode check from the PR gate (3%).
- The lock gate's own observation and digests cost 0.2 ms per tick, and lockstep's RAM compare
  0.03 ms. Step 5's verification overhead is already small. The cost to remove is the
  compatibility bridge.

Once targets 1-5 land, game logic and settlement are about 4 ms per tick (about 250 ticks/s).
To reach the 600 ticks/s target needs profiling again after that.
