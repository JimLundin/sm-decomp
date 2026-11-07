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
turns) are unreachable from the moonwalk rows (4Ah's unaimed jump row comes first). The ground
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
  animation the next frame); C-chart-i, C-chart-j.
- The straight jump aiming down facing right (17h) has hold right + jump + shoot → 13h after the
  diagonal rows; 18h has a second left + jump + aim-down row in its place. Spec row "aiming `Down`,
  facing right: forward, jump and shoot held"; C-chart-k, C-chart-l.
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
| Moonfall's vector | M1 r03:12, lead-in r03:10-11 (turn pose C0, mt Eh, into pose 19 with `0B36` = 0); C-moonfall-jump and C-moonfall-nolimit start from r03:10 and match r03's `0AFA`.`0AFC` and `0B2E`.`0B2C` frame by frame to 58 (checked by script); their x is the rule's, r03's own x meets the room at 50-51 | `$91:F36E`, `$90:913A` |
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
| PoseFitAfterMoves | `$90:E725` (movement `$90:E94B`, then `$91:E8B6`, then `$91:EB88`), `$91:F404` (`$91:FDAE` pose fit, then `$91:F433` initialise, then `$91:FBBB` the jump transition with `$91:FC8A` the 10 px lift) | `$91:EADE` (the aim step) runs before `$91:F404` |

Recorded vectors: S1 is samus-input-poses record 511 → 512 (block at column 6, row 12, BTS 54h:
14h x-flipped); S2 is r01 record 716 → 717 (BTS 12h, first frame of a fall, `0B2E`.`0B2C` = 0).
The surfaces are `$94:8B2B` rows 14h (flipped) and 12h; 10h in that table is a column with no solid
pixel, written 16 in the vectors.

**Turning with only run, shoot or jump held (root's question, 2026-11-07).** `$91:81A9` runs the
lookup failure `$91:82D9` (command 2, which ends the skid) when `8B` = 0, or when the pose's
transition table is non-empty and nothing in it matches. The ground turns' tables are non-empty
(25h at `$91:ACF4`, C0h at `$91:AFAC`); the air turns' (2Fh, 30h, 87h, 88h, 8Fh, 93h, ...) are
empty, so there a held button returns without a command. Gaps v4's reading of R141 holds for the
air only. On record: samus-input-poses 119 → 120 (pose C0h, `8B` = 8000, `0B4A` 1 → 0 at base
speed 0), then 120 → 121 moves 0.C000 (R11, R12).
