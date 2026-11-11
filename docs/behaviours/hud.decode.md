# The HUD and the mini-map: decode notes

For the observer, the adversary and the domain harness's decoder only. Implementers never see this
file (MODEL §5). It holds every source the spec draws on: the original's routines, its RAM and its
codes. Addresses are the bank logs' (`$TEMP/banklogs/bank_*.txt`,
https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 32 rows cached in `$TEMP/collab/more-caches/cache` were dumped per frame by
`$TEMP/collab/hud-spec/dump` (a cdylib over `oracle_record`, run by `dumprun.py`; output in
`$TEMP/collab/hud-spec/out`): the HUD's WRAM tilemap and the explored bitmap whenever they change,
the HUD's VRAM rows, the words of §State and §Inputs, and the APU port 1 writes. The bank logs were
then read to name the cases and pin the arithmetic. The observer's reference model, `model.py` in
the same directory, runs a frame in the spec's terms (cells, map cells, items); `gen.py` turns each
record pair into a vector of the spec's format and runs the model over it.

Over the 14,947 record pairs that ran one frame of gameplay in one room (game state 8 at both,
`05B6` advanced by one, the same room `079B`), with the 20 pairs where a count fell left out (the
first rounds' shot mask; `why_shot.py` reports them apart, below the table):

| Part | Frames | Model | Null model |
| --- | ---: | ---: | ---: |
| status bar (every cell but the window's, the values last shown) | 14,927 | 14,927 | 14,328 |
| selection (`09D2`, `0A04`) | 14,927 | 14,927 | 14,850 |
| mini-map (the window's 15 cells, the explored bitmap), pose unchanged | 11,900 | 11,900 | 10,516 |
| mini-map, pose changed | 3,027 | 3,019 | 2,673 |
| all of the HUD, pose unchanged | 11,900 | 11,900 | 10,077 |

The 8 pose-change frames it misses are turnarounds and pose transitions (chain-0012 769,
chain-0057 65, chain-0059 263, chain-0097 155, 429, 518, samus-grapple-walljump 53,
samus-screw-attack 524) where bank `$91`'s pose change moved her across a screen boundary after the
mini-map read her (`$90:E734` runs before `$91:EB88`); with record N's position all 8 match. The
other 3,019 pose-change frames are only consistent with the model, not proof of it: the mask covers
them because it is keyed on `0A1C`, not on the writer (§Harness), so report them as masked.

**The 20 count-fell pairs** (`why_shot.py`): in none of them did the weapons cancel the selection
(chain-0059 431 and chain-0100 7 change it by Select; the other 18 keep it), so the shot's cancel
is false on all 20. Run with it false, the model agrees on all 20: status bar 20, selection 20,
mini-map 20 (11 of them pose-change frames). A count also falls without a shot (a crystal flash),
and a shot that doesn't cancel still fell under the old mask: the count is a proxy, which is why
§Harness keys the cancel on its writers.

`mm_bytes.py` is the mini-map's update byte for byte (`$90:A91B`, `$90:AA43`, `$90:AB75`). On
4,000 random states at every column the window can start in (`xcheck.py`) it and the spec's cell
rules (`mm_cells.py`) agree on every cell and every explored bit, and `xcheck_vectors.py` runs every
vector with a mini-map (36 of them, the four `BossBlanksTheMap` ones aside) through it: 34 agree.
The two that differ are spec §1.5's choices, C-mm-first-columns-row-1 and C-mm-her-cell-outside
(§Findings, the map's outer edge); C-mm-last-row and C-mm-last-columns-row-30 agree only because
`mm_bytes.py` pads the bitmap with zeroes. No record reaches the map's first or last two columns,
so `FirstColumnsShift` and `LastColumnsShift` rest on that cross-check.

`vectors.py` picked the recorded frames (at most two per outcome tag, from different rows, from
frames the model matches, outside the masks), which are now the harness cases (§The harness cases),
and `cons.py` wrote the constructed vectors. Round 3's files are in `$TEMP/collab/hud-spec-r3`:
`model.py` is the reference model over a picture set read from the vector file, `convert.py` and
`build.py` wrote the current `scenarios/vectors/hud.toml` (the invented pictures, the twins, the
named bosses), and `migrate.py` (seed 7) then gave every picture a fresh number and made the
mini-map's off a State (§Adversary 79's fixes), and `vecrun.py` runs the 72 vectors: 72 of 72 agree.
`real_content.py` checks the file for the game's content (§The harness cases, §Adversary 79's
fixes). Round 4's are in `$TEMP/collab/hud-r4`: `scripts/edit_vectors.py` wrote the file's
column-first keys and its three new vectors (§Adversary 83's observer items), `model.py` reads the
keys column first and `vecrun.py` ran the 75 vectors: 75 of 75 agree (82 of 82 after §Adversary 92's
fixes). `mutants.py` runs eight mutants of `model.py`, each killed (the three new ones by one new
vector each, and by no vector of round 3's file). `xcheck4.py` runs the 39 vectors with a mini-map
update through `mm_bytes.py`, the file's invented map blank and diagonal in place of the vanilla
ones: 37 agree, and the two that differ are spec §1.5's choices, as above. `real_content.py` there
reads the HUD's keys column first: 0 hits. `scripts/ovr.rs` and `scripts/join.rs` (cdylibs over
`verify`, run by `scripts/run.py`) count the overrun records of cache-v2 (§Harness).

**Row names.** The chain rows go by opaque ids, as in the camera's notes, and so does one row
whose name is a lexicon match:

| Id | Row |
| --- | --- |
| r01 | chain-0012-91F8 |
| r02 | chain-0013-92FD |
| r03 | chain-0014-96BA |
| r04 | chain-0057-9FBA |
| r05 | chain-0059-A293 |
| r06 | chain-0097-ACF0 |
| r07 | chain-0100-B07A |
| r08 | chain-0106-AD5E |
| r09 | chain-0220-D017 |
| r10 | chain-0224-D0B9 |
| r11 | chain-0232-D5A7 |
| r12 | samus-square-slope-reserve |

## State

| Quantity | RAM | Decode |
| --- | --- | --- |
| the HUD's cells | `$7E:C608`..`C6C7`, 3 rows of 32 words (BG3 tilemap rows 1 to 3; the top row, VRAM `$5800`, is uploaded only by the HUD's initialisation `$80:9A79` and isn't the domain's) | each word: picture bits 0-9, colour set bits 10-12, h-flip bit 14, v-flip bit 15; bit 13 (priority) is set in every word the HUD writes and in every recorded word: a word without it is a harness defect |
| the values last shown | `0A06` energy, `0A08` missiles, `0A0A` super missiles, `0A0C` power bombs, `0A0E` the item | counts as is; the item as below |
| the selected item | `09D2` (HUD item index) | 0 none, 1 missiles, 2 super missiles, 3 power bombs, 4 grapple, 5 x-ray |
| the one-shot item | `0A04` (auto-cancel HUD item index) | the selection is the one-shot item when `0A04` = `09D2` ≠ 0; not when `0A04` = 0; any other value is a harness defect. The original keeps `0A04` 0 or equal to `09D2`: Select writes both (`$90:C50A`..`$90:C516`), item cancel and `$90:BF25`/`$90:C091` clear both, `$90:BF41`/`$90:C0A6` clear `09D2` only when `0A04` is 0 |
| the mini-map is off | `05F7` of record N | ≠ 0. Set only by `$90:A7E2` (the bosses; the cached samus-ceres-push starts with it set), cleared only by the writers of `the mini-map turned on` (§Inputs): this domain's own memory, so State, read by the mini-map on later frames |
| the explored cells | `$7E:07F7`..`08F6` (the current area's map tiles explored) | bit `7 − c mod 8` of byte `page × 80h + y × 4 + (c mod 32) div 8` is cell `(page × 32 + c mod 32, y)`; the map's row 0 is the bitmap's row 0 (rooms start at row 1: `07A3 + 1`) |

**RAM shape** (fields decoded as one word copied as is): the HUD's cells, one word each. A cell is
a picture, its colour set and its flips, which is what the player sees; the word's fields are
those, with the priority bit, which is constant, dropped. The counts last shown are counts.

**The item order.** The spec's item order (none, missiles, super missiles, power bombs, grapple,
x-ray) is `09D2`'s code order. It is kept because it is behaviour the player sees: Select cycles in
it and it is the icons' left-to-right order on the HUD (§1.1). An implementer may declare the items
in any order; the cycling's order is the spec's data.

Not State: `077C` (the palette bits the highlight routine is passed, written and read inside
`$80:9CEA`); `060B` (the bit index of her map cell, written at `$90:AA4F` and read back at
`$90:AA56`..`$90:AA6A` in the same update; `$86:EBDD` and `$A0:9577` use the word for other things);
the scratch `$12`..`$34`.

## Inputs

| Input | RAM | Decode |
| --- | --- | --- |
| frame count | `05B5` low byte of record N (the 8-bit counter the NMI increments when the frame finished, `$80:95ED`; zeroed at a door) | as is; the HUD reads it during frame N+1's main processing, before NMI N+1 increments it |
| energy | `09C2`, `09C4` of N+1 | as is; damage lands before the HUD's turn (`$A0:9785`, `$A0:A306`); the running-out-of-health check (`$82:DB69`) after it |
| reserve | `09D6`; `09C0` (reserve health mode) | `auto` when `09C0` = 1 (0 not obtained, 2 manual) |
| missiles, supers, power bombs | `09C6`/`09C8`, `09CA`/`09CC`, `09CE`/`09D0` of N+1 | count, capacity |
| equipment | `09A2` (equipped items) | bit 4000h grapple, 8000h x-ray |
| item select, item cancel | `8F` (newly pressed) and `8B` (held) of record N, against the bindings `09BA` (item select) and `09B8` (item cancel) | `Pressed` when `8F` has the binding; cancel `Held` when only `8B` has it |
| can switch | `0A42` (Samus current state handler) and `0A1C` (pose) at N+1 | `0A42` = `$E695` (normal; the demo handler `$E6C9` belongs to the demos) and `0A1C` ∉ {0, 9Bh} (facing forward, `$90:DCE0`) |
| the shot's cancel | the weapons' stores `$90:BF25`, `$90:BF41`, `$90:C091`, `$90:C0A6` | true when one of them is hit in frame N+1 (an instruction hit, as `collected` is named); cache-v2 watches all four on every row |
| why no click | `0A1F` low byte, `0D32`, `0A78` at N+1 | 3 spin jumping, 14h wall jumping; `0D32` ≠ `$C4F0` the grapple beam is out; `0A78` ≠ 0 time stopped |
| collected | the item PLM instructions | a hit at `$80:99CF`, `$80:9A0E`, `$80:9A1E`, `$80:9A2E` or `$80:9A3E` in frame N+1 names it; cache-v2 watches all five on every row |
| the mini-map runs | `0A44` (Samus new state handler) at N+1 | runs for `$E725` (normal), `$E7F5` (title demo, `$90:E801`), `$E86A` (appearance), `$E8DC` (locked), `$E8EC` (elevator), `$E8AA` (Ceres, which calls normal); not for `$E7D2` (debug), `$E833` (intro demo: the spec's "while the game's opening shows her"), `$E8D6` (station), `$E8D9` (drained), `$E902` (gunship). The title demos themselves are the demos' domain |
| the mini-map turned on | the stores of 0 to `05F7` | true when one is hit in frame N+1; the writers: `$80:C4AF` (a load at a station), `$82:8767` (the demos' load), `$82:E1C4` (a door's start, before the new room's enemies load at `$82:E4D4`, so a boss of that room turns it off after), `$88:82F3` (the new room's effects setup), `$90:F233` (Samus command 8, called only in game states 6, 1Fh and 28h, before state 7: part of a new game or a load), `$A4:90C4` (Crocomire's skeleton flowing down the river). Only the last, `$A4:90C4`, can land in a compared pair; the others are a door's, a load's or a new game's, outside gameplay or in skipped pairs |
| a boss encountered | a call of `$90:A7E2` outside game state 8, with the boss `179C` (3 Kraid, 5 Ridley in Norfair, 6 Crocomire, 7 Phantoon, 8 Draygon; 1 Ridley in Ceres) | the calls: Crocomire `$A4:8A7F` (in its initialisation AI `$A4:8A5A`), Draygon `$A5:86F7` (`$A5:8687`), Ridley `$A6:A122` (`$A6:A0F5`, before its Ceres/Norfair branch, so Ceres Ridley too), Kraid `$A7:AA7C` (`$A7:A959`), Phantoon `$A7:CE24` (`$A7:CDF3`); these five run from Load enemies `$A0:8A1E`, called in the door transition (`$82:E4D4`) and at game load (`$80:A0CA`), never on a game-state-8 frame, so no gameplay pair can show them: the door and load domains supply the event, as a step outside a frame. Only while the boss lives: Kraid's initialisation returns at `$A7:A9DB` when he is dead (the test at `$A7:A976`), Crocomire's branches past the call when the area's mini-boss is dead (`$A4:8A7A`), Ridley's returns at `$A6:A10D` when the area's boss is dead (`$A6:A0F5`..`A0FF`); Draygon's and Phantoon's have no test, but their rooms (`$8F:DA60`, `$8F:CD13`) take the state for their boss dead (`$8F:DA8C`, `$8F:CD3F`, condition `$E629`), whose enemy population is empty (`roomstates.py` reads the ROM's room states), so neither runs. Ceres Ridley (`$A0:E13F`) shares Ridley's initialisation with boss id 1, which the table lacks: the map blanks and nothing is explored; Norfair Ridley (`$A0:E17F`) is id 5. Ceres's room state for Ridley dead (`$8F:E0E1`) has no `$E13F`. The screens from the table at `$90:A83A` (boss ids 3 Kraid, 5 Ridley, 6 Crocomire, 7 Phantoon, 8 Draygon, Ah Mother Brain); boss id 1 isn't in it, so Ceres Ridley's screens are none |
| Mother Brain falls | a call of `$90:A7E2` in frame N+1 with `179C` = Ah | Mother Brain's `$A9:8809` is in her first phase's body function `$A9:87E1`, in gameplay: it runs once her glass is destroyed (`$80:8233` with 2), Samus's x is below ECh (`$A9:87F1`) and her brain's health is 0 (`$A9:87F9`), with the form set to fake death (`$A9:8805`), the music changed and her wall sealed: her first form falls, and the bosses' domain supplies the event |
| the HUD's pictures | the importer (§The HUD's pictures) | by name; the clean model holds none |
| her position | `0AF6`, `0AFA` of N+1 | pixels; masked on pose-change frames (§Harness) |
| her room | `07A1`, `07A3` (room map X, Y); `07A5`, `07A7` (width, height in blocks) | cell `[07A1, 07A3 + 1]` |
| her map | `079F` (area); the tilemap at the long pointer `$82:964A + area × 3` (800h words: the left page's 32 rows of 32 words, then the right page's); the charted bitmap at `$82:9717 + area × 2` (bank `$82`, 100h bytes, laid out as the explored bitmap); `0789` (current area's map collected) | a picture is a word's bits 0-9, 14 and 15 (its colour bits are replaced); a cell `(c, y)` is word `page × 400h + y × 20h + c mod 32` |
| the beyond pictures | the 800h words after the area's tilemap | in ROM the area maps are consecutive 1000h-byte blocks of bank `$B5` (Brinstar `$8000`, Crateria `$9000`, Norfair `$A000`, Wrecked Ship `$B000`, Maridia `$C000`, Tourian `$D000`, Ceres `$E000`), so the beyond pictures are the left page of the block after: Crateria's are Norfair's, Brinstar's Crateria's, Ceres's whatever `$B5:F000` holds. Beyond `(c, y)` is word `800h + y × 20h + c` |

## Sources per entry

| Entry | Source |
| --- | --- |
| EnergyGauge | `$80:9B8B`..`$80:9BF8`: the compare with `0A06`, the divisions by 100 (`$4204`/`$4206`), the tank loop over `$80:9CCE` (14 offsets) with `$80:9BD3` (empty 3430h) and `$80:9BDC` (full 2831h), the two digits `$80:9D98` from the table `$80:9DBF` |
| AmmoCounts | `$80:9BFB`..`$80:9C52` (capacity tests `09C8`, `09CC`, `09D0`; compares with `0A08`..`0A0C`); `$80:9D78` (three digits), `$80:9D98` (two), the table `$80:9DD3`; the debug three-digit super missiles (`05CF` & 1F40h, `$80:9C2C`) never in vanilla |
| AutoReserveSign | `$80:9B4E`..`$80:9B87` (`09C0` = 1; `$80:998B` lit, `$80:9997` empty) |
| SelectionHighlight | `$80:9C55`..`$80:9C6F` (highlight `09D2` with 1000h, unhighlight `0A0E` with 1400h); `$80:9CEA` (the six cell tests against 2C0Fh, `AND #$E3FF : ORA`, missiles three wide by `$80:9D41`); offsets `$80:9D6E` |
| SwitchClick | `$80:9C72`..`$80:9C92` (sound 39h, library 1, `$80:9049`) |
| OneShotFlash | `$80:9C96`..`$80:9CA7` (`05B5` & 10h) |
| IconsAppear | `$80:99CF` (missiles, tests `C61C` & 3FFh = 0Fh), `$80:9A0E`, `$80:9A1E`, `$80:9A2E`, `$80:9A3E` through `$80:9A4C` (the same test on its first cell); icons `$80:99A3`..`$80:99C7`; callers `$84:89A9`, `$84:89D2`, `$84:89FB`, `$84:891A`, `$84:8941` |
| CyclingItems | `$90:C4B5`..`$90:C519` (`$90:C4D9` select, the loop `$90:C4EF`..`$90:C508` through the handlers `$90:C539`: `$90:C551`, `$90:C564`, `$90:C577` count = 0, `$90:C58A`, `$90:C5AE` not equipped); called from `$90:DCED` when the pose isn't facing forward (`$90:DCE0`), from the normal current state handler `$90:E6C0` |
| CancellingTheItem | `$90:C4BD`..`$90:C4C7` (newly pressed cancel: `0A04` = 0, then `$90:C4E9`); the weapons' cancel `$90:BF25`, `$90:BF41`, `$90:C091`, `$90:C0A6` |
| OneShotItem* | `$90:C4C9`..`$90:C4D7` (`$16` = holding cancel), `$90:C50A`..`$90:C516` |
| Switching | the switched-to handlers `$90:C545`..`$90:C5C3` (clear the charge, reload the suit palette, the grapple's function); the arm cannon flag `0AAA` at `$90:C519`..`$90:C534` |
| Exploring* | `$90:A943`..`$90:A984` |
| MiniMapWindow | `$90:A98D`..`$90:AA41` (the window's bits), `$90:AA43`..`$90:AB46` (the tiles, the three rows, palette 3 or 2, `0789`) |
| HerBlink* | `$90:AB4A`..`$90:AB59` (`05B5` & 8 = 0: `C680` \|= 1C00h) |
| MiniMapStill* | `$90:A91E`..`$90:A940` (`05F7`, the room's width and height in blocks); the new state handlers that call `$90:A91B` (§Inputs) |
| BossBlanksTheMap | `$90:A7E2`..`$90:A839` and `$90:A8A6` |
| DiagonalPicture | `$90:AAF2`..`$90:AB05` (`$12` = 3 is the third column; `C67C + X` & 1FFh = 28h), `$90:AB5F`; the picture 028 is the importer's (§The HUD's pictures) |
| FirstColumnsShift | `$90:A991`..`$90:A99A` (the origin a byte back), `$90:A9F9`..`$90:AA29` with `$90:AB75` (the 8-bit `ADC #$7D`), `$90:AA6A` (no page correction on the left page) |
| LastColumnsShift | `$90:AA0A`..`$90:AA25` (the 8-bit `SBC #$7C` lands on the left page's next row), `$90:AB3C` (the tile index's step past the right page: +7C0h bytes from a right-page row end leaves the map) |

`scan.sh --mirror docs/behaviours/hud.md docs/behaviours/hud.decode.md` finds 4 of 19 entries
(21 %, limit 30 %) single-routine, marked *: each is a branch or a stretch of one of the two big
routines (`$80:9B44`, `$90:A91B`) or of the switch (`$90:C4B5`), not a routine of its own.

## Findings behind the spec's choices

- **The map's columns run on.** Across the halves (her cell in columns 30 to 33) the original's
  page correction gives the true neighbours: the spec states one 64-wide map. 1,529 recorded
  frames read across (chain-0057, chain-0059, samus-screw-attack).
- **The first two columns.** Her cell in column 0 or 1 puts the window's origin a byte back, at
  the end of the row above (`$90:A999`); the page correction then replaces the in-page byte with
  the right page's byte of that row above (`$90:AA19`). The tiles start at the row above's end
  (`$90:AA6D`, no correction on the left page) and step onto the right page at the row's end
  (`$90:AB3C`). Both land on the cell 32 columns along and a row higher, for every window cell.
- **The last two columns.** The page correction on the right page (`$90:AA23`, `X − 7Ch` in 8
  bits) lands on the left page's row below, replacing the in-page byte; the byte past it is the
  right page's next row's first. The tiles are the right cells (`$90:AA6D`) until the row's end,
  where the step `+7C0h` (`$90:AB3E`) leaves the area's 1000h bytes.
- **The blink** is an `ORA #$1C00` on whatever the window drew, so colour sets 2 and 3 both become
  7: the spec says it takes the blink colours.
- **The diagonal test** reads the HUD word the window just drew, `AND #$01FF`, so the picture's
  bit 9 is ignored and the flips don't matter; and only when the explored branch drew it.
- **The click and the records.** Of the 58 recorded highlight moves with no mute, 36 show sound
  39h on APU port 1 within three records, 22 don't; one muted move shows one (another source).
  `why_click.py` reads the sound queue (`0643` start, `0646` next, `0656`..`0665`, dumped per
  record by `snd_dump` in `$TEMP/collab/hud-spec/dump`, output `snd/`) for each of the 22:
  - 19 were queued (39h in the queue at N+1) and reach port 1 later, 3 to 10 records after the
    move (6 at +3, 5 at +4, 4 at +5, 2 at +6, 1 at +7, 1 at +10): the queue sends one sound at a
    time, each after a 0 written for two records, and the sounds ahead of it go first. The
    three-record window was too short.
  - 2 were dropped at the queue (chain-0106-AD5E 394 and 415): it already held 6 waiting sounds,
    the most `$80:9049` allows for this sound (`$80:9051`..`$80:9060`).
  - 1 is the last record of its row (terrain-quicksand 456): 39h is in the queue, the row ends.

  None is unexplained: every request was made, as `SwitchClick` says, and the queue's capacity and
  pacing decide when it is heard. The queue's other refusals (`05F5` sounds disabled, game state
  28h and up for demos, `0592` a power bomb exploding) reach none of the 22. The sound domain owns
  the queue.
- **The pickup's blank test** is `AND #$03FF` = 0Fh (picture only), the highlight's is the whole
  word = 2C0Fh: the spec keeps both.
- **Counts past their cells.** Three digits of a count over 999 read the division's quotient past
  the digit table (`$80:9D8D`): at 10 the next table's first word, past it the timer's code
  (`$80:9DE7`). Two digits past 99 the same. No vanilla capacity reaches it (missiles 230, super
  missiles and power bombs 50): spec §1.5 bounds the counts at 999 and 99 (docs/LEGACY.md).
- **The map's outer edge.** The window reads past the 64 by 32 map in three places: her cell in row
  31 (or below), the plain window's bottom row; `FirstColumnsShift` with her cell in row 1, the top
  row reads row -1; `LastColumnsShift` with her cell in row 30 or 31, the state comes from row 32 or
  33. On the left page row 32 is the right page's row 0 (bitmap byte `80h + c div 8`, tile word
  `400h + c`), and on the right page row -1 is the left page's row 31 (bitmap byte `7Ch + c div 8`,
  tile word `3E0h + c`, the index `(row + page) × 4 + col div 8`); on the right page row 32, and on
  the left page row -1, lie outside the explored and charted bitmaps (the RAM after `$7E:08F6` or
  before `$07F7`; bank `$82` past the charted table) and the area's tilemap. Her cell in row 32 on
  the left page marks the right page's row 0. The seven area maps draw and chart cells only in
  columns 2 to 57 and rows 1 to 22 (`edges.py`), so no vanilla room reaches any of these, nor either
  column rule. Spec §1.5 defines every such cell as unexplored, uncharted and blank, and exploring
  it as doing nothing (docs/LEGACY.md); the column rules themselves are kept, for randomised maps.
- **Initialising the HUD** (`$80:9A79`, at `$82:8063`) draws the frame (`$80:988B`, `$80:98CB`),
  the icons of what she has, the counts, zeroes `0A06`..`0A0E` and calls `$90:A8EF`, which jumps
  into the window's drawing with stale `$12`/`$16`/`$22` ("broken" in the bank log). It is the load
  flow's, out of scope; the old engine starts from a checkpoint's HUD.

## The HUD's pictures (the importer's)

Spec §1.4 names the pictures and the vectors invent them (MODEL §1 decision 13): the importer
reads the vanilla set from the ROM, and the harness hands it to the model. The vanilla set, with
the colour sets the rules give each role:

| Name | Vanilla pictures | Colours | Source |
| --- | --- | --- | --- |
| blank | `00F` | 3 | the whole word 2C0Fh the highlight tests (`$80:9CEA`) |
| digit 0 to 9 | `009`, `000`, `001`, `002`, `003`, `004`, `005`, `006`, `007`, `008` | 3 | the digit tables `$80:9DBF`, `$80:9DD3` |
| full tank, empty tank | `031`, `030` | 2, 5 | `$80:9BDC` (2831h), `$80:9BD3` (3430h) |
| reserve sign, by rows | `033`, `046`; `047`, `048`; `033`, `046`, the bottom row flipped top to bottom | lit 7, dim 3 | `$80:998B` (lit), `$80:9997` (dim) |
| missiles' icon, by rows | `04B`, `049`, `04B`; `04C`, `04A`, `04C`, the third column flipped left to right | plain or highlighted | `$80:99A3`..`$80:99C7` |
| super missiles', power bombs', grapple beam's, x-ray scope's icons | `034`/`035`, `036`/`037`, `038`/`039`, `03A`/`03B` (top row, bottom row), the second column flipped left to right | plain or highlighted | `$80:99A3`..`$80:99C7` |
| the map's blank | `01F` | charted (3) | the window's blank word 2C1Fh (`$90:AA43`..`$90:AB46`) |
| the diagonal | `028` | | `$90:AAF2`..`$90:AB05` |

Her map's pictures are the area tilemap's words (§Inputs, her map), bits 0-9 with the flips; the
importer keeps their numbers, so the diagonal's test reads the same bits as the original's. The
colour sets stay in the spec: they are the rules' roles (a highlight, a blink), not content; the
palettes' colours are the palette importer's.

## The harness cases

The vectors hold RAM values and invented content only (MODEL §1 decision 13). Round 2's file carried
the game's content in all 62 vectors: the HUD's vanilla pictures in every one, the pictures of a
real area map around her in the 18 recorded ones, map picture numbers an area map uses in 38, and a
vanilla room's place and size in 27 (the 18 recorded, and 9 constructed one-screen rooms at (20,
10), which is a Norfair room's). `$TEMP/collab/hud-spec-r3/real_content.py` checks a vector file for
each kind (the HUD's vanilla picture numbers; a map picture number any area map uses; three or more
of a vector's map cells agreeing with an area map in place; a room cell and size equal to one of the
261 rooms the doors reach from the landing site and from Ceres's start): round 2's file has 6,425
hits, round 3's 0. The constructed vectors' rooms at (20, 10) are now a screen taller.

So the 18 recorded vectors left the file. Each is a **harness case**: the integrator's harness runs
the record pair with the importer's pictures and the area's map, and must match the record at N+1 in
every compared part (§The harness). Each has a constructed twin, named C-R and the case's number:
its frame's RAM values (the counts, capacities, reserve, equipment, buttons and mutes, the
selection, the frame count, her position, her room's size, the explored cells), with the scene moved
four map rows down, the map's pictures invented (`twin_pic` in `convert.py`, then renumbered), the
charted cells two in three of the pictured ones, the frame's cells (its label and border, which no
rule draws) the blank, the HUD's pictures the invented set and the before's window cells the
invented pictures under them, colours kept. Their after is the reference model's, and each twin
takes the same outcome tags as its record (`checktw.py`).

| Case | Row and record (N+1) | Entries | Twin |
| --- | --- | --- | --- |
| R1 | r05 228 | AutoReserveSign (dim), EnergyGauge, HerBlink (lit), MiniMapWindow (across the halves) | C-R1 |
| R2 | r07 134 | AmmoCounts, AutoReserveSign (lit), EnergyGauge, HerBlink, MiniMapWindow | C-R2 |
| R3 | r10 13 | AutoReserveSign, EnergyGauge (two rows of tanks, mixed), HerBlink (unlit), MiniMapWindow | C-R3 |
| R4 | samus-grapple-release 11 | as R3 | C-R4 |
| R5 | area-brinstar 1 | EnergyGauge (unchanged), HerBlink, MiniMapWindow | C-R5 |
| R6 | area-maridia 2 | AutoReserveSign (lit), EnergyGauge, HerBlink, MiniMapWindow | C-R6 |
| R7 | samus-grapple-release 231 | AmmoCounts (missiles), AutoReserveSign, EnergyGauge, HerBlink, MiniMapWindow | C-R7 |
| R8 | r05 185 | AmmoCounts (power bombs), AutoReserveSign, EnergyGauge, HerBlink, MiniMapWindow (halves) | C-R8 |
| R9 | area-maridia 393 | CyclingItems, SelectionHighlight (from none), SwitchClick, and the status bar | C-R9 |
| R10 | r06 662 | the same, the click muted (spin jump) | C-R10 |
| R11 | area-maridia 396 | CyclingItems, SelectionHighlight (moves), SwitchClick | C-R11 |
| R12 | r09 92 | as R11 | C-R12 |
| R13 | r04 265 | CancellingTheItem, SelectionHighlight (to none), SwitchClick, MiniMapWindow (halves) | C-R13 |
| R14 | r05 6 | as R13 | C-R14 |
| R15 | r11 269 | CyclingItems (wraps to none), SelectionHighlight, the click muted (time stopped) | C-R15 |
| R16 | area-brinstar 170 | Exploring, MiniMapWindow, HerBlink | C-R16 |
| R17 | r01 55 | as R16 | C-R17 |
| R18 | samus-ceres-push 2 | MiniMapStill (off), EnergyGauge | C-R18 |

Each case starts from record N's State (§State) and takes record N+1's inputs (§Inputs); none is
in a mask. The harness already runs every record pair, so these are frames it must match by
name: a named case that is skipped or masked is a harness defect.

## Round 2's gate, the spec's side (adversary 57, S1-S4)

What the spec and vectors now state, for the next implementer's shapes:

- **S1, each boss's screens named by the boss.** `BossBlanksTheMap`'s table has a row per boss,
  "any other" gone: the inputs name only these seven. Every boss has its own vector, by name:
  Kraid (C-mm-boss-alone), Draygon (C-mm-boss-draygon), Crocomire (C-mm-boss-across), Phantoon
  (C-mm-boss-phantoon), Mother Brain (C-mm-boss), Ridley in Norfair (C-mm-boss-ridley) and in
  Ceres (C-mm-boss-alone-none). A swap of two bosses' screens now fails, except Kraid's with
  Draygon's, which are equal. The vector test reads the name; its search by screens goes.
- **S2, the frame's boss is Mother Brain's fall.** Two inputs: `a boss encountered`, a step of
  its own (the other six bosses), and `Mother Brain falls`, a frame's `bool`, which also turns the
  mini-map off before its turn (`MiniMapStill`). C-mm-boss is re-keyed as her fall with the
  mini-map on before (and her energy changed, so the status bar is seen drawn), C-mm-boss-across as
  Crocomire's room entered.
- **S3, the one-shot item is the selected item or none.** §1 and the vectors' `one_shot` say so:
  a `bool` on the selection, never true with none selected. A record whose `0A04` is neither 0
  nor `09D2` decodes as a defect (§State).
- **S4, the points the spec backs.** The two blank tests are said once, as differing on purpose,
  with a vector each way (C-highlight-tinted, C-icon-drawn-tinted); digit d's picture is the d-th
  of §1.4's digits; the diagonal's test is stated as a modulus, `p mod 512 = d mod 512`; §1.4
  names colour set 3's roles (the blank's, the digits', the dim sign's, the charted cells'); and
  `OneShotFlash`'s summary gives the flash's order as its rules do (16 plain, then 16
  highlighted: adversary 52's N3).

What stays the implementer's: the keying of the bosses' screens (named fields, a total match),
the form of the two boss inputs, the encoding of the one-shot mark, the picture set's type and
how the model takes it, the step's name in place of `update_mini_map`, and the wording of the
docs adversary 57 lists (`DIGITS`, the modulus, `Reading::read_from`'s row, `first_half` per
user, the colour set's role).

**The switches.** `OneShotFlash`, `HerBlink` and `BossBlanksTheMap` are `switch: **Parameter**`:
`Rules` holds their values (the flash's and the blink's frames each way, each boss's screens),
and the reference rules the spec's. `DiagonalPicture` is no longer one: its value is a picture,
and pictures are content (§1.4), so the importer names it with the others and `Rules` drops it.
Every other entry is a fixed rule.

## Adversary 79's fixes

- **F2, fresh numbers.** Round 3's first invention kept vanilla patterns: the twins' frame cells
  were the vanilla label and border plus 300h, the constructed vectors' map pictures their
  invented numbers plus 100h, the digits and icons vanilla plus one constant (digit d at 1B0h + d
  against vanilla d − 1), and a vector's id named 228h. `migrate.py` now draws every picture
  number afresh: one random bijection on the low nine bits (seed 7, bit 9 kept, so a picture 512
  from the diagonal stays 512 from it), onto numbers that are neither a vanilla HUD picture, nor
  one the area maps use, nor one the file used before. The twins' frame cells are the blank. The
  vector is C-mm-diagonal-512-apart (round 4's name: the diagonal is now the larger number).
  `real_content.py` (over `real_content_lib.py`) now also
  checks:
  - the named set for one constant offset shared by three or more pictures;
  - every HUD cell outside the window for a named picture (so no frame picture survives in any
    numbering);
  - the map and HUD numbers for a constant offset that carries nine in ten of them onto vanilla
    ones;
  - a vector's map cells against the area maps in place or moved by up to 4 cells either way,
    under any offset;
  - ids and comments for a vanilla number.

  The vanilla HUD set is the named pictures and every picture the recorded HUD rows show (47).
  Round 2's file: 12,137 hits; the gated round-3 file: 131 (4 named offsets, 126 frame cells, the
  id); the current file: 0.
- **F3, rules a vector now tells from their absence.** C-sign-lights and C-sign-dims start from
  the other state of the sign; C-energy-unchanged-stale and C-ammo-unchanged-stale from digits and
  a tank that the rule would redraw if it ignored the value last shown. `mutants.py` runs the
  gate's mutants of `model.py`: `sign_never` is killed by C-sign-lights and C-sign-dims,
  `energy_always` by C-energy-unchanged-stale, `ammo_always` by C-ammo-unchanged-stale; without
  these four vectors none of the three is killed.
- **F4.** BossBlanksTheMap's arm for Mother Brain's fall is a "So": her fall sets the mini-map's
  off in the entry's first arm, and MiniMapStill does the rest.
- **F5, the mini-map's off is State** (the root's ruling): BossBlanksTheMap sets it, MiniMapStill
  reads it, and the input `the mini-map turned on` clears it before the frame's update (a new arm
  of MiniMapStill, checked by C-mm-turned-on; `turned_on_ignored` is killed by it alone). The
  vectors carry it as `mini_map_off` in before and after. §6 names the suppliers of `the mini-map
  turned on` (doors and the load flow, Crocomire's fight) and of `the mini-map runs` (her
  control).
- **F9.** The header points at §6.

## Adversary 83's observer items

- **F4, the diagonal on a cell shown charted.** The test runs only when the explored branch drew
  her window cell (`$90:AAF2`..`$90:AB05`, inside the explored branch of `$90:AA43`..`$90:AB46`;
  `mm_bytes.py` line `if e[k] & 0x8000`). Her window cell shows another cell's state only under
  the two column rules, so the plain window never tells the two apart. Two new vectors on
  `LastColumnsShift` at column 62, each with the diagonal as her cell's picture and the map
  collected: C-mm-diagonal-charted (the cell 32 columns back and a row lower charted, not
  explored: her window cell shown charted, the cell over hers stays unexplored) and
  C-mm-diagonal-shifted (that cell explored: the cell over hers joins). `diagonal_on_charted`
  (adversary 83's mutant) is killed by the first alone, `diagonal_never_shifted` by the second
  alone; `xcheck4.py` agrees with both. The spec states it as a "So" of `DiagonalPicture`.
- **F5, a turn-on and Mother Brain's fall in one frame.** Both are enemy AI, run by the main enemy
  routine (`$A0:8FD4`, from `$82:8B65`) before the Samus new state handler that runs the mini-map
  (`$90:E722`, `$82:8B69`): Crocomire's `$A4:90C4` stores 0 to `05F7`, and Mother Brain's
  `$A9:8809` calls `$90:A7E2`. So the enemy processed later has the last word. No room holds both
  (Crocomire in Norfair, Mother Brain in Tourian), so §5 chooses the turn-on first, which is the
  landed model's order, and docs/LEGACY.md lists it. C-mm-turned-on-and-fall (C-mm-turned-on with
  her fall added: off before, off after, the window blank and her four screens explored, no
  update) kills `turn_on_after_fall` alone.
- **F10, boss-kraid-defeated 801: the harness's frames, not the spec.** Record 800 is an overrun
  (cut `O`: the NMI interrupted the main processing, `wait_entry` none). The frame begun after
  record 799 ran Select (`8F` holds item select at 799 and still at 800: the overrun's NMI doesn't
  read the pad) and moved `09D2` from none to missiles before the NMI, and drew the HUD after
  it: at 800 `09D2` is missiles and `0A0E` none; at 801 both are missiles. Every watched hit of
  that frame lies in interval 800, none in 801. The harness's pair (799, 800) is skipped (`05B6`
  equal), and its pair (800, 801) passes the `05B6` test, so it starts from a half-run frame:
  missiles selected, Select still pressed, and the model moves on to the super missiles. Taken from
  cut to cut, the frame is (799, 801): from none, Select gives missiles, as recorded. The spec
  needs nothing, and no LEGACY entry: §Harness now keys the pairs on the cut.
  - In cache-v2, 177 pairs pass today's tests with an overrun at N (10 rows; boss-phantoon 43,
    chain-0232-D5A7 72, item-charge-beam 23, boss-kraid-defeated 15, chain-0314-DAE1 15), and
    only Kraid's 801 mismatches. Joined from cut to cut, each of the 177 frames is a gameplay pair
    with `05B6` advanced by exactly one (168 over one mid-frame record, 8 over three, 1 over
    five), so the compared count holds.
  - The shot's-cancel hit at chain-0314-DAE1 240 is such a frame: record 240 is an overrun, so
    today the hit falls in the skipped pair (239, 240) and the compared pair (240, 241) doesn't
    see it. Joined, the frame (239, 241) holds it.
  - The camera's harness has the same frame rule (`05B6` alone); it is the camera's to check.
