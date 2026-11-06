# Samus's motion: behaviour spec

The observer's spec for MODEL §4, to the MODEL §5 template. Vectors:
[`scenarios/vectors/samus-motion.toml`](../../scenarios/vectors/samus-motion.toml) (V ground,
A air, P pose writes).

## Header

**Scope.** One domain, because one set of speeds has writers across all of it (Jim, 2026-11-06):
- the ground: standing, running, turning, crouching, moonwalking, the transitions, running into a
  wall, the morph ball and spring ball on the ground, the step down, the arm-pump step;
- the air: the normal jump, the spin jump, the fall, the wall jump, the ball in the air and its
  bounce, turning in the air, the space jump;
- pose selection's writes to the State: the pose-change commands, the vertical results (the
  landing stop, the bounce, the ceiling) and the writes a new pose makes when it starts (§7).

**Out of scope** (each an input where it feeds us): collision (an interface, §6; what is known in
§10), which pose comes next, pose fit (the y shift when the radius changes), animation, sounds,
liquids and the speed booster (inputs fixed false), knockback, the bomb jump, grapple, shinespark
and the outside pushes (§10).

**Baseline: about 1140 lines.** `crates/game/src/movement.rs` 516 + `movement/air.rs` 434 (`wc -l`
at `ccb075c`), plus pose's speed writes, about 190 (`pose.rs` lines 395-466 and 517-636, located
with `grep -n`, not read). Tests excluded.

**Target: 500 lines** for `game::model::samus` motion, excluding tests and `data` tables: rows and
their enums ~90, speeds and caps ~60, the vertical step ~50, the wall-jump check ~30, step down,
elevator and arm pump ~40, pose writes ~90, the collision interface ~40, contacts ~30, slack ~70.
The air reuses the ground's machinery (§4): one horizontal step, one cap, one table. Over 500 is a
review stop; over the baseline a strict failure (MODEL §4).

**Notation.** `P.S` is a 16-bit pixel word and a 16-bit subpixel word in hex: `2.C000` is 2.75 px.
Record N holds the state at the end of frame N; its pad is the one frame N+1 reads.

---

## 1. Quantities

### 1.1 State

| Quantity | Units, type | Read by |
| --- | --- | --- |
| `position` | x, y: P.S, her centre; moves wrap mod 2^32 | every move; output |
| `base_speed` | P.S ≥ 0, `Speed` | `BaseSpeed`, `HorizontalStep`, `NoInputStops`, `Decelerate` (§7) |
| `extra_run` | P.S ≥ 0, `Speed` | `ExtraRunSpeed`, `HorizontalStep`, `JumpOrFall` and `Fold` (§7) |
| `momentum` | `bool` | `ExtraRunSpeed` |
| `mode` | `SpeedMode { Accelerating, TurningAround, Decelerating }` | `BaseSpeed`, `HorizontalDirection`, `BallAtRest`, `NoInputStops`, `SpinJumpDrift`, `WallJumpDrift`, `BallDropStops` |
| `vertical_speed` | P.S ≥ 0, a magnitude, `Speed` | `VerticalStep`, `JumpRelease`, `StartedFalling`, `SpaceJump`, `BallBounce` |
| `vertical` | `VerticalMotion { None, Up, Down }` | `VerticalStep`, `JumpRelease`, `StartedFalling`, `SpaceJump`, the airborne rows (§4), `Vertical(Falling)` (§7) |
| `bounce` | `Bounce { None, First, Second }` | `BallBounce`, the ball-in-air rows |

**One State for every movement.** Nothing saves or restores a field when the movement changes; a
row that doesn't use a field leaves it. *Airborne* means `vertical != None`.

### 1.2 Inputs

