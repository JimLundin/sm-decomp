# How Samus looks, part 1, her animation's timing: decode notes

For the observer, the adversary, the integrator and the root only: implementers never see this
file. It holds the sources by entry, the RAM mapping, the importer's format, the harness note and
the Budget section of [samus-looks.md](samus-looks.md). Bank logs:
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
  (`poses.py`, from `data::assets::samus::import::map`). Per frame over the cache
  (`cachecheck.py`): **18,233 of 18,233** frames of play in the domain match on all three words
  (`0A94`, `0A96` through the step map, `0B3E`); the null model (nothing changes) matches 506.
  On the vectors (`vec.py`): 88 of 88.
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
The super special request, `0A2C` (its pose) and `0A32` (its command), is not State either:
the frame's start resets it to none and 0 (`$90:E695`), and a clip's end writes it (`$90:84B6`,
command 3, "transition animation finished"), which the pose change takes (`$91:EB88`). Lockstep
compares both words, so the RAM translation writes them from the frame's outputs: on a frame that
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
| standing restarts | movement type 0 at N, pose 1 or 2, pad `8B` at N & the shoot binding `09B2` (`$90:A3AD`); samus-motion's `StandingAnimation::Restarts` |
| wall jump readied | samus-motion's `WallTest::Readied`. In `cachecheck.py`, from the records: the pose unchanged, a kick step, `0A96` at N+1 on it and at N neither on it nor one before (the kick step is reached no other way) |
| vertical speed | samus-motion's `vertical speed after her moves` where its model runs the frame; elsewhere `0B2E`.`0B2C` at N+1 read as signed, then negated when `0B36` = 1 (up), so a rise the pull took past zero (`FFFF.F400` rising, samus-ceres-push 989) is the small `+0000.0C00`, as samus-motion gives it and as `BMI` reads it. The original reads `0B2E` alone at `$90:A5AC` (`CMP #5`, `BMI`), right after the falling movement (`$90:A590` → `$90:9168`) and before the pose change: `0B2E`.`0B2C` holds the size and `0B36` the direction, so the test is the size's, whichever way she moves (root decision a) |
| heading down | samus-motion's `heading down` where its model runs the frame; elsewhere `0B36` = 2 at N+1. The two differ on a landing (as her moves leave her she heads down; N+1 shows none), where the landing's clip starts either way |
| sounds waiting | `(0648 low − 0645) & 0Fh` at N: library 3's queue, `$80:9158` |

Which inputs samus-motion's model doesn't yet report is §Requests to samus-motion.

## Sources per entry

