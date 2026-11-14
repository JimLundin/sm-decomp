# Samus moving through the world: decode notes

For the observer, the adversary and the domain harness's decoder only. Implementers never see this
file (MODEL §5). It holds every source the spec draws on: the original's routines, its RAM and its
codes. Addresses are the bank logs' (`$TEMP/c4a/logs*`, https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 32 rows in `tools/oracle/movement_rows.txt`, cached in
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
| r09 | chain-0314-DAE1 |
| r10 | chain-any-0003-DFD7 |

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
| dashing | `0B3C` | ≠ 0 (the running momentum flag: written 1 by `$90:9796`/`$90:97CC`, cleared by `$91:DE53`); written back as 1 or 0 |
| pose | `0A1C`, `0A1E` | body and action from the movement type `0A1F` (below); facing `0A1E` low byte 4 Left, 8 Right, 0 Screen |

What she remembers (spec §1.2), for the decoder and the adversary only: carrying on and skidding
are `0B4A` (0 follows the pad, 2 carries on, 1 skids; a skid of zero is `0B4A` = 1 with `0B46` =
0); a dash to lose is `0B3C` = 0 with `0B42` ≠ 0; the moonfall is `0B36` = 0 in the air; a spin
carried on is `0A27` ∈ {3, 14h}. None is in the spec's State but dashing (`0B3C`, the table above)
and jump held through. The horizontal speed's sign is exact
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
| Dash | `$90:973E`, `$90:97D5` (the cap), `$90:9789` (the booster's cap, `$90:9F0D`), `$91:DE53` (losing it), `$90:E5CE` (the wall stop's share) |
| Dashing | `0B3C`: set in the running handler's speed `$90:9789`..`$90:97D2` (movement type 1, run held, not submerged: `$90:976E`); cleared by `$91:DE53` from the movement handlers, before the animation (`$90:A3D0` standing, not 00h or 9Bh; `$90:A551` the ball 1Dh/41h following the pad with `0B36` = 0; `$90:A5D9` the falling ball; `$90:A685` the ground turn; `$90:A6CF`, `$90:A712` the spring ball's; `$90:A768` against the wall; `$90:A79E`, `$90:A7BB` the turns in a jump and a fall; `$90:E5E9` the wall stop), and from the pose change, after it (`$91:EC76` command 1, `$91:ECD3` command 2, `$91:EC8E` command 8 and 6 falling into it, set by `$91:8304` from `$91:8332` on a failed lookup; `$91:F66F` the spin reversed; `$91:FA45` the ball turning); `$90:A57C` (crouched: the dash 0, the flag kept); `$91:F8D3`, `$91:F952`, `$91:F98A` (a turn's start joins the dash, the flag kept); `$91:EB88` (`$91:EBC3`: a super special or special pose skips the command), `$91:E8B6` (a landing writes its own pose and command 5 over the pad's) |
| TurningRound | `$91:F543` (turn start and spin reverse), `$90:A68F` (turning handler, its sink and its end), `$90:A67C` (the creep), `$91:81A9` with `$91:82D9`, `$91:8304` and `$91:ECD0` (letting go: the failed transition lookup's stop command clears `0B4A` after the move) |
| AimingAndShooting | `$91:EADE` (the step: any prospective pose `0A28` of the running movement type, her own included), `$91:82E4` (the failed lookup's decelerate command keeps her pose as the prospective pose, so a running pose braking steps every frame), `$91:81E4` (a found row naming her own pose sets no prospective pose: no step), `$94:971E` (its block stop), `$90:A3AD` (the shot) |
| RidingTheElevator* | `$90:A392`, `$90:A397` (two branches of the facing-forward handler); the body: `$90:EC22` (the radius from the frame's pose) runs under the normal state handler, not the ride's `$90:E713` that `$91:E3F6` installs |
| Jumping | `$90:98BC` (the speed table; `$90:9919` and `$90:99A6`, the booster's share of the dash), `$91:FBBB` (take-off), `$91:F624` and `$91:F8D3` (jump starts), `$90:A488` (space jump); jump held through: the count `0AF4` (`$90:EAB3`, from `$90:EB35`'s draw once a gameplay frame: `INC` with jump held now and in `0DFE`, else `STZ`), the one-frame pose input handler `$90:E926` (count 1 to 8: jump ORed into `8F` for `$91:8000` alone, `STZ 0AF4` at `$90:E93A`), installed by `$91:F1EC` (landed, downward result grounded: `$90:E676` gives it for movement types 2, 3, 6 and 14h, and Ah and 19h out of scope) and by animation delay instruction 8 `$90:8370` (the turns, A4h-A7h and E0h-E7h, BFh-C4h; not when `0A28` is 4Bh, 4Ch, 19h or 1Ah) |
| FollowingTheGround | `$90:923F` (the pull), `$90:E61B` (walking off); the kept speed: `$90:923F` again (the ground handlers' only vertical move reads and writes no speed), `$90:E676` (landed in a ground movement type: no change, so `$91:F010` doesn't run), `$90:A61C` with `$90:92C7` (the change of shape moves by it when `0B36` ≠ 0; `$90:A640` clears it on landing); the ball on the ground: `$90:A521` (morph ball) and `$90:A69F` (spring ball) call `$90:92B8`, the clone of `$92C7`, on both branches (`A549`/`A569`, `A6C7`/`A6E7`): with `0B36` ≠ 0 it runs `$90:90C4` and `$90:90E2` (moves by the speed, then the pull grows it) and returns carry set, which skips `$90:923F` (no ground pull) and, at rest, the halt at `A551`-`A561`; `$90:E676` gives 04 for movement types 4 and 11h, so `$91:E931` returns and the floor doesn't land it (BG1), `$91:EFEF` (walking off: speed 0, unless `0B36` = 1: `$91:EFEF` then keeps the speed and the direction, item-charge-beam 7 → 8 keeps 2.4000 up), `$90:98BC` (a jump sets it) |
| MovesAsShePosed | `$90:A337` (the handler of the frame-start pose), `$91:EB88` (the pose taken after) |
| RisingAndFalling | `heading down` is `0B36` = 2 as the animation reads it (`$90:8032`, the rising hold), after the movement handler and before the pose change: a ceiling's down (`$91:E8E5` → `$91:EFC4` → `$91:EFDF`), a landing's none, a walk off's down and a bounce's up come at the pose change (0 in the moonfall, `$90:913A`; a speed kept on the ground keeps its direction, `$90:923F`); `$90:90E2` and `$90:913A` (gravity), `$90:910C` (the limit), `$90:8FDC`, `$90:9046` and `$90:90C4` (the rise's end: `0B2E` read as signed, ends when negative), `$91:F22B` (bounce 2 at 0.0); `rising` is `0B36` = 1 at the frame's end, which the ground handlers keep as they keep the speed (`$90:923F` writes neither), so a crouch the unmorph's end hands over in a rise holds it (item-charge-beam 7) |
| DriftInTheAir | `$90:900D`, `$90:9174`, `$90:91A8` (normal jump and fall), `$90:906C` and `$90:9093` (spin; its stop still calls `$90:8EA9`, which adds the extra run speed) |
| Moonfall | `$91:EBAF`, `$91:F36E` (the hand-over), `$90:913A` (no limit with direction 0), `$90:9D35` (the wall-jump test reads no direction), `$90:99CD` (the kick sets direction up); the turn in it: `$90:A790` and `$90:A7AD` (turning in a jump, in a fall) call `$90:92B8`, which with `0B36` = 0 returns carry clear, so `$90:923F` (the ground pull) runs and `0B2C`/`0B2E` are not touched; `$90:E65A`/`$90:E676` give 04 (no change) for movement types 17h and 18h |
| WallJump | `$90:9D35` (arming and kick test), `$90:90B3` (one distance, `$90:9E9F` = 8, for both), `$91:FA0F` (kick), `$94:967F` (the probe) |
| Landing | `$91:F010`, `$90:A640` (upright and changing), `$91:F1FC`, `$91:F25E` (the ball): by `0B20`, `$91:F20A` (not bouncing: `0B2E` `CMP #3`, read after `$90:90E2` grew it, since `$90:90E2` copies the speed to `$12.$14`, grows `0B2C`/`0B2E`, then moves by the copy), `$91:F22B` (first bounce: any speed, rises at `$90:9EB5` − 1 = 0.0), `$91:F245` (second bounce: rests); spring ball `$91:F27F`, `$91:F2A3`, `$91:F2C0` alike |
| TravelledDistances | `$90:9373`..`$90:946D` (ordinary move results), `$90:9826`..`$90:98BB` (enemy-shortened moves), `$94:971E`..`$94:97BE` (signed block-adjusted travel), `$90:EB02`..`$90:EB1A` (interval clear); `$90:B218`..`$90:B2F5` (W1 inheritance) |
| BlocksStopFlush | `$94:9722`, `$94:976B` (a 0.0 move skips block collision), `$94:8F49`, `$94:8F82` (horizontal and vertical block collision), `$94:8D2B`, `$94:8DBD` (stepped), `$94:9543`, `$94:959E`, `$94:95F5` (the order), `$90:E4AD` (the limit) |
| WallsAndCeilings | `$90:E5CE` (wall), `$90:E606` and `$91:EFDF` (ceiling) |
| SolidEnemies | `$A0:AAC8`, `$A0:AA53`, `$A0:AABC` (overlapped: next enemy, then 0 so blocks run), `$90:9D80`, `$90:9DF8`, `$90:93B1`, `$90:93EC` and `$90:9440` (enemy test first; on a hit the move skips block collision, as the bank log's zebetite-skip note says) |
| SlopesCarry | `$90:9376`, `$94:87F4` (seat after every horizontal move, sets `0DBA`), `$90:924C` (the pull reads `0DBA`), `$94:976B` (a 0.0 vertical move skips block collision), `$94:976F` (cleared by a non-zero vertical move), `$94:8E38` (square slope stop sets it), `$94:86FE` (rising into a non-square ceiling slope: `$12 = max(0, ...)` and `$14 = 0`, so no move), `$94:87E4` (the floor slope stop is `max(0, ...)` too: never upward); the carry: `$94:95A1` (`$18` = y + `$12.$14` with the subpixel carry, so the target row includes it) and `$94:879E`..`$94:87EE` (`$12 = max(0, $12 + top − 1 − ($18 + 0B00 − 1) % 16)`, `$14 = 0`: the move is the uncarried pixel part corrected by the carried row); `$94:8790`/`$94:8717` (only the block at her centre column), `$94:87B9`/`$94:8742` (a ceiling never stops a downward move, a floor never a rise) |
| WordwiseTopSpeed* | `$90:9A99` |
| PoseFollows | `$91:EB88` (one change a frame: super special `0A2C`, then special `0A2A`, then `0A28`), `$91:E8B6` (the vertical collision result `0DC6` overwrites the pad's `0A28` and sets command 5), `$91:EADE` (the wall stop overrides both); the spec's three items fold the wall stop into `WhatHerMovesMet` and the lookup failure into the chart's last rows, `$91:EC85` falling into `$91:EC8E` (the dash goes with the speed); the morph ball: `$91:F404` runs `$91:FDAE` then the pose's initialisation, and for the morphing transition 37h/38h `$91:F7CE` tests `09A2` bit 0004: without it `$91:F7EC` puts the old pose back, so `$91:F404` returns carry set (the pose changed under it) and `$91:EBEE` skips the lowering and `$91:ED0E`'s write: no lowering, no placement, no fall-back row (r02:489, pose 28h with 0600h pressed and `09A2` = 0, stays 28h at y 00B0) |
| PadChart | `$91:81A9` (a row fits when its newly-pressed and held masks are all set; first fit wins; same pose: no change), `$91:81F4` (bindings to default bits), the transition table `$91:9EE2`..`$91:B00F`, `$91:F404` (crouched into standing: the bottom edge kept, y − 5; `$91:8087`'s own y − 5 is dead); the last rows: `$91:81DB` (pad 0 or no fit, unless the pose's table is empty), `$91:82D9`, `$91:8304` and `$91:8332` (the effect per movement type), the pose definitions `$91:B629` byte 2 (the pose with no buttons), `$91:EC50` (decelerate), `$91:ECD0` (stop), `$91:EC85`, `$91:EC8E`; `$91:8146` (movement type Fh: no lookup at all, the changing-body row) |
| Aiming | the table's aim rows per pose, `$91:E9F3` (landing keeps the aim, down becomes forward), `$91:EB74` (the wall stop keeps the aim) |
| WhatHerMovesMet | `$91:E8F2` (falling poses, `$91:E921`), `$91:E95D`..`$91:EA47` (landing poses, ball bounces), `$91:EABE` (wall-jump pose), `$91:EADE` (ran-into-wall poses, the aim step's block); walking off from the crouch: `$90:E65A` gives 00 (airborne) for movement type 5 as for 0, 1, 10h and 15h, so `$91:E921` hands over 29h/2Ah, upright falling (item-charge-beam 7 → 8: 28h → 2Ah) |
| AnimationEnds | the end unless a jump: instruction 8 `$90:8370` (`$90:8378`..`$90:838D`: `0A28` ∈ {4Bh, 4Ch, 19h, 1Ah} returns with no request; in the demos, `0A60` = E91Dh, it never does); `handed over` is `0A32` = 3 with `0A2C` bit 15 clear; the animation delay table `$91:B010` (its go-to-pose instructions), `$91:EB88` (the super special pose wins); the unmorph's end: 3Dh/3Eh's sequences `$91:B4E6`/`$91:B4EA` end in instruction Dh (`$90:84B6`) to 27h/28h, the crouch, with no test of the air (records: boss-kraid-defeated 779-782, item-charge-beam 5-7, item-reserve-tank 1115-1116); the morph's end: instruction 9 (`$90:839A`) chooses the ground ball when the speed magnitude `0B2E`.`0B2C` is 0 and the air ball otherwise (spec question 20) |

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

The domain harness (`verify::harness::movement`) decodes the previous spec's types; its frame
skips (game state `0998`, lag, the elevator `0E18`, pose fit, handlers `0A44` and `0A58`, the
deferred behaviours) and the bridge's answers are in git history at `1dc3a12`
(`docs/behaviours/samus-movement.decode.md`). They carry over once the implementer's new types exist.

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

Gaps 20 and 24-28 of `samus-movement.gaps.md`, from the round-8 harness. The records are read with
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

**Gap 31, the speed booster on a sloped floor (a harness mask until round 17, when the model took the booster and the mask went).** With `0B42` ≥ 2.0 running with
run held, `$90:9789` grows the dash past the model's cap, so x is masked already; on a slope the
surface row at her centre column (`$94:95A1`, `$94:87DB`) and the pull `$90:923F` (`0DBC`.`0DBE` +
1) both follow it, so y is the booster's there too. A local run with `dash_cap` 10.0000 and x
unmasked: y matches on every masked frame except shinespark 110, 118, 142 and 146 (the same on all
four rows), where x is a pixel ahead (03B4.0FFF against 03B3.FFFF at 110; the model's growth of
the dash isn't the booster's), so those differ by the booster too. The mask's test: a slope block
(type 1) within one block of her feet (her centre's column ± 1, her bottom's row ± 1); it takes
y on 24 frames of samus-mockball and 81 of each shinespark row. Shinespark 198 (left, up-left) and
185 (right, up) are outside it and still differ (y 1 px).

## The camera's asks (C1-C3 of world-camera.gaps.md, 2026-11-08)

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
speeds masked in samus-movement's harness). Answer: the placement rules are the spec's, the
positions and what the looks meet are the push's; the camera's harness masks y (`camera_from`) on
the frames samus-movement's harness defers as a block-inside push.

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

The rule, one predicate for both harnesses (filed for the integrator as gap 33): samus-movement's
harness defers the knockback's end as it defers its handler, every frame that starts with
`0A52` ≠ 0 and `18AA` = 0 under the normal handler (here the record ending 400; 394 starts in 4Fh,
outside the domain, and 395-399 are compared). The camera's rule (adversary 42, M4): its harness
masks y on the frames samus-movement's harness defers as the knockback's end, by that same
predicate, where her pose changes or samus-movement's step places her; x stays compared, since the
lead's turn during a knockback is the camera's own `LeadingHer`. `rising` on the tail is
samus-movement's (`RisingAndFalling`: every tail frame matches the records); at the end, and during
the knockback itself, it is the knockback (damage) domain's output (spec §10): the records show
`0B36` = 1 at every knockback start (pose 54h; for example chain-0059-A293 65, 139, 246, 585),
which samus-movement's harness defers.

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
own output, the items switched on, and `types::Equipment` and the bridge's conversion go.
Why: the original reads one word, `09A2`, the equipped items, which is exactly progress's items
switched on; a second representation (`Equipment` with its `other_suit` rule) is a conversion the
original doesn't have, and it hid an item motion reads, the morph ball. The items it reads, from
the readers of `09A2` in banks `$90`/`$91`: hi-jump `$90:98F7`, `$90:9984` (the jump speeds); space
jump `$90:A46E`; morph ball `$91:F7CE`; spring ball `$91:E83D`, `$91:E86A`, `$91:F80F`..`$91:F878`,
instruction 9's mask; screw attack `$90:8445`, `$91:D9F7`, `$91:DA0B`, `$91:F459` (sprite and
palette); varia `$91:D629`, gravity `$91:D551`, `$91:DE6A`.. (palette) and `$90:8198`, `$90:9741`,
`$90:98C2`, `$90:9BD4`, `$90:9C24`, `$90:9C5B`, `$90:E9D9` (liquids, out of scope); speed booster
`$90:9789`, `$90:84FF`, `$90:8558`, `$91:E644`, and `$90:9919`, `$90:99A6` (every jump and wall jump
starts half the dash faster: the booster's higher jump, `Jumping`); also gravity in liquids
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

## Round 15's amendments (2026-11-10)

From adversary 71's observer items (V1-V7, Q1), and adversary 73's O1 and O2. 258 vectors
(248 − 2 + 12) and 51 pose steps.

**V1, the wall jump with hi-jump.** `$90:9984` reads `09A2` bit 0100h for the kick's speed:
4.A000 plain, 5.8000 with the boots. Record: samus-walljump-16 47 → 48 (`09A2` F32F, 81h → 83h,
`0B2E`.`0B2C` 5.8000, `0B36` 1, y 044C.DFFF kept). C-walljump-hijump is WK1 (item-morph-ball 709-712,
`09A2` 0) with the boots: no lead-in frame reads them (a spin's rise, jump let go, no jump start),
so only the kick's speed changes, 4.A000 → 5.8000.

**V2, the morph's end with and without the spring ball.** Instruction 9 (`$90:839A`) picks the
ball by the speed magnitude (0 the ground ball, else the air ball) and by `09A2` bit 0002h (the
spring ball's poses 79h/7Ah, 7Dh/7Eh, else 1Dh/41h, 31h/32h). Records: item-reserve-tank 564 →
565 (37h → 1Dh, `09A2` 1004), r09 415 → 416 (38h → 7Ah, F32F, at rest, nothing moves) and r09
17 → 18 (38h → 7Eh, F32F, falling: y + 1.DC00, speed 1.F800). The pose steps carry no items, so
four constructed vectors carry the item instead of an `items` key on pose steps:
C-morph-end-ball and C-morph-end-spring (at rest on a floor: the pull's 1 px stopped flush) and
C-morph-end-ball-air and C-morph-end-spring-air. The air pair starts in a straight jump aimed
Down, so the morph is in the air by every reading: a vector starting in `Morphing` with a speed
is on the ground by spec §1.2, where the table's "on the ground" and the original's
magnitude test differ (question 20). Worked by hand: falling at 1.0000 the jump moves 1 px and
the pull makes 1.1C00; the morph lowers her 9 px (0101.0000 → 010A.0000, nothing in reach); the
morph's frame moves 1.1C00 and the pull makes 1.3800.

**V3, space jump's press without the item.** `$90:A46E` tests bit 0200h before the window;
without it the spin goes on to its movement (`$90:A4D7`, `$90:9040`) with no jump.
C-space-jump-no-item is SJ1 (samus-space-jump-chain 64) without it: the spin's row for forward
and jump held keeps the pose, and the fall goes on: y 0478.1FFF + 2.A000 = 047A.BFFF, speed
2.A000 + 0.1C00 = 2.BC00.

**V4, the long window's low edge.** 32777 = 8009h: `CMP #9` leaves 8000h, negative, so `BPL`
doesn't skip: a press. C-held-long-edge is C-held-long at 32777.

**V5, the unmorph's end in the air through the crouch's frame.** The end hands over 27h/28h
(movement type 5) with `0B36` and the speed kept; the crouch's handler `$90:A337` pulls her by
`$90:923F` and the ground's landed result is "no change" (`$90:E676`), so a floor holds her and
keeps the speed; no floor walks her off (`$90:E61B`, `$91:EFEF`: speed 0 unless `0B36` = 1).
Record: boss-kraid-defeated 779 → 780 (3Dh → 27h, y + 1.8800, speed 1.A400) and 780 → 782 (`05B6`
advances once over two records: 9F49, 9F4A, 9F4A; the crouch's frame walks off into 29h, y
0169.ABFF → 016A.ABFF, speed 0). C-unmorph-air-end-off is that frame after C-unmorph-air-end
(y 0102.F400 → 0103.F400). C-unmorph-air-end-floor moves the three frames to y 010D.4C00 over a
floor at 120h: the ball's 9 px looks reach row 011D, the unmorph's move ends at 0110.4000 (bottom
pixel 011F), and the crouch's 1 px is stopped flush at 0110.FFFF, the speed 1.A400 kept. A mutant
that drops the end's "on the ground" fails both.

**V6, R19, R22 and R26.** R22 (samus-walljump-22 32) and R26 (samus-walljump-28 32) are dropped:
the three rows start from the same state, so their frames are byte-identical in every compared
quantity. R19 stays.

**V7, the take-off at a vector's start.** It counts as in the air, and spec §1.2 now lists it,
with the turn in a jump or a fall that the vector test already treats so. The original: the
take-off poses (4Bh, 4Ch, 55h-5Ah) are movement type 2, the straight jump's, and `$91:FBBB` sets
`0B36` = 1 with the jump speed the frame she changes into one, so a take-off is always rising;
the selection for lead-ins (`lead.pl`, above) already excluded a take-off at magnitude 0 as no
plain start. J3 and W16 start in the take-off, rising at 4.E000.

**Q1, the landing crouched instead.** Confirmed, and one rule corrected. `$91:F404` returns carry
set when pose fit changed `0A1C` (`$91:FFA7` writes 27h/28h, or `$91:FE82` the old pose on a
refusal), and `$91:EBEE` then skips the landing's command 5 (`$91:EFC4` → `$91:F010`): no
`$91:F02A` (`0B4A`, `0B46`/`0B48`), no `$91:F1D3` (`0B2C`/`0B2E`, `0B36`, `0B20`), and no
`$91:F1EC`, so `0A60` is not set to `$E926`. `$91:EC10` clears `0DC6` either way. So she is 27h
(movement type 5, on the ground) with `0B36` = 2 and the speed her fall's pull left, 3.1C00: a
downward speed kept on the ground, which `$90:A337`'s pull (`$90:923F`) neither reads nor grows.
The spec said a landing pose fit refuses or crouches still leaves "landed the frame before"; it
doesn't, and `Jumping` now says so (C-fit-land-tunnel-held: the count 3 → 4, `landed_or_ended`
false). C-fit-land-tunnel-next starts where C-fit-land-tunnel ends (a start can't carry the
landing pose's body size through a lead-in, so it is given as the start): crouched, she moves by
her dash only and halts (x speed 0), the pull holds her at 0110.FFFF, and the speed stays 3.1C00.

**O1, the skid through a landing crouched instead (adversary 73).** The skipped `$91:F02A`
includes `$91:F02D STZ $0B4A`, so after a refused or crouched-instead landing `0B4A` keeps the
jump's value: 2 carries on (a spin's drift, C-fit-land-tunnel), 1 skids (a spin reversed early,
`$91:F624`). PoseFitAfterMoves' arm now names "whether she carries on or skids" with the speeds.
C-fit-land-tunnel-skid, worked by hand: spinning right at 0.C000, left pressed, the step reaches
1.6000 (the top speed: she carries on) and moves it, and the reversal leaves her skidding (`0B4A`
1); falling from 0111.0000 at 2.0000 to 0113.0000 (2.1C00). The landing's frame: the skid brakes
in the air, 1.6000 − 0.8000 = 0.E000, moved right (0101.6000 → 0102.4000); the fall's 2.1C00
reaches row 120h, stopped flush at 0114.FFFF (2.3800); the landing's body (21) finds 2 px above
and none below, so the crouch is taken instead, raised 4 px to 0110.FFFF. The skid is the step's
output (§1.2, read by the camera after Samus's handler), and no compared quantity showed it, so
the format gains `after.skidding`, compared only where given. The next frame can't show it:
the crouch moves by the dash alone, `$90:8EA9` reads her facing for `0B4A` 0 and 2 alike, and the
crouch then clears `0B4A`. That is also why C-fit-land-tunnel-next may start following the pad,
where the real continuation still carries on; its comment says so. With a local reading of the new key, a
mutant that resets her to following the pad when the crouch is taken instead fails
C-fit-land-tunnel-skid (`skidding` false) and nothing else new.

**O2, three arms that restated consequences (adversary 73).** Space jump's press without the
item (SJ1's complement), the landing refused or crouched instead "is no landing here" (the
complement of the corrected condition), and the floor under the unmorph's end (the ground's
"the floor never lands her") are now "So" bullets citing their vectors.

**The current model against the amended vectors.** 257 of 258 pass with the round's test reading
the new key; C-fit-land-tunnel-held fails (`landed_or_ended` true): new behaviour, the corrected
rule above. C-fit-land-tunnel-skid passes the model's own `skidding` output, but the current test
has no `after.skidding` key, so it fails there on the format only. The round-14 gate's mutants,
each one line in the clean crate, run against these vectors and reverted: M2b (the wall jump
ignores hi-jump) fails C-walljump-hijump; M3 (the spring ball's item test false) fails
C-morph-end-spring and C-morph-end-spring-air; M15 (space jump without the item) fails
C-space-jump-no-item; M5 (long counts from 32,778) fails C-held-long-edge; M9 (the unmorph's end
drops "on the ground") fails C-unmorph-air-end-off and C-unmorph-air-end-floor.

**Units.** `tools spec-units` with these notes: 213.0, unchanged (motion 79.0, pose chart
58.0): every new rule is a consequence of an arm already counted, so the budget stays 7,133, with
motion and the pose chart at their caps (2,813 and 1,584).

## Round 16's amendments (2026-11-10)

From adversary 78's observer items (V1-V3) and its finding I3, and adversary 81's findings 1, 3
and 4. 262 vectors (258 + 4) and 51 pose steps.

**V1, a landing pose fit refuses.** `$91:FDAE` refuses in three places: an enemy one-side handler's
re-look at the new radius (`$91:FE9E`, `$91:FEDF`) hitting, a block one-side handler's re-look at
the old radius (`$91:FF20`, `$91:FF49`) hitting, and `$91:FFA7` with `0B00` < 8. A landing's growth
is from the spin or wall jump (radius 0Ch), or a straight jump or a fall (13h), to 15h; a ball's
landing keeps its radius, so `$91:FFA7` never refuses one. With blocks alone the block re-look can't
hit: a landing leaves her flush (the floor's room `0A3C` 0), so `$91:FF49` re-looks up by g again,
the same rows `$94:96AB` tested first. A floor room of r ≥ 2 makes the re-look `$94:96E3` alone for
9 − r < 8 px at row top − (9 − r), which neither the 8 px look (row top − 8) nor the 9 px look
tested; but with block-aligned floor faces that row is always F − 33 for a floor at F, in the same 8
px quarter as top − 8, so the first look hits it too. A sloped floor breaks the alignment: the down
look through `$94:86FE` stops on the last air row, anywhere in the block. So C-fit-land-refused: a
spin (radius 0Ch) falling at 3.0000 from 0101.0000, x 0108, meets a solid enemy (centre 108h,117h,
radius 8,8) 2 px below (`$A0:A8F0`, gap 2: stopped by 2 px, subpixel 0 kept) at 0103.0000. The
landing's radius 15h gives g = 9. Enemy looks: up none; down touching (room 0, `0AFC` = 0), flags 2;
`$91:FEDF` re-looks up 9 at radius 15h, `|0FA − 117h|` = 1Dh, not under 15h + 8: clear. Block looks:
up 8 (top EFh, block (100h,E0h), air) and 9 (EEh, air): none; down 8 (bottom 116h, the slope at
(100h,110h), its top row 1 at the centre column 8) stops with room 2 and writes no subpixel; flags
2. `$91:FF49`: `$12` = 2 − 9 = −7, `$94:96E3` up 7 at radius 0Ch: top F0h, the stepped block
(100h,F0h)'s upper half, solid (`$94:8DBD`, both halves for her span 103h-10Ch), carry set, `0AFC` =
0. `$91:FE82` puts `0A20` back, `$91:F404` returns carry set, and `$91:EBEE` skips command 5:
`0B2E`.`0B2C` stays 3.1C00 (the fall's pull), `0B36` 2, and no `$91:F1EC`, so no `$E926` and no
landing for jump held through. The spec's same-side arm, "original only" before, now names the
block's re-look and cites the vector. A second solid enemy above her at the new radius would refuse
a landing on an enemy through `$91:FEDF` too; the vector format holds one enemy, so the slope
carries it. The round-15 model passes it; the gate's mutant R4 (a refused change keeps what she met)
fails it alone (speed_y 0).

**V2, under bounce 1's threshold.** C-bounce-under: the ball at 0118.0000 falling at 2.D400
meets the floor at 120h, stops flush at 0119.FFFF, and the pull makes 2.F000. `$91:EA15` (not
bouncing) reads `0B2E` = 2, under 3, and goes to `$91:EA2D`: the ball's rest pose, then the
landing's command (`0B20` 0, speeds 0). The model passes it;
mutant B1 (bounce 1 from 2 px) fails it alone (bounces 1, speed −1.0000, pose kept).

**V3, facing the screen counts as right.** A convention the vector pins, not a difference in RAM.
Facing the screen she never moves (`$90:A383`), and `$90:8EA9` would read `0B4A` 1 with facing
word 0 as rightward, so "a leftward speed facing the screen" is the decoding's reading, not the
game's: the RAM checked is `0B4A` cleared with `0B46` kept. The spec states the convention, and
the model and the bridge must agree on it. Pose 00h/9Bh with `0E18` = 0:
`$90:A383` returns before any move (speeds kept), and `$91:804D` runs `$91:81A9`; with nothing
held (`8B` = 0) it goes to `$91:82D9`, whose `$91:8304` gives movement type 0 command 2, and the
pose is kept. `$91:EBEE` runs command 2, `$91:ECD0`: `0B4A` = 0. So a skid facing the screen
(`0B4A` 1, decoded leftward) ends with the speed kept, and `0B4A` 0 with facing word 0 decodes
rightward. C-screen-skid: −1.0000 facing the screen, nothing held → +1.0000, not skidding. The
chart's "standing still, nothing held: a skid ends" already said so; the spec now says it where
facing the screen is described. The model passes it; mutant S1 (`Side::of(Screen)` is Left)
fails it alone (speed_x −1.0000). The facing-forward handler skips her move, not the pose
input, so the skid's end comes as on any standing frame.

**V1's push (adversary 81, finding 1).** C-fit-land-pushed is C-fit-land-refused without the
stepped block: `$91:FF49`'s 7 px re-look meets air, `0A34` has no bit 1 (no enemy above), and
`0AFA` −= 7, so she is at 00FC.0000, placed there, and the landing's command 5 runs (speeds 0,
`$E926`). She overlaps the enemy by 2 px. The same-side arm is now checked on both its outcomes.

**The facing-screen body (adversary 81, finding 4).** Poses 00h and 9Bh have a Y radius of 18h
(`$91:B629`), so C-elevator and C-screen-skid give [5, 24], C-screen-skid flush on 120h at
0108.FFFF.

**I3, the stepped block's quarters.** The format header now allows exactly two characters per
string, each `#` or `.`, and no other character.

**The current model against the amended vectors.** All 262 vectors, the 51 pose steps and the
catalogue test pass. The gate's three survivors, each one line and reverted: R4 fails
C-fit-land-refused, B1 fails C-bounce-under, S1 fails C-screen-skid, and each fails nothing
else.

**Units.** `tools spec-units` with these notes: 213.0 → 214.0 (pose fit 16.3 → 17.3): the
same-side arm was original only and is now checked. The budget is 7,169, with pose fit 624;
motion and the pose chart stay at their caps (2,813 and 1,584).

## Round 17's amendments (2026-11-10): the speed booster

Domain S2's first part (docs/architecture/domains.md), and adversary 84's observer items (V4, V5,
blind points 2, 8, 9b and 16, and R-B1's recount). 284 vectors (262 + 22) and 59 pose steps (51 +
8).

**What the booster changes in her motion, and where.** The readers of `09A2` bit 2000h in banks
`$90`/`$91` are `$90:9789` (the dash's cap), `$90:84FF` and `$90:8558` (animation delays: her looks
and the boost counter), `$91:E644` (the pause screen's equipment change: an item switched mid-pose),
and `$90:9919` and `$90:99A6` (the jump and wall-jump speeds). Of the readers of the boost counter
`0B3E`/`0B3F` (speed boosting is counter 4): `$90:81C9` (lava cancels it and zeroes the dash:
liquids), `$90:87C5` and `$90:EEE7` (echoes), `$90:A413` and `$90:EE64` (footsteps and their dust),
`$90:F596` (the echo sound), `$91:D9EA` (contact damage `0A6E`), `$91:F7B0` (the crouch's shine: the
shinespark's charge) and `$90:D00F`/`$90:D306` (the spark itself). None of them moves her. So the
motion is the cap and the jumps; the counter, which `$90:852C` advances as the running animation
reaches its delay instructions, is the shinespark's and her looks'.

- **The cap.** `$90:9789`: with the booster, `0B42` compared with `$90:9F0D` (7) and `0B44` with
  `$90:9F13` (0), each `BMI` (the `below` of `WordwiseTopSpeed`), else `0B42`.`0B44` = 7.0000;
  without it `$90:97C7`..`$90:97F1` the same with `$90:9F19` (2). The gain is `$90:9F01`/`$90:9F07`
  (0.1000) either way, `$90:97F3`. Records: samus-shinespark-right 73 → 74 (6.F000 → 7.0000), 74 →
  75 (7.0000 kept, SB1, SB2), the same in the other three shinespark rows; samus-mockball 0-41
  (2.6000 → 4.F000, MB1 at 26 → 27, on the flat floor at 1232 between the slopes of 10-23 and
  91-94). The run's base speed stays capped at 2.C000 (`$90:9BD1`'s table has no booster column):
  the move is 9.C000 at the cap (shinespark-right 74 → 75: 0254.FFFF → 025E.BFFF).
- **The jumps.** `$90:9919` and `$90:99A6`, after the speed from the table: `LSR` of `0B42` added to
  `0B2E`, and `0B44` added to `0B2C` with its own `CLC`, so the subpixel's carry never reaches the
  pixel part. The callers of `$90:98BC`: `$91:FC94` (take-off, after the crouch lift), `$91:FCA9`
  (spin start, not from a spin or a wall jump), `$91:FC3D` (the spring ball's hop from the ground),
  `$91:F268` (the spring ball landing with jump held), `$90:A4C4` (space jump), `$91:8124`
  (knockback's end, out of scope); of `$90:9949`, `$91:FC13` (the kick). The dash read is the one
  the frame's movement handler left: the pose transition runs after `$90:973E`. Records:
  samus-mockball 41 → 42 (MB2: 4.F000 grows to 5.0000, 6.0000 + 2 = 8.0000), item-wave-beam 6 → 7
  (WB1: 2.0000 grows to 2.1000, without the booster it would stay 2.0000; 7.1000), r08 186 → 187
  (R8J: no run held, 0.F000 kept, 6.F000), r08 227 → 228 (1.2000, 6.2000), and every other rise
  below. No record has a wall jump with a dash (all nine kicks with the booster have `0B42` = 0) or
  a jump without hi-jump and the booster, so the carry's loss and the wall jump are constructed.
- **What ends the dash** with the booster is `$91:DE53` (cancel speed boosting: `0B3C` = 0, a dash
  to lose) and the zeroing writes the spec already has, from the same callers (`$90:A3D0`..
  `$90:A7BB`, the halts; `$90:E5E9`, the wall stop; `$91:EC76`/`$91:EC8E`/`$91:ECD3`, the change's
  speed effects; `$91:EFA5`, `$91:F66F`, `$91:FA45`, the turns); none tests the booster. MB4 is
  samus-mockball 109 → 110 (the spring ball let go: 5.0000 gone with the halt).
- **The wall-jump kick keeps the dash.** `$91:F2D3` zeroes the base speed and `0B4A` only;
  `$91:FA76` writes the dust. So a dash carried through the spin (WK1's 1.2000) is there at the kick
  and, with the booster, adds to it.

**Per frame over the records** (`$TEMP/collab/samus-spec-r17/perframe.py`, cache-v2, area-maridia
left out): running at a frame's start and after it with run held, the dash at N+1 from the dash at N
by the cap the items give agrees on 2,628 of 2,680 frames, against 2,119 for the cap without the
booster. The 52 misses are frames the movement handler doesn't grow it on: 27 on which she doesn't
move at all (the game skipped her that frame, as chain-0314-DAE1 275 → 276), 7 whose pose change the
same frame zeroes it (a halt or a turn), 13 in terrain-quicksand's sand (`$90:9766`) and 5 in
chain-0234-D51E 70-74, under water. A rise starting at the jump or wall-jump speed, its speed at N+1
from the items and the dash: 188 of 189, against 166 for the plain speed; the miss is
samus-ceres-push 989 (a push).

**The vectors.** Recorded: MB1, SB1, SB2, MB2, WB1, R8J, MB3 (samus-mockball 100 → 101, the spring
ball rolling 3.4000 + 5.0000) and MB4, each on a flat floor the record shows her flush on. MB3 and
MB4 start rolling, following the pad (`0B4A` 0); the mockball's first rolling frames (75-77) still
carry on from the spin and have no plain start within reach. Constructed, each worked by hand from
the cited vector: C-booster-dash, C-booster-dashcap-a and -b (C-dashcap-a and -b at 7.0000),
C-booster-carry (J1: E000 + 7000 carries, 4.5000), C-booster-walljump (WK1: 4.A000 + 0.2000),
C-booster-space-jump (SJ1 with 3.4000: 6.0000 + 1 + 0.4000), C-spring-hop and C-booster-hop,
C-booster-bounce (C-bounce with a dash), C-spring-land-held and C-booster-spring-land (`$91:F25E`:
jump held, `0B20` = 0 and the jump, carry set so no landing command: the speed and dash stay).

**Harness.** Gap 31's mask (x where `0B42` ≥ 2.0 with the booster, and y on the slope frames that
follow it) is the old cap's: with the round's model the harness can compare x and y there again. The
frame selection's "0B42 below 2.0" (How the spec was made) is no longer needed for the booster; it
remains a fact of the earlier vectors.

**V4, a change of shape landing with a run.** `$90:A640`: a landing (`0DC6` = 0401h) in the
transition poses zeroes `0B2C`/`0B2E`, `0B36` and `0B20`, and nothing else: `0B46`.`0B48`, `0B4A`
and the dash stay. r04 331 → 332: unmorph 3Dh falling at 2.3000, `0B4A` 2, base 1.0000, dash 0.2000;
x + 0.2000 (`$90:9348`, the dash alone), y flush at 00B0.FFFF, `0B36` 0, base 1.0000. samus-mockball
70 → 71: morph 37h at 2.8400, base 1.4000, dash 5.0000; x + 5.0000, y 04B9.FFFF. Both start carrying
on (`0B4A` 2), which a vector's start can't give, so C-unmorph-land-run and C-morph-land-run start
following the pad with the same quantities; the change of shape moves by the dash alone either way.
Adversary 84's mutant M1 (`Settles` as `Land`) gives speed_x 0 on both. The bounces: `$91:ED11`
zeroes `0B20` (and the vertical speed and direction) when a change of shape starts, so they are
always 0 when one lands, and `$90:A640` zeroes them again; the vectors compare bounces 0. What
`$91:ED11` does to a bounce's rise was the spec's question 21, answered in round 19 (below).

**V5, the implementer's two refusals.**
- A crouched turn other than `Ground`: the crouched turning poses are 43h/44h (movement type 0Eh,
  the ground turn) and 97h-9Ah and A2h/A3h (aimed up, down and up-forward: movement type 17h, the
  turn in a jump, `$91:B629`). The pose map names all six ground turns, and the transition table
  reaches them only from the crouched poses, on the ground. The turns in a jump or a fall (2Fh/30h,
  87h/88h, 8Fh-96h, 9Eh-A1h) and the moonwalk's are upright. So as a name a crouched turn is always
  `Ground`, which §2 says, and a crouched `Fall`, `Jump` or `MoonwalkJump` turn is not a pose.
- But the aimed ones move as a jump turn (adversary 86, O1): `$90:A790` runs `$90:92B8`, which at a
  Y direction other than none moves her by her vertical speed with its calculations (`$90:90C4`,
  `$90:90E2`), and only at none pulls her by `$90:923F`; the ground turn's `$90:A67C` runs
  `$90:923F` alone, keeping the speed. In cache-v2 the aimed ones come up on 94 frames
  (boss-phantoon 64, samus-ceres-push 30), every one with `0B36` = 0; of the unaimed, only 43h has a
  kept speed (samus-shinespark-left and -up-left 184-189, +1.3400, kept). With no vertical motion
  the two handlers move her alike, so the records can't show the difference, and I had no oracle run
  from a constructed start: the spec's question 22, not a rule.
- The spring ball's hop on the ground: the hop (7Fh/80h, movement type 12h) is entered only by
  `$91:FC18` with the jump's rise, and leaves for 79h/7Ah (at rest) or the falling 7Dh/7Eh by a
  landing (`$91:F25E` and `$91:F27F`..`$91:F2C0`); a bounce keeps it in the air. So it is never on
  the ground at a frame's start; §1.2 now says the hop is in the air from its start to its rest, and
  a start at a vertical speed of 0 there has just started to fall.

**Blind 2, the body's first frame.** `$90:EC22` sets `0B00` from the pose at the start of each frame
(`$90:E6B0`, before the pose input); within the frame only `$91:FDAE` (pose fit, a body that grows)
and `$91:ECF1` (the shrink's look) write the new pose's radius. So a change with neither (J1's spin
start, 21 → 12) ends the frame with the old radius, and the new one holds from the next frame
(samus-mockball 42: 21, 43: 12). The spec's §1.3 says so, and the vector format's next_pose radius
is the record's at the frame's end.

**Blind 8, the fall's limit at exactly 5.** `$90:910C`: `CMP #$0005`, `BEQ` past the add, so only a
pixel part of 5 stops it. A speed set at 6 px or more grows on (C-fallcap-past). The pull, 0.1C00,
is under a pixel, so a fall that grows from below always passes through 5.

**Blind 9b, the pose steps' speed, place and items.** The pose-step format gains `items`, `speed_x`
and `in_the_air`; their defaults are what the steps meant before (no items, speed 0, in the air in
the airborne poses). The new steps: PS-run-kept (samus-grapple-walljump 0 → 1, pose 11h kept at
2.C000 with nothing held) and PS-run-stops; PS-ball-rolls (samus-mockball 75 → 76, 79h → 7Bh) and
PS-ball-rests (109 → 110, 7Bh → 79h); PS-ball-air-kept, PS-spring-air-kept, PS-spring-hop and
PS-ball-unmorphs, the chart's ball rows by the spring ball and the ground. The landed vector test
reads none of the three keys (`deny_unknown_fields`), so with it unchanged the file fails to parse;
with a local reading of them (`items` to the chart's spring ball, `speed_x` to its speed,
`in_the_air` over the action's default), the round-16 model passes all 59 steps.

**Blind 16, parameters.** MODEL §3: a `Parameter` is a value in `Rules`, and every value the model
reads comes from `Rules`. The spec now defines the switch line (Header, Switches): `parameter` where
the entry's params are numbers a game may set (SpeedBuildsAndBrakes, Dash, AimingAndShooting,
RidingTheElevator, Jumping, FollowingTheGround, PoseFitAfterMoves, ShrinkingLowersHer,
RisingAndFalling, WallJump, Landing, BlocksStopFlush), `none` where it has none or its number is the
pixel a subpixel part reaches into, which the boxes' whole pixels then measure (SolidEnemies' 1 px),
and the Toggle for WordwiseTopSpeed. "Mechanic and parameter" kinds became "mechanic": the switch
line carries it. The round-16 catalogue gives Parameter to SpeedBuildsAndBrakes and Jumping alone,
so its catalogue test still passes; the next round's catalogue follows the switch lines.

**R-B1, motion's units recounted.** Motion counts 84.0 units now (79.0 before this round): 71 arms,
8 cells, 31 operators. Ten of its arms count 0 as original only or record only (SpeedBuildsAndBrakes
3, TurningRound, AimingAndShooting's shot, Jumping's carry with a dash, DriftInTheAir's carry,
Moonfall's end by a kick, WallJump's enemy test, Landing's turn end), and the frame's order
(`PoseFollows`, `PoseFitAfterMoves`, `MovesAsShePosed`) is prose, not arms; vectors for all of them
would add about 10 units. The booster's own arms (+4.7 units here, counted in motion) raise
motion's budget by 36 × 5.0 = 180, as every area's budget is k × its units. The old code shown
beside it had no booster rule (c7f927ab6's `extra_run_speed` has one cap and no jump bonus).

**The current model against the amended vectors.** With the landed test and a local reading of the
pose steps' three keys: every old vector and pose step passes, and 12 of the 22 new vectors fail,
all of them the booster: MB1, SB2, C-booster-dash and C-booster-dashcap-b (the cap: the model's dash
stays 2.0000), and MB2, WB1, R8J, C-booster-carry, C-booster-walljump, C-booster-space-jump,
C-booster-hop and C-booster-spring-land (the jump's share of the dash; WB1 also the cap). SB1 and
C-booster-dashcap-a pass by the part-by-part test (6.F000 and 7.F000 have a subpixel part below 0,
so they grow under any cap), and MB3, MB4, C-spring-hop, C-spring-land-held, C-unmorph-land-run,
C-morph-land-run, C-fallcap-past and the eight pose steps pass.

**Adversary 86's observer items.** O1 is in V5 above and the spec's question 22: the aimed crouched
turns' movement type is the jump turn's, which the records can't tell from the ground turn's (they
never have one with a vertical motion), so it stays open until an oracle run from a constructed
start shows it. O2: C-booster-bounce, C-bounce with the booster and a dash of 3.0000, right held:
`$91:F212` sets the bounce speed from `$90:9EB5`/`$90:9EB7` alone and never reads `0B42`, so it
rises at 1.0000; the round-16 model passes it. O3: the Switches note says which tables are fixed
(those whose params read "none (the table is data)") and that `SpeedBuildsAndBrakes`' table is a
parameter; SolidEnemies stays `none` because its 1 px is the pixel a subpixel part reaches into,
measured on the boxes' whole pixels (`$A0:A8F0`), not a distance of its own; PoseFitAfterMoves'
params name the 8 px small body; Jumping's params say the booster share's arithmetic is fixed.

**Units.** `tools spec-units` with these notes: 214.0 → 219.0 (motion 79.0 → 84.0: Dash 3.3 → 4.7,
the booster's cap; Jumping 11.0 → 13.7, the booster's rise and its formula; Landing 6.7 → 7.7, the
spring ball's landing with jump held, original only before). The budget is 7,704 → 7,884: motion
2,844 → 3,024, the other areas unchanged.

## Round 18's amendments (2026-11-11): what a frame reports for her animation

The outputs samus-appearance's second half reads (samus-appearance.decode, "Requests to samus-movement"; its
gaps 1 and 2), and the queued observer items O1, J3 (adversary 88, item 5), I3 (item 3), V4 and V5
(adversary 84, items 9 and 10). 308 vectors (284 + 24) and 63 pose steps (59 + 4). Scratch:
`$TEMP/collab/samus-spec-r18/` (`r18dump` over cache-v2 with `0A2E`, `0A2A`, `0B46`..`0B4A`,
`0AF8` added to the looks dump's words; `dashref.py`, `endcheck.py`, `recann.py`, `ref18.py`).

**Where the original sets each output.**

- **Dashing** is the running momentum flag `0B3C` (spec `Dashing`; Sources, above). Set only in
  the running handler's speed, before the move, so a run into a wall sets and clears it in one
  frame (`$90:9789` then `$90:E5CE` from the move, `$90:9370`..`$90:93E3`). Cleared by
  `$91:DE53`, which zeroes the boost too but not the dash: in the movement handlers, before the
  animation (`$90:E73B`), and in the pose change after it. Each clearing site is a rule of the
  spec's: the standing, ran-into-wall, turning, ball-at-rest and falling-ball handlers and the wall
  stop are "her moves", the failed lookup's commands 1, 2, 6 and 8 are the let-go effects other
  than `none` (`$91:8332`: spin jumping, 3, has 0), and `$91:F66F`/`$91:FA45` the spin reversed and
  the ball turning (`$91:F624`, `$91:FA0F`: a facing changed from a spin, a wall jump or a ball).
  The crouch's handler `$90:A573` zeroes the dash without it, so a crouch from a run keeps dashing
  at 0 (boss-kraid-defeated 607-613, its let-go at 614 clears it); a turn's start joins the dash in
  its initialisation (`$91:F931`, `$91:F969`, `$91:F9A1`) and its handler clears the flag the next
  frame. `$91:EB88` runs the prospective command only when neither a super special nor a special
  pose wins (`$91:EBC3`, `BRA` past `$91:EBF5`), and a landing's vertical collision result
  writes its pose and command 5 over the pad's (`$91:E8B6`), so a landing or an animation's end
  replaces the let-go's clearing. Out of scope: lava (`$90:81CE`), the suits' pick-up (`$91:D4FE`,
  `$91:D5D6`), the grapple (`$91:EFA5`), the pause screen's equipment change (`$91:E655`), the
  quicksand's reaction to her inside it (`$84:B408`, the one writer of `0B3C` outside banks `$90`
  and `$91`), and water, lava or acid without the gravity suit, where no dash begins (`$90:975D`,
  `$90:976E`). The elevator's start (`$91:E3F6`) zeroes the
  dash (`$91:E475`) but not the flag: facing the screen on the ride she stays dashing.
- **Dash began, dash lost**: `0B3C` 0 at N and set in the moves; set at N or in the moves and 0 at
  N+1. A dash begun and lost in one frame shows in no record (both inside frame N).
- **Dashing after her moves**: `0B3C` as the animation reads it (`$90:84EF`, `$90:857D`), after
  the movement handler and before `$91:EB88`. No record cuts there.
- **Heading down**: `0B36` = 2 as the rising hold reads it (`$90:8032`), inside the animation,
  after the movement handler: what her moves met writes the direction only at the pose change
  (`$91:E8B6`'s command 5: a ceiling down at `$91:EFDF`, a landing none at `$91:F010`, a walk off
  down at `$91:EFEF`, a bounce up), and so do the pose change's rises. So a rise into a ceiling is
  not heading down on its frame: on a straight jump's rising step with 1 frame left the game holds
  the picture 3 frames more (adversary 94, F1; chain-0059-A293 200 is such a ceiling, off the
  hold). The records show `0B36` at N+1, which is her moves' value on every frame without
  command 5 and without a rise the pose change starts. On the ground the kept speed keeps its
  direction (`$90:923F` writes none: 996 running frames with a speed, all 2), falling at 0 is 2
  (samus-bomb-jump-vertical 1035), the moonfall is 0 (`$90:913A`: the 102 spinning, 194 jumping
  and 30 jump-turn frames with a speed and direction 0 are chain-0014-96BA's and
  chain-0232-D5A7's moonfalls), a rise is 1.
- **Handed over**: `0A32` = 3 with `0A2C` bit 15 clear at N+1 (the super special command
  "transition animation finished", `$91:F36E`). No hand-over's pose is taller than the pose it
  ends (the 78 ending poses' radii against their operands' in `$91:B629`), so pose fit never
  refuses one, and the pose she ends the frame in is the table's.
- **The end's kind** is an input: samus-appearance's end from the frame's start (its harness's
  `Playhead::end_ahead` over the importer's clips). F8h is the end unless a jump; F9h, FAh, FCh and
  FDh the plain end. F8h ends every turn (25h, 26h, 2Fh, 30h, 43h, 44h, 87h, 88h, 8Bh-A3h,
  BFh-C4h) and every landing (A4h-A7h, E0h-E7h).
- **The end gives way**: `$90:8370` returns with no request when the pad's prospective pose
  `0A28` (written by `$91:81EC` before the movement, with `$90:E926`'s jump held through ORed in)
  is 4Bh, 4Ch (the take-off aimed forward) or 19h, 1Ah (the spin), outside the demos. The landings'
  transition rows (`$91:A0EC` and its left twin) take off aimed (55h, 57h, 59h) with up or an aim
  button, which don't give way; the ground turns' rows (25h/26h, 8Bh-8Eh, BFh-C4h) name only
  4Bh/4Ch and 19h/1Ah; the turns in a jump, a fall, the crouched turns and 9Ch/9Dh have empty
  tables. Over the cache (`endcheck.py`): 8 ends gave way, all ground turns to the pad's take-off
  or spin (samus-screw-attack 243, 315, 387, 495, 531, 675 and two more), and 165 ends unless a
  jump handed over (17 landings from a jump, 9 from a spin, 91 ground turns, 32 jump turns, 14 fall
  turns, 2 moonwalk turns), with none in a demo; the two other gave-way frames (area-maridia 427,
  chain-0106-AD5E 726) are the x-ray's, outside the domain.

**AnimationEnds' table against the original.** `handover.py` reads each of the 78 ending poses'
go-to-pose operands from `$91:B010` and compares them, through the pose map, with the table's
rows: all agree but the aimed `Crouching` (F1h-F6h → 85h, 86h, 71h-74h) and the aimed
`StandingUp` (F7h-FCh → 03h-08h), which keep their aim. The table now says "aim kept" there; the
round-17 model's rows end them unaimed, and P-crouch-end-aimed and P-stand-end-aimed fail it. On
record only terrain-quicksand 391 (F3h → 71h), a push's row.

**The reference model.** `dashref.py`, the spec's `Dashing` rules over the cache's frames of play
in the domain (samus-appearance's frame set: game state 8, the frame count running, both poses in the
pose map), taking the let-go from the original's command `0A2E` at N+1 and the wall stop from a
running pose taking `AgainstWall` with the dash 0 or, in the air, the base speed and dash 0 with x
flush: **16,210 of 16,210** frames' `0B3C` at N+1; the null model (it stays as it was) 16,021. By
rule: 90 dashes begun (79 running on, 10 into a spin, 1 into a turn), 50 ended in her moves (14
landings from a jump and 1 from a spin, 7 standing, 1 against the wall, 13 turns, 14 wall stops: 9
running, 2 jumping, 3 spinning), 50 at the pose change (14 running's "as running", 3 jumping's, 3
falls' "dash 0", 3 balls' let-go, 9 crouched let-goes, 15 spins reversed, 3 balls turning). `ref18.py` over
the vectors: the 24 constructed or given-way vectors and the 5 constructed `heading_down` ones agree
with their hand-worked facts (which row a frame takes, a wall stop, the phase at the end), the pose
steps' ends unless a jump take off where the rule says, and the 95 recorded vectors' `dashing`,
`heading_down` and starts are their records' (`recann.py`), 0 disagreeing.

**For the harness (the integrator).** The State gains dashing, decoded from record N as `0B3C`
≠ 0 and written back as 1 or 0 (the boost `0B3E` stays samus-appearance's). Compared at N+1: dashing,
`0B3C` ≠ 0; dash began and dash lost, from N to N+1 (a dash begun and lost inside one frame, a run
into a wall, is a vector's only: C-dash-begins-wall); heading down, `0B36` = 2 at N+1 where
`0A2E` at N+1 isn't 5 and the pose change starts no rise (elsewhere it is unverified: a landing's
frame heads down, a ceiling's doesn't, a walk off's doesn't, and the pose change's rises keep the
moves' value; the cache has no ceiling on the rising hold's frame); handed over, `0A32`
= 3 with `0A2C` bit 15 clear. The input "her animation's end" is samus-appearance's end from record N
(its harness's `Playhead::end_ahead` over the importer's clips, the step at `0A96` and the time
left `0A94`), in place of today's `animation_ends`. `dashing after her moves` has no record cut:
samus-appearance's harness checks it through her run's pace and the boost. Nothing new is masked; the
x-ray's two gave-way frames above are outside the domain.

**The current model against the amended vectors.** With the landed vector test, the new keys
stripped, the ends read as `true` and `Dashing` dropped from the entries (`strip.py`, not
committed): every vector and pose step passes but R-turn-gives-way, C-landing-gives-way,
P-turn-gives-way and P-landing-gives-way (the model hands the end over: standing, rising 0) and
P-crouch-end-aimed and P-stand-end-aimed (unaimed). The 20 dashing vectors' motion and poses pass
as they stand. The catalogue test refuses `Dashing` until the round adds it.

**O1, gap 37.** `$90:EC22` sets `0B00` from the frame's pose as each frame starts, under the
normal state handler (`$90:E6B0`); the ride's handler `$90:E713` (from `$91:E44D`, in `$91:E3F6`,
which turns her to the screen as the elevator starts) doesn't call it, so the ride's frames use the
radius word the pose before set: the standing pose's 21 px when she steps on standing, on the 906
cache frames of pose 00h on a ride. §1.3 and `RidingTheElevator` say so. No vector: her y on the
ride is the elevator's, and the body is the caller's input, which the bridge gives as gap 37 says.

**J3.** The take-off's own body is 19 px (4Bh's radius), as the jump's. Record r04 474 shows 21 px
at the take-off's start because `0B00` holds the standing pose's until the frame sets the
take-off's, so J3's start pose now gives 19 px, the body the frame's moves use, and its next_pose's
19 px is that same body: no shrink.

**I3.** A local reading of the landed test that logs each pose whose body falls back found 14
constructed vectors, not 12: C-wall, C-enemy-ahead, C-touch-wall, C-kind-spikes,
C-kind-reacting-solid and C-kind-stand-in (`AgainstWall`) and C-slope-land, C-enemy-down,
C-enemy-overlap, C-step-down, C-down-even, C-down-odd, C-slope-edge and C-slope-floor-empty
(`Landing` from `Jump`). Each now gives its chosen pose as next_pose with its body, the starting
body's 21 px (their invented bodies are equal, so nothing in them changes), and the format says a
vector gives every body its frames read. With the test refusing a missing body (a local patch), every
vector passes; the refusal itself is the implementer's.

**V4 and V5** were answered in round 17 (above, "V4, a change of shape landing with a run", "V5,
the implementer's two refusals"): C-unmorph-land-run and C-morph-land-run tell `Settle` from
`Land`; a crouched turn is always `Ground` (§2) and the spring ball's hop is in the air from its
start (§1.2), so neither of V5's cases arises.

**The gaps file** keeps gap 36 alone; the answered gaps (1-35, 37) are in git history at
`858925a3e`.

**Vector format** (for the round's brief): `animation_ends` takes the end's kind, `"End"` or
`"EndUnlessJump"` (the 15 old `true`s by their pose, the game's kinds); a pose step's `met` gains
`"animation_end_unless_jump"` (P11 and P19 take it); `before.dashing`; `after.dashing`,
`after.dashing_after_moves`, `after.dash_began`, `after.dash_lost`, `after.handed_over` and
`after.heading_down`, each compared only where given; and the test refuses a vector whose frame
reads a body it doesn't give.

**Adversary 94's fixes** (spec gate, LAUNCH AFTER FIXES). F1: `heading down` is reported as her
moves leave her (above), with an arm for the ceiling's frame (C-ceiling, now false), W13 (a
landing) and MB2 (a spin started over a kept downward speed) worked from her moves, WK1's dropped,
and C-dash-landing and C-slope-run-off comparing it; samus-appearance's input row names the point. F2:
C-dash-landing-letgo (a fall's let-go replaced by the landing) and C-dash-takeoff-end (a take-off's
let-go replaced by its end, handed over), both dashing. F3: the ride's body is the caller's input
(§1.3), and `RidingTheElevator`'s line a note. F4: "either ball" in `Dashing`, and
C-dash-spring-rest (`$90:A6CF`). F5: C-landing-gives-way-held, a give-way through jump held
through. F6: the out-of-scope list above. F7: `Dashing`'s `checked:` gives the frames each rule
alone decides (adversary 94's table) and names what is constructed only. F8: samus-appearance §8 q2
is decided (a), for its engine-wiring round. F10: the format note says a recorded vector's end
kind is its pose's in the game's clips. 312 vectors and 63 pose steps. `ref18.py` agrees with all
34 constructed and given-way vectors and the 95 recorded ones; its mutants "only `Ball` clears",
"a landing takes the let-go", "an animation's end takes the let-go" and "heading down at the
frame's end" each fail a vector (C-dash-spring-rest; C-dash-landing-letgo; C-dash-takeoff-end;
C-ceiling and C-slope-run-off). The ceiling's arm adds 1 unit.

**Units.** `tools spec-units` with these notes: 219.0 → 229.0 (motion 84.0 → 92.0: `Dashing` 5.0,
`RisingAndFalling` 5.7 → 8.7 for `heading down` and the ceiling's frame, `RidingTheElevator`
unchanged; pose chart 58.0 → 60.0, `AnimationEnds` 2.7 → 4.7 for the end that gives way and
`handed over`). The budget is 7,884 → 8,244: motion 3,024 → 3,312, pose chart 2,088 → 2,160.

## Round 19's amendments (2026-11-11): adversary 96's observer items

Adversary 96 (the round gate of round 18) queued F3 to F6, F8 and F9 for the observer. Each is
answered from the bank logs and the cache-v2 dumps (`$TEMP/collab/samus-spec-r18/out`, read by
`$TEMP/collab/samus-r19/*.py`).

**F3, a wall jump's rise ends with jump let go.** Wall jumping (movement 14h, `$90:A734`) runs
`$90:8FB3`, whose rise check (`$90:8FDC`..`$90:8FF9`) zeroes the speed and sets direction down when
the direction is up and jump isn't held, as the normal jump's and the spring ball's do; only the
normal jump's transition poses (4Bh, 4Ch, 55h-5Ah) skip it. `RisingAndFalling`'s arm now names the
wall jump. Record: samus-grapple-walljump 114 → 115 (R-walljump-letgo): pose 83h, pad 0000,
`0B2E`.`0B2C` 5.4800 up; at 115 y unchanged (`0AFA`.`0AFC` 00ED.93FF) and 0.1C00 down. It is the
cache's only wall-jump frame rising with jump let go (`wjletgo.py`).

**Found with F3: the wall jump's let-go into the screw attack's or space jump's spin takes no
effect.** The same frame keeps `0B46`.`0B48` 1.6000 and `0B4A` 2, though the pad chart's last row
for wall jumping (`$91:81DB` → `$91:82D9`, command 6 from `$91:8332`) zeroes them. Why:
`$91:EB88` runs the prospective pose's command only when `$91:F404` returns carry clear, and
`$91:F404` returns carry set when the pose's initialisation changed the pose. The spin's
(`$91:F624`) replaces 19h/1Ah with the screw attack's 81h/82h (`09A2` bit 0008) or the space
jump's 1Bh/1Ch (bit 0200), outside a liquid or with the gravity suit (liquids are out of scope).
So command 6 (`$91:EC85`, falling into `$91:EC8E`: speed 0, mode 0, `$91:DE53`, dash 0) is
skipped, and dashing stays as it was. The spin's own initialisation (the turn's speed join, the
spin carried on) ran before the test, so it stands. Of the lookup failures, only wall jumping's
lands in a spin (`$91:B629` byte 2 of 83h/84h is 19h/1Ah); the found rows' commands are 0. Over
the cache, all 4 wall-jump let-goes into a spin keep the speed and mode (samus-grapple-walljump
114, samus-walljump-16 57, samus-walljump-22 63, samus-walljump-28 69, all with the screw
attack), the frames the harness misses on base speed and mode in those rows. The new `PadChart`
arm says so (R-walljump-letgo; C-walljump-letgo-space, space jump alone; C-walljump-letgo-plain,
neither item, where the halt acts and ends dashing). The items table names the two items.

**F4, the ride takes no pose.** The riding state handler (`$90:E8EC`, installed in `0A44`) runs
the movement handler, the minimap and the animation, and no pose input handler and no
`$91:EB88`: her pose can't change on the ride. Over the cache, 906 ride frames, none with a pad
held, the pose kept on each (`ride.py`). `RidingTheElevator` says the ride is decided before a
pose is chosen; C-elevator-jump (jump pressed, which on foot takes off) and C-elevator-side (left
held, which on foot turns) end facing the screen, not handed over. The jump held through count
(`0AF4`, at the draw) still runs on the ride; no vector compares it there.

**F5, gap 38: a change of shape lands in its moves.** `$90:A61C` moves her by her vertical speed
(`$90:92C7`, or `$90:923F` with no direction), then at `$90:A640`..`$90:A651`, with `0DC6` = 0401h
(landed, downward result no change), zeroes `0B2C`/`0B2E`, `0B36` and the bounce state `0B20`,
in the movement handler, before the animation reads the direction. `Landing`'s change-of-shape
arm now says "within her moves", and `RisingAndFalling` has the arm "a change of
shape whose move meets the floor → not heading down". The 15 frames of gap 38 are exactly the
cache's movement-Fh frames with `0B36` 2 at N and 0 at N+1, all command 0 at N+1
(`shapeland.py`). Vectors: R-morph-land (item-reserve-tank 563 → 564, the morph falling at
1.1800 onto the floor, dashing, run and right held); C-unmorph-land-run (chain-0057-9FBA 332) and
C-morph-land-run (samus-mockball 71) now compare heading down. The bounce state's zeroing here
is unobservable: a change of shape never holds a bounce (adversary 97's O1, below).

**F6, dashing is her dash's keeping.** `0B3C` is one bit: `$90:973E` zeroes the dash outside a run
with run held only when it is 0. The spec now says a dash she has while not dashing is the dash
she is to lose (§1.2, `Dash`, `Jumping`, `Landing`), and a start that keeps its dash gives
`before.dashing = true`. Adversary 96's D1 prototype's vector lines are taken, 25 vectors: the
recorded R99, R100, R101, R102, R125, R137, SH3 and R8J, and the constructed C-limit,
C-walljump-hijump, C-step-far, C-step-left, C-span-step, C-morph-no-ball-air, C-shrink-floor,
C-near-wall-item, C-slope-roll-carry, C-slope-roll-carry-next, C-booster-walljump,
C-booster-space-jump, C-spring-hop, C-booster-hop, C-spring-land-held, C-booster-spring-land and
C-booster-bounce. `flagcheck.py` gives every recorded vector's start from `0B3C` at N (lead-ins
included): three more had the flag set and now say so, R97 (r03:215), R132 (r07:73) and J1
(r03:470), none comparing a field the flag changes; all 165 recorded starts then agree.
C-dash-lost-not-dashing and C-dash-kept-dashing tell the two apart.

**F8, her vertical speed as her moves leave her.** §1.4 gains `vertical speed after her moves`,
for samus-appearance's `FastFall` (root decision (a)). It is `0B2E`.`0B2C` signed by `0B36` after the
movement handler, before `$91:EB88`: the point heading down is read (`$90:8032`). `$90:90E2` adds
the pull before the move (`$90:90F7`..`$90:9124`, then `$90:9132`..`$90:9163`), and an upward
move's collision only sets `0DC6` (`$90:93EC` → `$90:E606`), so a rise stopped by a ceiling has
its pulled speed there (C-ceiling: −5.0000 + 0.1C00 = −4.E400); the ceiling's zeroing is command 5
at the pose change (`$91:EFDF`). A landing likewise keeps its pulled speed (W13: +1.DC00 +
0.1C00), a change of shape's landing has 0 (F5), and the pose change's rises come after (J2 0,
MB2 +0.E000). The vectors give `after.speed_y_after_moves`. samus-appearance's input row for her
downward speed still names motion's speed at the frame's end; its round 3 changes that row and
`FastFall`'s rule to this output.

**F9, two conditions that can't happen.** (a) "handed over" was worded "the pose she ends the
frame in is the one her animation's end handed over"; it is now "her animation's end handed over
her pose this frame", and `AnimationEnds` says the end's pose is always the one she ends in (no
hand-over's pose is taller, above, "Handed over"). (b) `AnimationEnds` now says an end never ends
her dashing at the pose change: the only hand-over with a dash effect is a turn's end, and a
turn's moves end dashing first (`Dashing`).

**For the harness (the integrator).**
- Drop the gap-38 mask (`movement.rs`, "spec gap 38"): the model then reports not heading
  down on the 15 frames.
- Compare `vertical speed after her moves` with `0B2E`.`0B2C` signed by `0B36` at N+1 on the
  frames heading down is compared on (no command 5 at N+1, no rise the pose change starts), and
  mask it by the same writers elsewhere.
- samus-grapple-walljump 115 and the three wall-jump let-goes (samus-walljump-16 58,
  samus-walljump-22 64, samus-walljump-28 70) should then match on every field; nothing new is
  masked.

**Adversary 97's fixes** (spec gate, LAUNCH AFTER FIXES).
- **O1, question 21.** A change of shape's row runs command 7 (`$91:ECDA`); at `$91:ED11`..`$91:ED1F`,
  with the bounce state `0B20` nonzero, it zeroes `0B20`, `0B2C`/`0B2E` and `0B36`: no vertical
  speed, not falling, bounces 0, at the pose change. `0B20` is nonzero only in a ball pose between
  a bounce and the landing that rests it, so the only change of shape that sees one is the
  unmorph from a ball (or spring ball) in the air after a bounce. `ShrinkingLowersHer` gains the
  arm, with C-unmorph-on-bounce (C-place-0 on a bounce's rise: −1.0000, bounces 1; her moves
  leave −0.E400, the pose change 0). No record has one. So no bounce reaches a change of shape:
  `Landing`'s change-of-shape arm drops "bounces 0" and C-unmorph-land-bounced goes (its start
  can't be reached). The hand-over's clear (`$91:F36E`, super special command 3, which also zeroes
  the bounce state) can't be observed in scope either: the bounce state is nonzero only in a ball
  pose, and no in-scope ball pose's animation hands over (the hand-over instructions `$90:8370`,
  `$90:839A`, `$90:84B6` name no ball pose but the unused 3Fh and 40h); the morph's own end hands
  over from `Morphing`, which command 7 has already cleared.
- **O2, a change of shape's rise into a ceiling.** `$90:A61C` ends with `STZ $0DC6` (`$90:A654`)
  whatever its move met, so `$91:E8B6` sees no collision: no command 5, no ceiling stop. The move
  stops flush (`$90:93EC`'s block collision) and the speed stays as `$90:90E2` left it, pulled,
  still rising, until spent. `WallsAndCeilings` gains the arm, with C-morph-ceiling (the morph at
  −6.0000 under a ceiling at y 64: y 0047.0000, −5.E400, rising). No cache frame has one (24
  change-of-shape rises, all unstopped).
- **O3.** C-takeoff-loses-dash (not dashing, dash 0.4000: dash 0, no move) and its twin
  C-takeoff-keeps-dash (dashing: she moves 0.4000 and keeps it; jump alone held, the take-off's
  last row, as running, then ends her dashing). `Dash`'s arm cites C-takeoff-loses-dash in place of
  W16, whose dash is 0.
- **O4.** C-elevator gives `after.speed_y_after_moves` (0), and C-screen-kept-speed faces the
  screen off the elevator with a downward speed she keeps (+0.E000, as a start on the ground may
  give): her speed after her moves is that speed, heading down.
- **O5.** `RidingTheElevator`'s arm names the pad alone; no clip of facing the screen hands over.
- **N1, the other places the pose's initialisation changes the pose** (`$91:F404` returns carry
  set, so `$91:EBEE` skips the row's command). In scope: pose fit's refusal (`$91:FE85`) and the
  crouch taken instead (`$91:FFC4`), both already "none of the chosen pose's effects"
  (`PoseFitAfterMoves`), and the morph refused without the ball (`$91:F7EF`, `PoseFollows`).
  Through found rows only, whose command is 0 (`$90:E6A7` clears it): the moonwalk's
  (`$91:F8A1`/`$91:F8A9`) and the turns' (`$91:F913`..`$91:F99E`). Dead or unused: running's
  `$91:F534`/`$91:F53C` (`0DF8` always 0), the damage boost, the unused 39h and 3Ah. Out of scope,
  with the shinespark: the normal jump's windup (`$91:F56C`/`$91:F579`), the damage boost's let-go
  4Fh/50h → 4Dh/4Eh, and the falling-at-0 jump's special command 4 (`$91:80F7`). So the wall
  jump's let-go into the screw attack's or space jump's spin is the one new case.

**The reference model.** `refpatch.py` applies the amendment to the landed model with adversary
96's D1 (`DashIs` dropped) and to its vector test (the new key), in scratch, never committed:
the wall jump in the rise's let-go; the ride's pose not chosen; the change of shape's landing in
its moves; a change of shape's ceiling stopping the move only (O2); a change of shape from a
bouncing ball ending its vertical motion and bounces (O1); the wall jump's last-row halt left out
with the screw attack or space jump; the speed captured at heading down's point, a ceiling's
pulled. **All 326 vectors and 63 pose steps pass.** Each part left out (`MUTANT=`) fails vectors:
F3 3 (R-walljump-letgo, C-walljump-letgo-plain, C-walljump-letgo-space), F4 2 (C-elevator-jump,
C-elevator-side), F5 3 (R-morph-land, C-unmorph-land-run, C-morph-land-run), the let-go's skip 2
(R-walljump-letgo, C-walljump-letgo-space), F8 9, the ceiling's pull 1 (C-ceiling), O1 1
(C-unmorph-on-bounce), O2 1 (C-morph-ceiling); adversary 97's two survivors are killed now, the
take-off keeping a dash while not dashing (C-takeoff-loses-dash) and facing the screen reporting
0 (C-screen-kept-speed); D1 left out (the start not dashing) fails the kept dashes, as adversary
96 found for its 25. Over the cache, the amendment's rules give the original's value on its
frames, where the landed model gives none: 15 of 15 change-of-shape landings not heading down, 4
of 4 let-goes into the screw attack's spin keeping the speed and mode, 1 of 1 wall-jump rise
ended; no frame has O1's or O2's case. **The landed model** (no D1), with the new key stripped,
fails 12 of 326: C-dash-lost-not-dashing, C-elevator-jump, C-elevator-side, C-morph-ceiling,
C-morph-land-run, C-takeoff-loses-dash, C-unmorph-land-run, C-unmorph-on-bounce,
C-walljump-letgo-plain, C-walljump-letgo-space, R-morph-land and R-walljump-letgo.

**Units.** `tools spec-units` with these notes: 229.0 → 235.3 (motion 92.0 → 95.0:
`RisingAndFalling` 8.7 → 10.7 for the change of shape's landing and the vertical speed after her
moves, the wall jump joining an existing arm; `RidingTheElevator` 1.0 → 2.0 for the pose kept on
the ride; `Dash` unchanged, its arm now read from dashing; pose chart 60.0 → 61.0, `PadChart` 35.0
→ 36.0 for the wall jump's let-go with the screw attack or space jump; with adversary 97's fixes,
pose fit 17.3 → 18.3, `ShrinkingLowersHer` 2.3 → 3.3 for the change of shape from a bouncing
ball, and collision 59.7 → 61.0, `WallsAndCeilings` 3.0 → 4.3 for a change of shape's rise into
a ceiling and its pulled speed): 229.0 → 235.3 in all. The budget is 8,244 → 8,472: motion 3,312
→ 3,420, pose fit 624 → 660, pose chart 2,160 → 2,196, collision 2,148 → 2,196.

**The gaps file** keeps gap 36 alone; gap 38 and the unmasked wall-jump frame are answered here.

## Ceres's free run (2026-11-12): why samus-movement holds the least of rungs M0 to M2

The domain free run held 106 of chain-any-0003-DFD7's 344 frames (M0), and 1,779 of the 6,647
frames of the eleven Ceres rows (`every_domain_runs_free_against_the_cache`, CI's lean cache, at
`16bc904e6`). The per-frame harness compares 2,752 of those frames and matches 2,728. Read with
`--nocapture`, then with three scratch runs of the free run (never committed): her pose fed from
the records, then the engine's pose change on the model's chosen pose (`samus_movement_bridge::
step::next`, `pose::change`, `samus_appearance_bridge::animate`), with and without the animation
fed. Records read through a scratch dump of `0AF6`..`0B4A`, `008B`, `008F`, `0A1C`, `0A2E`,
`0B00`, `0AFE` and the harness's enemy list; bank logs `$90` and `$91`.

**The main cause is outside the spec: the free run never moves her pose.** `Movement::step` runs
the model and `settle`, and nothing writes the pose the model chose back into the carried State,
so the carried pose, its facing, movement type, radius, history and animation stay those of the
seed. The next frame's moves then read the wrong pose: chain-any-0004-E021 alternates 0Fh and 11h
every frame (the aim buttons), and every segment ends one frame after its seed, x a pixel off.
With the pose and its animation carried as the engine carries them, the free run holds every
compared frame of chain-any-0002 to 0005 and 0007 to 0010, and 182 of 0003's 344 (M0's row):

| Row | held, now | held, pose carried | compared (per frame) |
| --- | ---: | ---: | ---: |
| chain-any-0001-DF45 | 84 | 148 | 149 |
| chain-any-0002-DF8D | 66 | 120 | 120 |
| chain-any-0003-DFD7 | 106 | 182 | 182 |
| chain-any-0004-E021 | 53 | 102 | 102 |
| chain-any-0005-E06B | 57 | 94 | 94 |
| chain-any-0006-E0B5 | 1,068 | 1,469 | 1,491 |
| chain-any-0007-E06B | 60 | 96 | 96 |
| chain-any-0008-E021 | 52 | 100 | 100 |
| chain-any-0009-DFD7 | 96 | 163 | 163 |
| chain-any-0010-DF8D | 52 | 91 | 91 |
| chain-any-0011-DF45 | 85 | 159 | 164 |

The integrator's part: the free run's step applies the frame's pose change to the carried State
(the model's chosen pose through the bridge's `next`, the engine's `pose::change`) and takes her
animation either from samus-appearance's model or from record N+1 (a neighbour's answer); both
gave the same counts here. `pose` is spec §1's quantity "Read by every behaviour; output", and
no harness compares it, so the per-frame harness hides the same fault: it decodes the pose from
every record.

**The remaining divergences, in the domain, each with its cause:**

| Row, frame | Field | Cause | Resolution |
| --- | --- | --- | --- |
| 0001: 102 | x, mode | facing the screen, right held: the original turns her (`$91:A0DE`: right held → 26h, left held → 25h, nothing else), a skid of zero (`$91:F8D3` from 0 or 9Bh goes straight to the speed join and `0B4A` = 1); the model ran on, facing the screen | `PadChart`: the facing-the-screen rows; CE2, C-screen-turn, C-screen-jump |
| 0011: 4 | y, mode, placed | a spin's skid braked to exactly 0 (`$90:9B1F`: 0.8000 − 0.8000 is not negative, so `0B4A` stays 1) moves 0 by `$90:8EA9`, whose way is `0B4A` and the facing, not the speed: rightward. `$90:93B1` meets the solid enemy at (224, 639) touching her right side, which clears `0AFC` and halts her (`0B4A` 0). The model moved the 0 along her facing | `SpeedBuildsAndBrakes`, `SolidEnemies`: a skid braked to 0 moves the old way; CE1, C-skid-zero-enemy |
| 0006: 523 | (pose) | moonwalking with the way she walks held: `$91:A8AC`/`$91:A874` give 25h/26h, the turn facing the way she walks, as the moonwalk's jump keeps it; the model faced back. Shows in the free run once the pose is carried | `PadChart`'s moonwalk row, the `Turning` cell's facing, `TurningRound`; CE3, C-moonwalk-turn |
| 0006: 535, 541, 547, ..., 601 (12 frames, every 6th) | mode | a crouched turn skids like every turn: `$91:F8D3` (movement Eh, 43h/44h) and `$91:F952` (17h, the aimed crouched turns 97h-9Ah, A2h, A3h) both end with the dash joined and `0B4A` = 1; the chart's crouched row said no effect, so the model follows the pad and creeps a frame early | `PadChart`'s crouched row, `TurningRound`; CE4, C-crouch-turn-skid |
| 0006: 10, 259, 261, 267, 273, 279; 0011: 38, 91, 95, 144 | mode | a spin's skid that braking ends (`$90:9B1F`: below zero, speed 0, `0B4A` 0) runs on in `$90:9040` that frame: the faced side held (`$90:9076`..`$90:9091`) sets `0B4A` = 2 at `$90:90AA`. Unperceived: a spin decides afresh each frame (the same test), and every pose a spin changes into sets the mode again (`$91:F543`, `$91:F60D`, `$91:F624`, the landing's and the wall jump's), and the scroll reads only `0B4A` = 1 (`$90:95D4`) | a mask, below |
| 0006: 539, 541 | (pose) | gap 37: the crouched turn aimed down-forward hands over the crouch facing the old way | open |

**The mask the harness adds.** "unperceived: a spin's skid ended by braking, carrying on or
following the pad, which its next frame decides afresh": `mode` alone, no writer
(`Hides::NoWriter`), on a frame that starts in a spin (19h, 1Ah, 1Bh, 1Ch, 81h, 82h) with
`0B4A` = 1 and ends in a spin with `0B4A` = 2 and `0B46`.`0B48` = 0. Over the Ceres rows it
hides exactly the 6 frames of chain-any-0006 and the 4 of chain-any-0011 above.

**Outside the domain** (skipped frames; each ends a segment as skipped):

| Frames | Why the domain doesn't run | Owner |
| --- | --- | --- |
| about 161 per row; 1,689 in all (game states 9h and Bh) | the door's passage: the room fades, scrolls and loads | R1 (doors and elevators), R2 (room loading) |
| chain-any-0001: 101 (handler `$90:E8DC`, locked) | the elevator's descent into Ceres, she faces the screen and is locked | R1 (doors and elevators) |
| chain-any-0011: 1,959 (game states 20h, 21h, 22h) | Ceres's escape ends: the elevator up and the station's explosion | B8 (Ceres Ridley and the escape), F4 (cinematics) |
| chain-any-0006, 0009, 0010: 26 knockback frames, 115 frames of 50h (the damage boost), 3 of 53h/54h, and 7 knockback starts masked | hits (Ridley's fight) | S4 (hurt and knockback) |
| chain-any-0006: 2 frames of handler `$90:E90E` | her frame handler runs no movement (not traced) | B8 (Ridley's fight), to confirm |

So, with the pose carried, M0's row holds 182 of its 344 frames, every frame the domain runs in;
the other 162 are the door's (R1).

**Units.** `tools spec-units` with these notes: 235.3 → 239.7 (motion 95.0 → 96.0,
`SpeedBuildsAndBrakes` 11.0 → 12.0 for a skid braked to 0 moving the old way; pose chart 61.0 →
63.3, `PadChart` 36.0 → 38.3 for the facing-the-screen rows, the moonwalk's turn and the crouched
turn's effect; collision 61.0 → 62.0, `SolidEnemies` 9.0 → 10.0 for a move of 0 touching an
enemy). The budget is 8,823 → 8,987: motion 3,562 → 3,600, pose chart 2,287 → 2,375, collision
2,287 → 2,325.

**Vectors.** 9 new, 389 → 398: recorded CE1 to CE4, constructed C-skid-zero-enemy,
C-screen-turn, C-moonwalk-turn and C-crouch-turn-skid, and the pose step C-screen-jump. The landed
model fails 7 of them, each as its amendment says (CE1, C-skid-zero-enemy, CE2, CE3,
C-moonwalk-turn, CE4, C-crouch-turn-skid) and the pose step C-screen-jump (it takes off); it
passes C-screen-turn, where the standing rows already turn her. The observer's reference for the
amendments (`$TEMP/collab/sm-ceres/ref_ceres.py`) agrees with all 8 new motion vectors.

## Size baseline (tokens, 2026-11-07)

Restated at c7f927ab6 with declarations apart (Jim, 2026-11-07: types and the bridge are not
counted against the model). Old code, the files below: logic 13,342, types 1,781, data 153. The
round-7 model: logic 9,265, types 2,090, data 7,386. The bridge (`game/src/samus_movement_bridge*`, the
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
| `types` | 1,764 | 3 |
| `clean/samus_movement` (catalogue, chart and its rows, collide, lib, rules) | 12,320 | 3,369 |
| `game/src/samus_movement_bridge*` (the world, the frame, the pose map) | 2,250 | 3,701 |
| Total | 16,334 | 7,073 |

The pose map (`samus_movement_bridge/pose.rs`, 3,610 of data) is which sprite shows each pose: the drawing
domain's data, held in the bridge until the engine draws from the pose itself.

## Directional travel outputs

`TravelledDistances` supplies W1 `InheritedMotion` with the last collision-returned signed
additive travel for each direction. The quantity is not the magnitude of a left/up move.

| Quantity | RAM | Decode and encode |
| --- | --- | --- |
| left travel | `0DAA`.`0DAC` | join whole/fraction as a wrapping signed 32-bit quantity |
| right travel | `0DAE`.`0DB0` | join whole/fraction as a wrapping signed 32-bit quantity |
| up travel | `0DB2`.`0DB4` | join whole/fraction as a wrapping signed 32-bit quantity |
| down travel | `0DB6`.`0DB8` | join whole/fraction as a wrapping signed 32-bit quantity |

Two words represent one measured distance, so none adds a one-word RAM-shape item. For example,
left/up travel `-0.4000` encodes FFFF.C000, not 0000.4000. Negation borrows between the two words.

The writers after the block sweeps copy `$12`.`$14` without taking its magnitude: left
`$90:9393`..`$90:939A`, right `$90:93CE`..`$90:93D5`, up `$90:9431`..`$90:9438`, down
`$90:945F`..`$90:9466`. Enemy detection returns a nonnegative whole distance and zero fraction
on collision (`$A0:A8F0`..`$A0:AB44`); the direct movement helpers apply that distance and store
it, negating it for left/up (`$90:9842`..`$90:9870`, `$90:988D`..`$90:98BB`). Right/down store
it unchanged (`$90:9826`..`$90:9841`, `$90:9871`..`$90:988C`). They assign, never accumulate.
An accepted zero still replaces the previous value in that direction.

Flat stops `$94:8F49`..`$94:8FBA` and stepped stops `$94:8D2B`..`$94:8EF7` clear `$14`, clamp
`$12` in its signed direction, and independently snap the movement-axis subposition. Thus a
right/down stop may put her at fraction FFFF but reports fraction zero; a left/up stop reports
negative whole travel and fraction zero even if her old position had a fraction. Enemy contact
can clear y subposition independently (`$A0:AAC8`), including on a horizontal move. This is why
subtracting centres, even before/after a single collision, is not the receipt. A contact one
pixel beyond the nominal whole reach can report that integer distance as `SolidEnemies` says:
W13's downward distance is 2.0000 although its start speed is 1.DC00.

A non-square slope walk changes `$12`.`$14` before the ordinary horizontal writer
(`$94:84D6`..`$94:8585`). Its fractional result is reported. Non-square vertical stops
`$94:876D`..`$94:877A` and `$94:87E4`..`$94:87F1` set fraction zero; floor correction and
ceiling cancellation give the whole travel specified by `SlopesCarry`. The separate alignment
`$94:87F4`..`$94:8919` writes position, not these distances. The near-wall look and pose-fit/
shrink tests likewise never reach these writers. The aim-change step `$91:EADE`..`$91:EB6F`
uses `$94:971E` directly, without the ordinary directional writer: neither a successful aim
step nor its blocked result replaces the earlier ordinary travel. Pose-fit pushes, shrinking
and the take-off lift also leave it alone.

### Lifetime and the integration boundary

`$90:EB05`..`$90:EB1A` clears all eight words. Normal PRE `$90:E695` and its `$90:E6C9` variant
call W1 first, then this clear; `$90:E713` runs existing shots before the same clear. Normal POST
`$90:E725` supplies movement after that. A skipped clear retains the words, and a skipped move
without a clear retains each unmodified direction. Do not infer either operation from lag,
a position change, a stored handler pointer or the final speed. The surrounding caller decides
which interval starts; movement itself must expose its accepted fractional results.

The source boundary watch in `$TEMP/collab/night-20261113/w1-observer/input-boundary-check.py`
and its log establishes preceding-cut inputs at the cannon: five active transitions match
all watched inputs, including the distances, and the spawn boundary reads cut 156. The full
instrumented replay matches all 345 cached cuts. `D-inherited-cut` uses only RAM observations
from r10 155 and 156: the right distance changes from 4.6000 to 3.E000, as a braking run loses
0.8000, and the other distances are zero. W1 consumes 3 whole / 57344 fraction before the
interval clear. Its own `InheritedMotion` deliberately combines bytes from different named
distance parts; this amendment preserves all parts and does not change that arithmetic.

`samus_movement::Outcome::travelled`, available through bridge `Ran::outcome`, reports the
complete fresh signed accepted travels, including their fractions. The engine keeps the
caller-held `TravelledDistances` in `State::movement_travel`, a pinned canonical model State.
Normal and elevator-carried PRE run the projectile pass before refreshing it; normal POST
stores the fresh output. The decoder reads all four paired signed quantities directly.
The named harness witnesses R2, R61, R144, W11, W13, W20 and the inherited cut compare all four
against their recorded end cuts with the same comparator used for a zero-travel null.
This is a bounded output proof: gap 38 names the missing receipt-write information needed for
movement under a skipped PRE. Weapon inheritance and broader caller variants remain unwired.
`after.travelled` checks the fresh normal-step results. `distance_history` checks only isolated
caller lifetime and selection of measured travels; it is not a replacement collision harness.

### Reference and proof boundary

The private Python reference is `$TEMP/collab/night-20261113/s1-vector-fix/reference.py`. It
recomputes the new accepted travels from the existing movement rules and invented geometry,
then independently compares the six recorded movement cases with their cached RAM. The
seventh recorded case is the boundary-watched inherited distance. It agrees with 26/26
movement expectation deltas and 7/7 lifetime histories; recorded distance agreement is 7/7
against a zero-distance null model's 0/7. The reference preserves
every original observation in 345 movement vectors and 64 pose steps, and the unchanged
landed-model baseline suite passes 15 tests. Its focused reference does not claim a new
movement free-run or wider weapon integration. Those need the next confined model and adapter.
The mutation probes for net displacement, fractional snapping, unsigned lengths, aim-step
replacement and accumulation each fail the focused reference. This proof covers the bounded
output delta, not all cached transitions; source-pinned interval guards and the next model
are required before a distance free-run score can be claimed.
The registered vector decoder accepts `after.travelled` and `distance_history`, requires each
directional expectation to name all four directions, and compares the model's movement outputs
and isolated caller histories separately.

**Units.** The official counter adds six arms: 239.7 to 245.7 units, with collision
62.0 to 68.0 and the other areas unchanged. Its 423 vector ids comprise 352 movement vectors,
64 pose steps and seven distance histories. The 19 existing cases given new expectations and
seven additional movement cases contain no changes to the original observations.

### Leftward travel beside a block

`C-distance-left-block` starts falling left at speed 2.0000 and x 0106.8000.
`SpeedBuildsAndBrakes` gains 0.C000 in a fall, and `WordwiseTopSpeed` replaces
2.C000 with the falling cap 1.0000. Source `$90:9B1F`..`$90:9B50` performs
that gain and cap; the falling row is selected by `$90:9BD1`, as in the existing
speed table's decode. With no dash, the left move is -1.0000. Its target centre
is 0105.8000 and leading pixel is 0100 (256), outside the invented solid block
whose horizontal extent is 240..255. `$94:9543` sees the leading column at 256,
so `$94:8F6B`'s stop and fraction clear are not reached; `$94:974B` adds the
unstopped move, retaining fraction 8000. The observed distance is -1.0000 and
position is 0105.8000. A block nearby does not imply collision.

`C-distance-left-block-stopped` keeps the same geometry and adds a retained dash
of 1.0000. The capped speed remains 1.0000, so its left request is -2.0000.
Its target leading pixel is 255, inside the block. `$94:8F6B`..`$94:8F81` accepts
signed whole travel -1, clears both the accepted fraction and position fraction,
and `$94:9730` applies that travel. Its distance is also -1.0000, but its position
is 0105.0000. This pair checks the difference between an unstopped whole-pixel
move and a blocked move with the same accepted travel.

The private reference is `$TEMP/collab/night-20261113/s1-vector-fix/reference.py`.
It derives the gain and cap before collision and checks position.x for both
constructed cases. Every other observation of the amended spec is preserved.
The focused proof has 26 movement distance checks and seven lifetime histories;
352 movement vectors, 64 pose steps and seven histories give 423 vector ids.
The spec's six distance arms and its 245.7 units are unchanged. Setting the unstopped case's expected fraction to 0000 fails this position check,
even though its distance remains -1.0000.

### Vocabulary review

The following exact phrase additions to `scan_allow.txt` resolve inherited false positives.
Each is player vocabulary or existing recorded provenance, not an original routine name.
They add no single-word or broad-stem exemption. Their lexical scope is the exact normalised
phrase, so the adversary must review these additions before export.

| Phrase | Existing context | Why it is player vocabulary |
| --- | --- | --- |
| slope solid | header: blocks, slopes and solid enemies | two adjacent obstacles, coordinated in prose |
| pose turn | `TurningRound`: samus-input-poses turns | an existing record row followed by the visible turns it covers |
| change pose | `MovesAsShePosed`: she changes pose | a visible change of body and action |
| new pose | `PoseFitAfterMoves`: new pose's body and speeds | the visible pose chosen this frame |
| above below | `PoseFitAfterMoves`: what is above or below her | spatial relations of the obstacles around her |
| grapple walljump | samus-grapple-walljump, in the spec and eight vector sources | an existing recorded row id naming player actions; its identity stays intact |
| no block | `SolidEnemies`: an enemy decides the move and no block | the consequence of hitting a solid enemy |
| block below | `SlopesSlowTheWalk`: a solid block below a column | the obstacle underneath a surface |
| air spikes | `OtherBlockKinds` and vector kind `AirWithSpikes` | passable cells carrying a visible hazard |
| block shots | `OtherBlockKinds`: a block her shots break | the player's way to break it |
| block grapple | `OtherBlockKinds`: a block her grapple holds | a visible grapple anchor |
| no solid | vector header: a slope column has no solid pixel | the column's measurable shape |
| frame pose | vector header: a lead-in frame's pose | the visible pose in that frame |

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

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen: the old code's
score under MODEL §4's weights, 13,169.75 (`tools size` over the old code at `c7f927ab6` and its
two cuts), per unit of this spec's official count, 193.3 (`tools spec-units` at `d9ee05f7c`),
times 0.55 is 37.47, which is 37.5 to one decimal. An area's budget is k × its units; its old code
(`old`, the score of the old files that area replaces at `c7f927ab6`, measured before lookups were
data, duplication was priced and visibility and impl headers were free) is for reference and
doesn't bound it. The budget is an
indicator: the fit is score ÷ budget, and being over it costs in proportion to how far over, never
as a wall; the old-code comparison is a guideline assessed under MODEL §1 decision 19.

- Area motion (old 4,019): SpeedBuildsAndBrakes, Dash, Dashing, TurningRound, TransitionFiringAim, AimingAndShooting,
  RidingTheElevator, Jumping, FollowingTheGround, MovesAsShePosed, RisingAndFalling,
  DriftInTheAir, Moonfall, WallJump, Landing, WordwiseTopSpeed, 1.2, 9
- Area pose fit (old 1,549): PoseFitAfterMoves, ShrinkingLowersHer
- Area pose chart (old 2,263): HowSheIsDrawn, PoseFollows, PadChart, Aiming, WhatHerMovesMet,
  AnimationEnds
- Area collision (old 4,893): TravelledDistances, BlocksStopFlush, WallsAndCeilings, SolidEnemies, SlopesCarry,
  SlopesSlowTheWalk, TouchedCells, OtherBlockKinds

Budget: 37.5 × 245.7 units = 9,212 (motion 3,600, pose fit 687, pose chart 2,375, collision 2,550)

From the spec's §7 (the pose chart), moved with the header's:

**Size.** Inside the header's 1,500 tokens of logic for the chart: the chart's reader and the
effects. The tables below are data records (about 120 rows), counted as data.

Current official count: 248.7 units over 429 vector ids; budget 9,324 weighted tokens
(motion 3,712, pose fit 687, pose chart 2,375, collision 2,550).

## Transition firing aim

`TransitionFiringAim` sources: $91:F404-F422 only initializes a changed ordinary pose;
$91:F433/F468-F4A1 initializes its named family and repeats ground-turn initialization when
another turn becomes ground turning. $91:F543-F599 diverts a straight jump into shinespark;
$91:F5CF-F5E6 otherwise publishes the new pose firing direction on newly pressed shoot.
$91:F8D3-F903 publishes the old direction only when entering a ground turn from moonwalking;
$91:F952-F989 can turn the moonwalk jump into that ground turn. Other families do not write it.

Mapping: pending aim is $0B5E. Jump publication carries a high marker, moonwalk publication a
different marker; only the low heading byte affects W1's $90:BA5F-BA67 heading choice. Decode
these as optional named Heading, preserving the source encoding at an observed edge. Pose
heading is content from $91:B62C in each pose descriptor; never a raw code in the clean model.
The fresh output means publication/no publication, not replacement by absent pending state.

Caller: $90:EB02-EB20 clears pending after ordinary/elevator/locked PRE. POST pose application
then publishes for the next PRE. Drawing $90:EB35-EB42/EAB3-EAD4 writes prior newly pressed
controls; demo restoration $90:E82D/E864 restores them. W1 remains their consumer.

Fresh M0 observation over 345 recovered records: pending only81; prior-new SHOOT only81/157.
80 PRE spin suppresses held/new shoot, POST enters straight jump and publishes;81 released
shoot makes no attempt but PRE clears pending, so82 has none.156 ordinary standing fires,
157 has count1/cooldown15 and prior-new shoot. Existing decode's private boundary oracle
matched345/345 cuts; that replay is prior evidence, not a fresh replay of this amendment.

Harness: compare publication using the independent preceding pose/controls and the actual
pose change outcome, never use following pending as an input. Only the ordinary pose path is
claimed. For a skipped PRE, no publication must preserve caller-held pending rather than clear
it. Equipment/cinematic forced pose initializations and shinespark are outside this amendment.
The constructed vectors reuse complete movement frames and add a publication expectation;
pose headings are invented typed content. C-aim-jump-press differs from its held/released cases
only in shoot press; both moonwalk variants use the old content heading, not the new pose.

AF3 `import::projectile` already imports `MuzzleAim::Aimed(Heading)`, `PreviousAim` and
`Forbidden`. All 22 source straight-jump descriptors and six moonwalking descriptors are Aimed;
the producer reads those semantic headings, and leaves other descriptor kinds distinct for W1.
No second ROM-content extraction is required. C-aim-same-pose is R126's full movement case with
a publication expectation: a straight jump already aimed down-forward keeps its pose on a new
shoot press and publishes none. Shinespark handover is not represented in this movement model;
it remains an explicitly excluded external pose boundary, not a guessed supported vector.
