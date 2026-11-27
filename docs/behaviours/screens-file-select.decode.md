# File select: decode and harness

Companion to the [file select spec](screens-file-select.md). Keep these notes outside confined
exports.

## Sources

Watched first in the `any-power-on` records, cuts 393 to 464, then pinned in the bank logs.
Game state 4 (`$82:89E5`) calls the file select menu `$81:93FB`, which runs the routine of the
menu index `$0727`.

### MenuLoads

Source: [Bank 81](https://patrickjohnston.org/bank/81):
- index 0, `$81:944E..94A2`: handle fading out (already dark); `$80:836F` set forced blank and
  wait for NMI (cut 393, a finished cut inside the pass); queue sound 1 of library 3
  (`MenuOpens`); HDMA off; index 1; map VRAM for the menu, load the menu tiles, scrolls 0, the
  file select palettes. The pass ends at 395 (394 is lag);
- index 1, `$81:9E93..9ED5`: the planet and stars tilemap to BG2; index 2 (one pass, 396);
- index 2, `$81:9ED6..A052`: the slot chosen last and copy and clear (`CursorStarts`); the menu
  tilemap; each slot loaded or cleared through `$81:8085` (P2) with its energy and time drawn;
  the helmets; fade delay `$0723` and counter `$0725` 1; `$80:8382` clear forced blank and wait
  for NMI (cut 400, inside the pass: `inidisp` 00h); index 3. The pass runs as lag 397 to 399
  and ends at 401.

So the load is five finished cuts (393, 395, 396, 400, 401); two of them (393 and 400) are NMI
waits inside a pass, after which no new pass begins and no random number is drawn: the record
draws at 393, 396 (the pass begun at 395), 397 (lag; the pass begun at 396, seen at 400) and
402, and not at 395 or 401.

### CursorStarts

Source: as MenuLoads, plus in index 2 `$81:9ED6..A052`: the slot chosen last from SRAM
`$70:1FEC` with its complement at `$70:1FEE` (valid when 0 to 2 and the slot AND the complement
is 0, else slot A) into `$0952`; each slot loaded or cleared through `$81:8085` (P2); copy and
clear only when a slot holds a file.

### MenuBrightens

Source: Bank 80 `$80:894D` (handle fading in: while the counter `$0725` is above 0, decrement
it; otherwise counter = delay `$0723` and the level up one, unless at 15). Bank 81 index 3
`$81:A058..A086` (fade in; at 15, index 4). The delay is 1, so one level per two finished
frames: 0 at 400 to 402, 1 at 403, ..., 15 at 431. The menu pace is the delay plus one.

### MenuDims

Source: `$80:8924` (handle fading out: the wait as in `MenuBrightens`, then the level down one,
at 1 forced blank and level 0). Index 20h `$81:94A3..94D4` (fade out; at level 0, game state 2,
index 0, the menu RAM cleared). The delay is 1, as for `MenuBrightens`: 14 at 436, ..., 1 at
462 and 463, dark with game state 2 at 464.

### ItemPressed

Source: Bank 81 index 4 `$81:A1C2..A311`: A or Start (mask 1080h) first, ahead of Up, Down and
B. On a slot: sound 2Ah of library 1 (`ChoiceMade`), index 1Fh, the helmet's animation started,
SRAM `$70:1FEC` = the slot and `$70:1FEE` its complement, then `$81:8085` loads the slot, and
on an empty slot `$81:B2CB` makes a new file (P2). On copy (3) or clear (4): sound 37h
(`CursorMoved`) and indexes 5 or 13h. On exit (5): index 21h, no sound.

### CursorMoves

Source: as ItemPressed, Bank 81 index 4 `$81:A1C2..A311`: without A or Start, Up (0800h), then
Down (0400h), then B (8000h). B: sound 37h twice and index 21h (back to the title, a soft reset
at dark). The cursor's moves follow `$81:A25E..A2B1`, with the non-empty slots `$0954` from
index 2; each move asks for sound 37h. The cursor table `$81:A312` places the missile, which is
the picture's.

### HelmetTurns

Source: Bank 81 index 1Fh `$81:9D77..9DC2`: A or Start, or a slot's helmet at frame 7 with its
animation timer 0, sets index 20h. The choice sets the chosen slot's timer to 1 (`$81:A236`);
`$81:9DE4` steps its frame once every 8 frames of work, and the turn ends at frame 7 with the
timer 0 on the 57th pass of index 1Fh. Index 2 zeroes all three helmets (`$81:9FF7..A00C`), so only
the chosen one turns, and the original's check of all three reads only that one. The turn is
fully determined, so this owner counts it (the root's ruling): 57 frames of work after the
choice, the Parameter `helmet_turn`. No frame of work in `Confirming` holds across a frame end,
so its frames of work are its finished frames. In the movie, A latched at 433 ends it at 434.

## RAM mapping

| Quantity | Source | Notes |
| --- | --- | --- |
| screen | the record's `inidisp` at the cut | 80h from 392 to 399, 00h (lit, level 0) at 400 to 402 |
| cursor | `$7E:0952` | 0 to 5 in the menu's order |
| fade wait | `$7E:0725` | the screen fade counter; the menu pace less one is the delay `$0723` |
| stage, load step | `$7E:0998` (game state 4), `$7E:0727` (menu index) | `Loading`: indexes 0 to 2 and the five finished cuts; `Brightening`: 3; `Choosing`: 4; `Confirming`: 1Fh; `Leaving`: 20h; `OptionsBegin`: game state 2; `TitleReturns`: 21h; `CopyBegins`: 5; `ClearBegins`: 13h. Compared through their effects only |
| file chosen | `$70:1FEC` (SRAM), written by P2 | the choice reported to P2 |
| sounds asked for | the sound queues of libraries 1 and 3 | `MenuOpens` library 3 sound 1 (queued after the wait at 393, so asked for at 395); `CursorMoved` library 1 sound 37h; `ChoiceMade` library 1 sound 2Ah |
| finished frames, random number (the frame owner's) | `$7E:05B6`, `$7E:05E5` | as in the title notes |
| whether the game waited for the frame end (borrowed) | the record's cut kind | `Requested` NMI: yes; `Overrun`: no |

## What save memory's owner (P2) must state

- On the load's fourth finished frame end (the pass that runs from 396 through the lag cuts to
  the wait at 400; `$0954` is still FFFFh at 397 to 399 and settled at 400), for each of the
  three slots: whether it holds a valid
  file (P2 checks each one and may clear a damaged one), and the slot chosen last, valid only
  when save memory's record of it names slot A, B or C (0 to 2) and that value AND its stored
  complement is 0 (a weaker test than the two agreeing: slot B with a complement of 0 passes).
  In the movie both read FFFFh at 393, so none is valid and the cursor starts on slot A.
- On a slot chosen: save memory records the choice (the slot, and its check copy: two bytes in
  this movie, visible at the lag cut 432); the chosen slot's file is loaded into the game, or,
  when the slot is empty, a new file is made: the new game's progress (`run_progress`'s fresh
  progress), with the current area's map not collected; a loaded file also brings its explored map. In the movie the slot
  is empty, so the new file is made in the frame that ends at 433.
- SRAM is unchanged otherwise from 432 to 567; the intro at 568 starts from that new file.

## Harness contract

Row `any-power-on`, continuing the title's run: the title's `FileSelectBegins` at 392 hands
over `Loading` at step 0 with the screen `Blank(0)`; the frame owner carries the count and the
random number on. Save memory's answers come from P2: in the movie no slot holds a file and no
valid slot was chosen last. The run plays 393 to 464; cut 465 is the options menu's and stops
the run as Unsupported until it is owned.

Compared, per cut:
- `screen`, every cut;
- `finished frames` (the frame owner's), every cut;
- `random number` (the frame owner's), at finished cuts, drawing none at the finished cut after
  a frame end where the frame goes on;
- `cursor`, at finished cuts from the load's fourth (400);
- the sounds asked for, at finished cuts: the entries added to the library 1 and 3 queues since
  the last finished cut, against the sounds asked for since it. Expected and played are sorted,
  so the order within one interval is not compared.

Planted fault: the menu pace 3. The run must fail on `screen` at 403 (model 0, record 01h).

Not compared, and why (the root's ruling: the picture is not compared):
- VRAM, CGRAM and OAM (the menu tiles and text, the planet, the helmets, the missile cursor):
  the picture's owners and AF5.
- SRAM (P2's): read only for save memory's answer of the slot chosen last, not compared. The
  choice's write is part-done at the lag cut 432, and P2's to compare.
- The queues' heads and timers: the audio side's.
- The helmets' animation and the missile's: the picture's.

Borrowed (named in the runner's `BORROWED`), under MODEL decision 35: whether the game waited
for each frame end (the cut kind).

## Reference

`$TEMP/collab/title-752/scratch/fsmodel.py` runs the spec on every vector, and 50 of 50 agree.
`fspercut.py` runs it per cut over `any-power-on` 393 to 464 from the hand-over, with the frame
owner's count and draws as stated, and all 72 cuts agree:
- screen 72 of 72;
- finished frames 72 of 72;
- random number 67 of 67 (the finished cuts);
- cursor 64 of 64.

A do-nothing model (the hand-over State frozen) agrees at 0 of 72 cuts: screen 8 of 72, finished
frames 0 of 72, random number 0 of 67, cursor 0 of 64.

Recorded vectors take `before` from the last finished cut ahead of the one they model.
