# How Samus looks, part 1, her animation's timing: decode notes

For the observer, the adversary, the integrator and the root only: implementers never see this
file. It holds the sources by entry, the RAM mapping, the importer's format, the harness note and
the Budget section of [samus-appearance.md](samus-appearance.md). Bank logs:
<https://patrickjohnston.org/bank/> (banks `$90`, `$91`, `$80`).

## How the spec was made

- **Watched first.** A dump of the 44 cached rows (`$TEMP/collab/scenes/cache-v2`, a cdylib over
  `oracle_record`, scratch `$TEMP/collab/looks-spec/dump`) printed, per record, `0A94` (the time
  left), `0A96` (the step), `0B3E` (the boost), the pose `0A1C`/`0A1E`, the old pose
  `0A20`/`0A22`, `0A28`/`0A2C`/`0A32`, `0A9C`, `0B3C`, `09A2`, `09C2`, the vertical speed and
  direction, the pad `8B`/`8F`, the new state and movement handlers `0A44`/`0A58`, the pose input
  handler `0A60`, the FX, the frame count `05B6`, the game state `0998`, the sound queue indices
  `0643`..`0648` and `0A68`. Frame by frame the time left counts down, the step moves on when it
  runs out, a pose change resets both, and a run with momentum alternates at another pace; the
  routines settled the exact holds, the order and the edge cases.
