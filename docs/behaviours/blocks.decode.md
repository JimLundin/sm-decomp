# Blocks that break and come back, and the scroll cells: decode notes

For the observer, the adversary, the asset importer and the domain harness's decoder only.
Implementers never see this file (MODEL §5). It holds every source the spec draws on: the
original's routines, its RAM and its codes. Addresses are the bank logs' (`$TEMP/banklogs/bank_*.txt`,
https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 44 rows cached in `$TEMP/collab/scenes/cache` (the 32 rows of
`$TEMP/collab/more-caches/cache` and 12 more) were dumped per frame by
`$TEMP/collab/blocks-spec/dump` (a cdylib over `oracle_record`, run by `dumprun.py`; output in
`$TEMP/collab/blocks-spec/out`: per record Samus's position and pose, every PLM slot that changed,
the 2x2 cells at every occupied slot's block, the scroll table, and every level cell that changed)
and read over time. The bank logs were then read to name the cases and pin the edge cases. Each
row was then recorded again on lsnes with the instructions that start this part's reactions
watched (`watch.txt`, `rec_hits.sh`: the setups `$84:CE37`, `$84:CDEA`, `$84:CE83`, `$84:CEDA`
with its branches `$84:CF0C` (bomb) and `$84:CEF4` (power bomb), `$84:CE6B`, `$84:B3C1`, `$84:CF67`,
`$84:CF2E`, `$84:CFA0`; the scroll touch `$84:B393` and its trigger `$84:B3AD`; the centre's
inside spawn `$94:995E`; the refused spawn `$84:84F9`). Each re-recording's `records.bin` equals
the cache's byte for byte (all 44), so the hits (`hitcache/<row>/hits.txt`) belong to the cached
frames. The observer's tools, all in `$TEMP/collab/blocks-spec`:

- `importer.py`: the asset importer's mapping, run on the ROM (`rom/sm.smc`): it writes the
  game's reactions in the spec's format (§2) at test time, for the harness and the per-frame run,
  types each cell's mark and what she meets inside it, refuses what the original would misread,
  and gives the decoder its maps (below). Its output is never committed.
- `scan_rooms.py`: the importer's scan of every room (§The importer's scan).
- `invented.py`: the vectors' reactions, invented in the same format (§The harness cases).
- `model.py`: the reference model, in the spec's terms (cells, marks, places, steps).
- `harness.py`: the per-frame decoder and comparison over the dumps: decode frame N's State, run
  the model one frame, compare with frame N+1; its own inputs are read from N+1 (below).
  `harness_hits.py` runs it with the inputs the hits decide. `dbg.py ROW FRAME` prints one frame.
- `gen.py` picked the recorded frames (the first version's 20, then at most two per new outcome,
  from different rows, from frames the model matches with the hits' inputs; `rec.toml`), all of
  them harness cases now; `cons.py` writes the vectors, all constructed, over `invented.py` (their
  after from the model), `assemble.py` writes `scenarios/vectors/blocks.toml` with the invented
  reactions they use, and `vec.py` runs every vector through the model: 86 of 86 agree.
- `cut.py` cuts the old code into the areas' files in `old/` for `tools size`.

**Results** (`harness_hits.py`). Of the 20,615 frames that ran one frame of gameplay in one room
(§The harness; 125 of them with the run paused), 786 are frames where a reaction of this part ran,
the screens changed, or the hits show a start or a scroll touch; the model matches the decoded
State on all 786, and on the 19,829 others. A null model (nothing changes) matches 183 of the 786
(the scroll touches whose changes are already in force), and 431 on the cells and screens alone.
Every start is decided by a hit: 111 starts on 116 frames (98 crumble, 3 shot, 4 power bomb, 1
super missile and 5 bomb block starts, each bomb block's weapon from its branch: 3 power bombs, 2
bombs) and 197 scroll touches (and 79 repeat touches of a reaction already touched that frame).
What the hits don't carry is the cell: a start's cell is the one record N+1 shows it on, and a
scroll touch's is the scroll reaction nearest her not yet touched that frame; on the frames where
the screens change (8), that choice is checked by the screens.

Without the hits (`harness.py`, every start read from N+1): 603 of 603 on the frames where a
reaction ran or the screens changed, 116 of them with a start or a touch copied from N+1, and the
weapon of the 2 bomb block starts of item-reserve-tank read wrongly as a power bomb (the State
after the frame is the same; only the sound differs). The hits also show what N+1 can't: the
setups that ran and started nothing (r03's bomb blocks: a shot at 185, her touches at 458, 460 and
461; r11's crumble blocks under a power bomb's blast at 140 to 158, ten times), the same setup
shared by other parts' PLMs (`$84:B3C1` under `$B781` and `$B7BF`, boss-phantoon and
boss-kraid-defeated), and 36 refused spawns, all in terrain-quicksand's paused frames 423 to 455,
where part 4's quicksand fills every place.

**Row names.** As in the other decode notes, the chain rows go by opaque ids:

| Id | Row |
| --- | --- |
| r01 | chain-0012-91F8 |
| r02 | chain-0013-92FD |
| r03 | chain-0014-96BA (bomb blocks met and left whole) |
| r04 | chain-0057-9FBA (the crumble bridge, PLM `$D044`) |
| r05 | chain-0059-A293 |
| r06 | chain-0097-ACF0 (the crumble bridge, PLM `$D054`) |
| r07 | chain-0100-B07A |
| r08 | chain-0106-AD5E |
| r09 | chain-0220-D017 |
| r10 | chain-0224-D0B9 |
| r11 | chain-0232-D5A7 (two power bomb blocks, PLM `$D088`) |

The other rows go by their names: item-charge-beam holds the super missile block (`$D08C`, frame
53) and the bomb blocks a power bomb breaks (122, 126, 774), item-reserve-tank the bomb blocks a
bomb breaks (572, 638), area-brinstar, item-morph-ball and r02 the scroll touches.

## The harness cases

The vectors hold invented content only (`$TEMP/collab/blocks-spec/invented.py`): no reaction,
cell, look or scroll change of the game is committed (MODEL §1 decision 4). So every recorded frame
is a **harness case**: the integrator's harness, run on the records with the importer's reactions
and the room's own cells, must match it, and a constructed vector over the invented content carries
the same rule for the implementer. The scroll touches are cases too: their cells, looks, screens and
change lists are the room's, and their touches are the harness's inference (the scroll cell nearest
her, the way `Down`), not recorded inputs. The first version's ids are kept here so its history
reads.

| Id | Frame | What it shows | The rule's vector |
| --- | --- | --- | --- |
| R1 | r04:110 | a crumble block landed on starts in the highest free place (39 held), takes the crumble's whole look, waits its first wait | C-place-highest |
| R2 | r06:39 | the same on the other bridge, two places held | C-place-highest |
| R3 | area-norfair:25 | a 1x2 and a 1x1 shot block hit in one frame take 38 and 37; their first shows run in the frame | C-shot-regrows, C-shot-blast, C-place-two |
| R4 | item-morph-ball:1236 | a shot block's start below five held places | C-shot-regrows |
| R5 | r11:120 | one power bomb's blast starts two power bomb blocks | C-pb-first-wait, C-place-two |
| R6 | r04:113 | a sound and a show in one step's run while another crumble counts down | C-run-sound-then-show, C-run-wait |
| R7 | r06:42 | a sound and a show in one run | C-run-sound-then-show |
| R8 | area-norfair:29 | two shows of a 1x2 and a 1x1 pattern | C-pattern-1x2-air, C-pattern-2x1-regrow |
| R9 | r04:115 | three crumbles at three steps in one run | C-run-order |
| R10 | area-norfair:33 | two more shows | C-pattern-2x1-regrow |
| R11 | area-norfair:26 | two waits counting down | C-run-wait |
| R12 | r04:111 | a crumble's first wait counting down | C-run-wait |
| R13 | r04:159 | a crumble growing back among 18 running | C-regrow-air, C-regrow-hold |
| R14 | area-norfair:38 | two reactions ending | C-run-end |
| R15 | r06:55 | a start takes a lower place while 39 ends in the run | C-regrow-touch-again |
| R16 | r04:160 | a reaction ends after growing back while a start takes place 20 | C-regrow-hold, C-regrow-touch-again |
| R17 | area-brinstar:92 | the Morph Ball Room's shaft: a scroll cell touched twice in the frame opens a screen once | C-scroll-move, C-scroll-twice |
| R18 | item-morph-ball:92 | the same frame on the other route (equal to R17 line for line) | C-scroll-move |
| R19 | area-brinstar:391 | another of the shaft's scroll cells, touched twice, closing that screen | C-scroll-move, C-scroll-twice |
| R20 | item-morph-ball:391 | the same (equal to R19) | C-scroll-move |
| R21 | item-charge-beam:53 | a super missile block's start, the highest free place past held ones | C-super-by-super, C-place-highest |
| R22 | item-reserve-tank:572 | a bomb's blast starts a 2x2 bomb block past its sound | C-bomb-by-bomb |
| R23 | item-charge-beam:122 | a power bomb's blast starts a bomb block before her move touches a scroll cell | C-bomb-by-power-bomb, C-order-hit-then-touch, C-scroll-holds-place |
| R24 | area-brinstar:93 | a scroll cell touched on every frame she stays in it: its changes are already in force (a null model matches it) | C-scroll-move |
| R25 | r02:47 | the same in r02 | C-scroll-move |
| R26 | item-charge-beam:139 | inside a scroll cell at her centre and a move reaching it, two bomb blocks counting down | C-scroll-centre, C-scroll-twice |

All 26 are among the 786 frames the reference model matches with the importer's reactions
(`harness_hits.py`). None was re-derived as a vector: each rule already had a constructed vector,
and a re-derived after would be the model's, not the record's.

## The split of W2

The four parts are cut by verifiability and by the PLM families they hold:

| Part | PLMs (bank `$84`) | Collision tables (bank `$94`) |
| --- | --- | --- |
| 1, this spec | `$CFFC`..`$D024` (bombed special), `$D030`..`$D0D4` less `$D094` (crumble, speed, shot, power bomb, super missile, bomb), `$B703` (scroll PLM), `$B6FF` (scroll block touch), `$B63B`..`$B647` (extensions), `$B62F` (nothing) | special `$9139` and area tables `$91D9`..; shootable `$9EA6`; bombable `$936B`, `$A012`; bombed `$9DA4` |
| 2, doors and gates | `$C842`..`$C8CA` (doors), `$C7E2`/`$C83E` (generic shot trigger), `$C806`..`$C836` (gate triggers, gates), `$BAF4` and the other boss doors, `$B974` (gate block) | shootable BTS `40h`..`4Eh` |
| 3, grapple, spikes, hazards | `$D0D8`..`$D0E8` (grapple), `$D094` (enemy breakable), spike reactions `$94:8E83`.., `$B6CB`/`$B6CF` (plants) | grapple and spike reaction tables |
| 4, room-scripted cells | `$B713`..`$B73F` (quicksand, sand falls), `$B64B`/`$B64F` (treadmills), `$B70F` (ice physics), stations `$B6D7`.., `$B76B` (save), the boss and escape PLMs | special air area tables `$9A06`.. |

Items (`$EED3`, `$EED7`..`$EFxx`) are W3's.

## State

| Quantity | RAM | Decode |
| --- | --- | --- |
| a cell's kind | level data `$7F:0002 + 2i`, bits 12-15 (block type) | 0 `Air`; 3, 4, 7 `ReactingAir` (special air, shootable air, bombable air); 5 `StandInAcross`; 8 `Solid`; Bh, Ch, Fh `Block` (special, shootable, bombable block); Dh `StandInDown`; others are other domains' kinds |
| a cell's mark | BTS `$7F:6402 + i` | through the class's table with the area `$079F` (§The importer's mapping, cells), and for special air with BTS 80h and up the inside table; a stand-in's reach is the BTS signed. An `Air` or `Solid` cell doesn't read its BTS; the decoder types it as the mark of the reaction running on it (its first cell by the family's class, the others as reaches) |
| a cell's look | level data bits 0-9 (tile), 10 (x flip), 11 (y flip) | as is |
| a place | PLM slot `x = 2 × place`, `$1C37,x` ≠ 0 | slot index ÷ 2 |
| its reaction and script | `$1C37,x` (PLM ID) | `importer.BY_ID`: the ID's reaction and which script (`$D098`..`$D0B4` the bomb block's `touched`, `$D0B8`..`$D0D4` its `broken`, `$CFFC`..`$D024` and the lists `$C91C`/`$C922` `revealed`, the rest `broken`); the bombed special IDs name only the shape, so the reaction is the cell's mark |
| its cell | `$1C87,x` (block index × 2) | ÷ 2, then `(i mod w, i div w)`, `w` = `$07A5` |
| its next step | `$1D27,x` (instruction list pointer) | the index of the step whose instruction is at that address (`importer.script`, `at`); after a draw the pointer is past it, so it names the next step |
| its wait | `$7E:DE1C,x` (instruction timer) | as is |
| what it grows back as | `$1E17,x` (PLM respawn block) | bits 12-15 as a kind: `Block` or `ReactingAir`; the low bits are the whole look, which is the reaction's. Junk (the last occupant's) for the `$B3C1` shot blocks, which never read it |
| a scroll reaction | `$1C37,x` = `$B703`, `$1DC7,x` (room argument: its scroll change list in bank `$8F`) | the list: byte pairs (screen index, value) up to a byte with bit 7 set; screen `(n mod w, n div w)` with `w` = `$07A9`; value 0 `Closed`, 1 `Short`, 2 `Open` |
| its touched | `$1E17,x` bit 15 (the trigger, set by `$84:B393`, which also moves `$1D27,x` past the sleep and sets the timer to 1) | set: touched. The scroll update (`$84:8B55`) clears it in the next pass, so only a paused frame leaves it set at a commit |
| other parts' reactions | any other `$1C37,x` | held |
| a hold until the run | `$1C37,x` = `$B62F` or `$B633` (nothing) or `$B6FF` (the scroll touch), whose list is a delete (`$84:AAE3`) | the place is held; only a paused frame leaves one at a commit, and the next pass deletes it |
| the room's screens | `$7E:CD20` | as the camera decodes them |