| Input | Type | From | Read by |
| --- | --- | --- | --- |
| `locomotion` | `Locomotion` (§2), at the frame's start | pose | the rows |
| `facing` | `Facing { Left, Right, Forward }`; Forward counts as Right where a side is picked | pose | `HorizontalDirection`, `ArmPumpStep`, `SpinJumpDrift` |
| `radius` | x, y px | pose | every request (§6) |
| `pad` | held: run, shoot, jump, left, right; newly pressed: jump | controller | `ExtraRunSpeed`, `JumpRelease`, `NoInputStops`, `BallDropStops`, `SpinJumpDrift`, `WallJumpCheck`, `SpaceJump`, `SpringBallJump`, `StandingShotRestartsAnimation` |
| `pose_request` | `PoseRequest { Forced, None, Pose { running: bool } }`, settled (§8) | pose | `ArmPumpStep` |
| `spin_ready` | `bool`: the spin animation is at its wall-jump frames | animation | `WallJumpCheck` |
| `was_spinning` | `bool`: two pose updates back she was spin jumping or wall jumping | pose | `WallJumpCheck` |
| `equipment` | hi_jump, space_jump: `bool` | items | `Jump`, `SpaceJump` |
| `elevator_moving` | `bool` | elevator | `ElevatorRide` |
| `on_slope` | `bool` | collision | `StepDown` |
| `submerged`, `speed_booster` | `bool`, **fixed false** | liquid, items | the tables, `ExtraRunCap` |
| `PoseStart`, `PoseCommand` | §7 | pose | `pose_writes` |

### 1.3 Contacts (results of the frame, not State)

| Contact | Type | Read by |
| --- | --- | --- |
| `wall_stop` | `bool` | `ArmPumpStep`; pose |
| `vertical_result` | `VerticalResult { None, Landed(LandKind), Falling(FallKind), HitCeiling, WallJump }` | pose, which passes it back as `Vertical` (§7) |
| `ran_into_wall` | `bool`, from `arm_pump_step` | pose |
| `wall_jump_armed` | `bool` | animation |
| `restart_standing_animation` | `bool` | animation |

`FallKind { Airborne, MorphBall, SpringBall, NoChange }` and `LandKind { Grounded, MorphBall,
SpringBall, NoChange }` come from the frame-start locomotion (§2).

### 1.4 Not State (Read by: none)

The total speed (local to a frame), the X-speed-killed direction (it is `wall_stop`), the distances
moved, the stored vertical result (cleared every frame), the vertical acceleration (derived from
`submerged`), the "falling" flag (never read), and the divisor and pushes (§10).

---

## 2. Locomotion

| Variant | Poses (codes in §9) | Fall kind | Land kind |
| --- | --- | --- | --- |
| `FacingForward` | standing, facing the screen | Airborne | NoChange |
| `Standing { plain }` | other standing and landing poses; `plain`: no aim, not landing | Airborne | NoChange |
| `Running` | running | Airborne | NoChange |
| `Ball { spring, at_rest }` | the ball on the ground | MorphBall / SpringBall | NoChange |
| `Crouching` | crouching | Airborne | NoChange |
| `TurningOnGround` | the on-ground turn | NoChange | NoChange |
| `Transition` | the crouch, stand, morph and unmorph transitions | NoChange | NoChange |
| `Moonwalking` | moonwalking | Airborne | NoChange |
| `RanIntoWall` | ran into a wall | Airborne | NoChange |
| `Jumping { take_off }` | the normal jump; `take_off`: its first poses | NoChange | Grounded |
| `SpinJumping` | the spin, space and screw-attack jumps | NoChange | Grounded |
| `Falling` | falling | NoChange | Grounded |
| `BallInAir { spring, falling }` | the ball in the air; the morph ball is always `falling` | NoChange | MorphBall / SpringBall |
| `WallJumping` | the wall jump | NoChange | Grounded |
| `TurningInAir` | turning while jumping or falling | NoChange | NoChange |

---

## 3. Parameters

`SpeedTable` (`data::movement::NORMAL_SPEEDS`; decel is 0.8000 in every entry):

| Locomotion | accel | max |
| --- | --- | --- |
| Standing, FacingForward, Crouching, TurningOnGround, RanIntoWall, TurningInAir | 0.C000 | 0.0000 |
| Running | 0.3000 | 2.C000 |
| Ball | 0.C000 | 3.4000 |
| Transition, Jumping, BallInAir { falling: false } | 0.C000 | 1.4000 |
| SpinJumping, WallJumping | 0.C000 | 1.6000 |
| Falling, BallInAir { falling: true } | 0.C000 | 1.0000 |
| Moonwalking | 0.C000 | 0.8000 |

