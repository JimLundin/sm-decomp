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
| slowdown | `0A9C` at N+1, the animation's own buffer: `$90:8078` the X speed divisor `0A66` (0 out of liquids); `$90:80B8` water: 3 (`$90:80CB`) with no gravity suit test; acid 2 (`$90:8219` → `$90:824C`, `$90:9E95`) with no gravity suit test either; lava (`$90:81C0`) 2, and 0 with the gravity suit (`$90:81DB` → `$90:820F`). FX types 8, Ah and Ch run `$90:8077` (RTS) and leave it as it was: the liquids' domain must carry it |
| the start's slowdown | `$91:FB08`'s `$12`, worked out again for the pose she takes: the gravity suit gives `0A66` (`$91:FB0E` → `$91:FB3F`); otherwise the FX surface `195E` (liquid physics on) or the lava/acid surface `1962` against the **new pose's** bottom boundary (`$91:FB1D`..`$91:FB3D`): 3 in water (`$90:9E93`), 2 in lava or acid (`$90:9E95`). In the cache both are 0 on every frame of play, so `cachecheck.py` takes `0A9C` for it |
| run held | pad `8B` at N & the run binding `09B6` |
| handed over | `0A32` = 3 at N+1 with `0A2C` a pose (bit 15 clear): the super special "transition animation finished" took the frame's pose. On the 311 end frames on record it agrees with "an end, or an end unless a jump with `0A28` at N+1 not 4Bh, 4Ch, 19h or 1Ah outside the demo's `0A60` = E91Dh" (the 8 that gave way), and it is false on every other frame |
| dashing | `0B3C` at N ≠ 0, or a dash began |
| dash began | `0B3C` 0 at N, non-zero at N+1 (`$90:9796`, `$90:97CC`, in the movement handler, before the animation) |
| dash lost | `0B3C` non-zero at N, 0 at N+1 (`$91:DE53`; also `$91:E655`, the pause screen's equipment change) |
| standing restarts | movement type 0 at N, pose 1 or 2, pad `8B` at N & the shoot binding `09B2` (`$90:A3AD`); samus-movement's `StandingAnimation::Restarts` |
| wall jump readied | samus-movement's `WallTest::Readied`. In `cachecheck.py`, from the records: the pose unchanged, a kick step, `0A96` at N+1 on it and at N neither on it nor one before (the kick step is reached no other way) |
| vertical speed | samus-movement's `vertical speed after her moves` where its model runs the frame; elsewhere `0B2E`.`0B2C` at N+1 read as signed, then negated when `0B36` = 1 (up), so a rise the pull took past zero (`FFFF.F400` rising, samus-ceres-push 989) is the small `+0000.0C00`, as samus-movement gives it and as `BMI` reads it. The original reads `0B2E` alone at `$90:A5AC` (`CMP #5`, `BMI`), right after the falling movement (`$90:A590` → `$90:9168`) and before the pose change: `0B2E`.`0B2C` holds the size and `0B36` the direction, so the test is the size's, whichever way she moves (root decision a) |
| heading down | samus-movement's `heading down` where its model runs the frame; elsewhere `0B36` = 2 at N+1. The two differ on a landing (as her moves leave her she heads down; N+1 shows none), where the landing's clip starts either way |
| sounds waiting | `(0648 low − 0645) & 0Fh` at N: library 3's queue, `$80:9158` |

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
| StandingRestart | `$90:A3AD`..`$90:A3C7` (in `$90:A383`: poses 1, 2 with shoot held, `0A94` = 10h, `0A96` = 0), then the countdown `$90:801E` the same frame |
| WallJumpReady | `$90:9DB2`..`$90:9DD7` (in the wall jump check `$90:9D35`: `0A94` = 1, `0A96` = 0Ah, or 1Ah in 81h/82h), then the countdown `$90:8021` |
| SpinTurnedFar | `$90:9D55`..`$90:9D68` (`0A96` − 0Bh, or − 1Bh in 81h/82h, `BMI`), inside the wall jump check `$90:9D35`, in the movement of the frame that reads it, from `0A96` and the pose `0A1C` as that frame starts (§Findings, "Turned far from the frame's start"); the lists `$91:B384`, `$91:B391`, `$91:B39E`, `$91:B491` |
| FastFall | `$90:A5AC`..`$90:A5C5` (in `$90:A58D`, poses 29h, 2Ah, 67h, 68h: `0B2E` ≥ 5 signed, `0A96` < 5 signed: `0A94` = 8, `0A96` = 5); lists `$91:B34A`, `$91:B353` |
| RisingHold | `$90:8011`..`$90:8050` (poses 4Dh, 4Eh: `0B36` ≠ 2, `0A96` = 1 and `0A94` = 1 → `0A94` = 4, then the countdown); list `$91:B326` |
| RunPace | `$90:84EF`..`$90:851D` (`0B3C` ≠ 0, movement type 1: the delay from `$91:B5D3`, or `$91:B5DE`'s list for `0B3F` with the booster); `$90:8538`..`$90:857B` (an instruction byte with run held, no booster: frame 0, `$91:B5D3`'s first byte) |
| BoostStages | `$90:857D`..`$90:85D8` (the word `DEC` of `0B3E`, `BIT #$00FF`; counter + 100h unless bit 0400h; `$91:B61F` the timers 1, 1, 1, 1, 2; frame 0 and the stage's first byte); `$90:9796`..`$90:97A6` (begun with the booster: `0B3E` = 0001h) and `$90:97CC`..`$90:97D2` (without: 0); `$91:DE53` (cancel: `0B3E` = 0 when `0B3C` ≠ 0); the boosting sound 3, library 3, `$80:914D` |
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
  - `StandingRestart`, the restart's hold (10h, `$90:A3AD`..`$90:A3C7`): C-restart-adjusted at
    25, step 0 run down to 24;
  - `FastFall`, the speed and the hold (5 and 8, `$90:A5AC`..`$90:A5C5`): C-fast-fall-adjusted at
    3.0000 and 12, a fall at 3.8000 jumps to the fast fall step, run down to 11, where the
    game's 5 would let the clip play on;
  - `RisingHold`, the hold (4, `$90:8011`..`$90:8050`): C-rising-adjusted at 7, run down to 6;
  - `BoostStages`, the laps each stage lasts (`$91:B61F`): C-laps-adjusted-began and
    C-laps-adjusted-up and C-laps-adjusted-boosting at 3, 2, 4, 1 and 3 laps (a dash begun:
    stage 0's 3 laps; stage 1's last lap ends: stage 2 with its 4; boosting, its last lap ends:
    stage 4 with its 3).
  The root made the first four parameters (2026-11-11): their numbers already sit in the model's
  tuning, and the goal is that each behaviour can be adjusted.
- **A dash lost after the animation.** `$91:DE53` is called from the movement handlers (before the
  animation: `$90:A3D0` standing, `$90:A551`.. the other halts, `$90:E5E9` the wall stop) and from
  the pose change's speed effects (`$91:EC76`, `$91:EC8E`, `$91:ECD3`, after it). On record every
  loss read as "after" (`dashing` from `0B3C` at N): item-charge-beam 52 and chain-0013-92FD 210
  run the plain pace on the frame the dash goes, with the pose unchanged. A loss in a running
  movement handler before the animation (the wall stop) would make `dashing` false that frame;
  samus-movement's model must report `dashing` as of the animation's turn (§Requests).
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

Her sprite from the step (`$90:85E2`..`$90:8A4B`: the halves, the chest sprite, the origin's
offsets, the tiles uploads `$92:8000`), the flicker (`$90:85E2`'s invincibility test), the
palettes (`$91:D6F7`: the suits, the visor `$91:D83F`, the charge, the screw attack's and the
speed booster's palettes `$91:D9B2`/`$91:DA5C`, the hurt flash `$91:D8A5`), the atmospheric
graphics (splashes, dust, `$90:80E6`, `$90:A413`), and the echoes' pictures. Old code:
`game::samus_drawing` 605, `game::samus_palette` 183, `game::atmospheric` 1,082 (domains.md).

## Scan

`tools/cleanroom/scan.sh` on the spec and the vectors: 0 (address-form 0, address-known 0, name 0,
enum-order 0, ram-field 0, single-routine 0, entry-point 0, route 0). `score_scan.sh`: 0.
`scan.sh --mirror` on the spec and these notes: 2 of 19 entries single-routine (10 %, limit
30 %), 0 unsourced. FootContact and SpinStarts each have one source mechanism; the spec
states their observable opportunities and leaves the public API form to the implementer.

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

Budget: 37.5 × 56.7 units = 2,125 (clip 1,525, moves 225, run 375), 21 entries, 52 arms,
157 vectors.

Old code (old 1,525): `game/src/animation.rs` and the animation's part of `game/src/pose.rs`,
together, not split by area.

- Area clip: 1.4, ClipPlays, ClipControls, Breathing, ClipEnds, WallJumpBranch, HandOverSlowdown,
  StartingClip, FootContact, SpinStarts, SpinEnds, AtTheDoor, PerformedAnimationDecisions,
  SuspendedAppearance
- Area moves: StandingRestart, WallJumpReady, SpinTurnedFar, FastFall, RisingHold
- Area run: RunPace, BoostStages, BoostingLap


## Stride, spin and passage sound opportunities

### Sources and existing interface

`FootContact`: `$90:A3E5` running dispatch moves X/Y first, then `$A3EE..A3FF` reads
`0A96` and exactly `0A94 == 1`; contact content is the nonzero low byte at `$90:A424`
indexed by that animation frame. Import this as a contact-step set through `Imported::frames`,
not a new execution table or numeric movement code. `$A401` requests graphics before sound
suppression; `$A404..A419` checks `1F51`, `179C`, `0A68`, then `0B3E & 0400`.
`$A41B/A41E` asks for sound 6 in library3, cap 6 (`$80:914D`). The sound suppressors have
no effect on the already-run graphic request. Current `Clip` has no contact markers;
`Playhead` already supplies step/time. `Samus::movement` already supplies Running and
`Samus::animation.boost.stage` supplies boosting. `Inputs::figure` is the frame start;
`figure_after` is the animation's final figure, not the contact-phase figure after movement.
The contact opportunity must be captured where running movement has executed, before
`samus_appearance_bridge::animate`, not discovered from a resulting sender write.

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
The engine needs a typed `preparation_movement` alongside its pose history; current movement
still comes from `Samus.movement`. `Inputs::figure` at frame start cannot substitute for
the actual preparation snapshot. Model output
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

Vector source alias `M0` names row `chain-any-0003-DFD7`. The 45 `[[opportunity]]` cases supplement the 98 existing animation vectors. Each compares
semantic sounds, independent graphical contact and unchanged four-quantity animation State;
recorded guard cases use invented content, never copied contact tables or clip holds.
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

## Causal animation observations and continuation

`PerformedAnimationDecisions` and `SuspendedAppearance` observe the existing animation rules and
section6 order. Sources/RAM mapping remain the entries for move-driven playhead changes,
`ClipPlays`, `RunPace`, `BoostStages`, `StartingClip`, `HandOverSlowdown` and section9's independent
contact/selection opportunities. No animation rule or palette/liquid/echo producer is added.

The existing public `Ahead::of` and read-only contact/pose-selected opportunities supply their
current named answers. The whole public `update` currently combines earlier move-driven
changes, current-clip play, selected-clip initialization and dash loss. Complete pre-effect
observation therefore requires a confined interface amendment. Exact Rust form is left to the
implementer; ordinary and suspended execution must use the same existing rule implementation.
No expected future figure or second `update` supplies the earlier playing answer.

### Invented vectors and proof

The additive `suspended_update` schema inherits existing Animation, Opportunity or Work cases,
including complete content, Rules, input and ordinary output comparisons. All added appearance
cases are invented. C-op-kept-clip compares countdown before running-to-running KeepPlayhead;
C-kept-run-advance adds the ordinary case that moves from step1 to step2 of the old running clip
with its invented hold5, while the selected clip's corresponding hold is9. C-op-kept-advance
observes the current-clip advance and keeps its resulting hold5. This distinguishes actual
playing from an anticipated selected figure or restored earlier playhead.

C-op-current-play counts the old jumping clip down before starting the actual falling clip.
C-op-restart-play and its adjusted version show the actual restart hold before countdown,
including altered restart Rules. C-op-contact-order preserves Footstep then PlainSpin from two
independent read-only opportunities, with the complete animation State unchanged. A suppressed
contact retains its graphics. These composed read-only cases are not an extra animation update.

The private timing-opportunities-479/reference.py derives the observation quantities directly
from inherited inputs, named Rules and invented clips, without importing the model or using
expected future State. Each checked quantity and adjacent observed order has a caught comparison mutant. Evaluation
faults substitute the future clip for current playing and repeat/reverse contact/selection;
the checks catch these actual evaluation changes. No event-list roundtrip is claimed as
continuation proof. These are independent selected-observation checks, not actual
model/production checkpoint proof.
The confined implementation must exercise every actual offered/completed opportunity, including
unlisted ones, and compare complete State, sound/graphic order and input/output progress with
ordinary execution. Existing ordinary/opportunity/work vectors retain every result.

### Integration obligations

At contact, production supplies the actual stride figure/playhead and fresh owner suppressors;
at selection it supplies the actual selected figures. Playing consumes the current clip after
its earlier move changes; initialization consumes the completed playing result and actual
accepted selection. Boost stage can be read fresh at multiple real opportunities without
advancing animation. Prepared movement history stays caller-owned, saved independently of pose
history. Mutable borrowed environment/colour/cinematic/sound premises require actual freshness.
The selected source timing arms, source initialization context, exact costs and hardware
continuation remain outside this model. Palette/graphics/sound transport, publication
materializations, complete runtime codec/atomic rollback and indexed replay/null/fault proof
remain explicit integration obligations.

### Changed-resume and serialized restoration witnesses

The additive `resume_case` pairs change an external semantic premise before its actual request
or decision, and after its actual answer but before dependent effects. Future unanswered inputs
are fresh; an already returned/local chosen answer is retained with its causal inputs and
applied once. Before/after pairs prevent accepting both premature caching and late recomputation.
All content/world quantities are invented. There is no source hardware guard or timing result
in this schema. Complete interrupted work and provider/output progress must serialize, the
original objects must be discarded, and a fresh deserialized instance must reproduce every
query/effect/output exactly once. Clone-only equality cannot satisfy these tests.

Movement replaces the slope-bearing world after ordinary travel, either before floor support
querying or after its actual lifted answer. Appearance replaces actual accepted selection after
current-clip playing, either before or after selected initialization was decided, and changes a
contact suppressor either before its read or after its completed publication. Missing production
freshness remains an integration limitation; these clean observations retain model totality.

The private `timing-opportunities-479/reference_v2.py` continues a selected semantic reference
calculation after serializing and discarding its interrupted instance. Six changed-resume
cases agree; eight evaluation faults catch caching unmade floor/selection/suppression inputs,
requerying/recomputing actual retained answers, repeating a completed contact and omitting a
pending answer during restoration. These are selected reference decisions, not a claim that
actual clean model State or all ordinary vectors have been serialized and replayed. The
confined implementation retains that complete actual-model proof obligation.