Not State: a sleeping scroll PLM's timer (its
sleep leaves the timer at 0, so it counts down from FFFFh and re-runs the sleep after 65,536
frames, which changes nothing); `$1CD7,x` (the pre-instruction: an RTS for every reaction here);
`$7E:DE6C,x` (the draw instruction pointer, read only while drawing); `$1D77,x`, `$1DC7,x` and
`$7E:DF0C,x` for the block reactions (zeroed at spawn, never read); the BTS of `Air` and `Solid`
cells (kept, and read only once a step makes the cell a reacting kind or a stand-in again).

**RAM shape** (fields decoded as one word copied as-is): the wait, a frame count, which is what the
player sees as "how long until the next picture"; the look, the tile word's low 12 bits, which is
the picture the cell shows. Nothing narrower holds either.

The spec's enum orders are alphabetical, or spatial where stated, never the RAM codes': the kinds
against the block types (0, 3/4/7, 5, 8, Bh/Ch/Fh, Dh); the families `Bomb`, `Crumble`,
`PowerBomb`, `Shot`, `Speed`, `SuperMissile`; the ways `Down`, `Left`, `Look`, `Right`, `Up`
against `$0B02`'s 3, 0, Fh, 1, 2; the families a `Nothing` is met as, `Bomb`, `Crumble`, `Shot`,
against the block types Fh, Bh, Ch; the points top to bottom, `Top`, `Centre`, `Bottom`, against
`$1E73`'s 2, 1, 0; the hits `Beam`, `Bomb`, `Grapple`, `Missile`, `PowerBomb`, `SuperMissile`
against `$0C18`'s bits 8-11 (0, 5, none, 1, 3, 2).