The coasting rows never read their entry; the ROM keeps them.

`Rules`, air: `gravity` 0.1C00; `fall_cap_pixel` 5; `jump` 4.E000 (hi-jump 6.0000); `wall_jump`
4.A000 (hi-jump 5.8000); `bounce` 1.0000; `wall_jump_reach` 8 px; `space_jump_window` [2.80,
5.00) in 8.8; `crouch_jump_lift` 10 px. Ground: extra-run step 0.1000 and cap 2.0000;
`step_down_extra`, `elevator_step` and `arm_pump_step` 1.0; `max_displacement` 15;
`standing_shot_timer` 0x10.

---

## 4. Motion rows

The row comes from (`locomotion`, `mode`, `bounce`) at the frame's start (`LocomotionAtFrameStart`).

```
enum Horizontal { Ground, Coast, None, Air { release: bool, run: bool, drift: Drift } }
enum Drift { StopWithoutInput { still_moves: bool }, Spin, WallJump }
enum VerticalStep { StepDown, AirOrStepDown, Elevator, Air, Nothing }
enum After { Keep, Stop { cancel: bool }, StopUnlessAirborne, EndTurn, EndTransition, NoVertical }
```

- **Horizontal.** `Ground`: `ExtraRunSpeed`, `BaseSpeed { brake: true }`, `HorizontalStep`.
  `Coast`: `HorizontalStep` with base taken as 0. `None`: nothing. `Air`: `ExtraRunSpeed` if
  `run`, `JumpRelease` if `release`, `BaseSpeed { brake: false }`, then the drift
  (`NoInputStops`, `SpinJumpDrift` or `WallJumpDrift`), then `HorizontalStep` unless the drift
  stopped it.
- **VerticalStep.** `StepDown`. `AirOrStepDown`: if airborne, `Air`, else `StepDown`. `Elevator`:
  `ElevatorRide`. `Air`: `StartedFalling`, then `VerticalStep`. `Nothing`.
- **After.** `Stop { cancel }`: `CancelSpeedBoost` if cancel; extra_run = base = 0; mode
  Accelerating. `StopUnlessAirborne`: `Stop { cancel: true }` unless airborne. `EndTurn`:
  `CancelSpeedBoost`, extra_run = 0, result None (`TurnIgnoresFall`). `EndTransition`:
  `TransitionLanding`, result None. `NoVertical`: result None.

| Row | Horizontal | VerticalStep | After |
| --- | --- | --- | --- |
| FacingForward | None | Elevator | NoVertical |
| Standing, RanIntoWall | Coast | StepDown | Stop { cancel: true } |
| Running, Moonwalking | Ground | StepDown | Keep |
| BallAtRest (Ball, at_rest, Accelerating) | Coast | AirOrStepDown | StopUnlessAirborne |
| Ball (otherwise) | Ground | AirOrStepDown | Keep |
| Crouching | Coast | StepDown | Stop { cancel: false } |
| TurningOnGround | Ground | StepDown | EndTurn |
| Transition | Coast | AirOrStepDown | EndTransition |
| Jumping { take_off } | `ExtraRunSpeed`, mode = Accelerating, Coast | Nothing | Keep |
| Jumping; BallInAir { spring, !falling } not bouncing | Air { release, run, StopWithoutInput } | `VerticalStep` only | Keep |
| WallJumping | Air { release, run, WallJump } | `VerticalStep` only | Keep |
| SpinJumping | `SpaceJump`; Air { release, run, Spin }; `WallJumpCheck` | `VerticalStep` unless the check fired | Keep |
| Falling | Air { run, StopWithoutInput } | Air | Keep |
| BallInAir { falling } | `BallDropStops`; Air { StopWithoutInput { still_moves } } | Air | Keep |
| BallInAir { !falling } bouncing | Air { StopWithoutInput { still_moves } } | Air | Keep |
| TurningInAir | Ground | AirOrStepDown | EndTurn |

The spring ball and the morph ball share their rows; `spring` changes only the kinds and
`SpringBallJump`.

---

## 5. Behaviours

