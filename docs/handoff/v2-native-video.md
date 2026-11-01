Lane V2 handoff (native video), written by root from V2's final report, 2026-11-01

## State
- On main: the scanline-interrupt frame tables and the HDMA tables, both checked on
  the full route in CI (`SM_IRQ_TABLE`, `SM_HDMA_TABLE`).
  - The HDMA fix: the IRQ plan records each interrupt's control stores, and the HDMA
    planner overlays them, because a window channel reads `$7E:00AB`, which the
    line-0 interrupt rewrites.
- On land/batch13 (not yet on main), step 4, the typed producers: the Samus tile
  upload, palette+OAM, the tilemap queue and animated tiles.
  - They feed the verifier's NativePpu through typed calls (`Ports::set_vram_mode`,
    `upload_vram`, `upload_oam`, `upload_cgram`, `sm_game::display::video::{Upload, Video}`).
  - The `$21xx` writes still drive the emulated PPU.
  - Evidence:
    - SM_NATIVE_LINES on rooms 1, 6, 100, 200 and 320: 0 rows.
    - check-lock on those rooms: matches.
    - one mutant per producer: all caught (6,738 rows).
  - +335 production lines (parallel path).

## Next: step 5, making the typed path the production one
1. Start with the Samus tile upload. Production writes the emulated PPU's memory and
   ports through the typed calls. Delete that producer's `$21xx` writes. The
   native-lines dual check stays as the proof.
2. Repeat per producer, then the remaining producers. Each commit removes code.
3. Then take mosaic and sprite timing off the beam, and route direct seed/snapshot
   loads (verify/seed.rs, snapshot_io) into NativePpu. Native video is then the only video.

## Traps
- Deleting a producer's DMA register writes changes the final DMA channel state and
  drops the DMA start's bus pre-reads, which may matter on the faithful oracle.
  Check whether either is observed first.
- The typed VRAM upload sets the mode before the pointer. Animated tiles write the
  pointer first, which is equivalent only while the address remap is 0.
- The HDMA overlay covers only the interrupt control stores. A door-transition
  resume can still write memory the plan doesn't see.
- The HDMA "one pass late" mutant gives 0 rows, because the store lands long before
  the read. Only the without-the-fix control discriminates.

## Tools (on this machine)
`$TEMP/collab/laneV2`:
- `jobrun.sh`, `all16.sh`: per-segment HDMA/IRQ checks over full-route jobs.
- `natrun.sh`: native lines.
- `lockrun.sh`: check-lock on the chain sample.
- `mut4.py`: the per-producer mutants.

Route bundles are restored in the laneV2 and laneV2b worktrees.
