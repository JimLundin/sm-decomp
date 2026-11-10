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
8, 12h, 13h, 14h); `0B36` = 0 only with magnitude 0 outside those types. On the ground `0B36` = 2
with a non-zero magnitude is plain: it is the speed she keeps on the ground (`FollowingTheGround`), which the
vertical speed shows (the ground handlers' vertical move `$90:923F` reads no speed, and a ground
movement type landing is "no change" in `$90:E676`). `0B36` = 2 with magnitude 0 on the ground is
not plain: a change of shape or the ball on the ground then moves her under the pull (`$90:92C7` and
`$90:92B8` test the direction alone),
where with `0B36` = 0 it doesn't; no record has one outside a bomb jump
(samus-bomb-jump-vertical 1036), and a start gives it as `falling` (spec §1.2, BG2; gap 29). From
the checked frame
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

**Inputs.** pad held `8B`, newly pressed `8F`; the items switched on are `09A2`, the equipped
items (0001 varia, 0002 spring ball, 0004 morph ball, 0008 screw attack, 0020 gravity, 0100
hi-jump, 0200 space jump, 1000 bombs, 2000 speed booster, 4000 grapple, 8000 x-ray), which is
progress's items switched on; the bridge hands that set over as it is;
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
| AimingAndShooting | `$91:EADE` (the step: any prospective pose `0A28` of the running movement type, her own included), `$91:82E4` (the failed lookup's decelerate command keeps her pose as the prospective pose, so a running pose braking steps every frame), `$91:81E4` (a found row naming her own pose sets no prospective pose: no step), `$94:971E` (its block stop), `$90:A3AD` (the shot) |
| RidingTheElevator* | `$90:A392`, `$90:A397` (two branches of the facing-forward handler) |
| Jumping | `$90:98BC` (the speed table), `$91:FBBB` (take-off), `$91:F624` and `$91:F8D3` (jump starts), `$90:A488` (space jump); jump held through: the count `0AF4` (`$90:EAB3`, from `$90:EB35`'s draw once a gameplay frame: `INC` with jump held now and in `0DFE`, else `STZ`), the one-frame pose input handler `$90:E926` (count 1 to 8: jump ORed into `8F` for `$91:8000` alone, `STZ 0AF4` at `$90:E93A`), installed by `$91:F1EC` (landed, downward result grounded: `$90:E676` gives it for movement types 2, 3, 6 and 14h, and Ah and 19h out of scope) and by animation delay instruction 8 `$90:8370` (the turns, A4h-A7h and E0h-E7h, BFh-C4h; not when `0A28` is 4Bh, 4Ch, 19h or 1Ah) |
| FollowingTheGround | `$90:923F` (the pull), `$90:E61B` (walking off); the kept speed: `$90:923F` again (the ground handlers' only vertical move reads and writes no speed), `$90:E676` (landed in a ground movement type: no change, so `$91:F010` doesn't run), `$90:A61C` with `$90:92C7` (the change of shape moves by it when `0B36` ≠ 0; `$90:A640` clears it on landing); the ball on the ground: `$90:A521` (morph ball) and `$90:A69F` (spring ball) call `$90:92B8`, the clone of `$92C7`, on both branches (`A549`/`A569`, `A6C7`/`A6E7`): with `0B36` ≠ 0 it runs `$90:90C4` and `$90:90E2` (moves by the speed, then the pull grows it) and returns carry set, which skips `$90:923F` (no ground pull) and, at rest, the halt at `A551`-`A561`; `$90:E676` gives 04 for movement types 4 and 11h, so `$91:E931` returns and the floor doesn't land it (BG1), `$91:EFEF` (walking off: speed 0, unless `0B36` = 1: `$91:EFEF` then keeps the speed and the direction, item-charge-beam 7 → 8 keeps 2.4000 up), `$90:98BC` (a jump sets it) |
| MovesAsShePosed | `$90:A337` (the handler of the frame-start pose), `$91:EB88` (the pose taken after) |
| RisingAndFalling | `$90:90E2` and `$90:913A` (gravity), `$90:910C` (the limit), `$90:8FDC`, `$90:9046` and `$90:90C4` (the rise's end: `0B2E` read as signed, ends when negative), `$91:F22B` (bounce 2 at 0.0); `rising` is `0B36` = 1 at the frame's end, which the ground handlers keep as they keep the speed (`$90:923F` writes neither), so a crouch the unmorph's end hands over in a rise holds it (item-charge-beam 7) |
| DriftInTheAir | `$90:900D`, `$90:9174`, `$90:91A8` (normal jump and fall), `$90:906C` and `$90:9093` (spin; its stop still calls `$90:8EA9`, which adds the extra run speed) |
| Moonfall | `$91:EBAF`, `$91:F36E` (the hand-over), `$90:913A` (no limit with direction 0), `$90:9D35` (the wall-jump test reads no direction), `$90:99CD` (the kick sets direction up); the turn in it: `$90:A790` and `$90:A7AD` (turning in a jump, in a fall) call `$90:92B8`, which with `0B36` = 0 returns carry clear, so `$90:923F` (the ground pull) runs and `0B2C`/`0B2E` are not touched; `$90:E65A`/`$90:E676` give 04 (no change) for movement types 17h and 18h |
| WallJump | `$90:9D35` (arming and kick test), `$90:90B3` (one distance, `$90:9E9F` = 8, for both), `$91:FA0F` (kick), `$94:967F` (the probe) |
| Landing | `$91:F010`, `$90:A640` (upright and changing), `$91:F1FC`, `$91:F25E` (the ball): by `0B20`, `$91:F20A` (not bouncing: `0B2E` `CMP #3`, read after `$90:90E2` grew it, since `$90:90E2` copies the speed to `$12.$14`, grows `0B2C`/`0B2E`, then moves by the copy), `$91:F22B` (first bounce: any speed, rises at `$90:9EB5` − 1 = 0.0), `$91:F245` (second bounce: rests); spring ball `$91:F27F`, `$91:F2A3`, `$91:F2C0` alike |
| BlocksStopFlush | `$94:9722`, `$94:976B` (a 0.0 move skips block collision), `$94:8F49`, `$94:8F82` (horizontal and vertical block collision), `$94:8D2B`, `$94:8DBD` (stepped), `$94:9543`, `$94:959E`, `$94:95F5` (the order), `$90:E4AD` (the limit) |
| WallsAndCeilings | `$90:E5CE` (wall), `$90:E606` and `$91:EFDF` (ceiling) |
| SolidEnemies | `$A0:AAC8`, `$A0:AA53`, `$A0:AABC` (overlapped: next enemy, then 0 so blocks run), `$90:9D80`, `$90:9DF8`, `$90:93B1`, `$90:93EC` and `$90:9440` (enemy test first; on a hit the move skips block collision, as the bank log's zebetite-skip note says) |
| SlopesCarry | `$90:9376`, `$94:87F4` (seat after every horizontal move, sets `0DBA`), `$90:924C` (the pull reads `0DBA`), `$94:976B` (a 0.0 vertical move skips block collision), `$94:976F` (cleared by a non-zero vertical move), `$94:8E38` (square slope stop sets it), `$94:86FE` (rising into a non-square ceiling slope: `$12 = max(0, ...)` and `$14 = 0`, so no move), `$94:87E4` (the floor slope stop is `max(0, ...)` too: never upward); the carry: `$94:95A1` (`$18` = y + `$12.$14` with the subpixel carry, so the target row includes it) and `$94:879E`..`$94:87EE` (`$12 = max(0, $12 + top − 1 − ($18 + 0B00 − 1) % 16)`, `$14 = 0`: the move is the uncarried pixel part corrected by the carried row); `$94:8790`/`$94:8717` (only the block at her centre column), `$94:87B9`/`$94:8742` (a ceiling never stops a downward move, a floor never a rise) |
| WordwiseTopSpeed* | `$90:9A99` |
| PoseFollows | `$91:EB88` (one change a frame: super special `0A2C`, then special `0A2A`, then `0A28`), `$91:E8B6` (the vertical collision result `0DC6` overwrites the pad's `0A28` and sets command 5), `$91:EADE` (the wall stop overrides both); the spec's three items fold the wall stop into `WhatHerMovesMet` and the lookup failure into the chart's last rows, `$91:EC85` falling into `$91:EC8E` (the dash goes with the speed); the morph ball: `$91:F404` runs `$91:FDAE` then the pose's initialisation, and for the morphing transition 37h/38h `$91:F7CE` tests `09A2` bit 0004: without it `$91:F7EC` puts the old pose back, so `$91:F404` returns carry set (the pose changed under it) and `$91:EBEE` skips the lowering and `$91:ED0E`'s write: no lowering, no placement, no fall-back row (r02:489, pose 28h with 0600h pressed and `09A2` = 0, stays 28h at y 00B0) |
| PadChart | `$91:81A9` (a row fits when its newly-pressed and held masks are all set; first fit wins; same pose: no change), `$91:81F4` (bindings to default bits), the transition table `$91:9EE2`..`$91:B00F`, `$91:F404` (crouched into standing: the bottom edge kept, y − 5; `$91:8087`'s own y − 5 is dead); the last rows: `$91:81DB` (pad 0 or no fit, unless the pose's table is empty), `$91:82D9`, `$91:8304` and `$91:8332` (the effect per movement type), the pose definitions `$91:B629` byte 2 (the pose with no buttons), `$91:EC50` (decelerate), `$91:ECD0` (stop), `$91:EC85`, `$91:EC8E`; `$91:8146` (movement type Fh: no lookup at all, the changing-body row) |
| Aiming | the table's aim rows per pose, `$91:E9F3` (landing keeps the aim, down becomes forward), `$91:EB74` (the wall stop keeps the aim) |
| WhatHerMovesMet | `$91:E8F2` (falling poses, `$91:E921`), `$91:E95D`..`$91:EA47` (landing poses, ball bounces), `$91:EABE` (wall-jump pose), `$91:EADE` (ran-into-wall poses, the aim step's block); walking off from the crouch: `$90:E65A` gives 00 (airborne) for movement type 5 as for 0, 1, 10h and 15h, so `$91:E921` hands over 29h/2Ah, upright falling (item-charge-beam 7 → 8: 28h → 2Ah) |
| AnimationEnds | the animation delay table `$91:B010` (its go-to-pose instructions), `$91:EB88` (the super special pose wins); the unmorph's end: 3Dh/3Eh's sequences `$91:B4E6`/`$91:B4EA` end in instruction Dh (`$90:84B6`) to 27h/28h, the crouch, with no test of the air (records: boss-kraid-defeated 779-782, item-charge-beam 5-7, item-reserve-tank 1115-1116); the morph's end: instruction 9 (`$90:839A`) chooses the ground ball when the speed magnitude `0B2E`.`0B2C` is 0 and the air ball otherwise (spec question 20) |

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
| PoseFitAfterMoves | `$90:E725` (movement `$90:E94B`, then `$91:E8B6`, then `$91:EB88`), `$91:F404` (`$91:FDAE` pose fit, then `$91:F433` initialise, then `$91:FBBB` the jump transition with `$91:FC8A` the 10 px lift); the push itself `$91:FDAE`..`$91:FFED` with `$94:96AB` and `$A0:A8F0`; the camera: `$91:FF42`, `$91:FF6F`, `$91:FF89`, `$91:FF98` and `$91:FFE9` write `0B14` = `0AFA` after the push (not `0B16`); `$91:FF87` (nothing found) and a refusal write nothing; `$90:96FF` makes the camera's speed from `0B14`.`0B16`, and `$90:957A` sets them to her y at the frame's end | `$91:EADE` (the aim step) runs before `$91:F404` |
| ShrinkingLowersHer | `$91:F758` (transition poses set command 7), `$91:ECDA` with its table `$91:ED36` and `$94:96AB` (the look, result ignored); the placement: `$91:ED0E` writes `0B14` after the lowering for the poses below DBh (35h-38h lowered, 3Bh-3Eh by 0) and F1h-F6h (5 px, `$91:ED2D`); F7h-FCh (standing up aimed) and the rest from DBh return at `$91:ED26`/`$91:ED2B` with no write | runs after `$91:F404` only when the pose stands (`$91:EBEE`) |

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

## Round 7's gaps (2026-11-07)

The round-7 model failed 9 of 203 vectors. Each is settled from the records
(`$TEMP/respec-motion/out`) and the bank logs; the scripts are `$TEMP/r7v-scripts/press.pl` and
`apply.pl`.

**Presses (SH1, SH2, SH3, C-shrink-floor).** The pad chart reads newly pressed buttons for every
button, not only jump: the transition-table lookup `$91:81A9` matches each row's required newly pressed
input (`$8F`) as well as its held input. The vector format gave only `jump_pressed`, so
these four, whose crouch or morph needs down newly pressed, could not be stated. New key
`pressed`: record N's pad AND NOT record N − 1's, jump left to `jump_pressed`, bits outside the
format's buttons (Select `2000`, item cancel Y `4000`) dropped. SH1 r02:94 8220 → 8400 (down), SH2
samus-bomb-jump-vertical:848 (down), SH3 r04:311 (right, down); C-shrink-floor is SH3's frame, so
down too. `press.pl` over every recorded vector: 30 frames press a non-jump button, written by
`apply.pl` on the checked frame and on lead-in frames (R62's and W15's first lead-in frames press
run and right, which no inference from the pads can give). It also found S1's pad short of aim
down (samus-input-poses:511 is 8120): fixed; a ground turn with forward held keeps the turn either
way. r04:307 holds left and right together (8380, R102), against the spec's "no record does"
(`PadChart`): answered below (left and right held together).

**Against the wall with forward held (R18, R88, R117, R149).** `$91:EADE` runs for any prospective
pose of movement type 1: it moves her 1 px forward with block collision (`$94:971E`) after the
solid-enemy test (`$A0:A8F0`), and on a collision sets the prospective pose to ran into a wall by
the aim (`$91:EB74`). Pose 89h/8Ah's row for forward held gives 09h/0Ah, so the step comes and the
wall stops it. Records: samus-input-poses:537-539 (89h, pad 8100/8120, x 00AB.FFFF kept),
r02:84-88 (8Ah), r05:267-270 (8Ah), area-norfair:327-330 (89h with right held four frames, x
004B.FFFF kept). Each arrived by a run that stopped flush (538 from 00A9.C7FF at 4.0000; 327 from
004A.6FFF), so a block stood in her rows at x + 5 (right) or x − 5 (left). The worlds lacked it:
R18 `wall_right = 176`, R149 `wall_right = 80`; R88 and R117 face left, which the format has no
wall key for, so a `solid_block` beside her feet ([1056, 176], [1760, 176]) stands for it. The
row it was in is not in the dump; any row of her body stops the step the same way. The spec was
right; `AimingAndShooting` now says so for this case.

**C-aim-enemy.** 1.D000 + 0.3000 = 2.0000; the wordwise cap (`$90:9A99`) against 2.C000: pixel
parts equal, 0000 not equal to C000 and not below it, so the top is taken: 2.C000. The run reaches
the enemy (end 268.C000, pushed 1 px: centre 269 against 281, closer than 13) and stops 2 px on at
010C.0000, subpixel kept: a wall stop while running (`$90:E5CE` sets `0DCE` for any solid
collision, enemy or block), so `$91:EADE` takes the ran-into-wall pose from the current pose's
aim (Forward, 89h) and no step comes. The implementer's result was right. The vector's values fit a
2.0000 run, which running can't keep. C-aim-enemy now starts at 010B.0000 with 0.D000 (grown to
1.0000, kept): the run ends at 010C.0000 touching the enemy without reaching it (centres 268 and
281, both subpixels 0 so no push), and the step reaches it: no move, y subpixel cleared, aim
`DownForward`. The old frame is kept as C-aim-enemy-stop with the right outcome (AgainstWall
`Forward`, 010C.0000, 010B.FFFF, speed and dash 0).

**Left and right held together (round 8's queue, 2026-11-07).** r04:307-308 holds 8380 (R102: a
spin facing right, both just pressed): the spin drifts on, `x` +0.E000 (0.C000 built, the 0.2000
dash), so her motion reads the side she faces as held. The chart: `$91:81F4` passes the d-pad bits
through unmasked and `$91:81A9` takes the first row of the pose's list (`$91:9EE2` + 2 × pose) whose
masks fit, so with both held a row naming forward and one naming back both fit and the list's order
decides. `$TEMP/arms/lrtab2.pl` runs every pose of `spec.pl`'s situations, both facings, every other
button held or pressed, with both sides held: the outcome is always the one of a single side held
alone (never a third), forward's or back's by situation. `$TEMP/arms/lr.pl` adds the spec's
both-held ranks to `$TEMP/adv16/spec.pl`; `$TEMP/arms/chk_lr3.pl` (adv16's strict checker on the
both-held pads) then leaves 2,160 mismatches, all the take-off's facing-left back row (30h), the
row adv16 left out as unreachable, which with both held behaves as for right held; adv16's
single-side check with the added ranks is unchanged at 1,080. The situations where the list's order
departs from the ranks: standing, landing and against the wall (left/right turn row before forward;
forward with up or down before the moonwalk's two-button row); running and the spin (forward
first); the take-off (jump + back first); the straight jump, aimed down and gun drawn included (back before forward, and before
forward + aim without jump: the forward + aim rows name jump); the moonwalk (the way she looks
first); the ball on the ground (right first); the spring ball falling (7Dh/7Eh: the other side first)
and hopping (7Fh/80h: right first).

## Round 9's gaps (2026-11-08)

Gaps 20 and 24-28 of `samus-motion.gaps.md`, from the round-8 harness. The records are read with
`$TEMP/collab/r9-spec/lvl.sh` (the harness's decode, step and oracle, plus `0DBA`, `05B6`,
`0B14`/`0B16`, `0DA6`/`0DA8` and the blocks around her from `7F:0002` and `7F:6402`), the slope
shapes from the ROM table `$94:8B2B` (BTS & 1Fh: 12h a 45-degree triangle, 13h a rectangle whose
surface is row 0, bit 40h the x flip, bit 80h a ceiling).

**Gap 26 (AS4).** samus-space-jump-chain 12: running (09h), `0B46`.`0B48` = 0, pad 0. The lookup
fails (`$91:81A9`), `$91:8304` gives running's 1 (decelerate), and with base speed 0 makes it 2
(stop), carry clear, so the prospective pose is pose 09h's no-button pose (01h). The running
handler then accelerates (`0B4A` = 0): 0.3000, x 0055.2FFF → 0055.5FFF. `$91:EBF5` runs the stop
(`$91:ECD0`: `0B4A` = 0, cancel boosting), which leaves the speed. Record 13: 01h, 0.3000, `0B4A`
0. With a base speed other than 0 at the lookup, `$91:EC50` runs after the move and tests the
speed again (0 → the stop; AS3's frame).

**Gaps 27, 20 and 24 (SL1-SL3, C-slope-carry-kept).** The kept vertical speed takes no part: the
running handler (`$90:A3E5`) moves by `$90:8E64` and pulls by `$90:923F`, neither reads `0B2C`/
`0B2E`; the only slope code that does is the horizontal reaction `$94:84D6` (the walk factor,
only at speed 0). The pixel is the carry: `$94:95A1` builds the target `$18` with the subpixel
add, so from y subpixel FFFF any pull with a subpixel part lands the target row one lower, and
`$94:87DB` corrects the uncarried `$12` by that row. samus-shinespark-left 9: y 048B.FFFF, pull
6.C000 (2.C000 + 3.0000 + 1), target 0492.BFFF, bottom row 6 of block (60h, 4A0h), BTS 52h
(triangle, x flipped) at x 62h: top row 3; `$12` = 6 + 3 − 1 − 6 = 2: 048D.FFFF (the model's
048E.FFFF is the uncarried row). samus-mockball 91 (SL1): 04B9.FFFF + 9.4000, bottom row 9 of
(280h, 4C0h), top 5 at x 284h: 9 + 5 − 1 − 9 = 4, 04BD.FFFF; the rectangle at (270h, 4C0h) is
not her centre's block. 92 (SL2): 04BD.FFFF + 9.4000, row 13, top 13 at x 28Ch: 8, 04C5.FFFF.
chain-0220-D017 39: 006B.FFFF + 4.9000, row 4 of (E0h, 80h), top 3 at x E2h: 2, 006D.FFFF.
chain-0220-D017 43 (SL3): 0078.FFFF + 4.D000, row 1 of (F0h, 90h), top 3 at x F2h: above the last
air row, so no stop; the rectangle at (E0h, 90h) is not her centre's; she moves the whole pull
(007D.CFFF) and walks off (29h). Both rows have 0B3C set and a dash; the spring ball's 5.0000 and
the run's 1.0000 are not changed by the booster in these frames (the records show them kept and
grown by 0.1000).

**Gap 28, the camera.** `0DA6`.`0DA8` at N+1 is |y − (`0B14`.`0B16`)| + 1.0, the pixel words
compared first (`$90:96FF`); `0B14`.`0B16` at the frame's start is her y then (`$90:957A`). From
it each vector's `camera_from`: PF1 (item-morph-ball 1310: 02C0.FFFF → 02B3.FFFF, speed B.0000:
02BD.FFFF), PF2 (r02 178: 1.5C00, 00AB.A3FF), PF3 (r02 548: 1.7800, 00AB.87FF), U1 (r07: 1.0000,
008B.FFFF), W13 (item-morph-ball 1227: 0.6C00 with the pixels equal, 029B.9400), SH1 (r02 94),
SH2 (samus-bomb-jump-vertical 848), SH3 (r04 311: 1.0000 after a 9 px lowering, 009D.B7FF), W11
(item-morph-ball 682: 2.C000, her start 02A9.0000). item-morph-ball 738 (18h → 52h in the air, the
look down meeting the floor: 1.8C00, 023D.73FF) agrees.

**Gap 28, the turn in the moonfall (MT1).** chain-0232-D5A7: 17 moonwalk 4Ah with jump pressed,
18-23 turn BFh (`0A94` = 1 at 23), 24 spin 82h with `0B36` = 0 (the moonfall), right pressed (25:
81h), left pressed at 33 (34: 82h), down pressed at 35 (36: 18h, a straight jump aimed down; the
moonfall goes on, `0B2E` FFFE), right pressed at 37 (38: 92h, movement type 17h, radius 13h).
38 → 39: x 0078.BFFF → 0078.7FFF (braked 0.C000 → 0.4000, the old way), y 0195.F3FF → 0197.33FF
(1.4000: the pull), `0B2E`.`0B2C` FFFE.7800 kept. The same to 43; 43 → 44 the turn ends into 17h
and 44 → 45 falls by 1.8800 (FFFE.7800 → FFFE.5C00). The ledge is (60h, 1A0h); the pit is air.
Lead-in pads: 0280 (jump pressed) at 17, 4200 at 18 (item cancel `4000` dropped), then as listed.

**R9-2, the second bounce.** No record holds a second bounce; C-bounce-second, C-bounce-third and
C-bounce-pulled are worked from `$91:F20A`..`$91:F245` and `$90:90E2`.

**SlopesCarry's units (adversary 27, item 7).** Against `$94:84D6`..`$94:8919`, `$94:8DBD` and
`$94:9763`: the entry stated every rule but two counted nothing (the ceiling push cited records
only; the clamp is original only and, after the floor check, is reached only by a move with no
horizontal move before it) and one rule was missing (the carry). The re-lay adds the carry arm
(SL1, SL2, C-slope-carry-kept), gives the ceiling push C-slope-ceiling-push (samus-walljump-16:44
started following the pad, which the spin with forward held moves the same), and widens the
passing arm to what `$94:8742`/`$94:87B9` and the centre-column test let through
(C-slope-ceiling-side): 11 counted arms to 13, 12.0 units to 16.3. After adversary 28 (F4) the
stop is stated once, as the formula's arm with the no-carry, carry and clamp cases as its
consequences: 12 arms, 15.0 units.

**Adversary 28's fixes.** F1: the moonwalk's jump turn keeps her facing (4Ah facing left into BFh
facing left, chain-0232-D5A7 17 → 18), the one turn that doesn't face back; `PadChart`'s cell says
so. F3: `camera_from` worked from `$91:FDAE`..`$91:FFED` and `$91:ED0E` for C-fit-ceiling (pushed
down to 0100, the unmorph's command 7 writes the same: 0100.8000), C-fit-land-tunnel (the crouch
instead raises her to 0110, `$91:FFE9`: 0110.0000) and C-fit-step-enemy (refused: no write, so the
frame's start, 00F7.8000).

**Gap 29, falling at 0 on the ground (BG2; the model's since round 10).** The bomb
jump's handler (`$90:E025` → `$90:E032`, straight `$90:E066`) runs `$90:90E2` and never changes her
pose: the ball stays 1Dh (movement type 4). When the rise is spent, `$90:90C4` sets `0B36` = 2 with
`0B2E`.`0B2C` 0; the next pass sees the direction down and ends the bomb jump (`$90:E07D`, `$90:8F1B`:
handler `$A337`) without a move. A knockback taken in the ball keeps her pose (`$90:DF15`) and its
end leaves her the same (`$91:F31D`). So a bomb jump's or a knockback's end is the only way to
start a frame on the ground with `0B36` ≠ 0 at magnitude 0. samus-bomb-jump-vertical 1036 is the
first normal frame: `$90:A521` → `$90:92B8` tests `0B36` alone (`$90:92BB`: none returns carry
clear), so the ball runs `$90:90C4` and `$90:90E2` (0 px) and returns carry set, which skips the
ground pull `$90:923F` (`$90:A54C`/`$90:A56C`) and, at rest, the halt `$90:A551`-`$90:A561` (with
`0B36` = 0 and no floor the pull moves her the whole of it: SL3's walk-off, unchanged). A 0.0 move
skips block collision (`$94:976B`), so it finds no floor: the walk-off `$91:EFEF` sets speed 0,
down, and the fall pose (31h) starts from rest (record 1037: 0, down). The spring ball's handler
(`$90:A69F`, movement type 11h) and the change of shape (`$90:A61C` → `$90:92C7`) test the
direction alike. BG2 is 1036 → 1037, its start's `falling` the `0B36` = 2 at magnitude 0. The
harness masked y there until the model made it (movement type 4 or 11h, `0B36` = 2, magnitude 0;
over the 32 rows it met 1036 only). Round 10 makes it: the bridge reads `0B36` = 2 at magnitude 0
in a ground pose as `Vertical::GroundFalling`, the mask is gone and 1036 → 1037 matches
(lane/wire-r10).

**Falling at 0 in a standing pose (BG3; adversary 32).** The ground handlers for standing
(`$90:A383`), running (`$A3E5`), crouching (`$A573`), turning on the ground (`$A67C`),
moonwalking (`$A694`) and against the wall (`$A75F`) move vertically only by `$90:923F`, which
never reads `0B36`; landing gives "no change" (`$90:E676`), so `0B36` = 2 at magnitude 0 stays
until a jump, a walk-off (`$91:EFEF`, which writes the same) or a transition (`$90:A61C` →
`$90:92C7`: `$90:90C4`/`$90:90E2` move 0 and grow the magnitude by 0.1C00; a 0 move skips block
collision, so `0DC6` is "falling" with `$90:E65A`[Fh] = 04, no walk-off). `$91:ECDA` clears `0B36`
only with `0B20` ≠ 0. Reached only by a knockback's end (`$91:F31D`) or a bomb jump's, out of scope.

**Gap 30, the moonwalk's jump (MT2).** Jump pressed while moonwalking (49h, facing word 8) takes
C0h (facing word 8, movement type Eh) after the frame's move; `$91:F8D3` (turning on the ground,
its pose from `$91:F9EA` for the moonwalk) adds `0B42` to `0B46` and sets `0B4A` = 1 whatever the
facing. With the facing kept, `0B4A` = 1 points her speed leftward (`$90:8EA9`), against the
0.8000 the moonwalk just moved her. chain-0014-96BA 4 → 5: 019B.8000 → 019C.0000, `0B46`.`0B48`
0.8000, `0B4A` 1; 5 → 6: `$90:9A7E` brakes 0.8000 (the turn's table) to 0, x kept, and the turn's
let-go (nothing held) clears `0B4A`. Modes 1 and 2 brake alike there, so no later move shows it,
but the scroll (`$90:95A0`, run after the Samus handler in `$82:8B8A`) reads `0B4A` = 1 at
`$90:95D4` and leads the camera backwards, as for the moonwalk (`$90:95C2`): the skid is the
camera's input. chain-0232-D5A7 17 → 18 (4Ah → BFh) and samus-input-poses 118 → 119 (49h → C0h)
show `0B4A` = 1 the same.

**Gap 31, the speed booster on a sloped floor (harness mask).** With `0B42` ≥ 2.0 running with
run held, `$90:9789` grows the dash past the model's cap, so x is masked already; on a slope the
surface row at her centre column (`$94:95A1`, `$94:87DB`) and the pull `$90:923F` (`0DBC`.`0DBE` +
1) both follow it, so y is the booster's there too. A local run with `dash_cap` 10.0000 and x
unmasked: y matches on every masked frame except shinespark 110, 118, 142 and 146 (the same on all
four rows), where x is a pixel ahead (03B4.0FFF against 03B3.FFFF at 110; the model's growth of
the dash isn't the booster's), so those differ by the booster too. The mask's test: a slope block
(type 1) within one block of her feet (her centre's column ± 1, her bottom's row ± 1); it takes
y on 24 frames of samus-mockball and 81 of each shinespark row. Shinespark 198 (left, up-left) and
185 (right, up) are outside it and still differ (y 1 px).

## The camera's asks (C1-C3 of camera.gaps.md, 2026-11-08)

Read with `$TEMP/collab/motion-out/mdump.sh ROW FRAME...` (records N and N+1, the harness's
decode, step, oracle, placement and masks, and the blocks around her; `BUILD=1` rebuilds against
the newest `verify` rlib in integrate-ground's target).

**C1, `rising`.** The camera's aim reads `0B36` = 1 at the scroll (`$90:95A0`, after the Samus
handler). `0B36` = 1 from the rise's start (`$90:98BC` and `$91:FBBB` a jump or the take-off,
`$90:99CD` the kick, the bounces `$91:F22B`, `$91:F27F`) to the frame-start test that finds
`0B2E` negative or jump let go (`$90:8FDC`, `$90:9046`, `$90:90C4`: `0B36` = 2, magnitude 0) or
a ceiling (`$90:E606` only sets the vertical collision result `0DC6` = 4, hit ceiling; the writer
is `$91:EFDF`, run from `$91:EFC4` by that result, which zeroes `0B2E`.`0B2C` and stores `0B36` =
2 at `$91:EFEB`; recorded: samus-walljump-16 42 → 43, a spin at 4.B000 up with jump held stopped
by the ceiling's block at row 430h, y kept at 044B.37FF, `0B36` 1 → 2 at 0.0000, and the same at
samus-walljump-28 43, the model agreeing). Records: r04:474 (J2, take-off: 1, 4.E000), r03:470 (J1, spin
start: 1), samus-ceres-push:989 (E1: 1 with `0B2E`.`0B2C` FFFF.F400, past zero) and 990 (E2: 2,
0.1C00), r02:173 (R91, jump let go: 2). Each frame's `rising` is the spec's arm; the model's
`Vertical::Rising` at the frame's end is it.

**C2, terrain-quicksand 62, 75, 151, 389.** The blocks around her are type 3 (special air), BTS
85h with 81h at row C0h: quicksand, air to the model. 62: 82h → 6Ch (spin, left and aim down
held: the spin's aim row, W15's), extra y displacement `0B5C`.`0B5A` 1.C000: 00AC.A800 + 0.1C00
+ 1.C000 = 00AE.8400, and the record's 00A7.8400 is 7 px up, g = 13h − Ch, room 0 below (the
look down stopped by the sand's reaction at once); `$91:FF42`.. write `0B14` = 00A7 after the
push, so the camera's speed is from 00A7.A800 and its pixel equals her y's: the view holds. 75:
81h → 6Bh, 00AD.B000 + 0.1C00 + 1.C000 − 7 = 00A8.8C00, the same. The model there has no push
and no sand: the body grows about its centre (nothing in reach), so `camera_from` is her start y.
151: A5h → 36h, `1E71` = 1, down pressed; the frame moves her 0.3000 (00AA.F000 → 00AB.2000)
and the crouch lowers her 5 px (00B0.2000), `$91:ED0E` writing `0B14` = 00B0. 389: A6h → F3h,
`1E71` = 2, 00B6.73FF + 0.3000 + 5 = 00BB.A3FF. The model walks off there (it sees no floor:
y + 1.0000, falling), so no crouch starts. Every one is a frame `pushed(next)` defers (x, y,
speeds masked in samus-motion's harness). Answer: the placement rules are the spec's, the
positions and what the looks meet are the push's; the camera's harness masks y (`camera_from`) on
the frames samus-motion's harness defers as a block-inside push.

**C3, r05:400.** Not a landing: a knockback's end. r05 391-393 run the knockback's handler
(`DF38`, pose 54h, `18AA` 9 → 6); 393 → 394 hands over to the normal handler in 4Fh, then 4Eh and
52h (movement type 2) with `0B36` = 0 and a negative magnitude (FFFF.E400 at 394, growing 0.1C00
a frame through FFFF.5800 at 399, no limit with direction 0, `$90:913A`), `0A52` = 5 throughout,
`18AA` 0 from 399. The harness's decode reads direction 0 in the air as the moonfall, and the
moves match the oracle to 399. At 400 the floor (row C0h, solid) stops her at 00AD.FFFF; with
`0A52` ≠ 0 the knockback's end (`$91:F31D`) clears it, sets the super special pose `0A2C` = 52h
(her own), which wins over the landing's `0A28` = A5h (`$91:EB88`), and leaves `0B36` = 2 at
magnitude 0: falling at 0, not landed (401: 0.1C00, kept on the floor; 402 lands in E5h, pushed
up 2 px). The model lands her in A5h (half-height 15h against 13h), pushes her up 2 px to
00AB.FFFF and reports `camera_from` 00AB.A3FF. Answer: 400 is out of scope (knockback, spec §10);
395-399 are hers.

Who writes what on the tail (adversary 61, F1). Before its last frame nothing in the normal path
reads the knockback: in bank `$90` only `$90:8EDF` (called by the knockback's handlers `$DF53`
and `$DF64` and by the bomb jump), `$90:91FB` (morphed bouncing), `$90:95B7` (the horizontal
scroll), `$90:DDE9` and `$90:DF38` read `0A52`. `$91:ED4E`, the knockback's start, leaves the pose
input handler alone, and the handler hands over to the normal one, `$A337`. So the tail's frames
run her own motion: x by the normal horizontal move (`$90:8EA9`), y by `$90:90E2` with `0B36` = 0,
the moonfall's path (no fall limit), and her pose by the pad. The knockback's end is one frame.
`18AA` is counted down once a frame at the frame's very end (`$A0:9169`, called last in the
gameplay loop at `$82:8BB3`, after the new-state handler `$90:E722` and the scroll), to no lower
than 0. So a record whose `18AA` is 0 starts a frame whose hit check `$90:DDE9`, run after her
motion, takes its timer-zero branch (`$DE20`): with `0A52` ≠ 0 and a movement type other than
knockback it sets the super special pose to her pose (or, after a transition animation's end,
`0A32` = 8) and the command "knockback finished". `$91:F31D` then clears `0A52`, sets the normal
handler, zeroes the bounce and her y speed, sets falling with `0B36` = down, aligns her bottom
with the old pose (`$90:EC7E`) and keeps her pose: falling at 0, whatever the motion did, landing
or not. The other writers of `18AA` set it at a hit (`$A0:986D`, `$A0:992C`, `$A0:A57E`: 5;
`$94:8EA2`, `$94:8EDD`, `$94:AAD2`, `$94:AB4B`: Ah; `$94:9879`: 10) or clear it (`$90:D66F`,
`$90:D6A4`, `$90:D6D9`, `$90:D78F`, `$90:DDFC`, `$91:8128`, `$91:E150`, `$A0:A549`, `$82:DCD8`).

The rule, one predicate for both harnesses (filed for the integrator as gap 33): samus-motion's
harness defers the knockback's end as it defers its handler, every frame that starts with
`0A52` ≠ 0 and `18AA` = 0 under the normal handler (here the record ending 400; 394 starts in 4Fh,
outside the domain, and 395-399 are compared). The camera's rule (adversary 42, M4): its harness
masks y on the frames samus-motion's harness defers as the knockback's end, by that same
predicate, where her pose changes or samus-motion's step places her; x stays compared, since the
lead's turn during a knockback is the camera's own `LeadingHer`. `rising` on the tail is
samus-motion's (`RisingAndFalling`: every tail frame matches the records); at the end, and during
the knockback itself, it is the knockback (damage) domain's output (spec §10): the records show
`0B36` = 1 at every knockback start (pose 54h; for example chain-0059-A293 65, 139, 246, 585),
which samus-motion's harness defers.

## Round 13's amendments (2026-11-09)

**Jump held through (gap 32).** The scenes engineer's run (`oracle corpus --only
samus-screw-attack --only chain-0059-A293 --watch-words 0AF4,0A1C,008B --watch-pcs
90EAB3,90E926,90E93A,908370`, on the lsnes lock) read `0AF4` at each cut. This paragraph numbers
by record: record N holds the state at frame N's start and frame N's pad, so a press is at the
same number in the vectors and the gaps, which number a frame by its start, and anything a frame
does is one higher here than there (the turns end on frames 98 and 105, shown at records 99 and
106). The run: the handler at the
landing's next frame reads 2 (samus-screw-attack 175, 607), 8 (chain-0059-A293 175) and 1
(chain-0059-A293 108, after firings at 100 and 107 that `$90:8370` installed at the turns' ends,
99 and 106). The vectors' frames were read with `mdump.sh` from the more-caches cache: the press
is in record N's pad (`8F` bit 0080h at 171, 603, 165, 96), so the count is 0 at N+1. JH4's run
press (8000h at 104, so on lead-in 9) is listed. From the press alone the count at the handler of
record 108 would be 10 (107 - 97), not the 1 the turns' restarts leave. The window is signed
(`$90:E92E` `CMP #$0009` / `BPL`), so counts 8009h-FFFFh also fire, and `$90:EAB3`'s `INC` wraps;
the spec states it as original only, and LEGACY lists it. Each landing's floor is her y plus 21 at the landing record (B0h,
B0h, B0h, C0h). C-held-9 is JH3 started at 164 (`0AF4` would be 9 at 174), its after the
round-12 model's, which has no jump held through, worked against the landing's rows: left held,
forward, `Running`, x 1 px left.

**The placement as an option (adversary 56, finding 4; the camera's LEGACY entry "A placement
of 0 px after another mover").** The writers of `0B14` the step owns, each read in bank `$91`:
`$91:FF42` (pushed down from above), `$91:FF6F` (pushed up from below), `$91:FF93` and
`$91:FFA2` (a solid enemy's push), `$91:FFE9` (the crouch taken instead, only when the old body
is shorter: `$91:FFD9` skips the write otherwise), and `$91:ED0E` (above: the transition poses,
after `$91:F404` returns carry clear, so not after a refusal or the crouch taken instead, which
change `0A1C`). `$91:FF76`/`$91:FF87` (nothing in reach) write nothing. So "no placement" is
exactly the frames none of these runs, and a placement of 0 px is `$91:ED0E` for 3Bh-3Eh with
nothing pushing: unmorphing in open air (C-place-0, worked by hand: moves 0 at a fall's speed 0,
the 9 px looks unstopped, so `0AFA` and `0B14` stay 0100). Standing up aimed (F7h-FCh) has no
write of its own; on a floor pose fit's look down is stopped at once, so `$91:FF6F` pushes her
up or the change is refused, and the spec says so as original only. The rename from
`camera_from` to `placed` makes "none" a value: W11 and C-fit-step-enemy now say it.

**WK1's input.** The kick frame needs "spin turned far" (§1.3), which the vector format had no
key for, so the round-12 vector test ran it with the input false and the frame read `Readied`.
The format now gives `spin_turned_far` per frame; WK1 sets it on its checked frame (712, the
frame after 711's readying), and the round-12 model passes it.

## Round 14's amendments (2026-11-10)

From adversary 61 §9 (V1-V5) and adversary 67's finding 4.

**V1, jump held through built and read back (spec §1.2).** The two parts are `0AF4` (the count)
and `0A60` = `$E926` (the frame before: installed by `$91:F1EC` and `$90:8370`, reset at
`$90:E944`). Nothing in the original ties either to the rest of her state: `$90:EAB3` counts with
`8B` and `0DFE` alone, and `$90:E926` reads `0AF4` alone, whatever the pose. So any count and
either value are a start a caller may give. The format gives them as `before.jump_held` and
`before.landed_or_ended`, read back as `after.jump_held` and `after.landed_or_ended`. The
constructed vectors, worked by hand: C-held-built (count 5, `$E926` installed, jump held: `$E926`
ORs jump into `8F`, the standing row takes off, `$90:E93A` zeroes `0AF4`, and `$90:EAB3` at the
frame's end, jump in `8B` and in `0DFE`, makes it 1; `0A60` is reset to `$E913`); C-held-long
(40000 = 9C40h: `CMP #9` leaves 9C37h, negative, so `BPL` doesn't skip: a press); C-held-edge
(32776 = 8008h: `CMP #9` leaves 7FFFh, positive, no press; `INC` to 32777); C-held-wrap (FFFFh,
`$E913` installed: `INC` wraps to 0); C-held-landed (PF2's frame, r02:179, with jump held at 3:
the landing installs `$E926` through `$91:F1EC`, the count goes on to 4). Those that compare the
read-back fail on the current model only because it has no reading back (see below).

**V2, the crouch taken instead at the crouch's height.** C-fit-tunnel-aimed: crouched aimed up
(85h, y radius 10h, `$91:B629` byte 6) in a tunnel with jump pressed; the take-off (4Bh, 13h) finds
no room, so `$91:FFD9` takes the crouch 27h and skips `$91:FFE9`'s write, the old radius being
the crouch's own: no rise and no `0B14` write, though the pose changes from 85h to 27h.

**V3, jump let go on the landing's next frame.** `$90:E926` tests `0AF4` only (`BEQ`, `CMP #9`,
`BPL`), never `8B`, so with the count 1-8 the press comes whether jump is held or not. `0AF4` is
counted at `$90:EB42`, the draw, after the pose input, so the frame's own let-go reaches the count
only at its end (`STZ` at `$90:EACA`). C-held-let-go is JH3 with the checked frame's pad left alone:
the landing's row "jump pressed" names no held button, so it takes off as JH3 does.

**V4, the knockback's tail.** Already narrowed on main: spec §10 "A knockback's end" and gap 33
defer only the frame that ends it (C3 above); nothing to change.

**V5, an upward motion on the ground outside a push.** Yes. Over the 44-row cache, `0B36` = 1 in
a movement type on the ground (0, 1, 4, 5, Eh, Fh, 10h, 11h, 15h) holds on 26 frames under the
normal handler `$A337`, beside the bomb jump's 27 (`$E032`), and under no other handler. 24 are in
the transition type Fh while morphing in a rise (chain-0043-9AD9 294-299, item-reserve-tank
101-106) or unmorphing in one (item-charge-beam 1-6, item-reserve-tank 514-519), where her rise
goes on (`$90:A61C` moves her by it; jump let go doesn't end it, chain-0043-9AD9 295-299 with
`8B` = 0100h). 2 are in the crouch: item-charge-beam 7 and item-reserve-tank 520 (a record's WRAM is read where its length is followed, 128 KiB on, by the
VRAM's: the length's bytes also occur at byte 16 of some headers). At
item-reserve-tank 520 → 521 the crouch rising at 2.2400 is pulled 1.0000 from 005B.EFFF to
005C.EFFF, keeps the speed, and falls in 2Ah: C-ground-rise exactly. At item-charge-beam 7 the unmorph's end hands over the crouch 28h in the air (instruction Dh,
above), still rising at 2.4000; the crouch's handler `$90:A573` pulls her 2.C000 (`$90:923F`),
finds no floor, and `$91:EFEF` keeps the speed and the direction because `0B36` = 1; 8 is the fall
2Ah rising at 2.4000, and the rise runs under the pull to 16. A recorded vector for the rise
during a change of shape (chain-0043-9AD9 297, y, the speed and `rising` alone) isn't added: the
format compares y only with x, and her x there, still against a horizontal speed of 0.C000 during
the morph, rests on a wall the record doesn't show. So `rising` is `0B36` = 1 at the
frame's end on the ground too, and the spec now says so (`RisingAndFalling`, `FollowingTheGround`,
`AnimationEnds`, `WhatHerMovesMet`'s crouched row). terrain-quicksand 196 is the push's, as
adversary 61 found. The constructed vectors C-ground-rise-kept and C-ground-rise, worked by hand:
crouched rising at 2.4000, pulled 1.0000 (speed 0); on a floor the pull is stopped flush, the
speed kept; in open air it moves 1.0000 and walks off into 29h, the speed kept. C-unmorph-air-end:
the ball falling at 1.6C00 presses up (3Dh); that frame moves 1.6C00 and the pull makes 1.8800;
the next frame `$90:A61C` moves 1.8800 and makes 1.A400, and the sequence's end hands over 27h.

**The items switched on (adversary 67, finding 4).** Decided: yes. Her motion reads progress's
own output, the items switched on, and `clean_types::Equipment` and the bridge's conversion go.
Why: the original reads one word, `09A2`, the equipped items, which is exactly progress's items
switched on; a second representation (`Equipment` with its `other_suit` rule) is a conversion the
original doesn't have, and it hid an item motion reads, the morph ball. The items it reads, from
the readers of `09A2` in banks `$90`/`$91`: hi-jump `$90:98F7`, `$90:9984` (the jump speeds); space
jump `$90:A46E`; morph ball `$91:F7CE`; spring ball `$91:E83D`, `$91:E86A`, `$91:F80F`..`$91:F878`,
instruction 9's mask; screw attack `$90:8445`, `$91:D9F7`, `$91:DA0B`, `$91:F459` (sprite and
palette); varia `$91:D629`, gravity `$91:D551`, `$91:DE6A`.. (palette) and `$90:8198`, `$90:9741`,
`$90:98C2`, `$90:9BD4`, `$90:9C24`, `$90:9C5B`, `$90:E9D9` (liquids, out of scope); speed booster
`$90:9789`, `$90:84FF`, `$90:8558`, `$91:E644`, and `$90:9919`, `$90:99A6` (every jump and wall jump
starts half the dash faster: the booster's higher jump, out of scope); also gravity in liquids
`$90:994F`, `$90:99DC`, `$90:9A2F`, `$91:F68A`, `$91:F6EB`, the screw attack's spin pose and sprite
`$91:F6B2`, `$91:F713`, `$91:E7A8`, `$91:E7B7`, `$91:E894`, and varia `$91:E741`, `$91:E757`. Bombs,
grapple and x-ray aren't read in her motion.

**The recorded vectors' items (adversary 69, finding 2).** Each recorded vector lists every item
`09A2` holds at the record of its first frame's start (equal, in every one, to its checked
frame's start), the items §1.3 says change nothing it compares included. All 158 were read from
cache-v2 (r01-r08 are its chain rows); 105 changed, 103 of them in an item motion reads
(`MorphBall` 103, `HiJumpBoots` 63, `SpringBall` 44, `SpaceJump` 31). The round-13 model under
the local reading below gives the same 10 failures as before (it has no morph ball input yet). The format's `items` lists them by progress's `Item` names. The morph ball's
rule is new to the spec (`PoseFollows`): NM1 is r02:490 (record 489, crouched 28h, `8F` 0600h,
`09A2` 0; record 490 the same pose, x and y, `0A2E` 7), and C-morph-no-ball-air is SH3 without the
morph ball, worked by hand: its moves as SH3's, then the pose restored, so y 0094.B7FF, the rise
ended at the frame's start.

**Room content: harness cases (MODEL §1 decision 13).** Six recorded vectors carried the ROM's
content; they leave the vector file and become harness cases for the integrator's harness,
which loads the room through the importer and compares the frame:

| Case | Row and record (N+1) | Content the importer loads | Compared | Constructed vector |
| --- | --- | --- | --- | --- |
| S1 | samus-input-poses 512 | the sloped floor at block (96, 192): its shape and walk factor | x, y, horizontal speed, dash | C-slope-turn-walk |
| S2 | r01 717 | the sloped floor at block (832, 1200) | x, y, both speeds | C-slope-fall-walk |
| SL1 | samus-mockball 92 | the sloped floors at blocks (624, 1216) and (640, 1216) | x, y, both speeds, dash | C-slope-roll-carry |
| SL2 | samus-mockball 93 | the sloped floor at block (640, 1216) | x, y, both speeds, dash | C-slope-roll-carry-next |
| SL3 | chain-0220-D017 44 | the sloped floors at blocks (224, 144) and (240, 144) | x, y, both speeds, dash, the pose | C-slope-run-off |
| W15 | item-morph-ball 817 | the cells at blocks (1104, 656), the item, and (1104, 672) | x, y, vertical speed, the near-wall test's outcome, the cells reached | C-near-wall-item |

Each case starts from the record as the vector did (the decode of State above; W15 with its two
lead-in frames, 815 and 816), and its expected values are the record's at N+1, which the removed
vectors gave. The constructed vectors keep each case's motion and invent the room: new
positions, new surface shapes (none of them a shape of the ROM's) and other factors from the
spec's list (0.B0, 0.D8), and, for W15, the scene moved by 256 px right and 128 px up with a
reacting-air cell. Each was worked by hand from the spec's arithmetic (their comments give the
rows and columns), and the current model agrees with each. The recorded vectors that stay give
only RAM: a floor, wall or block a record shows her stopped flush against, and solid enemies'
centres and radii from enemy RAM (`0F7A`..`0F84`).

**The current model against the amended vectors.** 248 vectors (236 − 6 + 18) and 51 pose steps.
With the vector test unchanged, the first vector with `items` (JH1) stops the run: every vector
fails the format until the round's test reads it. With a local reading of the new keys (`items`
to the old flags, the jump-held keys accepted and their read-back reported missing), 238 pass and
10 fail:
- the format only, the read-back the model has no way to give: C-held-edge, C-held-wrap,
  C-held-landed;
- new behaviour: C-held-built and C-held-long (no built count: no take-off, and no read-back),
  C-ground-rise-kept and C-ground-rise (an upward speed on the ground), C-unmorph-air-end (the
  crouch, not the fall), NM1 and C-morph-no-ball-air (the morph ball).

The vectors whose input changed (JH1, JH2, SJ1, C-moonfall-jump; SH2, SH3 and C-shrink-floor,
which now switch the morph ball on) pass under that reading, and so do C-fit-tunnel-aimed,
C-held-let-go, the five slope vectors and C-near-wall-item.

**Units.** `tools spec-units` with these notes: 209.0 → 213.0 (motion 77.0 → 79.0, pose chart
56.0 → 58.0, pose fit and collision unchanged), so the budget is 7,092 → 7,133: motion's
36 × 79.0 = 2,844 meets its cap, 2,813 (70% of 4,019), and the pose chart stays capped at 1,584.

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

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here, and `tools report` the areas' old
figures and the commit.

Size is counted in logical tokens: each identifier, keyword, literal and operator is one, and
so is each bracket pair; comments, layout and tests are free. Two kinds of token are counted
apart and reported, not held against the target: the values of constants and statics (tables of
the game's data, such as the chart's rows), and declarations (the `struct`, `enum` and `type`
items and attributes that define the model's shapes; define as many as the behaviour needs). The
target is on everything else, the logic.

**Budget (MODEL §1 decision 7).** k = 36 weighted tokens per spec unit, frozen: the old code's
score, 12,723 (`tools size` at spine `bc6f74f9c`, over the old code at `c7f927ab6` and its two
cuts), per unit of this spec's official count, 193.3 (`tools spec-units` at `d9ee05f7c`), times
0.55 is 36.2. An area's budget is k × its units, capped at 70% of its old code (`old`, the score
of the old files that area replaces, counted the same way at `c7f927ab6`). The budget is an
indicator: the fit is score ÷ budget, and being over it costs in proportion to how far over, never
as a wall; over an area's old code is a failure.

- Area motion (old 4,019): SpeedBuildsAndBrakes, Dash, TurningRound, AimingAndShooting,
  RidingTheElevator, Jumping, FollowingTheGround, MovesAsShePosed, RisingAndFalling,
  DriftInTheAir, Moonfall, WallJump, Landing, WordwiseTopSpeed, 1.2, 9
- Area pose fit (old 1,549): PoseFitAfterMoves, ShrinkingLowersHer
- Area pose chart (old 2,263): HowSheIsDrawn, PoseFollows, PadChart, Aiming, WhatHerMovesMet,
  AnimationEnds
- Area collision (old 4,893): BlocksStopFlush, WallsAndCeilings, SolidEnemies, SlopesCarry,
  SlopesSlowTheWalk, TouchedCells, OtherBlockKinds

Budget: 36 × 213.0 units, capped at 70% of old = 7,133 (motion 2,813 capped, pose fit 588, pose chart 1,584 capped, collision 2,148)

From the spec's §7 (the pose chart), moved with the header's:

**Size.** Inside the header's 1,500 tokens of logic for the chart: the chart's reader and the
effects. The tables below are data records (about 120 rows), counted as data.