All **Fixed** except `SpeedCap` (Toggle). For every entry:
- `below(a, b)` on 16-bit words: bit 15 of (a − b) mod 2^16 is set. 0x1000 is not below 0xC000;
  0xC000 is below 0x4000 and below 0; 0x8000 is below 0. Equal words are not below each other.
- `negative(w)`: bit 15 of w. Adding P.S carries the subpixel into the pixel, mod 2^16.

### LocomotionAtFrameStart
- Timing. On the frame she changes pose she still moves as her old pose did: row, table entry and
  kinds come from the frame-start locomotion. checked: oracle (V6, V9, V13, V14, A8).
- Source: `$90:A337`.

### SpeedTable
- Parameter. Each way of moving has its own accel, max and decel (§3); the entry comes from the
  frame-start locomotion. checked: oracle (V1-V6, V10, A1, A5). Source: `$90:9BD1`.

### BaseSpeed
- Mechanic. She speeds up by the row's accel to its max, and brakes by its decel.
- Rule: if mode is TurningAround, or `brake` and mode is Decelerating: base −= decel, and if
  `negative(base.pixel)`, base = 0 and mode = Accelerating. Otherwise base += accel, then
  `SpeedCap`, which reports `capped` when it snapped. So in the air Decelerating accelerates.
- checked: oracle (V1-V6, V10, A1, A5, A9, A10); original only (V15, V16).
- Source: `$90:9A7E`, `$90:9B1F`.

### SpeedCap
- Quirk, **Toggle**. Speed overshoots, or snaps up, to the max.
- `Wordwise` (vanilla): keep if `below(base.pixel, max.pixel)`; else if the pixels differ, base =
  max (`capped`); else keep if the subs are equal or `below(base.sub, max.sub)`, else base = max
  (`capped`). 2.1000 snaps up to 2.C000 (V3); 3.C000 is kept over 3.4000 (V5); 0.C000 over 0 is
  kept and 1.8000 snaps (V6); 1.C000 is kept over 1.4000 in the air (A10); 2.2000 snaps down to
  1.4000 (A5); 0.C000 over 0.8000 snaps (V18).
- `Exact`: base = min(base, max), 32-bit unsigned; `capped` when it lowered base.
- checked: oracle for `Wordwise` (V3, V5, V6, A1, A5, A10); `Exact` constructed (`-exact` vectors).
- Source: `$90:9A99`.

### TurnCreep
- Quirk, follows `SpeedCap`. Turning from standing slides 0.C000 every other frame along her
  facing (base 0, 0.C000, 0). checked: oracle (V6). Source: `$90:A67C`.

### ExtraRunSpeed
- Mechanic. Holding run while running adds speed, up to 2 px; momentum carries it elsewhere.
- Rule: running (`Running`, run held, not submerged): momentum = true, then `ExtraRunCap`.
  Otherwise extra_run = 0 unless momentum. checked: oracle (V1-V4, V8, A1); original only (V17).
- Source: `$90:973E`.

### ExtraRunCap
- Quirk (invisible in vanilla). Test the *old* extra_run: add the step if `below(pixel,
  cap.pixel)` or `below(sub, cap.sub)`; else extra_run = cap. So 2.F000 grows to 3.0000, then
  snaps to 2.0000. Doesn't follow `SpeedCap`. checked: original only (V11). Source: `$90:97D5`.

### CancelSpeedBoost
- Mechanic. momentum = false (the boost counter and palette are deferred). checked: oracle (V8,
  V14, V25, A8, A11). Source: `$91:DE53`.

### HorizontalStep
- Mechanic. total = (base, or 0 when coasting) + extra_run; d = ±total on the heading from
  `HorizontalDirection`, then `DisplacementClamp`; `move_horizontal` (§6) with `TestEnemies`. A
  zero total still asks, so a solid enemy on the heading stops her (V26-V26d).
- checked: oracle (V1-V10, V14, A1, A4-A11); original only (V26*). Source: `$90:8EA9`.

### HorizontalDirection
- Mechanic. Read after this frame's `BaseSpeed`. TurningAround: leftward when facing Right, else
  rightward. Otherwise leftward when facing Left, else rightward. checked: oracle (V6, A9);
  original only (V15, V15b). Source: `$90:8EA9`.

