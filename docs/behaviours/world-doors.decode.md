# Doors between rooms, part 1, the passage: decode notes

For the observer, the adversary, the integrator and the domain harness's decoder only.
Implementers never see this file (MODEL §5). It holds every source the spec draws on: the
original's routines, its RAM and its codes. Addresses are the bank logs'
(https://patrickjohnston.org/bank/; local copies `$TEMP/collab/banklog/80.txt`, `82.txt`).

## How the spec was made

Behaviour first. Every row of CI's lean record cache (`$TEMP/collab/ci-cache/rows`) was searched
for game states 9, Ah and Bh; 18 rows hold a passage (the recorded cases below). Each was dumped
per frame by two cdylibs over `verify::cache` (`$TEMP/collab/r1-spec/dump/dump.rs` and
`$TEMP/collab/r1-fix/dump/dump2.rs`, run through `$TEMP/collab/wire-r7/run.py`; output
`d-<row>.txt` and `e-<row>.txt`, local only): the cut, the game state, the door transition
function, the room, the door's direction, destination and speed, her position, previous position,
pose, movement type, animation counter, charge and speeds, the pad and the shoot binding, layers 1
and 2, the BG scrolls, the scrolling frame counter and finished flag, the palette numerator, the
256 colours and the 256 target colours, the three sound queues, the music queue (timers, start
index, music timer), sounds disabled `05F5`, the mini-map `05F7`, the NMI request flag `05B4`, the
8-bit frame counter `05B5`, the frame counter `05B6`, the lag counter `05BA`, the timer status,
the CRE bitsets, and each enemy slot's id and frame counter. They were read over time; the bank
logs were then read to pin the fade's arithmetic, the alignment's rounding, the vertical doors,
the nudge, the IRQ's commands and the music queue.

The reference model, `$TEMP/collab/r1-fix/ref/passage.py` (`step`: one frame), is run over the
dumps by `ref/model3.py`: from the touch's record it steps every frame, with the recorded timing
as T1's input (below), and compares what the table lists. Its non-fade inputs from the records: the sound
queues' emptiness at the last frame of work (`0643`..`0648`), the music's readiness from the
music queue (below), the timer (`0943`), the CRE bitsets (`07B1 | 07B3` bit 0), the doorway's
arrival (`0791`, `0927`, `0929`, `092B`, `092D`), the new room's target colours (`7EC200`) during
`Brightening`, and, at the frame `Arriving` ends, the colours the loading wrote to sprite palettes
6 and 7 (`7EC1C0..7EC1FF`).

Over the 18 passages (2,893 frames, 785 frames of work):

| Compared | Model | Do-nothing |
| --- | --- | --- |
| every frame: her position, where the camera last saw her, the camera | 2,893 | 1,756 |
| every frame: her pose and speeds as the touch left them | 2,893 | 2,893 |
| every frame: a finished frame (`05B6` steps) | 2,893 | 1,815 (finished when the work ends) |
| every frame: the HUD's frame count `05B5` (1 on a restarting frame, else counting finished frames) | 2,893 | 2,750 (counting frames of work) |
| frames of work: the phase and its data | 785 | 44 |
| frames of work: the 256 colours | 756 | 355 |
| frames of work: the silence (sound 71h, library 2) queued exactly on the first | 785 | 767 |
| frames of work: sounds held (`05F5` = FFFFh) exactly while passing | 785 | 751 |
| the first frame of work: the mini-map on (`05F7` = 0) | 18 | 16 |
| the first frame of work: her sound at the door, library 1, as G6's rule picks it | 18 | 0 |
| the last frame of work: her spin's sound, library 1, as G6's rule picks it | 16 | 15 |
| frames of work the enemies don't run: no enemy's frame counter `0FA4` steps | 207 | 207 |
| frames of work the enemies run: some enemy's frame counter steps | 452 of 487 | 452 |

The phase figure no longer reads its own field: the music's readiness comes from the queue. With
the music fixed at ready the phase falls to 761 of 785, so the queue's input is the one that
holds. The 35 enemy frames are two rows whose rooms have no enemy in view (`chain-0097-ACF0`
both sides, `chain-0014-96BA` one side): the enemy framework processes only those in view (C1).
The 29 colour misses are G2's two writers. Over the ten Ceres passages alone: 1,689 of 1,689
frames, 457 of 457 phases, 445 of 457 colour frames (the 12 of row 10). The loading's writes
taken as input: 182 colours over 17 passages.

`ref/vecrun2.py` runs every vector through the same `step`: 94 of 94 agree. `ref/vecgen2.py`
writes the vector file and refuses to unless the 15 vectors below, worked by hand from the
spec's rules and not from `step`, agree with it (they do).

### Vectors worked by hand

- `leaving` (Quieting, TheFades, WhatRuns): the always-lit `5EF7` and the icon `3DEF` keep their
  colour as goal, the other `7FFF` and the timer's `0210` (timer not running) go to `0000`; the
  enemies run and all is drawn; the silence and the held sounds, `AtTheDoor`; cost `Other`, a
  finished frame, no restart.
- `fade-half` (Fading): `1A6B` toward `0000` at `t` 7, `d` 12, divisor 6: red 11:
  `⌊(2816 − ⌊2816 / 6⌋) / 256⌋` = 9; green 19: `⌊(4864 − 810) / 256⌋` = 15; blue 6:
  `⌊(1536 − 256) / 256⌋` = 5, so `15E9`. `0421` toward `7FFF`: each channel 1 to
  `⌊(256 + ⌊30 × 256 / 6⌋) / 256⌋` = 6, so `18C6`.
- `bright-step` (Fading): `2108` toward `7FFF` at `t` 5, divisor 8: each channel 8 to
  `⌊(2048 + 736) / 256⌋` = 10, `294A`; `00A0` toward `03E0`: green 5 to `⌊(1280 + 832) / 256⌋` =
  8, `0100`.
- `square-down` (Squaring): y `0280`, `mod 256` = 128: y + 1, `0281`.
- `set-off-right` (SettingOff, Sliding, Carrying): the arrival's x `0100` less 256 is `0000`, the
  step makes it `0004`; her x `00EC.0000` + `0.C800` = `00EC.C800`; seen `00EC`; slide 1.
- `place-right` (WhereSheLands, Arriving): x `0004` + `EC` = `00F0`, y `0000` + `94` = `0094`,
  subpixels kept; the slide doesn't step on the placing frame; `FetchTiles`, cost `FetchTiles`.
- `slide-down` (Sliding, Carrying, FinishedFrames): y `0240.8000` + `1.8000` = `0242.0000`, the
  view `0250` + 4 = `0254`; a finished frame and a restart though T1 is asked nothing.
- `slide-adjusted-down-end` (Sliding, F5): with 41 frames, frame 41: she `0280.8000` +
  `1.8000` = `0282.0000`; the view `02BC` + 4 = `02C0`, then onto the arrival `0300`; she lands,
  `Landing(0)`.
- `land-left`: `01E6` & ~7 = `01E0`. `clear-left`: `01E3`, `mod 256` = 227: `01E0` − 8 = `01D8`.
- `forced-wait` (FinishedFrames, Landing): music not ready, work not done:
  `Music { started: false }` stays, a finished frame, a restart.
- `slide-adjusted-up-end` (Sliding): step 8, 41 frames, frame 41: she `0072.4000` −
  `1.8000` = `0070.C000`; the view `0040` − 8 = `0038`, 24 short, then onto the arrival `0020`;
  `Landing(0)`.
- `slide-adjusted-left-end` (Sliding, Carrying): 32 frames at `1.0000`, frame 32: she
  `0108.9800` − `1.0000` = `0107.9800`; the view `000C` − 4 = `0008`, 8 short, then onto the
  arrival `0000`; `Landing(0)`.
- `carry-adjusted-up` (Carrying, Sliding): she `0084.4000` − `2.4000` = `0082.0000`; the
  view `0080` − 8 = `0078`; slide 21.
- `bright-steps-adjusted-end` (Fading): `d` 6, `t` 8 = `d` + 2: play goes on, sounds let
  through, `SpinAgain`.

### Mutants

Each a one-line change to `passage.py` (`MUTANT`), run over the 70 vectors by
`vecrun2.py TOML NAME...`; every one dies. The first six hard-code a Parameter value on one way
or one fade only:

| Mutant | Killed by |
| --- | --- |
| the step fixed at 4 on the vertical axis only | `carry-adjusted-up` |
| the length up fixed at 57 | `slide-adjusted-up-end` |
| the length left fixed at 64 | `slide-adjusted-left-end` |
| the speed up fixed at 1.8000 | `carry-adjusted-up` |
| the speed left fixed at 0.C800 | `slide-adjusted-left-end` |
| `d` fixed at 12 in `Brightening` only | `bright-steps-adjusted-end` |
| slide length across fixed at 64 | `slide-adjusted-end`, `slide-adjusted-left-end` |
| slide length up/down fixed at 57 | `slide-adjusted-down-end`, `slide-adjusted-up-end` |
| speed up/down fixed at 1.8000 | `carry-adjusted-down`, `carry-adjusted-up` |
| no snap at the slide's end | `slide-up-end`, `slide-adjusted-end`, `slide-adjusted-down-end`, `slide-adjusted-up-end`, `slide-adjusted-left-end`, `loading-done` |
| no forced finished frames | `work-pending-slide`, `slide-right`, `forced-wait` |
| slide step fixed at 4 | `slide-adjusted`, `carry-adjusted-up` |
| `d` fixed at 12 | `bright-steps-adjusted-end`, `steps-adjusted`, `steps-adjusted-last`, `steps-adjusted-end` |
| speed across fixed at 0.C800 | `slide-adjusted-left-end`, `carry-adjusted` |
| the down slide's frame 0 moves | `set-off-down`, `slide-down-first` |
| placing drops her subpixel | `place-right`, `place-left`, `place-pending` |
| the timer's colours kept whether it runs or not | `leaving` |
| the sounds' wait ignored | `silence-waits` |
| the music's wait ignored | `music-waits` |
| squaring rounds 128 up | `square-down` |
| `Up` sets off 256, not 255 | `set-off-up` |
| `Up` without the 32 px | `set-off-up` |
| seen written at the landing | `land-right`, `land-left` |
| the bottom row stepped up too | `clear-bottom` |

## State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| the phase | game state `0998` (9, Ah, Bh), then the door transition function `099C` (below) | |
| `t` | palette change numerator `7EC400` | the denominator `7EC402` = Ch is `d` |
| the arrival: way | door direction `0791` & 3: 0 `Right`, 1 `Left`, 2 `Down`, 3 `Up` | bit 2 spawns the closing cap (W2's, `$82:E8EB`); stale until taken |
| the arrival: view | `0927`, `0929` (door destination, `[door + 6]`, `[door + 7]` × 100h, `$82:DE12`) | `0929` += 20h for `Up` at `$80:ADEB` |
| the arrival: speed | `092D`.`092B` (`[door + 8]` / 100h; negative: C8h or 180h by the direction) | the doorway's "distance to spawn" |
| the slide's frames | door transition frame counter `0925` | finished flag `0931` bit 15 |

**Phases from `099C`** (the record at a frame of work holds the function the next frame of work
runs, which is the phase the spec names): `Quieting { asked: false }` game state 9 or Ah (`$E169`,
`$E17D`, `$E1B7`); `Quieting { asked: true }` `$E29E`; `Darkening` `$E2DB`; `Squaring` `$E2F7`
(arrival not taken) and `$E310`; `BeforeTheSlide(n)`, `n` the frames of work since the squaring
ended: `$E353`, `$E36E`, `$E38E`; `Arriving` `$E3C0` (fetching `Tiles`) and `$E4A9` with `0931`
bit 15 clear (`Contents` or `Done`: a record can't tell the contents fetched from the contents
being fetched, so the harness compares the two as one); `Landing(n)`, `n` the frames of work
since the slide ended: `$E4A9` with `0931` bit 15 set (its NMI wait at `$82:E540` passed: the
snap follows), then `$E659`; `Music { started: false }` `$E664`, `{ started: true }` `$E6A2`;
`Brightening` `$E737`; play is game state 8. `placed` is the model's own: she is placed
at the start of `$E3C0` (`$82:E3C5`..`$82:E3E2`), on the frame after `$E38E`'s. Check `0931` only
with `$E4A9`: at `$E3C0`'s record it still holds the last slide's bit. `$E675` is unused.

**Outputs:** her position `0AF6`.`0AF8`, `0AFA`.`0AFC`; where the camera last saw her `0B10`,
`0B14` (pixel words only); the camera `0911`, `0915` (pixel words; `090F`, `0913` untouched);
layer 2 `0917`, `0919`; the colours `7EC000..7EC1FF`, the target colours `7EC200..7EC3FF`;
the mini-map `05F7` = 0; sounds disabled `05F5` = FFFFh, enabled 0 (`$82:E757`); the silence:
sound 71h, library 2 (`$82:E26F`); her sounds: `$84:8250` (Samus command 1Dh, `$90:F471`) at
`$82:E26B` and `$84:8270` (Samus command 1Ch, `$90:F41E`) in `$E737`'s last step; finished
frames: `05B6` steps, `05B5` the HUD's count; the slide's screen: `$212C` = 11h or 10h
(`$80:9781`/`$80:977D`, `$80:97EA`/`$80:97E6`).

## Inputs

| Spec | RAM | Notes |
| --- | --- | --- |
| the door touched | `078D` (door pointer), set by the block collision's door BTS (`$94`), game state 9 | samus-movement's `TouchedCells` (the doorway id from the importer) |
| the sounds sent | `0643`..`0648`: each of the three queues' next index − start index, `& Fh`, is 0 | `$82:E2B0` |
| the music ready | the music queue at the record before the frame of work, after one step of `$80:8F0C` (run by the HDMA object handler `$88:84B9` at the main loop's start, `$82:894B`): if the music timer `063F` is 1, the entry at the start index `063B` leaves; ready when every timer `0629`..`0637` is then 0 (`$80:8EF4`) | 41 of 41 waiting frames of work |
| either room keeps the items dark | `(07B3 | 07B1) & 1` after `$82:DDF1` (the destination's CRE bitset and the previous) | read by `$82:E221`, `$80:9773`, `$80:97DC` |
| the timer | `0943` ≠ 0 | |
| the work done | the record's cut is `Requested` | T1; see the contract below |

**T1's contract.** The passage names a cost event for each frame of work: `FadeStep` (each
`$E2DB` and `$E737` call, with the enemies' run inside it), `BuildRoom` (`$E36E`: the room
and state headers `$82:DE6F`, `$82:DEF2`, the effects `$88:82C1`, the level, scrolls and CRE
`$82:EA73`), `FetchTiles` (`$E3C0` after the placing: the tileset's
decompression and VRAM uploads), `FetchContents` (`$E4A9` up to `$82:E524`), and `Other` (every
other function: one frame on record). T1 answers, per frame, whether that cost is spent. It
reports only the work's cost: the slide's wait (`$82:E526`) is the passage's, so a T1 fitted from
the records must not take `FetchContents`'s record length (the wait) as its cost; the records
bound that cost by the slide's end only. Nor does T1 own the forced finished frames
(`FinishedFrames`): the IRQ's door commands own them. Until T1 exists, the diagnostic decodes
"the work done" as the cut being `Requested`, for every cost event. For fades it also decodes
colour progress independently from the interrupted source location and cursor below. Runtime
T1 must produce real semantic progress from elapsed costs; native progress is not a runtime
input. A cost receipt cannot certify palette completion merely by certifying frame completion.

**The HUD's colour groups** (`TheFades`; the importer's, by palette index; `7EC000 + 2 ×
index`): always lit 9, 10 (BG3 palette 2 colours 1, 2), 13, 14 (palette 3 colours 1, 2), 17 to 19
(palette 4 colours 1 to 3), 29 (palette 7 colour 1); icons 20 to 23 (palette 5), 28 (palette 7
colour 0); timer 209, 210, 212, 221 (sprite palette 5 colours 1, 2, 4, 13); visor 196 (sprite
palette 4 colour 4).

## Sources per entry

| Entry | Sources |
| --- | --- |
| TheTouch | `$82:E169` (game state 9), `$82:E17D` (returns carry set for a door, so `$82:E1B7`, game state Ah, runs the same frame); the records |
| PassageTime | `$82:E169`, `$82:E1B7`, `$82:E288` and each function's `STA $099C`; `$82:E526` (the busy-wait), `$82:E540` (its NMI wait); the records' cuts |
| FinishedFrames | the IRQ's door commands: horizontal 16h/18h/1Ah (`$80:97C1`, `$80:97DA`, `$80:980A`, which sets `05B4` = 1 and `05B5` = 0 at `$80:9823`), vertical 10h/12h/14h (`$80:9758`, `$80:9771`, `$80:97A9`, `$80:97BA`); set at `$82:E3EB`..`$82:E400` (placing, after `$82:DF69` waits for vblank's end, so the first is the frame after the placing's) and `$82:E492`, restored to gameplay at `$82:E706` (`$E6A2`); the NMI's finished branch `$80:95E5`..`$80:95F4` |
| Quieting | `$82:E1B7` (`05F7`, `$84:8250`, sound 71h, `05F5`), `$82:E29E` |
| TheFades | `$82:E1E1`..`$82:E267` (the target colours; the HUD's groups `$82:E1F1`..`$82:E264`), `$82:E2DB`, `$82:E737`, `$84:8270` (her spin's sound), `$82:E757` (sounds enabled) |
| Fading | `$82:D961`, `$82:DA02`, `$82:DA4A`, `$82:DAA6` (the quotient through `$4204`/`$4206`/`$4214`) |
| Squaring | `$82:E2F7` (the arrival, `$82:DE12`; `LeaveRoom`), `$82:E310`, `$80:A3AB` (layer 2 and the BG scrolls each frame of it) |
| Building | `$82:E353` (`DrawTopRow` for `Up`, `$80:AD1D` → `$80:AF89`), `$82:E36E` (`BuildRoom`) |
| SettingOff | `$82:E38E`, `$80:AD30`..`$80:AE07` |
| Sliding | `$80:AE4E`..`$80:B031`, run from the IRQ's begin-HUD commands (`$80:979B`, `$80:97FC`) |
| Carrying | `$82:DE12` (the speed), `$80:AE82`, `$80:AEC6`, `$80:AF49`, `$80:AFCB` |
| Arriving | `$82:E3C0` (placing, `FetchTiles`), `$82:E4A9`..`$82:E524` (`FetchContents`: `$82:E4AA`..`$82:E4E0` and the library background), `$82:E526` (the wait), `$82:E52B` (the visor), `$82:E53C` (the PLM handler) |
| WhereSheLands | `$82:E3C5`..`$82:E3E2` (placing), `$82:E547`..`$82:E55F` (the snap), `$82:E6A2`..`$82:E6E8` (stepping clear; the bottom row's branch is unfinished: `CMP #$00E0` falls through) |
| Landing | `$82:E540`..`$82:E564`, `$82:E659` (`$87:8064`), `$82:E664` (`$80:8EF4`, `$82:E0D5`), `$82:E6A2` (`$90:F084`, the liquid) |
| SheKeepsHerMotion | the records (nothing in the passage writes her pose or speeds) |
| WhatRuns | `$A0:8EB6`/`$A0:8FD4`/`$A0:884D` in `$82:E1CA`, `$82:E29E`, `$82:E2E0` (not on the finishing call), `$82:E737` (with `$86:8104`, `$87:8064`); `$87:8064` in `$82:E659`; `$84:85B4` at `$82:E53C`; the timer drawn `$82:E292` after every function while `0943` ≠ 0 (so not after `$E4A9`'s first frame of work, which the routine's second ends) and `$82:E267` at game state Ah, skipped with the CRE bit |

## Findings behind the spec's choices

- **The phases are the player's**: the 15 functions are regrouped into eight phases, each
  delay counted from something a player sees (the view squared, the slide's end, the track's
  start), whose counts hold every function's frame of work, so every record and vector holds;
  the function that reads the doorway is `Squaring`'s first frame of work.
- **The fade's first step changes nothing**: `$82:DAA6` returns the current colour for a
  numerator of 0. So both fades spend a frame of work before anything moves; the records show it.
- **The slide runs in the IRQ**, not in the frames of work: the records step layer 1 by 4 every
  frame, overrun frames included. Its first step is in `SettingOff`'s frame of work; the IRQ
  command is set in the placing routine after `$82:DF69` waits for vblank's end, so the next
  step is two frames after `SettingOff`'s (rows 1 to 10: `0925` = 2 at `SettingOff` + 2). For a
  door up, the horizontal chain (16h) runs until the end of the placing (`$82:E49D`): still one
  step a frame.
- **The same IRQ commands finish every frame** from that step to `$E6A2`: they set the NMI
  request flag and zero the 8-bit counter, so the NMI uploads, reads the pad and counts every
  frame (2,893 of 2,893 frames; `05B5` is 1 after each).
- **Placing shows on an overrun frame**: the routine writes her position at its start, before
  the decompressions, so the record one frame after `SettingOff` holds her placed even when its
  cut is `Overrun`.
- **Where the camera last saw her** is `0B10`/`0B14`, which the placing and every slide step write;
  the snap and the nudge write only `0AF6`/`0AFA`.
- **The camera's subpixels** `090F`, `0913` are never written by the passage.
- **Kept colours**: the CRE test guards both the icon colours and the timer's (`BNE $E26B` skips
  both, and the timer's drawing); `chain-any-0006-E0B5` has the timer running and the bit set,
  and its timer colours fade.
- **Leaving also clears the save station's used flag** `1E75` (`$82:E1C7`): P2's.

## The harness (for the integrator)

- **Rows:** every cached row with game state 9: the eleven Ceres chain rows but the last, and
  `area-maridia`, `area-norfair`, `boss-kraid-defeated` (ends in `BeforeTheSlide(1)`),
  `samus-input-poses`, `chain-0014-96BA`, `chain-0097-ACF0` (ends in `Brightening`),
  `chain-0106-AD5E`, `chain-0224-D0B9`.
- **Decoded at the touch:** her position, the camera, `0B10`/`0B14`, the colours and targets.
  The phase is decoded from `0998`/`099C` (State).
- **Inputs per frame:** as `ref/model3.py` reads them (above): the music's readiness from the
  queue, never from the phase compared; the doorway's arrival from the importer's `Doorway`
  record for the touched door (the World records), which must equal `0791`/`0927`/`0929`/`092B`
  once taken.
- **Compared against the records** (the position/timing diagnostic is `model3.py`; the all-cut
  palette diagnostic is `passage-fade/native.py` below): every frame, her position,
  `0B10`/`0B14`, the camera, a finished frame (`05B6` steps) and the frame count (`05B5`); every
  frame of work, the phase and its data; every fade cut, logical `t`, coverage and
  `7EC000..7EC1FF` under the named-writer limits below; the silence (library 2's queue),
  sounds held (`05F5`), the mini-map (`05F7`), the enemies' runs (each slot's `0FA4`: none steps
  when they don't run), and the library-1 sound at the first and last frames of work as G6's rule
  picks it (samus-appearance's, until its spec owns it).
- **Not comparable on records, vectors only:** the cost event (T1 has no record of its own), the
  room's calls (R2's effects are compared in R2), the animated tiles' and the projectiles' runs,
  what is drawn and the timer's drawing (OAM is not in the lean records), and the slide's screen
  (`$212C` is not recorded).
- **Masked, with their writer:** colours 113 to 115 in `boss-kraid-defeated` (Kraid's corpse, at
  each enemy run); colours 41 to 46 in `chain-any-0010-DF8D` from its first `Brightening` frame of
  work (the Ceres door's elevator beacon, `$A6:F850` → `$A6:F86B`, written while the numerator
  `7EC400` is 0); colours the loading writes (sprite palettes 6 and 7, `$A0:8A1E`) are inputs on
  every `Arriving` frame, taken from the next record; the frame they change counts as unverified. Layer 2 and the BG scrolls are the backgrounds' and E2's.
- **Inject only the named writer's colours** on the `Arriving` frames: sprite palettes 6 and
  7, colours 224 to 255, not every colour that differs, so a fade error on that frame still shows
  (the model does so: 756 of 785 either way).

## The harness cases

Real content no vector may carry, loaded by the importer in the integrator's harness:

| Case | Row:frame | What | Checks |
| --- | --- | --- | --- |
| H1 | every passage, its first `Squaring` frame of work | the doorway's arrival from the World records | Squaring, SettingOff |
| H2 | every passage, its `Brightening` frames | the new room's colours as the room's loading set them | Brightening |
| H3 | every passage, its first frame of work | the HUD's colour groups and the rooms' CRE flags | TheFades |
| H4 | every frame of every passage | her pose, speeds and momentum | SheKeepsHerMotion |

**Recorded cases** behind the vectors (positions and colours moved to invented ones):

| Case | Row: touch frame | Way | Vectors |
| --- | --- | --- | --- |
| R1 | `chain-any-0001-DF45`: 250 | Right | touch, leaving, dark-items |
| R2 | `chain-any-0002-DF8D`: 120 | Right | dark-step, dark-last, dark-end, work-pending |
| R3 | `chain-any-0003-DFD7`: 182 | Right | set-off-right, place-right, place-pending, slide-right, slide-end, loading-done, land-right, clear-none, work-pending-slide |
| R4 | `chain-any-0004-E021`: 102 | Right | silence-sent, reading |
| R5 | `chain-any-0005-E06B`: 94 | Right | music-waits, music-ready, forced-wait |
| R6 | `chain-any-0006-E0B5`: 1610 | Left | dark-timer-dark |
| R7 | `chain-any-0007-E06B`: 96 | Left | bright-step, bright-end, bright-pending |
| R8 | `chain-any-0008-E021`: 100 | Left | silence-waits |
| R9 | `chain-any-0009-DFD7`: 169 | Left | settle |
| R10 | `chain-any-0010-DF8D`: 112 | Left | set-off-left, place-left, slide-left, land-left, clear-left |
| R11 | `area-maridia`: 23 | Left | loading-waits |
| R12 | `area-norfair`: 120 | Right | square-done |
| R13 | `boss-kraid-defeated`: 1090 | Right | building |
| R14 | `samus-input-poses`: 251 | Left | dark-first, bright-first |
| R15 | `chain-0014-96BA`: 222 | Right | prepare |
| R16 | `chain-0097-ACF0`: 663 | Right | square-up (14 frames of squaring) |
| R17 | `chain-0106-AD5E`: 482 | Left | square-down |
| R18 | `chain-0224-D0B9`: 201 | Right | touch-passing |

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on
samus-movement. There is no old code: the engine has no passage (its only game mode is
gameplay), so no area has `old` for the guideline comparison (MODEL §1 decision 19).

- Area passage: TheTouch, PassageTime, FinishedFrames, Quieting, TheFades, Fading, Squaring, Building, SettingOff, Sliding, Carrying, Arriving, WhereSheLands, Landing, SheKeepsHerMotion, WhatRuns

Budget: 37.5 × 77.7 units = 2,912 (passage 2,912)

## First Quieting production boundary

The original first-entry order is `$82:E1C4` mini-map, `E1C7` last save use,
`E1CA`/`E1CE` enemy selection and run, `E1D2` normal layered drawing, `E1D6`
inanimate body drawing, `E1D9` source/destination CRE flags, `E1E1`..`E264` target
colours, conditional `E267` timer drawing, `E26B` Samus sound selection, `E272`
room silence, `E279` holding requests. This source order is not automatically a spec
ordering rule. Request-before-hold is observable: the queue admission checks `05F5`.
The enemy run can write screen colours; target colours must use the resulting colours.
Independent mini-map and save-use writes can be applied anywhere before the compared cut.

### Drawing contract

`WhatRuns`'s `drawn` includes both normal layered drawing and the extra inanimate Samus
body, not just one call to the gameplay drawing helper. `$A0:884D` invokes `$90:EB35`
at layer3 (`$A0:8871`), draws its layer queues, then the graphics-drawn hook (`$A0:88BD`).
Only afterwards does `$82:DFC7` invoke `$90:8A00` (unless the door is an elevator).
The extra draw appends the same frozen pose/frame's body and queues its tiles; it neither
runs her animation/movement nor changes her drawing handler. Its OAM ordering matters.
It does not draw the cannon, atmospheric effects or trails a second time.

Normal `$90:EB35` invokes cannon state/drawing, projectile drawing (`$93:8254`),
auto-jump/hurt/input-history/previous-health maintenance (`$90:EAB3`) and sound/time-up
maintenance (`$90:F576`). These are the neighbour's drawing obligations, not movement.
The spec's unchanged pose/speed/momentum rule does not license skipping these writers.

Projectile drawing ends at `$93:82F7` with `$90:B6A9`: existing trails age and draw even
while PRE is paused. No new trail spawns in this pass: the spawn (`$90:B657`) is produced
by projectile execution (`$90:B0FC` for the ordinary beam), which `WhatRuns` pauses.
For checked ordinary power content `EndsImmediately`, a validated retained-empty trail
pool gives a real zero-request drawing receipt. Do not manufacture a PRE receipt or a
history append just to satisfy a helper. Normal layer3 input history is still updated.
A draw helper may share body/tiles without temporarily changing her drawing handler.

M0 records 182->183 and 183->184 independently keep animation words `0A94..0A97`,
cannon state `0AA6..0AAC`, auto-jump `0AF4`, hurt flash `0A48`, previous health `0A12`,
and trail memory `D658..D7BF` unchanged. These observations do not prove all drawing writers
are inactive for arbitrary restored State. Admission must validate unsupported neighbour
features or the production neighbour must supply them. OAM must compare whole.

### Inputs and retained state

Source/destination items-dark are room-header content, not selected room-version state:
`$82:DDF1` copies current CRE `07B3` to previous `07B1`, reads destination header byte8,
then uses `(07B1 | 07B3) & 1`. Import both rooms' flags from the player's ROM at the edge;
never derive them from a predicted passage phase. B8 escape-running is `0943 != 0`.
M0 first entry has both room flags false and escape inactive; admission validates actual
native seed and imported room content, not unvalidated constants.

Last-save-use `1E75` is real retained behaviour state: `$84:B5A2` reads it to prevent repeat
use, and `$84:B03A` sets it. It is not an unobserved hardware leftover. The first entry clears
it. M0 has it inactive before/after entry. A bounded inactive-state capability needs actual
validation at seed/restoration; supporting active use needs P2's typed owner and reset event.

### Timing and prefix limitation

M0 `chain-any-0003-DFD7`: frame182 Requested is state9/E17D; frame183 Requested is
stateBh/E29E; frame184 Requested is stateBh/E2DB with fade numerator0. First-entry effects
all complete by 183; last timing admits work and next-cut timing certifies completion for
this Requested first entry. Completion-only execution cannot model the fade's partial writes:
WRAM palette hashes differ at Overrun 187 and 188 before Requested 189 advances its numerator.
Do not weaken the existing frames-of-work harness into a complete-cut claim.

Audio is required from the native seed, not first introduced at the door. M0 cut0 already
holds a library3 outstanding acknowledgment; it clears/re-arms through 2..6. There are 78 cuts
with audio state changes through183, including pending backlog at 81..84. A last pre-door sound
at 174 clears/re-arms through 176..180. Request admission and sending alone can check the first entry in isolation but cannot
produce this native prefix without the earlier request and acknowledgment owners. No production readiness may be injected at 182.

Independent decoder: `$TEMP/collab/night-20261115/observer/first_cut.py`.

## Every fade cut: palette progress and retained work

The native reference is `$TEMP/collab/night-20261115/passage-fade/ref/passage.py`, vectors
`ref/vecrun2.py`, and the native diagnostic `passage-fade/native.py`. These read the pinned lean
records at `$TEMP/collab/night-20261113/recovered-cache-20261114/rows`; no extracted palette
content is committed. A colour is written atomically by `$82:DA3A`, after its three components
have been calculated (`$82:DA4A` through `$82:DAF6`). The loop treats screen colours in ascending
physical palette order (`$82:DA1E`..`$82:DA44`). A current colour equal to its target still counts
as treated; no write is needed. A cut within the component arithmetic shows the old whole
colour, never a partly changed channel.

### Independent progress decoding

`7EC404 / 2` identifies the currently treated colour, not necessarily the count completed.
At an interruption inside `$82:DA4A`..`$82:DAF6`, or before `$82:DA3A`, completed coverage is
that colour's index. At `$82:DA3D`..`$82:DA42`, the colour was treated: add one. At `$82:DA21`
the store of the new cursor has not occurred: use CPU X/2, not the stale `7EC404`. At
`$82:DA44`..`$82:DA49`, all256 have been treated. Decode uses source position and registers,
never colour equality or expected colour bytes. Neighbour work before palette treatment has
coverage0; neighbour work afterwards has coverage256. Each completion cut certifies256.

The original increments `7EC400` at `$82:DA44`, before the frame's remaining work finishes.
The model's phase `t` instead stays the current work step until total completion; the decoder
retains that logical step across overrun cuts and interprets an advanced original numerator
as palette-complete work still in progress. `Darkening` runs enemy/drawing work after the fade
(`$82:E2E0`..`$82:E2EC`); `Brightening` runs tiles, enemies, projectiles and drawing before it
(`$82:E737`..`$82:E752`). These writer instants must be supplied by the real neighbour owners.
No original call boundary is an implementer-facing type or ordering requirement.

M0 `chain-any-0003-DFD7` has these independently decoded cuts:

| Cut | Kind | Current logical step | Colours treated | Work complete |
| --- | --- | --- | --- | --- |
| 185 | Overrun | 0 | 219 | no |
| 186 | Requested | 0 | 256 | yes |
| 187 | Overrun | 1 | 101 | no |
| 188 | Overrun | 1 | 219 | no |
| 189 | Requested | 1 | 256 | yes |
| 190 | Overrun | 2 | 102 | no |
| 191 | Overrun | 2 | 220 | no |
| 192 | Requested | 2 | 256 | yes |

The numerator's zero step treats colours without changing them. At187 the first101 have
changed; at188 the next118 have changed without applying the first101 again. Completing at189
changes only the remaining37, then advances logical `t` to2. The frame is unfinished at187/188.
Coverage256 is valid retained state while neighbouring work remains; completion clears it.

### Native evidence and diagnostic limits

The palette check covers1383 fade cuts, including868 overruns:842 interrupt colour treatment
and26 interrupt other work. It agrees1383/1383 against a per-cut do-nothing170/1383 under the exact
writer exclusions below. There are353282 compared colour-cuts and766 excluded colour-cuts.
This is an observer diagnostic, not an engine ratchet, floor or whole-engine gain.
It carries its own predicted colours within fade segments except `boss-kraid-defeated`, whose missing
owner is diagnosed per cut using the preceding native palette. No arbitrary differing colour
is injected. M0 carries its predicted palette separately within each of its two fade segments:80/80 cuts
with every colour compared. The corresponding carried null matches4/80; the per-cut no-op
matches8/80. Both carried palettes start from exactly the same native palette at each fade
segment's entrance and receive no palette resets within that segment. The null keeps those
colours unchanged, while the model applies fade treatment. The room-loading gap between the
two segments is outside this comparison and supplies the next segment's starting palette to
both equally. Work step, completion and progress remain observed timing inputs: this proves
palette carry within fade segments, not a whole-R1 State free run. The native model does not produce T1's elapsed costs,
U1's replies, neighbouring OAM or their timings, and does not prove the whole passage's outputs.

The Kraid exclusion of113..115 remains the corpse flash writer. The additional full-group
writer is `$A7:AC24` -> `$A7:B337` -> `$A7:B394`, with stores `$A7:B3BE` and `$A7:B3C5`:
BG palette7 colours112..127 and sprite palette7 colours240..255. At cuts1094,1097,1100,1103,1106,
1109,1112,1115,1118,1121,1124,1127,1130,1131 of `boss-kraid-defeated`, retained Kraid health
`0F8C` is0 before and after. Independent dispatcher evidence at every window: slot0 is in
`17AC`'s active list, `0F78` names Kraid, `0FA6` is its bank, `0F8A` selects main AI,
`0A78` and `185E` leave time unfrozen, `0F88` does not paralyse it, and `0FA4` advances once.
`$A0:9002`/`$A0:9018` select that active slot/bank; `$A0:905A`..`$A0:9088` select its header's
main pointer; `$A0:90A3` dispatches it. The Kraid header's main pointer is `$A7:AC21`.
`$A7:AC24` calls the palette handler unconditionally before the current death function.
`$A7:B33F` takes the dead case and the palette copy executes. The windows are selected by
these source execution conditions, never by palette mismatches. At1094, t0, the palette copy
still executes although its outputs already match: equal writes are not a reason to omit it.
Only those14 writer cuts get the expanded mask in the per-cut diagnostic. Source writes every
colour in both groups, including equal colours. All other colours remain compared.

With just the narrow exclusions the corrected progress decoder agrees1370/1383 (353688
compared and360 excluded colour-cuts; per-cut do-nothing164/1383). The extra14 exclusions account for
406 fewer comparisons, not model gain. The missing real Kraid palette owner prevents carrying
that row through later unmasked cuts; a continuous harness must obtain those colours from its
owner rather than re-seed from the record. The Ceres beacon remains colours41..46 on its
`Brightening` cuts, source `$A6:F850` -> `$A6:F86B`. No masks or floors were changed in code.

### Constructed progress checks and actual faults

All94 vectors pass, preserving the70 existing cases. The16 palette progress cases cover starting,
continuing, no progress, completing after partial work, all colours done while work continues,
both fade directions, zero/goal/final steps, last colour and adjusted steps. Hand arithmetic:
with d12,t1, channel31 toward0 becomes28; channel0 toward31 becomes2. With d2,t1,31 becomes15.
At t=d+1 the colour is exactly the goal; t0 and t=d+2 change nothing. Physical slots at100/101,
218/219 and255 make off-by-one coverage observable. Existing Parameter coverage remains complete.

Actual local reference faults fail: suppressing unfinished writes catches `fade-start-overrun`'s
colours and retained coverage; restarting treatment from colour0 catches
`fade-continue-overrun`'s already-treated prefix; advancing the work step early catches
`fade-colours-done-pending`'s retained t. The native repeated-prefix fault first fails M0's palette at188 and reduces its match to14/80.
The native completion-only fault first fails M0's
palette at187 and reduces its match from80/80 to8/80. Faults live only in local Python and are
not committed. Full Cargo gate is deferred until the root grants the shared build slot.

### Retained semantic neighbour work

Colour coverage alone does not show whether Brightening's neighbours have been run before
colour0. The aggregate status in the spec separates unstarted, partial and completed work,
even at coverage0 or256. Each owner must retain its own partial continuation State; the
aggregate certifies every required owner, not an original routine boundary. Source ordering
above proves the required pre/post-palette placement of neighbour writes, but does not supply
real T1 durations or owners' continuation models. The native palette diagnostic does not
exercise these new work receipts; it remains observed timing plus palette carry only.

Eight constructed receipt cases add: unstarted at0, first start at0, restored underway at0,
restored complete at0, completion of underway work at0, and Darkening's post-palette start,
resume and completion at256. `ref/vecrun2.py` compares start versus resume separately;
`ref/passage.py` requires explicit aggregate receipts, never defaults completion from `done`.
A real local repeat-start fault (`repeat_start`) fails restored underway and completed cases,
including `neighbours-resume-at-zero`, `neighbours-done-at-zero` and `neighbours-dark-complete`.
The final Darkening step has no enemy/layered-drawing obligation; B8's timer overlay remains
its separately retained work when active, and T1 cannot certify total work before its real
completion. The constructed model does not pretend to certify B8's partial overlay execution.
