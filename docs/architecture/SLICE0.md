# Slice 0: the plan to green

Planning lane `lane/slice0-plan`, from `rebuild/spine` `3037294`, 2026-11-06. It turns
[ARCHITECTURE §15](ARCHITECTURE.md#15-build-order) ("the spine, by one lane", then slice 0)
into lanes of at most 30 minutes and 300k tokens each. Every lane ends with something a
test or a comparison can fail. Behaviour comes from the bank logs; `sm/src` and
`crates/sm_game` only say *what* exists, never how to shape it (Retro rule 1).

## 1. What the two rows exercise

Both rows start from the same lsnes seed, `100-input-phase:17` (`683b992…`), with the same
pads (`3091dcd…`). `area-brinstar` runs 250 Ticks (facts `area:brinstar`, `room:9E9F`).
`item-morph-ball` runs 1,200 (facts `item:morph-ball`, `room:9E9F` and the message box
completing, today a probe that §2's facts lane retypes). What the oracle already measured
on these seeds ([MEASUREMENTS](../oracle/MEASUREMENTS.md) (c), (d)):

- **No room load.** The HDMA object handler returns at line 241-248 on all 250 frames of
  `area-brinstar`, and neither row appears among the rows with busy frames. The seed is in
  game state 8 (main gameplay) in the room. So slice 0 has no door, no decompression at run
  time and no cost model (D9).
- **The message box.** In `item-morph-ball` the handler runs on only 802 of 1,200 frames:
  about 398 frames are the box's lag frames, run from inside the item PLM's instruction
  (`$84:88F3` → `$85:8080`: open `$844C`, interaction `$846D`, close `$8589`). The decoder
  already maps these wait sites (`crates/verify/src/strand.rs`).
- **The HUD split.** Two IRQ commands, at lines 0 and 31, each writing `$2109`, `$2130`,
  `$2131` and `$212C` in the last third of the line. For the line program (D11) that is
  "takes effect from the next line": one rule value, no dot model needed for slice 0.
- **Sound.** `area-brinstar` makes no CPU port writes at all. `item-morph-ball` makes 3 on
  port 0 (the item fanfare through the music queue) and about 25 requests on ports 1-3,
  echoed by the driver in about half a frame (median about 8,500 SPC cycles).

What the ROM places in the room (bank logs `$8F`, `$A1`, `$84`; the default state `$9EB1`,
because only it places the Morph Ball the row collects; "Zebes is awake" selects `$9ECB`):

| Content | In the room | Slice 0 needs |
| --- | --- | --- |
| Room header `$8F:9E9F` | area 1, 8×3 screens, tileset 6, music `06,07`, FX `$8290`, layer 2 scroll `C1,C1`, scroll table `$9EED`, library background `$BAF4` | the header and state selection decoded into a typed room record; level data (`$C5:DED5`) decompressed at content load so the engine can check it against the seed's blocks |
| PLMs `$8F:867E` | five scroll PLMs (`$B703`), nine extensions (`$B647` up, `$B63B` right), the Morph Ball (`$EF23`, list `$84:E3EF`, setup `$EE64`) | the `Plm` family: scroll PLMs (Samus touching one changes scroll areas), extensions (setup only), the item (draw, collected check, pickup op) |
| Enemies `$A1:93AC` | two parts of the Morph ball eye (`$E6BF`), the elevator (`$D73F`), four Blue Brinstar face blocks (`$EA7F`) | the `Enemy` family and three species' AI, as their bank logs say; no enemy projectiles unless the census shows one |
| Doors | four door pointers, no door PLMs in this state | nothing: no door is crossed. ARCHITECTURE §7.2 names "door caps" as slice 0 PLMs; this room's default state has none, so the PLM users are scroll, extension and item. |

What the rows run per Tick (D6): the main loop's HDMA object handler (the music queue, the
HDMA objects present, layer blending), the scan-out with the two HUD IRQs, then RNG, OAM
clear, game state 8's gameplay pass (Samus input and movement, block collision, PLMs, the
enemies in `$A0` order, the camera and the level scroll uploads, the HUD, Samus and enemy
drawing), the sound handler, OAM finalisation and the NMI wait; then the NMI's batch
commit, counters and pad auto-read. Samus moves as the pads drive her: which poses, whether
she arrives on the elevator, whether she shoots, which HDMA objects, palette FX and
animated tiles are live (tileset 6 is an animated-tile tileset in the old handoff's list),
and how many RNG calls each frame makes are not known from the documents. Lane A1 measures
them before any domain lane starts, and the domain list below grows or shrinks by its
table, not by guess.

## 2. The lanes

Each line: what is true afterwards; files; proof; must not touch. "Lockstep" means the
one-step localiser of §3. `(∥)` marks a lane that can run beside the spine.

### A. Before the spine's first Tick (all parallel with B1)

| Lane | Delivers | Files | Proof | Must not touch |
| --- | --- | --- | --- | --- |
| A1 census (∥) | A table per row: game state, Samus pose IDs and movement types per frame, live PLM, enemy, projectile, sprite-object, HDMA-object, palette-FX and animated-tile slots, sound requests `(library, id)`, music queue entries, RNG calls per frame, lag and overrun frames | `oracle` measure subcommand (tier a, d), this document's §1 | the table, with the command and record hashes in the commit | engine crates |
| A2 record cache (∥) | `tools records ROW` writes each frame's `FrameRecord` for both rows to a local cache (ROM-derived, `$TEMP`, never committed); `verify::source` reads it | `crates/tools`, `crates/verify/src/source.rs` | the cache, re-read, gives the same records twice; its frame count equals the row's | engine crates |
| A3 span fold (∥) | RELOCK §8 step g's engine side: `verify::fold` turns per-frame (State, `TickOutput`) pairs into an `EngineRow` with the recorder's rules (quiescent cuts from `Recorded`, `apu` over busy spans, `video` and `timing` every frame) | `crates/verify/src/fold.rs` | the oracle as engine: folding the cached records' own decoded States and streams reproduces the recorder's marks for both rows, stream by stream | `oracle`'s recorder |
| A4 facts (∥) | typed facts: `room`, `area`, `item` over the projected State, and "the message box was shown and closed" over the main strand's frames, replacing the probe | `crates/verify/src/facts.rs`, `scenarios/` row text | each fact holds on the decoded oracle run of its row and fails on a mutated State | the lock's fact results |
| A5 compositor (∥) | the `gate1/render` line compositor moved into `runtime` (D1's closed list), PPU-domain 15-bit output with per-line brightness and forced blank (D17) | `crates/runtime/src/compose/` | composing each cached record's VRAM, CGRAM, OAM and per-line registers (tier a) reproduces the record's picture on every frame of `area-brinstar`; time per frame reported | `game` |
| A6 size report (∥) | `tools size`: authored lines per crate and per D24 domain, against FOUNDATION §9's table | `crates/tools` | a report on this branch; CI prints it on every landing | budgets themselves |

### B. The spine, by one lane at a time (Retro rule 25)

| Lane | Delivers | Proof |
| --- | --- | --- |
| B1 session and Tick | `game::Session::{open(State, &Content), step, begin/end}` with D6's five steps, `MainStrand` stepping over `Next`, the beam-sync point per frame (D7), the NMI commit and lag paths, counters, the pad latch and auto-read; the `Recorded` timing source from a lock row's `overruns` and `nmi_off`; unbuilt gameplay phases are empty functions in the bank `$82` order | unit tests for commit, lag, overrun and the latch; over both rows, `Counters` and `Controller` after each step equal the next decoded record's |
| B2 lockstep tool | `tools lockstep ROW [--from K] [--ticks N] --stream S`: §3's loop over the cache | it reports the first differing field on a deliberately broken counter rule and nothing on B1 |
| B3 content base | `data::rom`: ROM identity, LoROM mapping, the `AddressMap` side output, the decompressor, the room header and state selection for `9E9F`; `verify`'s `RoomMap` replaced by the map | header fields equal the bank log's; decompressed level size equals the header's; the decoder's room ID comes from the map |
| B4 level | `RoomScoped.blocks`, BTS and scroll areas from content; the decode rule from WRAM; the block kinds the room uses | the blocks built from content equal the decoded seed's |
| B5 video commit | `VideoMemory` writes only through commit points (K4); the NMI batch: VRAM transfer queue, OAM and CGRAM DMA, register mirrors; the `ppu` encoder; the `vram`, `cgram`, `oam`, `ppu` projections | unit tests; the decoded State's `ppu` projection equals each record's PPU struct at every quiescent cut |
| B6 line program | the IRQ strand's HUD commands at lines 0 and 31, HDMA channels from their tables, `LinePoint` changes, the `DisplayFrame` | per line, the engine's register state over `area-brinstar` equals tier a's |
| B7 interpreter | `Program<F>`, `Cursor`, `Family` (D13), the slot store's spawn, run and delete policies (D12); content decode of the PLM ops slice 0 uses | unit tests; the `actors` projection for PLMs is equal under lockstep on `area-brinstar` (scroll and extension PLMs; the item idles) |

Files: `crates/game/src/{session,tick,nmi,lines,interpret}.rs` and their `state/` parts,
`crates/data/src/{rom,content,decompress}.rs`, `crates/verify/src/decode.rs` and
`coverage.rs` for each new field, `crates/tools` for B2. B lanes must not touch `runtime`,
`oracle` or the lock format, and add no gameplay beyond their row.

B1 must land before B2 and C; B3-B7 follow in that order but B3 and B4 may go beside B5
and B6 once B1 has landed, because they touch `data` and room state only.

### C. Domains, built against the oracle (lockstep first)

Each lane adds its State fields, their decode rules and coverage entries, and the rules,
and ends when its stream shows no difference under lockstep over every quiescent cut of its
row, then over a free run of N Ticks from the seed. Lanes of different domains run in
parallel once B7 has landed; the only shared lines are the module lists in `state/mod.rs`,
`decode.rs` and `coverage.rs`. Files: one module per domain under `crates/game/src/`
(`samus/`, `camera.rs`, `plm/`, `enemy/`, `hud.rs`, `sound/`, `message_box.rs`, `effects/`),
its decode rules in `verify`, its ROM tables in `data`. A C lane must not touch another
domain's module, the Tick order (B1's, changed only by a B-style lane), or `oracle`.

| Lane | Domain | Stream, row |
| --- | --- | --- |
| C1 | the main loop body: RNG, OAM clear and finalise, the gameplay phase order | `rng`, `area-brinstar` |
| C2 | Samus poses (bank `$91` pose table decoded) and pose transitions from input | `samus`, `area-brinstar` |
| C3 | Samus movement for A1's poses: run, jump, fall, land (and the elevator ride if A1 shows it) | `samus` |
| C4 | Samus block collision for the room's block kinds | `samus` |
| C5 | the camera, scroll areas, scroll PLM activation, BG scroll, level tilemap uploads | `camera`, `vram` |
| C6 | Samus drawing: animation (`SamusAnimation` family), per-pose tile DMA, palette, the OAM encoder and the intent sprites (D16) | `oam`, `vram`, `cgram` |
| C7 | enemy processing in `$A0` order, the elevator and face blocks | `actors`, `oam` |
| C8 | the Morph ball eye | `actors`, `oam` |
| C9 | the HUD: energy, the mini-map and its explored cells (the first `progress` fields) | `vram`, `progress` |
| C10 | A1's visual effects: animated tiles, palette FX, FX, layer blending, HDMA objects (each is a family user only if A1 shows it live) | `vram`, `cgram`, `ppu`, `effects` |
| C11 | `SoundCpu`: the music queue, three libraries, the driver echo model (D18) fitted from (d), the port log | `apu`, `item-morph-ball` |
| C12 | the item: draw, collected check, the pickup op (collected and equipped bits, the room's item bit, the fanfare) and `Next::Call(MessageBox)` | `progress`, `actors`, `apu` |
| C13 | the message box: open, shown (the button wait on the auto-read pad), close; its BG3 tilemap and HDMA table; return into the PLM pass | every typed stream over the box's lag frames; a Checkpoint at the midpoint restores and continues with equal digests |

### D. Free run, speed and exit

| Lane | Delivers | Proof |
| --- | --- | --- |
| D1 gate | RELOCK step g: `tools verify` free-runs both rows from the decoded seed with `Recorded` timing, folds (A3), compares (`verify::lock`), writes the ratchet of green (row, stream) pairs; CI runs it | the ratchet file; a decoder mutant breaks a row |
| D2…Dn red streams | one lane per red stream: lockstep over its first differing span, fix | the pair turns green; nothing else turns red |
| D-speed | profile and fix to D23 (incremental VRAM digest, the compositor's Mode 1 BG3 slow path) | `tools verify` reports ≥ 1,000 Ticks/s with every stream, ≥ 3,000 without `video`, ≥ 5,000 headless, on one CI core |
| D-exit | determinism (Checkpoint, restore, advance), one mutant per stream and one in the decoder, the intent view rendered at width 256 equal to the exact picture on both rows | the exit list of §15, each item a CI job |

D1 needs the lsnes lock (RELOCK steps d-e, root jobs; `scenarios/locks/` is not on this
base yet). Until it lands, A3's fold is proved against the recorder's own output.

## 3. The lockstep loop

`tools lockstep` (B2) is the development loop for every C lane, and only a localiser
(Retro rule 13): for each frame k of the cache it decodes record k, opens a Session from
that State, steps it once with frame k+1's pad and its `Recorded` timing, and compares the
named stream's projection with record k+1's decoded State, through `Canonical::diff` so the
first differing field path is named. Busy and non-quiescent cuts are skipped as D21 says.
Domains not yet built stay as decoded and are not compared, which is honest because the
whole State is replaced from the oracle every step and nothing in the engine adopts
(P6). A lane then runs the same stream in free run for N Ticks from the seed, which shows
the error that one-step lockstep can't: state its domain carries that the decoder reads
wrongly.

The order follows dependence: counters and input (B1), then RNG and the phase order (C1),
then Samus (C2-C4), because the camera, drawing, PLM touches and enemies all read her
position.

## 4. Speed and size checkpoints

| When | Measure | Bar |
| --- | --- | --- |
| B1 | headless empty Ticks per second | ≥ 50,000 (the frame skeleton's overhead must be negligible) |
| A5 | compositor time per frame on `area-brinstar` | ≤ 0.3 ms (a third of the 1,000 Ticks/s budget) |
| A3 | fold time per frame, `video` included | ≤ 0.2 ms |
| each C lane | headless Ticks per second on its row, reported in the commit | no fall of more than 10 % without a reason in the commit |
| D1 onward | D23's three modes in CI on every landing | D23 at slice 0's exit |
| every landing | A6's size report against D24 | a domain past its upper bound stops for a design review |

## 5. Open engineering questions, with recommendations

1. **Which oracle feeds the lockstep cache.** The seed is an lsnes state; lsnes runs at
   about 32-50 frames/s and its `video` stream isn't deterministic across runs (RELOCK §6).
   *Recommend:* cache lsnes records for the memories and typed streams (A2), and take the
   picture for A5 and D1 from wherever D25's `ReferenceProfile` takes `video`; A1 reports
   whether LakeSnes can start from this seed.
2. **The box's button wait.** `$85:846D` reads the joypad registers in a spin, not the
   latched pad. *Recommend:* `Condition` on the Tick's auto-read pad, checked in C13's
   lockstep; if it is one frame off, a rule value says which frame's pad, not a new input.
3. **Family users beyond the three listed in D2.** If A1 shows animated tiles or palette FX
   live, they implement `Family` in slice 0. *Recommend:* allowed (the trait's
   implementation list is the budget, not the trait count); C10 updates §4.3's table.
4. **Content tests need the ROM.** *Recommend:* B3's tests read the ROM path from the
   environment in CI's Verify job (ci-data), with synthetic byte tests for the decompressor
   in Quality.
5. **The gameplay pass order.** *Recommend:* B1 lists the phases from the bank `$82` log's
   `$8B44` call list, checked against FOUNDATION §5.1's "(confirm)" rows, so C lanes fill
   functions and never reorder.

## 6. Questions for Jim

None. Every choice above is engineering within ARCHITECTURE's decisions.
