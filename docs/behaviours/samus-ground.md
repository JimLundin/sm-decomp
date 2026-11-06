# Samus on the ground: behaviour spec

The observer's spec for MODEL §4, written to the MODEL §5 template. Its vectors are in
[`scenarios/vectors/samus-ground.toml`](../../scenarios/vectors/samus-ground.toml).

## Header

**Scope.** On the ground: standing, running, turning, crouching, moonwalking, the
crouch/stand/morph/unmorph transitions, running into a wall, the morph ball and the spring ball;
the speed rules they share (base speed, extra run speed, the caps); the step down onto lower ground;
the arm-pump step; one horizontal and one vertical sweep against blocks.

**Out of scope.** The air (jumps, falling, the wall jump), pose selection (which pose comes next),
animation, footstep sounds and graphics, the speed booster's boost counter and palette, solid
enemies, liquids beyond choosing the speed table, and the outside pushes listed in §9. Where one of
these feeds the domain, it is an **input** (§1.2).

**Baseline:** 516 lines (`crates/game/src/movement.rs` at `rebuild/spine` `982b203`, ground motion
plus the ground half of collision).

**Budget: 300 lines** for `game::model::samus` ground motion and the two sweeps, excluding tests
and the `data` tables. Going over is a review stop (MODEL §4).

**Notation.** A position or speed is `P.S`: a 16-bit pixel word and a 16-bit subpixel word, both
hex, so `2.C000` is 2.75 px. A record N holds the state at the end of frame N; its pad is the pad
that frame N+1 reads.

---

## 1. Quantities

### 1.1 State (the declared fields)

| Quantity | Units | Type | Read by |
| --- | --- | --- | --- |
| `position` | x and y, each P.S; her centre | `Position { x: Fixed, y: Fixed }`, 32-bit, moves wrap mod 2^32 | every entry; output |
| `base_speed` | P.S, never negative | `Speed` | `BaseSpeedAccelerate`, `BaseSpeedDecelerate`, `HorizontalStep` |
| `extra_run` | P.S, never negative | `Speed` | `ExtraRunSpeed`, `ExtraRunCap`, `HorizontalStep` |
| `momentum` | | `bool` | `ExtraRunSpeed` |
| `mode` | | `SpeedMode { Accelerating, TurningAround, Decelerating }` | `BaseSpeedAccelerate`, `BaseSpeedDecelerate`, `HorizontalDirection`, `BallAtRest` |
| `on_square_slope` | | `bool`: the last vertical sweep stopped on a square-slope block | `StepDown` (next frame) |