## Inputs

| Input | RAM or source | Decode |
| --- | --- | --- |
| the frame's requests | one list, in the game's order: Samus's current state handler (`$90:E695`) runs the inside handling (`$94:9B60`) then the projectiles (`$90:DCDD`); the enemies run next (`$82:8B58`..`$82:8B69`); her movement and pose updates in the new state handler (`$90:E731`, the pose-change looks `$91:EB88` from `$90:E745` after her moves); the enemy projectiles; then the PLM handler (`$82:8B71`). Other parts' spawns in each phase take their places there (plants `$9A26`, quicksand `$9A86` inside; quicksand and stations among the touches) | built by the bridge from each domain's report, in that order |
| inside | samus-motion's model: its inside report, which the original makes in Samus block inside handling (`$94:9B60`), the point in `$1E73` (`$94:9B72` bottom, `$94:9BA4` centre, `$94:9BD9` top); stand-ins followed (`$94:9B40` → `$9411`, `$9447`) | the point must be added to samus-motion's output (spec Header) |
| touch | samus-motion's model: `TouchedCells`, with the collision direction `$0B02` low nybble (0 left, 1 right, 2 up, 3 down) set by the move (`$94:8355`..`$94:8470`) and Fh by the wall-jump test (`$94:9683`) and the pose-change looks (`$94:96E9`) | `Look` is Fh |
| how she moves | dashing: speed boosting, `$0B3E & 0F00h = 0400h`; shinesparking: pose `$0A1C` in C9h..CEh; screw attacking: pose 81h or 82h | the three `bool`s, from the record until their domains exist |
| hit | W1's model, when it exists (the projectiles are handled in Samus's current state handler `$90:E6C0`, after the inside handling `$90:E6BC` and before her movement, which runs in the new state handler `$90:E731`, so hits come before her touches): the block shot reactions (`$94:A1B5`, `$94:A1D6`, through `$A175`/`$A195`), the bomb's five cells (`$94:9CF4`: the cell, above, right, left, below) and the power bomb's box edges (`$94:A06A`), each through Block bombed reaction `$94:A052`; S6's for the grapple (Block grapple reaction pointers `$94:A83B`: shootable `$9E55`/`$9E73`, bombable `$9FD6`/`$9FF4`, special air and block none) | the weapon from the projectile type `$0C18,x` bits 8-11 at `$0DDE`; `Grapple` from the grapple beam's collision (`$94:A85B`), which doesn't set `$0DDE` |
| another part's start | the other parts' models (until they exist, the harness copies their slots from N+1) | a place |
| the run paused | `$1C23` bit 15 clear at commit N+1: the PLM handler returns at once (`$84:85BA`); set and cleared by the room's loading and the door transition | `true` when clear |
| the room's placements | the room state's PLM population (bank `$8F`), spawned in order by Spawn room PLM (`$84:846A`) | `$B703` scroll (its list), `$B63B`/`$B63F` stand-in across with reach −1 (BTS FFh) and +1, `$B643`/`$B647` stand-in down with −1 and +1; any other ID another part's |

