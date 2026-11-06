# Samus on the ground: behaviour spec

The observer's spec for MODEL §4, written to the MODEL §5 template. Its vectors are in
[`scenarios/vectors/samus-ground.toml`](../../scenarios/vectors/samus-ground.toml).

## Header

**Scope: ground motion.** Standing, running, turning, crouching, moonwalking, the
crouch/stand/morph/unmorph transitions, running into a wall, the morph ball and the spring ball on
the ground: the motion table's rows; the speed rules and both caps; momentum; the direction; the
displacement clamp; the step-down request; the wall stop's effect on the speeds; the arm-pump step
(a second entry point, run by pose selection later in the frame: `ArmPumpStep`); and the contacts
handed to other domains.

**Out of scope.**
- **Collision** is an interface (§5): motion asks for a move and gets the result back. How blocks
  are found (the sweep, slopes, square-slope quadrants, the scan order) is the collision domain's,
  listed in §9.
- **Liquids and the speed booster** are deferred (§9). Only the normal speed table is in scope;
  `submerged` and `speed_booster` are inputs fixed false for the pilot.
- The air (jumps, falling, the wall jump), pose selection (which pose comes next, and the speed
  changes it makes when a pose starts: see `mode`), animation, sounds and graphics, solid enemies
  and the outside pushes (§9). Where one of these feeds the domain, it is an **input** (§1.2).

**Baseline: 516 lines** (`wc -l crates/game/src/movement.rs` at `rebuild/spine` `48d52f2`). That
file holds ground motion and the ground half of collision together. The observer may not read it,
so the ground-motion share isn't measured; 516 is an upper bound for this narrower scope.

**Budget: 200 lines** for `game::model::samus` ground motion, excluding tests and the `data`
tables. Estimate: the row table and its enums ~50, the speed rules and both caps ~50, extra run
and the clamp ~25, step down, elevator and the arm-pump step ~30, the collision interface (the
request and result types) ~15, the contacts ~20. **`ArmPumpStep` counts** against this budget: it
is a motion (a 1.0 px move through the collision interface) with its own entry point, which pose
selection calls once it has settled this frame's pose request; its result goes back to pose
selection. Going over is a review stop (MODEL §4).

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

**One State for every movement.** These five fields are one State, kept whatever the
locomotion, on the ground and in the air: every movement reads and writes the same speeds and
mode, and nothing saves or restores them when the movement changes. A locomotion whose row doesn't
use a field leaves it as it is.

**The State is shared with pose selection.** This domain writes `mode` in two places: back to
`Accelerating` when a deceleration reaches zero (`BaseSpeedDecelerate`), and in the `Stop` After
(§3). Pose selection (out of scope) writes the State when a new pose starts, between one frame's
motion and the next, and it writes it the same way whichever movement the pose belongs to. Its
writes, in domain terms:
- the on-ground turn starts: `TurningAround`; at the same moment extra_run is added into
  base_speed and extra_run becomes 0;
- the morph ball starts rolling the other way: `TurningAround`, with the same fold and a
  `CancelSpeedBoost`;
- the morph transition starts from a spin jump: `Decelerating`;
- several air poses start (jumping, falling, the wall jump): `Decelerating` or `Accelerating`;
- a pose change's command (pose selection picks it with the new pose):
  - *decelerate*: if base_speed is 0, as *stop*; otherwise extra_run is added into base_speed,
    extra_run = 0, mode = `Decelerating`, and `CancelSpeedBoost`;
  - *stop*: mode = `Accelerating` and `CancelSpeedBoost`; the speeds are kept;
  - *kill X speed*: base_speed = 0 and mode = `Accelerating`;
  - *kill run speed*: extra_run = 0 and `CancelSpeedBoost`;