- **One coordinate order.** Every place in the vector file is column first: the HUD's `changed`
  keys were "row,column" and are now "column,row", as her map's keys and every `[column, row]`
  value already were (the blind review's item 10). The spec's notation says so. The vector
  test's reading of `changed` changes with it, the next implementer's.
- **The blind review's other points** are the implementer's (names, shapes) or the root's,
  except item 14: §1.3's frame count now says what a lag frame is.

## Adversary 92's fixes

- **A, only her own window cell.** The diagonal test is in row 1's explored branch, at the
  window's middle column (`$90:AAF2`, `$12` = 3). C-mm-diagonal-beside puts the diagonal on
  explored cells beside hers and below it, with her own cell another picture and the cell over
  hers unexplored before and after. It kills `diag_any_column` and `diag_any_row` alone. The
  "So" now says only her own window cell counts.
- **B, her column mod 64.** `$90:A94B`..`$90:A955` keep the page bit and the low five bits.
  C-mm-column-wraps is C-mm-first-columns-0 with her room's cell in column 63 and her in its
  second screen across: the same cells, her cell column 0. It kills `her_cell_no_wrap`.
- **C, the room's blocks down.** `$90:A933`..`$90:A93F` return when `0AFA` div 16 ≥ `07A7`.
  C-mm-outside-below has her y past the blocks and her x inside. It kills `outside_x_only`.
- **D, the diagonal's own mod 512.** `[pictures].diagonal` is now 2E4 (bit 9 set), and the five
  diagonal vectors trade 0E4 and 2E4: C-mm-diagonal shows the diagonal's own number, and
  C-mm-diagonal-512-apart (C-mm-diagonal-512-past before) the number 512 below it. They kill
  `diag_d_unreduced`. No other vector's explored window cell has the low nine bits E4h.
- **E, off their usual conditions.** None of these is gated in the original: `$A4:90C4` and
  `$A9:8809` are enemy AI, whatever her handler; `$90:A7E2` blanks unconditionally; the weapons'
  stores don't read the switch's gate. C-mm-turned-on-not-running, C-mm-boss-not-running,
  C-mm-boss-while-off and C-shot-cancels-cannot-switch kill `turn_on_needs_runs`,
  `fall_needs_runs`, `fall_needs_on` and `shot_cancel_only_switching`, one each alone. The spec
  says each as a "So".
- **F,** §Harness: cache-v2's figures, and every cut but `R` folds into its frame.
- `scripts/edit_vectors2.py` and `scripts/edit_spec2.py` in `$TEMP/collab/hud-r4` wrote these.
  `vecrun.py`: 82 of 82. `mutants.py`: 17 mutants, each killed; the nine new ones by no vector of
  the 75-vector file. `xcheck4.py`: 41 of 43, the two §1.5's choices. `real_content.py`: 0 hits.

## The harness (for the integrator)

The HUD is downstream of everything it shows: nothing in another domain reads its cells, and only
the weapons and her look read its selection, on later frames. It can be verified on its own from
the records, with the other domains' values decoded as inputs.

- **Frames:** every pair N, N+1 of consecutive records at a frame's end of every row: records
  whose NMI returned to the main loop's wait (cut `R`, `NmiKind::Requested`). Every record whose
  cut isn't `R` folds into its frame: `O` (the NMI interrupted the main processing, an overrun),
  `V` (vblank with the NMI disabled) and `S` (vblank ended with the NMI still running) are
  mid-frame. Such a record is never a pair's end, and a pair spans it, with the hits of every
  interval between (frame N+1 is all of them). This is samus-looks' rule (its `folds`).
  Skipped, by name and never by outcome: game state other than 8 at N or
  N+1 (the HUD routine runs only in gameplay and pause; pause is its own domain), `05B6` not
  advanced by exactly one, a room change (`079B` differs: the door's domain). Counted as a
  harness defect: a pair that doesn't decode, or a HUD word without its priority bit.
- **Decoded from record N:** the State: the HUD's cells, the values last shown, `09D2`, `0A04`,
  the explored cells; the frame count and the pad (§Inputs).
- **Decoded from record N+1:** every other input, at the HUD's turn; the HUD's pictures and
  her map's from the importer (§The HUD's pictures).
- **Named cases:** R1 to R18 (§The harness cases) are compared by name, so their skip or mask
  fails the harness.
- **Compared at N+1:** the HUD's 96 cells (picture, colour set, flips) and the values last shown;
  the selection and the one-shot item; the explored cells. The whole HUD tilemap is compared, not
  values: it is the output, and a stale cell is behaviour.
- **Masks.** Each is keyed on its writer, and covers only the frames that writer makes. The one
  below is interim: it goes once its writer is decoded, and until then its frames are reported as
  masked, never as passed.
  - **Her position at the mini-map's turn.** Writer: the pose change's placement, `$91:EB88`,
    which runs after the mini-map (`$90:E734` calls `$90:A91B` first) and can move her across a
    screen boundary. The decode that removes the mask is either of:
    - samus-motion's model run on the frame, taking her position before its pose change places
      her, as the camera harness does for `camera_from`; or
    - the records: `0AF6` and `0AFA` captured at an instruction hit on `$90:A91B` (the mini-map's
      entry, or `$90:E734`'s call of it) in frame N+1.

    Until then mask the mini-map's 15 cells and the explored cells only on frames with a hit on
    `$91:EB88`'s writes to `0AF6`/`0AFA` that moved her `x div 256` or `y div 256`. cache-v2
    records the hit on `$91:EB88` but not what it wrote, and samus-motion's model reports only
    `camera_from` (a y, and only where pose fit or a shrink moved her), not where her moves left
    her, so neither decode is available yet. Today the mask falls back to the frames the writer
    can reach: `0A1C` differs,
    the mini-map runs and isn't off (on any other frame it never reads her), and her position at
    N+1 lies within 32 px of a screen's edge on either axis (the pose change's placements, pose
    fit's push, the shrink, the take-off's lift and the aim step, move her by less). On cache-v2
    that is 882 frames, and 886 with the frames taken from cut to cut (4 joined frames see a pose
    change across a mid-frame record; unmasked, those 4 match). With the mask off, the same 10
    mini-map frames mismatch under both rules, every one inside the mask, as the first cache's 8
    were (§How the spec was made, where the mask was 669 frames on that cache; keyed on `0A1C` alone
    it was 3,039, and on `0A1C` with the mini-map reading her 2,705).
  - **The shot's cancel** needs no mask: its writers, the weapons' four stores, `$90:BF25` and
    `$90:C091` (clear `09D2` and `0A04`) and `$90:BF41` and `$90:C0A6` (clear `09D2`, reached
    only when `0A04` is 0), are decoded from their hits (§Inputs), and the selection is compared
    on those frames like any other.
  - **The click** (`Click`) isn't compared until the sound domain's request stream exists (U1);
    report the model's requests beside the APU port writes. Of the 22 recorded moves with no 39h
    in three records, each has a cause in the queue (§Findings, the click).
- **What the integrator needs from the records.** cache-v2 watches the shot's cancel, the
  pickups, `$90:A7E2`, `$90:A91B`, `$90:E734` and `$91:EB88` on every row. Still needed, at the
  next re-record (then a ci-data bundle the root uploads):
  - `0AF6`, `0AFA` peeked at the hit on `$90:A91B` (her position at the mini-map's turn; the
    hit's peek is empty today), or none if samus-motion's model supplies it;
  - the six stores of 0 to `05F7` (§Inputs, `the mini-map turned on`): none is watched, so
    `turned_on` is false because no one watched it;
  - for the click's check later: `0643`, `0646` and `0656`..`0665` per record.
- **Null model:** nothing changes (the cells, the values last shown, the selection, the explored
  cells as at N): on cache-v2, 17,975 of the 20,615 compared frames, and 17,954 from cut to cut
  (the first cache: 10,077 of the 11,900 frames with neither of its masks).
- **Per-behaviour attribution:** `model.py`'s tags name the rule each frame takes; `vectors.py`'s
  `entries_of` maps tags to entries.
- **Free run:** from a seed frame, carry the HUD's own State forward and take only the inputs from
  the records; report the first differing frame per part (status bar, selection, mini-map).
  Re-seed at a room change (the door's domain resets the frame count and redraws nothing of the
  HUD, so the HUD carries over: a free run across doors is the better test once doors run).

## Size baseline (tokens, 2026-11-08)

`tools size` (the wire-cam counter, `$TEMP/collab/wire-cam/size.dll`, the quality-terms weights),
at spine `387640758`, over cuts of the old code in `$TEMP/collab/hud-spec/old`, measured together:

| Area | Old code | Score | Logic | Types | Data | Depth | Lines |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| status | `game/src/hud.rs` (all of it: the phase and its upload, the draws, the highlight); `game/src/state/hud.rs`: `Cell`, `HudTilemap` and their methods | 1,253.75 | 1,366 | 211 | 251 | 82.75 | 252 |
| minimap | `game/src/mini_map.rs`; `data/src/map.rs`: `Page`, `MapCell::layout`, `LayoutIndex` and its methods, `MapCells::contains`, `contains_at`, `insert`, `AreaMap::tile`, `AreaMaps::of`; `game/src/state/hud.rs`: `MiniMap` | 1,665.5 | 1,741 | 454 | 47 | 144.5 | 347 |
| total | | 2,919.25 | 3,107 | 665 | 298 | 227.25 | 599 |

Lines are code lines (`$TEMP/collab/scripts/code_lines.py`: no blanks, comments or tests). Not
counted: `MapCells::from_bitmap`, `to_bitmap` and `AreaMaps::load` (the importer and the decoder's),
`tick.rs`'s `mini_map` (the bridge's). The selection has no old code: the old engine kept the
selected item in the State and never changed it.

**k out of sample:** the old code scores 59.7 a unit on the status bar (1,253.75 over 21.0) and
64.1 on the minimap area (1,665.5 over 26.0); 62.1 over both, which at 0.55 would give k = 34.2
against the frozen 36. Neither cap binds: status 756 of a 877 cap, minimap 936 of 1,166.

## Scan

`tools/cleanroom/scan.sh docs/behaviours/hud.md scenarios/vectors/hud.toml`: 0 in every check
(`SM_SCAN_DLL_DIR=$TEMP/sm-cleanroom-hud3o`). The HUD added only multi-word player phrases to
`tools/cleanroom/scan_allow.txt`: `x ray scope`, `ray scope`, `item select`, `item cancel`, `missile
tank`, `pick up` and `mother brain` (the progress spec, landed first, holds `x ray scope`, `ray
scope`, `missile tank` and `mother brain` too, so the HUD's own section keeps the other three). No
single word: the spec is worded so that `hud`, `map`, `area`, `row`, `column` and the rest never
join a lexicon phrase (the title's comma, the area named `minimap`, "second column, left to right",
"cells drawn as a diagonal", "the game's opening" for the intro demo).

**The model's scans, ruled at round 1's gate (adversary 52).** The root accepted each hit below for
this round; none is a leak.

- **The mirror scan's one hit.** `scan.sh --mirror` on the model finds 1 of 3 tagged public
  functions mapping onto one routine: `mini_map::update`, whose first named entry, `Exploring`,
  resolves to `$90:A91B`. Accepted, because its boundary and order are the spec's own: "the
  mini-map updates" is `MiniMapStill`'s condition, and §5 gives the order (her cell explored, the
  window, the diagonal cell after, the blink last). It isn't to be hidden by making it
  `pub(crate)`: the check is textual, so that would change how the shape looks, not the shape.
- **The name scan's prose phrases.** "two column", "top left", "column right", "column left", "up
  one", "down one", "abov below", "draw til", "til blank", "draw item", "room map", "bos room" and
  "word per" are the spec's own wording in doc comments. Accepted.
- **The enum-order hit.** `tests/support/toml.rs`'s `Value` (Str, Int, Bool, Array, Table) is a
  TOML value's kinds; its match with a RAM code table is a coincidence. Accepted.
- **`AreaMap`**, the one name hit on a type ("area map", 18 uses), is renamed: round 2 calls it
  `HerMap`.
- **A count past what its cells hold** stops the session (`HudFault::CountPastCells`), where the
  old engine left the cell unchanged. Accepted: unreachable in vanilla, whose capacities stop below
  the cells' reach, and listed in docs/LEGACY.md.

**The model's scans, ruled at round 2's gate (adversary 57).** The root accepted each hit below for
this round; none is a leak.

- **The name scan's new hit,** "updat mini map", is the step `HudState::update_mini_map`, which
  replaced round 1's `mini_map::update`. Accepted for round 1's reason: its boundary is
  `MiniMapStill`'s "the mini-map updates" and its order is §5's. The other hits are round 1's
  accepted prose phrases and the `Value` enum order. The next round names the step for its entries.
- **The mirror scan can't see the round's steps.** It finds public functions by the text `pub fn`
  and `pub const fn`, and round 2 made every rule a `pub(crate)` step on the state, so it reports 0
  of 0 tagged functions. This is the scan's blind spot, not a hidden shape: an engineer widens it to
  `pub(crate) fn`, or to every function whose doc names a `Spec:` entry.

**BossBlanksTheMap's per-boss table** was unchecked at round 2: the vector test picked each
vector's boss by the screens it blanked, so a swap of two bosses' screens passed. Round 3's
vectors name the boss (`boss` in a "BossEvent" step, `mother_brain_falls` in a frame), one vector
per boss (§Round 2's gate), so the test reads the name.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here, and `tools report` the areas' old
figures and the commit.

Size is counted in logical tokens: each identifier, keyword, literal and operator is one, and so is
each bracket pair; comments, layout and tests are free. Two kinds of token are counted apart and
reported, not held against the target: the values of constants and statics (tables of the game's
data, such as the HUD's pictures and where each count sits), and declarations (the `struct`, `enum`
and `type` items and attributes that define the model's shapes; define as many as the behaviour
needs). The target is on everything else, the logic.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on samus-motion. An
area's budget is k × its units; its old code (`old`, for reference, not a bound) is the `tools size`
score of the old code that area replaces, at spine `387640758`: for the status bar `game/src/hud.rs` and the
cells of the HUD in `game/src/state/hud.rs`; for the mini-map `game/src/mini_map.rs` and the
map layout it walks in `data/src/map.rs`, without the ROM loader and the bitmap conversions. The
selection has no old code: the old engine never changed the selected item. The budget is an
indicator: the fit is score ÷ budget, and being over it costs in proportion to how far over, never
as a wall; over an area's old code is a failure.

- Area status (old 1,254): 1, 1.4, EnergyGauge, AmmoCounts, AutoReserveSign, SelectionHighlight,
  SwitchClick, OneShotFlash, IconsAppear
- Area selection: CyclingItems, CancellingTheItem, OneShotItem, Switching
- Area minimap (old 1,666): Exploring, MiniMapWindow, HerBlink, MiniMapStill, BossBlanksTheMap,
  DiagonalPicture, FirstColumnsShift, LastColumnsShift

Round 3 adds one arm (`MiniMapStill`: the mini-map turned on clears its off, now State) and
one operator (`DiagonalPicture`'s modulus on both sides): 55.0 → 56.3 units, minimap 936 → 984.
The arm the gated draft added for Mother Brain's fall is a "So" (adversary 79 F4).

Round 4 adds no arm and no operator: its three vectors are cited by arms that were there, and
`DiagonalPicture`'s new bullet and §5's order are a "So" and prose. 56.3 units.

Budget: 37.5 × 56.3 units = 2,112 (status 787, selection 300, minimap 1,025)