### DisplacementClamp
- Mechanic. A pixel word not negative and not below 16 becomes 15; a negative one below −15
  becomes −15; the sub is kept. Unreachable in vanilla motion. checked: original only (V12).
- Source: `$90:E4AD`.

### WallStop
- Mechanic. A stopped horizontal move, on the ground or in the air: `CancelSpeedBoost`, extra_run
  = base = 0, mode Accelerating, `wall_stop`. The step down still uses this frame's total.
  checked: oracle (V8, A11); original only (V20, V21). Source: `$90:E5CE`.

### StepDown
- Mechanic. `move_down` by 1.0 if `on_slope`, else by total + 1.0 (this frame's, from before
  `WallStop`). checked: oracle (V7, V10, V13); original only (V21). Source: `$90:923F`.

### BallAtRest
- Mechanic. The still ball coasts and stops, unless airborne; braking or turning, it takes the
  `Ball` row. checked: oracle (V10); original only (V16, V19b). Source: `$90:A521`.

### TurnIgnoresFall
- Quirk. `EndTurn` clears the result after the step down, so a turning Samus sinks by total + 1.0
  a frame onto lower ground instead of falling. checked: oracle (V7). Source: `$90:A68F`.

### TransitionLanding
- Mechanic. In `EndTransition`, a `Landed` result is `EndVertical` (§7); the speeds are kept.
  checked: oracle (V13, A16); original only (V13b). Source: `$90:A640`.

### LandingContact
- Mechanic. A down move: a stop is `Landed(land kind)`, a full move `Falling(fall kind)`, unless
  `WallJumpCheck` fired. An up move: a stop is `HitCeiling`, else None. checked: oracle (V9, V10,
  V25, A14, A16); original only (V19*, V21b). Source: `$90:E61B`, `$90:E606`.

### ElevatorRide, ElevatorHeadsUp
- Mechanic and quirk. If `elevator_moving`: `move_down` by 1.0 with heading `Up` and
  `SkipEnemies`; no speed change. checked: original only (V23). Source: `$90:A392`, `$90:A397`.

### StandingShotRestartsAnimation
- Mechanic. `Standing { plain: true }` with shoot held: `restart_standing_animation`.
  checked: original only (V24). Source: `$90:A3AD`.

### FloorFlushSubpixel, EnemyTouchZeroesYSubpixel
- Quirks of §6's contract. A block stop leaves that axis's subpixel FFFF moving right or down, 0
  moving left or up. A solid enemy already touched on the tested side sets the **Y** subpixel to 0,
  on either axis, with distance 0. checked: oracle (V7, V8, A11, A14); original only (V20, V26,
  V27). Source: `$94:8F82`, `$A0:AAC8`.

### ArmPumpStep
- Quirk. Changing aim while running moves her 1 px forward. Second entry point, `arm_pump_step`,
  after `step` and the settled `pose_request` (§8); it reads the frame-start locomotion and
  facing, `wall_stop` and `pose_request`, writes only `position`, and returns `ran_into_wall`.
- Rule: `Forced`: nothing. Else `wall_stop` and `Running`: `ran_into_wall`. Else `Pose { running:
  true }`: `enemy_ahead` by 1.0 forward (a hit: `ran_into_wall`, no move); otherwise
  `move_horizontal` by 1.0 forward with `SkipEnemies`, and a stop gives `ran_into_wall`. No speed
  change, no step down. checked: oracle (V1-V4, V8, V9, V14); original only (V22b, V23, V27).
- Source: `$91:EADE`.

### JumpRelease
- Mechanic. Letting go of jump ends the rise at once. If vertical is Up and (jump not held or
  `negative(vertical_speed.pixel)`): vertical_speed = 0, vertical = Down. checked: oracle (A2,
  A6, A12b). Source: `$90:8FDC`.

### StartedFalling
- Mechanic. If vertical is Up and `negative(vertical_speed.pixel)`: vertical_speed = 0, vertical
  = Down. checked: original only (A20). Source: `$90:90C4`.

