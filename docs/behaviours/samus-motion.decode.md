# Samus moving through the world: decode notes

For the observer, the adversary and the domain harness's decoder only. Implementers never see this
file (MODEL §5). It holds every source the spec draws on: the original's routines, its RAM and its
codes. Addresses are the bank logs' (`$TEMP/c4a/logs*`, https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 32 rows in `tools/oracle/samus_motion_rows.txt`, cached in
`$TEMP/collab/more-caches/cache`, were dumped per frame (`$TEMP/obs-ground/dump`, driven by
`$TEMP/respec-motion/one.ps1`, output in `$TEMP/respec-motion/out`) and read over time. The
previous spec's arithmetic, which was pinned from the bank logs, was reused only for exact values
and edge cases. Vectors come from `$TEMP/respec-motion/scripts/vec.pl` (frame selection), `tag.pl`
(entries) and `lead.pl` (perceivable quantities and lead-ins); the constructed ones were worked by
hand from the rules, and `cons.pl` rewrote them to perceivable quantities.

**Row names.** The vectors and the spec name the chain rows by opaque ids, because a chain row's
name carries a room header's address:

| Id | Row |
| --- | --- |
| r01 | chain-0012-91F8 (Landing Site) |
| r02 | chain-0013-92FD |
| r03 | chain-0014-96BA |
| r04 | chain-0057-9FBA |
| r05 | chain-0059-A293 |
| r06 | chain-0097-ACF0 |
| r07 | chain-0100-B07A |
| r08 | chain-0106-AD5E |

**Frame selection for recorded vectors.** Record N is the State before, record N+1 after; the pad
is record N's. Taken when: the frame counter `05B6` advances by one (no lag); `0A66` is 0; the
movement type `0A1F` is one of the domain's; the pose `0A1C` and the radii `0AFE`/`0B00` are the
same at N and N+1 (or, for a landing, `0B36` goes from non-zero to 0 with y's subpixel FFFF); the
slope flag `0DBA` is 0 at both; extra run `0B42` is below 2.0 (no speed booster); the x move equals
± the frame's total speed `0DBC`.`0DBE`; in the air, `0DC6` is 0 at N+1 and the y move equals the
frame-start vertical velocity, or 0 at the apex; on the ground, y unchanged with y.sub FFFF and the
floor (y + y radius) on a 16 px line. Excluded rows: chain-0220-D017, chain-0224-D0B9,
chain-0232-D5A7, area-maridia (water), terrain-quicksand, samus-ceres-push (pushes). area-brinstar
and samus-square-slope-reserve give few or none (elevator, slopes). At most one vector per kind
(activity, vertical, drive, pad direction, run) per row, nine per row. The pad's bindings are taken
as the defaults (jump A `0080`, run B `8000`, shoot X `0040`); a row with other bindings would be
misread (not checked: `09B2`-`09B6` are not in the dump).

**Lead-ins (`lead.pl`).** A vector's start must be *plain* (spec §1.2's last paragraph): `0B3C` ≠ 0
or `0B42` = 0; `0B4A` = 0, or 1 with `0B46` > 0; `0B36` = 1 only with a non-zero magnitude;
`0B36` = 2 with magnitude 0 only in an airborne movement type (2 outside the take-off poses, 3, 6,
8, 12h, 13h, 14h); `0B36` = 0 only with magnitude 0 outside those types. From the checked frame
`lead.pl` walks back while each frame passes the selection test above, with radii free to change
(a pose may change only from movement type 0, 1 or 10h on the ground into a jump, spin, fall or
turn, from a take-off pose into a jump, or by aim alone, which the x-move test keeps clear of the
arm-pump step), to the nearest plain start, at most 120 frames. A world of open air takes the floor
of a ground frame met on the way if every later airborne frame stays above it. A start with a bounce
in play drops the vector. Of 149 recorded frames, 118 were kept (gaps-v3 dropped six, below), 23 of them with a lead-in of 1 to
12 frames; 31 are dropped (no plain start reachable). The two constructed moonfall vectors were
dropped: they set the moonfall as a start value, and no lead-in through a turn's end is worked yet.

## State