## The importer's mapping

The importer reads the ROM once and writes the reactions; it is the only code that knows these
layouts. `importer.py` is its observer-side version, for the integrator: at test time the
harness runs it on the player's ROM and gets the game's reactions in the spec's format (§2),
keyed by mark, as the model takes them. Nothing it writes is committed; the vectors' reactions
are invented (§The harness cases).

**Reactions, by PLM ID.** The family and the scripts' start come from the PLM's setup, which the
importer knows by address (these are code, so their content is the spec's arms; the importer only
names which family a setup is):

| Setup | Family | Whole look | First wait | PLMs |
| --- | --- | --- | --- | --- |
| `$CE37` | `Crumble` | `0BC` | 4 (`$84:CE5A`) | `$D044`..`$D060` |
| `$CDEA` | `Speed` | `0B6` | 1 | `$D030`..`$D040` |
| `$CE83` (touch), `$CEDA` (hit) | `Bomb` | `058` | 1 | `$D098`..`$D0B4`, `$D0B8`..`$D0D4` (same BTS, +20h) |
| `$CE6B` | `Shot` | `052` | 1 | `$D064`..`$D070` (they grow back) |
| `$B3C1` | `Shot` | none (keeps its look) | 1 | `$D074`..`$D080` |
| `$CF67` | `SuperMissile` | `09F` | 1 | `$D08C`, `$D090` |
| `$CF2E` | `PowerBomb` | `057` | 1 | `$D084`, `$D088` |

The `revealed` scripts: `$C91C` (power bomb blocks, set by `$84:CF60`), `$C922` (super missile
blocks, `$84:CF99`), and for the crumble and speed blocks the list of the bombed special PLM their
BTS names in `$94:9DA4` (`$CFFC`..`$D008` by shape, `$D024` for speed).

**Scripts, by instruction.** A list is read from the header's second word to its delete; a
`go to` is followed and its target inlined (the bomb blocks' `touched` lists jump into the shared
body), so a script never loops:

| Instruction | Step |
| --- | --- |
| `$8C7C`, `$8C10`, `$8C46` (queue sound, library 2, at most 1, 6, 3 queued), and the other libraries' | `Sound`: the sound by its library and number (`BlockCrumbles` 2:0Ah, "block crumbled"; `BlockBreaksOnContact` 2:06h, "block destroyed by contact damage"), the queue limit |
| a word below 8000h, then a draw instruction pointer | `Show`: the word is the frames; the draw instruction is the pattern |
| `$8B17`, `$8B05` (draw PLM block: the respawn block written back and drawn) | `Regrow` |
| `$86BC` (delete) | `End` |
| `$8724` (go to) | none: followed |

**Patterns, by draw instruction.** A header word (bit 15 a column, else a row; low byte the count),
that many level words, then either 0000h (the end) or a signed X and Y byte offset from the PLM's
block for the next header. Each level word gives a cell: its offset, its look (bits 0-11), and what
it becomes by its type: 0 `Air`; Bh, Ch, Fh `Itself`; 5 `StandInAcross`; Dh `StandInDown`. No list
of this spec draws another type.

**Cells.** The importer types each cell's mark at the room's load, by its block type, after
refusing a cell some asker of which reads past its table's end (§The importer's scan):

| Type | Mark from |
| --- | --- |
| Bh, 3 (special) | BTS < 50h: `$94:9139`; 80h..8Fh the area's table through `$94:92E9` (Brinstar 82h..85h speed blocks). `$B6FF` → `Scroll`; `$B62F`, `$B633`, `$B653`..`$B65B` → `Nothing` met as `Crumble`; a reaction's PLM → that reaction; any other PLM → `Other`. Special air with BTS 80h..8Fh also has an inside reaction, from the area's inside table (`$94:9B06`): `$B62F` or `$B653`..`$B65B` → she holds a place when inside it; any other PLM is another part's (plants `$B6CB`/`$B6CF`, quicksand `$B713`.., ice `$B70F`), which reaches this part as that part's request. Special air below 50h has no inside spawn but the scroll cell's (46h, centre only, `$94:9956`) and part 4's treadmills (8..Bh) |
| Ch, 4 (shootable) | `$94:9EA6` for BTS < 50h (0..Bh the shot, power bomb and super missile blocks; 40h..4Fh doors and gates, part 2; 45h items); Ch 80h..87h the area's table (`$94:9FC6`, all `$B62F`); `$B62F` → `Nothing` met as `Shot`; 4 with BTS ≥ 80h → `Inert` (`$94:9E5C`: no spawn, carry clear) |
| Fh, 7 (bombable) | BTS < 10h: `$94:936B` (touch) and `$94:A012` (hit), the same block's two PLMs, `$B62F` for 8..Fh → `Nothing` met as `Bomb`; BTS ≥ 80h → `Inert` (`$94:9348`, `$94:9FF4`: solid to her and her weapons, or air for type 7, with no spawn) |
| 5, Dh | the reach, BTS signed |