`on_square_slope` is the only quantity whose cross-frame use is easy to miss. Each vertical sweep
that moves clears it, and a vertical stop on a square slope sets it. The next frame's step down
reads it (with this frame's `SlopeSeat`): see `StepDown`. It was false at every frame's end in slice
0, which has no slopes.

### 1.2 Inputs (owned by other domains)

| Input | Type | From | Read by |
| --- | --- | --- | --- |
| `locomotion` | `Locomotion` (§2) | the pose, as it stands at the frame's start | `LocomotionAtFrameStart`, the motion table |
| `facing` | `Facing { Left, Right, Forward }` | the pose | `HorizontalDirection`, `BallAtRest`, `ArmPumpStep` |
| `radius` | x, y in px | the pose domain (in slice 0: x = 5; y = 0x15 standing, running and turning, 0x10 crouching and the ball transitions, 7 ball, 0x18 the elevator pose) | the sweeps |
| `pad` | buttons held: run, shoot, left, right | the controller | `ExtraRunSpeed`, `StandingShotRestartsAnimation` |
| `new_running_pose` | `bool`: the pose update chose a new pose that runs | pose selection | `ArmPumpStep` |
| `airborne` | `bool`: her vertical motion is not none | the air domain | the ball and transition rows |
| `submerged` | `bool`: a liquid's surface is above her bottom edge, and liquid physics apply | the room's liquid | `SpeedTableRow`, `ExtraRunSpeed` |
| `gravity_suit`, `speed_booster` | `bool` | equipment | `SpeedTableRow`, `ExtraRunSpeed`, `ExtraRunCap` |
| `elevator_moving` | `bool` | the elevator | `ElevatorRide` |
| `frame_parity` | `Even` / `Odd` | the frame counter | `VerticalScanAlternates` |
| blocks | the room's block types and slopes | `data::level` | the sweeps |

### 1.3 Results of the frame (Contacts, not State)

`step` returns these to the pose update the same frame. None of them is read on a later frame.

| Result | Type | Read by |
| --- | --- | --- |
| `wall_stop` | `bool`: `WallStop` fired this frame | `ArmPumpStep` (same frame) |
| `vertical` | `Vertical { None, Landed(LandingKind), Falling, HitCeiling }` after the row's After | the pose update (same frame) |

### 1.4 Read by: none (they leave the State)

MODEL decision 3 applies to each of these:

- **total speed** (base + extra run). It is computed and used inside one frame: every row that
  steps down steps horizontally first, so the step down always reads this frame's value. Nothing
  reads it on a later frame. The value left over on a frame with no horizontal step is never
  read. It is a local value in `HorizontalStep`, not stored.
- **the X-speed-killed direction.** Its two codes (left / right) are never told apart; the only
  reader asks "was it killed". It is read the same frame and cleared by the pose update. It
  becomes the `wall_stop` contact.
- **the distances moved** left, right, up and down. No code reads them.
- **the vertical result** as a stored value. It is cleared at the end of every frame; it is the
  `vertical` contact.
- **horizontal slope detection.** Always on: only an unused movement type turns it off. `SlopeSeat`
  always runs.
- **the speed divisor, the extra X and Y displacement and the deceleration multiplier.** Out of
  scope (§9, answer 2).

---

## 2. Locomotion

`Locomotion` names the way Samus moves this frame. It comes from the pose at the frame's start
(`LocomotionAtFrameStart`). Every variant is one row of the motion table (§3).

| Variant | Poses (domain) |
| --- | --- |
| `FacingForward` | standing, facing the screen |
| `Standing` | every other standing pose, including landing from a jump |
| `Running` | running, at every aim |
| `Ball { spring: bool, at_rest: bool }` | the morph ball or spring ball on the ground; `at_rest` for the still ("facing") ball pose, false for the rolling ("moving") ones |
| `Crouching` | crouching, at every aim |
| `TurningOnGround` | the on-ground turn |
| `Transition` | the crouch, stand, morph and unmorph transitions, at every aim |
| `Moonwalking` | moonwalking, at every aim |
| `RanIntoWall` | the "ran into a wall" poses |

The pose codes for each variant are in §8.

---

## 3. Motion table

Columns:
- **Horizontal:** `Speed` = `ExtraRunSpeed`, then `BaseSpeedAccelerate` or `BaseSpeedDecelerate`
  by `mode`, then `HorizontalStep`; `Hold` = `HorizontalStep` with the base speed taken as 0 (only
  `extra_run` moves her, base_speed is unchanged); `None` = no horizontal step.
- **Vertical:** `KeepToGround` = `StepDown`; `GroundUnlessAirborne` = the air domain's vertical
  motion if `airborne`, else `StepDown`; `None`.
- **After:** what runs after the vertical step. `Zero` = extra_run = 0, base_speed = 0, mode =
  Accelerating. `Cancel` = `CancelSpeedBoost`.

| Row | Horizontal | Vertical | After |
| --- | --- | --- | --- |
| `FacingForward` | None | `ElevatorRide` | vertical = None |
| `Standing` | Hold | KeepToGround | Cancel, Zero; `StandingShotRestartsAnimation` |
| `Running` | Speed | KeepToGround | nothing |
| `Ball`, `at_rest` and mode Accelerating | Hold | GroundUnlessAirborne | if not airborne: Cancel, Zero. If airborne: nothing |
| `Ball`, otherwise | Speed | GroundUnlessAirborne | nothing |
| `Crouching` | Hold | KeepToGround | Zero (no Cancel) |
| `TurningOnGround` | Speed | KeepToGround | Cancel; extra_run = 0; vertical = None (`TurnIgnoresFall`) |
| `Transition` | Hold | GroundUnlessAirborne | `TransitionLanding`; vertical = None |
| `Moonwalking` | Speed | KeepToGround | nothing |
| `RanIntoWall` | Hold | KeepToGround | Cancel, Zero |

The spring ball and the morph ball share one row. They differ only in which poses are at rest.

---

## 4. Behaviours

Every entry is **Fixed** except `SpeedCap`, the one Toggle (MODEL §3). "Parameter" values live in
`data` tables.

### LocomotionAtFrameStart
- kind: Timing. summary: on the frame Samus changes pose, she still moves the way her old pose moved.
- Rule: the row and the speed-table entry come from the locomotion at the frame's start. The pose
  update later in the frame does not re-run motion.
- switch: Fixed. checked: oracle (V6, V9, V13).
- Source: `$90:A337`.

### SpeedTableRow
- kind: Mechanic. summary: each way of moving has its own acceleration, top speed and braking.
- Rule: the entry for the locomotion gives (accel, max, decel), each P.S. The normal table applies
  unless she is `submerged` without the gravity suit; then the water table, or the lava/acid table,
  applies.
- Params (normal table), as accel / max / decel:

  | Locomotion | accel | max | decel |
  | --- | --- | --- | --- |
  | Standing, FacingForward | 0.C000 | 0.0000 | 0.8000 |
  | Running | 0.3000 | 2.C000 | 0.8000 |
  | Ball (morph or spring) | 0.C000 | 3.4000 | 0.8000 |
  | Crouching | 0.C000 | 0.0000 | 0.8000 |
  | TurningOnGround | 0.C000 | 0.0000 | 0.8000 |
  | Transition | 0.C000 | 1.4000 | 0.8000 |
  | Moonwalking | 0.C000 | 0.8000 | 0.8000 |
  | RanIntoWall | 0.C000 | 0.0000 | 0.8000 |

  The water table: Running and Ball accel 0.0400, max 2.C000, decel 0.0800.
- switch: Parameter. checked: oracle (V1-V6, V10) for the normal table; original only for the water
  and lava tables.
- Source: `$90:9BD1` (tables `$90:9F55`, `$90:A08D`).

### BaseSpeedAccelerate
- kind: Mechanic. summary: Samus speeds up by the row's acceleration each frame, up to its top speed.
- Rule: when mode = Accelerating: base_speed += accel, then `SpeedCap`.
- switch: Fixed. checked: oracle (V1-V5, V10).
- Source: `$90:9A7E`.

### SpeedCap
- kind: Quirk. summary: speed briefly overshoots, or snaps *up* to, the top speed.
- Rule, `SpeedCap::Wordwise` (vanilla). After adding, compare the pixel words, then the subpixel
  words. "a is below b" means (a − b) mod 2^16 has bit 15 set.
  - pixel below max.pixel: keep.
  - pixels differ otherwise: base = max.
  - pixels equal: keep if the subpixel is equal to max.sub or below it, else base = max.

  Consequences in the oracle:
  - Running: 1.E000 + 0.3000 = 2.1000. 0x1000 − 0xC000 = 0x5000, bit 15 clear, so it is past the
    cap and becomes 2.C000: a free 0.B000 (V3).
  - Ball: 3.0000 + 0.C000 = 3.C000. 0xC000 − 0x4000 = 0x8000, bit 15 set, so it is below the cap
    and kept one frame above the max 3.4000. Next frame 4.8000 caps (V5).
  - Turning (max 0): 0 + 0.C000 is kept, then 1.8000 caps to 0 (`TurnCreep`, V6).
- Rule, `SpeedCap::Exact`: base = min(base, max), comparing the whole 32-bit values.
- `SpeedCap` covers the base speed only. The extra run speed has its own cap (`ExtraRunCap`), which
  doesn't follow this switch.
- switch: **Toggle** (`Wordwise` vanilla | `Exact`). checked: oracle (V3, V5, V6) for `Wordwise`;
  `Exact` is not compared (V3, V4, V5 and V6 have `Exact` variants).
- Source: `$90:9A99`.

### TurnCreep
- kind: Quirk. summary: during the on-ground turn, Samus slides 0.C000 px every other frame.
- Rule: none of its own: `SpeedCap::Wordwise` on the turning row. Base alternates 0, 0.C000, 0. She
  moves along her facing, which is the *new* facing once mode is back to Accelerating. Under
  `Exact` there is no creep.
- switch: follows `SpeedCap`. checked: oracle (V6).
- Source: `$90:A67C`.

### BaseSpeedDecelerate
- kind: Mechanic. summary: when Samus brakes or reverses, her speed falls by the row's deceleration.
- Rule: when mode = Decelerating or TurningAround: base_speed −= decel. If the pixel word then has
  bit 15 set, base_speed = 0 and mode = Accelerating. No cap applies.
- switch: Fixed. checked: oracle (V6, first frame).
- Source: `$90:9ABA`.

### ExtraRunSpeed
- kind: Mechanic. summary: holding run while running adds speed, up to 2 px a frame (7 with the speed booster).
- Rule, on every `Speed` row:
  - *Running* means: not `submerged` (with the gravity suit, never submerged), locomotion is
    `Running`, and run is held. Then momentum = true (the speed booster's boost start is out of
    scope), and `ExtraRunCap` decides the new extra_run.
  - Otherwise: if momentum is false, extra_run = 0; else extra_run is kept.
- Quirk: the water and lava columns of the extra-run table are never used, because a submerged
  Samus is "not running" before the column is chosen.
- Params: step 0.1000; cap 2.0000, or 7.0000 with the speed booster. switch: Parameter.
- checked: oracle (V1-V4, V8); original only for the speed booster and liquids.
- Source: `$90:973E` (table `$90:9F01`).

### ExtraRunCap
- kind: Quirk. summary: (invisible in vanilla play) the run-speed cap is tested before the add, word by word.
- Rule: when running, test the *old* extra_run against the cap:
  - pixel below cap.pixel: add the step;
  - otherwise, subpixel below cap.sub: add the step;
  - otherwise: extra_run = cap.

  So the test happens before the add (extra_run can end a frame one step above the cap), and a
  pixel word *above* the cap still adds when its subpixel is below. 2.8000 grows to 2.9000; 2.F000
  grows to 3.0000, and the next frame snaps to 2.0000. From 0 in steps of 0x1000 she lands exactly
  on 2.0000 or 7.0000, so vanilla play never shows it.
- switch: Fixed. It does not follow `SpeedCap` (answer 3). checked: original only (V11, constructed).
- Source: `$90:97D5`.

### CancelSpeedBoost
- kind: Mechanic. summary: run speed carries through a jump and is lost when she stops.
- Rule: momentum = false (the boost counter and palette also reset; out of scope). It runs in the
  `Cancel` After of the rows (§3) and in `WallStop`. The crouching row does not cancel.
- switch: Fixed. checked: oracle (V8, V12).
- Source: `$91:DE53`.

### HorizontalStep
- kind: Mechanic. summary: Samus moves sideways by her total speed.
- Rule: total = (base_speed or 0 under `Hold`) + extra_run. The displacement is +total along the
  direction (`HorizontalDirection`), −total against it, clamped by `DisplacementClamp`. One
  horizontal sweep (§5) moves her. Then `WallStop`, then `SlopeSeat`.
- switch: Fixed. checked: oracle (V1-V10).
- Source: `$90:8EA9`.

### HorizontalDirection
- kind: Mechanic. summary: during a turn, Samus keeps sliding the way she was going.
- Rule: mode TurningAround: the step goes against the facing. Otherwise along it.
- switch: Fixed. checked: oracle (V6).
- Source: `$90:8EA9`.

### DisplacementClamp
- kind: Mechanic. summary: Samus never moves more than 15 px sideways in a frame.
- Rule: the displacement's pixel word is clamped to [−15, +15] as a signed word, and the subpixel
  word is kept. So right, 10.4000 becomes 0F.4000. Left, −10.4000 (FFEF.C000) becomes FFF1.C000,
  which is −0E.4000: leftward, she moves at most 15 − sub.
- Unreachable on the ground in vanilla (the largest total is 9.C000), and reachable through the
  out-of-scope pushes (§9).
- Params: 15. switch: Parameter. checked: original only (V12, constructed).
- Source: `$90:E4AD`.

### WallStop
- kind: Mechanic. summary: running into a wall stops her dead, flush against it.
- Rule: if the horizontal sweep collided: `CancelSpeedBoost`; extra_run = 0, base_speed = 0, mode =
  Accelerating; `wall_stop` = true. Otherwise `wall_stop` = false and the vertical result is
  cleared.
- switch: Fixed. checked: oracle (V8).
- Source: `$90:E5CE`.

### SlopeSeat
- kind: Mechanic. summary: walking across a slope keeps her feet on its surface.
- Rule: after every horizontal step:
  - Feet: the block under her centre x at her bottom edge (y + ry − 1). If it is a non-square slope,
    not upside down: A = surface(column) − (bottom mod 16) − 1, where column = x mod 16, mirrored if
    the slope is x-flipped. If A < 0: y += A (raises her), and she counts as seated this frame.
  - Head: the block under her centre x at her top edge (y − ry). If it is a non-square slope and
    upside down, the mirror rule pushes her down when she is inside it, and she counts as seated.
- Params: `data::level::slope` surfaces. switch: Fixed. checked: original only.
- Source: `$94:87F4` (surfaces `$94:8B2B`).

### StepDown
- kind: Mechanic. summary: walking off a ledge or down a slope, Samus sticks to the ground beneath her.
- Rule: one downward vertical sweep (§5) by:
  - 1.0 px, if she was seated this frame (`SlopeSeat`) or `on_square_slope`;
  - otherwise total + 1.0 px (1 is added to the pixel word, the subpixel kept). The total is the
    unclamped one.

  A hit gives `Landed`, flush with the surface. A miss moves her the whole way and gives `Falling`.
- Params: `step_down_extra` = 1.0 px. switch: Parameter. checked: oracle (V7, V10, V13).
- Source: `$90:923F`.

### TurnIgnoresFall
- kind: Quirk. summary: a turning Samus sinks onto lower ground instead of falling.
- Rule: the turning row clears the vertical result after its step down, so a `Falling` step down
  doesn't start a fall. She sinks by total + 1.0 px a frame until she lands (V7: three frames).
- switch: Fixed. checked: oracle (V7).
- Source: `$90:A68F`.

### TransitionLanding
- kind: Mechanic. summary: landing during a crouch or morph transition ends the vertical motion.
- Rule: if the vertical result is `Landed` and this locomotion's landing kind is "no change", the
  air domain's vertical speed, vertical direction and ball bounce are reset to none.
- switch: Fixed. checked: oracle (V13).
- Source: `$90:A640`.

### ElevatorRide
- kind: Mechanic. summary: on a moving elevator, Samus facing the screen rides it down.
- Rule: no horizontal step and no speed change. If `elevator_moving`: one vertical sweep down 1.0,
  with no solid-enemy test, and with slope reactions told the move is *upward* (Quirk). If not,
  nothing moves.
- switch: Fixed. checked: original only.
- Source: `$90:A392`.

### StandingShotRestartsAnimation
- kind: Mechanic. summary: shooting while standing un-aimed restarts the standing animation.
- Rule: on the `Standing` row, for the plain standing poses (no aim), with shoot held: animation
  timer = 0x10, animation frame = 0. It is output for the animation domain, not State here.
- switch: Fixed. checked: original only.
- Source: `$90:A3AD`.

### FloorFlushSubpixel
- kind: Quirk. summary: (invisible) on flat solid floor her vertical subpixel is FFFF.
- Rule: a sweep that stops on a solid block sets the subpixel first: down or right, FFFF; up or
  left, 0. The distance it then moves has subpixel 0, so the subpixel stays. A non-square slope stop
  leaves the subpixel alone.
- switch: Fixed. checked: oracle (V7, V8).
- Source: `$94:8F82`.

### SlopeSlowdown
- kind: Mechanic. summary: running up or down a steep slope covers less ground.
- Rule: in the horizontal sweep, each non-square slope block (not upside down) in the leading
  column, while her vertical speed is 0, scales the displacement: d = d × factor / 256, done on d
  reduced to 8.8 (its subpixel low byte is lost) and negated back for a leftward step. The block
  never stops her, and the scan continues, so two slope rows scale twice.
- Params (by slope shape): 12, 14, 15 → 0xC0; 16, 17 → 0xD8; 1B, 1C → 0x80; E, F → 0xB0; others
  0x100. switch: Parameter. checked: original only.
- Source: `$94:84D6` (factors `$94:8586`).

### VerticalScanAlternates
- kind: Quirk. summary: which block she lands on, when two differ under her, flips every frame.
- Rule: the vertical sweep scans her columns left to right on even frames and right to left on odd
  ones. The first block that collides decides.
- switch: Fixed. checked: original only.
- Source: `$94:9763`.

### ArmPumpStep
- kind: Quirk. summary: changing aim while running moves Samus an extra pixel forward.
- Rule, in the pose update after all motion:
  - if `wall_stop` and the locomotion is `Running`: the pose becomes "ran into a wall"; no step.
  - else if `new_running_pose`: test 1.0 px forward against solid enemies (a hit: "ran into a
    wall"). Otherwise move 1.0 px forward through the horizontal sweep, with no `SlopeSeat`, no
    `WallStop` and no step down. A block hit: "ran into a wall".

  It changes no speed. With no step down after it, it can lift her off a downward slope.
- Params: 1.0 px. switch: Fixed. checked: oracle (V2-V4, V9; V1 shows no step on an unchanged pose).
- Source: `$91:EADE`.

---

## 5. Collision: one sweep

A sweep moves Samus by a signed P.S distance `d` along one axis. It reads her position, radii and
`d` (and `frame_parity`, vertically). Solid enemies are tested first by the enemy domain; if one
blocks, it adjusts `d` and no block test runs.

**Horizontal.** Leading edge: x + d + rx − 1 moving right (d's pixel word ≥ 0), x + d − rx moving
left. One column, edge >> 4. Rows from her top ((y − ry) >> 4) to her bottom ((y + ry − 1) >> 4), at
the current y, top first; the first block that collides ends the scan.
- air: pass.
- solid: right, d = max(0, (edge & ~F) − rx − x).0000 and x.sub = FFFF; left, d = min(0, (edge | F)
  + 1 + rx − x).0000 and x.sub = 0.
- non-square slope: pass, scaling d (`SlopeSlowdown`).
- square slope: only the half of the block the edge is in, by row half, from the quadrant table;
  a solid quadrant stops her as a solid block does, aligned to 8 px.
- doors, spikes, special and bombable blocks: their own domains.

Then x += d. The result is "collided".

**Vertical.** Edge: y + d + ry − 1 moving down, y + d − ry up. One row. Columns from (x − rx) >> 4 to
(x + rx − 1) >> 4, in `VerticalScanAlternates` order; the first collision ends the scan. A sweep
that moves clears `on_square_slope`.
- solid: down, d = max(0, (edge & ~F) − ry − y).0000 and y.sub = FFFF; up, d = min(0, (edge | F) +
  1 + ry − y).0000 and y.sub = 0.
- non-square slope, only in the column under her centre x; going down it must not be upside down
  (going up, it must be): A = surface(column) − (edge mod 16) − 1. If A ≤ 0: d = max(0, d.pixel +
  A).0000, y.sub unchanged.
- square slope: quadrants, aligned to 8 px; y.sub = FFFF down, 0 up; `on_square_slope` = true.

Then y += d. Down: `Landed(kind)` on a collision, else `Falling`. Up: `HitCeiling` or None. The
landing kind comes from the locomotion: ground, ball, spring ball, or no change (`Transition`).

**Slopes and the step down together.** Up a slope, `SlopeSeat` raises her and the step down is only
1.0, which finds the surface she was seated on. Down a slope she isn't seated, and the step of total
+ 1.0 px pulls her onto the lower surface, which covers slopes up to 45°. Steeper, or a drop deeper
than that, and she is `Falling`.

---

## 6. Order that matters

Everything else in the frame is free (MODEL §1). These orders are visible at the frame's end:

1. The row is chosen from the locomotion at the frame's start, before the pose update.
2. `ExtraRunSpeed` comes before the base speed, and both before the horizontal step: the step uses
   the updated speeds.
3. The horizontal step (with `WallStop` and `SlopeSeat`) comes before the step down, which reads
   this frame's total and this frame's seat.
4. The After runs after the step down: `Standing`, `Crouching` and `RanIntoWall` move by the old
   extra_run this frame, then zero it.
5. `ArmPumpStep` comes after all motion, and so after `WallStop`.
6. In a sweep the first block that collides wins, in scan order.
7. A solid stop sets the subpixel before the move is added (`FloorFlushSubpixel`).

---

## 7. Answers to the implementer

1. **Crouching, moonwalking and the spring ball.** They all run on the ground path.
   - `Crouching`: `Hold` horizontally (only extra_run moves her), then `StepDown` unconditionally
     (no airborne test), then extra_run = 0, base_speed = 0, mode = Accelerating. It does **not**
     cancel momentum. V13's second frame shows the zeroing.
   - `Moonwalking`: the `Speed` row with its own table entry (max 0.8000), then `StepDown`, then
     nothing.
   - The spring ball: the morph ball's row exactly, with the spring-ball poses as `at_rest`.
2. **The speed divisor and the extra X and Y displacement are out of scope.** They are 0 in every
   slice-0 record, nothing in this domain writes them, and the original writes them only from other
   domains: the displacements from treadmill blocks, enemies that carry Samus and some PLMs, the
   divisor from two enemy and boss effects. The displacements and the deceleration multiplier are
   reset every frame by the block-inside check, so they are per-frame pushes, not State. The ground
   model has no divisor, no displacement and no multiplier. Their rules are kept in §9 for the
   domain that adds them.
3. **Two behaviours.** The base speed's cap (`SpeedCap`) adds, then compares, and snaps when the
   pixel word is above. The extra run's cap (`ExtraRunCap`) compares before the add, and still adds
   when the pixel word is above but the subpixel is below. The rules differ, so `SpeedCap::Exact`
   doesn't change the extra run. `ExtraRunCap` is Fixed: vanilla never reaches it, and there is no
   named use for a corrected version. The old V11 `Exact` row is removed. V11 now runs under both
   settings with the same result.

---

## 8. Decode notes

ROM codes, for `verify`'s decode and the `data` tables only. The model never names them.

- **mode:** 0 Accelerating, 1 TurningAround, 2 Decelerating.
- **facing** (the pose's direction byte): 4 Left, 8 Right, 0 Forward.
- **momentum:** a word, 0 or 1.
- **on_square_slope:** a word, 0 or 1. The decode reads the "position adjusted by slope" flag at the
  frame's end. At that point the horizontal seat has already been consumed and cleared, so the flag
  means exactly "the last vertical sweep stopped on a square slope".
- **Locomotion from the pose** (movement type, then pose):
  - type 0 Standing: poses 00 and 9B are `FacingForward`; the rest (01-08, 47, 48, A4-A7, D5,
    D6, E0-E7) are `Standing`. The plain standing poses for `StandingShotRestartsAnimation` are 01
    and 02.
  - type 1 `Running`: 09-12, 45, 46.
  - type 4 `Ball { spring: false }`: `at_rest` 1D (right) and 41 (left); rolling 1E and 1F.
  - type 5 `Crouching`: 27, 28, 71-74, 85, 86, D9, DA.
  - type E `TurningOnGround`: 25, 26, 43, 44, 8B-8E, 9C, 9D, BF-C4.
  - type F `Transition`: 35-38, 3B-3E, F1-FC. Poses 39, 3A, 3F, 40 and DB-DE are unused; the
    original gives DB-DE no motion at all.
  - type 10 `Moonwalking`: 49, 4A, 75-78.
  - type 11 `Ball { spring: true }`: `at_rest` 79 and 7A; rolling 7B and 7C.
  - type 15 `RanIntoWall`: 89, 8A, CF-D2.
- **the vertical result:** 0 none, 1 landed, 2 falling, 4 hit ceiling (5 is the wall jump's).
  Landing kind is a second byte, per movement type.
- **the X-speed-killed flag:** 0, 4 (left) or 8 (right); decoded as `wall_stop` = non-zero, and only
  needed if a cut falls between motion and the pose update, which it doesn't.
- **pad:** the motion reads run (B by default) and shoot (X by default). The vectors also name
  left, right, jump (A) and aim_down (L) for the pose update.

**RAM shape** (MODEL §5): the State fields whose decode is one word copied as-is.
- `position` x and y: the pixel and subpixel words are the quantity itself.
- `base_speed`, `extra_run`: the speed is its P.S value; no smaller form preserves the cap quirks.
- `momentum`, `on_square_slope`: one bool each, already the domain's value.
- `mode` is decoded to an enum, so it isn't copied as-is.

---

## 9. Deferred: pushes from other domains

Not in this domain's model. Recorded for the domain that adds them (treadmills, carrying enemies,
the slowing effects).

- **Extra X displacement** is added to ±total, before the clamp (right and left).
  If the sum's sign disagrees with the direction the step started in, the collision flag is cleared
  and nothing is killed.
- **Extra Y displacement**, when non-zero, replaces the step down: down by it + 1.0 when its pixel
  word is ≥ 0, up by its magnitude when negative.
- **Speed divisor** n (capped at 4): total = (base + extra_run) >> n. The subpixel sum carries into
  the pixels, then the pixel sum's low byte is shifted, its lost bits fill the subpixel's top, and
  the subpixel's low bits are lost. Exact only for pixel sums below 256.
- **Deceleration multiplier** m (non-zero): the decel step is (decel.sub high byte × m) as the
  subpixel and the high byte of (decel.pixel low byte × m) as the pixel.

---

## 10. Open questions

1. **The stale speed table.** The water or lava table is chosen while submerged, but nothing in
   motion chooses the normal table back. The block-inside check resets it to normal every frame
   when her bottom point is in air or a solid block, but not in a slope block. So leaving liquid
   directly onto a slope may keep the water table. Slice 0 never leaves the normal table. Is it
   reachable in vanilla play? If it is, the table choice becomes a State quantity.
2. **A skipped pose check.** The X-speed-killed flag is cleared by the pose update's wall check,
   which is skipped when another system forces a pose that frame. Then the flag survives to the
   next frame's check. That needs a wall stop and a forced pose on the same frame, outside this
   domain; never seen in the records.
3. **Door blocks as floor.** In the elevator room the floor row is a door-type block that behaved
   as solid (V7). The door reaction belongs to the door domain.
4. **Speed booster and liquids** (cap 7.0, the boost counter, the water table) are not exercised in
   slice 0.
5. **Morph-ball turning momentum** belongs to pose selection and was not read.
