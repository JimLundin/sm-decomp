# Native video handoff (lane F1, 2026-10-31)

Branch `laneF1/video2` (pushed; tip e2e31f7ed on origin/land/batch13 edbf6e54a).
Plan: `docs/ENDGAME_SURVEY.md` "Item 2: video". Steps 1-3 and step 4's tables
and beam-timed inputs are done; typed producers are not started.

## Architecture now
- The rasterizer was already native and shared: `sm_runtime::rendering`
  (`evaluate_sprites`, `render_line`, windows, colour math). The emulated PPU
  (`sm/src/snes/ppu.rs`) only builds `LineConfig` from C fields.
- `rendering::registers::Registers`: typed decode of $2100/$2105-$2114/
  $211A-$2120/$2123-$2133 with the scroll and Mode 7 latches, window native
  flags, COLDATA selects, mosaic start line (1 per frame, 0 after $2106), SETINI.
- `rendering::memory::{Ports, VideoMemory, OwnedMemory}`: OBSEL, OAM address and
  table latch, VMAIN increment/remap, VRAM prefetch, CGRAM pair, reads $2138-$213B.
- `NativePpu` (verifier-only, `ppu.rs` `mod native_line`): Registers + Ports +
  OwnedMemory + own overflow and parity, fed by every `ppu_write`/`ppu_read`;
  renders each line with no emulated or beam state.
- `sm_game::display::hdma::frame_table`, `sm_game::scene::irq::frame_table`:
  plan a frame's HDMA passes and scanline interrupts over scratch copies.

## Env checks (`cfg(sm_verify_dll)`, off when unset)
| Env | Check (log env = name + `_LOG`) |
| --- | --- |
| `SM_PPU_MODEL_CHECK=N` | models vs C PPU on each access of every Nth frame (panics, no log) |
| `SM_CROSS_RENDER=N` | every Nth frame redrawn with LakeSnes layer selection |
| `SM_NATIVE_LINES` | NativePpu line vs C line; resync logged if state diverged |
| `SM_HDMA_TABLE` / `SM_IRQ_TABLE` | frame plan vs the beam's HDMA writes / interrupts |

`full-route.yml` rooms jobs: CROSS_RENDER=30, PPU_MODEL_CHECK=1 nightly and 30
on PRs, NATIVE_LINES nightly, HDMA/IRQ tables always. Any log line fails the
job; only frame/line/counts print (pixels are ROM-derived). Chain rooms 1, 2,
6, 24, 55, 100, 150, 200, 250, 300, 320 are clean on every check, and each
check catches a mutant (see the commit messages).

## Steps 4-5 still to do
1. Typed producers, one per commit, exact on the chain sample, no new sm_game
   RAM sites. Start with the Samus tile upload (`sm/src/game/display_transfers.rs`
   ~245, `scene::samus_tile_upload`): write NativePpu VRAM through a typed call
   beside the $2116-$2119 DMA, check equality, then drop the $21xx path. Then
   display_refresh, the `Registers` trait users, HDMA objects.
2. Seed import (`verify/seed.rs`) and snapshot load (`snapshot_io.rs`,
   `cpu_infra::restore_snapshot`) must load NativePpu directly; restore_snapshot
   currently calls `native_line_restored` to resync.
3. NativePpu owned outside verification, with HDMA/IRQ tables applied per line.
4. Step 5: settle without the beam; PPU, `snes/dma.rs`, LakeSnes become oracle-only.

## Traps
- Local test/exe execution is blocked; unit tests and the detached binary run
  only in CI. DLL replays through `verify_dll.py` work.
- Heredocs collapse backslashes: write edit scripts with the Write tool.
- `verify_dll.py --no-build` rejects a DLL whose source changed: don't edit
  sm/src or crates while a replay batch runs.
- Mine/theirs PPUs share thread_local state keyed by pointer; logs appear twice.
- Native window logic ignores mask logic (Reference doesn't); equal so far.
- Out-of-range VRAM alias words read two C snapshot padding bytes (`Source`).
- IRQ handlers 9/12 poll the door resume every frame; the table records it.
- `Ppu.mosaicEnabled`/`mosaicModulo` are inert caches; nonzero forces a resync.
- `$TEMP/collab/laneF1/bs2.sh ROOM...` builds and runs chain rooms with their
  room expectation; native libs are keyed in `$TEMP/collab/native-*`.