- a landing (the air domain's) stops the speeds as its pose starts.

So the model reads the State as the last writer left it; a vector sets in `before` what pose
selection would have left. Every vector here that starts in `TurningAround` or `Decelerating`
stands for one of those events.

`on_square_slope` (an earlier draft's State field) belongs to the collision domain (§9). Motion
only reads the answer to "was she placed by a slope" (`on_slope`, §1.2).

### 1.2 Inputs (owned by other domains)

| Input | Type | From | Read by |
| --- | --- | --- | --- |
| `locomotion` | `Locomotion` (§2) | the pose, as it stands at the frame's start | `LocomotionAtFrameStart`, the motion table, `ArmPumpStep`, `StandingShotRestartsAnimation` |
| `facing` | `Facing { Left, Right, Forward }` | the pose at the frame's start | `HorizontalDirection`, `ArmPumpStep` |
| `radius` | x, y in px | the pose domain (in slice 0: x = 5; y = 0x15 standing, running and turning, 0x10 crouching and the ball transitions, 7 ball, 0x18 the elevator pose) | the collision requests (§5) |
| `pad` | buttons held: run, shoot | the controller | `ExtraRunSpeed`, `StandingShotRestartsAnimation` |
| `pose_request` | `PoseRequest { Forced, None, Pose { running: bool } }`: this frame's pose request as pose selection has settled it, after applying this frame's `vertical` (§6). `Forced` is a pose another system imposes (knockback, a finished transition) | pose selection | `ArmPumpStep` (the second entry point) |
| `airborne` | `bool`: her vertical motion is not none | the air domain | the `StepDownUnlessAirborne` rows |
| `air_vertical` | `Vertical` (§1.3): the air domain's result of its vertical move this frame | the air domain | the `StepDownUnlessAirborne` rows, when `airborne` |
| `elevator_moving` | `bool` | the elevator | `ElevatorRide` |
| `on_slope` | `bool`: the collision domain placed her by a slope (this frame's horizontal move seated her on one, or the last vertical stop was on a square slope) | collision | `StepDown` |
| `submerged` | `bool`: liquid physics apply to her (below a liquid's surface, without the gravity suit). **Fixed false for the pilot.** | the room's liquid | `SpeedTableRow`, `ExtraRunSpeed` |
| `speed_booster` | `bool`. **Fixed false for the pilot.** | equipment | `ExtraRunCap` |

`facing = Forward` reaches only the `FacingForward` row in vanilla play (the facing-forward poses
are the only ones without a side). Where a rule picks a side, Forward counts as **Right**
(`HorizontalDirection`, `ArmPumpStep`).

### 1.3 Results of the frame (Contacts, not State)

`step` returns these the same frame, except `ran_into_wall`, which `arm_pump_step` (the second
entry point, §6) returns later in the frame. None of them is read on a later frame by this domain.

| Contact | Type | Read by |
| --- | --- | --- |
| `wall_stop` | `bool`: `WallStop` fired this frame | `ArmPumpStep` (same frame, later); pose selection |
| `vertical` | `Vertical { None, Landed, Falling(FallKind), HitCeiling }`, after the row's After | pose selection (it settles `pose_request` from it before `ArmPumpStep`) |
| `ran_into_wall` | `bool`, from `arm_pump_step`: it asks for the "ran into a wall" pose | pose selection |
| `end_vertical_motion` | `bool`: the air domain's vertical speed, vertical direction and ball bounce reset to none (`TransitionLanding`) | the air domain |
| `restart_standing_animation` | `bool`: the standing animation restarts (timer = `standing_shot_timer`, frame 0) (`StandingShotRestartsAnimation`) | animation |

`FallKind { Airborne, MorphBall, SpringBall, NoChange }` tells pose selection which falling pose
to start. On the ground a landing never asks for a pose change, so `Landed` carries no kind.
`HitCeiling` is produced only through `air_vertical`: ground motion never moves up.

### 1.4 Read by: none (they leave the State)

MODEL decision 3 applies to each of these:

- **total speed** (base + extra run). It is computed and used inside one frame: every row that
  steps down steps horizontally first, so the step down always reads this frame's value
  (`StepDown`). It is a local value, not stored.
- **the X-speed-killed direction.** Its two codes (left / right) are never told apart; the only
  reader asks "was it killed". It is the `wall_stop` contact.
- **the distances moved** left, right, up and down. No code reads them.
- **the vertical result** as a stored value. It is cleared every frame; it is the `vertical`
  contact.
- **the speed divisor, the extra X and Y displacement and the deceleration multiplier.** Out of
  scope (§9).

---

## 2. Locomotion

`Locomotion` names the way Samus moves this frame. It comes from the pose at the frame's start
(`LocomotionAtFrameStart`). The pose codes for each variant are in §8.

| Variant | Poses (domain) | Fall kind |
| --- | --- | --- |
| `FacingForward` | standing, facing the screen | Airborne |
| `Standing { plain: bool }` | every other standing pose, including landing from a jump; `plain` for the two plain standing poses (no aim, not landing) | Airborne |
| `Running` | running, at every aim | Airborne |
| `Ball { spring: bool, at_rest: bool }` | the morph ball or spring ball on the ground; `at_rest` for the still ball pose of her facing, false for the rolling ones | `spring` false: MorphBall; true: SpringBall |
| `Crouching` | crouching, at every aim | Airborne |
| `TurningOnGround` | the on-ground turn | NoChange |
| `Transition` | the crouch, stand, morph and unmorph transitions, at every aim | NoChange |
| `Moonwalking` | moonwalking, at every aim | Airborne |
| `RanIntoWall` | the "ran into a wall" poses | Airborne |

The **Fall kind** column is the `FallKind` a `Falling` result carries (`LandingContact`).

---

## 3. Motion table

The row key is (`locomotion`, `mode`): every locomotion has one row, except `Ball`, whose
`at_rest && mode == Accelerating` case is its own row (`BallAtRest`).

```
enum Horizontal { Speed, Coast, None }
enum VerticalStep { StepDown, StepDownUnlessAirborne, Elevator }
enum After {
    Keep,
    Stop { cancel: bool },
    StopUnlessAirborne,
    EndTurn,
    EndTransition,
    NoVertical,
}
```

- **Horizontal.** `Speed`: `ExtraRunSpeed`, then `BaseSpeedAccelerate` or `BaseSpeedDecelerate`
  by `mode`, then `HorizontalStep`. `Coast`: `HorizontalStep` with the base speed taken as 0, so
  only `extra_run` moves her; base_speed and mode are unchanged. `None`: no horizontal move.
- **VerticalStep.** `StepDown`: `StepDown`. `StepDownUnlessAirborne`: if `airborne`,
  `vertical = air_vertical` and no step down; else `StepDown`. `Elevator`: `ElevatorRide`.
- **After**, applied after the vertical step:
  - `Keep`: nothing.
  - `Stop { cancel }`: if `cancel`, `CancelSpeedBoost`; then extra_run = 0, base_speed = 0, mode
    = Accelerating.
  - `StopUnlessAirborne`: `Stop { cancel: true }` if not `airborne`; else nothing.
  - `EndTurn`: `CancelSpeedBoost`; extra_run = 0; vertical = None (`TurnIgnoresFall`).
  - `EndTransition`: `TransitionLanding`; then vertical = None.
  - `NoVertical`: vertical = None.

| Row | Horizontal | VerticalStep | After |
| --- | --- | --- | --- |
| `FacingForward` | None | Elevator | NoVertical |
| `Standing` | Coast | StepDown | Stop { cancel: true } |
| `Running` | Speed | StepDown | Keep |
| `BallAtRest` (`Ball`, at_rest, mode Accelerating) | Coast | StepDownUnlessAirborne | StopUnlessAirborne |
| `Ball` (otherwise) | Speed | StepDownUnlessAirborne | Keep |
| `Crouching` | Coast | StepDown | Stop { cancel: false } |
| `TurningOnGround` | Speed | StepDown | EndTurn |
| `Transition` | Coast | StepDownUnlessAirborne | EndTransition |
| `Moonwalking` | Speed | StepDown | Keep |
| `RanIntoWall` | Coast | StepDown | Stop { cancel: true } |

Every row moves her, including `Crouching`, `Transition`, `Moonwalking`, the spring ball and
`RanIntoWall`: each runs its horizontal step and its vertical step every frame (the coasting rows
by extra_run, which is 0 unless momentum carried it in), and the rows whose After is `Keep` keep
the speeds pose selection left (§1.1). Source: `$90:A573`, `$90:A61C`, `$90:A694`, `$90:A69F`,
`$90:A75F`.

The spring ball and the morph ball share their rows; they differ only in which poses are at rest
and in their fall kind. `StandingShotRestartsAnimation` and `ArmPumpStep` are not row columns:
each has its own condition.

---

## 4. Behaviours

Every entry is **Fixed** except `SpeedCap`, the one Toggle (MODEL §3). "Parameter" values live in
`data` tables or `Rules`.

### Numeric rules

These hold for every entry below.

- **`below(a, b)`**, for two 16-bit words: bit 15 of (a − b) mod 2^16 is set. It is a signed
  compare of the difference, the same for pixel and subpixel words. Every "below" in this spec
  means it. Worked examples:
  - 0x1000 is **not** below 0xC000: 0x1000 − 0xC000 = 0x5000, bit 15 clear (`SpeedCap`, V3).
  - 0xC000 is below 0x4000: 0xC000 − 0x4000 = 0x8000, bit 15 set (`SpeedCap`, V5).
  - 0xC000 is below 0x0000: 0xC000 − 0 = 0xC000, bit 15 set (`TurnCreep`, V6).
  - 0x8000 is below 0x0000 (`ExtraRunCap`, V11a); 0xF000 is below 0x0000 too (V11b).
  - Equal words are never below each other.
- **negative(w)**: bit 15 of the word w is set. A displacement moves right when its pixel word is
  not negative.
- **Adding to a P.S value** adds the subpixels mod 2^16 and carries into the pixel word, which
  wraps mod 2^16. Positions wrap mod 2^32.

### LocomotionAtFrameStart
- kind: Timing. summary: on the frame Samus changes pose, she still moves the way her old pose moved.
- Rule: the row, the speed-table entry and the fall kind come from the locomotion at the frame's
  start. The pose update later in the frame does not re-run motion.
- Params: none. switch: Fixed. checked: oracle (V6, V9, V13, V14).
- Source: `$90:A337`.

### SpeedTableRow
- kind: Mechanic. summary: each way of moving has its own acceleration, top speed and braking.
- Rule: the entry for the locomotion gives (accel, max, decel), each P.S. With `submerged` false
  (the pilot), the normal table applies.
- Params, the normal table (`data::movement::NORMAL_SPEEDS`), as accel / max / decel:

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

  Only `Speed` rows read the table. The rows for Standing, FacingForward, Crouching, Transition and
  RanIntoWall exist in the ROM but are never read by ground motion, because those rows coast or
  don't move (Transition's max 1.4000 never applies). They stay in the table for byte-exact data.
- switch: Parameter. checked: oracle (V1-V6, V10).
- Source: `$90:9BD1`.

### BaseSpeedAccelerate
- kind: Mechanic. summary: Samus speeds up by the row's acceleration each frame, up to its top speed.
- Rule: when mode = Accelerating: base_speed += accel, then `SpeedCap`.
- Params: the row's accel and max. switch: Fixed. checked: oracle (V1-V5, V10).
- Source: `$90:9A7E`.

### SpeedCap
- kind: Quirk. summary: speed briefly overshoots, or snaps *up* to, the top speed.
- Rule, `SpeedCap::Wordwise` (vanilla), after the add:
  - `below(base.pixel, max.pixel)`: keep.
  - else, if the pixel words differ: base = max.
  - else (pixels equal): keep if the subpixels are equal or `below(base.sub, max.sub)`; else
    base = max.

  Consequences in the oracle:
  - Running: 1.E000 + 0.3000 = 2.1000. Pixels equal; 0x1000 is not below 0xC000, so base = 2.C000:
    a free 0.B000 (V3).
  - Ball: 3.0000 + 0.C000 = 3.C000. Pixels equal; 0xC000 is below 0x4000, so it is kept one frame
    above the max 3.4000. Next frame 4.8000 caps (V5).
  - Turning (max 0): 0 + 0.C000: pixels equal, 0xC000 below 0, kept; then 1.8000 caps to 0
    (`TurnCreep`, V6).
  - Moonwalking (max 0.8000): 0 + 0.C000 = 0.C000: 0xC000 is not below 0x8000, so it caps to
    0.8000 (V18).
- Rule, `SpeedCap::Exact`: base = min(base, max), comparing the whole 32-bit values unsigned.
- `SpeedCap` covers the base speed only. The extra run speed has its own cap (`ExtraRunCap`),
  which doesn't follow this switch.
- Params: the row's max. switch: **Toggle** (`Wordwise` vanilla | `Exact`). checked: oracle (V3,
  V5, V6) for `Wordwise`; `Exact` is not compared (V3, V4, V5 and V6 have `Exact` variants).
- Source: `$90:9A99`.

### TurnCreep
- kind: Quirk. summary: during the on-ground turn from standing, Samus slides 0.C000 px every other frame.
- Rule: none of its own: `SpeedCap::Wordwise` on the turning row. Base alternates 0, 0.C000, 0. She
  moves along her facing, because mode is back to Accelerating (`HorizontalDirection`). Under
  `Exact` there is no creep.
- Params: none. switch: follows `SpeedCap`. checked: oracle (V6).
- Source: `$90:A67C`.

### BaseSpeedDecelerate
- kind: Mechanic. summary: when Samus brakes or turns, her speed falls by the row's deceleration.
- Rule: when mode = Decelerating or TurningAround: base_speed −= decel. If the pixel word is then
  negative, base_speed = 0 and mode = Accelerating. No cap applies.
- Params: the row's decel. switch: Fixed. checked: oracle (V6, first frame); original only for a
  decelerate that doesn't reach zero (V15, V16, constructed).
- Source: `$90:9ABA`.

### ExtraRunSpeed
- kind: Mechanic. summary: holding run while running adds speed, up to 2 px a frame; a kept momentum carries it into other rows.
- Rule, on every `Speed` row:
  - *Running* means: not `submerged`, locomotion is `Running`, and run is held. Then momentum =
    true, and `ExtraRunCap` decides the new extra_run.
  - Otherwise: if momentum is false, extra_run = 0; else extra_run is kept.
- Params: none of its own (the step and cap are `ExtraRunCap`'s). switch: Fixed.
- checked: oracle (V1-V4, V8); original only for momentum kept on a non-running row (V17,
  constructed).
- Source: `$90:973E`.

### ExtraRunCap
- kind: Quirk. summary: (invisible in vanilla play) the run-speed cap is tested before the add, word by word.
- Rule: when running, test the *old* extra_run against the cap:
  - `below(extra_run.pixel, cap.pixel)`: add the step;
  - otherwise, `below(extra_run.sub, cap.sub)`: add the step;
  - otherwise: extra_run = cap.

  So the test happens before the add (extra_run can end a frame one step above the cap), and a
  pixel word *above* the cap still adds when its subpixel is below. 2.8000 grows to 2.9000 (0x8000
  is below 0); 2.F000 grows to 3.0000, and the next frame snaps to 2.0000 (0 is not below 0). From
  0 in steps of 0x1000 she lands exactly on 2.0000, so vanilla play never shows it.
- Params: step 0.1000; cap 2.0000 (7.0000 with the speed booster, deferred). switch: Fixed; it
  does not follow `SpeedCap`. checked: original only (V11, constructed).
- Source: `$90:97D5`.

### CancelSpeedBoost
- kind: Mechanic. summary: run speed carries through a jump and is lost when she stops.
- Rule: momentum = false (the boost counter and palette also reset; deferred). It runs in the
  `Stop { cancel: true }`, `StopUnlessAirborne` and `EndTurn` Afters, and in `WallStop`.
- Params: none. switch: Fixed. checked: oracle (V8, V14, V25); original only (V12, V15).
- Source: `$91:DE53`.

### HorizontalStep
- kind: Mechanic. summary: Samus moves sideways by her total speed.
- Rule: total = (base_speed, or 0 on a `Coast` row) + extra_run. The heading is the side
  `HorizontalDirection` picks; the displacement is +total heading right or −total heading left,
  then `DisplacementClamp`. Motion asks the collision interface for a horizontal move with that
  heading and displacement, testing solid enemies (§5), and applies the result.
- A total of 0 still asks. No block can stop a zero move, but a solid enemy on the heading side
  can, so a still Samus pressed against a solid enemy gets a `WallStop` every frame (and
  `EnemyTouchZeroesYSubpixel`). The heading, not the displacement's sign, names the side: a
  turning Samus with base_speed 0 tests the side she slides toward, against her facing (V26d).
- Params: none. switch: Fixed. checked: oracle (V1-V10, V14); original only for a zero move
  against an enemy (V26, V26b, V26c, V26d, constructed).
- Source: `$90:8EA9`.

### HorizontalDirection
- kind: Mechanic. summary: during a turn, Samus keeps sliding the way she was going.
- Rule: read **after** this frame's `BaseSpeedDecelerate`. If mode is TurningAround, she moves
  leftward when facing Right and rightward otherwise. In the other modes, she moves leftward when
  facing Left and rightward otherwise. So a decelerate that reaches zero this frame also turns the
  step to her facing, which matters when extra_run is not zero (V15b).
- Params: none. switch: Fixed. checked: oracle (V6); original only for a moving turn (V15,
  constructed).
- Source: `$90:8EA9`.

### DisplacementClamp
- kind: Mechanic. summary: Samus never moves more than 15 px sideways in a frame.
- Rule: if the displacement's pixel word is not negative and not `below` 16, it becomes 15; if it
  is negative and `below` −15 (0xFFF1), it becomes −15. The subpixel word is kept. So right,
  10.4000 becomes 0F.4000. Left, −10.4000 (FFEF.C000) becomes FFF1.C000, which is −0E.4000:
  leftward, she moves at most 15 − sub.
- Unreachable on the ground in vanilla (the largest total is 9.C000, with the speed booster), and
  reachable through the deferred pushes (§9).
- Params: `max_displacement` = 15 px. switch: Parameter. checked: original only (V12, constructed).
- Source: `$90:E4AD`.

### WallStop
- kind: Mechanic. summary: running into a wall stops her dead, flush against it.
- Rule: if the horizontal move stopped: `CancelSpeedBoost`; extra_run = 0, base_speed = 0, mode =
  Accelerating; `wall_stop` = true. The step down still uses this frame's total (`StepDown`).
- Params: none. switch: Fixed. checked: oracle (V8); original only leftward (V20, constructed).
- Source: `$90:E5CE`.

### StepDown
- kind: Mechanic. summary: walking off a ledge or down a slope, Samus sticks to the ground beneath her.
- Rule: motion asks the collision interface for a downward move (§5) by:
  - `step_down_extra` (1.0 px), if `on_slope`;
  - otherwise total + `step_down_extra`, where total is the one `HorizontalStep` computed this
    frame, before `WallStop` zeroed the speeds and unclamped.

  A stop gives `Landed`, a move the whole way gives `Falling(fall kind of the locomotion)`.
- Params: `step_down_extra` = 1.0 px. switch: Parameter. checked: oracle (V7, V10, V13); original
  only for the total after a wall stop (V21, constructed).
- Source: `$90:923F`.

### BallAtRest
- kind: Mechanic. summary: the still morph ball sheds any speed it has, unless it's in the air.
- Rule: the `BallAtRest` row (§3) is chosen when the locomotion is `Ball` with `at_rest` and mode
  is Accelerating. A `Ball` in another mode takes the `Ball` row and keeps braking or turning, even
  in an at-rest pose.
- Params: none. switch: Fixed. checked: oracle (V10); original only for a braking ball at rest
  (V16, constructed) and an airborne one (V19b, constructed).
- Source: `$90:A521`.

### TurnIgnoresFall
- kind: Quirk. summary: a turning Samus sinks onto lower ground instead of falling.
- Rule: the `EndTurn` After clears the vertical result after its step down, so a `Falling` step
  down doesn't start a fall. She sinks by total + 1.0 px a frame until she lands (V7: three
  frames).
- Params: none. switch: Fixed. checked: oracle (V7).
- Source: `$90:A68F`.

### TransitionLanding
- kind: Mechanic. summary: landing during a crouch or morph transition ends the vertical motion.
- Rule: in the `EndTransition` After: if vertical is `Landed`, `end_vertical_motion` = true.
- Params: none. switch: Fixed. checked: oracle (V13, a step-down landing); original only for an
  airborne landing and for no landing (V13b, constructed).
- Source: `$90:A640`.

### LandingContact
- kind: Mechanic. summary: falling off the ground tells pose selection which fall to start.
- Rule: a `Falling` result carries the fall kind of the frame-start locomotion (§2). A landing
  carries none.
- Params: the §2 fall-kind column (`data`). switch: Fixed. checked: oracle (V9, V10, V25 for `Landed`);
  original only for each fall kind (V19, V19-spring, V21b, constructed).
- Source: `$90:E61B`.

### ElevatorRide
- kind: Mechanic. summary: on a moving elevator, Samus facing the screen rides it down.
- Rule: no horizontal move and no speed change. If `elevator_moving`: one downward move by
  `elevator_step` with heading `Up` and no solid-enemy test (§5, `ElevatorHeadsUp`). If not,
  nothing moves. The row's After makes vertical None either way.
- Params: `elevator_step` = 1.0 px. switch: Fixed. checked: original only (V23, constructed).
- Source: `$90:A392`.

### ElevatorHeadsUp
- kind: Quirk. summary: (invisible in the slice) the elevator's step down tells the blocks it is going up.
- Rule: `ElevatorRide`'s move goes down by `elevator_step` but its request's heading is `Up`
  (§5). Where she moves and which blocks stop her follow the distance's sign (down); only the
  reactions that ask for the heading (slopes and the special blocks, the collision domain's) react
  as if she moved up. The elevator shafts the slice reaches hold only air, solid, door,
  extension and scroll-trigger blocks, which don't ask (item-morph-ball:0-681), so no record
  shows it.
- Params: none. switch: Fixed. checked: original only.
- Source: `$90:A397`.

### StandingShotRestartsAnimation
- kind: Mechanic. summary: shooting while standing plainly restarts the standing animation.
- Rule: if the locomotion is `Standing { plain: true }` and shoot is held:
  `restart_standing_animation` = true.
- Params: `standing_shot_timer` = 0x10 frames (for the animation domain). switch: Fixed. checked:
  original only (V24, constructed).
- Source: `$90:A3AD`.

### FloorFlushSubpixel
- kind: Quirk. summary: (invisible) flush against a floor her vertical subpixel is FFFF; against a wall, FFFF or 0 by side.
- Rule: part of the collision interface's contract (§5). A move stopped by a solid block leaves
  the subpixel on that axis at FFFF when she moved right or down, 0 when she moved left or up
  (`Blocker::Block { subpixel }`).
- Params: none. switch: Fixed. checked: oracle (V7, V8); original only leftward (V20,
  constructed).
- Source: `$94:8F82`.

### EnemyTouchZeroesYSubpixel
- kind: Quirk. summary: (invisible) pressing against a solid enemy, from any side, sets her vertical subpixel to 0.
- Rule: part of the collision interface's contract (§5). When a solid-enemy test finds an enemy
  she already touches on the tested side, her Y subpixel becomes 0, whatever the axis tested: a
  horizontal move (`Blocker::Enemy { touching: true }`), a downward move, and the arm-pump's enemy
  test (`EnemyHit { touching: true }`). Motion writes the 0; the distance is 0 in every touching
  case. An enemy found further away (not touching) writes nothing.
- Params: none. switch: Fixed. checked: oracle for a move onto an enemy (item-morph-ball:1228, the
  air domain's landing on the solid enemy: y 029B.9400 becomes 029B.0000 with no pixel moved; the
  ground rows on it, V1 and V2, keep 0); original only for the horizontal move and the arm-pump
  test (V26, V26d, V27, constructed).
- Source: `$A0:AAC8`.

### ArmPumpStep
- kind: Quirk. summary: changing aim while running moves Samus an extra pixel forward.
- Entry: `arm_pump_step`, the domain's second entry point, separate from `step`. Pose selection
  calls it once per frame, after `step` and after it has settled `pose_request` from this
  frame's `vertical` (§6), and before it starts the new pose. It reads the State's `position`,
  the frame-start `locomotion` and `facing` (the pose hasn't changed yet), `radius`, this frame's
  `wall_stop` contact and `pose_request`. It writes only `position` (x by the move, and the Y
  subpixel through `EnemyTouchZeroesYSubpixel`), and returns `ran_into_wall`. It changes no speed
  and no mode.
- Rule:
  - `pose_request` is `Forced`: nothing (no test, no move; `ran_into_wall` = false);
  - else if `wall_stop` and the frame-start locomotion is `Running`: `ran_into_wall` = true; no
    step. This holds whatever the request, `None` included;
  - else if `pose_request` is `Pose { running: true }`: forward is left when facing Left and right
    otherwise. Motion asks the enemy test for `arm_pump_step` forward (§5): a hit gives
    `ran_into_wall` = true and she doesn't move (a touching hit zeroes her Y subpixel). Otherwise
    one horizontal move by `arm_pump_step` forward, with no solid-enemy test and no slope seat
    (§5, `SkipEnemies`); a stop sets `ran_into_wall` = true (and leaves her flush, as every stop
    does). There is no `WallStop`, no step down and no speed change;
  - otherwise nothing.

  With no step down and no slope seat after it, it can lift her off a downward slope. Because the
  request is settled after `vertical`, running off a ledge while the aim changes gives no step:
  the fall replaces the running request with a falling one.
- Params: `arm_pump_step` = 1.0 px. switch: Fixed. checked: oracle (V2-V4, V9, V14; V1 shows no
  step without a pose change; V8 the wall branch); original only for a block stop (V22b,
  constructed), Forward (V23), an enemy hit (V27, constructed), the `Forced` request and the
  fall replacing the request.
- Source: `$91:EADE`.

---

## 5. Collision interface

Motion never looks at blocks. It asks the collision domain three things, each with the **body**:
her position and her radius (x, y).

| Request | Asks | Gets back |
| --- | --- | --- |
| `move_horizontal(body, heading, d, enemies)` | a horizontal move by the signed P.S displacement `d`, heading `Left` or `Right` | `Moved { distance, stop: Option<Stop> }` |
| `move_down(body, heading, d, enemies)` | a downward move by the P.S distance `d` ≥ 0, heading `Down` or `Up` | `Moved { distance, stop: Option<Stop> }` |
| `enemy_ahead(body, heading, d)` | is a solid enemy within `d` on the `Left` or `Right` heading? | `Option<EnemyHit { touching: bool }>` |

- **The heading** is the side the request faces, given apart from the distance's sign. It names
  the side the solid-enemy test looks at, and the side heading-aware block reactions (slopes,
  special blocks; the collision domain's) react to. The distance's sign decides where she moves
  and the block sweep's leading edge.
  - Horizontal: the heading is the side `HorizontalDirection` picked (or the arm-pump's forward).
    In this domain `d`'s sign agrees with it whenever `d` ≠ 0. A zero `d` tests no block but still
    tests solid enemies on the heading (`HorizontalStep`).
  - Down: `Down` for every step down; `Up` for `ElevatorRide`'s (`ElevatorHeadsUp`).
- `enemies` is `TestEnemies` or `SkipEnemies`. Every move here tests solid enemies, except
  `ElevatorRide`'s and `ArmPumpStep`'s (which asked `enemy_ahead` first). A horizontal
  `TestEnemies` move ends with the slope seat (§9, it feeds `on_slope`); a `SkipEnemies` one
  doesn't.
- `distance` is the signed displacement actually made. Motion adds it to the position on that
  axis.
- `Stop { side: Side, blocker: Blocker }`, with `Side { Left, Right, Below }`, says she was
  stopped and on which side; `Blocker` says by what, and which subpixel motion writes before
  adding `distance`:
  - `Block { subpixel: u16 }`: a solid block; motion writes `subpixel` to that axis's subpixel:
    FFFF for `Right` and `Below`, 0 for `Left` (`FloorFlushSubpixel`).
  - `Enemy { touching: bool }`: a solid enemy. When `touching`, `distance` is 0 and motion writes
    0 to the **Y** subpixel, on either axis (`EnemyTouchZeroesYSubpixel`). Otherwise she moves up
    to the enemy and nothing is written.
  - `Slope`: a slope; nothing is written (the collision domain defines it).
- `EnemyHit { touching }` follows the same rule: when `touching`, motion writes 0 to the Y
  subpixel.
- A horizontal stop, by any blocker, is "the horizontal move stopped" for `WallStop`. A downward
  stop is `Landed`.
- `on_slope` (§1.2) comes from the same domain.

The vectors use a flat stand-in for the collision domain: block-aligned solid floor, walls and
ceiling, described in the vector file's header. It is enough for every vector here; the collision
domain's own spec replaces it.

---

## 6. Order that matters

Everything else in the frame is free (MODEL §1). These orders are visible at the frame's end:

1. The row is chosen from the locomotion at the frame's start, before the pose update.
2. `ExtraRunSpeed` comes before the base speed, and both before the horizontal step: the step uses
   the updated speeds.
3. `HorizontalDirection` reads mode after this frame's `BaseSpeedDecelerate` (V15b).
4. The horizontal step (with `WallStop`) comes before the step down, which reads this frame's
   total (V21).
5. The After runs after the step down: `Standing`, `Crouching` and `RanIntoWall` move by the old
   extra_run this frame, then zero it (V14).
6. `ArmPumpStep` is not part of `step`. The frame runs, in this order:
   1. pose selection reads the pad and makes its first pose request (before motion);
   2. `step`: the row's motion, `ElevatorRide` included, giving `wall_stop` and `vertical`;
   3. animation and the hit check (other domains; a hit or a finished transition can force a pose);
   4. pose selection applies `vertical` to its request: `Falling` replaces it with the falling
      pose of its fall kind; `HitCeiling` replaces it with the current pose; `Landed` on the
      ground leaves it. That settles `pose_request`;
   5. `arm_pump_step` with this frame's `wall_stop` and the settled `pose_request`;
   6. pose selection starts the requested pose (or "ran into a wall", if `ran_into_wall`).

   So the arm-pump step sees this frame's motion, and its trigger depends on this frame's
   vertical result. Source: `$90:E725`, `$91:E8B6`, `$91:EB88`.

---

## 7. Answers to the implementer

1. **Crouching, moonwalking and the spring ball.** They all run on the ground path.
   - `Crouching`: `Coast` horizontally (only extra_run moves her), then `StepDown` unconditionally
     (no airborne test), then `Stop { cancel: false }`. It does **not** cancel momentum. V13's
     second frame shows the zeroing.
   - `Moonwalking`: the `Speed` row with its own table entry (max 0.8000), then `StepDown`, then
     `Keep`.
   - The spring ball: the morph ball's rows exactly, with the spring-ball poses as `at_rest`, and
     its own fall kind.
2. **The speed divisor and the extra X and Y displacement are out of scope** (§9).
3. **Two caps.** The base speed's cap (`SpeedCap`) adds, then compares, and snaps when the pixel
   word is above. The extra run's cap (`ExtraRunCap`) compares before the add, and still adds when
   the pixel word is above but the subpixel is below. `SpeedCap::Exact` doesn't change the extra
   run. V11 runs under both settings with the same result.
4. **Crouching, Transition, Moonwalking, the spring ball and RanIntoWall have motion** (gap G5).
   Each row in §3 runs every frame, as in the original; the oracle judges any change they cause.
5. **One State across every movement** (gap G6). The speeds and mode are one State, written by
   this domain and by pose selection's commands whichever movement is current (§1.1).
6. **The arm-pump step's order** (gap G1): §6 item 6 and `ArmPumpStep`'s entry point.
7. **A zero horizontal move** (gap G2) tests solid enemies on its heading (`HorizontalStep`, §5).
8. **The Y subpixel** (gap G3): `EnemyTouchZeroesYSubpixel`, reported through `Blocker::Enemy`
   and `EnemyHit`.
9. **The elevator's heading** (gap G4): `Up` (`ElevatorHeadsUp`); it changes nothing the slice
   reaches.

---

## 8. Decode notes

ROM codes, for `verify`'s decode and the `data` tables only. The model never names them.

- **mode:** 0 Accelerating, 1 TurningAround, 2 Decelerating.
- **facing** (the pose's direction byte): 4 Left, 8 Right, 0 Forward.
- **momentum:** a word, 0 or 1.
- **Locomotion from the pose** (movement type, then pose):
  - type 0 Standing: poses 00 and 9B are `FacingForward`; 01 and 02 are `Standing { plain: true }`;
    the rest (03-08, 47, 48, A4-A7, D5, D6, E0-E7) are `Standing { plain: false }`.
  - type 1 `Running`: 09-12, 45, 46.
  - type 4 `Ball { spring: false }`: `at_rest` 1D (right) and 41 (left); rolling 1E and 1F.
  - type 5 `Crouching`: 27, 28, 71-74, 85, 86, D9, DA.
  - type E `TurningOnGround`: 25, 26, 43, 44, 8B-8E, 9C, 9D, BF-C4.
  - type F `Transition`: 35-38, 3B-3E, F1-FC. Poses 39, 3A, 3F, 40 and DB-DE are unused; the
    original gives DB-F0 no vertical step.
  - type 10 `Moonwalking`: 49, 4A, 75-78.
  - type 11 `Ball { spring: true }`: `at_rest` 79 and 7A; rolling 7B and 7C.
  - type 15 `RanIntoWall`: 89, 8A, CF-D2.
- **the vertical result:** 0 none, 1 landed, 2 falling, 4 hit ceiling (5 is the wall jump's). The
  second byte is the pose request, from two tables by movement type (one for falling, one for
  landed): falling 0 Airborne, 1 MorphBall, 3 SpringBall, 4 NoChange; landed is 4 (no change) for
  every ground movement type.
- **the X-speed-killed flag:** 0, 4 (left) or 8 (right); `wall_stop` = non-zero.
- **the radius in the records** lags the pose by one frame: the radius used by frame N's motion is
  the one in record N.
- **pad:** run is B, shoot is X by default. The vectors also name left, right, jump (A) and
  aim_down (L) for the pose update.

**RAM shape** (MODEL §5): the State fields whose decode is one word copied as-is.
- `position` x and y: the pixel and subpixel words are the quantity itself.
- `base_speed`, `extra_run`: the speed is its P.S value; no smaller form preserves the cap quirks.
- `momentum`: one bool, already the domain's value.
- `mode` is decoded to an enum, so it isn't copied as-is.

---

## 9. Deferred

### To the collision domain

What is known, recorded for that domain's spec. Not checked further here.

- **The horizontal sweep.** Leading edge: x + d + rx − 1 moving right (d's pixel word not
  negative), x + d − rx moving left, pixel words. One column, edge >> 4. Rows from her top
  ((y − ry) >> 4) to her bottom ((y + ry − 1) >> 4), top first; the first block that collides
  ends the scan. Solid: right, d = max(0, (edge & ~F) − rx − x).0000 and x.sub = FFFF; left,
  d = min(0, (edge | F) + 1 + rx − x).0000 and x.sub = 0.
- **The vertical sweep.** Edge y + d + ry − 1 down, y + d − ry up; one row, so a move of 16 px or
  more can pass a floor (long-sweep tunnelling; V12's 11.4000 step down reaches past the floor's
  first row, so V12 doesn't compare y). Columns (x − rx) >> 4 to (x + rx − 1) >> 4, left to right
  on even frames and right to left on odd ones (the first collision decides). Solid: down,
  d = max(0, (edge & ~F) − ry − y).0000, y.sub = FFFF; up, d = min(0, (edge | F) + 1 + ry − y).0000,
  y.sub = 0.
- **Non-square slopes.** Horizontally they never stop her, and each one in the leading column,
  not upside down, while her vertical speed is 0, scales d by a per-shape factor (12, 14, 15 →
  0xC0; 16, 17 → 0xD8; 1B, 1C → 0x80; E, F → 0xB0; others 0x100; over 256), done on d reduced to
  8.8. Vertically, only the column under her centre; A = surface(column) − (edge mod 16) − 1, and
  if A ≤ 0, d = max(0, d.pixel + A).0000 with y.sub unchanged.
- **The slope seat** after every horizontal move that tests solid enemies (not after the
  arm-pump's): feet on a non-square slope raise her by A and count as seated; an upside-down slope at her head pushes her down. It feeds `on_slope`.
- **Square slopes:** quadrant tables, aligned to 8 px; a vertical stop on one sets
  `on_square_slope` (State of that domain), read by the next frame's `on_slope`. Each vertical
  move clears it.
- **The heading.** The vertical slope reactions (square and non-square) and the special blocks'
  reactions branch on the request's heading, not on the distance's sign; the solid, door, spike
  and bomb reactions use the sign. The elevator's request says `Up` while it moves down
  (`ElevatorHeadsUp`).
- **The solid-enemy test.** It looks on the heading only, with |d| (the magnitude). The target
  pixel is the pixel word of her position moved |d| on the heading, then one pixel further on the
  heading: rightward unless x.sub and |d|.sub are both 0; leftward when x.sub ≠ |d|.sub. So a zero
  move finds an enemy she touches only when x.sub ≠ 0 (V26c). An enemy is found when the target,
  with her current y, overlaps it on both axes (|centre difference| < radius sum, pixel words);
  then, on the heading: her edge (x + rx right, x − rx left, from her current x) equal to the
  enemy's near edge (its x ∓ its radius) is touching: distance 0 and Y subpixel 0; short of it,
  the distance is the gap; past it (already inside), the enemy is skipped. A vertical test is the
  same on y, with her current x.
- **A horizontal move with no collision clears the vertical result** (before any step down). No
  ground row reads it in between; the air domain's rows may.
- Doors, spikes, special and bombable blocks: their own reactions. In the elevator room the floor
  row is a door-type block that behaved as solid (V7).
- Out-of-room block lookups: not read.
- An airborne leftward stop at item-morph-ball:711 ended with x.sub FFFF, not 0. It is an air
  frame (spin jump, near a wall-jump check), so it doesn't touch this spec; the collision or air
  observer should explain it.

### Liquids and the speed booster

- `submerged` picks the water or the lava/acid table in `SpeedTableRow`; the water
  table gives Running and Ball accel 0.0400, max 2.C000, decel 0.0800. The other rows and the
  lava/acid values were not read. Nothing in motion chooses the normal table back; the block-inside
  check resets it when her bottom point is in air or a solid block, but not in a slope block, so
  leaving liquid onto a slope may keep the water table (if reachable, the table choice becomes
  State: `SpeedTableKind { Normal, Water, LavaAcid }`).
- A submerged Samus is "not running" in `ExtraRunSpeed`, so the water and lava columns of the
  extra-run table are never used.
- The speed booster: cap 7.0000 in `ExtraRunCap`, the boost counter and timer reset when running
  starts, and the palette; `CancelSpeedBoost` resets them.

### Pushes from other domains

- **Extra X displacement** is added to ±total before the clamp. If the sum's sign disagrees with
  the direction the step started in, the collision flag is cleared and nothing is killed.
- **Extra Y displacement**, when non-zero, replaces the step down: down by it + 1.0 when its pixel
  word is not negative, up by its magnitude when negative.
- **Speed divisor** n (capped at 4): total = (base + extra_run) >> n, exact only for pixel sums
  below 256.
- **Deceleration multiplier** m (non-zero): the decel step is (decel.sub high byte × m) as the
  subpixel and the high byte of (decel.pixel low byte × m) as the pixel.

---

## 10. Open questions

1. **A skipped pose check.** The X-speed-killed flag is cleared by pose selection's wall check,
   which is skipped when another system forces a pose that frame. Then the flag survives to the
   next frame's check. It needs a wall stop and a forced pose on the same frame; never seen in the
   records.
2. **Moonwalking's direction.** Moonwalking moves backwards in play, but its row moves along the
   facing unless mode is TurningAround, and the moonwalk pose's start writes no mode. Which pose
   change gives her the backward step was not read (pose selection). V18 checks only the speed.
3. **The elevator descent** (item-morph-ball:0-680) moves her 1 and 2 px on alternate frames with
   the subpixel 0, so the elevator itself seems to set her y; `ElevatorRide`'s 1.0 can't be told
   apart there. V23 is constructed.
4. **The arm-pump step's facing** is the frame-start pose's facing, not the new pose's. They agree
   in every record.
