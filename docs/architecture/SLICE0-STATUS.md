# Slice 0: status at `rebuild/spine` `34b9c57`

Measurement lane `lane/slice0-lock`, 2026-11-06. Lockstep (`verify::lockstep::lockstep`, [SLICE0
§3](SLICE0.md#3-the-lockstep-loop)) over both rows' A2 caches, and the free-run gate
(`verify::gate::free_run` through `crates/tools/src/gate.rs`) over the relock bundle's seeds
and pads, both against `scenarios/locks/lsnes.toml` with the Content `verify::bundle::content`
loads (poses, transitions, delays, horizontal and vertical speeds, slopes). Each lockstep Tick
starts from frame k's decoded State, so a difference never carries into the next Tick.

## Lockstep

| Row | Ticks compared | Ticks fully equal | Undecodable | Skipped |
| --- | --- | --- | --- | --- |
| `area-brinstar` | 476 | **0** | 0 | 0 |
| `item-morph-ball` | 1,427 | **389** | 0 | 0 |

Ticks differing per field group, and the first differing frame (k+1) with its first field.

| Group | `area-brinstar` | `item-morph-ball` | First difference |
| --- | --- | --- | --- |
| clock/counters | 0 | 1 (817) | `completed`, `lag`, `requested_byte` at the pickup |
| controller | 0 | 1 (817) | `pressed`, `repeat`, `repeat_delay` |
| game time | 0 | 1 (817) | `frames` |
| RNG | 0 | 0 | |
| main/irq strand (`mode`, `main`, `irq`, `timing`) | 0 | 1 (817) | `main.frames.len` 1 vs 4, `main.waiting` |
| Samus pose (pose, previous, last different, facing, super special) | 0 | 61 (712) | `pose` 001A vs 0084 |
| Samus movement | 0 | 61 (712) | the movement variant; speeds from 712, momentum from 1333 |
| Samus position | 8 (314) | 117 (314) | `y.pixels` 01D8 vs 01D9; `x` from 688 |
| Samus animation | 0 | 67 (711) | `frame`, `timer` |
| Samus equipment | 0 | 1 (817) | `collected`, `equipped` (the Morph Ball bit) |
| camera | 343 (72) | 581 (72) | `y.subpixels` at 72, `y.pixels` from 74; `x` from 683 |
| enemies (slots, elevator) | 476 (1) | 1,028 (1) | slot 2 `extra_properties`, slot 0 `cursor.delay`, every Tick; `elevator.armed` at 688; occupancy variables from 1217 |
| room blocks/scroll | 121 (2) | 211 (2) | `blocks[5317]` tile 008E vs 008F; `scroll_areas[13]` at 92 |
| HUD | 61 (7) | 131 (7) | `tilemap.cells[60].palette` |
| message box | 0 | 1 (817) | `message` 0 vs 9 |
| video memory (vram, cgram, oam, batch copies) | 476 (1) | 1,035 (1) | `oam.low[0].x` every Tick; `vram[22620]` from 7 (73 / 345); `cgram[196]` from 453 (5 / 130) |
| committed registers (`committed`, batch registers, fixed colour) | 342 (74) | 567 (74) | BG scroll `y`, the camera's; windows and fixed colour from 1250 |
| DMA | 64 (84) | 149 (84) | `dma[1].source` |

The fields not listed (game time beyond `frames`, energy and ammo, `room.room`, `room.width`,
the enemies' species, positions, health, `hud.drawn`, mode 7, mosaic, tilemaps) are equal on
every Tick of both rows. The 389 equal Ticks of `item-morph-ball` are, by their count, the
message box's lag frames (not checked frame by frame); elsewhere enemies and OAM differ on every Tick.

## Free-run gate

`tools gate` over both rows: `green no` for each, no stop, the ratchet
(`scenarios/locks/lsnes.green.txt`) holds.

| Stream | `area-brinstar` | `item-morph-ball` |
| --- | --- | --- |
| `timing` | equal | equal |
| `vram`, `cgram`, `oam`, `ppu`, `apu` | differ from span 0 | differ from span 0 |
| `sram`, `video`, `wram` | absent from the engine | absent from the engine |
| `input`, `samus`, `camera`, `room`, `progress`, `actors`, `effects`, `rng` | not locked | not locked |

## Blockers, ranked by the Ticks they hold back

1. **Enemies**: slot 0's instruction delay and slot 2's extra properties, every Tick
   (C8 the Morph ball eye, C7 the elevator). With OAM, the only difference on 83 Ticks of
   `area-brinstar` and 255 of `item-morph-ball`.
2. **OAM**: `oam.low[0].x` every Tick (C6 Samus drawing, then C7 and C8 for enemy sprites).
   Blocks `oam` in the gate.
3. **Camera and BG scroll**: `camera.y` from 72, `x` from 683, the committed scroll with it,
   343 / 581 Ticks (C5). Next in line after 1 and 2 on 164 / 217 Ticks.
4. **Room blocks and scroll areas**: a block's tile from frame 2, scroll area 13 from 92,
   121 / 211 Ticks (B7 scroll PLMs, C5; C7 if it is a face block).
5. **VRAM, HUD and DMA**: the HUD tilemap palette and `vram[22620]` from 7, DMA source from 84
   (C9 HUD, C5 level uploads, C6 per-pose tile DMA). Blocks `vram` and `ppu`.
6. **Samus**: `y` from 314 (C4 collision), then on `item-morph-ball` the elevator: armed at
   688, `x` and pose 0084 from 711-712 (C3 the ride, C7 the elevator), 61-117 Ticks.
7. **Pickup and message box**: one Tick, 817, where the oracle collects the item and calls
   the box (C12, C13); `cgram` from 453 and the window and fixed colour from 1250
   (C10, C13). `apu` needs C11.

## Reproduce

Runners outside the repository (ROM-derived inputs stay in `$TEMP`): a cdylib over `verify`
built with `$TEMP/ohx/b7a/build.sh`, loaded by `$TEMP/ohx/run.py` under the cached Python;
`lock LOCK CACHE ROM ROW` prints the `Report` with the per-group union above, `gate` runs
`crates/tools/src/gate.rs`'s `run` with `LOCK $TEMP/ohx/relock-e/stage/relock-v2-lsnes ROM
scenarios/locks/lsnes.green.txt area-brinstar item-morph-ball`. ROM sha256 prefix
`12b77c4bc9c1832c`. Lockstep runs in 0.3 s and 0.6 s.