### VerticalStep
- Mechanic. She rises and falls under gravity, moving by the speed she had before it.
- Rule: d = vertical_speed. Then if vertical is Down, vertical_speed += gravity unless its pixel
  word is exactly `fall_cap_pixel` (`FallCap`); otherwise vertical_speed −= gravity. Then
  `move_down` by d if vertical is Down, else `move_up` by d (None counts as up).
- checked: oracle (A1, A4-A7, A9-A11, A13, A14, A16). Source: `$90:90E2`.

### FallCap
- Quirk. The fall speed stops growing when its pixel word is exactly 5 (5.1400 in vanilla: 4.F800
  + 0.1C00); a speed set higher keeps its value. checked: original only (A21).
- Source: `$90:910C`.

### NoInputStops
- Mechanic. In the air, letting go of left and right stops her drift, unless she is braking or
  turning: if mode is Accelerating and neither is held, base = 0 and there is no horizontal move
  (`still_moves`: the move still runs, by extra_run). checked: oracle (A12b, A13).
- Source: `$90:900D`, `$90:9174`, `$90:91A8`.

### SpinJumpDrift
- Mechanic. A spin jump drifts only while the side she faces is held. If `capped`, or mode is
  TurningAround, or her facing side is held: mode Accelerating becomes Decelerating, and she moves.
  Otherwise base = 0, mode = Accelerating, no move. checked: oracle (A1, A2, A6, A8, A9).
- Source: `$90:906C`.

### WallJumpDrift
- Mechanic. A wall jump always moves; mode Accelerating becomes Decelerating. checked: oracle
  (A4). Source: `$90:902D`.

### BallDropStops
- Mechanic. A falling ball with neither left nor right held, in mode Accelerating:
  `CancelSpeedBoost`, extra_run = base = 0. checked: original only (A22). Source: `$90:A5CD`.

### WallJumpCheck
- Mechanic. After a spin jump's horizontal move, pressing away from a wall within 8 px arms the
  wall jump, and a new jump press fires it.
- Rule, only if `was_spinning`: pressing left probes the **right** side, else pressing right
  probes the left; neither: nothing. `probe_wall(heading, wall_jump_reach)` (§6). Not
  `spin_ready`: a hit gives `wall_jump_armed`. `spin_ready`: a hit with distance below the reach
  and jump newly pressed gives the result `WallJump`, and the row skips its vertical step.
- checked: oracle (A2 armed, A3 fired). Source: `$90:9D35`.

### WallProbeSubpixel
- Quirk. The wall probe writes the X subpixel as a stop would (FFFF probing right, 0 probing
  left) when it finds a block, without moving her. So a spin jump moving left ends at x.sub FFFF
  (item-morph-ball:711, 712, the frames the ground spec left open). checked: oracle (A2, A3).
- Source: `$94:967F`, `$94:8F49`.

### SpaceJump
- Mechanic. With space jump, a new jump press while falling in a spin jump jumps again. Before
  the horizontal move: if `space_jump`, vertical is Down, the speed's middle bytes (pixel low byte
  and sub high byte, 8.8) are in `space_jump_window`, and jump is newly pressed: `Jump`.
  checked: original only (A23). Source: `$90:A488`.

