# Samus's motion: decode notes

For the observer and the domain harness's decoder only. Implementers don't read this file (MODEL §5): it maps the spec's quantities to the original's RAM.


ROM codes and RAM, for `verify`'s decode only. Record N gives the State and the inputs of frame
N+1; the harness compares the State with record N+1.

**State.**

| Quantity | RAM | Decode |
| --- | --- | --- |
| position | x `0AF6`.`0AF8`, y `0AFA`.`0AFC` | as is |
| base_speed | `0B46`.`0B48` | as is |
| extra_run | `0B42`.`0B44` | as is |
| momentum | `0B3C` | ≠ 0 |
| mode | `0B4A` | 0 Accelerating, 1 TurningAround, 2 Decelerating |
| vertical_speed | `0B2E`.`0B2C` | as is |
| vertical | `0B36` | 0 None, 1 Up, 2 Down |
| bounce | `0B20` low byte | 0 None, 1 First, 2 Second (the high byte, 6, marks the spring ball; nothing here reads it) |

**Inputs.**
- locomotion: movement type `0A1F` and pose `0A1C`. Ground: type 0 `FacingForward` (00, 9B),
  `Standing { plain }` (01, 02), else `Standing`; 1 Running; 4 Ball (at rest 1D, 41); 5 Crouching;
  E TurningOnGround; F Transition; 10 Moonwalking; 11 Ball spring (at rest 79, 7A); 15
  RanIntoWall. Air: 2 `Jumping` (`take_off` for 4B, 4C, 55-5A); 3 SpinJumping; 6 Falling; 8
  `BallInAir { spring: false, falling: true }`; 12 spring, not falling; 13 spring, falling; 14
  WallJumping; 17, 18 TurningInAir. Other types: the frame is not in the domain.
- facing: `0A1E` low byte: 4 Left, 8 Right, 0 Forward. radius: `0AFE`, `0B00` of record N+1 (the
  record lags the pose by a frame).
- pad: held `8B`, newly pressed `8F`; bindings: jump `09B4`, run `09B6`, shoot `09B2`.
- equipment `09A2`: 0100 hi-jump, 0200 space jump.
- was_spinning: `0A27` (two-updates-back movement type) is 3 or 14. spin_ready: animation frame
  `0A96` ≥ 0B (≥ 1B in the screw-attack poses 81, 82).
- pose_request, PoseStart, PoseCommand: derived from records N and N+1 (the pose change, and the
  command `0A2E`) until pose selection is its own domain.
- elevator_moving; on_slope (`0DBA` ≠ 0 or the square-slope flag); submerged is false.
- vertical result `0DC6`: 0 None, 1 Landed, 2 Falling, 4 HitCeiling, 5 WallJump; kind `0DC7`.

**The domain harness** (`verify::harness::samus_motion`) decodes as above, with these choices:
- elevator_moving: elevator status `0E18` ≠ 0. on_slope: `0DBA` ≠ 0 only; the square-slope
  flag isn't decoded.
- The new pose of a frame is record N+1's pose when it differs from record N's. Its start: a
  turn type (E, 17, 18) after none: `Turn` (replaced); ball to ball with the X direction
  changed: `BallReverse`; type 3 after 3 or 14: `SpinReverse` if the direction changed; type 3
  otherwise: `SpinStart`; a take-off pose after none: `TakeOff` (`from_plain_crouch` after
  poses 27, 28); type 2 or 6 otherwise: `JumpOrFall`; 12 after 11: `SpringLaunch`; 14 after
  another: `WallJumpStart`. `started_falling`: type 6 after another type.
- Its command: `Vertical(result)` when the model's result is passed (not None, not a NoChange
  kind); else record N+1's prospective pose change command `0A2E`: 1 Decelerate, 2 Stop, 6
  KillXSpeed, 8 KillRunSpeed; 0, 3, 4, 5 and 7 give None. With no new pose, `pose_writes` runs
  only for a passed result. `arm_pump_step`'s request is `Pose { running: type 1 }` for a new
  pose, else None.
