# Slice 0: status at `rebuild/spine` `93fa4a4`

Measurement lane `lane/slice0-status2`, 2026-11-06. Lockstep (`verify::lockstep::lockstep`,
[SLICE0 §3](SLICE0.md#3-the-lockstep-loop)) over both rows' A2 caches, and the free-run gate
(`crates/tools/src/gate.rs`) over the relock bundle's seeds and pads, both against
`scenarios/locks/lsnes.toml`, on the ROM's `verify::bundle::content` and the room map
`verify::bundle::rooms` builds from it (enemy programs and Samus's tiles definitions, so
occupied slots and tile uploads decode). Each Tick starts from frame k's decoded State.

Was, at `34b9c57`: Ticks fully equal 0 / 476 (`area-brinstar`), 389 / 1,427 (`item-morph-ball`).

## Lockstep

| Row | Ticks compared | Ticks fully equal | Undecodable frames | Pairs skipped |
| --- | --- | --- | --- | --- |
| `area-brinstar` | 476 | **90** | 0 | 0 |
| `item-morph-ball` | 1,416 | **479** | 9 | 11 |

The 9 undecodable frames (711, 780-787) hold `NMI_COPY_SAMUS_TOP_HALF_SRC` = `$CD99`, a top
tiles definition `SamusTiles` doesn't have; the 11 Ticks touching them are skipped.

Ticks differing per field group (first differing frame k+1), and the first field.

| Group | `area-brinstar` | `item-morph-ball` | First difference |
| --- | --- | --- | --- |
| clock, controller, game time, main/irq strand, message box, equipment | 0 | 1 (817) | the pickup: the oracle collects the item and calls the box |
| RNG | 0 | 0 | |
| Samus pose (with `y_radius`) | 0 | 4 (797) | `pose` 0052 vs 00A5; `y_radius` at 1259, super special at 1265 |
| Samus movement | 0 | 3 (797) | the movement variant |
| Samus position | 0 | 56 (682) | `y.pixels` 02A8 vs 02A9 (the elevator); `x` from 1267 |
| Samus animation | 0 | 7 (722) | `frame`, `timer` |
| camera | 1 (414) | 50 (414) | `y.pixels` 01FB vs 01FF; `y.subpixels` from 683, `x` from 817 and 1267 |
| enemies, elevator | 0 | 2 (688) | `elevator.armed`; occupancy variables at 1250 |
| room blocks/scroll | 121 (2) | 209 (2) | `blocks[5317]` tile 008E vs 008F; `scroll_areas[13]` at 92 |
| HUD | 61 (7) | 129 (7) | `tilemap.cells[60].palette`, the mini-map |
| video memory (vram, cgram, oam, batch, Samus tiles) | 74 (7) | 269 (7) | `vram[22620]` (the HUD); `oam` from 414; `cgram[196]` from 741; Samus tiles from 722 |
| committed registers | 1 (414) | 51 (414) | BG scroll `y`, the camera's; windows and fixed colour from 1250 |
| DMA | 282 (1) | 831 (1) | `dma[1].source` 91E0 vs C6C8 |

Ticks whose only differing group is DMA: 199 / 494; room blocks/scroll alone: 103 / 103;
HUD + video memory + DMA: 59 / 118; room blocks/scroll + DMA: 10 / 68. Enemy slots, which
differed on every Tick, are equal on every Tick of both rows; OAM differs only from 414.

## Free-run gate

`tools gate` over both rows: `green no` for each, no stop, 477 and 1,428 frames. New green pair
`area-brinstar cgram`, added to `scenarios/locks/lsnes.green.txt` with `--update`.

| Stream | `area-brinstar` | `item-morph-ball` |
| --- | --- | --- |
| `timing` | equal | equal |
| `cgram` | **equal** | differs from span 0 |
| `vram`, `oam`, `ppu`, `apu` | differ from span 0 | differ from span 0 |
| `sram`, `video`, `wram` | absent from the engine | absent from the engine |
| `input`, `samus`, `camera`, `room`, `progress`, `actors`, `effects`, `rng` | not locked | not locked |

## Blockers, ranked by the Ticks they hold back

1. **DMA channel 1** (C6e): `dma[1].source` from frame 1, 282 / 831 Ticks, the only
   difference on 199 / 494. C6e's open item: channel 1's registers from `$80:933A` and
   `$80:8C83`, which the NMI's other DMAs leave behind.
2. **Room blocks and scroll areas** (B7 scroll PLMs, C5; the scroll PLM lane): block 5317's
   tile from frame 2, scroll area 13 from 92, 121 / 209 Ticks, alone on 103 / 103.
3. **The mini-map** (C9's next step or the map lane): HUD cell 60's palette and `vram[22620]`
   from 7, 61 / 129 Ticks; with DMA, all that differs on 59 / 118.
4. **Samus's tiles definitions** (C6e, `data::pose::tiles`): top half `$CD99` is missing,
   9 frames undecodable and 11 Ticks skipped on `item-morph-ball`; Samus tile uploads differ
   from 722 (8 Ticks).
5. **The elevator ride** (C3, C7): `elevator.armed` at 688, `y` from 682, camera `y` subpixels
   from 683, pose 00A5 at 797; then `x` and camera `x` from 1267, 38-56 Ticks.
6. **Camera** (C5): `area-brinstar` 414, one Tick, `y` 4 pixels apart, OAM and BG scroll with it.
7. **Pickup and message box** (C12, C13): Tick 817; `cgram[196]` from 741 (C6 palette or
   C10), windows and fixed colour from 1250 (C10, C13); `apu` needs C11.

## Reproduce

Runners outside the repository (ROM-derived inputs stay in `$TEMP`): a cdylib over `verify`
including `crates/tools/src/gate.rs`, built with `$TEMP/ohx/b7a/build.sh`, loaded by
`$TEMP/ohx/run.py` under the cached Python; `lock LOCK CACHE ROM ROW` prints the `Report`
with the per-group union and the undecodable frames, `gate` runs `gate::run` with
`LOCK $TEMP/ohx/relock-e/stage/relock-v2-lsnes ROM scenarios/locks/lsnes.green.txt
area-brinstar item-morph-ball`. In the repository, `tools lockstep LOCK CACHE ROM ROW` runs
the same lockstep. ROM sha256 prefix `12b77c4bc9c1832c`. Lockstep runs in 0.3 s and 0.7 s.
