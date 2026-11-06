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

**A2 status (`lane/slice0-a2`).** Done, as `oracle corpus ... --from-frame-pads DIR --cache
OUT [--cache-pictures]` (the corpus runner already plays rows from frame pads on lsnes) and
`verify::cache` (not `tools records` and `verify::source`: `source` holds the WRAM sources,
and this machine can't run the `tools` executable). Each row's `OUT/<row>/` holds
`records.bin`, every frame's whole `FrameRecord` in `oracle_record::codec`'s encoding, and
`index.txt`, the row's lock pins (`seed_record`, `seed_words`, `pads`, `frames`) and one
`frame offset length cut` line per frame, written last. `verify::cache::Cache::open` refuses
a cache whose pins differ from the `verify::lock::Row`; `verify::cache::fold` folds the
records through `data::span::SpanFold` under the recorder's rules. Proof (2026-11-06, both
rows with pictures, 62 s on lsnes): every machine stream's marks, `video` included, equal
`scenarios/locks/lsnes.toml`'s for both rows; a second read gives the same records;
`verify::decode` decodes all 477 quiescent frames of `area-brinstar` and 1,055 of 1,428 of
`item-morph-ball`, the other 373 `UnknownYieldSite` at ten `$85` message-box addresses
(360 at `$85:8496`). 600 MB with pictures (330 KB a frame), read and decoded at about
7,700 frames/s.

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

**B1 status (`lane/slice0-b1`).** Done: `Session::{open, step, begin, Begun::end}` in D6's
order; the dispatcher over `Frame` with `Next::{Continue, Call, Return, Sync, Yield}`, the
step budget and the depth check; the main loop's and gameplay pass's phases in the bank `$82`
order, empty; the NMI's commit as the ordered `NmiCommand` list (only the pad latch does
anything), the lag path and the counters as the bank `$80` handler keeps them (8-bit lag and
maximum); `TimingSource::{Recorded(Schedule), Modeled}` with no trait; `NmiTiming::Disabled`
for a vblank with NMIs off. Its unit tests and the `empty_ticks` example compile but did not
run on the lane's machine (Defender blocks test executables). Remaining:

- run the unit tests and `cargo run --release -p game --example empty_ticks` in CI, and
  record the Ticks/s;