**Reactions.** The importer refuses a list with a `Show` of 0 or of 8000h and more frames (the
original waits 65,536 frames, or reads the word as an instruction), and a bomb block's broken list
whose first instruction isn't a queue sound (`$84:CF0C` skips three bytes for a bomb). None of the
game's lists does.

## The importer's scan

`scan_rooms.py` walks every room the doors reach from the landing site and from Ceres (261 rooms,
322 states), decompresses each state's level data, adds the scroll PLMs' cells, and for each
reacting cell (types 3, 4, 7, Bh, Ch, Fh) reads the reaction each asker gives it from the tables:
her touch (`$94:9139`/`$92E9`, `$936B`), her inside points (`$9966`, `$9B06`), a shot (`$A175`), a
blast (`$A032`) and the grapple (`$A83B`). 5,756 cells in 136 combinations of type, BTS and area.

- **Past a table's end:** no cell of the game. The importer refuses them (`docs/LEGACY.md`,
  "Reacting cells whose tables end before their mark"): special BTS 50h..7Fh (the touch table and
  the inside jump table have 80 entries; past them the touch reads the area tables and the inside
  jumps into the PLM table as code) and 90h and up (16 per area); a special block's BTS 10h..7Fh
  (the bombed table has 16: Brinstar's `D024` at 1Ah..1Dh, then pointers and code) and 88h and up
  (8 per area: Crateria 8Ah..8Dh read Brinstar's `D024`); shootable 50h..7Fh, and the shootable
  block's 88h and up; bombable 10h..7Fh (into `$938B`'s code and `$A032`).
- **Askers that disagree** in the game's rooms: 10 combinations, 2,964 cells, all special air whose
  touch and inside reactions differ, and none of them this part's to answer twice:
  - Wrecked Ship BTS 08h, 09h (370 cells, 5 rooms): touch `$B62F` (`Nothing` met as `Crumble`),
    inside the treadmills (part 4; no place);
  - Brinstar 80h, 81h (14 cells, 3 rooms): touch `$B633` (`Nothing`), inside the plants `$B6CB`,
    `$B6CF` (part 3; its start takes a place, as another part's request);
  - Maridia 80h..85h (2,580 cells): touch and inside both part 4's quicksand.
- **Inside a cell that holds a place:** no cell of the game (no special air at BTS 80h and up
  whose inside table gives `$B62F` or `$B653`..`$B65B`); the arm stands for a custom room.
  `Inert`: no cell of the game either.
- **Content:** no `Show` outside 1..7FFFh, every bomb block's broken list begins with a sound, and
  no reacting cell's patterns reach past its room's edge (1,866 reacting cells of this part's
  reactions over the states).
- **Scroll cells:** 265, each with its scroll PLM.

Reveal lists draw `Itself` on a cell whose type is already the family's, so a cell's class never
changes between its family's class and air or solid: the mark typed at load stays right.

**What asks a cell's PLM** (why `Nothing` carries a family): her touch spawns through the Samus
block collision reactions (`$94:94D5`/`$94:94F5`) for special and bombable cells, block and air,
never for shootable ones (solid by body, `$94:8F49`); a beam or missile through the shot reaction
pointers (`$94:A175`/`$94:A195`) for shootable and bombable cells, block and air, never for special
ones (`$94:9D5B`/`$94:9D59`); a bomb's or a power bomb's blast through the bombed reaction pointers
(`$94:A032`) for shootable and bombable cells and special blocks, never special air (`$94:9D59`);
the grapple through the grapple reaction pointers (`$94:A83B`) for shootable and bombable cells,
never special ones (`$94:A7C9`, `$94:A7CD`); her inside points through the inside reaction pointers
(`$94:9B40`) for special air only.
`$B62F` (setup `$84:B3CF`, an RTS) and `$B633` (`$84:B3D0`, CLC) both leave the carry clear and
delete themselves at the next pass (`$84:AAE3`), so they hold a place until it and answer air.
`$B637` (SEC) is unused.

## Sources per entry

| Entry | Source |
| --- | --- |
| Places | Spawn PLM (to current block index) `$84:84E7` (free slot from 4Eh down, `$84:84ED`..`$84:84F7`; none: return, carry unchanged); the setups that delete themselves (`STA $1C37,y` = 0 in `$CE37`, `$CDEA`, `$CE83`, `$CEDA`, `$CF2E`, `$CF67`, `$CFA0`) |
| TheScriptsRun | PLM handler `$84:85B4` (slots 4Eh down to 0; paused while `$1C23` bit 15 is clear, `$84:85BA`), Process PLM `$84:85DA` (timer decrement `$84:85E0`, instruction loop `$84:85EE`, draw `$84:85FD`..`$84:861A`), sleep and delete `$84:86B4`, `$84:86BC`; Game state 8 `$82:8B44` (the PLM handler after Samus's handlers and the projectiles, `$82:8B71`) |
| PatternCells* | Process PLM draw instruction `$84:861E` (row and column loops, `$84:8664` offsets); the BTS is never written by a draw |
| Regrowing | draw PLM block `$84:8B17`/`$84:8B05` (`$1E17` written back, timer 1, `$84:8B34`); the respawn block `[block] & F000h | tile` in each setup (`$84:CE49`, `$84:CE25`, `$84:CE72`, `$84:CEC8`, `$84:CF4F`, `$84:CF88`) |
| RoomStart | Spawn room PLM `$84:846A`; extension setups `$84:B33A`..`$84:B35B` (write block and BTS `$84:82B4`, delete `$84:B333`); scroll PLM setup `$84:B371` (`3046h`: special air, BTS 46h; skip the debug draw `$84:B366`) |
| ScrollCells | special air collision `$94:906F`/`$94:909D` and special block collision `$94:90CB`/`$94:9102` (BTS 46h → `$B6FF`); inside reaction `$94:9956` (centre only); scroll block touch setup `$84:B393` (search by block index `$84:B39D`, triggered flag `$84:B3A8`, `$1D27` += 2, timer 1; the hang `$84:B3A6`); scroll PLM list `$84:AF86`, process air scroll update `$84:8B55` |
| CrumbleBlocks | special block collision `$94:90CB`/`$94:9102` → `$94:9139` BTS 0..7; setup `$84:CE37` (`$0B02` = 3) |
| SpeedBlocks | `$94:9139` BTS Eh/Fh, Brinstar 82h..85h (`$94:91F9`); setup `$84:CDEA` |
| BombBlocks | bombable collision `$94:932D`/`$94:934C`, `$94:92F9`/`$94:9313` → `$94:936B`, setup `$84:CE83`; bombable shot/bombed/grappled reaction `$94:9FF4`/`$94:9FD6` → `$94:A012`, setup `$84:CEDA` (bomb: `$1D27` += 3 past the list's sound, `$84:CF0C`; the grapple reads a stale `$0DDE`, LEGACY) |
| ShotBlocks | shootable shot/bombed/grappled reaction `$94:9E73`/`$94:9E55` → `$94:9EA6` BTS 0..7; setups `$84:B3C1`, `$84:CE6B` (no weapon test, so the grapple breaks them too, `$94:A83B`) |
| SuperMissileBlocks | `$94:9EA6` BTS Ah/Bh; setup `$84:CF67` |
| PowerBombBlocks | `$94:9EA6` BTS 8/9; setup `$84:CF2E` |
| BombsReveal | Block bombed reaction pointers `$94:A032` (special block → `$94:9D71`, `$94:9DA4`); setup `$84:CFA0`; the lists `$84:C8EC`..`$84:C8FE`, `$84:C928` |
| BlocksMarkedNothing | `$B62F`, `$B633`, `$B653`..`$B65B`, `$84:B3CF`, `$84:B3D0`, `$84:AAE3`; the reaction pointers above (§The importer's mapping, what asks a cell's PLM); the PLM tables' `$B62F` entries (`$94:9139`, `$94:9DA4`, `$94:9EA6`, `$94:936B`, `$94:A012`); inside, special air `$94:9B16` (BTS 80h and up, the area's inside table `$94:9A06`..`$9AE6`) at each point (`$94:9B72`, `$9BA4`, `$9BD9`); `Inert`, `$94:9E5C`, `$94:9348`, `$94:9FF4` |
| AnswersToHerMotion | the carries of the setups above and of the collision routines (`$94:90E1`, `$94:9343`, `$94:906F`'s `CLC`, `$94:92F9`'s `CLC`), Samus block collision reaction pointers `$94:94D5`/`$94:94F5`; `$B62F`'s setup `$84:B3CF` (an RTS: carry from `$84:8500`'s `ASL`, clear) |
| FullPlacesLetHerThrough | `$84:84F9` returns without touching the carry, which the `ASL` before the spawn cleared (`$94:90D8`, `$94:9339`), so `$94:90E1`'s `BCC` lets her through |
| WhatTheScreenRedraws | Draw PLM `$84:8DAA` from `$84:861A` and `$84:8B50` only; the setups write level data without drawing |
| Sounds | the queue sound instructions `$84:8C10`, `$84:8C46`, `$84:8C7C` (`$80:90CB`, `$80:90B7`, `$80:90C1`) |

`scan.sh --mirror` finds 1 of 18 entries (5 %) single-routine, `PatternCells`, marked *: its
content is the draw routine's, which the format names by what each cell becomes.

## Findings behind the spec's choices

- **The places' numbers** (for the adversary). The spec numbers the places 0 to 39 and starts a
  reaction in the highest free one because that is what can be seen: 40 at once and no more
  (`FullPlacesLetHerThrough`), and the run's order, which orders the redraws (the tile encoder's
  queue) and decides between two scroll cells naming one screen (C-scroll-order). The number is the
  slot index halved, not the slot's byte offset, and nothing in the spec indexes by it.
- **Lag cuts.** In chain-0232's power bomb blast the game lags every other frame: the PLM pass of
  a frame runs in the interval of a record whose NMI is a lag NMI (`$05B6` unchanged), and the
  frame commits at the next record. The harness pairs commits (records where `$05B6` moved), with
  the lag records' cell changes folded in. Pairing records by `$05B6` + 1 alone compares a
  mid-frame State and fails 14 frames.
- **Full places.** With no free slot `$84:84E7` returns with the carry as the caller left it. Every
  special and bombable block caller has just done `ASL` on the BTS (high byte 0), so the carry is
  clear and the block is air to her. Constructed only; no record fills the places.
- **The bomb's silent break.** `$CEDA` skips the list's first three bytes for a bomb and not for a
  power bomb; every bomb block's list begins with a three-byte queue sound, so the spec says it by
  behaviour, "without its leading `Sound` step, if it has one", and the importer refuses a list
  that doesn't begin with one. A separate `bombed` script would add a script name the importer
  writes; the rule adds none.
- **The grapple.** The grapple beam's block collision asks shootable and bombable cells by the same
  routines as a shot (`$94:A83B`), so a shot block breaks under it (`$CE6B`, `$B3C1` test no
  weapon) and a cell marked `Nothing` met as `Shot` or `Bomb` holds a place. The bomb, super
  missile and power bomb setups read `$0DDE`, the last projectile handled, which a grapple hit
  doesn't set (`docs/LEGACY.md`). Special cells don't answer it.
- **A scroll touch on a paused frame.** `$84:B393` sets the trigger and moves the list past its
  sleep at the touch; the update runs only in the next pass, which a paused frame skips, so the
  touch is State there (`C-scroll-paused`, `C-scroll-after-pause`). Each further touch still
  spawns `$B6FF`, a hold until the run. No paused frame of the 44 rows touches one.
- **Inside cells.** `$94:9B16` spawns from the area's inside table for special air with BTS 80h and
  up at each point she is inside, before her weapons and her touches, so a hold there pushes a
  crumble touched in the same frame one place down. The game's rooms have none that holds a place
  (§The importer's scan); the plants and the quicksand there are parts 3's and 4's.
- **The regrown look.** Every setup stores the respawn block as the block's type with a fixed tile
  and no flips. r04's bridge (tile 16Ah) grows back as 0BCh (r04:110, 159): hidden crumble blocks show
  themselves after one cycle.
- **The crumble's first wait.** `$CE37` sets the timer to 4 at the spawn, before the frame's pass,
  so the first step runs on the third frame after the touch (r04:110 → 113).
- **The look's touches.** The wall-jump test and the pose-change looks set the direction Fh, so the
  crumble (and the stations, quicksand: other parts) refuse them, while the speed block, the bomb
  block and the scroll trigger don't test the direction.
- **Two touches of one scroll cell in a frame.** The flag test at `$84:B3A8` is what stops a second
  `$1D27` += 2, which would skip the update; so the spec states "touched" as a state, changed once.
- **The scroll cell's re-write.** `$8B55` rewrites the block's type as special air, which changes
  nothing in vanilla (the cell never stops being special air); not an arm.
- **Reactions paused.** While `$1C23` is clear the pass doesn't run, but touches still spawn: in
  terrain-quicksand 420-422 the quicksand's touch PLMs (part 4) pile up in 39 of the 40 slots, at
  423-455 the hits show three spawns a frame refused (`$84:84F9`), and the first pass after deletes
  them all. The spec states it as an input, the run paused (`TheScriptsRun`); who pauses it, and
  when, is the room's start's and the doors' (R1, R2). No paused frame of the 44 rows has a
  reaction of this part running, so the arm is checked by C-run-paused alone.
- **Weapons before her moves.** The hits put a frame's weapon starts before her touches: in
  item-charge-beam 121 the power bomb's blast starts the bomb block (`$84:CEDA`, line 70) before her
  move touches a scroll cell (`$84:B393`, line 142), so the bomb block takes the higher place
  (item-charge-beam:122). The game handles projectiles in Samus's current state handler (`$90:E6C0`) and moves her
  in the new state handler (`$90:E731`), after the enemies (`$82:8B58`..`$82:8B69`). The first
  version of the spec had her touches first.
- **A scroll touch every frame.** The trigger `$84:B3AD` fires on every frame she touches a scroll
  cell (area-brinstar 92 to 123, one or two a frame), not only when the screens change: the update
  rewrites screens that already have their kinds. A second touch of a touched reaction in the frame
  runs `$84:B393` and skips the trigger (79 times).
- **A scroll cell with no scroll PLM.** The touch's search for the scroll PLM on its block finds
  none and loops for ever (`$84:B3A6`). No room of the game has one (§The importer's scan), so the
  spec leaves it to the room's loading, which refuses such a room (`docs/LEGACY.md`, "A scroll
  cell with no scroll reaction").
- **Patterns past the room's edge** carry onto the next or the previous row (the draw adds 2 per
  cell to the block index), or write outside the room past its top or bottom; no block of the game
  sits there (§The importer's scan). The spec states patterns in cells, and the room's loading
  refuses such a room (`docs/LEGACY.md`, "Patterns past the room's edge").
- **A `Show` of 0 frames** waits 65,536 frames (`$84:85E0` decrements first); the spec allows 1 to
  32,767 and the importer refuses the rest.

## The harness (for the integrator)

- **Frames:** every pair of consecutive commits (records where `$05B6` moved since the record
  before) of every row, with the cell changes of the lag records between them. Skipped, by name and
  never by outcome: game state other than 8 at either (`$0998`); `$05B6` not advanced by exactly
  one; a room change (`$079B`, or the level reloaded). A frame with the PLMs disabled at either
  commit (`$1C23` bit 15 clear) is run with the run paused when it is clear at N+1.
  Counted as a harness defect: a pair that doesn't decode, a scroll touch that finds no scroll PLM.
- **Decoded from commit N:** the State (§State): every occupied slot, the 2x2 cells at each slot's
  block, the screens.
- **Inputs:** one list of requests in the game's order (§Inputs): the cells she is inside, then
  her weapons' hits, then her touches, from samus-motion's model on the same frame (as the camera's
  harness takes `camera_from`) and W1's; her three `bool`s from N+1 (screw attacking: pose 81h or
  82h only); the run paused from `$1C23` at N+1. The other parts' starts aren't requests until
  their models exist: their slots are copied from N+1 (the masks below). Her weapons' hits from W1's model when it exists. Until then, and in this note's
  numbers, the observer's harness decides them from the watched instructions (`harness_hits.py`):
  each setup hit between the commits is matched, by family, to the slot that holds a new reaction
  of this part at N+1, which gives the start's cell (a crumble's touch, `Down`; a shot block's hit,
  a beam; a power bomb block's, a power bomb; a super missile block's, a super missile; a bomb
  block's, the weapon its branch names); a setup with no new slot is a refused start; each
  `$84:B393` is a scroll touch, the trigger after it says it is the reaction's first that frame,
  and its cell is the scroll reaction nearest her not yet touched; `$94:995E` before it makes it a
  centre's. A start no hit decides would be reported apart as unverified; over the 44 rows there
  is none. The cache's `hits.txt` carry the watch list of the scenes lane; the integrator needs
  `watch.txt`'s added to the cache recording (`--watch-pcs`) to decide the starts as here.
- **Reactions:** the game's, from the importer at test time (§The importer's mapping), never the
  vectors' invented ones. The harness cases (§The harness cases) are frames it must match.
- **Compared at N+1:** each place (empty, another part's, or its reaction, script, cell, next step
  and wait; what it grows back as where a `Regrow` is ahead), the cells' kinds and looks, the marks
  of reacting cells and stand-ins, the screens.
- **Masks (fields, not frames):** the other parts' slots are held: their contents aren't compared,
  and a slot another part's reaction takes or frees between N and N+1 is copied from N+1
  (unverified), so the free slot a start takes is chosen as the original chose it. The order of a
  frame's starts among the other parts' spawns isn't known from the record; one frame of the 786
  (r06:538) has another part's slot change beside this spec's reactions, and matches.
- **Null model:** nothing changes: 183 of the 786 (the scroll touches already in force); 431 on
  cells and screens.
- **Per-behaviour attribution:** `model.py`'s tags name what each frame did; `gen.py`'s `ENTRIES`
  maps them to the spec's entries. Oracle-checked by these 44 rows: `Places`, `TheScriptsRun`
  (but its paused arm), `PatternCells`, `Regrowing`, `ScrollCells`, `CrumbleBlocks` (its start),
  `ShotBlocks`, `SuperMissileBlocks` (the super missile's start), `PowerBombBlocks` (the power
  bomb's start), `BombBlocks` (the blasts' starts, and with no vector the refusals); new since the
  32 rows: `SuperMissileBlocks` and `BombBlocks`. Vectors only: `SpeedBlocks`, `BombsReveal` (its
  start; its refusal is on record with no vector), `BlocksMarkedNothing`,
  `FullPlacesLetHerThrough`, `RoomStart`, `AnswersToHerMotion`, the grapple's arms, the paused
  scroll touch, the outputs.
- **Rows to cache** for the vectors-only entries: chain rows of the 100% route through the speed
  blocks of Crateria's and Brinstar's dash rooms, the bomb blocks of the Morph Ball's way out
  (screw attacked or dashed), hidden crumble blocks a bomb shows; the scenario lock's
  `special-beam-*`, `power-bomb-*` rows.
- **Free run:** from a seed commit, carry the cells and places forward, take only the inputs from
  the records (or the other domains' models); report the first differing frame per place and per
  cell. Re-seed at a room change.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here, and `tools report` the areas' old
figures and the commit.

Size is counted in logical tokens: each identifier, keyword, literal and operator is one, and so is
each bracket pair; comments, layout and tests are free. Two kinds of token are counted apart and
reported, not held against the target: the values of constants and statics (tables of the game's
data), and declarations (the `struct`, `enum` and `type` items and attributes that define the
model's shapes; define as many as the behaviour needs). The scripts are imported assets (§2): they
are data the importer writes, not code. The target is on everything else, the logic.

**Budget (MODEL §1 decision 7).** k = 36 weighted tokens per spec unit, frozen on samus-motion. An
area's budget is k × its units, capped at 70% of its old code (`old`, the `tools size` score of the
old code that area replaces, at spine `387640758`). The old code covers the places and the scroll
cells, but of the blocks that break only the start of one shot block (175.5), so breakables has no
cap: as the plan proposes for domains without old code, its budget is k × units alone. The budget
is an indicator: the fit is score ÷ budget, and being over it costs in proportion to how far over,
never as a wall; over an area's old code is a failure.

- Area places (old 1,252): 2, Places, TheScriptsRun, PatternCells, Regrowing, RoomStart,
  WhatTheScreenRedraws, Sounds
- Area scroll (old 1,145): ScrollCells
- Area breakables: CrumbleBlocks, SpeedBlocks, BombBlocks, ShotBlocks, SuperMissileBlocks,
  PowerBombBlocks, BombsReveal, AnswersToHerMotion, BlocksMarkedNothing, FullPlacesLetHerThrough

Budget: 36 × 54.3 units = 1,956 (places 804, scroll 144, breakables 1,008)

### The old code (tokens, 2026-11-08)

`tools size` (the wire-cam counter, `$TEMP/collab/wire-cam/size.dll`), over cuts of the old code at
spine `387640758` in `$TEMP/collab/blocks-spec/old` (`cut.py`), measured together so the clumps
are counted across them:

| Area | Old code | Score | Logic | Types | Data | Lines |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| places | `plm.rs`: `NO_PLMS`, `RoomPlms`, `slot_index`, `free_slot`, `fill`, `write_block`, `extension_block`, `spawn` (less its item arm), `spawn_population`, `process`, `run_below` (less its item arms); `plm/crumble.rs`: `Instruction`, `crumble_tile`, `LIST`, `next_instruction`, `place`, `run` | 1,251.5 | 1,150 | 64 | 0 | |
| scroll | `plm.rs`: `SCROLL_TRIGGER`, `TouchFault`, `slot_on`, `touch_scroll_block`, `update_scroll`; `collision.rs`: `Touch`, `Touches` and its impls; `collision/inside.rs`: all | 1,145 | 1,122 | 116 | 4 | |
| breakables | `plm/crumble.rs`: `SHOT_BLOCK_1X1`, `TYPE_KEPT`, `ShotFault`, `shoot_block` | 175.5 | 169 | 32 | 6 | |
| total | | 2,572 | 2,441 | 212 | 10 | 405 |

Lines by `$TEMP/collab/scripts/code_lines.py` over the three cuts. Not counted: `plm/draw.rs`
(drawing PLMs into the tilemap, E2's), `plm/item_gfx.rs` and the item paths (W3's),
`collision.rs`'s `Room`, `ScanOrder` and `Surroundings` (samus-motion's bridge), and
`samus_world.rs`'s mapping of the clean touches (the bridge's). The areas below state places' old
code as 1,252 and scroll's as 1,145; breakables has no cap.

**The ceiling** (decode-only; the plan's proposal for domains without old code): the whole-game
port at `archive/old-engine` holds this part's mechanism in `room_mechanisms/scheduler.rs` (1,161),
`commands.rs` (4,739), `creation.rs` (1,817), `rules/scroll_setup.rs` (482),
`projectile_block_reactions.rs` (1,717) and `player_collision/inside.rs` (914): 10,830, shared with
the other three parts (`$TEMP/collab/reviews/sz_old.txt`).

**k out of sample:** the old places and scroll score 2,396.5 over their 26.3 units, 91.1 a unit
(samus-motion's 65.8, the camera's 53.7); 0.55 of it would give 50.1 against the frozen 36, so
the caps don't bind. The budget is 36 × 54.3 = 1,956 (places 804, §2's four cell terms included;
scroll 144; breakables 1,008).