- **The proposal.** The pose update takes the first set (bit 15 clear) of record N+1's super
  special `0A2C`, special `0A2A` and prospective `0A28` poses (`$91:EB88`). A super special or
  special proposal is `Forced` for the arm-pump step (`$91:EADE` runs only for `0A28`) and gives
  command None: its own commands (`0A32`, `0A30`) aren't the prospective `0A2E`. A new pose
  that differs from the proposal was replaced (`$91:F404`'s carry), so its command is skipped
  (G10).
- **`AnimationEnd` (G12):** a super special proposal with `0A32` = 3 ("transition animation
  finished", `$91:F36E`) gives `PoseCommand::AnimationEnd`, ahead of a passed vertical result,
  with the start decoded as above and the arm-pump step `Forced`. chain-0014-96BA:11,
  samus-input-poses:125 and chain-0232-D5A7:24 match.
- **A proposed pose equal to hers.** Record N+1's prospective pose `0A28` with bit 15 clear is a
  new pose even when it equals the current one: `$91:EBE7` stores it and `$91:F404` finds no
  change, so nothing starts (`start: None`), but its command `0A2E` runs and the arm-pump step
  (`$91:EADE`) reads it. Before this, Decelerate, Stop and KillRunSpeed without a pose change
  were lost (area-maridia:192-207, chain-0012-91F8:858-864, chain-0097-ACF0:552-559, ...).
- **The bridge** (`Replay`, the harness's collision): each move is answered on its axis from
  record N+1's position. A move landing on the target's pixel is unblocked. A move the target
  falls short of in the asked direction, or lies behind, is stopped by a `Block` whose subpixel
  is the target's and whose distance is the whole-pixel difference. A move the target lies
  beyond is made unblocked, so a wrong speed shows. A zero move is never stopped.
  `enemy_ahead` finds nothing. `probe_wall` finds a wall when record N+1 starts a wall jump
  (distance 0, so it fires) or her X subpixel at N+1 differs from the body's (distance = reach;
  the subpixel is written, `WallProbeSubpixel`). Its `touching` (G8): the probe fires, the wall
  jump check's enemy `0E1C` at N+1 isn't FFFF (`$90:9D35` writes FFFF for a block) and y's
  subpixel at N+1 is 0. No recorded frame reaches it.
- **The slope seat** (G11): an enemy-testing horizontal move has `seat` when record N's
  movement type is an air one, the slope flag `0DBA` at N+1 is not 0 and her vertical direction
  is the same, and not None, at N and N+1 (no landing, no ceiling): the whole-pixel part of y at
  N+1 less that of y at N moved by her vertical speed at N. On the ground, and on a landing,
  the vertical move's answer already reaches the target. samus-walljump-16/22/28:44 match.

**Frames the harness skips.** The harness skips whole frames (it has no per-field mask):
- record N not in gameplay (`0998` ≠ 8), its cut not a requested NMI (a lag frame), or the
  main strand at N waiting inside the gameplay pass, a PLM or the message box (the strand
  decode; this settles open question 5: item-morph-ball's 399 message-box frames);
- the elevator moving (`0E18` ≠ 0): the elevator writes y beyond `ElevatorRide`'s step (the
  first 453 frames of both rows; every other field matched on them before this skip);
- the y radius changing between records N and N+1 (pose fit), or pose fit in the frame itself:
  the pose changes to one whose Y radius (pose definitions, `$91:B62F`) is larger, her
  vertical direction is the same at N and N+1 and not None, and y at N+1 isn't y at N moved by
  her vertical speed (G13: `$91:FDAE` runs before the record's radius changes);
- Samus's handlers at record N not running this domain's motion: the new state handler `0A44`
  other than E725 (normal), E8AA (Ceres) or E8EC (elevator), each of which runs the movement
  handler; or the movement handler `0A58` other than A337 (normal). E8DC (locked: the gunship
  landing, chain-0012-91F8:1-470), E94F (x-ray, time frozen) and E90E (none) are "runs no
  motion"; 946E grapple release, DF38 knockback, E025-E066 bomb jump, D068-D40D shinespark and
  D678-D75B crystal flash are named deferred behaviours.
- A deferred behaviour (spec §10), counted by name:
  - a block-inside push: record N+1's extra displacement `0B56`-`0B5C`, decel multiplier `0B4C`
    or quicksand flag `1E71` ≠ 0 (set at the frame's start by `$94:9B60`, before the motion, and
    kept to the next frame's start; terrain-quicksand);
  - liquids: her bottom (y + y radius − 1) below the water's surface `195E` (≥ 0, physics on:
    `197E` bit 2 clear) or, without water, the lava/acid surface `1962`, at N or N+1, without the
    gravity suit (`09A2` 0020), as `$90:9741` and `$90:98BC` test it;
  - the speed booster (`09A2` 2000): extra_run's pixel ≥ 2 at N (its cap is 7.0, `$90:9789`), or
    extra_run ≠ 0 at N or N+1 on a frame she rises faster than before (`$90:9919` adds it to
    the jump; running, it grows by 0.1000 in the same frame: chain-0106-AD5E:254);
  - a knockback start: movement type 0A at N+1 and not at N;
  - the movement types of deferred behaviours at N (G9): 0A knockback, 16 grapple, 19 damage
    boost, 1B shinespark.
- Open: terrain-quicksand:13, y 1 px short (model 0049, oracle 004A), the same shape as the
  seat frames; the seat rule doesn't answer it. Not yet traced.
- Open: item-morph-ball:1311, the crouch take-off: the model's y is 10 px above the oracle's.
  The model applies `crouch_jump_lift` on the frame the pose changes in the record; the
  oracle's lift falls on a pose-fit frame the radius rule skips. The pose and the radius are a
  frame apart in the record: a decode question, not a model gap.

The notes from before the harness:
- Frames where Samus didn't run: item-morph-ball:818-1216 repeat one record while the pad changes
  (the item's message box). Test the game state, not record identity (open question 5).
- A frame whose y radius changes between records N and N+1 has a pose-fit shift in y
  (item-morph-ball:739, 1259, 1304, 1311, 1327): don't compare y there until pose fit is an input.

**RAM shape** (State fields decoded as one WRAM word copied as-is): none. position, base_speed,
extra_run and vertical_speed are each two words joined into one P.S quantity (the cap quirks need
both words); momentum is a word tested ≠ 0; mode, vertical and bounce are enums from codes (bounce
from the low byte only).

---