- `open(State, &Content)`: there is no `Content` yet, so `open` takes `Rules`;
- the auto-repeat delays (the original's two work-RAM words) are `Rules.auto_repeat` until
  a row shows them change; then they join `Controller` with their decode;
- `verify`'s decoder: map a `VblankWithoutNmi` cut to `NmiTiming::Disabled`, and decide what
  a `NmiStillRunning` cut's timing is;
- the proof over both rows (`Counters` and `Controller` after each step equal the next
  decoded record's) needs A2's cache and B2's loop.

**B2 status (`lane/slice0-b2`).** Done as `verify::lockstep::lockstep(&Cache, &RoomMap,
frames)` and `tools lockstep LOCK CACHE ROW [FROM [TICKS]]` (compiled, not run here: this
machine runs it through a scratch DLL). For each pair of cached frames (k, k+1) that both
decode, with k quiescent, a `Session` opens on decode(k), steps one Tick with frame k+1's
pad and its cut's timing (`R` on time, `O` overrun, `V` disabled), and every field the
coverage table marks `Decoded` is compared with decode(k+1) through `Canonical::diff`
(`main` only at quiet cuts). The auto-repeat delays come from the first record's
`JOYPAD_CTR_REPEAT_FIRST` and `JOYPAD_CTR_REPEAT_NEXT` (`$7E:0087`, `$7E:0089`): both rows
hold 0 and 0 on every decoded frame. Run 2026-11-06 on A2's cache: `area-brinstar` 476
Ticks compared, none skipped; `item-morph-ball` 1,051, 376 pairs skipped for the 373
`UnknownYieldSite` frames. No Tick is equal yet, because game time and the RNG don't run.
B1 holds: the counters, the controller, `main`, `irq`, `timing.last` and `video.nmi_requested`
agree on every Tick except 24 of `item-morph-ball`'s, from frame 826, all inside the message
box, where the original's strand waits on lag NMIs (`Wait::LagFrame`) and B1's stub
`MessageBox` frame returns at once, so the engine commits instead (C13's). First differing
frame per field, the domain lanes' worklist:

| Field | `area-brinstar` | `item-morph-ball` | Owner |
| --- | --- | --- | --- |
| `clock.game_time.frames` (then `.seconds`) | 1 (27) | 1 (27) | gameplay `HealthAndTime` |
| `rng.word` | 1 | 1 | C1 |
| `video.memory.oam` | 1 | 1 | C1 (OAM clear and finalise), B5 |
| `samus.position.y.pixels` (`.subpixels`) | 1 (316) | 1 (316) | C3 |
| `video.memory.vram` | 7 | 7 | B5 |
| `room.camera.y.subpixels`, `.pixels` | 72, 74 | 72, 74 | camera |
| `video.memory.cgram` | 453 | 453 | palette FX |
| `samus.position.x`, `room.camera.x` | equal | 683 | C3, camera |
| counters, controller, `main` | equal | 826 | C13 (message box) |

Equal on every Tick of both rows: `game_time.hours`, `.minutes`, `mode`, `irq`, `timing.last`,
`samus.equipment.*`, `room.room`. Remaining: the "deliberately broken counter rule" proof
(a mutant `Rules` or `nmi.rs` edit run through lockstep) needs a machine that runs the
tests; `--stream S` filtering waits on the stream projections.

**B3 status (`lane/slice0-b3`).** Done: `data::rom` (`Rom::new` checks the 3 MiB size and
the SHA-256 `12b77c4b...` with `RomError`; `RomAddress { bank, offset }`; `Lorom` maps an
address to its file offset for any image, so synthetic images test the decoders);
`data::compression::decompress` (bank `$80`'s `$80:B0FF`/`$80:B119` format, all eight
commands and the extended header, `DecompressError::{Truncated, UnwrittenReference,
TooLong}`); `data::room` (`decode` of a header and its state list into `Room`, `Area`,
`Condition` for each routine `$8F:E5E6`-`$E678`, `RoomState` with the level data
decompressed at load, `Room::select(&Progress)`, `RoomKey` as the stable key and
`RoomMap` giving dense `RoomId`s in load order, `Rooms::load`); `data::address_map`
(`AddressMap` of `Location::{RoomHeader, DoorList, State(_, _, StatePart)}`).
`verify::lockstep::slice_zero_rooms` now comes from `RoomMap::slice_zero()`, so the Morph
Ball Room's ID is 0, not the placeholder 7. Tests: synthetic ones for every command, the
hash, the mapping, header decode, selection and each condition (Quality); the ROM test
`the_morph_ball_room_decodes_as_the_bank_log_says` reads `SM_ROM` or `rom/sm.smc` and
returns early without it, so it proves something only in CI's Verify job. All 28 ran on
the lane's machine through a scratch DLL against the real ROM: `$9EB1` for the seed, level
data `$C5:DED5` decompressing to 18,434 bytes (`2 + 0x3000 + 0x1800`, no layer 2), PLMs
`$8F:867E`, enemies `$A1:93AC`. Remaining for B4:

- turn `LevelData.bytes` into typed layer 1 blocks, BTS and (where present) layer 2; the
  length tells which: `2 + 3n` without layer 2, `2 + 5n` with it, `n` the blocks;
- the scroll table (`RoomState.scroll`, `$8F:9EED`) as scroll areas;
- typed decoders for the referenced records (PLM and enemy populations, FX, library
  background), which today are `RomAddress`es; `layer2_scroll`, `cre` and the scrollers
  stay raw bytes until a rule reads them;
- `Content`: `Rooms` is its first member, and `Session::open` should take it.

**C1 status (`lane/slice0-c1`).** Done: `Rng::advance` (bank `$80` log, `$80:8111`), called
once per main-loop iteration, the only call slice 0's phases make so far; `GameTime::tick`
(bank `$82` log, `$82:DB69`: signed compares against 60, the 99:59:59.59 cap from 100 hours),
the gameplay pass's `HealthAndTime` phase; and the sprite buffer's frame on `NmiBatch`:
`clear_high_oam` (`$80:8B1A`), `restart_sprites` (`$82:8957`) and `finalise_oam` (`$80:896E`:
Y = 240 from the drawn count on, nothing at 128 or more), with the NMI's OAM half of
`OamAndCgram` (`$80:933A`) copying the buffer to `video.memory.oam`. The decoder now reads
`video.batch.oam` from the original's buffer (`source::OAM`), so hidden entries keep the
stale tiles the original leaves; `video.batch.sprites` stays open (always 0 at a quiet cut).
Lockstep on A2's cache, 2026-11-06, before and after (Ticks differing, first frame):

| Field | `area-brinstar` (476) before | after | `item-morph-ball` (1,427) before | after |
| --- | --- | --- | --- | --- |
| `rng.word` | 476, 1 | equal | 1,028, 1 | equal |
| `clock.game_time.frames` | 476, 1 | equal | 1,028, 1 | 399, 817 |
| `clock.game_time.seconds` | 8, 27 | equal | 17, 27 | equal |
| `video.memory.oam` | 465, 1 | 476, 1 | 903, 1 | 1,427, 1 |
| `video.batch.oam` (new) | | 476, 1 | | 1,427, 1 |

`item-morph-ball`'s 399 game-time Ticks from 817 are the message box's lag frames, where the
original's strand waits and the engine's stub box returns at once (C13), the same Ticks as the
counters'. OAM now differs on every Tick (before, a stale copy of the previous frame's matched
where Samus's sprites didn't change), but only inside the drawn prefix: on every compared
Tick of both rows, every entry from the last non-hidden one on is equal (Y, the stale X, tile
and attributes, and the high table's bits), and the differing entries are Samus's (at most
23 and 27 sprites). The rest is C6's drawing. Remaining: the other RNG call sites join with
their domains (enemy AI, enemy projectiles, PLMs and Samus's effects; none runs in slice 0's
phases yet); the out-of-health branches of `HealthAndTime` join with the reserve-tank and
death game states; the Samus tile transfer flags reset with C6; the unit tests compiled but
did not run here (Defender), so they need CI. Lockstep time is unchanged (0.11 s and 0.17 s
for the two rows, decode included).

**C13 status (`lane/slice0-c13`), the box's control flow.** Done: `game::message_box::step`
runs `Frame::MessageBox` through bank `$85`'s display message box (`$85:8080`) in its order,
yielding `Wait::LagFrame` at each `JSR $8136`: initialise PPU (waits at `$8148`, `$81B2`),
clear tilemap (`$8203`), initialise message box (the tilemap writer's `$82B8`, the scroll
HDMA table's `$859C` via `$83BA`, the upload's `$8321`), play 2 lag frames (`$8578`, X = 2),
open (13 `$859B` waits, radius 0 to `$1800` by `$200`), the interaction countdown (`$8493`,
X = 360, or 10 for messages `$14`-`$16` and `$18`), `LOOP_INPUT` (`$84A3`: any auto-read
button), close (13 waits, `$1800` down to 0), clear tilemap, restore PPU (`$861C`, `$8651`),
play 2 lag frames, RTL. `message_box::display` is the entry C12's pickup op calls; it sets the
new `State::message` (`$1C1F`, decoded, compared) and pushes `MessageBoxAt::Start`, a
position never at a cut. Game time needs nothing of its own: the gameplay pass's
`HealthAndTime` comes after the PLM pass, so it stops while the box is up, as the original's
does. The PPU, tilemap, VRAM and HDMA work are named no-ops (`message_box::Work`): B5 takes
the uploads and register mirrors, B6 the scroll HDMA table and channel; the music queue and
sound effects of each lag frame are C11's. Lockstep on A2's cache, 2026-11-06 (Ticks differing,
first frame):

| Field | `item-morph-ball` (1,427) before | after |
| --- | --- | --- |
| `clock.counters.completed`, `.lag` | 399, 398 from 817 | 1, 817 |
| `clock.counters.max_lag` | 202, 870 | equal |
| `clock.game_time.frames` | 399, 817 | 1, 817 |
| `controller.pressed`, `.repeat`, `.repeat_delay` | 399, 817 | 1, 817 |
| `main.frames`, `main.waiting` | 399, 817 | 1, 817 |
| `message` (new) | | 1, 817 |
| Ticks equal on every field | 0 | 392 |

`area-brinstar` is unchanged (it has no box). Frame 817 is the pickup Tick itself: the item's
pickup op (C12) sets the message and calls the box. Remaining: the entry from C12; the box's
graphics (B5, B6); the save confirmation's Yes/No loop (messages `$17`, `$1C`) and the gunship
branch with saving; the unit tests compiled but did not run here (Defender).

**B5 status (`lane/slice0-b5`), the NMI's video commit points.** Done: `game::commit` holds the
three commit points the NMI's `COMMIT` list runs, the only writers of `video.memory` and
`video.committed`: `upload_oam_and_cgram` (bank `$80` log, `$80:933A`: the OAM buffer to OAM
and the whole palette buffer, the new `NmiBatch::cgram`, to CGRAM), `commit_registers`
(`$80:91EE`: the mirrors to `video.committed`) and `process_vram_writes` (`$80:8C83`: each
queued `VramTransfer` in order, then the queue empties). A transfer carries `VramStep::{Row,
Column}` (the destination's top bit) and `VramSource::{Words, Content}`: words read from a
State buffer, or ROM graphics by length that content (B3) doesn't decode yet, which write
nothing (`// open:`). The decoder reads `video.batch.cgram` from `PALETTE_BUFFER` (`$7E:C000`),
`video.batch.vram` from `vram_write_queue` (`$7E:00D0`, seven bytes an entry) up to
`VRAM_WRITE_QUEUE_TAIL` (`$7E:0330`), `video.batch.registers` from `REG_INIDISP` (`$51`),
`REG_TM` (`$69`), `REG_TS` (`$6B`) and `REG_BG1HOFS`-`REG_BG3VOFS` (`$B1`-`$BB`), and
`video.committed` from the record's `ppu`. Lockstep on A2's cache, 2026-11-06 (Ticks
differing, first frame):

| Field | `area-brinstar` (476) before | after | `item-morph-ball` (1,427) before | after |
| --- | --- | --- | --- | --- |
| `video.memory.vram` | 73, 7 | 73, 7 | 345, 7 | 345, 7 |
| `video.memory.cgram` | 5, 453 | 5, 453 | 130, 453 | 130, 453 |
| `video.committed` (new) | | 476, 1 | | 1,031, 1 |
| `video.batch.registers` `.forced_blank`, `.brightness`, `.main_screen`, `.sub_screen` (new) | | equal | | equal |
| `video.batch.registers.scroll` (new) | | 342, 74 | | 557, 74 |
| `video.batch.cgram` (new) | | 5, 453 | | 128, 453 |
| `video.batch.vram` (new) | | equal | | equal |

Why nothing moved: at every quiet cut the NMI has just emptied the queue and uploaded the
palette buffer, so the engine's commit of decode(k)'s batch reproduces what was already there.
What changes VRAM and CGRAM within a Tick is the main strand filling the batch: frame 7's first
VRAM difference is word `$585C`, in the HUD tilemap the gameplay pass queues from
`HUD_TILEMAP` (`$7E:C608`) to `$5820` every frame (the HUD lane), and CGRAM's at 453 is palette
FX writing the buffer (the palette FX lane). The scroll mirrors are the camera's (BG1 and BG2
from the layer positions). Remaining:

- done (`lane/slice0-b5b`): the BG1 scroll. The record holds each scroll register's ten
  bits, the PPU's, and the mirror the whole word the game keeps: `REG_BG1HOFS` holds
  `0400h` at frame 0 of both rows, the record's BG1HOFS 0 (likewise BG1VOFS `1200h` and
  `200h`, BG2HOFS `FEC0h` and `2C0h`). `$80:91EE` writes the mirrors in low, high pairs, so
  the PPU's latch rule leaves the word's low ten bits; the commit rule was wrong, not the
  decoder, and `commit_registers` now masks. The cache has no `writes` (no write-log tier),
  so this rests on the bank log and the frame-0 values;
- done (`lane/slice0-b5b`): the `ppu` encoder. `Registers` holds every register
  `$80:91EE` writes (OBSEL, BGMODE, MOSAIC, BGnSC, BGnmNBA, four scrolls, M7SEL and the
  mode 7 matrix, the windows, TM, TS, TMW, TSW, CGWSEL, CGADSUB, the fixed colour,
  SETINI), `NmiBatch.fixed_colour` the three COLDATA writes and `Video.dma` the eight DMA
  channels; each has a decode rule (the mirrors from work RAM, `committed` and `dma` from
  the record's `ppu`) and a lockstep comparator. `verify::ppu::encode` gives the record's
  register struct and `verify::ppu::digest` the fold's `ppu` (in `verify`: the record's
  layout is verification's, D6). Proof on A2's cache: encode(decode(record)) equals the
  recorded `ppu` digest at all 477 and 1,428 quiescent cuts, and committing a cut's decoded
  mirrors gives its decoded `committed` at all 477 and 1,029 of 1,428 (the 399 others are
  the message box's lag NMIs, which don't commit). Lockstep (Ticks differing, first frame):
  `video.committed` 342, 74 and 567, 74 (all the camera's scroll but, in
  `item-morph-ball`, CGADSUB from 817, BG3SC from 818 and the fixed colour from 1260: the
  message box); `video.dma` 64, 84 and 149, 84 (the NMI's DMAs don't set the channels
  yet); the new batch mirrors equal in `area-brinstar`, and in `item-morph-ball` equal but
  W34SEL, TSW (2, 1250) and the COLDATA writes (20, 1250). Open: the channels' settings
  from the NMI's DMAs and the HDMA queue; `REG_BGMODE_FAKE`'s mode 7 test; the packed
  registers' parts (newtypes over the PPU's byte for now);
- Samus's tiles (`$80:9376`), the animated tiles (`$80:9416`), mode 7 (`$80:8BBA`) and the
  read table (`$80:8EA2`) are still named no-ops; Samus's and the animated tiles need B3's
  graphics content;
- the unit tests in `game::commit` compiled but did not run here (Defender).

**C2 status (`lane/slice0-c2`), Samus's pose as typed state.** Done: `data::pose` decodes
bank `$91`'s pose definitions (`$91:B629`, 253 poses of 8 bytes, to `$91:BE11`) into
`PoseDefinition { facing: PoseFacing, movement: MovementType, next: Option<PoseId>, shot:
Shot, y_offset, y_radius }`: `MovementType` is the 28 movement types (`TryFrom<u8>`, $1C on
an error), `PoseFacing::{Forward, Left, Right, Unknown(u8)}` (the unused poses $21 and $22
hold 1 and 2; listed in LEGACY), `Shot::{Fires(ShotDirection), Withheld(u8)}` (high nybble
set: $FA, $FB, $FC, $FF). `game::state::samus` holds `pose: PoseId`, `previous` and
`last_different` as `PoseRecord { pose, facing, movement }` (the bytes the original copies
beside each pose word), `facing` (now `PoseFacing`) and `Movement`, one variant per movement
type (`Movement::from(MovementType)`, `Movement::kind()` back, byte-exact); `Landing` is gone
because the landing poses ($A4-$A7, $E0-$E5) are standing ones, so landing keys on the pose.
The decoder reads `SAMUS_POSE` (`$0A1C`), `SAMUS_POSE_X_DIR`, `SAMUS_MOVEMENT_TYPE`,
`SAMUS_PREV_POSE`, `SAMUS_PREV_POSE_X_DIR`, `SAMUS_PREV_MOVEMENT_TYPE2`,
`SAMUS_LAST_DIFFERENT_POSE` and its two bytes (`$0A1C`-`$0A27`); a pose past $FC or a
movement type past $1B is `Unrepresentable`. The ROM test ran through a scratch DLL against
the real ROM; on every decoded record of both rows the RAM's facing and movement type equal
the table's for the pose. Lockstep on A2's cache, 2026-11-06 (Ticks differing, first frame;
all new fields):

| Field | `area-brinstar` (476) | `item-morph-ball` (1,427) |
| --- | --- | --- |
| `samus.pose`, `samus.previous.pose` | equal | 173, 681 |
| `samus.last_different.pose` | equal | 172, 687 |
| `samus.movement`, `samus.previous.movement` | equal | 105, 681 |
| `samus.last_different.movement` | equal | 104, 687 |
| `samus.facing`, `.previous.facing`, `.last_different.facing` | equal | 4, 681 (687) |

Poses reached (frames, first..last): `area-brinstar` holds pose 0 (facing forward, the
elevator) on all 477 records, though her Y differs from frame 1 (the elevator's descent).
`item-morph-ball`: 0 (681, 0..680), then $25 turn (6), $0A/$12 run left (21, 20), $1A spin
left (52), $84 wall jump (22), $6C/$16/$18/$52/$14 normal jump left, $A5 land left (2), $02
stand left (1), $E4 land (2), $09/$11 run right (8, 6), $19 spin right (4), $17 jump (21),
$37/$3D crouch transitions (6, 6), $1D/$1E morph ball (1, 38), $27 crouch (1), $4B/$4D/$51
jump right (1, 1, 7), $6B jump aiming down-right (417, 817..1326, the message box's frames
included), $D1/$89 ran into wall (1, 54), $07 stand aiming down-right (40). Movement types
reached: standing, running, normal and spin jumping, morph ball on ground, crouching,
crouching transition, turning around on ground, wall jumping, ran into wall.
Remaining for C3/C4: the pose transitions from input (bank `$91`'s transition tables and
`$91:82EC`'s "new pose if not affected by buttons"), the movement each type runs, and the
data `Movement`'s variants carry (speeds); `samus.aim` stays open (it is the pose's `Shot`);
`SAMUS_NEW_POSE`, `_INTERRUPTED` and `_TRANSITIONAL` (`$0A28`-`$0A2C`) are not decoded; the
elevator ride in `area-brinstar` keeps pose 0 and moves Y; the verify and data unit tests
compiled but only the data ones ran here (Defender).

**C3a status (`lane/slice0-c3a`), Samus's pose from input.** Done: `data::pose::transition`
decodes bank `$91`'s pose transition table (`$91:9EE2`, a bank `$91` pointer a pose to
six-byte entries, newly pressed controls, held controls, new pose, ended by $FFFF) into
`PoseTransitions` of `Transition { pressed: Controls, held: Controls, pose }`; the ROM test
(scratch DLL, real ROM) passes. `game::pose` holds the `Content` the Session reads
(`Poses`, `PoseTransitions`; `Session::open` takes an `Arc<Content>`), the input handler
(`$91:8000`/`$91:81A9`: types 7, 9, $B and the crouching transitions take no input; the
first entry whose controls match, under the default bindings; else `$91:82EC`'s next pose,
or the same pose while running, normal jumping or falling as a ball with speed) and the
pose change after movement (`$91:EB88`: the table's facing and movement type when the pose
differs from the previous, then the history). `SAMUS_NEW_POSE` is a Tick-local
`PoseRequests` (consumed in the same frame); `_INTERRUPTED` and `_TRANSITIONAL` are not
modelled yet. `Movement::Unused` holds `UnusedMovement`; `u8::from` for `MovementType` and
`Shot` are exhaustive matches. Lockstep `item-morph-ball` (1,427 Ticks; differing, first):

| Field | before | after | Owner |
| --- | --- | --- | --- |
| `samus.pose`, `samus.previous.pose` | 173, 681 | 64, 687 | animation (turn end $25 to $02), C3b, C4 |
| `samus.last_different.pose` | 172, 687 | 6, 687 | same |
| `samus.movement` | 105, 681 | 63, 687 | same |
| `samus.facing`, `.previous.facing` | 4, 681 | equal | |
| `samus.position.x` | 215, 683 | 215, 683 | C3a-2 (running speed) |

Remaining: the movement handlers' horizontal motion (standing and running: X speed, the
acceleration and the running speed table, bank `$90`) was not reached; the pose animation's
transition instructions (the turn ends at 687 by animation, not input); the elevator's
"no input" case (C7); the controller bindings as State; the interrupted and transitional
requests (hurt, blocks); the tools binary's lockstep runs on empty content (it reads no ROM).

**C9 status (`lane/slice0-c9`), the HUD.** Done: `game::state::hud::Hud` holds the HUD
tilemap's three rows as typed cells (`video::TilemapEntry`: tile, palette, priority, flips;
`from_word` and `word` round-trip every word), the selected and auto-cancel items
(`HudItem`), the values it last drew (`Drawn`: energy, missiles, super missiles, power bombs,
the highlighted item) and the last highlight's palette; `Equipment` gains the super missile
and power bomb gauges and `ReserveMode`, `Counters` the 8-bit requested-NMI count
(`$80:95ED`). `game::hud::phase` is the gameplay pass's HUD phase, as `$80:9B44` runs it: the
AUTO reserve sign, the tanks and energy digits when energy differs from the drawn value, each
count with a ceiling when it differs, the selection's highlight and the auto-cancel flash,
then the queued upload of the State's tilemap to VRAM `$5820` in `video.batch.vram`, which the
NMI commits. Decoded from `HUD_TILEMAP` (`$7E:C608`), `HUD_ITEM_INDEX`,
`SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX`, `SAMUS_PREV_HEALTH` to `SAMUS_PREV_HUD_ITEM_INDEX`,
`HUD_ITEM_TILEMAP_PALETTE_BITS`, `SAMUS_(MAX_)SUPER_MISSILES`, `SAMUS_(MAX_)POWER_BOMBS`,
`RESERVE_HEALTH_MODE` and `NMI_FRAME_COUNTER_BYTE`. Lockstep on A2's cache, 2026-11-06 (Ticks
differing, first frame):

