# Engine performance plan (2026-11-03)

Jim (2026-11-03): the engine must run far above real time. A playable engine needs a comfortable 60
ticks per second, and a game this old should run headless at hundreds.

## Targets

| Mode | Target |
| --- | --- |
| Headless engine, no verification (`sm_tools` replay of inputs) | 600+ ticks/s (10x real time) on a CI runner |
| Engine with our renderer (`sm_app`) | a steady 60 fps with headroom on a laptop |
| Verified replay, digest lock (PR gate) | 100+ ticks/s per process |
| Verified replay, lockstep against the oracle | bounded by the oracle |

## Plan, in order

1. **Measure.** Add a headless engine-only ticks/s measurement for a fixed set of rows: no oracle,
   no RAM compare. Run a `perf` profile of the current main in CI. Record leases per tick, encode
   and decode bytes per tick, and the time per phase. These numbers become the baseline and are
   recorded here.
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

## Baseline (measured 2026-11-04)

One `perf record` run of `check-lock` per row on main `94a6fd263`, one process at a time on a
GitHub runner, built with frame pointers (#244). Seconds include process start, the Python
wrapper (about 10% of these runs) and `perf`'s own overhead. Runner CPUs differ by up to 1.8x, so compare
only within one runner.

| Row | Ticks | Seconds | Ticks/s |
| --- | ---: | ---: | ---: |
| `chain-any-0012-91F8` (Landing Site) | 1031 | 7.8 | 133 |
| `chain-any-0063-A59F` (Kraid) | 1266 | 8.1 | 156 |
| `chain-any-0097-B32E` (Ridley) | 2672 | 17.6 | 152 |

The earlier 11-24 ticks/s is gone: every frame now runs on one lease with one import.

**In a real gate job the rate is far lower.** A `lock-any` job on `94a6fd263` ran 58 rows, 28,530
ticks, across 4 worker processes: 547 process-seconds, or about 52 ticks/s per process and about
177 ticks/s for the job in 161 s. Four workers give only slightly more throughput than one.
The cause isn't established; the runner's 4 vCPUs are probably 2 physical cores, and memory
bandwidth may also limit it. Measure 2 and 3 workers before changing the count.

### Where a tick's time goes (Kraid, inclusive)

| Share | What |
| ---: | --- |
| 45% | `settle_on_lease`: the frame's settlement |
| 33% | `advance_beam`, of which `ppu_run_line` is 32% and `native_pixel` 18% self |
| 22% | `observed_inputs`, of which `InputProjection::new` is 21% |
| 22% | The NMI's `refresh` |
| 13% | `encode_spans`, mostly from `publish` (11%) |

Shares overlap, since one call can sit inside another. Game logic isn't near the top.

### The biggest targets

1. **`observed_inputs`, about 21%.** Every owned call refreshes the whole `ExternalInputs` snapshot
   on entry and return, walking the owner chain for each input byte. Read the fields the
   caller needs from the Session directly, then delete the snapshot.
2. **Pixels in the lock gate, up to 33%.** The lock digests video, so it needs pixels on checked
   frames. Check whether a sampled video stream would keep enough sensitivity before changing
   this; it would weaken the check, so it needs Jim's decision.
3. **`publish` and `encode_spans`, about 13%.** They go with the RAM export.

Per-row process start isn't worth chasing: a fit over that job's rows gives about 0.26 s per row
(3% of its verifier time) and about 18.6 ms per tick.

Re-profile after each of these. These figures say nothing about `sm_app`, which isn't measured.