| Entry | Source |
| --- | --- |
| ClipPlays | `$90:801E`..`$90:802D` (the countdown: `DEC`, then `BEQ`/`BPL`, so 0 or below moves on), `$90:82DC` (the list `$91:B010` by pose, the byte at the frame), `$90:84E3`..`$90:8526` (a delay byte plus the buffer `0A9C`) |
| ClipControls | `$90:8324`'s table; `$90:84C7` (FEh nn: back nn bytes), `$90:84DB` (FFh), `$90:8344` (F0h-F5h: carry clear, the timer left at the countdown's 0); `$90:8315` (carry set: the timer from the byte at the new frame plus the buffer) |
| Breathing | `$90:8346` (F6h: `09C2` ≥ 1Eh, signed `BMI`); `$90:8315`; lists `$91:B298` (poses 1, 2) and `$91:B2A3` (27h, 28h) |
| ClipEnds | `$90:84B6` (FDh), `$90:839A` (F9h), `$90:83F6` (FAh, unused), `$90:848B` (FCh, unused): each sets `0A2C` and command 3, carry clear; `$90:8370` (F8h: not in the demo `E91D`, and `0A28` a ground jump, returns carry clear with no request; else installs the auto-jump handler `$90:E926`, samus-motion's, and jumps to `$90:84B6`); `$91:EB88`/`$91:EBAF` (the super special pose wins the frame) |
| WallJumpBranch | `$90:841D`..`$90:8489` (+1, +0Bh, +15h from the FBh byte; sounds 31h, 3Eh, 33h on library 1, `$80:9049`); `$90:8315` |
| HandOverSlowdown | `$91:EBA4`..`$91:EBC0` then `$91:F36E` (command 3: `0A94` += `0A9C`), after `$91:FB08` set the first hold with its own `$12` (the start's slowdown) |
| StartingClip | `$91:FB08`..`$91:FB8C` (pose = old pose: nothing; `0A9A` bit 15: kept; else frame and its byte plus the start's slowdown `$12`, `$91:FB0E`..`$91:FB56`), with `0A9A` from the pose's initialisation `$91:F468`'s table `$91:F4A2`: `$91:F4DC` (standing, shot direction up 0 or 9 from a previous pose's up: 1), `$91:F50C` (running from running: 8000h), `$91:F5AF` (15h/16h from 55h/56h: 1), `$91:F5EB` (85h/86h from F1h/F2h: 1), `$91:F624` (spin from spin or wall jump: 1), `$91:F9F4` (ball from types 4, 8: 8000h), `$91:FA56` (spring ball from 11h-13h: 8000h); the shot directions `$91:B629` byte 3 |
| StandingRestart | `$90:A3AD`..`$90:A3C7` (in `$90:A383`: poses 1, 2 with shoot held, `0A94` = 10h, `0A96` = 0), then the countdown `$90:801E` the same frame |
| WallJumpReady | `$90:9DB2`..`$90:9DD7` (in the wall jump check `$90:9D35`: `0A94` = 1, `0A96` = 0Ah, or 1Ah in 81h/82h), then the countdown `$90:8021` |
| SpinTurnedFar | `$90:9D55`..`$90:9D68` (`0A96` − 0Bh, or − 1Bh in 81h/82h, `BMI`), read by `$90:9D35` on the next frame; the lists `$91:B384`, `$91:B391`, `$91:B39E`, `$91:B491` |
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
| F9h eeee gg aa GG AA, FAh gg aa, FCh eeee pp PP, FDh pp | `end` (the pose is samus-motion's `AnimationEnds`) | 7, 3, 5, 2 |
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
  a walk off a ledge into a fall's pose, carries the rise (samus-motion's `FollowingTheGround`).
  No record rises so fast in a fall's pose (the fastest 2.4000): C-fast-fall-rising.
- **The end before the pose** (adversary 89, finding 2; adversary 90, finding 1; the root's
  decisions). samus-motion's step reads `animation_ends` before it chooses her pose, and this
  domain reads the pose chosen. The spec restricts its inputs (§1.5) so that the end is a function
  of the frame's start (step, time left, clip); all 78 ending clips of the game allow it. The
  output is the end's kind (none, an end, an end unless a jump), not a `bool`: whether an end
  unless a jump hands over is samus-motion's, by its own pad chart (`$90:8370`'s test of `0A28`,
  which `$91:81EC` writes from the transition table before the movement; the collision writers
  `$91:E8B6`..`$91:EADD` never write 4Bh, 4Ch, 19h or 1Ah). The second half reads back whether
  it did (`handed over`), for `HandOverSlowdown`. The frame is then two calls: the end first, the
  rest with samus-motion's results (spec §6). Over the cache the end's kind from the frame's start
  is the original's on all 350 end frames (151 `end`, 199 `end-unless-jump`, 9 of them given way),
  each with a pose change, and none elsewhere.

- **A dash lost after the animation.** `$91:DE53` is called from the movement handlers (before the
  animation: `$90:A3D0` standing, `$90:A551`.. the other halts, `$90:E5E9` the wall stop) and from
  the pose change's speed effects (`$91:EC76`, `$91:EC8E`, `$91:ECD3`, after it). On record every
  loss read as "after" (`dashing` from `0B3C` at N): item-charge-beam 52 and chain-0013-92FD 210
  run the plain pace on the frame the dash goes, with the pose unchanged. A loss in a running
  movement handler before the animation (the wall stop) would make `dashing` false that frame;
  samus-motion's model must report `dashing` as of the animation's turn (§Requests).
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
  samus-motion's "landed or ended the frame before". In the demos (`0A60` = E91Dh) it always
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
    `$90:E8EC` the elevator, `$90:E8AA` the Ceres escape's timer, which runs the normal one first,
    `$90:E8B0`) — 1,506;
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
- **Outputs not compared yet:** the end's kind (samus-motion's harness reads it as
  `AnimationEnds`; on record it is checked against the hand-over above),
  `turned far` (a function of the State; samus-motion reads it), the sounds (until the sound
  domain's request stream exists: spin 31h, space jump 3Eh, screw attack 33h on library 1;
  speed booster 3 on library 3).
