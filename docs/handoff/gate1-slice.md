# Gate 1 slice

The Morph room slice and every locked corpus row replay headless in `crates/sm_host`,
with no 65816, SPC700 or DSP emulator and no C code, from a sealed prepared start, and
are compared stream by stream with the faithful oracle's lock. The Gate 1 workflow
(`.github/workflows/gate1.yml`, on pushes to `gate1/**`) runs them over eight shards.
Read the workflow's latest run for the row count; do not copy it here.

## How a row runs

`sm_pipeline gate1 [--shard I/N] [--slice ROW]... ROW...` runs from the repository root
(`all` is every locked corpus row). It still calls `tools/verify_dll.py` and
`tools/ci_seed_data.py restore` as subprocesses until those are ported
(docs/VERIFY_PIPELINE.md).

1. The verifier imports the row's seed, the only step that runs the CPU (accepted
   decision 1), and `SM_EXPORT_START=<path>` writes the start.
2. `sm_replay prepare` seals the content image and the start.
3. `sm_replay gate` replays the row: the base run with digests, then the same inputs
   again, three presentation rates, the audio queue held full and the save sink
   accepting later. Each schedule must leave game state and effects identical.
4. `sm_pipeline gate1` (`crates/sm_tools/src/bin/sm_pipeline/gate1.rs`) compares the
   eight machine streams and the tick count with the lock.
   A row the lock lacks is recorded against the faithful oracle on the runner, and its
   lines are printed (`LOCK+`) so the entry can be committed.
5. On the Morph, door and save rows, the windowed `sm_slice` runs headless (xvfb, the
   dummy sound device) and its per-Tick digests are compared with `sm_replay one`'s.

The Tick's closing NMI latches the *next* Tick's controller word, as on hardware, so
`TickInput::latched` carries it: a replay passes the next pad, live play the device's
word at vblank.

## The last Tick's latch

A replay keeps one input word past its Tick budget: the last Tick's closing NMI latches
the movie word after it, as the verifier's loop does (`wall-crawler-slope-step` is the
row whose next word differs).

## Where Gate 1 still falls short of VALIDATION

- `sm_host` is the old game's host (the transpiled game, the RAM image, a register-level
  PPU, DMA and HDMA models) with its emulators removed, and it uses `unsafe`. The
  measures count it as `sm/src`. The `sm_game` Session and the `sm_runtime` renderer
  are not yet what runs.
- Prepared content is the sealed ROM image read by address, not typed records, and the
  start's payload is the verifier checkpoint (CPU and SPC register blocks inert).
- The shell is SDL, not winit/wgpu/cpal (REVIEW allows this for Gate 1).
- Audio evidence is the APU port digest per Tick plus self-determinism.
- The difficult image cases are covered only as far as corpus rows reach them through
  the `video` stream; there is no dedicated fixture set.
- `HostResult` receipts are recorded only; the outbox bound holds by construction and
  is not measured. Performance is unmeasured against a budget.

## Next

1. Replace what `sm_host` borrows from the old game, starting with the renderer
   (the native renderer as replay output) and the frame lease.