| Field | `area-brinstar` (476) before | after | `item-morph-ball` (1,427) before | after |
| --- | --- | --- | --- | --- |
| `video.memory.vram` | 73, 7 | 73, 7 | 345, 7 | 345, 7 |
| of which the HUD rows outside the mini-map | 0 | 0 | 0 | 0 |
| of which the mini-map (columns 26-31) | 61, 7 | 61, 7 | 131, 7 | 131, 7 |
| of which BG1/BG2 and tiles (B4, B3) | 17, 244 | 17, 244 | 63, 244 | 63, 244 |
| of which sprites (C6) | | | 209, 681 | 209, 681 |
| of which BG3 below the HUD (C13's box) | | | 4, 820 | 4, 820 |
| `video.batch.vram` | equal | equal | equal | equal |
| `hud.tilemap` (new) | | 61, 7 (mini-map cells only) | | 131, 7 (mini-map cells only) |
| `hud.drawn.*`, `.selected`, `.auto_cancel`, `.item_palette` (new) | | equal | | equal |
| `samus.equipment.super_missiles`, `.power_bombs`, `.reserve_mode` (new) | | equal | | equal |
| `clock.counters.requested_byte` (new) | | equal | | 1, 817 (C13's box) |

Why VRAM's count didn't move: at a quiet cut the HUD rows in VRAM already hold the tilemap the
last NMI uploaded, so the engine's upload rewrites what was there. Proof that the engine's
upload is what writes them: with VRAM `$5820`-`$587F` zeroed in decode(k) before each Tick,
`area-brinstar` gives the same table (the engine restores every HUD word but the mini-map's);
`item-morph-ball` too until 818, after which the message box's lag frames run no gameplay pass
in either. Frame 7's first VRAM difference (`$585C`, row 1 column 28) is the mini-map's Samus
cell flashing between palettes 7 and 2. Remaining:

- open: the mini-map (bank `$90`'s update and its explored cells, with `progress`), which owns
  every remaining HUD difference: C9's next step, or the map lane's;
- open: the switch click (sound `39h`, C11) and the debug options' three-digit super missile
  count (vanilla never sets them);
- open: a count past what its cells hold (two digits past 99, three past 999) indexes past the
  digit table into the timer code (`$80:9DE7`); no vanilla count gets there, and the engine leaves the
  cell's tile;
- the HUD's initialisation (`$80:9A79`; the item icons on pickup, `$80:9A0E` to `$80:9A3E`) joins with room
  entry and item pickup (C12); the unit tests compiled but don't run on this machine.

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