- **Per-behaviour attribution** (frames on record that take each rule, `cases.py`): the time runs
  down 18,233, a shown step reached 4,581, again 500, rest 6, breathing loops 31 and on 2
  (samus-ceres-push 1085, 1316), ends 341 (151 `end`, 190 `end-unless-jump`) and 9 that gave way,
  branches 13 (9 plain, 4 screw attack; space jump: 0), a pose taken 4,205 (1,964 kept, 102 from
  step 1, 2,139 from step 0), restarts 29, readied 39, fast falls 4, rising holds 20 (boss-phantoon
  11, chain-0059-A293 1, samus-ceres-push 8: her pose the same and rising at N+1; the 12 outside
  the Ceres escape are the ones samus-motion's heading down brings into play in the harness, and
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
  the same and rising at N+1 and step 1 kept with 3 left, so samus-motion's heading down and
  record N+1's agree (not down). A landing from a straight jump (boss-kraid-defeated 297, 315)
  passes the hold's test by record N+1's direction but not by samus-motion's heading down, so it
  is no case. Checked in a scratch run of the harness with heading down from samus-motion's
  model and this note's frames (the Ceres escape's handler, the frame a message box opens, the
  frame a door starts): both hold, all 18,233 frames match (stage and laps 18,232, 1 masked), and
  the hold's exercised frames are 20. RunPace laps boss-kraid-defeated 988, chain-0012-91F8 638
  (C-lap-plain).
- **Null model:** nothing changes: 506 of 18,233.
- **Free run:** from a seed frame carry the four State words and take only the inputs; re-seed at
  each skipped stretch. A free run through samus-motion's model needs its outputs (§Requests).

## Requests to samus-motion

Answered in samus-motion's spec (§1.4 there, its round 18; its decode notes, "Round 18's
amendments", give the sources). Its step reports `dashing after her moves`, `dash began`, `dash
lost`, `handed over` and `heading down`, and from its round 19 `vertical speed after her moves`
(signed, as her moves leave her, beside `heading down`), besides `StandingAnimation::Restarts`,
`WallTest::Readied` and its pose, and its `AnimationEnds` takes the end's kind as its input and
gives way on an end unless a jump when its pad chart's pose is a take-off aimed forward (4Bh,
4Ch) or a spin (19h, 1Ah). The harness's decodes above agree with those outputs on every frame
these rules read: samus-motion's `dashing after her moves` differs from "`0B3C` at N, or a dash
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
`scan.sh --mirror` on the spec and these notes: 0 of 15 entries single-routine (0 %, limit
30 %), 0 unsourced.

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

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on samus-motion. An
area's budget is k × its units. The old code (for reference, not a bound) is the `tools size`
score at `f1df56ede` of `game/src/animation.rs` (1,245.5) and the animation's part of
`game/src/pose.rs` (`reset_animation` and `run_super_special`, 197.75, measured as an extract
alone): 1,443.25 together, not split by area (the old module interleaves the clip, her moves'
writes and the run's pace in one handler). The few lines of `pose.rs`'s `refresh` and `change`
that set and reset `new_pose_frame` are left out: a handful of tokens inside functions that do
the pose's other work. The old code modelled neither the boost stages (`blocks_world.rs`: the
engine keeps no speed booster's charge) nor any slowdown (`animation::frame_buffer` returns 0),
so its figure covers the clip and the moves; the run area has almost no old counterpart. The
budget is an indicator: the fit is score ÷ budget, and being
over it costs in proportion to how far over, never as a wall.

Budget: 37.5 × 37.7 units = 1,412 (clip 812, moves 225, run 375), 15 entries, 33 arms, 88 vectors
(82 constructed, 6 recorded).

Old code (old 1,443): `game/src/animation.rs` and the animation's part of `game/src/pose.rs`,
together, not split by area.

- Area clip: 1.4, ClipPlays, ClipControls, Breathing, ClipEnds, WallJumpBranch, HandOverSlowdown,
  StartingClip
- Area moves: StandingRestart, WallJumpReady, SpinTurnedFar, FastFall, RisingHold
- Area run: RunPace, BoostStages, BoostingLap
