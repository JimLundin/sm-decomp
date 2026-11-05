# Gate 1 slice: handover (2026-11-05, 07:50 Stockholm)

Branch `gate1/handover`, 9 WIP commits on main `41f29f4`. Not landed and not passing yet.
Squash before landing. Jim's target is Gate 1 by noon Stockholm. Full verification of
`41f29f4` passed (run 41979433489).

## The approach

`crates/sm_host` compiles the engine host from the same source as `sm`, with no CPU
emulator, SPC, DSP or native code. `sm` itself is unchanged in behaviour.

- `crates/sm_host/src/game` and `src/snes` are **symlinks** into `sm/src`; other modules
  are `#[path]` includes. A `sm_host` rustc cfg (`build.rs`) gates what differs.
- `stubs/`: `cpu.rs` and `apu.rs` are inert; `rom_patches.rs` is a stub; `ppu_ports.rs` is
  a Rust port of LakeSnes's PPU read, write and pixel, replacing `backend.c`.
- `cpu_infra.rs` holds the host's own beam, snapshot and frame.
- `session.rs`: `Session::open` / `step`, `TickInput`, `TickOutput`, Scene, effects, and a
  semantic effect trace. No file or device I/O.
- `sm_data::prepared`: sealed `PreparedContent` and `PreparedStart`. `sm_data::digest` moved
  out of `sm_tools::trace`.
- `sm_runtime::outbox`: CONTRACTS' one-batch outbox with per-sink acceptance; unit tests pass.
- `sm_tools`: `slice.rs` and the `sm_replay` bin (feature `replay`). `prepare` seals a start;
  `gate` runs the base replay plus repeat, presentation-rate and held-sink schedules;
  `one` is a single replay for the diagnostics.
- `sm_app`: `sm_slice` bin (feature `slice`), SDL window and sound without GL.
  **Only `cargo check`ed. It has never been linked or run** (no `libsdl2-dev` in my VM).
- The verifier's `SM_EXPORT_START=<path>` writes the start after seed import. That is the
  only step that runs the CPU (accepted decision 1).
- `tools/gate1.py <rows>` is the row runner, and on a mismatch locates the divergence
  (`SM_DIGEST_LOG`, `SM_DUMP_TICK`, `SM_DUMP_OUT`, in `observation/machine.rs`).
  `.github/workflows/gate1.yml` runs it on pushes to `gate1/**`, in about 4 minutes.
  It needs the ROM, so it only runs in CI, unless your VM has one.

## Where it stands

Replays run end to end with no crash and no fault, at roughly 120 to 240 ticks/s with
digests. Every row still **differs from the lock**.

Measured in the last run (head `a08a645`, rows `door-transition`, `samus-mockball`):
- The first difference is in WRAM only, in the controller words: `$8B`, `$8F`, `$93`, `$97`,
  `$A3`. It appears at tick 70 and tick 40 respectively, with VRAM following a tick later.
- Repeat runs, presentation rates and held audio or save sinks all left game and effects
  identical. Those schedules only compare the slice with itself, so they are not fidelity
  evidence. Neither row produced a save, so the save path has only run in unit tests.

## Next steps

1. **Likely cause, not yet tested:** `sm/src/game/menu_host.rs:125` reads
   `oracle_mode::nmi_input()`, and the verifier's loop calls
   `oracle_mode::set_next_frame_input(inputs.get(index + 1))` around each frame. The slice
   never calls it. Try that in `slice.rs` first. If it is the cause, the end-of-frame NMI
   latches the *next* input, which conflicts with `step` taking only one Tick's input and
   with live play, so settle how the Session gets it (for example a one-Tick input delay).
2. **`item-morph-ball` has no lock entry.** `gate1.py` records one with `--mode lock`, and
   that exits 2 with nothing printed; find out why. The row is marked `unlocked` in
   `scenarios/corpus.toml` ("WRAM drift lockstep masks ... not yet identified"), so it may
   really differ from the oracle. Report that as a divergence; do not mask it.
3. Get locked rows matching, then widen `ROWS` for the image evidence. An earlier run used:
   `area-ceres area-norfair area-maridia area-wrecked-ship xray-scan-91f8 boss-ceres-ridley
   power-bomb-room-entry boss-phantoon door-transition special-beam-plasma terrain-quicksand
   samus-mockball`. Add a save row (`save-memory`, `samus-gunship-save`) to exercise the save
   sink. The `ppu_ports` pixel path has no evidence of having run: compare `video` on a
   mosaic or EXTBG row.
4. Link and run `sm_slice` in CI: install `libsdl2-dev`, use xvfb with `SDL_AUDIODRIVER=dummy`,
   pass `--inputs --lock-out`, and compare with the lock.
5. Build with `sm/` absent: `git mv` the shared sources into `crates/sm_host`, have `sm`
   `#[path]`-include them, drop the symlinks (they break on Windows checkouts), then add a CI
   job that removes `sm` from the workspace and builds. `sm_host` still includes
   `verification/oracle_host.rs`, `ram_names.rs`, `machine.rs`, `verify/checkpoint.rs` and
   `oracle_mode.rs`.
6. Before landing: `sm_host` sets `warnings = allow`, so make it pass clippy and fmt. Check
   the package guard (sm_app now depends on sm_tools under `slice`, which the guard rejects
   for target packages, so move or justify it) and `tools/ram_ratchet.py`. Run the full local
   check list and row-only runs, since cfg gates touch `sm/src`.

## Be honest in the Gate 1 record

Even with every row matching, these fall short of VALIDATION's text and need saying:
- The host still keeps the RAM image, a register-level PPU, DMA and HDMA models, and `unsafe`.
- Prepared content is the sealed ROM image read by address, not typed records.
- The start's payload is the verifier checkpoint, CPU and SPC register blocks included (inert).
- The shell is SDL, not winit/wgpu/cpal, which REVIEW allows for Gate 1.
- Audio evidence is the `apu` queue digest per Tick plus self-determinism.
- `HostResult` receipts are recorded only; `most_waiting` is a constant, not a measurement.
- `Session` copies the Scene each Tick; performance is unmeasured against any budget.

## Leftovers

Throwaway branches I could not delete (branch deletes return 403): `mutant/*`,
`verify/mutant-*`, `verify/full-check`, `verify/full-281`, `verify/spawn-dispatch-rows`,
`profile/run`, `gate1/trial`, finished `collab/*`. The running log is on issue #254.
Workflow dispatch returns 403, so use branch pushes.