- **A faithful model of the routines** (`orig.py`, over the original's frame numbers) matched
  17,638 of 17,826 frames before the out-of-scope writers were skipped; every remaining frame was
  the x-ray scope (poses D5h, D6h, D9h, DAh), the grapple (B2h, B3h), the shinespark (C7h-CEh and
  its counter writes) or the crouch's shine.
- **The reference model** (`model.py`, over the spec's format only) is run through an importer of
  the delay lists parsed from the bank log text (`importer.py`, `clips.py`) and the pose map
  (`poses.py`, from `data::assets::samus::map`). Per frame over the cache
  (`cachecheck.py`): **18,233 of 18,233** frames of play in the domain match on all three words
  (`0A94`, `0A96` through the step map, `0B3E`); the null model (nothing changes) matches 506.
  On the vectors (`vec.py`): 98 of 98, the seven with adjusted rules included.
- The scripts are in `$TEMP/collab/looks-spec/`; `cases.py` lists the recorded frames per rule.

## State

| Spec quantity | RAM | Decode |
| --- | --- | --- |
| her step | `0A96` | the importer's step map of her pose's clip (§The importer): our step of the original's frame number. A frame number on an operand byte has no step: a harness defect (none on record) |
| the time left | `0A94` | as is: the frames left on the step is the quantity itself (RAM-shape: one word copied as is, because the count the player sees is that word) |
| the boost stage | `0B3F` (the high byte of `0B3E`) | as is, 0 to 4 |
| the laps left | `0B3E` low byte | as is (the speed boost timer: the laps the stage still lasts) |
| her kept action | `0A11` (the movement byte of `0A10`) | through the movement-type map to her action; written only by the preparation, `$90:EB2D`/`$90:EB30` (§The prepared movement snapshot) |

The rest of her appearance State is the cannon delay (`0AAA`, §Cannon selection readiness) and
the three colour-sequence quantities (`0A72`, `0B62`, §Ordinary dry palette preparation).

**At initialization** (spec §1.6): `$91:E00D` (Initialise Samus, for a demo's loading and the
Ceres and Zebes starts) clears `$0A02..0E0B` at `$91:E01F..E026`, which holds `0A94`, `0A96`,
`0B3E`, `0B62`, `0A10` and `0AAA`, then stores `0601h` in `0A72` at `$91:E0BF..E0C2` (delay 1,
entry 3) on every path, and zeroes the pose `0A1C` and movement word `0A1E` at
`$91:E12F..E132`. So step 0, time left 0, stage 0, laps 0, kept action movement type 0
(standing: `Still`), cannon delay `2 - 0` = 2, visor delay 1 at entry 3, charge glow reset.
The clear covers every quantity of her appearance State, and the routine is one call from the
loading of a game or a demo, ahead of any of her work that frame, so the spec makes her
initialization a part of the frame that sets the whole State, first in its frame (spec §1.3,
§1.6; `C-initialize`), not only the cannon delay that `CannonReadinessReset` names.
Demo data loading (`$91:8790`, after the clear) sets items, energy and the demo input, and its
demo setup the pose: other domains' (spec §7).

`0A9A` (the new pose's frame) and `0A9C` (the frame buffer, rewritten by the FX routine at the
start of every animation step, `$90:8078`) are not State: they never carry from one frame to the
next. `0A9A` is cleared at every frame's start (`$90:E6C3` → `$90:EB02`, `$90:EB1D`) and after a
pose change's start (`$91:F422`), which the super special path (`$91:EBAF`) skips, so at a frame's
end it holds what the pose's initialisation wrote on a hand-over. No rule reads the value a
frame leaves (every reader runs in the pose change, after the same frame's write), so it is not
compared (ARCHITECTURE §11.6). For the record, it is 1 when the frame handed over (`handed
over`), her pose changed and her step after is 1 (a `StartingClip` row from step 1), else 0: over
the cache's 23,432 `R` cuts in game state 8 that is right on every one, 346 hand-overs, one of
them from step 1 (chain-0314-DAE1 139 → 140, 56h → 16h).
The super special request, `0A2C` (its pose) and `0A32` (its command), is not State either: no
rule reads the value a frame leaves, so, as with `0A9A`, both words are out
of State, the RAM translation and the lockstep comparator (ARCHITECTURE §11.6). A clip's end
writes it (`$90:84B6`, command 3, "transition animation finished"). Its only readers are the
pose change (`$91:EB8E`, `$91:EB94`, `$91:EBBB`) and the hit interruption's test of command 3
(`$90:DE30`), both in the new state handler after the animation (`$90:E725`: `$90:E73B`
animate, `$90:E73E`, `$90:E745`; the demos' `$90:E7F5` and `$90:E833` alike). The main loop runs
the current state handler before it (`$82:8B58`, then `$82:8B69`), and that handler resets the
request to none and 0 before anything reads it (`$90:E6A4`, `$90:E6AD` normal; `$90:E6D8`,
`$90:E6E1` the demos'). The Samus command dispatcher (`$90:F084`) clears both too (`$90:F09E`,
`$90:F0A7`) whenever a command returns carry set, as every unlock does, so a lock lifted within
a frame hands on no request either. No other bank reads either word (every bank log searched for `$0A2C`,
`$0A32` and their long forms: bank `$9B`'s grapple writes them, and a reader is in `$90` and
`$91` only). The locked handler (`$90:E713`) doesn't reset it, but every new state handler set
with it runs no reader (`$90:E8DC`, `$90:E8D6`, `$90:E8D9`, RTL), save the demo's end
(`$91:8739`), which keeps the title demo's handler with her standing, a clip with no end, outside
game state 8. Until the integrator takes them out, lockstep compares both words, so the RAM
translation writes them from the frame's outputs: on a frame that
handed over (`handed over`), `0A2C` the engine's pose id of the pose she took in the plain sprite
(the end's own pose, before `HowSheIsDrawn` dresses it: chain-0232-D5A7 23 → 24 hands over 1Ah
and she shows 82h, the screw attack's) and `0A32` 3; on any other frame of hers FFFFh and 0; and
on a frame hers doesn't run (a message box's, the frame it closes) as the last frame left them.
Over the cache's cuts in the domain's poses: 341 hand-overs, 17,881 other frames and 3,512 carried
are right; 3 others hold command 1 with her pose, the knockback's end (`$91:F31D`'s writer, out of
the domain: boss-kraid-defeated 570, chain-0059-A293 399, chain-0106-AD5E 111). item-morph-ball's
4 hand-overs are at 687, 1265, 1310 and 1312.

`0A9C` is the liquids' (0 out of them). `0B40` (the echo sound flag, set with the
boosting sound at `$90:85A0`) is the echoes' (part 2).

## Inputs

All decoded at the animation's turn in frame N → N+1 (the records are quiescent cuts; a frame is
record N to the next `R` record):

| Input | Decode |
| --- | --- |
| her pose, her pose after | `0A1C` at N and N+1, through the pose map (key and dress) |
| her clips, the paces | the importer (§The importer) |
| energy | `09C2` at N: the animation reads it before the frame's damage (the periodic damage `$90:E74D` follows `Animate Samus` in `$90:E725`, and the enemies' contact damage comes later in the frame). 4 cuts cross 30 within a frame, none at a breathe step |
| items switched on | `09A2` at N+1: bits 2000h, 0200h, 0008h |
| submerged | `$90:841D`'s test: gravity suit `09A2` & 20h clear, and the FX surface `195E` (non-negative, below the top boundary from `$90:EC58`, liquid physics on: `197E` & 4 clear) or the lava/acid surface `1962`. Never true in the cache |
| the step slowdown | `0A9C` at N+1, the animation's own buffer: `$90:8078` the X speed divisor `0A66` (0 out of liquids); `$90:80B8` water: 3 (`$90:80CB`) with no gravity suit test; acid 2 (`$90:8219` → `$90:824C`, `$90:9E95`) with no gravity suit test either; lava (`$90:81C0`) 2, and 0 with the gravity suit (`$90:81DB` → `$90:820F`). FX types 8, Ah and Ch run `$90:8077` (RTS) and leave it as it was: the liquids' domain must carry it |
| the start slowdown | `$91:FB08`'s `$12`, worked out again for the pose she takes: the gravity suit gives `0A66` (`$91:FB0E` → `$91:FB3F`); otherwise the FX surface `195E` (liquid physics on) or the lava/acid surface `1962` against the **new pose's** bottom boundary (`$91:FB1D`..`$91:FB3D`): 3 in water (`$90:9E93`), 2 in lava or acid (`$90:9E95`). In the cache both are 0 on every frame of play, so `cachecheck.py` takes `0A9C` for it |
| run held | pad `8B` at N & the run binding `09B6` |
| handed over | `0A32` = 3 at N+1 with `0A2C` a pose (bit 15 clear): the super special "transition animation finished" took the frame's pose. On the 311 end frames on record it agrees with "an end, or an end unless a jump with `0A28` at N+1 not 4Bh, 4Ch, 19h or 1Ah outside the demo's `0A60` = E91Dh" (the 8 that gave way), and it is false on every other frame |
| the dash | from samus-movement's `dashing after her moves`, `dash began` and `dash lost` (its model: `dash_began` when the dash was off at the frame's start and her moves ran one, `dash_lost` when it was on or ran and is off at the end): off, on, began (began, dashing), lost (not began, dashing, lost), began and lost (began, lost), stopped (not dashing after her moves, lost). From the records alone: on when `0B3C` ≠ 0 at N and N+1; began when 0 at N and non-zero at N+1 (`$90:9796`, `$90:97CC`, in the movement handler, before the animation); lost when non-zero at N and 0 at N+1 (`$91:DE53`; also `$91:E655`, the pause screen's equipment change). Began and lost, and stopped, leave no trace in `0B3C` at the cuts (§Findings, "The dash's six values") |
| standing restarts | movement type 0 at N, pose 1 or 2, pad `8B` at N & the shoot binding `09B2` (`$90:A3AD`); samus-movement's `StandingAnimation::Restarts` |
| wall jump readied | samus-movement's `WallTest::Readied`. In `cachecheck.py`, from the records: the pose unchanged, a kick step, `0A96` at N+1 on it and at N neither on it nor one before (the kick step is reached no other way) |
| vertical speed | samus-movement's `vertical speed after her moves` where its model runs the frame; elsewhere `0B2E`.`0B2C` at N+1 read as signed, then negated when `0B36` = 1 (up), so a rise the pull took past zero (`FFFF.F400` rising, samus-ceres-push 989) is the small `+0000.0C00`, as samus-movement gives it and as `BMI` reads it. The original reads `0B2E` alone at `$90:A5AC` (`CMP #5`, `BMI`), right after the falling movement (`$90:A590` → `$90:9168`) and before the pose change: `0B2E`.`0B2C` holds the size and `0B36` the direction, so the test is the size's, whichever way she moves (root decision a) |
| heading down | samus-movement's `heading down` where its model runs the frame; elsewhere `0B36` = 2 at N+1. The two differ on a landing (as her moves leave her she heads down; N+1 shows none), where the landing's clip starts either way |
| sounds waiting | `(0648 low − 0645) & 0Fh` at N: library 3's queue, `$80:9158`. Read every frame from the sound domain's queue; not appearance State, no starting value (the engine's bridge gives 0 until the sound domain supplies it, gaps file) |
| cinematic, boss, special colours active | `1F51`, `179C`, `0A68` non-zero, as `FootContact` reads them (`$90:A404..A419`); `1F51` also for `SpinStarts` |
| shoot held, charge | `008B & 09B2`, `0CD0` (`AtTheDoor`, `$90:F471..F4A1`) |
| the frame's parts | her movement: the new state handler's normal, demo or elevator handler runs (§The harness). Preparation: the current state handler reaches `$90:EB02` (`$90:E6C3`, `$90:E70D`, `$90:E71C`). Palette opportunity: the pose-update PC `91eb88` hits (§Actions the opening reaches). First passage entry: `$84:8250`. Normal drawing: `$90:EB35` |

Which inputs samus-movement's model doesn't yet report is §Requests to samus-movement.

## Sources per entry

| Entry | Source |
| --- | --- |
| ClipPlays | `$90:801E`..`$90:802D` (the countdown: `DEC`, then `BEQ`/`BPL`, so 0 or below moves on), `$90:82DC` (the list `$91:B010` by pose, the byte at the frame), `$90:84E3`..`$90:8526` (a delay byte plus the buffer `0A9C`) |
| ClipControls | `$90:8324`'s table; `$90:84C7` (FEh nn: back nn bytes), `$90:84DB` (FFh), `$90:8344` (F0h-F5h: carry clear, the timer left at the countdown's 0); `$90:8315` (carry set: the timer from the byte at the new frame plus the buffer) |
| Breathing | `$90:8346` (F6h: `09C2` ≥ 1Eh, signed `BMI`); `$90:8315`; lists `$91:B298` (poses 1, 2) and `$91:B2A3` (27h, 28h) |
| ClipEnds | `$90:84B6` (FDh), `$90:839A` (F9h), `$90:83F6` (FAh, unused), `$90:848B` (FCh, unused): each sets `0A2C` and command 3, carry clear; `$90:8370` (F8h: not in the demo `E91D`, and `0A28` a ground jump, returns carry clear with no request; else installs the auto-jump handler `$90:E926`, samus-movement's, and jumps to `$90:84B6`); `$91:EB88`/`$91:EBAF` (the super special pose wins the frame) |
| WallJumpBranch | `$90:841D`..`$90:8489` (+1, +0Bh, +15h from the FBh byte; sounds 31h, 3Eh, 33h on library 1, `$80:9049`); `$90:8315` |
| HandOverSlowdown | `$91:EBA4`..`$91:EBC0` then `$91:F36E` (command 3: `0A94` += `0A9C`), after `$91:FB08` set the first hold with its own `$12` (the start's slowdown) |
| StartingClip | `$91:FB08`..`$91:FB8C` (pose = old pose: nothing; `0A9A` bit 15: kept; else frame and its byte plus the start's slowdown `$12`, `$91:FB0E`..`$91:FB56`), with `0A9A` from the pose's initialisation `$91:F468`'s table `$91:F4A2`: `$91:F4DC` (standing, shot direction up 0 or 9 from a previous pose's up: 1), `$91:F50C` (running from running: 8000h), `$91:F5AF` (15h/16h from 55h/56h: 1), `$91:F5EB` (85h/86h from F1h/F2h: 1), `$91:F624` (spin from spin or wall jump: 1), `$91:F9F4` (ball from types 4, 8: 8000h), `$91:FA56` (spring ball from 11h-13h: 8000h); the shot directions `$91:B629` byte 3 |
| StandingRestart | `$90:A3AD`..`$90:A3C7` (in `$90:A383`: poses 1, 2 with shoot held, `0A94` = 10h, `0A96` = 0), then the countdown `$90:801E` the same frame: the spec's restart's frames are 10h − 1 = 15 |
| WallJumpReady | `$90:9DB2`..`$90:9DD7` (in the wall jump check `$90:9D35`: `0A94` = 1, `0A96` = 0Ah, or 1Ah in 81h/82h), then the countdown `$90:8021` |
| SpinTurnedFar | `$90:9D55`..`$90:9D68` (`0A96` − 0Bh, or − 1Bh in 81h/82h, `BMI`), inside the wall jump check `$90:9D35`, in the movement of the frame that reads it, from `0A96` and the pose `0A1C` as that frame starts (§Findings, "Turned far from the frame's start"); the lists `$91:B384`, `$91:B391`, `$91:B39E`, `$91:B491` |
| FastFall | `$90:A5AC`..`$90:A5C5` (in `$90:A58D`, poses 29h, 2Ah, 67h, 68h: `0B2E` ≥ 5 signed, `0A96` < 5 signed: `0A94` = 8, `0A96` = 5), then the countdown the same frame: the fast fall's frames are 8 − 1 = 7; lists `$91:B34A`, `$91:B353` |
| RisingHold | `$90:8011`..`$90:8050` (poses 4Dh, 4Eh: `0B36` ≠ 2, `0A96` = 1 and `0A94` = 1 → `0A94` = 4, then the countdown): the rising frames are 4 − 1 = 3; list `$91:B326` |
| RunPace | `$90:84EF`..`$90:851D` (`0B3C` ≠ 0, movement type 1: the delay from `$91:B5D3`, or `$91:B5DE`'s list for `0B3F` with the booster); `$90:82FB` (every byte with bit 80h, so every control, goes to `$90:852C`: any control ends a lap); `$90:8538`..`$90:857B` (an instruction byte with run held, no booster: frame 0, `$91:B5D3`'s first byte) |
| BoostStages | `$90:857D`..`$90:85D8` (the word `DEC` of `0B3E`, `BIT #$00FF`; counter + 100h unless bit 0400h; `$91:B61F` the timers 1, 1, 1, 1, 2; frame 0 and the stage's first byte); `$90:9796`..`$90:97A6` (begun with the booster: `0B3E` = 0001h) and `$90:97CC`..`$90:97D2` (without: 0); `$91:DE53` (cancel: `0B3E` = 0 when `0B3C` ≠ 0); the boosting sound 3, library 3, `$80:914D` |
| CountedLapHold | `$90:8587` (the boost timer still non-zero after its `DEC`: `BNE` to `$90:85DA`, which returns the control byte), then `$90:830B`..`$90:8315`: the control runs from the dispatch table, and a control that returns carry set takes its hold from the delay list `[$00],y` plus `0A9C`, not from `$90:84E3`'s pace; `$90:8344` (F0h-F5h) leaves the timer at 0 |
| BoostingLap | `$90:85A6`..`$90:85B6`: the `JSL $80:914D` returns A from `$80:9155`, whose high byte (after `XBA` at `$90:85AA`) is library 3's queue count before the sound (`$80:9158`..`$80:9161`), or 3 when 6 or more wait (the early return keeps the sound in the high byte); that index picks both `$91:B61F` and `$91:B5DE` |

`scan.sh --mirror` on the spec and these notes: 0 of 15 entries single-routine (§Scan).

## The importer: clips from the delay lists

The importer (outside the clean room) turns the animation delay table `$91:B010` (one pointer per
pose) into the spec's clips, keyed by the pose map's key and dress:

| Original byte | Step | Size |
| --- | --- | --- |
| 00h-7Fh | a shown step, hold = the byte | 1 |
| F0h-F5h | `rest` | 1 |
| F6h | `breathe` | 1 |
| F7h | (drained: out of scope, the importer refuses it in the domain's poses) | 1 |
| F8h pp | `end-unless-jump` | 2 |
| F9h eeee gg aa GG AA, FAh gg aa, FCh eeee pp PP, FDh pp | `end` (the pose is samus-movement's `AnimationEnds`) | 7, 3, 5, 2 |
| FBh | `branch x y z`: the steps of the bytes 1, 0Bh and 15h after it | 1 |
| FEh nn | `again s`: the step of the byte nn before it | 2 |
| FFh | `again 0` | 1 |

Steps are numbered without the operand bytes, so the step map (step ↔ the original's frame
number) is the importer's: in the spin `$91:B384` the original's frames 0-12 are steps 0-9, then
10 (the kick, frame 0Bh) and 11. Marks: `kick` = the step of frame 0Bh in poses 19h, 1Ah, 1Bh,
1Ch, 83h, 84h and of 1Bh in 81h, 82h (in the wall jump `$91:B491` frame 0Bh is the spin
sequence's FEh, so its kick step is an `again`: readying there sets frame 0Ah and loops back to
frame 3, which is C-ready-walljump's case); `fast_fall` = the step of frame 5 in 29h, 2Ah, 67h,
68h; `rising` = step 1 in 4Dh, 4Eh. The paces: `$91:B5D3` (plain) and `$91:B5E8`, `$91:B5F3`,
`$91:B5FE`, `$91:B609`, `$91:B614` (stages 0 to 4), 10 holds each, the running clips' 10 shown
steps. Of the domain's 164 poses, the 78 whose clips end (an `end` or `end-unless-jump`) have no
kick, fast fall or rising mark and none runs, stands unaimed or is in a kept row (spec §1.5:
the end follows from the frame's start; checked in the scratch, 0 of 78 break it). Every `again`
and `branch` in the 253 lists lands on a delay byte (checked in
`importer.py`). The laps per stage, `$91:B61F`, are the spec's parameter, not content.

## Findings behind the spec's choices

- **Two slowdowns** (adversary 89, finding 1). The clip moving on adds `0A9C`, which water sets to
  3 whatever her suit; the pose's start adds `$91:FB08`'s `$12`, which is 0 with the gravity suit
  and otherwise tests the surface against the feet of the pose she takes. So in water with the
  gravity suit a started clip is `hold + 0` and a handed-over one `hold + 0 + 3`, and at a surface
  the two differ by whose feet are under it. The cache can't show it (`0A9C` is 0 on every
  frame): C-slow-*.
- **The fast fall reads the size** (root decision a). `0B2E` holds the vertical speed's size and
  `0B36` its direction, and `$90:A5AC` tests `0B2E` alone, so a fall's pose rising at 5 px a frame
  or more shows the fast fall picture: an unmorph's end that hands over the crouch in a rise, then
  a walk off a ledge into a fall's pose, carries the rise (samus-movement's `FollowingTheGround`).
  No record rises so fast in a fall's pose (the fastest 2.4000): C-fast-fall-rising.
- **The end before the pose** (adversary 89, finding 2; adversary 90, finding 1; the root's
  decisions). samus-movement's step reads `animation_ends` before it chooses her pose, and this
  domain reads the pose chosen. The spec restricts its inputs (§1.5) so that the end is a function
  of the frame's start (step, time left, clip); all 78 ending clips of the game allow it. The
  output is the end's kind (none, an end, an end unless a jump), not a `bool`: whether an end
  unless a jump hands over is samus-movement's, by its own pad chart (`$90:8370`'s test of `0A28`,
  which `$91:81EC` writes from the transition table before the movement; the collision writers
  `$91:E8B6`..`$91:EADD` never write 4Bh, 4Ch, 19h or 1Ah). The second half reads back whether
  it did (`handed over`), for `HandOverSlowdown`. The frame is then two calls: the end first, the
  rest with samus-movement's results (spec §6). Over the cache the end's kind from the frame's start
  is the original's on all 350 end frames (151 `end`, 199 `end-unless-jump`, 9 of them given way),
  each with a pose change, and none elsewhere.

- **Turned far from the frame's start** (adversary 101, F3). The original works it out in the
  wall jump check (`$90:9D55`), inside the movement handler, before the frame's animation: from
  `0A96` and the pose `0A1C` as they stand then. None of the domain's rules moves either before
  the movement (the animation and the pose change follow it), so they are the State and her
  pose at the frame's start, and the spec gives it ahead, as it gives the end. The step and clip
  a frame ends on, read the next frame, are the same values; adversary 101's prototype of the
  ahead read held the ratchet, the gate and every harness row. The compare is by step number
  (`BMI` on `0A96` − kick), so a later branch's steps count as past the kick
  (C-far-later-branch): in the wall jump `$91:B491` the space jump's and the screw attack's
  frames follow the spin's FEh at 0Bh.
- **The report's end** (adversary 101, F4). With the end and turned far both given ahead from
  the State and her pose's clip, the frame's own report needs neither: the spec's outputs are
  the State, the sound it asks for and the two values read ahead (spec §1.1), and the vectors
  hold the two apart (`ahead`).
- **One adjusted vector per parameter** (Jim, 2026-11-11). The domain's `Parameter` entries
  are five, each with a vector of its own values in the vector file's `rules` (a vector's own
  parameter values; the header gains it), worked by hand on invented clips:
  - `Breathing`, the threshold (1Eh, `$90:8346`): C-breathe-adjusted at 50, energy 40 goes on
    to step 3, held 9, where the game's 30 would loop;
  - `StandingRestart`, the restart's frames (15: the stored 10h, `$90:A3AD`..`$90:A3C7`, less the
    same frame's countdown): C-restart-adjusted at 24, the frame ends on step 0 with 24 left;
  - `FastFall`, the speed and the fast fall's frames (5 and 7: the stored 8 less the countdown,
    `$90:A5AC`..`$90:A5C5`): C-fast-fall-adjusted at 3.0000 and 11, a fall at 3.8000 jumps to
    the fast fall step with 11 left, where the game's 5 would let the clip play on;
  - `RisingHold`, the rising frames (3: the stored 4 less the countdown, `$90:8011`..`$90:8050`):
    C-rising-adjusted at 6, the frame ends with 6 left;
  - `BoostStages`, the laps each stage lasts (`$91:B61F`): C-laps-adjusted-began and
    C-laps-adjusted-up and C-laps-adjusted-boosting at 3, 2, 4, 1 and 3 laps (a dash begun:
    stage 0's 3 laps; stage 1's last lap ends: stage 2 with its 4; boosting, its last lap ends:
    stage 4 with its 3).
  The root made the first four parameters (2026-11-11): their numbers already sit in the model's
  tuning, and the goal is that each behaviour can be adjusted. The cold-read round (#735) states
  the three holds as the frames the step shows, the setting frame included, so a tuner's number
  is what the player sees: the original stores one more and counts it down the same frame
  (`$90:801E` runs after each writer). The reference values are unchanged in effect (16, 8, 4
  stored; 15, 7, 3 shown), and so are the three adjusted vectors' expected States; their rules
  are now 24, 11 and 6 frames.
- **A dash lost after the animation.** `$91:DE53` is called from the movement handlers (before the
  animation: `$90:A3D0` standing, `$90:A551`.. the other halts, `$90:E5E9` the wall stop) and from
  the pose change's speed effects (`$91:EC76`, `$91:EC8E`, `$91:ECD3`, after it). On record every
  loss read as "after" (`dashing` from `0B3C` at N): item-charge-beam 52 and chain-0013-92FD 210
  run the plain pace on the frame the dash goes, with the pose unchanged. A loss in a running
  movement handler before the animation (the wall stop) would make `dashing` false that frame (the spec's stopped);
  samus-movement's model must report `dashing` as of the animation's turn (§Requests).
- **The dash's six values** (cold-read round, #735). Beginning and losing are one lifecycle
  (spec §1.3's table). Every reachable combination of "dashing as the clip moves on" and "lost by
  the frame's end":
  - **began and lost** in one frame: on a running frame (`0A1F` = 1) with no dash, run held and
    no transition for the pad (the direction let go), the lookup failure `$91:82D9` →
    `$91:8304` gives running the command decelerate, keeping her pose (carry set, `$91:82E4`),
    or stop when her base speed is 0 (`$91:8332`, `$91:831B`..`$91:832D`); the running
    movement's `$90:9771`..`$90:97D2` begins the dash (`0B3C` = 1, `0B3E` = 0001h with the
    booster), the animation runs with it, and the pose change's command (`$91:EC16`: `$91:EC50`
    → `$91:EC76`, or `$91:ECD0` → `$91:ECD3`) cancels it at `$91:DE53`. The spin's
    initialisation turning round (`$91:F66F`) cancels in the pose change too. At the cuts
    `0B3C` is 0 at N and N+1, so the cached records can't tell it from off; samus-movement's
    model reports both flags (`dash_began` and `dash_lost` from its run state), and
    C-dash-began-and-lost works it on invented clips. Its trace on her State is the pace's hold
    as her clip moves on in the kept running pose, and every frame of such a deceleration with
    run held repeats it. Not reconciled with the records: the cache's decode reads these frames
    as off (`0B3C` 0 at both cuts), and the 18,233-frame match shows no difference, so on record
    none moved on to a shown step whose pace differs from the clip's hold, or none occurred.
  - **stopped**: a wall met in the running movement's X movement (`$90:E5CE` → `$90:E5E9`)
    cancels before the animation, so the clip moves on without the pace and the stage and laps
    are 0 by the frame's end (C-dash-stopped). The turn-around movements (`$90:A79E`,
    `$90:A7BB`) cancel before the animation as well, in poses that don't run.
- **The counted lap's hold is the clip's** (the cold-read round's question; `CountedLapHold`,
  a quirk). With the booster and laps left, `$90:8587` branches to `$90:85DA`, which returns the
  control byte to the dispatcher: an `again` takes its target's hold from the delay list
  (`$90:8315`), not the boosted pace that `$90:84E3` gives shown steps. The 16 counted laps on
  record (`cachecheck.py`) match it. Any control ends a lap: `$90:82FB` sends every byte with
  bit 80h through `$90:852C` (C-lap-at-rest, C-lap-count-rest).
- **The spin start's order.** `$91:F6B2`..`$91:F6C2` (right; `$91:F713`..`$91:F723` left) test
  the screw attack, then the space jump, then `0A9A` (already spinning, set at `$91:F634` from
  the old movement type 3 or 14h), then the cinematic: the space jump sounds before the
  "already spinning" test, the screw attack after it (`$91:F74A`), and only the plain spin
  checks the cinematic. Spec `SpinStarts` states the order once.
- **Sounds waiting and the queue's allowance.** The count is the sound domain's, read each frame;
  nothing in appearance starts it. The allowance each request carries (6 or 15, the `$80:9049` /
  `$80:914D` caps) is part of the request; whether the queue admits it is audio-transport's
  `OfferAcceptance`. No appearance rule reads the answer: the boosting lap reads the count before
  its sound, whatever the queue does with it.
- **Initialization is a part of its own.** Initialise Samus (`$91:E00D`) is called from
  `$82:8014` and `$82:8057`, game states 6, 1Fh and 28h (loading), where none of her work, item
  controls, palette opportunity or drawing runs; the spec's part sets the whole State and the
  cannon's reset is one consequence of it.
- **The boosting lap reads the sound queue.** At counter 4, `$90:859D`..`$90:85A6` queue the
  booster's sound through `$80:914D`, which returns with A's high byte the library-3 queue count
  (`(0648 − 0645) & 0Fh`), so `$90:85AA`'s `XBA : AND #$00FF : ASL` indexes `$91:B61F` and
  `$91:B5DE` by it: 0 on all 5 records (boss-phantoon 187, samus-shinespark-* 50), giving
  `0B3E` = 0401h and the first lap's frame 0 held stage 0's first byte (3). X picks only that
  hold and the timer (`$90:85C3`..`$90:85D2`); the lap's later delays come from `$90:84E3`, which
  reads the stored counter `0B3F` = 4 (`$90:8507`..`$90:850F`): the boosting pace (boss-phantoon
  187 → 188, 0301h → 0401h; mutant A of adversary 91 fails 180 harness frames). With 6 or more
  waiting (the early return at `$80:9164`) the high byte is the sound, 3. With 5, X = 0Ah reads `$91:B629` (the pose
  definitions) as the timer and `$91:B5E8`'s bytes as a pointer: LEGACY.
- **The turn's end gives way.** `$90:8370` returns carry clear with no super special request when
  the pad's pose `0A28` is a ground jump, so the frame rests on the F8h byte with the timer at 0,
  and the pad's jump is the pose change (samus-screw-attack 243, 315, 387, 495, 531, 675 and two
  more). Outside a demo it also installs `$90:E926` (jump counted pressed next frame), which is
  samus-movement's "landed or ended the frame before". In the demos (`0A60` = E91Dh) it always
  hands over.
- **No-op steps rest with the timer at 0.** F0h-F5h return carry clear, so the countdown's 0
  stays and the next frame's `DEC` gives FFFFh, `BPL` fails and the frame moves on: one frame,
  whatever the buffer (6 on record: chain-0012-91F8 982, chain-0057-9FBA 232, 241, 251,
  chain-0059-A293 147, terrain-quicksand 1). An end step likewise leaves 0.
- **The super special path skips `$91:F404`'s same-pose test** but `$91:FB08` makes its own, and
  `$91:F36E` adds the buffer whatever `$91:FB08` did; the inputs' domain never ends a clip without
  a pose change.
- **Riding the elevator** (`$90:E8EC`) runs the animation but no pose update; the pose doesn't
  change on it, so no rule differs.
- **The arm cannon skip.** `$91:F4DC` tests the shot direction byte (`$91:B629` + 3) for 0 or 9,
  which the turns' FAh fails: so a turn aimed up isn't "aimed up" (spec `StartingClip`).
- **Time stopped, message boxes and the pause** stop the main loop, not the animation's rules:
  the frame count `05B6` doesn't advance, and the frame a message box closes runs only the PLM
  phase's tail (item-charge-beam 694: the counter advances, Samus doesn't run).

## The harness (for the integrator)

- **Frames:** each pair of consecutive `R` (quiescent) records of every row: an `O` record is
  mid-frame. Skipped, by name and never by outcome:
  - not a frame of play: game state `0998` ≠ 8 at N, or at N+1 neither 8 nor 9 (9 is a door's
    start: her frame runs first, then the door, so it is compared: 8 on record), or the new state
    handler `0A44` at N not one that runs the animation (`$90:E725` normal, `$90:E7F5` and
    `$90:E833` the demos,
    `$90:E8EC` the elevator, and `$90:E8AA` the Ceres escape's, which calls the normal handler
    (`JSL $90:E725` at `$90:E8B0`) and then runs the escape's timer) — 1,506;
  - the frame count stalled: `05B6` equal at N and N+1, unless it moved from N−1 to N (message
    boxes, the item fanfare) — 2,796. The pair where it first stalls is the frame a message box
    opens: her frame runs, then the box (the PLM's message box waits inside the frame), so it is
    compared — 14 on record, each moving her animation on (item-morph-ball 816, item-charge-beam
    295, 901, item-energy-tank 239, item-reserve-tank 111, 657, 1211, item-plasma-beam 145,
    item-wave-beam 36, chain-0043-9AD9 416, chain-0163-A890 27, chain-0234-D51E 152,
    samus-bomb-jump-vertical 54, samus-square-slope-reserve 36);
  - a message box ended the frame: `05B6` equal at N−1 and N (the pair after a stall) — 5;
  - a pose outside the domain at N or N+1 (not in the pose map: x-ray, grapple, shinespark,
    knockback, drained, crystal flash, the Draygon grab, the appearance) — 1,688.

  A step that doesn't decode (a frame number on an operand byte) is a harness defect: none.
- **Decoded:** the State from record N, the inputs as §Inputs.
- **Compared at N+1:** `0A96` (through the step map of the clip of the pose at N+1), `0A94`,
  `0B3E`. Per frame over the cache: 18,233 of 18,233; null 506. Of them, 2,001 are the Ceres
  escape's (samus-ceres-push, samus-input-poses: `$90:E8AA`), 14 the frames a message box opens
  and 8 the frames a door starts (area-maridia 22, area-norfair 119, boss-kraid-defeated 1089,
  chain-0014-96BA 221, chain-0097-ACF0 662, chain-0106-AD5E 481, chain-0224-D0B9 200,
  samus-input-poses 250).
- **Mask:** the boost (`0B3E`) on frames where it changes with no dash: `0B3C` 0 at N and at
  N+1, so no dash at N, none beginning and none lost, and `0B3E` differing from N to N+1. No rule
  of the animation changes it then. Writer: the shinespark's (S3) end of the shine, which clears
  counter 4 with no dash (terrain-quicksand 156 → 157, at the morph). 1 frame. The shine charge's
  timer `0A68` is the glow's and writes no boost: the 517 frames it is non-zero on match unmasked
  but that one.
- **Outputs not compared yet:** the end's kind (samus-movement's harness reads it as
  `AnimationEnds`; on record it is checked against the hand-over above),
  `turned far` (read ahead from the State and her pose's clip; samus-movement reads it), the sounds (until the sound
  domain's request stream exists: spin 31h, space jump 3Eh, screw attack 33h on library 1;
  speed booster 3 on library 3).
- **Per-behaviour attribution** (frames on record that take each rule, `cases.py`): the time runs
  down 18,233, a shown step reached 4,581, again 500, rest 6, breathing loops 31 and on 2
  (samus-ceres-push 1085, 1316), ends 341 (151 `end`, 190 `end-unless-jump`) and 9 that gave way,
  branches 13 (9 plain, 4 screw attack; space jump: 0), a pose taken 4,205 (1,964 kept, 102 from
  step 1, 2,139 from step 0), restarts 29, readied 39, fast falls 4, rising holds 20 (boss-phantoon
  11, chain-0059-A293 1, samus-ceres-push 8: her pose the same and rising at N+1; the 12 outside
  the Ceres escape are the ones samus-movement's heading down brings into play in the harness, and
  record N+1's direction gives 19 more, all landings), plain pace 760 and 58 laps, boosted pace
  715, dashes began 96 and lost 109, stages up 50, laps counted 16, boosting laps 5 (all 0
  waiting). Vectors only: the slowdown (`0A9C` is 0 on every frame of play in the cache),
  submerged, the space jump's branch, sounds waiting other than 0, a fall's pose rising at 5 px or
  more.
- **The harness cases** (recorded frames that read a game clip, compared through the importer's
  clips; each rule's constructed vector carries it): FastFall chain-0232-D5A7 224, 230,
  samus-grapple-release 254, samus-shinespark-up-left 394 (C-fast-fall); BoostingLap boss-phantoon
  187, samus-shinespark-left 50, -right 50, -up 50, -up-left 50 (C-boost-quirk-0); WallJumpBranch
  samus-screw-attack 73, samus-walljump-16 57, -22 63, -28 69 (C-branch-screw), chain-0043-9AD9
  61, 192, 205 (C-branch-spin); WallJumpReady boss-phantoon 3, chain-0014-96BA 50,
  chain-0043-9AD9 22, 36 (C-ready); ClipEnds gave way samus-screw-attack 243, 315
  (C-end-gives-way); rests chain-0057-9FBA 232, 241 (C-rest); breathing area-maridia 247, 337
  (C-breathe-loop); breathing on samus-ceres-push 1085, 1316 (C-breathe-on; the Ceres escape's
  handler); StartingClip from step 1 boss-phantoon 2, 61, chain-0014-96BA 20
  (C-spin-from-spin and the aimed-up rows); BoostStages up area-maridia 13, boss-phantoon 127, 152,
  172; laps counted samus-shinespark-left 72, 93 (C-lap-count); RisingHold boss-phantoon 570
  (4Eh, the first hold of its rise) and chain-0059-A293 298 (4Eh) (C-rising): each holds, her pose
  the same and rising at N+1 and step 1 kept with 3 left, so samus-movement's heading down and
  record N+1's agree (not down). A landing from a straight jump (boss-kraid-defeated 297, 315)
  passes the hold's test by record N+1's direction but not by samus-movement's heading down, so it
  is no case. Checked in a scratch run of the harness with heading down from samus-movement's
  model and this note's frames (the Ceres escape's handler, the frame a message box opens, the
  frame a door starts): both hold, all 18,233 frames match (stage and laps 18,232, 1 masked), and
  the hold's exercised frames are 20. RunPace laps boss-kraid-defeated 988, chain-0012-91F8 638
  (C-lap-plain).
- **Null model:** nothing changes: 506 of 18,233.
- **Free run:** from a seed frame carry the four State words and take only the inputs; re-seed at
  each skipped stretch. A free run through samus-movement's model needs its outputs (§Requests).

## Requests to samus-movement

Answered in samus-movement's spec (§1.4 there, its round 18; its decode notes, "Round 18's
amendments", give the sources). Its step reports `dashing after her moves`, `dash began`, `dash
lost`, `handed over` and `heading down`, and from its round 19 `vertical speed after her moves`
(signed, as her moves leave her, beside `heading down`), besides `StandingAnimation::Restarts`,
`WallTest::Readied` and its pose, and its `AnimationEnds` takes the end's kind as its input and
gives way on an end unless a jump when its pad chart's pose is a take-off aimed forward (4Bh,
4Ch) or a spin (19h, 1Ah). The harness's decodes above agree with those outputs on every frame
these rules read: samus-movement's `dashing after her moves` differs from "`0B3C` at N, or a dash
began" only where her moves end the dash, and while running at the frame's start that is a wall
stop alone, whose frame hands her `AgainstWall`, so the clip the frame ends on starts anew
(`StartingClip`) and the stage goes to 0 as the dash is lost either way. `heading down` is `0B36`
= 2 at N+1, and `handed over` the decode above.

## Part 2, left for the next round

Her sprite from the step outside the elevator contract (`$90:85E2`..`$90:8A4B`, including the
chest sprite, and the tiles uploads at `$92:8000`), the flicker (`$90:85E2`'s invincibility test), the
palettes (`$91:D6F7`: the suits, the visor `$91:D83F`, the charge, the screw attack's and the
speed booster's palettes `$91:D9B2`/`$91:DA5C`, the hurt flash `$91:D8A5`), the atmospheric
graphics (splashes, dust, `$90:80E6`, `$90:A413`), and the echoes' pictures. Old code:
`game::samus_drawing` 605, `game::samus_palette` 183, `game::atmospheric` 1,082 (domains.md).

## Scan

`tools/cleanroom/scan.sh` on the spec and the vectors: 0 (address-form 0, address-known 0, name 0,
enum-order 0, ram-field 0, single-routine 0, entry-point 0, route 0). `score_scan.sh`: 0.
`scan.sh --mirror` on the spec and these notes: 3 of 22 entries single-routine (13 %, limit
30 %), 1 unsourced. FootContact,
CannonItemDelay and CannonReadinessReset each have one source mechanism; the spec states their
observable rules and leaves the public API form to the implementer.

**Invented content (decision 13).** `invented_check.py` tokenises the vectors' clips and every
game clip the importer reads (251) into shown holds and control kinds, and flags a vector clip
that equals a game clip, has a game clip's shape (its kinds in order, holds ignored), has a game
clip's holds shifted by a constant, or shares a run of 4 holds with one; it flags a pace that is
uniform, equals one of the game's six pace lists, differs from one by a constant, keeps one's
pattern of differences, or shares a run of 4 holds. Round 1's content gave 104 findings; the
content now gives 0.

## Budget

Moved from the spec's header by design (MODEL §1 decision 10): the implementer is not told the
score or the budget. `tools spec-units` reads k and the areas here, and `tools report` the areas'
old figures and the commit.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on samus-movement. An
area's budget is k × its units. The old code (for reference, not a bound) is the `tools size`
score, under the tools at `5e2e4ee51` (score v2: lookups as data, duplication), of the files at
`f1df56ede`: `game/src/animation.rs` (1,282.25) and the animation's part of `game/src/pose.rs`
(`reset_animation` and `run_super_special`, an extract measured with `animation.rs`, whose code
it repeats: 242.75, of which duplication 45; 197.75 alone): 1,525 together, not split by area
(the old module interleaves the clip, her moves' writes and the run's pace in one handler). The few lines of `pose.rs`'s `refresh` and `change`
that set and reset `new_pose_frame` are left out: a handful of tokens inside functions that do
the pose's other work. The old code modelled neither the boost stages (`world_blocks_bridge.rs`: the
engine keeps no speed booster's charge) nor any slowdown (`animation::frame_buffer` returns 0),
so its figure covers the clip and the moves; the run area has almost no old counterpart. The
budget is an indicator: the fit is score ÷ budget, and being
over it costs in proportion to how far over, never as a wall.

Budget: 37.5 × 60.0 units = 2,248 (clip 1,287, moves 225, run 412, cannon 212, palette 112),
23 entries, 56 arms, 193 vectors. The kept action's preparation arm in `SpinEnds` is 1.0 of
them; naming each input once counts no unit. The cold-read round (#735) moved 58.3 to 60.0:
`CountedLapHold`'s arm (+1.0, run), the palette's refused and unrecognized arms (+2.0), and the
clip area's slowdown formulas written out in words (−1.3).

Old code (old 1,525): `game/src/animation.rs` and the animation's part of `game/src/pose.rs`,
together, not split by area.

- Area clip: 1.4, ClipPlays, ClipControls, Breathing, ClipEnds, WallJumpBranch, HandOverSlowdown,
  StartingClip, FootContact, SpinStarts, SpinEnds, AtTheDoor
- Area moves: StandingRestart, WallJumpReady, SpinTurnedFar, FastFall, RisingHold
- Area run: RunPace, BoostStages, CountedLapHold, BoostingLap
- Area cannon: CannonItemDelay, CannonReadinessReset
- Area palette (old 183): OrdinaryDryColours
- Area hurt (old 0: no old code models the pulse): HurtColours, 12
- Area elevator body (old 0): ElevatorStandingPicture


## Stride, spin and passage sound opportunities

### Sources and existing interface

`FootContact`: `$90:A3E5` running dispatch moves X/Y first, then `$A3EE..A3FF` reads
`0A96` and exactly `0A94 == 1`; contact content is the nonzero low byte at `$90:A424`
indexed by that animation frame. Import this as a contact-step set through `Imported::frames`,
not a new execution table or numeric movement code. `$A401` requests graphics before sound
suppression; `$A404..A419` checks `1F51`, `179C`, `0A68`, then `0B3E & 0400`.
`$A41B/A41E` asks for sound 6 in library3, cap 6 (`$80:914D`). The sound suppressors have
no effect on the already-run graphic request. The step, the time left and the stage it
reads are the State as the frame starts, and Running is her pose as the frame starts (spec
§1.3): the stride runs before the frame's pose selection, so her pose after is not the
contact-phase pose. The contact opportunity must be captured where running movement has
executed, before `samus_appearance_bridge::animate`, not discovered from a resulting sender
write.

`SpinStarts`: `$91:F624..F757`, the spinning pose initialisation. Old movement `0A23`
spin or wall jump sets a first-step skip, which suppresses plain/screw spin-start but not
SpaceJump. Surface/gravity checks precede item selection. Plain right `$F6C6/F6CE` and
left `$F727/F72F` reject cinematic `1F51`; SpaceJump `$F6D4/F6D7`, `$F735/F738`
does not reject cinematic or a continuing spin. ScrewAttack `$F74A/F752` rejects the
step skip, not cinematic. Sounds 31/3E/33 in library1, cap 6. These are not `$90:8455`,
which is the already-modelled WallJumpBranch control. `$90:F376` belongs to command14,
whose `$F34F` facing test makes it unreachable for ordinary valid pose facing; it is not
a generic native spin-start producer. The new selection opportunity uses typed Figure,
items and Liquid inputs; no new animation progression or motion. `StartingClip` remains
sole owner of the animation start. A continuing spin's sprite-only selection is a useful
non-M0 test: SpaceJump may sound while plain and ScrewAttack do not.

`SpinEnds`: normal draw `$90:EB35`, after body/projectiles and `$EAB3`, calls `$F576`.
`$F5A8..F5CB` reads previous movement `0A11` and current `0A1F`; prior spin/wall and
current neither asks sound 32, library1, cap 15. `0A11` is a separate retained preparation
snapshot, NOT `Samus.previous: PoseRecord`, which the decoder maps from `0A20/0A22/0A23`.
It is her appearance State's kept action (spec §9.1); current movement is her pose after.
Her pose as the frame starts cannot substitute for it on a frame without preparation. Model output
is a pure semantic decision, with no acknowledgement, animation update or motion step.
Drawing can repeat this decision if remembered movement remains unspun; the sender retry
itself never invokes it. The engine must call it on each real normal drawing opportunity.

`AtTheDoor`: `$90:F471..F4A1`, invoked by `$84:8250` on first Quieting entry.
Current movement `0A1F` spin/wall asks 32 regardless of pad/charge. Otherwise shoot
released (`008B & 09B2 == 0`) and `0CD0 < 0010` asks 2. Both library1, cap 15.
The comparison is signed; the supplied clean charge domain0..32767 excludes negative words.
This first-entry request precedes the sound hold and is distinct from the same work's draw.
R1 owns the first-entry opportunity; S5 owns selection of the Samus sound.

### Native prefix evidence and admission

Row `chain-any-0003-DFD7`, pinned lsnes recovered cache, native Requested cuts0..183:
contact producer queue changes at10,64,74,131,141,151,164,174 in cut0->183.
The actual preceding native seed has no library3 pending queue or outstanding ACK; it is
Running on a contact with1 frame left. Seed->cut0 admits a NEW Footstep, and cut0 then
holds its outstanding ACK. Cut0 is not the seed; these two windows must not be conflated. SpinStarts changes library1 queue at16
and80, both plain spin poses from Running, no items/gravity/submersion/cinematic.
SpinEnds queue changes29 and81. The second sound is sent at86/87; those sender writes do
not define its appearance trigger. AtTheDoor asks SamusSilence183. All 13 new requests are
supported by independent semantic queue comparison over 183 cut transitions: reference
183/183, no-request model170/183. This is a per-frame localiser, not an engine free run:
movement/pose inputs are observed at their real cuts. Sender state/replies are not predicted.

`evidence.py` independently checks every cut0..183: `1F51`, `179C`, `0A68`, `0CD0`,
`0DC0`, `0B40`, `05D1`, `0943` all zero; area `079F` is Ceres exclusively and
`0B3E & 0400` never set. `dry.py` checks both liquid surface branches against Samus's
top, gravity and liquid-disable input: dry 184/184; SpaceJump/ScrewAttack/gravity absent 184/184.
These predicates validate the observed admission; production must validate the native
seed/restoration and preserve the owner restrictions, not inject false values at each cut.
Engine State currently lacks cinematic/boss/special-colour and retained charge/echo controls.
They require typed owner inputs or a validated bounded inactive capability; broad support
cannot come from constants in the bridge. The clean SpinEnds decision itself accepts all
supplied movement pairs; its output must not be disabled by engine capability admission.

Neighbouring `$F579..F58E` consumes retained charge-resume `0DC0` and may request 41;
`$F591..F5A4` cancels an echo in `0B40` with sound 25 when boosting stops;
`$F5CF..F5E1` may retain charge resumption for an unspin at charge>=16 with shoot held.
`$F5E4..F638` contains debug and Zebes time-up. They are not implemented by this amendment.
Admission must establish their inactive owners, including no time-up event (event0E) and
no debug state; `0943 == 0` alone is Ceres escape admission, not proof of Zebes event0E.
`family157.py` separately verifies `D821 & 40 == 0` on all 184 cuts0..183 (event0E
through `$80:8233`, events at `D820`). Native admission for this neighbour is proven.
The decode must check the event bit at seed/restoration as well as retained charge/echo;
a future owner supplies their outputs/state changes before admitting an active restore.

### Graphics and comparison

Ceres `$90:ED91` dispatches contacts to `$EE64`; common graphics return unless boost
stage byte equals04. With admitted unboosted dry Ceres the graphic request changes no
atmospheric memory or OAM. This proves a real zero-effect graphical opportunity, not a reason
to suppress graphics with a sound flag. Active boosting dust and non-Ceres liquid contacts
stay part2; the contact output remains separate so its owner can run them. Compare whole
`samus.animation`, atmospherics and OAM on admitted M0, and semantic request families at
producer opportunities. Do not mask a graphical slot merely because sound was suppressed.

The other M0 library3 requests at 57/124 are SamusLanded, not CeresDoorOpening:
`door_inventory.py` reads each newly admitted queue slot at the prior next-index and
identifies the shared landing/wall-jump cue; `landing_inventory.py` proves both are ordinary
landings from NormalJumping to Landing(Jump), not wall jumps. Source `$91:F010` calls
`$F046`, and `$F08F` asks for the landing cue, library3, cap 6. `$F078..F09D` selects hard
landing when the signed whole speed is at least 5; otherwise the subpixel speed must be
nonzero (`$F082/F085`) for the shared soft cue, even when whole speed is nonzero.
Without that fractional word, the soft branch also skips graphics. Cinematic
suppresses sound; graphical landing still runs at `$F0A1`. M0 admits the soft cue at 57/124.
The actual pre-stop speed is movement-owned; current M1 Outcome publishes that speed but
no landing/Met event or semantic sound, so these requests require an owner extension.
`Met::Floor` is currently private to movement resolution; remembered landed-or-ended jump
history conflates distinct causes and cannot be used as an admission event.

At 157, `family157.py` and `door_inventory.py` identify CeresDoorOpening against the
newly admitted queue slot and source `$A6:F6C0`, the single B8 cue. It is neither footstep
nor impact, and has no B8 variant at 57/124. Firing at 157 belongs to weapons.
The audio driver's replies remain U1: these opportunities cannot produce them, and exact
native sound-State equivalence is not claimed here.

### Vectors and checks

Vector source alias `M0` names row `chain-any-0003-DFD7`. The 47 `[[opportunity]]` cases (§The whole-frame
update) supplement the animation vectors. Each compares
semantic sounds, independent graphical contact and the whole State the frame leaves: a contact
or selection case is a frame with her movement, so its clip plays (on invented clips); a drawing
or passage case leaves the four animation quantities unchanged. Recorded guard cases use
invented content, never copied contact tables or clip holds, so their `after` animation is the
invented clip's, not the record's.
`reference.py`: existing98/98, opportunities45/45, native183/183 versus null170/183.
Contact vectors supply the actual retained stride stage, with matching before/after State:
stage 3 requests Footstep, stage 4 suppresses its sound while preserving graphical contact.
Two composition checks retain contact plus spin-start and WallJumpBranch plus SpinEnded
requests, graphical contact and all four final animation quantities, with at most one update.
Eight planted faults are caught: drawing advancement fails C-end-frozen on time left;
aggregated contact loss fails C-contact-and-start on contact; treating stage 3 as boosting
fails C-contact-stage3 on its sound. Five further decision faults are caught: wrong contact frame, graphics suppressed with sound,
plain continuation sounds, spin-to-wall wrongly ending, and charge threshold 15 instead of 16.
Observer artifacts live in `$TEMP/collab/night-20261115/appearance-spec/`.

FootContact, SpinStarts, SpinEnds and AtTheDoor belong to area clip. The old animation code has no generic producer counterpart; this amendment does
not rebuild part2 rendering. `tools spec-units` counts 50.7 units, a budget of 1,900,
using the fixed k and the areas above.


### The prepared movement snapshot and boundary accounting

The kept action is appearance State (spec §1, §9.1): it carries over every frame that skips
preparation, so it is memory, and the State holds all she remembers.
The sole explicit writer in inspected banks is `$90:EB2D/EB30`: copy the current word
`0A1E` (facing/movement) into `0A10`; `0A11` is its movement byte. Current-state preparation
calls this from normal `$E695` at `$E6C3`, demo `$E6C9` at `$E70D`, and locked `$E713`
at `$E71C`. It follows input/projectile effects and precedes ordinary motion/new-state pose
selection. Pose-change history `0A23` is a different writer/lifetime, often refreshed to the
current action before drawing. Normal `$F576` reads the prepared snapshot; neither it nor
the extra body-only inanimate draw updates it. A locked current-state preparation still
refreshes it even when animation/motion do not run. Current-handler RTL and Quieting work
that bypass preparation retain it; do not key the refresh to a general frozen-animation flag.

`drawing_snapshot.py` reads the actual preceding `any-boundary/0003.record` and all native
cuts0..183. The snapshot recurrence is184/184: refresh at actual ordinary preparation through
182 and hold at183. At29 and81 prepared movement is SpinJumping, while current and pose-change
history are NormalJumping; using0A23 instead gives exactly2 missed unspin opportunities.
The next preparation at30/82 captures NormalJumping. Seed/0/182/183 all retain Running; the
first passage's normal drawing is frozen and does not fabricate another preparation.

The selected S5 producer windows are: cut0->183 has13requests (8Footstep,2PlainSpin,2SpinEnded,
SamusSilence183); seed->182 has13 (9Footstep includingcut0,2PlainSpin,2SpinEnded);
seed->183 has14 including first-entry silence. Seed library3 queue and ACK are empty before
its genuine cut0 Footstep admission. These counts exclude the separately owned landing57/124,
Ceres opening157 and firing157 families. `snapshot_reference.py` exercises capture-after-earlier
effects, hold, locked preparation, repeated normal drawing and body-only inanimate drawing;
pose-history substitution and frame-start substitution faults fail their named cases.

**Her action at the preparation is her pose's as the frame starts** (spec §1.5). The movement
word `0A1E` is written only by Initialise Samus (`$91:E132`) and pose initialisation
(`$91:F433`, `$91:F468`). Its callers, from banks `$82`, `$90` and `$91`: her pose update
(`$91:EB88`, `$91:F404`) and the hack handler (`$90:E12E`, from `$90:E738`) in her new state
handler; Samus commands, item pickups, making her face forward, the crystal flash, Draygon's
grab and release and the drained controller, run by enemies, PLMs or her new state handler;
equipment changes on the pause screen; demo setup; and the x-ray scope's pose input handler
(`$91:FCAF`). Game state 8 (`$82:8B44`) runs only the debug and palette-object handlers before
her current state handler (`$82:8B58`), and that handler reaches the preparation after her
pose input handler, suit palette index, acceleration, block-inside handling and HUD/projectile
handling (`$90:E6B0..E6C0`). So within a frame only the x-ray scope, outside the domain's
poses, changes her action before the preparation, and every other writer's change shows in
the record the frame starts from. Bank `$94`, run from the same handler, only reads her pose
(`0A1C`, `0A1E`); bank `$9B`'s one writer, `$9B:B3A7`, sets the death pose, outside the
domain's poses. This is a reading of the bank logs' callers, not checked
callee by callee for an indirect pose write.

## The whole-frame update

MODEL §1 decision 35 compares owners' State and output bytes at each NMI cut and leaves
intra-frame bus timing to T1. The spec therefore states the domain as a direct update of one
frame: the four animation quantities, cannon readiness and the three colour-sequence quantities
the frame leaves, with the frame's outputs. It exposes none of the original's query points and no
resumable part-frame update. The catalogue holds 22 entries.

**Values read part-way through the frame, as whole-frame outputs** (spec §1.1):

- *The end and turned far*, read by samus-movement before its step: the read-ahead (spec
  §1.1), one query of the State and her pose's clip as the frame starts, not a partial frame:
  it changes nothing, and the frame that follows gives the same values. A query, because the
  frame needs her pose at its end, which samus-movement's step picks after reading them.
  Evidence: §Requests to samus-movement; the harness reads both from record N.
- *Graphical contact*, read by part 2 for the stride's splash or dust in the same frame: the
  stride's contact opportunity. Evidence: §Stride sources, the contact read sits after her
  running movement and before the animation update, and reads the frame-start step and time
  left (the update has not yet run).
- *Cannon may follow*, read by the picture of her gun later in the frame: the delay as the
  frame's item controls leave it. Evidence: §Cannon sources, controls at `$90:DCDD` in the
  normal and demo handlers, the drawing read at `$90:C5C4..C5F7` after them in the same frame;
  chain-0059-A293 428 to 430.
- *Colour sequences*, read by drawing and palette upload later in the frame: the three
  quantities as the frame's palette opportunity leaves them. Evidence: §Ordinary dry palette
  preparation, the opportunity sits between the selected-pose work and Health Maintenance, and
  video and colour transitions read later.
- *The frame's sounds*, read by the sound domain: one ordered list (§9.2), the stride's, the
  animation update's, the spin start's, drawing's; a passage's first entry adds `AtTheDoor`'s
  before drawing's. The order is the frame's own: the stride's contact in running movement
  (`$90:A3E5`), then the current clip's play, where `WallJumpBranch` and `BoostStages` ask, then
  the selected pose's initialisation (`$91:F624..F757`), then normal drawing (`$90:EB35`), as
  §Stride sources gives them; `AtTheDoor`'s request precedes the sound hold and that frame's
  drawing.

No other domain reads a value of this domain between those points: the HUD's early mini-map
reads samus-movement's position, not appearance.

**One set of inputs.** Each fact of the frame is one input (spec §1.3), read by every rule that
needs it: the figure, the items, submerged and cinematic among them. Her pose is two inputs
only because the pose as the frame starts and the pose she ends it in are different values (the stride and the preparation come before
samus-movement's pick, drawing and the palette opportunity after it). The stride's step, time
left and boost stage are the State's, not inputs. The palette opportunity reads no position: the original's bottom calculation enters only the surface comparisons, which the
water and lava levels already stand for (§Sources and exact mapping), and the arm holds the
same outcome whatever the position. The spin sounds are one set of requests that
`WallJumpBranch` and `SpinStarts` both ask for (spec §9.1). The appearance State holds the kept
action (§The prepared movement snapshot), and has a starting value (spec §1.6, §State).

**Harness note.** The comparison is per frame, from record N to record N+1 (§The harness),
with the masks named there. No vector compares part-frame work. Indexed replay cites
`ClipPlays` and `AtTheDoor` from this domain. The kept action is `0A11` at N+1 through the
movement-type map; the harness doesn't compare it yet, and a case that draws reads it from the
carried State, never from record N+1.

**Vectors.** 189: 99 animation frames, 47 single-rule sound cases, 3 combined frames, 19
cannon histories (each step a frame's controls, a skip, an initialization, an outside drop, an
unrelated event or a drawing), 19 palette histories and 2 refusal boundaries, beside the
`[initialization]` table. In the palette histories `palette_updates` are another producer's
writes in the reset's frame, and `applications` whether that frame applied the reset. Every
kind of case names the frame's inputs alike (`pose`, `pose_after`, `items`, `submerged`,
`cinematic`, ...). Each case keeps the meaning the arms it cites give it:

- the contact cases give her pose and the stride's clip and read her step, time left and stage
  from `before`; each is a frame with her movement, so `after` shows the stride's clip moving
  on (to its rest from the contact step) or counting down;
- the selection cases give her pose and pose after, their clips, and the items as a list; a
  changed pose with the same action changes her facing, and `C-start-same` keeps one spinning
  pose with the space jump, so that only the unchanged pose silences it; each is a frame with
  her movement, so `after` shows her clip playing and the clip of the pose she takes starting
  (a run to a spin from step 0, a spin or wall jump to a spin from step 1, a run to a run
  kept), and the two submerged cases are in water (both slowdowns 3);
- the drawing cases give the kept action in `before` and `after` and her pose; a drawing case
  whose pose changes is a whole frame with her movement, preparation and drawing, since a frame
  without her movement keeps her pose: `R-unspin` is the prepared frame it was on record (cut 81
  prepared her spin, §The prepared movement snapshot), with invented facing and aim and the
  invented spin and fall clips in place of her animation; `C-end-prepared` is the same frame
  from a kept run, and `C-end-prepared-replaces` a prepared drawing frame alone, for the
  preparation's two arms;
- the passage cases give her pose;
- `C-initialize` is a frame with her initialization alone, from a State with every quantity
  changed, which leaves the `[initialization]` State;
- two more combined frames are whole frames with one set of inputs and their parts, so each
  takes a pose after her clip's: `C-branch-and-end` falls out of the wall jump (the fall's
  clip from step 0, held 9), `C-contact-and-start` takes the spin (its clip from step 0, held
  6), since no single-rule case's pose agrees with the other rule's;
- the stride's invented clip is a `[clips]` entry with its contact steps, and gains a shown
  step after its rest (spec §1.5);
- the palette histories give `pose_after` and `items`, with no position, and no `action` or
  `screw_attack` observation beside the pose and items.

**Reference model.** `$TEMP/collab/appearance-gaps-scratch/reference.py`, written from the
spec's whole-frame rules on the frame's one set of inputs, without the model. It agrees with
every vector: animation 99/99, opportunities 47/47, frames 3/3, cannon 19/19 (57 steps),
palette 50/50 (19 histories and 31 refused cases), and the `[initialization]` table. A
do-nothing model agrees on 0/99, 8/47, 0/3, 7/19 and 31/50. Sixteen planted faults each fail:
a contact or selection case leaving her animation as it was (28 cases), initialization
resetting the cannon delay alone (1 frame, C-initialize),
the end read after the moves (8 vectors), the boosting lap's quirk off (3), no kept rows (4, and the case C-start-other),
no hand-over slowdown (3), stage 3 silencing contact (1), the frame's sound order reversed (2
frames), a switch event restarting the delay (4 histories), a drop restarting it (1), the
frame-end selection in place of the controls' (1, C-cannon-drop-after-change), a screw-attack
spin admitted (2), other producers' palette writes lost (1), the preparation not keeping her
action (2), drawing reading her pose as the frame starts in place of the kept action (3) and
the selection compared by action alone (1, C-start-space-continued). Per-frame record
agreement for the same rules is stated in §The harness, §Cannon selection readiness and
§Ordinary dry palette preparation.

## Cannon selection readiness

### Responsibility and the observed seam

Appearance owns readiness because the visible effect is the cannon retaining its former
open/closed disposition while a new HUD item is already selected. HUD owns selection and
one-shot mode; its `Switching` entry explicitly leaves effects to weapons and her look.
The uncharged power-beam contract owns firing eligibility, not this visual delay. Neither
ordinary animation playhead changes nor a switch event determines readiness.

The canonical retained lean cache was recovered from
`lockstep-cache-5663de2c4b548c68.tar.xz.enc`, SHA-256
`4101a60be21f26718004c607e7b031a3ac3ac934864c1f22235465e377c41b7f`.
Watching 63 rows / 40,898 records found 129 selected-item changes, 239 ready/not-ready
transitions and 171 changes of cannon open state, but no one-shot-mode-only record transition.
This watch precedes the source derivation. On `chain-0059-A293`, the HUD selects missiles at
429 while the cannon is still closed; at 430 readiness returns and its open disposition changes.
Changing to super missiles at 431 waits again, though both items need the cannon open.
`boss-kraid-defeated` 800..802 demonstrates that an intervening NMI can leave readiness alone.
The opportunity count comes from independent watched control execution, not a final pose or
a frame counter that may cross a lag cut at another point.

### Sources by entry and RAM mapping

- `CannonItemDelay`: `$90:C4B5..C534` observes the starting item in `$12`, then compares the
  final HUD item at `$C519..C51E`. Changed identity stores 1 at `$C531..C534`; unchanged
  identity advances the retained quantity, stopping at 2, at `$C520..C52C`.
  `$90:DCDD..DCED` performs this observation only when her HUD controls run and she does
  not face forward. The normal/demo callers are `$90:E695` and `$90:E6C9`; frozen time is
  checked after the observation, so it is not itself a readiness suppressor.
- `CannonReadinessReset`: `$91:E00D..E027` initializes Samus for starting at Ceres/Zebes or
  loading a demo, clearing `$0A02..0E0B`, including the readiness word. Pose initialization
  at `$91:F433..FB07` is a different operation and supplies no reset of this quantity.
- *Cannon may follow* (the frame's output, spec §1.1 and §10): `$90:C5C4..C5F7` reads
  readiness before the selected HUD item, while the cannon is not already opening/closing,
  later in the frame than the controls. `$90:C5EE` compares against ready; when readiness is
  absent it does not change the open disposition.

| Semantic quantity | Source | Validation-only encoding |
| --- | --- | --- |
| selected item identity | `$09D2` | none 0, missiles 1, super missiles 2, power bombs 3, grapple 4, x-ray 5 |
| one-shot selection mode | `$0A04` | independent of identity; zero or the selected item's encoding |
| eligible opportunities remaining | `$0AAA`, word | `2 - source_word`, for source states 0..2 |
| cannon may follow selection | `$0AAA` read at `$90:C5EB` | ready exactly when source word is 2 |
| source open disposition, observed only | `$0AA6`, low byte | separate from readiness; cannon animation/draw remains downstream |
| physical select/cancel evidence | `$008F` against `$09BA` / `$09B8` | newly pressed physical masks; decoded independently of readiness |

No native gate or later expected readiness is a production initializer. An initial current
readiness owner must be admitted from an independent initial observation or the actual clean
initialization output, and then carried. The accepted bounded Drawing source projection's
reached ready branch is insufficient to initialize the delay histories. The source word is
incidental storage; the clean quantity is the remaining number of eligible control observations.
The model decides its representation and updates. Source branches belong here, never in an adapter.

### Vectors, reference proof and limits

The `cannon_readiness` table supplies 14 constructed and five recorded histories, each a
sequence of frames (§The whole-frame update).

| Recorded vector | Preceding record | Compared record | Witness |
| --- | --- | --- | --- |
| `R-cannon-select` | chain-0059-A293 428 | 429 | changed item waits |
| `R-cannon-follow` | chain-0059-A293 429 | 430 | unchanged next controls become ready |
| `R-cannon-same-item` | chain-0059-A293 20 | 21 | actual same-item control event does not restart delay |
| `R-cannon-lag` | boss-kraid-defeated 800 | 801 | no performed control observation; delay remains |
| `R-cannon-deselect` | area-maridia 255 | 256 | changed identity to none waits |

Private observer evidence lives at `retained-time-479/cannon-readiness-observer/`:
`OBSERVATIONS.private.toml`, `RECORD-PROOF.private.toml`, `watch.py`, `reference.py` and
`prove_records.py`, the record proof. The whole-frame reference (§The whole-frame update)
agrees with all 19 histories, and its cannon faults (a switch event restarting the delay, a
drop restarting it, the frame-end selection in place of the controls') each fail.

The carried scoped reference agrees on 26,608 observations over 63 contiguous segments;
the same segments' do-nothing owner agrees on 24,526. It includes 123 changed-control
selections and 16 performed same-item control events. Start readiness is independently
observed once per segment and carried, never replaced from the next expected readiness.
Independent watched `$90:C4BD` hits supply the performed control opportunities. The
selected-item neighbour is observed, not modeled by this reference; this is not runtime,
full-frame or indexed acceptance. Two external-selection-change pairs are explicitly excluded
from inferring the control answer from end-of-frame selection; constructed external-drop
histories cover the unchanged clock. Out-of-play boundaries and uninitialized source words
are outside this scoped record comparison, not silently passing cases.

Initialization's full two-observation history, mode-only changes and a drop after changed
controls are constructed from source-qualified rules, not claimed as recorded histories.
Cannon opening/closing animation, pictures, the open-target table, charge/palette switch effects
and arbitrary corrupted source words are outside this amendment. Source initialization resets
readiness, but an ordinary pose/clip change or room passage without reinitializing Samus does not.
No animation contract changes, and no timing-profile admission is claimed.

## Ordinary dry palette preparation

### Responsibility and recorded opportunity

Appearance owns the reached Samus palette reset alongside the animation and cannon parts of
the look. The historical `samus_palette` module only partially implemented this responsibility.
The separately qualified room palette-object opportunity supplies room effects. Its empty-slot
result has no bearing on whether the Samus reset applies. The preserved current palette remains
available for video upload. The target palette and the reset metadata remain available for
their later readers. Keep one authority for the current and target palettes across both
producers. Only the sequence metadata is owned and changed by the ordinary Samus reset.

`R-ordinary-run-colours` is the first retained source frame of `chain-any-0003-DFD7`, between
the selected-pose work and Health Maintenance, source records 4142–4228 inclusive. The retained
source packet `frame-work-contract-549/selector-s5-display/palette-source-v2/manifest.toml` pins
the source with SHA-256 `310e39057995f2589b3c8dc6f89729a791536443ebc1464fa651364194318b83`.
The independent review `review-selector-s5-palette-v2/adversary-palette-v2.md` is pinned at
`ce7ae70be0c44dafbba3e77b84b192fc371e7d9fff33cafae96ae555ccb9854f`.
Those private packets supply one bounded source proof and 36 controls. Exported RGB patterns
and detailed example geometry are invented. The recorded action, admitted neighbour classes,
metadata and application count are checked against the retained source. No ROM colours
are serialized in the vectors.

### Sources and exact mapping

- `OrdinaryDryColours`: bank 90's ordinary pose work and palette opportunity, followed by
  Health Maintenance. Bank 91's palette path `$91:D6F7..D705` reaches the inactive charge
  reset at `$91:D7B0`, the visor decision `$91:D83F..D8A4` and the ordinary unchanged-colour
  path in `$91:D9B2` onward. The standard-room visor reset is the store at `$91:D8A0`.
  The red and orange alternatives admit separate source branches through `$91:D856..D867`,
  which decrement the sequence delay and may advance the sequence. Their future colour output
  belongs to the corresponding unqualified effects.
  The action enters only after that reset: special type zero dispatches to `$91:D9B2`, whose
  dry path reaches the movement-type test at `$91:D9DA..D9E8`. Spin jumping (`$91:D9F7`)
  tests screw attack in the equipped items (`$09A2` bit `0008h`); without it, and for every
  other movement type except wall jumping (`$91:DA0B`), control reaches the speed-boost test at
  `$91:D9EA` and returns carry set at `$91:D9F5`, so the suit palette load at `$91:D717` is
  skipped and neither bank is written. With screw attack, the spin either loads a screw-attack
  palette and advances its own frame (`$91:DA1B..DA41`), clears that frame at animation step
  zero (`$91:DA43`), or returns carry clear from step `1Bh` on and loads the suit palette, so
  that spin stays unsupported. Wall jumping is not reached by the opening.
- Suit selection: bank 90 tests the currently enabled Gravity and Varia items in that order and
  produces the neutral Power or the Varia/Gravity selection. The packet checks those item tests,
  branch destinations and latch stores independently of the palette snapshot. The output of
  the completed SuitPalette opportunity is distinct from a numerically equal initial latch.
- Grapple: bank 9B's dormant callee `$9B:C4F0` tests a newly pressed bound-fire input and can
  reach FireOrCancel, `$9B:C51E`. The packet binds the four dormant publishers, both current
  classes and the palette comparison. Other grapple representations remain unsupported.
- Room treatment: bank 88's actual blending dispatcher and standard screen setup at
  `$88:807D/$88:8081` place BG1/BG2 and sprites on the main screen and BG3 on the subscreen.
  Its red and orange setups supply the alternative classes. The source check binds these
  to the visor decision and validates the backdrop literal stores independently.
- *Colour sequences* (the frame's output, spec §1.1 and §11): the three quantities as the
  opportunity leaves them, which the frame's later drawing and palette upload read. The
  whole-frame reference checks that a frame applies the reset at most once.

| Semantic quantity | Source mapping |
| --- | --- |
| appearance override | `$0A4A`, inactive at zero in the bounded proof |
| stored glow from a charged shot; beam charging | `$0B18`, `$0CD0`, each inactive at zero |
| special colouring | `$0ACC`, inactive at zero |
| charge-glow position | `$0B62`, reset at zero. Ordinary charge sequence advances by two source units per picture at `$91:D782..D78F`. Other effects reuse this storage; their representations require separate qualification |
| visor delay and sequence position | `$0A72`, low byte is eligible ticks left, high byte divided by two is the zero-based sequence entry. The ordinary store sets one and entry three |
| suit selection | `$0A74`, independently produced by bank 90. Decoder admits Power, Varia and Gravity. Only Power selects this arm |
| dormant or busy grapple | `$0D32`, bounded semantic decoder as above |
| room colour treatment | `$1982`, standard/red/orange only, as above |
| water and lava levels | `$195E`, `$1962`, absent when the corresponding signed surface is negative |
| her pose after | `$0A1C`, `$0A1F`; the original also reads `$0AFA` and one immutable pose geometry read for her bottom, which enters only the liquid surface comparisons, so the spec's water and magma levels stand for it and no rule reads her position. Dry admitted poses and positions give the same palette outcome. The admitted upright actions are movement types 0 (Still and Landing(Jump) poses), 1 (Running), 2 (Jumping), 3 (Spinning), `10h` (Moonwalking) and `17h` (Turning(Jump)) |
| screw attack | `$09A2` bit `0008h`, read only for movement types 3 and `14h` |
| speed glow and hit colour pulse | `$0B3E`, `$0A48`, each inactive at zero in the bounded proof |
| current and target palettes | `$C000..C1FF`, `$C200..C3FF`, both complete ordered RGB555 palettes. Source checks preserve every byte. Admit this RGB555 representation only if all 512 high bits are clear at the edge; otherwise report unsupported. The recorded state meets that condition |

### Harness, causality and limits

Compare the three metadata quantities before and after the frame's opportunity. Compare all
256 entries of each palette, the absence of recolouring requests, and whether the frame
applied the reset. Neither colour bank is read or written by the source interval.
The observed reset performs both stores once and leaves three already-ready metadata
quantities equal. Do-nothing matches all three recorded metadata values, but misses the
observed application. The stale reset vectors are constructed contrasts, independently of
this same-value recorded write. Temporary pose geometry scratch and stack activity have no
persistent palette owner and are excluded from this comparison. The packet verifies all
18 RAM uses / 36 bytes, one geometry content use / two bytes and four persistent bytes of
writes. The retained frame's hardware replay still owns the actual timed physical work.

For the captured frame only, `one-frame-479/palette-evidence/watch.py` cross-checks the
origin in `retained-time-479/receipt-profile-bridge/origin-wram-v1/` (corrected-origin WRAM
SHA-256 `bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db`) against the
source's completed-return origin and the pre-use snapshot. It checks intervening writes
using WRAM address normalization, including CPU mirrors. The ten independent guard
observations besides suit and her pose after must be current from their owners;
an origin carry for a borrowed fact requires the actual bounded writer-free history,
including interruption preservation. The first-frame diagnosis does not itself supply a runtime receipt. The checked origin ranges cover ten non-suit guard
fields, both metadata words and both banks. Every such range was equal and had zero
intervening writer sites in the captured interval. The suit latch was equal but had one
intervening writer, verified against the actual Power producer site. An origin-only
value therefore cannot establish that suit. Revalidate the actual selected Movement and
Appearance publications, completed equipment-dependent suit selection, actual Environment
and qualified room and grapple owners before the opportunity. Missing provenance, a later
writer (including a same-value write), an unqualified interrupt or an unsupported
neighbour stops admission. A source snapshot, a display layer blend for haze, a camera
scroll predicate and the room's empty palette-object return each have different contracts
from the required current observations.

`one-frame-479/palette-evidence/ref-v2.py --record` runs the observer reference and
source record check. Five histories and 16 unsupported alternatives pass, with 17 caught
faults. Across constructed and recorded histories the metadata
comparison is 15/15 versus 6/15 for do-nothing. The strictly recorded comparison remains
3/3 for both, and one application versus zero for do-nothing. The constructed shared
palette history changes current and target colours independently in the reset's frame and
compares both updated banks after it. Each of the 1,024 invented colours identifies its
fixture and slot. Restoring retired colours, rotating four slots, swapping positions and
swapping banks each fail. Only the source witness checks actual physical sites and
complete preserved source banks. Runtime integration must still prove the required indexed
replay and the current unsupported frame boundary. The wider colour effects remain outside
this amendment.

### Actions the opening reaches

`opening:N` in the vectors names cut N of row `chain-any-0003-DFD7` (native return N + 1).
The census reads the local lean row (`lean.bin` SHA-256
`4ec15ff682cca809452753bf1b4019afaffffd6072f1c18b16390d169595ce4a`, `hits.txt`
`e3155233c94009c8a636f384d294bd3c7d37bbf119fa1a645afdec3eeae2ad0f`). A cut has a palette
opportunity when the watched pose-update PC `91eb88` hits in it. Through the 184-return
opening, cuts 0 to 182 have one opportunity each (183). Cut 183 is in door transition, game
state `0Bh`, and has none; its target bank changes outside the opportunity.

| Action (pose codes) | First cut | Opportunities |
| --- | --- | --- |
| Running (`09`, `0A`, `0F`, `11`, `12`) | 0 | 94 |
| Spinning (`19`, `1A`) | 16 (request 17) | 14 |
| Jumping (`14`, `17`, `18`, `4E`, `51`, `52`) | 29 | 59 |
| Turning(Jump) (`2F`, `30`) | 36 | 12 |
| Landing(Jump) (`A4`, `A5`) | 57 | 2 |
| Still (`07`) | 156 | 1 |
| Moonwalking (`4A`) | 157 | 1 |

Every opportunity has the same neighbours: Power suit (`$0A74` zero), standard room
treatment (`$1982` = 2), both surfaces negative, grapple at `$C4F0`, `$0B18`, `$0CD0`,
`$0ACC`, `$0A4A`, `$0B3E` and `$0A48` zero, and screw attack absent. Visor and charge words
are `0601h` and zero at every cut, and neither bank changes between consecutive cuts 0 to 183.
The records therefore show same-value stores only. The passive instruction capture
`opening-clock-source-480/airborne-source/instructions.bin` (SHA-256
`8834307cf304fa80a354cd77368ba6da3e6c13ae01697aa6ce8990ba00fc89d6`) shows the executed path
for cuts 0 to 57: every opportunity runs the charge reset (`$91:D7B0`), the visor store
(`$91:D8A0`), the not-boosting test and carry-set return, never `$91:D717`. The 13 spin
opportunities in cuts 16 to 28 also run the spin branch and fall to the not-screw path. Cuts 58
to 182 rest on the lean records and the code path above. The stale-reset vectors for the new
actions are therefore original and constructed, as for Running.

The reference (`$TEMP/collab/night-20261122/observer-colours-scratch/ref-v3.py --record ROW`)
passes 18 histories and 31 unsupported alternatives, with 20 caught faults; the whole-frame
reference (§The whole-frame update) passes all 19 histories.
They include admitting a screw-attack spin, refusing every spin and the previous Running-only
admission. Across histories the metadata comparison is 54/54 fields against 24/54 for
do-nothing and 18/18 histories against 8/18. Over the record it admits 182/182 compared
opportunities (cuts 1 to 182), against 93/182 for the Running-only rule and 0 applications for
do-nothing. Metadata agrees 546/546 fields for both the model and do-nothing, since every store
is same-value. Both banks are unchanged at 182/182, and each recorded history matches its cut's
action. For indexed replay, the integrator compares the opening's palette opportunities at the
cuts above with the same declared fields as `R-ordinary-run-colours`.

### Palette budget

The old implementation is the palette area of the [domain inventory](../architecture/domains.md):
183 tokens for `samus_palette`, using the earlier engine's baseline. The bounded entry contributes 1.0 unit (one arm for
OrdinaryDryColours). The admitted actions widen the one OrdinaryDryColours arm without a new
outcome, so the units are unchanged. At k = 37.5 the increment is 37.5.
The general active colour effects remain outside this allowance.

## Her hurt colours and the push's poses (#753, spec §12)

Records: lean cache `SM_CACHE=$TEMP/collab/ci-cache/rows`, `chain-any-0006-E0B5` (0006), the
five tail hits at 609, 705, 801, 897 and 993. Scratch: `$TEMP/collab/knock-753/scratch/hurt.py`
(per record: her colour row in the current, target and shown colours, `18A8`, `0A48`, `0A4A`,
`0ACC`) and `hurt2.py` (the five hits side by side). All five are alike: the row holds the hurt
colours on the hit's record and two and four records later, the suit's otherwise; the target
colours never change; the shown colours equal the current ones every record; `0A48` reads 2 on
the hit's record, 3 the next, 40 at 38 records on, and 0 from 58 on.

### Sources

- **The count.** `$91:ED66` (the push's start) sets `0A48` = 1; `$90:EADF` sets it to 1 when
  `09C2` < `0A12` (the previous frame's energy) with `0A48` = 0. Both come before the palette
  handler in the frame.
- **HurtColours.** `$91:D6F7` (Samus palette handling, every frame) ends in `$91:D8A5`: with
  `0A4A` (super special palette flags: shinespark, crystal flash, hyper beam) non-zero it goes to
  `$91:D954` instead; with `0A48` = 0 it returns. Count 2: sound 35h, library 1, allowance 6
  (`Hurt`), unless `1F51` (cinematic) ≠ 0, or the new state handler is `$E8D9` with pose 54h;
  then the suit's colours. Count < 7: odd → `$91:A380` (the hurt colours, 20h bytes) by
  `$91:DD5B` into the current colours' row 4 (`7E:C180`..`C19F`); even → `$91:A3A0` (the
  intro's grey) in a cinematic, else `$91:DEBA` (the suit's colours). Then `INC`; at 28h
  (`$91:D911`) with the grapple inactive: movement type 3 or 14h → `$90:F084` command 1Ch (the
  spin's sound again); else `0CD0` ≥ 10h with shoot held → `0DC0` = 1 (the charge's sound
  resumes); with the grapple busy and below its cancel → sound 6, library 1, allowance 9. At
  ≥ 3Ch → 0.
- **The push's poses** (no entry; harness cases). 53h/54h and 4Fh/50h take the ordinary pose change (`$91:EB88`) and their clips
  play as any other: 0006 609 step 0 hold 2, 610 step 0 hold 1; 611 (50h) step 0 hold 8 counting
  down; 897 step 0 hold 2, 899 step 1 hold 10h, 902 step 1 hold Dh (`0A96`, `0A94`).

### RAM mapping

| Quantity | RAM | Decode |
| --- | --- | --- |
| hurt count | `0A48` | as is, 0 to 3Bh |
| her row of the current colours | `7E:C180`..`C19F` | 16 colours, BGR555 |
| `Hurt` | sound 35h, library 1, allowance 6 | |

Out of scope, original only: the drain's skip of `Hurt` (`$91:D8BD`..`D8CB`, handler `$E8D9` and
pose 54h), the grapple's sound 6 at 40 (`$91:D93F`) and the charge flag `0DC0` at 40 (weapons').
The push's start, read here, is samus-movement §12's "push started" output (`$91:ED66`).

### Harness note

Compared each frame: the hurt count and the 16 colours of her row of the current colours (and the
shown colours, which equal them). The colours are content: the recorded vectors use invented
fixtures, and 0006's hit windows are harness cases with the imported hurt and suit colours. The
`Hurt` request is compared at the audio boundary (MODEL decision 37). The flicker while she is
protected is drawing, not compared by the played run, and stays part 2's.

### The reference model

`$TEMP/collab/knock-753/scratch/hurtmodel.py`: HurtColours per frame over 0006 609-1057 (the five
hits and their counts to 0), seeded from each frame's starting record, the fall read from `09C2`
and the push's start from `0A52`. 449 frames: model 449, do-nothing 154 (the count moves on most
frames). `Hurt` is asked at 610, 706, 802, 898 and 994, the frame after each hit. No count reaches
40 while she spins or charges in 0006, so the spin's sound again is constructed only
(C-hurt-spin-again).

## The standing picture at the elevator

### Sources and inputs

| Entry | Source and mapping |
| --- | --- |
| ElevatorStandingPicture | [Bank 90](https://patrickjohnston.org/bank/90/) `$90:85E2..864D` selects the pose and step's maps. For side-facing standing `$90:868D..8696` includes the lower map. During rotation `$90:8C1F..8C4E` uses the projection at [bank 8B](https://patrickjohnston.org/bank/8B/) `$8B:8A52..8AD8`. Then `$90:8C94..8CC2` subtracts the signed pose Y offset and layer origin. `$92:8000..808C` selects art transfers separately from the room's regular queue. [Bank 80](https://patrickjohnston.org/bank/80/) `$80:9376..9415` transfers their strips during requested publication. |
| Position and scope | Room centre `$0AF6/$0AFA`, camera `$0911/$0915`, pivot `$0080/$0082`, earlier room matrix `$0078..007F`. The signed byte at `$91:B62D + 8 × pose` is the upward art offset. The existing importer negates it for the downward displacement `Frame::origin_y`. Compare calculated sprite origin with `$0B04/$0B06`. `$0A1C/$0A96` select the pose and step. The lock leaves the animation count at 10 in `$0A94` for this witness. No invincibility, knockback or special palette timer is active at `$18A8/$18AA/$0A68`. The gun remains closed. |
| Picture content | [Bank 92](https://patrickjohnston.org/bank/92/) `$92:9263` and `$92:945D` locate the upper and lower starting map indices for each pose. Add the displayed animation frame to each index. `$92:808D` maps these indices to ordinary spritemaps. The existing `import::samus` already imports both halves and frame tiles with the pose and displayed step as the key. |
| Tile content | `$92:D94E` points to each pose's four-byte frame descriptors. Each half selects a group and entry. The group pointers are at `$92:D91E` and `$92:D938` for upper and lower respectively. A seven-byte entry gives the 24-bit source and two 16-bit strip sizes. Split the contiguous source at the first size. A zero second size leaves its reserved region unchanged. The upper destinations are VRAM words `$6000/$6100` and the lower destinations are `$6080/$6180`. The existing `Video::transfer_samus_tiles` resolves the queued definition IDs through `SamusTiles` and graphics content. |
| Prepared and published drawings | `$7E:0370..058F` holds the pending OAM and its high table. The recorded timer occupies slots 0–16 and the body occupies 17–23 on cuts 165–241. These are witnessed positions in the shared draw order. The picture producer supplies pieces to the existing object writer. Compare the physical OAM lanes only once publication completes, retaining both lanes as expected when interrupted. `$071D/$071E` mark pending tile transfers and `$071F/$0721` hold the corresponding definitions. |

### Standing look admission

The pose definitions at [bank 91](https://patrickjohnston.org/bank/91/) `$91:B629`
and the [Movement pose mapping](samus-movement.decode.md) identify the standing poses.
For `Forward`, `Up`, `UpForward` and `DownForward` the right-facing poses are
`01h/03h/05h/07h` and the left-facing poses are `02h/04h/06h/08h`, respectively.
The Movement PadChart can supply all four aims with the gun stowed and the legs
together. Its striding variants `51h/52h` are straight jumps. There is no striding
standing pose and no standing `Down` pose. The screen-facing Power-suit pose has
a separate chest draw at `$90:8697`. All eight side-facing standing poses use
both halves through the carry-set return at `$90:8695`.

For both sides the delay pointers at `$91:B010` lead to four displayed steps for
`Forward`, two for `Up` and one each for `UpForward` and `DownForward`.
The art probe reads the pose's displacement, both ordered maps and tile strips.
At step zero the three aimed poses each differ from `Forward` in both maps and
in the tile strips. `Up` and `UpForward` share their step-zero art on each side.
The next `Up` step changes the upper map and tile strips. The corresponding `UpForward` step is absent. These aimed
clips fall outside the elevator reach's standing picture.

The reach at `$89:ACCD..AD17` calls `$90:F125..F14F`, which selects the
unaimed standing pose by facing. `$91:FB08` preserves the clip when that pose
matches the previous pose and initializes it when they differ. The movie takes
the left-facing `Forward` pose on cut 164 and holds step zero. At cut 165, with
the recorded centre, camera, view and step fixed, substituting left-facing
`Up` changes both maps and the tile strips. This is a comparison with original
art for another pose, outside the locked-reach witness. The constructed pose
cases check that the standing boundary refuses each aimed pose and striding
legs on both sides. The missing-step case requests a step present on the other
side and absent at the requested facing in the invented art.

### Witness and limitations

Use the `chain-any-0011-DF45` records from the elevator reach at 164 through
publication at 241. Samus reaches as the final ordinary room work runs. The standing
picture and tile regions first appear at 165. The final locked drawing is already
prepared at 239 and is retained through 240, while the prior body remains in live
OAM. Cut 241 publishes the final room picture. Compare the whole object pieces,
including the high bits, and all four imported tile strips. Other OAM contributions
belong to the room and timer owners.

The reference uses imported sprite content and the recorded locked pose and centre.
It matches 75 prepared drawings, 75 requested publications and 77 tile-window cuts.
A held body drawing first differs at 165. Keeping the reach cut's art changes 286
sprite VRAM words at 165. Reversing the sprite halves or rounding affine products
toward zero also fails at 165. Using the late shaft matrix fails at 166. The forward
constructed drawings check both facings, a nonzero step and sprite attributes.
The three-cut constructed video case starts with different live
and prepared pictures and strip identities. It catches early publication at
prepare, dropped publication at publish and packing the lower strips into an
absent second upper strip. The reference run leaves room and video scope distinct.

The station's setup replaces the entire sprite graphics window
(`$8B:C1ED` writes VRAM `$6000..7FFF`) before cinematic cut 283 begins the
settled-picture comparison. The scoped witness above supplies the claim for the
earlier room picture. The HUD grid at `$5820..587F` has its own producer and
survives this cinematic setup.

The previous scope has no production body picture or reserved body tile producer.
The separate object and video encoders remain available to consume those outputs.

### Budget

At `1ef154efc` the current production drawing has no owner for this picture.
For historical context, `tools size` gives 2,679 weighted tokens to
`sm_game/src/scene/samus_body.rs` and 2,511 to
`sm_game/src/scene/samus_position_tiles.rs` at `18cc7f8b0`. Those files include
general body, echoes and positions outside this standing scope.

The four elevator body arms add 4.0 spec units and 150 weighted tokens at k = 37.5.