### Jump
- Mechanic. vertical_speed = `jump` (hi-jump's if equipped), vertical = Up. checked: oracle (P5).
- Source: `$90:98BC`.

### SpringBallJump
- Mechanic. Landing as a spring ball with jump held jumps again: bounce = None, `Jump`.
  checked: original only (P8). Source: `$91:F25E`.

### BallBounce
- Mechanic. A ball landing fast bounces twice before it rests. On `Landed(MorphBall |
  SpringBall)`, by `bounce`: None, with vertical_speed.pixel not below 3: bounce First, vertical
  Up, speed `bounce`; First: bounce Second, Up, speed `bounce` − 1.0; Second, or None slower than
  3: `EndVertical`, `LandingStop`. checked: original only (P7). Source: `$91:F1FC`, `$91:F25E`.

---

## 6. Collision interface

Every request carries the body: position and radius. `heading` names the side tested; the sign
of the distance decides where she moves.

| Request | Gets back |
| --- | --- |
| `move_horizontal(body, heading Left/Right, d, enemies)` | `Moved { distance, stop: Option<Stop> }` |
| `move_down(body, heading Down/Up, d ≥ 0, enemies)` | `Moved { .. }` |
| `move_up(body, d ≥ 0, TestEnemies)` | `Moved { .. }`, distance ≤ 0 |
| `enemy_ahead(body, heading, d)` | `Option<EnemyHit { touching }>` |
| `probe_wall(body, heading, d)` | `Option<WallHit { distance_px, subpixel: Option<u16> }>`: solid enemies first, then blocks (`subpixel` from a block: `WallProbeSubpixel`); nothing moves |

- `enemies` is `TestEnemies` except for `ElevatorRide` and `ArmPumpStep`'s move. A horizontal
  `TestEnemies` move ends with the slope seat, which feeds `on_slope`.
- `Stop { side, blocker }`: `Block { subpixel }`, written to that axis; `Enemy { touching }`,
  touching: distance 0 and Y sub = 0; `Slope`. Any horizontal stop is `WallStop`'s trigger.
- The vectors' flat stand-in is described in the vector file's header.

---

## 7. Pose selection's writes

Third entry point `pose_writes(start: Option<PoseStart>, command: PoseCommand)`, called when the
new pose starts (§8). It writes the State only.

**Order:** the start's writes, then the command, **unless** the start replaced the pose with
another (every turn start does, and the space-jump and screw-attack substitutions): then the
command is skipped.

**`PoseStart`**:
- `Turn` (on the ground or in the air): `Fold` (base += extra_run, extra_run = 0), mode
  TurningAround.
- `BallReverse` (a ball pose facing the other way), `SpinReverse` (a spin jump after a spin or wall
  jump, facing the other way): `Fold`, `CancelSpeedBoost`, mode TurningAround (A8).
- `JumpOrFall` (a normal jump or falling pose): mode = Decelerating if extra_run ≠ 0, else
  Accelerating (P4).
- `TakeOff { from_plain_crouch }` (the normal jump's first pose): `Jump`; from the crouch without
  aim, also y −= `crouch_jump_lift` (P5).
- `SpinStart` (a spin jump not after a spin or wall jump), `SpringLaunch` (the spring ball in the
  air, from the spring ball on the ground): `Jump`.
- `WallJumpStart` (not after a wall jump): vertical_speed = `wall_jump` (hi-jump's if equipped),
  vertical = Up (P2).

**`PoseCommand`**:
- `Decelerate`: if base = 0, as `Stop`; else `Fold`, mode Decelerating, `CancelSpeedBoost`,
  `KillYIfStartedFalling` (P6).
- `Stop`: mode Accelerating, `CancelSpeedBoost`.
- `KillXSpeed`: base = 0, mode Accelerating, `KillYIfStartedFalling` (P9).
- `KillRunSpeed`: extra_run = 0, `CancelSpeedBoost`, `KillYIfStartedFalling` (P9).
- `Vertical(result)`, which pose selection passes whenever `vertical_result` isn't None and its
  kind isn't NoChange:
  - `HitCeiling`: vertical_speed = 0, vertical = Down (P10).
  - `Falling`: unless vertical is Up: bounce = None, vertical_speed = 0, vertical = Down (P11).
  - `Landed(Grounded)`: `LandingStop`, `EndVertical` (P1, P3).
  - `Landed(MorphBall | SpringBall)`: `SpringBallJump` if spring and jump held, else `BallBounce`.
  - `WallJump`: base = 0, mode Accelerating (P2).
- `None`: nothing.

`EndVertical`: vertical_speed = 0, vertical = None, bounce = None. `LandingStop`: base = 0, mode
Accelerating; extra_run is kept (the standing row zeroes it the next frame).
`KillYIfStartedFalling`: if the new pose is Falling and the old one wasn't: vertical_speed = 0,
vertical = Down. Which command comes with which pose is pose selection's.

Catalogue entries, all Mechanic and Fixed:
- `PoseStartWrites` (the starts). checked: oracle (P2, P4, P5, A8); original only (P12).
  Source: `$91:F543`, `$91:F624`, `$91:F8D3`, `$91:FA0F`, `$91:FBBB`.
- `PoseCommandWrites` (the four commands and `KillYIfStartedFalling`). checked: original only
  (P6, P9). Source: `$91:EC50`, `$91:FB8E`.
- `VerticalResultWrites` (`Vertical`, `LandingStop`, `EndVertical`). checked: oracle (P1, P2,
  P3, A14, A15); original only (P7, P8, P10, P11). Source: `$91:EFC4`, `$91:F010`.
- `PoseWriteOrder` (Timing: start before command; a replaced pose skips the command). checked:
  oracle (P2); original only (P12). Source: `$91:EB88`, `$91:F404`.

---

## 8. Order that matters

Visible at the frame's end:
1. The row comes from the frame-start locomotion.
2. `ExtraRunSpeed`, `JumpRelease`, `BaseSpeed`, the drift, then the horizontal step: the step uses
   this frame's speeds, and `HorizontalDirection` reads mode after `BaseSpeed` (V15b).
3. `SpaceJump` before the horizontal step; `WallJumpCheck` after it and before the vertical step.
4. The vertical step moves by the speed from before this frame's gravity (A7, A12b).
5. The horizontal step (with `WallStop`) before the step down, which reads this frame's total.
6. The After after the vertical step (V14).
7. The frame: pose input → `step` → animation and hits → pose selection settles its request from
   `vertical_result` (Falling: the fall pose of its kind; HitCeiling: the current pose) →
   `arm_pump_step` → the new pose starts, with `pose_writes` (§7).

---

## 9. Decode notes

In [samus-motion.decode.md](samus-motion.decode.md), for the observer and the harness's decoder only.

## 10. Deferred

**To collision.** The sweeps: horizontal leading edge x + d + rx − 1 right, x + d − rx left, one
column, rows top to bottom; vertical edge y + d + ry − 1 down, y + d − ry up, one row, columns in
alternating order by frame parity; a move of 16 px or more can pass a floor. Solid stops are flush:
right d = max(0, (edge & ~F) − rx − x), sub FFFF; left d = min(0, (edge | F) + 1 + rx − x), sub 0;
likewise vertically. Slopes: per-shape horizontal scaling while vertical speed is 0; the slope
seat; square-slope quadrants and `on_square_slope`. The heading drives the slope and special-block
reactions. The solid-enemy test: the target is one pixel past |d| on the heading (right unless both
subs are 0, left when they differ); touching: distance 0 and Y sub 0; past it: skipped. Doors,
spikes, special and bombable blocks; the elevator room's floor is a door-type block that behaves as
solid (V7).

**Liquids and the speed booster.** `submerged` picks the water or lava/acid tables (water: Running
and Ball accel 0.0400, max 2.C000, decel 0.0800), gravity 0.0800 / 0.0900, jump 1.C000 / 2.C000
(hi-jump 2.8000 / 3.8000), wall jump 0.4000 / 2.A000 (hi-jump 0.8000 / 3.8000), space-jump window
from 0.80. Leaving liquid onto a slope may keep the water table. The speed booster: extra-run cap
7.0000, and a jump adds extra_run.pixel / 2 to the speed's pixel and extra_run.sub to its sub, with
no carry between them.

**Pushes.** Extra X displacement before the clamp; extra Y displacement, which replaces the step
down and is the take-off's only vertical move; the speed divisor; the deceleration multiplier.

---

## 11. Open questions

1. A wall stop and a forced pose on one frame: the X-speed-killed flag would survive to the next
   frame's check. Not in the records.
2. Moonwalking's backward step: which pose start gives it was not read. V18 checks only the speed.
3. The elevator descent moves 1 and 2 px on alternate frames, so the elevator sets y itself; V23
   is constructed.
4. item-morph-ball:797, landing on the face block at 4.9800: y goes 029A.2BFF → 029B.0000, a
   move of 0.D401 with the subpixel cleared, though an enemy not yet touched should write nothing.
   A collision question (the solid-enemy distance); A15 doesn't compare y.
5. The harness's skip rule for the message box needs its RAM word confirmed.
6. item-morph-ball:1311 lifts y by 13 at take-off from the crouch: 10 is `crouch_jump_lift`, 3 is
   taken to be pose fit. P5 doesn't compare y.