| Quantity | RAM | Decode |
| --- | --- | --- |
| position | x `0AF6`.`0AF8`, y `0AFA`.`0AFC` | as is |
| horizontal speed | `0B46`.`0B48`, facing `0A1E`, `0B4A` | magnitude `0B46`.`0B48`; leftward when facing left (`0A1E` low byte 4) with `0B4A` ≠ 1, or facing right or the screen with `0B4A` = 1 |
| dash | `0B42`.`0B44` | as is |
| vertical speed | `0B2E`.`0B2C` (a magnitude), `0B36` | `0B36` = 2: + magnitude; else − magnitude (two's complement over 32 bits) |
| bounces | `0B20` low byte | 0, 1, 2 |
| pose | `0A1C`, `0A1E` | body and action from the movement type `0A1F` (below); facing `0A1E` low byte 4 Left, 8 Right, 0 Screen |

What she remembers (spec §1.2), for the decoder and the adversary only: carrying on and skidding
are `0B4A` (0 follows the pad, 2 carries on, 1 skids; a skid of zero is `0B4A` = 1 with `0B46` =
0); a dash to lose is `0B3C` = 0 with `0B42` ≠ 0; the moonfall is `0B36` = 0 in the air; a spin
carried on is `0A27` ∈ {3, 14h}. None is in the spec's State. The horizontal speed's sign is exact
except for the skid of zero, which the spec states as a behaviour.

Pose decode by movement type: 0 Upright Still (poses 00, 9B face the screen); 1 Running; 2
Jumping (TakingOff for 4B, 4C, 55-5A); 3 Spinning; 4 Ball, Still for 1D, 41, else Rolling; 5
Crouched Still; 6 Falling; 8 Ball Falling; E, 17, 18 Turning; F by pose: 35, 36, F3 Crouching, 37,
38 Morphing, 3B, 3C StandingUp, 3D, 3E Unmorphing; 10 Moonwalking; 11 SpringBall (Still for 79,
7A, else Rolling); 12 SpringBall Jumping; 13 SpringBall Falling; 14 WallJumping; 15 AgainstWall.
Other types are outside the domain (0A knockback, 16 grapple, 19 damage boost, 1B shinespark).

**RAM shape** (fields decoded as one word copied as-is): none. Positions, speeds and the dash join
two words; the two speeds are signed from a magnitude and a direction; bounces are mapped.

**Inputs.** pad held `8B`, newly pressed `8F`; equipment `09A2` (0100 hi-jump, 0200 space jump);
spin turned far: animation frame `0A96` ≥ 0B (≥ 1B in poses 81, 82); elevator moving: `0E18` ≠ 0.

## Sources per entry

Each entry lists only the routines or branches that carry its content, not the helpers it shares
with others (adversary-2 §1's count): 2 of 18 entries (11 %) are single-routine, marked *,
and `scan.sh --mirror` agrees.

| Entry | Source |
| --- | --- |
| SpeedBuildsAndBrakes | `$90:9BD1` and its table (the speeds by situation), `$90:9A7E` (speed-up), `$90:9B1F` (braking), the standing, crouching, transition, ran-into-wall and facing-forward handlers' tail stops in `$90:A337`'s table, `$90:A521` (the ball) |
| Dash | `$90:973E`, `$90:97D5` (the cap), `$91:DE53` (losing it), `$90:E5CE` (the wall stop's share) |
| TurningRound | `$91:F543` (turn start and spin reverse), `$90:A68F` (turning handler, its sink and its end), `$90:A67C` (the creep), `$91:81A9` with `$91:82D9`, `$91:8304` and `$91:ECD0` (letting go: the failed transition lookup's stop command clears `0B4A` after the move) |
| AimingAndShooting | `$91:EADE` (the step), `$94:971E` (its block stop), `$90:A3AD` (the shot) |
| RidingTheElevator* | `$90:A392`, `$90:A397` (two branches of the facing-forward handler) |
| Jumping | `$90:98BC` (the speed table), `$91:FBBB` (take-off), `$91:F624` and `$91:F8D3` (jump starts), `$90:A488` (space jump) |
| FollowingTheGround | `$90:923F` (the pull), `$90:E61B` (walking off) |
| MovesAsShePosed | `$90:A337` (the handler of the frame-start pose), `$91:EB88` (the pose taken after) |
| RisingAndFalling | `$90:90E2` and `$90:913A` (gravity), `$90:910C` (the limit), `$90:8FDC`, `$90:9046` and `$90:90C4` (the rise's end: `0B2E` read as signed, ends when negative), `$91:F22B` (bounce 2 at 0.0) |
| DriftInTheAir | `$90:900D`, `$90:9174`, `$90:91A8` (normal jump and fall), `$90:906C` and `$90:9093` (spin; its stop still calls `$90:8EA9`, which adds the extra run speed) |
| Moonfall | `$91:EBAF`, `$91:F36E` (the hand-over), `$90:913A` (no limit with direction 0), `$90:9D35` (the wall-jump test reads no direction), `$90:99CD` (the kick sets direction up) |
| WallJump | `$90:9D35` (arming and kick test), `$90:90B3` (one distance, `$90:9E9F` = 8, for both), `$91:FA0F` (kick), `$94:967F` (the probe) |
| Landing | `$91:F010`, `$90:A640` (upright and changing), `$91:F1FC`, `$91:F25E` (the ball) |
| BlocksStopFlush | `$94:9722`, `$94:976B` (a 0.0 move skips block collision), `$94:8F49`, `$94:8F82` (horizontal and vertical block collision), `$94:8D2B`, `$94:8DBD` (stepped), `$94:9543`, `$94:959E`, `$94:95F5` (the order), `$90:E4AD` (the limit) |
| WallsAndCeilings | `$90:E5CE` (wall), `$90:E606` and `$91:EFDF` (ceiling) |
| SolidEnemies | `$A0:AAC8`, `$A0:AA53`, `$A0:AABC` (overlapped: next enemy, then 0 so blocks run), `$90:9D80`, `$90:9DF8`, `$90:93B1`, `$90:93EC` and `$90:9440` (enemy test first; on a hit the move skips block collision, as the bank log's zebetite-skip note says) |
| SlopesCarry | `$90:9376`, `$94:87F4` (seat after every horizontal move, sets `0DBA`), `$90:924C` (the pull reads `0DBA`), `$94:976B` (a 0.0 vertical move skips block collision), `$94:976F` (cleared by a non-zero vertical move), `$94:8E38` (square slope stop sets it), `$94:86FE` (rising into a non-square ceiling slope: `$12 = max(0, ...)` and `$14 = 0`, so no move), `$94:87E4` (the floor slope stop is `max(0, ...)` too: never upward) |
| WordwiseTopSpeed* | `$90:9A99` |
| PoseFollows | `$91:EB88` (one change a frame: super special `0A2C`, then special `0A2A`, then `0A28`), `$91:E8B6` (the vertical collision result `0DC6` overwrites the pad's `0A28` and sets command 5), `$91:EADE` (the wall stop overrides both); the spec's three items fold the wall stop into `WhatHerMovesMet` and the lookup failure into the chart's last rows, `$91:EC85` falling into `$91:EC8E` (the dash goes with the speed) |
| PadChart | `$91:81A9` (a row fits when its newly-pressed and held masks are all set; first fit wins; same pose: no change), `$91:81F4` (bindings to default bits), the transition table `$91:9EE2`..`$91:B00F`, `$91:F404` (crouched into standing: the bottom edge kept, y − 5; `$91:8087`'s own y − 5 is dead); the last rows: `$91:81DB` (pad 0 or no fit, unless the pose's table is empty), `$91:82D9`, `$91:8304` and `$91:8332` (the effect per movement type), the pose definitions `$91:B629` byte 2 (the pose with no buttons), `$91:EC50` (decelerate), `$91:ECD0` (stop), `$91:EC85`, `$91:EC8E`; `$91:8146` (movement type Fh: no lookup at all, the changing-body row) |
| Aiming | the table's aim rows per pose, `$91:E9F3` (landing keeps the aim, down becomes forward), `$91:EB74` (the wall stop keeps the aim) |
| WhatHerMovesMet | `$91:E8F2` (falling poses, `$91:E921`), `$91:E95D`..`$91:EA47` (landing poses, ball bounces), `$91:EABE` (wall-jump pose), `$91:EADE` (ran-into-wall poses, the aim step's block) |
| AnimationEnds | the animation delay table `$91:B010` (its go-to-pose instructions), `$91:EB88` (the super special pose wins) |

**The pose chart (spec §7).** The pad chart is the transition table regrouped by situation, with
facing made relative (forward, back) and aim moved into one rule, so neither the table's layout,
its pose numbers nor its row order appear. Situations map to movement types `0A1F`: standing 0
(and the landing poses A4h-A7h, E0h-E7h), running 1, taking off 2 (4Bh, 4Ch, 55h-5Ah), straight jump 2,
spinning 3, ball 4 and 8, crouched 5, falling 6, turning Eh, 17h, 18h (crouched: 43h, 44h, 97h-9Ah,
A2h, A3h), moonwalking 10h, wall jumping 14h, against the wall 15h. A moonwalk pose's `0A1E` is
the way she walks (4Ah: 04), so its spec facing is that way. The effects are the commands `0A2E` 1
(decelerate), 2 (stop), 6 (kill x speed, falls through into 8) and 8 (kill run speed); command 5
(vertical collision) is the `Landing`, `FollowingTheGround`, `WallsAndCeilings` and `WallJump`
entries. Command 6 has no RTS and falls through into command 8, so a change that kills the run
speed also zeroes and cancels the dash: `PoseFollows`'s last fact.

**Pose-chart vectors.** `$TEMP/pose-chart/tr.pl` lists every recorded pose change in the
`$TEMP/gaps-v3/out` dumps (2,651 changes, 343 kinds: most are aim changes while running);
`$TEMP/pose-chart/tt.pl` prints the transition table and pose definitions from the ROM by pose;
`$TEMP/pose-chart/gen.pl` turns the list in `$TEMP/pose-chart/list.txt` into the `[[pose_step]]`
entries, with the pose decode above and aim from the pose's name. The record is N, the pad N's,
just pressed N's pad less N−1's, the after pose N+1's. `met` was set by hand from the movement
types (airborne to ground: floor; ground to falling: no_floor; to 15h: wall; to 14h: wall_kick;
from 0Fh, Eh, 17h, 18h or a take-off pose: animation_end).

**The chart's ranks.** The spec's rows are ranked (pressed first, more buttons
first, "beats" for the rest) rather than listed in the table's order. `$TEMP/pose-chart/chk.pl`
with the rows in `$TEMP/pose-chart/spec.pl` compares the ranked chart with `$91:81A9`'s first fit
over every pad (each of 8 buttons not held, held or newly pressed; left with right and up with
down skipped) for the right-facing poses 01h, 03h, 05h, 07h, A4h, A6h, E0h-E6h, 09h, 0Bh, 0Fh, 11h,
4Bh, 55h-59h, 4Dh, 51h, 15h, 17h, 69h, 6Bh, 19h, 1Bh, 81h, 29h, 2Bh, 2Dh, 6Dh, 6Fh, 27h, 71h, 73h,
85h, 1Dh, 1Eh, 31h, 79h, 7Bh, 7Dh, 7Fh, 25h, BFh, 8Bh, 8Dh, 9Ch, 2Fh, 87h, 8Fh-A0h, 43h, 97h, 99h,
A2h, 35h-3Dh, F1h-FBh, 4Ah, 76h, 78h, 83h, 89h: 0 mismatches. C1h and C3h (aimed moonwalk
turns) are never a moonwalk row's next pose (4Ah's jump row names the unaimed BFh), but they are
reached: the turn is re-picked from the moonwalk's shot direction (`$91:F9EA`, the turn re-pick
row below), so the spec's cell says "aim kept" (adversary-14 F2). The ground
turns aimed up or down-right (8Bh, 8Dh) have only press-jump rows, and 9Ch's table is empty, so
it turns as in the air.

**Both facings and the drawn gun (chart-left, 2026-11-07).** `chk.pl` now also runs every pose's
mirror (`mirror.pl` pairs each pose with the one whose name swaps right and left: 02h, 04h, ...,
0Ah, 0Ch, ..., 49h, 75h, 77h, ..., 84h, 8Ah; `FACE=R`, `L` or both), sets the facing per pose for
facing-dependent rows, and tells the gun-extended poses (0Bh/0Ch, 13h/14h, 67h/68h, by name) apart
as their own pose, with 13h/14h and 67h/68h added as situations. Every pad, both facings:
**0 mismatches** (`FACE=R` and `FACE=L` each print `mismatches 0`). Before the fixes the left
facing gave 1,464 (the moonwalk's both-aims row as `spec.pl` still had it, already right in the
spec, and the take-off below), and the gun 7,438. What the run found:
- The take-off facing left (4Ch, 56h, 58h, 5Ah) has a row hold right → 30h (turn) that its
  right-facing mirror (4Bh, 55h, 57h, 59h) lacks, placed before hold shoot: spec row "facing left:
  back held; beats shoot held". No record reaches it (every recorded take-off ends by its
  animation the next frame). A take-off lasts one frame and its animation's end always decides
  the next pose, so the pad never decides during it in play: the row is unreachable, left out of
  the spec, and the checker reports it as known.
- The straight jump aiming down facing right (17h) has hold right + jump + shoot → 13h after the
  diagonal rows; 18h has a second left + jump + aim-down row in its place. Spec row "aiming `Down`,
  facing right: forward, jump and shoot held"; C-chart-i, C-chart-j.
- The gun: every row naming shoot whose pose is `Forward` running, jumping or falling goes to
  0Bh, 13h or 67h. 0Bh's and 67h's hold-forward rows go to themselves, 09h's and 29h's to
  themselves (spec "gun as it was"); 13h's hold right goes to 51h (stowed). 13h lacks 4Dh's and
  51h's hold jump → 4Dh, so jump alone is the lookup failure; 67h lacks 29h's up + back and
  down + back → 87h, so up or down wins. Landing (`$91:E95D`: E6h/E7h with shoot held, else
  A4h/A5h or the aimed landing poses), walking off a ledge (`$90:E3E0` 29h/2Ah) and the take-off's
  animation end (4Bh → 4Dh) all give stowed poses.
- Records: G1 r08:394 (09h → 0Bh), G2 samus-ceres-push:858 (0Bh kept, pad 8100), G3 r04:262
  (29h → 67h), G4 r04:263 (67h kept, pad 8100), G5 samus-ceres-push:828 (4Dh → 13h), G6
  samus-ceres-push:829 (13h kept, pad 0080, handler `$90:A337`), G7 r08:243 (13h → 51h), G8
  samus-grapple-release:231 (68h → 2Eh, pad 0500, handler `$90:A337`), G9 r04:498 (19h → 13h).
  samus-ceres-push:524-529 hold 0Ch with pad 0 (the pose input isn't run there; not used).

**Forward and jump held in a straight jump (jump-tie, 2026-11-07).** The implementer's tie check
found the spec's "forward held" (striding) and "gun stowed: jump held" (together) rows both fitting
with forward and jump held, and nothing deciding between them. The table decides: in 4Dh/4Eh,
51h/52h, 13h/14h, 15h-18h and 69h/6Ah the hold forward + jump row (right + jump or left + jump →
51h/52h, moving forward) comes before hold jump (→ 4Dh/4Eh) and hold jump + shoot, in both
facings (`$TEMP/pose-chart/tt.pl 4D 4E 51 52 13 14 15 16 17 18 69 6A`). So forward wins: spec row
"forward held; beats shoot held and jump held". `chk.pl` reported 0 mismatches because its
`desc` names 51h and 4Dh alike (`Jumping/F/F`, no legs), so it never saw the legs. Reading every
situation's same-rank row pairs, this is the chart's only undecided one (the rest are exclusive,
ranked apart or have a "beats"). Records: L1 r07:224 (4Eh → 52h, pad 0280, nothing new; r05:176
the same), L2 samus-ceres-push:772 (4Dh → 51h, pad 0180, nothing new). C-chart-j (18h, left +
jump + shoot) takes 18h's left + jump → 52h, the first fit, so its after has the legs striding;
the vector had them together.

**Adversary-16's spec items (adv16, 2026-11-07).**
- The aimed-down straight jump facing left (§5b). 18h's table has no left + jump + shoot row: in
  17h's right + jump + shoot → 13h slot it repeats left + jump + aim-down → 6Ch, so left + jump +
  shoot falls to left + jump → 52h (`tt.pl 17 18`). The spec's general rows give the same (forward
  held beats shoot and jump held), so no facing-left row is missing; the spec now says so beside the
  table. C-chart-i (17h → 13h) and C-chart-j (18h → 52h, striding) agree with the table.
- The strict checker. `$TEMP/adv16/chk.pl` (every pad, both facings) and `$TEMP/adv16/pstep.pl`
  (every `pose_step`) decide by rank and "beats" alone and fail when more than one row survives,
  whatever the rows' results; `desc` now carries the legs (51h/52h "moving forward" → `/S`).
  Every pad: 0 ties; the only mismatches (1,080) are the take-off's facing-left hold right → 30h
  row above, left out as unreachable. Pose steps: 36 decided by the chart, 0 ties, 0 mismatches
  against the vectors or the table; 12 are decided by `AnimationEnds` or `WhatHerMovesMet`.
- A spin reversed early. `$91:F624` skids (base += extra, extra 0, `0B4A` = 1) when the old
  movement type `0A23` is 3 or 14h. `$91:EB88` writes the old pose after every pose it sets
  (`$91:EBF8`), so `0A20` is the spin itself from the spin's first frame: every reversal skids. Only
  the wall-jump check reads the old-old type `0A27` (`$90:9D38`). Record: r05:537 (19h from 26h at
  536; left + jump → 1Ah, `0B4A` 0 → 1, x +1.6000 the old way): SR1. Others: r07:92, r08:231,
  samus-ceres-push:868 (the spin's third or fourth frame).
- Landing while the body changes shape. `$90:E61B` takes the landed result from `$90:E676` by
  movement type: Fh (and Eh, 17h, 18h) give 4, no change, so `$91:E931` sets no pose, and
  `$91:8146` runs no lookup for Fh. The pose runs on to its animation's end. Records:
  samus-mockball:70 (37h lands, kept to 74, 79h at 75): LB1; r04:331 (3Dh lands, kept to 336, 27h
  at 337): LB2; item-morph-ball:1264 and r06:16 land on the animation's last frame (1Dh, 27h).
- Wall jump readied. `$90:9DB2` sets the animation frame to Ah (1Ah screw attack) with timer 1 and
  leaves `0A1C` alone: it is animation progress, not a pose. The next frame the spin counts as
  turned far (`$90:9D55`, frame ≥ Bh). Record: samus-walljump-16:46 (81h readied, `an` 1Bh at 47),
  47 (jump pressed, 83h at 48).
- Landing while turning (adversary-17 fix 1). `$90:E676` gives 4 (no change) for Eh, 17h and 18h
  too, so no landing pose is set. Unlike Fh's handler (`$90:A61C`, which on landed + no change
  zeroes `0B2C`/`0B2E`, `0B36` and `0B20`), the turn handlers `$90:A67C`, `$90:A790` and `$90:A7AD`
  zero only `0DC6`: `0B2E`.`0B2C` keeps growing by the pull while y stays at the floor (FFFF), and
  the pad lookup still runs (only Fh skips it, `$91:8146`). Records (`$TEMP/adv17s/turnfloor.pl`
  over `$TEMP/respec-motion/out`): r02:33-36 (88h), r05:104-105 (87h), r07:258-262 (2Fh),
  area-norfair:64-67 (92h), terrain-quicksand:253-257 (2Fh), samus-ceres-push:322 (30h), the
  ground turn item-morph-ball:685 (25h; r01:615 is on a slope). Ends: r02:37 (29h) lands at 38
  (A4h, `0B2E` 0), r07:263 (52h) at 264 (A5h). Vectors TL1 (r02:33), TL2 (r02:34, lead-in 33),
  TL3 (r07:258), TL4 (item-morph-ball:685); pose steps TP1 (r02:32), TP2 (item-morph-ball:684),
  TP3 (r07:257).
- The near-wall test's gate (adversary-17 fix 2). `$90:9D35` returns before anything unless
  `0A27` is 3 or 14h; both the readying branch (`$90:9D6B`) and the eligible branch (`$90:9DDD`)
  return with neither left nor right held; with a side held both run `$A0:A8F0` and `$94:967F`,
  whose writes happen whether or not jump is newly pressed (`$90:9E05`). RW1
  (samus-walljump-16:47, lead-in 44, 45): at 45→46 (82h → 81h, right held) the test writes `0AF8`
  0 (003C.5FFF → 003C.0000) and the reversal restarts the animation; at 46→47 it readies (`an`
  1Bh) and writes 0 again (003B.2000 → 003B.0000). The wall's face is x 30h (samus-walljump-28's
  flush x 0035.0000 less radius 5); the stand-in is one block at (20h, 440h).
- Pose fit (integrator gap 9). `$91:F404` runs `$91:FDAE` before `$91:F433` (pose init) and
  `$91:FBBB` (jump transitions; the crouch lift `$91:FC8A` is there, only from 27h/28h), so the
  push precedes the speeds and the lift. `$91:FDAE`: nothing for poses 0 and 9Bh or when the new
  y radius (`$91:B62F` + pose × 8) isn't larger; `0B00` = the old radius; g = new − old. Enemy
  looks `$A0:A8F0` up then down by g (`0A3E`, `0A40`), the one-side handlers `$91:FE9E`/`$91:FEDF`
  re-test the far side at the new radius (enemy hit: revert); block looks `$94:96AB` (8 first when
  g ≥ 8, `$94:96E3` with direction Fh, frame-parity order `$94:959E`/`$94:95F5`; `$12` returned as
  |distance|) up then down (`0A38`, `0A3C`); `$91:FF20` (above: y += g − room, after a down
  re-test), `$91:FF49` (below: y −= g − room, after an up re-test), `$91:FF89`/`$91:FF98` (enemy
  only), `$91:FFA7` (both: radius < 8 refuses; else pose 27h/28h by `0A1E`, y raised when the old
  radius is under 10h). A refusal writes `0A20` back to `0A1C` (`$91:FE82`). `0AFA` only, so the
  subpixel is kept. Records (`$TEMP/adv17s/fit2.pl` over `$TEMP/respec-motion/out`, ROM radii):
  every flat landing rises by g after the floor stop (r02:38, 179, 549; r04:10, 63, 500; ...),
  crouch to stand 5 px (r02:392), PF1; open-air growth has none (area-norfair:38, 49, 62). No clean
  record meets a ceiling or a solid enemy in pose fit. Vectors PF2 (r02:179), PF3 (r02:549), U1
  gains the entry; C-fit-ceiling, C-fit-tunnel, C-fit-ball-tunnel constructed. The chart's "5 px
  up" on crouch to stand is this push, not an effect of the row.
- Pose fit, adversary 18. The push writes `0AFA` only, but the looks write `0AFC` first: a look
  stopped by a solid block ends in `$94:8F82` (`0AFC` = FFFFh down, 0 up), by a stepped block in
  `$94:8DBD` (the same writes, and a downward stop sets `0DBA`, the seat), by a touching enemy in
  `$A0:A8F0` (0); a non-square slope (`$94:86FE`) writes nothing. The old engine's
  `a_ceiling_pushes_her_down_flush_with_it` (4000h → 0) agrees, so C-fit-ceiling from FA.8000
  ends at 0100.0000, not 0100.8000. Direction Fh (`$94:96E3`) sends every non-square slope to
  `$94:86FE`'s down branch, which returns clear for a ceiling (BTS bit 7): a look up passes sloped
  ceilings; for a sloped floor in an up look's row it compares her target bottom boundary mod 10h
  (question 18). Fh only silences the PLM setups that test the direction (stations' access, save
  station, chozo hand, crumble; quicksand partly): `$94:93CE` (door) sets game state 9 for a door
  to a room and `$94:905D` (spike block) runs the damage table whatever the direction (question
  17). Mixed sides (`$91:FDBD`): the enemy table `$91:FE8A` runs its one-side re-look (`$91:FE9E`,
  `$91:FEDF`) before the block looks; the block table's one-side handlers `$91:FF20`/`$91:FF49`
  re-look first and refuse on a hit, and only then test the enemy flags for `$91:FFA7`; same side,
  `$91:FF20`/`$91:FF49` shift by the block's room. 8 first: `$94:96AB` returns carry set on the
  8 px look's hit with that look's `$12`. Skipped effects: `$91:F404` returns carry set when the
  pose is not the one asked for, and `$91:EBEE` then skips `$91:EC16`'s command, the landing's
  `$91:EFC4` (`$91:F010`: `0B4A`, `0B46`/`0B48` = 0; `$91:F1EC`: the vertical stop) included.
  C-fit-tunnel can't show it with a speed: the crouched handler halts her in the move
  (`$90:A337`), so C-fit-land-tunnel (a spin landing crouched instead) carries it.
- The shrink (`ShrinkingLowersHer`): `$91:F433`'s init sets command 7 for the transition poses
  (`$91:F758`), and `$91:ECDA` runs `$94:96AB` at the new radius for `$91:ED36`'s distance (35h/36h
  5, 37h/38h 9; F1h-F6h 5) and adds the `$12` it returns to `0AFA` (adversary 19: the room on a
  stop, the distance otherwise); it clears the bounce state
  and Y speed only if bouncing (unreachable from the poses that start a morph). Runs only when
  the pose stands (`$91:EBEE`). Records (`$TEMP/adv18s/shrink.pl` over `$TEMP/respec-motion/out`):
  every start of 35h-38h moves y by the table's distance on top of the frame's move: r02:95, 105,
  115, 388, 405, 425; r07:28, 38, 59; samus-bomb-jump-vertical:843, 849; r04:312 and
  item-morph-ball:1259, samus-mockball:69 (17h → 37h in the air, radius 10 → 7, still 9 px).
  Vectors SH1 (r02:95), SH2 (samus-bomb-jump-vertical:849), SH3 (r04:312).
- Adversary 19. Order (`$91:FDAE`): `$A0:A8F0` up (`$91:FDFD`), down (`$91:FE1F`), the enemy table
  `$91:FE8A` (`$91:FE3C`; carry set goes straight to the revert at `$91:FE82`, so no block look
  runs), then `$94:96AB` up (`$91:FE4C`), down (`$91:FE64`), then the block table `$91:FE92`, whose
  `$91:FF20`/`$91:FF49` call `$94:96AB` again. The 8 px rule is `$94:96AB`'s own test on each
  call's `$12` (`|$12| & FFF8h` non-zero), so it covers the far-side look and `$91:ECDA`'s look;
  the 8 px look's `$12` is `($12 & FFF8h) | 8`, ±8 for every |distance| under 10h. Within pose fit
  the g-keyed reading gives the same position (every look shares her position, and a far-side
  look's 8 px look repeats the first look's on that side), so only the cells tell it apart:
  C-fit-ceiling (a 6 px far-side look, one look) and C-fit-step-ceiling (a 9 px one, after its
  8 px look). Cells: `$94:96E3` runs `$94:959E`/`$94:95F5` with `0B02` |= Fh, the same per-block
  reactions (`$94:952C`) as a move, so every look's blocks are touched in look order; the old
  engine passes `checks.touches` to `scan_y` in `block()` and `start_transition`. The shrink:
  `$91:ED05` adds `$12` as `$94:96AB` returns it, which on a stop is the room (`$94:8F82`:
  max(0, (target bottom & FFF0h) − radius − y)), so she is lowered by the room, not the table's
  distance; the old engine's `start_transition` shifts by the distance (a divergence of the old
  engine, unrecorded: no record meets a floor within reach). C-shrink-floor: SH3's frame with a
  floor at 160, room 5, `0AFC` FFFFh. The ball's x radius is 5 like every pose's (`$90:EC39`
  is the only write of `0AFE`), so the five constructed `[7, 7]` balls became `[5, 7]`.
- RW1 (adversary 18, G): the second lead-in frame's write (003C.5FFF → 003C.0000) is not
  observable at the checked frame (003C.5FFF − 0.E000 = 003B.7FFF → 003B.0000 either way), and a
  vector starting at record 44 has no spin carried on at frame 45, so the claim was dropped rather
  than adding RW0 (a start at record 43 meets a sloped ceiling: `0DBA` 1 at 44).

**The crouch-to-stand 5 px (chart-left).** The bank log calls `$91:8087`'s y − 5 dead, and it is:
it reads the movement type `0A1F` after `$91:81A9`, which only writes the prospective pose, so it
still reads crouching. But the 5 px happens: r02:391 → 392 (27h → 01h, pad 0100, y 00B0 → 00AB),
r06:17 → 18, r07:31 → 32 and r07:41 → 42 (28h → 02h, y 0090 → 008B) all lift her 5 px in the
frame of the change, with the radius 16 → 21: the pose change itself keeps her bottom edge
where it was (`$91:F404` and the collision on a pose change, `$91:FDAE`; not traced to the
instruction). The chart keeps the effect, worded as her feet staying put; U1 is r07:31.

**A finding for `TurningRound`.** The ground turning poses (25h, 26h) have table rows, so with only
run, shoot or jump held none fits and `$91:81DB` runs the lookup failure: command 2 ends the skid.
The air turning poses (2Fh, 30h, 87h, 88h) have empty tables, which return before the failure (R141).
So "she keeps skidding with a button held" holds in the air; on the ground it needs a record.

## Gaps v3 (the first implementation's nine)

| Vectors | Answer | Cause |
| --- | --- | --- |
| R29, R37 | dropped | she rises into a non-square ceiling slope (block type 1, BTS 93h above her at samus-walljump-28:61); `0DBA` stays 0 there, so the selection missed it |
| R30, R38 | restarted after the hit | the same frames, now from a plain falling start |
| R50, R51 | dropped | `0A58` = `$90:E025`/`$90:E032`: bomb jump |
| R67, R68 | dropped | `0A58` = `$90:946E`: released from grapple swing |
| R125 | spec | `DriftInTheAir`: the spin's stop moves by the dash |
| R133 | spec | `TurningRound`: lookup failure (pad 0000), command 2 |

Frame selection now also needs `0A58` = `$90:A337` at record N. New: J1 r03:470, J2 r04:474, J3
r04:475, A1 area-norfair:3 (poses 0Ah to 12h, +1 px), E1 samus-ceres-push:989 (`0A58` normal; the
row's push is elsewhere); constructed E2 (y from samus-ceres-push:990), E3, C-ceiling. The dumper
used is `$TEMP/gaps-v3/dump` (the ground dumper without its under-block probe, which overflows on
rows whose position wraps, plus the blocks above, below and beside her).

## Harness notes kept from the previous spec

The domain harness (`verify::harness::samus_motion`) decodes the previous spec's types; its frame
skips (game state `0998`, lag, the elevator `0E18`, pose fit, handlers `0A44` and `0A58`, the
deferred behaviours) and the bridge's answers are in git history at `1dc3a12`
(`docs/behaviours/samus-motion.decode.md`). They carry over once the implementer's new types exist.

## Gaps v4 (the second implementation's one, and its four readings)

| Question | Answer | Cause |
| --- | --- | --- |
| R141 (r08:116, run held, turning in the air) | spec | `$91:81A9` calls the lookup failure (`$91:82D9`, command 2, which ends the skid) only when `8B` = 0; with any button held and no transition found it returns without a command, so `0B4A` stays 1 (r08:117 shows 1, r08:118 0 once braking passes zero). Movement types Eh and 17h both map to 2 in `$91:8332`: air and ground agree |
| A downward stop on a sloped floor | spec, C-slope-land | `$94:86FE` down branch sets `$14` = 0 and clamps `$12`; `$94:9763` adds it, so `0AFC` is kept (r01:677 `57FF`, samus-walljump-22:93 `FBFF`, samus-input-poses:426 `93FF`); only the block whose column holds `0AF6` counts; no `0DBA`. The square kinds (`$94:8DBD`) write `FFFF` and set `0DBA` |
| The seat after a horizontal move | spec | `$94:87F4`: floor sets `0DBA` only when A < 0 (lifted ≥ 1 px); ceiling when A ≤ 0 (a 0 px push seats). Both skip square kinds (BTS & 1Fh < 5) and the other orientation (BTS bit 7) |
| Slopes and horizontal moves | spec | `$94:84D6` (non-square) always returns carry clear; `$94:8D2B` (square) stops flush at the 8 px quarter with `0AF8` FFFF/0 |
| The wall-jump test on an enemy | spec | `$90:9D80`/`$90:9D92` call `$A0:A8F0`, whose touching branch `$A0:AAC8` clears `0AFC`; the block probe `$94:967F` is skipped on a hit |

## Coverage (vectors for every entry and rule)

| Question | Answer | Source |
| --- | --- | --- |
| Moonfall's vector | M1 r03:13 (records 12 → 13), lead-in records 10 → 11 → 12 (turn pose C0, mt Eh, turn `MoonwalkJump`, into pose 19 with `0B36` = 0); C-moonfall-jump and C-moonfall-nolimit start from r03:10 and match r03's `0AFA`.`0AFC` and `0B2E`.`0B2C` frame by frame to 58 (checked by script); their x is the rule's, r03's own x meets the room at 50-51 | `$91:F36E`, `$90:913A` |
| The elevator | `0E18` ≠ 0 facing forward: move down 1.0 through block collision (`0B02` = 2, so a non-square slope takes the up branch), then return without the pull | `$90:A392`..`$90:A3AB` |
| Solid enemies | target = position ± `$12`.`$14`, then one more pixel unless (right, down) start and move subpixels are both 0, or (left, up) the target subpixel is 0 (the zebetite-skip increment); hit when \|enemy − target\| − enemy radius < Samus radius on both axes (`0F7A`/`0F7E` centre, `0F82`/`0F84` radius); gap from the current position: 0 touching (`0AFC` = 0, distance 0), > 0 distance with `$14` = 0, < 0 skipped; on a hit the callers kill x speed (`$90:E5CE`) or set the vertical result (`$90:E606`, `$90:E61B`) | `$A0:A8F0`..`$A0:AB7A`, `$90:93B1`, `$90:93EC`, `$90:9440` |
| Stepped blocks | the square slope kinds (BTS & 1Fh < 5, table `$94:8E54`, flips from BTS bits 6-7): horizontal picks the column half from the target edge (`$20` & 8) and the row halves from the top and bottom blocks of her span; vertical the reverse; distance `max(0, ($20` & FFF8h) − edge)` with FFFF, or the `+8` form with 0; the vertical down stop sets `0DBA` | `$94:8D2B`, `$94:8DBD` |
| Several blocks | horizontal rows from her top boundary's row down, first carry set wins; vertical columns left to right when `05B6` is even, right to left when odd | `$94:9543`, `$94:959E`, `$94:95F5`, `$94:9763` |
| Sloped edge rows | down: A = top offset − (target bottom % 16) − 1, stop when A ≤ 0 with `$12` = max(0, `$12` + A), `$14` = 0; up: the same on the flipped row, so the first air row stops and `$12` clamps to 0 | `$94:86FE` |
| Aim-change step | enemy test with 1.0 (a hit: ran into wall, no move); else `$94:971E` (block collision, moves by the clamped distance with `0AF8` written), no `$94:87F4` after it and no x speed kill | `$91:EADE`..`$91:EB4C` |
| Seat by the floor check | `0DBA` set only when lifted ≥ 1 px (gaps v4); C-seat-none shows the unseated pull crossing into the next block row | `$94:87F4` |

World keys for the harness: `stepped` is a square slope block (its quarters from `$94:8E54` after
the flips); `enemy` an interactive enemy solid to Samus (`0F86` bit 15, or frozen); `solid_block`
a type 8 block; `slope_ceiling` a non-square slope with BTS bit 7; `frame` the parity of `05B6`;
`elevator_moving` is `0E18` ≠ 0.

## Wiring gaps 2-5 (the integrator's, 2026-11-07)

| Entry | Source | Notes |
| --- | --- | --- |
| SlopesSlowTheWalk | `$94:84D6` and its table `$94:8586` (second word: the factor × 100h; the first word's speed modifier is computed and discarded), `$94:8FBB` (non-square kinds, BTS & 1Fh ≥ 5), `$94:9543` (the column scan, `$20` from the unscaled distance), `$80:82D6` (the 16 × 16 multiply) | only when BTS bit 7 is clear and `0B2C`/`0B2E` = 0; `$12`.`$14` is cut to `$12` low byte : `$15`, negated for leftward, multiplied, negated back over 32 bits. Measured by the dumper's `R`/`L` field (`0DAA`..`0DB0`): r01:625-630 BTS 55h/12h at C0, r06:123-126 BTS 56h/57h at D8, samus-input-poses:418 1.7000 → 1.1400, aim steps 1.0 → 0.C000 (r01) and 0.D800 (r06) |
| TouchedCells | `$94:9543`, `$94:959E`, `$94:95F5` (the scans, stop at the first carry set), `$94:9515`/`$94:952C` and the reaction tables `$94:94D5`/`$94:94F5`; `$94:9B60` (inside points bottom, centre, top at `0AF6`; the chained XOR at `$94:9BEB`), called from `$90:E6BC` in the current state handler, which game state 8 runs (`$82:8B58`) before the new state handler's movement (`$82:8B69`) | the collision direction `0B02` is what the reactions read |
| OtherBlockKinds | `$94:938B`/`$94:93CE` (door: destination ≥ 8000h starts game state 9 and returns carry clear, else solid), `$94:904B`/`$94:905D` (spike: its BTS table, then solid), `$94:9018`/`$94:901A` (spike air), `$94:906F`/`$94:909D` (special air: PLM, carry clear), `$94:90CB`/`$94:9102` (special block: the PLM's carry decides), `$94:92F9`/`$94:9313` (bombable air), `$94:932D`/`$94:934C` (bomb block: the PLM's carry, area-dependent BTS solid), `$94:9411`/`$94:9447` (extensions: block index ± BTS, re-dispatch; BTS 0 air) | the item's block is BTS 45h (`$84:EE64`, `$84:EE8E`, setup `$84:EEAB` returns carry clear); scroll cells BTS 46h (`$84:B393`) |
| PoseFitAfterMoves | `$90:E725` (movement `$90:E94B`, then `$91:E8B6`, then `$91:EB88`), `$91:F404` (`$91:FDAE` pose fit, then `$91:F433` initialise, then `$91:FBBB` the jump transition with `$91:FC8A` the 10 px lift); the push itself `$91:FDAE`..`$91:FFED` with `$94:96AB` and `$A0:A8F0` | `$91:EADE` (the aim step) runs before `$91:F404` |
| ShrinkingLowersHer | `$91:F758` (transition poses set command 7), `$91:ECDA` with its table `$91:ED36` and `$94:96AB` (the look, result ignored) | runs after `$91:F404` only when the pose stands (`$91:EBEE`) |

Recorded vectors: S1 is samus-input-poses record 511 → 512 (block at column 6, row 12, BTS 54h:
14h x-flipped); S2 is r01 record 716 → 717 (BTS 12h, first frame of a fall, `0B2E`.`0B2C` = 0).
The surfaces are `$94:8B2B` rows 14h (flipped) and 12h; 10h in that table is a column with no solid
pixel, written 16 in the vectors. 14h in that table is also a column with no solid pixel, four
rows past the edge, and it never meets her (adversary-14 F1): down (`$94:8780`) A = 14h -
(bottom % 16) - 1 >= 4, so no stop; up (`$94:8707`) A = (top % 16) + 4 > 0, so no stop; after a
horizontal move (`$94:87F4`) no lift, no push, not seated. Shape 1Bh (the upper half-width
triangle, used) has it in columns 0-7; 11h, 1Dh and 1Eh have it too but are unused. The vectors
write it as 17 (any value past 16 acts alike in both directions): the spec's "column far past",
C-slope-floor-far and C-slope-ceiling-far.

**Turning with only run, shoot or jump held (root's question, 2026-11-07).** `$91:81A9` runs the
lookup failure `$91:82D9` (command 2, which ends the skid) when `8B` = 0, or when the pose's
transition table is non-empty and nothing in it matches. The ground turns' tables are non-empty
(25h at `$91:ACF4`, C0h at `$91:AFAC`); the air turns' (2Fh, 30h, 87h, 88h, 8Fh, 93h, ...) are
empty, so there a held button returns without a command. Gaps v4's reading of R141 holds for the
air only. On record: samus-input-poses 119 → 120 (pose C0h, `8B` = 8000, `0B4A` 1 → 0 at base
speed 0), then 120 → 121 moves 0.C000 (R11, R12).

## The vector-format round (2026-11-07)

**The turn a `Turning` pose is in** (spec §2 `turn`), from the pose's name: 25h/26h, 43h/44h,
8Bh-8Eh, 97h-9Ah, 9Ch/9Dh, A2h/A3h are `Ground` (standing or crouching in the name); 2Fh/30h,
8Fh-92h, 9Eh/9Fh are `Jump` ("jumping" or "in air"); 87h/88h, 93h-96h, A0h/A1h are `Fall`; BFh-C4h are
`MoonwalkJump` (movement type Eh, as 25h/26h; C0h's table at `$91:AFAC` has 26h's rows, so the pad chart
reads them as ground turns). The turn's end hands over by the transition `$91:F36E` picks per pose: C0h
→ 19h with no rise (`Moonfall`). The take-off's and the straight jumps' back rows go to 2Fh/30h;
the falls' to 87h/88h; 4Ah's jump press to BFh.

**Start poses checked against the records** (`$TEMP/vec-fix/chk.pl`: each recorded vector's start,
lead-in, next and after pose against record N's `0A1C`, with aim, gun and turn from the pose's
name). Fixed: R14 (11h) and R135 (12h) are aimed `DownForward`, so aim down held changes nothing
and no aim step runs; R78 (70h), R126 (6Ch), R131 (08h), R146 (E4h) `DownForward`; R116 (6Eh) and
R140 (69h) `UpForward`; R46 (0Bh) has the gun `Drawn`; every recorded `Turning` pose has its turn
(M1, R1, R11, R12, R16, R17, R42, R64, R71, R72, R80-R83, R85, R87, R93, R94, R113, R114,
R132-R134, R141, R142, P11, P14, P18, P19, S1). M1's source was the start record (r03:12); the
header's is the after record, now r03:13. After the fixes the script reports no difference.

**Crouched, down and jump pressed together.** 27h's table (`$91:9EE2` + 2 × 27h) lists press down
(→ 37h) before press jump (→ 4Bh), and the lookup takes the first match. No record presses both.

**PF1** is item-morph-ball record 1310 → 1311: 27h (y radius 16) with run and jump pressed → 4Bh;
y 02C0.FFFF → 02B3.FFFF, 13 px: pose fit `$91:FDAE` keeps her feet for the new radius 19 (3 px),
then `$91:FC8A` lifts 10 px. The dumped radius `0B00` still reads 16 at 1311 and 19 at 1312: the
word lags the pose by a frame in the dump; y shows the push and the lift in the same frame. x is unchanged
(crouched: dash 0, so no move) and the base speed 3.4000 goes to 0.

**TouchedCells and OtherBlockKinds vectors** are constructed from the spec's rules: no dump
records the cells touched. World key `cells`: `DoorToRoom` / `DoorToNowhere` a door block whose
destination does / doesn't start a room change; `Spikes` / `AirWithSpikes` the spike block and spike
air; `BreaksToShots`, `HoldsGrapple`, `BreaksToBombs` the shot, grapple and bomb blocks;
`AirShotsActOn`, `AirBombsActOn` the shot and bomb air;
`ReactingBlock` / `ReactingAir` the special block and special air; `StandIn` the horizontal or
vertical extension block, `names` its target's corner. The unused air type behaves as plain air
and has no key (spec `OtherBlockKinds`).

## The pose map (gaps 6 and 7, 2026-11-07)

**What changed in the spec.** §2 gains the action `Landing` with the part `landed from` (`Jump`,
`Spin`), the part `legs` (`Together`, `Striding`), the gun `Drawn` on the landing that fires, the
turn's aim, the body during a change of shape, and the equipment seen only in the sprite (screw
attack, spring ball, the suit). With these, every sprite id the domain reaches is one §2 Pose,
except the spin and facing the screen, which the equipment decides.

**Sources.**

| Fact | Source |
| --- | --- |
| facing, movement type, the pose with no buttons | pose definitions `$91:B629` (bytes 0, 1, 2); a turn's byte 0 is the new facing, a moonwalk's the way she walks |
| landing poses: spin or wall jump (old movement type 3, 14h) → A6h/A7h; else by the shot direction (`$91:B62C`): up/up-forward/down-forward → E0h-E5h, forward with shoot held → E6h/E7h, forward or down → A4h/A5h | `$91:E95D`, table `$91:E9F3` |
| a landing ends into the standing pose of its aim (A4h, A6h, E6h → 01h; E0h → 03h; E2h → 05h; E4h → 07h) | `$91:B22D`..`$91:B253` (instruction F8) |
| a landing pose has no pose with no buttons (FFh), so the let-go row keeps it, aim included; a standing pose's is 01h/02h | `$91:82D9` (FFh: retain), `$91:B629` byte 2 |
| the spin: in a liquid without the gravity suit, 19h/1Ah; else screw attack → 81h/82h, space jump → 1Bh/1Ch, else 19h/1Ah; run on every change into movement type 3 (a jump, a spin reversed, a wall jump's end, the moonwalk turn's end) | `$91:F624`, `$91:F67F`..`$91:F74A` |
| the wall jump's spin is drawn by equipment too (one pose, three sequences) | `$91:B491` (instruction FB) |
| facing the screen: 9Bh with varia or gravity, else 00h; re-picked when equipment changes | `$91:E3F6`, `$91:E733` |
| a turn takes its pose from the shot direction of the pose before: standing `$91:F9C2`, crouching `$91:F9CC`, jumping `$91:F9D6`, falling `$91:F9E0`, the moonwalk with jump held `$91:F9EA`; the chart's rows only ever name the unaimed turns (25h, 2Fh, 87h, 43h, BFh) | `$91:F8D3`, `$91:F952`, `$91:F98A` |
| the air turns aimed down-forward and down are one sprite (91h, 95h) and end aimed down (18h, 2Eh); the crouched one (99h) ends down-forward (74h) | `$91:B401`, `$91:B415`, `$91:B429` |
| a jump turn's end: 2Fh → 52h (moving forward), 8Fh → 16h, 9Eh → 6Ah; a take-off's end: 4Bh → 4Dh, 55h → 15h, 57h → 69h, 59h → 6Bh | `$91:B3C5`, `$91:B3F7`, `$91:B43D`, `$91:B308`..`$91:B322` |
| legs: hold forward → 51h (moving forward), hold jump alone → 4Dh, hold shoot → 13h, in 4Bh, 4Dh, 51h, 13h, 15h, 69h's rows | transition table `$91:9EE2` |
| the morph's end: spring ball → 79h (ground) / 7Dh (air), else 1Dh / 31h | `$91:B4C8`, `$91:B4D1` (instruction F9, item 0002) |
| the body while changing shape: y radius 10h for 35h, 3Dh, F1h-F6h; 7 for 37h; 15h for 3Bh, F7h-FCh | `$91:B629` byte 6 |
| the crouched aimed turns (97h-9Ah, A2h, A3h) have the jump turn's movement type 17h; their tables are empty, so the chart's "crouched turning" row covers them | `$91:B629` byte 1 |

Records: 51h → E6h with shoot held (pad 8140, 8040, 0140) and 51h → A4h without, 13h → A4h with
the pad let go, 81h → A6h, 2Fh → 52h, 92h → 17h, 9Ah → 73h, 9Eh → 6Ah, 8Dh → 1Ah
(`$TEMP/pose-chart/all.txt`). No record has a turn in the air meeting the floor (spec §11 item 12):
`$91:E95D` would index `$91:E9F3` by the turn's shot direction (FAh-FCh), past the table.

**Vectors changed.** Start pose `Landing` from `Jump`: R54, R56, R69 (A4h/A5h), R118, R143 (A5h),
R146 (E4h). After pose `Landing`: P4 (A4h, from `Jump`), P5 (A6h, from `Spin`). Legs `Striding`:
R8, R40, R41, R66, R110, R115, P9, P18 (start, 51h/52h), P19, G7 (after). Body `Crouched` while
`Crouching`: R48, R89, R128, R129 (35h/36h, radius 16). `$TEMP/pose-map/edit.pl` made the edits;
the ids come from `$TEMP/vec-fix/chk.pl`'s records.

**From a §2 Pose to the id the game shows.** The Pose decides the id alone, except:
- `Spinning`: 81h/82h with screw attack, else 1Bh/1Ch with space jump, else 19h/1Ah (and 19h/1Ah in
  a liquid without the gravity suit);
- upright `Still` facing `Screen`: 9Bh with varia or gravity, else 00h.

A turn's facing is its new facing; a moonwalk's is the way she walks. A Pose the spec never
produces maps to its nearest sprite: an air turn aimed `DownForward` as `Down` (91h/92h, 95h/96h),
a crouched turn aimed `Down` as `DownForward` (99h/9Ah).

| Body | Action | Aim | Other parts | Right | Left | Screen |
| --- | --- | --- | --- | --- | --- | --- |
| Ball | Falling | Forward |  | 31h | 32h | - |
| Ball | Morphing | Forward |  | 37h | 38h | - |
| Ball | Rolling | Forward |  | 1Eh | 1Fh | - |
| Ball | Still | Forward |  | 1Dh | 41h | - |
| Crouched | Crouching | DownForward |  | F5h | F6h | - |
| Crouched | Crouching | Forward |  | 35h | 36h | - |
| Crouched | Crouching | Up |  | F1h | F2h | - |
| Crouched | Crouching | UpForward |  | F3h | F4h | - |
| Crouched | Still | DownForward |  | 73h | 74h | - |
| Crouched | Still | Forward |  | 27h | 28h | - |
| Crouched | Still | Up |  | 85h | 86h | - |
| Crouched | Still | UpForward |  | 71h | 72h | - |
| Crouched | Turning | DownForward | turn Ground | 9Ah | 99h | - |
| Crouched | Turning | Forward | turn Ground | 44h | 43h | - |
| Crouched | Turning | Up | turn Ground | 98h | 97h | - |
| Crouched | Turning | UpForward | turn Ground | A3h | A2h | - |
| Crouched | Unmorphing | Forward |  | 3Dh | 3Eh | - |
| SpringBall | Falling | Forward |  | 7Dh | 7Eh | - |
| SpringBall | Jumping | Forward |  | 7Fh | 80h | - |
| SpringBall | Rolling | Forward |  | 7Bh | 7Ch | - |
| SpringBall | Still | Forward |  | 79h | 7Ah | - |
| Upright | AgainstWall | DownForward |  | D1h | D2h | - |
| Upright | AgainstWall | Forward |  | 89h | 8Ah | - |
| Upright | AgainstWall | UpForward |  | CFh | D0h | - |
| Upright | Falling | Down |  | 2Dh | 2Eh | - |
| Upright | Falling | DownForward |  | 6Fh | 70h | - |
| Upright | Falling | Forward |  | 29h | 2Ah | - |
| Upright | Falling | Forward | gun Drawn | 67h | 68h | - |
| Upright | Falling | Up |  | 2Bh | 2Ch | - |
| Upright | Falling | UpForward |  | 6Dh | 6Eh | - |
| Upright | Jumping | Down |  | 17h | 18h | - |
| Upright | Jumping | DownForward |  | 6Bh | 6Ch | - |
| Upright | Jumping | Forward |  | 4Dh | 4Eh | - |
| Upright | Jumping | Forward | gun Drawn | 13h | 14h | - |
| Upright | Jumping | Forward | legs Striding | 51h | 52h | - |
| Upright | Jumping | Up |  | 15h | 16h | - |
| Upright | Jumping | UpForward |  | 69h | 6Ah | - |
| Upright | Landing | DownForward | landed from Jump | E4h | E5h | - |
| Upright | Landing | Forward | landed from Jump | A4h | A5h | - |
| Upright | Landing | Forward | landed from Jump, gun Drawn | E6h | E7h | - |
| Upright | Landing | Forward | landed from Spin | A6h | A7h | - |
| Upright | Landing | Up | landed from Jump | E0h | E1h | - |
| Upright | Landing | UpForward | landed from Jump | E2h | E3h | - |
| Upright | Moonwalking | DownForward |  | 77h | 78h | - |
| Upright | Moonwalking | Forward |  | 49h | 4Ah | - |
| Upright | Moonwalking | UpForward |  | 75h | 76h | - |
| Upright | Running | DownForward |  | 11h | 12h | - |
| Upright | Running | Forward |  | 09h | 0Ah | - |
| Upright | Running | Forward | gun Drawn | 0Bh | 0Ch | - |
| Upright | Running | UpForward |  | 0Fh | 10h | - |
| Upright | Spinning | Forward |  | 19h (neither (or in a liquid without the gravity suit)); 1Bh (space jump, no screw attack); 81h (screw attack) | 1Ah (neither (or in a liquid without the gravity suit)); 1Ch (space jump, no screw attack); 82h (screw attack) | - |
| Upright | StandingUp | DownForward |  | FBh | FCh | - |
| Upright | StandingUp | Forward |  | 3Bh | 3Ch | - |
| Upright | StandingUp | Up |  | F7h | F8h | - |
| Upright | StandingUp | UpForward |  | F9h | FAh | - |
| Upright | Still | DownForward |  | 07h | 08h | - |
| Upright | Still | Forward |  | 01h | 02h | 00h (first suit); 9Bh (varia or gravity) |
| Upright | Still | Up |  | 03h | 04h | - |
| Upright | Still | UpForward |  | 05h | 06h | - |
| Upright | TakingOff | DownForward |  | 59h | 5Ah | - |
| Upright | TakingOff | Forward |  | 4Bh | 4Ch | - |
| Upright | TakingOff | Up |  | 55h | 56h | - |
| Upright | TakingOff | UpForward |  | 57h | 58h | - |
| Upright | Turning | Down | turn Fall | 96h | 95h | - |
| Upright | Turning | Down | turn Jump | 92h | 91h | - |
| Upright | Turning | DownForward | turn Ground | 8Eh | 8Dh | - |
| Upright | Turning | DownForward | turn MoonwalkJump | C4h | C3h | - |
| Upright | Turning | Forward | turn Fall | 88h | 87h | - |
| Upright | Turning | Forward | turn Ground | 26h | 25h | - |
| Upright | Turning | Forward | turn Jump | 30h | 2Fh | - |
| Upright | Turning | Forward | turn MoonwalkJump | C0h | BFh | - |
| Upright | Turning | Up | turn Fall | 94h | 93h | - |
| Upright | Turning | Up | turn Ground | 8Ch | 8Bh | - |
| Upright | Turning | Up | turn Jump | 90h | 8Fh | - |
| Upright | Turning | UpForward | turn Fall | A1h | A0h | - |
| Upright | Turning | UpForward | turn Ground | 9Dh | 9Ch | - |
| Upright | Turning | UpForward | turn Jump | 9Fh | 9Eh | - |
| Upright | Turning | UpForward | turn MoonwalkJump | C2h | C1h | - |
| Upright | WallJumping | Forward |  | 83h | 84h | - |

**From an id to its §2 Pose.** Every one of the 253 ids: 164 map to a Pose, 89 are unreachable in
this domain (unused, or another domain's: grapple, knockback, damage boost, shinespark, crystal
flash, x-ray, drained, Draygon). Aim is `Forward` and the gun `Stowed` and the legs `Together`
unless the row says otherwise. Generated by `$TEMP/pose-map/map.pl` from `$TEMP/pose-chart/names.tsv`
and the pose definitions.

| Id | Original name | Body | Action | Facing | Aim | Other parts | Shown when |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 00h | Facing forward - power suit | Upright | Still | Screen | Forward |  | first suit |
| 01h | Facing right - normal | Upright | Still | Right | Forward |  |  |
| 02h | Facing left - normal | Upright | Still | Left | Forward |  |  |
| 03h | Facing right - aiming up | Upright | Still | Right | Up |  |  |
| 04h | Facing left - aiming up | Upright | Still | Left | Up |  |  |
| 05h | Facing right - aiming up-right | Upright | Still | Right | UpForward |  |  |
| 06h | Facing left - aiming up-left | Upright | Still | Left | UpForward |  |  |
| 07h | Facing right - aiming down-right | Upright | Still | Right | DownForward |  |  |
| 08h | Facing left - aiming down-left | Upright | Still | Left | DownForward |  |  |
| 09h | Moving right - not aiming | Upright | Running | Right | Forward |  |  |
| 0Ah | Moving left - not aiming | Upright | Running | Left | Forward |  |  |
| 0Bh | Moving right - gun extended | Upright | Running | Right | Forward | gun Drawn |  |
| 0Ch | Moving left - gun extended | Upright | Running | Left | Forward | gun Drawn |  |
| 0Dh | Unused. Moving right - aiming up | (unreachable) | | | | | unused: the running sprite aimed up; the chart sends running with up held to standing aimed `Up` |
| 0Eh | Unused. Moving left - aiming up | (unreachable) | | | | | unused: the running sprite aimed up; the chart sends running with up held to standing aimed `Up` |
| 0Fh | Moving right - aiming up-right | Upright | Running | Right | UpForward |  |  |
| 10h | Moving left - aiming up-left | Upright | Running | Left | UpForward |  |  |
| 11h | Moving right - aiming down-right | Upright | Running | Right | DownForward |  |  |
| 12h | Moving left - aiming down-left | Upright | Running | Left | DownForward |  |  |
| 13h | Facing right - normal jump - not aiming - not moving - gun extended | Upright | Jumping | Right | Forward | gun Drawn |  |
| 14h | Facing left - normal jump - not aiming - not moving - gun extended | Upright | Jumping | Left | Forward | gun Drawn |  |
| 15h | Facing right - normal jump - aiming up | Upright | Jumping | Right | Up |  |  |
| 16h | Facing left - normal jump - aiming up | Upright | Jumping | Left | Up |  |  |
| 17h | Facing right - normal jump - aiming down | Upright | Jumping | Right | Down |  |  |
| 18h | Facing left - normal jump - aiming down | Upright | Jumping | Left | Down |  |  |
| 19h | Facing right - spin jump | Upright | Spinning | Right | Forward |  | neither (or in a liquid without the gravity suit) |
| 1Ah | Facing left - spin jump | Upright | Spinning | Left | Forward |  | neither (or in a liquid without the gravity suit) |
| 1Bh | Facing right - space jump | Upright | Spinning | Right | Forward |  | space jump, no screw attack |
| 1Ch | Facing left - space jump | Upright | Spinning | Left | Forward |  | space jump, no screw attack |
| 1Dh | Facing right - morph ball - no springball - on ground | Ball | Still | Right | Forward |  |  |
| 1Eh | Moving right - morph ball - no springball - on ground | Ball | Rolling | Right | Forward |  |  |
| 1Fh | Moving left - morph ball - no springball - on ground | Ball | Rolling | Left | Forward |  |  |
| 20h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 21h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 22h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 23h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 24h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 25h | Facing right - turning - standing | Upright | Turning | Left | Forward | turn Ground |  |
| 26h | Facing left - turning - standing | Upright | Turning | Right | Forward | turn Ground |  |
| 27h | Facing right - crouching | Crouched | Still | Right | Forward |  |  |
| 28h | Facing left - crouching | Crouched | Still | Left | Forward |  |  |
| 29h | Facing right - falling | Upright | Falling | Right | Forward |  |  |
| 2Ah | Facing left - falling | Upright | Falling | Left | Forward |  |  |
| 2Bh | Facing right - falling - aiming up | Upright | Falling | Right | Up |  |  |
| 2Ch | Facing left - falling - aiming up | Upright | Falling | Left | Up |  |  |
| 2Dh | Facing right - falling - aiming down | Upright | Falling | Right | Down |  |  |
| 2Eh | Facing left - falling - aiming down | Upright | Falling | Left | Down |  |  |
| 2Fh | Facing right - turning - jumping | Upright | Turning | Left | Forward | turn Jump |  |
| 30h | Facing left - turning - jumping | Upright | Turning | Right | Forward | turn Jump |  |
| 31h | Facing right - morph ball - no springball - in air | Ball | Falling | Right | Forward |  |  |
| 32h | Facing left - morph ball - no springball - in air | Ball | Falling | Left | Forward |  |  |
| 33h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 34h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 35h | Facing right - crouching transition | Crouched | Crouching | Right | Forward |  |  |
| 36h | Facing left - crouching transition | Crouched | Crouching | Left | Forward |  |  |
| 37h | Facing right - morphing transition | Ball | Morphing | Right | Forward |  |  |
| 38h | Facing left - morphing transition | Ball | Morphing | Left | Forward |  |  |
| 39h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 3Ah | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 3Bh | Facing right - standing transition | Upright | StandingUp | Right | Forward |  |  |
| 3Ch | Facing left - standing transition | Upright | StandingUp | Left | Forward |  |  |
| 3Dh | Facing right - unmorphing transition | Crouched | Unmorphing | Right | Forward |  |  |
| 3Eh | Facing left - unmorphing transition | Crouched | Unmorphing | Left | Forward |  |  |
| 3Fh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 40h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 41h | Facing left - morph ball - no springball - on ground | Ball | Still | Left | Forward |  |  |
| 42h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 43h | Facing right - turning - crouching | Crouched | Turning | Left | Forward | turn Ground |  |
| 44h | Facing left - turning - crouching | Crouched | Turning | Right | Forward | turn Ground |  |
| 45h | Unused | (unreachable) | | | | | unused: a running pose the game turns straight into the ground turn when it is set |
| 46h | Unused | (unreachable) | | | | | unused: a running pose the game turns straight into the ground turn when it is set |
| 47h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 48h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 49h | Facing left - moonwalk | Upright | Moonwalking | Right | Forward |  |  |
| 4Ah | Facing right - moonwalk | Upright | Moonwalking | Left | Forward |  |  |
| 4Bh | Facing right - normal jump transition | Upright | TakingOff | Right | Forward |  |  |
| 4Ch | Facing left - normal jump transition | Upright | TakingOff | Left | Forward |  |  |
| 4Dh | Facing right - normal jump - not aiming - not moving - gun not extended | Upright | Jumping | Right | Forward |  |  |
| 4Eh | Facing left - normal jump - not aiming - not moving - gun not extended | Upright | Jumping | Left | Forward |  |  |
| 4Fh | Facing left - damage boost | (unreachable) | | | | | out of scope: damage boost |
| 50h | Facing right - damage boost | (unreachable) | | | | | out of scope: damage boost |
| 51h | Facing right - normal jump - not aiming - moving forward | Upright | Jumping | Right | Forward | legs Striding |  |
| 52h | Facing left - normal jump - not aiming - moving forward | Upright | Jumping | Left | Forward | legs Striding |  |
| 53h | Facing right - knockback | (unreachable) | | | | | out of scope: knockback |
| 54h | Facing left - knockback | (unreachable) | | | | | out of scope: knockback |
| 55h | Facing right - normal jump transition - aiming up | Upright | TakingOff | Right | Up |  |  |
| 56h | Facing left - normal jump transition - aiming up | Upright | TakingOff | Left | Up |  |  |
| 57h | Facing right - normal jump transition - aiming up-right | Upright | TakingOff | Right | UpForward |  |  |
| 58h | Facing left - normal jump transition - aiming up-left | Upright | TakingOff | Left | UpForward |  |  |
| 59h | Facing right - normal jump transition - aiming down-right | Upright | TakingOff | Right | DownForward |  |  |
| 5Ah | Facing left - normal jump transition - aiming down-left | Upright | TakingOff | Left | DownForward |  |  |
| 5Bh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 5Ch | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 5Dh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 5Eh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 5Fh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 60h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 61h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 62h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 63h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 64h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 65h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 66h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| 67h | Facing right - falling - gun extended | Upright | Falling | Right | Forward | gun Drawn |  |
| 68h | Facing left - falling - gun extended | Upright | Falling | Left | Forward | gun Drawn |  |
| 69h | Facing right - normal jump - aiming up-right | Upright | Jumping | Right | UpForward |  |  |
| 6Ah | Facing left - normal jump - aiming up-left | Upright | Jumping | Left | UpForward |  |  |
| 6Bh | Facing right - normal jump - aiming down-right | Upright | Jumping | Right | DownForward |  |  |
| 6Ch | Facing left - normal jump - aiming down-left | Upright | Jumping | Left | DownForward |  |  |
| 6Dh | Facing right - falling - aiming up-right | Upright | Falling | Right | UpForward |  |  |
| 6Eh | Facing left - falling - aiming up-left | Upright | Falling | Left | UpForward |  |  |
| 6Fh | Facing right - falling - aiming down-right | Upright | Falling | Right | DownForward |  |  |
| 70h | Facing left - falling - aiming down-left | Upright | Falling | Left | DownForward |  |  |
| 71h | Facing right - crouching - aiming up-right | Crouched | Still | Right | UpForward |  |  |
| 72h | Facing left - crouching - aiming up-left | Crouched | Still | Left | UpForward |  |  |
| 73h | Facing right - crouching - aiming down-right | Crouched | Still | Right | DownForward |  |  |
| 74h | Facing left - crouching - aiming down-left | Crouched | Still | Left | DownForward |  |  |
| 75h | Facing left - moonwalk - aiming up-left | Upright | Moonwalking | Right | UpForward |  |  |
| 76h | Facing right - moonwalk - aiming up-right | Upright | Moonwalking | Left | UpForward |  |  |
| 77h | Facing left - moonwalk - aiming down-left | Upright | Moonwalking | Right | DownForward |  |  |
| 78h | Facing right - moonwalk - aiming down-right | Upright | Moonwalking | Left | DownForward |  |  |
| 79h | Facing right - morph ball - spring ball - on ground | SpringBall | Still | Right | Forward |  |  |
| 7Ah | Facing left - morph ball - spring ball - on ground | SpringBall | Still | Left | Forward |  |  |
| 7Bh | Moving right - morph ball - spring ball - on ground | SpringBall | Rolling | Right | Forward |  |  |
| 7Ch | Moving left - morph ball - spring ball - on ground | SpringBall | Rolling | Left | Forward |  |  |
| 7Dh | Facing right - morph ball - spring ball - falling | SpringBall | Falling | Right | Forward |  |  |
| 7Eh | Facing left - morph ball - spring ball - falling | SpringBall | Falling | Left | Forward |  |  |
| 7Fh | Facing right - morph ball - spring ball - in air | SpringBall | Jumping | Right | Forward |  |  |
| 80h | Facing left - morph ball - spring ball - in air | SpringBall | Jumping | Left | Forward |  |  |
| 81h | Facing right - screw attack | Upright | Spinning | Right | Forward |  | screw attack |
| 82h | Facing left - screw attack | Upright | Spinning | Left | Forward |  | screw attack |
| 83h | Facing right - wall jump | Upright | WallJumping | Right | Forward |  |  |
| 84h | Facing left - wall jump | Upright | WallJumping | Left | Forward |  |  |
| 85h | Facing right - crouching - aiming up | Crouched | Still | Right | Up |  |  |
| 86h | Facing left - crouching - aiming up | Crouched | Still | Left | Up |  |  |
| 87h | Facing right - turning - falling | Upright | Turning | Left | Forward | turn Fall |  |
| 88h | Facing left - turning - falling | Upright | Turning | Right | Forward | turn Fall |  |
| 89h | Facing right - ran into a wall | Upright | AgainstWall | Right | Forward |  |  |
| 8Ah | Facing left - ran into a wall | Upright | AgainstWall | Left | Forward |  |  |
| 8Bh | Facing right - turning - standing - aiming up | Upright | Turning | Left | Up | turn Ground |  |
| 8Ch | Facing left - turning - standing - aiming up | Upright | Turning | Right | Up | turn Ground |  |
| 8Dh | Facing right - turning - standing - aiming down-right | Upright | Turning | Left | DownForward | turn Ground |  |
| 8Eh | Facing left - turning - standing - aiming down-left | Upright | Turning | Right | DownForward | turn Ground |  |
| 8Fh | Facing right - turning - in air - aiming up | Upright | Turning | Left | Up | turn Jump |  |
| 90h | Facing left - turning - in air - aiming up | Upright | Turning | Right | Up | turn Jump |  |
| 91h | Facing right - turning - in air - aiming down/down-right | Upright | Turning | Left | Down | turn Jump |  |
| 92h | Facing left - turning - in air - aiming down/down-left | Upright | Turning | Right | Down | turn Jump |  |
| 93h | Facing right - turning - falling - aiming up | Upright | Turning | Left | Up | turn Fall |  |
| 94h | Facing left - turning - falling - aiming up | Upright | Turning | Right | Up | turn Fall |  |
| 95h | Facing right - turning - falling - aiming down/down-right | Upright | Turning | Left | Down | turn Fall |  |
| 96h | Facing left - turning - falling - aiming down/down-left | Upright | Turning | Right | Down | turn Fall |  |
| 97h | Facing right - turning - crouching - aiming up | Crouched | Turning | Left | Up | turn Ground |  |
| 98h | Facing left - turning - crouching - aiming up | Crouched | Turning | Right | Up | turn Ground |  |
| 99h | Facing right - turning - crouching - aiming down/down-right | Crouched | Turning | Left | DownForward | turn Ground |  |
| 9Ah | Facing left - turning - crouching - aiming down/down-left | Crouched | Turning | Right | DownForward | turn Ground |  |
| 9Bh | Facing forward - varia/gravity suit | Upright | Still | Screen | Forward |  | varia or gravity |
| 9Ch | Facing right - turning - standing - aiming up-right | Upright | Turning | Left | UpForward | turn Ground |  |
| 9Dh | Facing left - turning - standing - aiming up-left | Upright | Turning | Right | UpForward | turn Ground |  |
| 9Eh | Facing right - turning - in air - aiming up-right | Upright | Turning | Left | UpForward | turn Jump |  |
| 9Fh | Facing left - turning - in air - aiming up-left | Upright | Turning | Right | UpForward | turn Jump |  |
| A0h | Facing right - turning - falling - aiming up-right | Upright | Turning | Left | UpForward | turn Fall |  |
| A1h | Facing left - turning - falling - aiming up-left | Upright | Turning | Right | UpForward | turn Fall |  |
| A2h | Facing right - turning - crouching - aiming up-right | Crouched | Turning | Left | UpForward | turn Ground |  |
| A3h | Facing left - turning - crouching - aiming up-left | Crouched | Turning | Right | UpForward | turn Ground |  |
| A4h | Facing right - landing from normal jump | Upright | Landing | Right | Forward | landed from Jump |  |
| A5h | Facing left - landing from normal jump | Upright | Landing | Left | Forward | landed from Jump |  |
| A6h | Facing right - landing from spin jump | Upright | Landing | Right | Forward | landed from Spin |  |
| A7h | Facing left - landing from spin jump | Upright | Landing | Left | Forward | landed from Spin |  |
| A8h | Facing right - grappling | (unreachable) | | | | | out of scope: grapple |
| A9h | Facing left - grappling | (unreachable) | | | | | out of scope: grapple |
| AAh | Facing right - grappling - aiming down-right | (unreachable) | | | | | out of scope: grapple |
| ABh | Facing left - grappling - aiming down-left | (unreachable) | | | | | out of scope: grapple |
| ACh | Unused. Facing right - grappling - in air | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| ADh | Unused. Facing left - grappling - in air | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| AEh | Unused. Facing right - grappling - in air - aiming down | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| AFh | Unused. Facing left - grappling - in air - aiming down | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| B0h | Unused. Facing right - grappling - in air - aiming down-right | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| B1h | Unused. Facing left - grappling - in air - aiming down-left | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| B2h | Facing anticlockwise - grapple swinging | (unreachable) | | | | | out of scope: grapple |
| B3h | Facing clockwise - grapple swinging | (unreachable) | | | | | out of scope: grapple |
| B4h | Facing right - grappling - crouching | (unreachable) | | | | | out of scope: grapple |
| B5h | Facing left - grappling - crouching | (unreachable) | | | | | out of scope: grapple |
| B6h | Facing right - grappling - crouching - aiming down-right | (unreachable) | | | | | out of scope: grapple |
| B7h | Facing left - grappling - crouching - aiming down-left | (unreachable) | | | | | out of scope: grapple |
| B8h | Facing left - grapple wall jump pose | (unreachable) | | | | | out of scope: grapple |
| B9h | Facing right - grapple wall jump pose | (unreachable) | | | | | out of scope: grapple |
| BAh | Facing left - grabbed by Draygon - not moving - not aiming | (unreachable) | | | | | out of scope: grabbed by Draygon |
| BBh | Facing left - grabbed by Draygon - not moving - aiming up-left | (unreachable) | | | | | out of scope: grabbed by Draygon |
| BCh | Facing left - grabbed by Draygon - firing | (unreachable) | | | | | out of scope: grabbed by Draygon |
| BDh | Facing left - grabbed by Draygon - not moving - aiming down-left | (unreachable) | | | | | out of scope: grabbed by Draygon |
| BEh | Facing left - grabbed by Draygon - moving | (unreachable) | | | | | out of scope: grabbed by Draygon |
| BFh | Facing right - moonwalking - turn/jump left | Upright | Turning | Left | Forward | turn MoonwalkJump |  |
| C0h | Facing left - moonwalking - turn/jump right | Upright | Turning | Right | Forward | turn MoonwalkJump |  |
| C1h | Facing right - moonwalking - turn/jump left - aiming up-right | Upright | Turning | Left | UpForward | turn MoonwalkJump |  |
| C2h | Facing left - moonwalking - turn/jump right - aiming up-left | Upright | Turning | Right | UpForward | turn MoonwalkJump |  |
| C3h | Facing right - moonwalking - turn/jump left - aiming down-right | Upright | Turning | Left | DownForward | turn MoonwalkJump |  |
| C4h | Facing left - moonwalking - turn/jump right - aiming down-left | Upright | Turning | Right | DownForward | turn MoonwalkJump |  |
| C5h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| C6h | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| C7h | Facing right - vertical shinespark windup | (unreachable) | | | | | out of scope: shinespark |
| C8h | Facing left - vertical shinespark windup | (unreachable) | | | | | out of scope: shinespark |
| C9h | Facing right - shinespark - horizontal | (unreachable) | | | | | out of scope: shinespark |
| CAh | Facing left - shinespark - horizontal | (unreachable) | | | | | out of scope: shinespark |
| CBh | Facing right - shinespark - vertical | (unreachable) | | | | | out of scope: shinespark |
| CCh | Facing left - shinespark - vertical | (unreachable) | | | | | out of scope: shinespark |
| CDh | Facing right - shinespark - diagonal | (unreachable) | | | | | out of scope: shinespark |
| CEh | Facing left - shinespark - diagonal | (unreachable) | | | | | out of scope: shinespark |
| CFh | Facing right - ran into a wall - aiming up-right | Upright | AgainstWall | Right | UpForward |  |  |
| D0h | Facing left - ran into a wall - aiming up-left | Upright | AgainstWall | Left | UpForward |  |  |
| D1h | Facing right - ran into a wall - aiming down-right | Upright | AgainstWall | Right | DownForward |  |  |
| D2h | Facing left - ran into a wall - aiming down-left | Upright | AgainstWall | Left | DownForward |  |  |
| D3h | Facing right - crystal flash | (unreachable) | | | | | out of scope: crystal flash |
| D4h | Facing left - crystal flash | (unreachable) | | | | | out of scope: crystal flash |
| D5h | Facing right - x-ray - standing | (unreachable) | | | | | out of scope: x-ray |
| D6h | Facing left - x-ray - standing | (unreachable) | | | | | out of scope: x-ray |
| D7h | Facing right - crystal flash ending | (unreachable) | | | | | out of scope: crystal flash |
| D8h | Facing left - crystal flash ending | (unreachable) | | | | | out of scope: crystal flash |
| D9h | Facing right - x-ray - crouching | (unreachable) | | | | | out of scope: x-ray |
| DAh | Facing left - x-ray - crouching | (unreachable) | | | | | out of scope: x-ray |
| DBh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| DCh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| DDh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| DEh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| DFh | Unused | (unreachable) | | | | | unused: no transition, landing, fall or animation leads to it in the domain |
| E0h | Facing right - landing from normal jump - aiming up | Upright | Landing | Right | Up | landed from Jump |  |
| E1h | Facing left - landing from normal jump - aiming up | Upright | Landing | Left | Up | landed from Jump |  |
| E2h | Facing right - landing from normal jump - aiming up-right | Upright | Landing | Right | UpForward | landed from Jump |  |
| E3h | Facing left - landing from normal jump - aiming up-left | Upright | Landing | Left | UpForward | landed from Jump |  |
| E4h | Facing right - landing from normal jump - aiming down-right | Upright | Landing | Right | DownForward | landed from Jump |  |
| E5h | Facing left - landing from normal jump - aiming down-left | Upright | Landing | Left | DownForward | landed from Jump |  |
| E6h | Facing right - landing from normal jump - firing | Upright | Landing | Right | Forward | landed from Jump, gun Drawn |  |
| E7h | Facing left - landing from normal jump - firing | Upright | Landing | Left | Forward | landed from Jump, gun Drawn |  |
| E8h | Facing right - Samus drained - crouching/falling | (unreachable) | | | | | out of scope: drained by a metroid or Mother Brain |
| E9h | Facing left - Samus drained - crouching/falling | (unreachable) | | | | | out of scope: drained by a metroid or Mother Brain |
| EAh | Facing right - Samus drained - standing | (unreachable) | | | | | out of scope: drained by a metroid or Mother Brain |
| EBh | Facing left - Samus drained - standing | (unreachable) | | | | | out of scope: drained by a metroid or Mother Brain |
| ECh | Facing right - grabbed by Draygon - not moving - not aiming | (unreachable) | | | | | out of scope: grabbed by Draygon |
| EDh | Facing right - grabbed by Draygon - not moving - aiming up-right | (unreachable) | | | | | out of scope: grabbed by Draygon |
| EEh | Facing right - grabbed by Draygon - firing | (unreachable) | | | | | out of scope: grabbed by Draygon |
| EFh | Facing right - grabbed by Draygon - not moving - aiming down-right | (unreachable) | | | | | out of scope: grabbed by Draygon |
| F0h | Facing right - grabbed by Draygon - moving | (unreachable) | | | | | out of scope: grabbed by Draygon |
| F1h | Facing right - crouching transition - aiming up | Crouched | Crouching | Right | Up |  |  |
| F2h | Facing left - crouching transition - aiming up | Crouched | Crouching | Left | Up |  |  |
| F3h | Facing right - crouching transition - aiming up-right | Crouched | Crouching | Right | UpForward |  |  |
| F4h | Facing left - crouching transition - aiming up-left | Crouched | Crouching | Left | UpForward |  |  |
| F5h | Facing right - crouching transition - aiming down-right | Crouched | Crouching | Right | DownForward |  |  |
| F6h | Facing left - crouching transition - aiming down-left | Crouched | Crouching | Left | DownForward |  |  |
| F7h | Facing right - standing transition - aiming up | Upright | StandingUp | Right | Up |  |  |
| F8h | Facing left - standing transition - aiming up | Upright | StandingUp | Left | Up |  |  |
| F9h | Facing right - standing transition - aiming up-right | Upright | StandingUp | Right | UpForward |  |  |
| FAh | Facing left - standing transition - aiming up-left | Upright | StandingUp | Left | UpForward |  |  |
| FBh | Facing right - standing transition - aiming down-right | Upright | StandingUp | Right | DownForward |  |  |
| FCh | Facing left - standing transition - aiming down-left | Upright | StandingUp | Left | DownForward |  |  |

**The empty column (gap 7).** `$94:8B2B`'s offset 10h. Vertical move, down (`$94:8780`): A =
10h − (target bottom % 16) − 1, stop when A ≤ 0, so only a target bottom on row 15 stops, with
`$12` = `$12` + 0 and `$14` = 0 (a landing on row 15; the integrator's "never stops her" holds for
the check after a horizontal move only). Up (`$94:8707`): A = 10h − (15 − target top % 16) − 1 =
target top % 16, so only a target top on row 0 stops. After a horizontal move (`$94:87F4`): the
floor's A = 15 − bottom % 16 ≥ 0, never lifted; the ceiling's A = top % 16, so a top edge on row 0
is pushed 0 px and seated (`0DBA` = 1). Vectors: C-slope-ceiling-empty (target top on row 0:
stopped), C-slope-ceiling-empty-on (row 1: moves on), C-slope-floor-empty (target bottom on row
15: lands), all constructed.

## Size baseline (tokens, 2026-11-07)

Restated at c7f927ab6 with declarations apart (Jim, 2026-11-07: types and the bridge are not
counted against the model). Old code, the files below: logic 13,342, types 1,781, data 153. The
round-7 model: logic 9,265, types 2,090, data 7,386. The bridge (`game/src/samus_world*`, the
integrator's): logic 3,272, types 192, data 11. The tables below are the earlier logic-and-types
counts.

Counted with `tools size` at 29c4d0876, tests excluded, as logic and data (the values of
constants and statics). The old code the domain replaces:

| Old code | Logic | Data |
| --- | ---: | ---: |
| `game/src/model/samus.rs`, `model/samus/` (motion, its pose) | 5,160 | 0 |
| `game/src/motion.rs` | 971 | 24 |
| `game/src/collision/world.rs` | 1,335 | 1 |
| `game/src/model/collision.rs` | 543 | 0 |
| `game/src/pose/contact.rs` | 1,065 | 0 |
| `game/src/collision.rs`, the block scans (`SlopeFlag` to `scan_y`) | 2,499 | 0 |
| `game/src/pose.rs`, the pad's pose requests (`PoseRequests` to `input`) | 572 | 91 |
| `game/src/collision/solid_enemy.rs` | 1,220 | 0 |
| `game/src/pose/fit.rs` (pose fit, in the domain from 4d34fe9ae) | 1,758 | 37 |
| Total | 15,123 | 153 |

`collision/inside.rs` (696) is not counted: the engine keeps its touch handling. The old engine
read the pose chart and the sprite list from the ROM, so its data column is near zero.

The new code at the same commit, round 5:

| New code | Logic | Data |
| --- | ---: | ---: |
| `clean_types` | 1,764 | 3 |
| `clean_model/samus-motion` (catalogue, chart and its rows, collide, lib, rules) | 12,320 | 3,369 |
| `game/src/samus_world*` (the world, the frame, the pose map) | 2,250 | 3,701 |
| Total | 16,334 | 7,073 |

The pose map (`samus_world/pose.rs`, 3,610 of data) is which sprite shows each pose: the drawing
domain's data, held in the bridge until the engine draws from the pose itself.
