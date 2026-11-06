# Slice 0: status at `rebuild/spine` `4c23c75`

## Current: engine wave 1, 2026-11-06

Lockstep, from `scenarios/locks/lockstep.ratchet.txt` (ratchet test, debug, about 250 Ticks/s):
`area-brinstar` 476 / 476 Ticks fully equal; `item-morph-ball` 1,324 / 1,427 (was 1,291 at
`05cc5ba`).

| `item-morph-ball` group | Ticks differing | First frame | First difference, cause |
| --- | --- | --- | --- |
| samus | 42 | 682 | `position.y.pixels` 02A8 vs 02A9, the elevator ride |
| room (camera, blocks, PLMs) | 54 | 683 | camera `y.subpixels`; shot block `blocks[5708]` and PLM slot 34 crumble from 1236; camera `x` from 1267 |
| video | 101 | 682 | OAM `y` (Samus); BG3 `vram[22982]` at 823 (message tilemap, `$85:8324`); `dma[6].count` at 822 (HDMA run, B6); restore at 1213/1214; HDMA object on channel 2 at 1249 |
| elevator | 1 | 688 | `elevator.armed` |
| hud | 1 | 1294 | `tilemap.cells[28].tile` |

Free-run gate at `4c23c75` (`lsnes.green.txt`: 7 pairs):

| Stream | `area-brinstar` | `item-morph-ball` |
| --- | --- | --- |
| `vram`, `cgram`, `oam` | **equal** | differ from span 0 |
| `sram`, `timing` | **equal** | **equal** |
| `ppu`, `apu` | differ from span 0 | differ from span 0 |
| `video`, `wram` | absent from the engine | absent from the engine |

`area-brinstar`'s red streams wait on the seed re-cut with the PPU tier (`ppu`), sound (C11,
`apu`) and the line program (B6, `video`); `wram` doesn't decide green.

The sections below are the measurement at `93fa4a4` and are kept for history.

Measurement lane `lane/slice0-status2`, 2026-11-06. Lockstep (`verify::lockstep::lockstep`,
[SLICE0 §3](SLICE0.md#3-the-lockstep-loop)) over both rows' A2 caches, and the free-run gate
(`crates/tools/src/gate.rs`) over the relock bundle's seeds and pads, both against
`scenarios/locks/lsnes.toml`, on the ROM's `verify::bundle::content` and the room map
`verify::bundle::rooms` builds from it (enemy programs and Samus's tiles definitions, so
occupied slots and tile uploads decode). Each Tick starts from frame k's decoded State.

Was, at `34b9c57`: Ticks fully equal 0 / 476 (`area-brinstar`), 389 / 1,427 (`item-morph-ball`).

## Lockstep at `93fa4a4`

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

## Free-run gate at `93fa4a4`

`tools gate` over both rows: `green no` for each, no stop, 477 and 1,428 frames. New green pair
`area-brinstar cgram`, added to `scenarios/locks/lsnes.green.txt` with `--update`.

| Stream | `area-brinstar` | `item-morph-ball` |
| --- | --- | --- |
| `timing` | equal | equal |
| `cgram` | **equal** | differs from span 0 |
| `vram`, `oam`, `ppu`, `apu` | differ from span 0 | differ from span 0 |
| `sram`, `video`, `wram` | absent from the engine | absent from the engine |
| `input`, `samus`, `camera`, `room`, `progress`, `actors`, `effects`, `rng` | not locked | not locked |

## Blockers at `93fa4a4`, ranked by the Ticks they hold back

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

Lockstep needs the ROM (`rom/sm.smc`, sha256 prefix `12b77c4bc9c1832c`) and the two rows' A2
record caches (`CACHE/area-brinstar`, `CACHE/item-morph-ball`; ROM-derived, so never committed).

```sh
cargo build --release -p tools
# One row: the report, then the ratchet lines (Ticks fully equal, differing Ticks per group).
target/release/tools lockstep scenarios/locks/lsnes.toml CACHE ROM area-brinstar
# Every ratchet row against scenarios/locks/lockstep.ratchet.txt: exit 1 when a fully-equal
# count falls or a group's differing count rises; prints the lines to commit when counts improve.
target/release/tools lockstep --check-ratchet scenarios/locks/lsnes.toml CACHE ROM \
  scenarios/locks/lockstep.ratchet.txt
```

Both rows run in about 1 s together (about 1,800 Ticks/s, release build). Where Defender blocks
executables, build the cdylib as `tools/lane/test_dll.py` does. `verify::ratchet` holds the
summary and the file format; `crates/tools/src/ratchet.rs` the command, whose test
`committed_ratchet_holds_against_the_cache` runs the check when `SM_CACHE` and `SM_ROM` are set.
The free-run gate is `tools gate LOCK BUNDLE ROM scenarios/locks/lsnes.green.txt area-brinstar
item-morph-ball`.

## Lockstep at `1d6a2da`

From `lockstep.ratchet.txt`: `area-brinstar` 335 / 476 Ticks fully equal (room 119 from frame 2,
video 32 from 84); `item-morph-ball` 1,075 / 1,427 (room 267 from 2, video 183 from 84, samus 55
from 682, elevator 1 at 688, hud 1 at 1294). The tables above are from `93fa4a4` and stale.
