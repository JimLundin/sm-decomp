# Samus moving through the world: behaviour spec

What the player sees Samus do when she runs, stops, turns, crouches, rolls, jumps, spins, falls,
wall-jumps and lands, and how the world's blocks, slopes, solid enemies, floors, walls and
ceilings stop her. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-motion.toml`](../../scenarios/vectors/samus-motion.toml).

## Header

**Scope (Jim, 2026-11-07).** One domain, because the player sees one thing: Samus moving through
the world. It covers her speeds and position, how her pose follows from input and the world where
that changes her speeds, and her collision with blocks, slopes and solid enemies.

**Not in this spec yet** (each an input where it feeds us; §11): which exact sprite pose follows
which input (the pose chart), pose fit (the push when her body grows), animation timing, liquids,
the speed booster's extra speed, knockback, bomb jumps, grapple, shinespark, pushes from the room
(Ceres, conveyors, quicksand), doors, spikes and special blocks, and slopes' sideways speed change.

**Baseline: about 3,600 lines.** The old motion code (about 950: `movement.rs` 516,
`movement/air.rs` 434), the old pose selection (`pose.rs` and `pose/`, 1,126) and the old collision
(`collision.rs` and `collision/`, 1,383, plus the model's collision 140), counted with `wc -l`,
tests excluded.

**Target: 1,600 lines** for the new model, tests and data tables excluded: motion about 500,
landings and pose changes' speed effects about 150, the pose chart about 450 (as data records where
it can be), collision with blocks and solid enemies about 400, slack 100. Over the target is a
review stop; over the baseline a strict failure.

**Notation.** A quantity written `P.S` is a pixel part and a subpixel part, each 16 bits, in hex:
`2.C000` is 2.75 px. A signed one has a sign in front: `-4.E000` is 4.875 px a frame upward.
Positions are her body's centre; x grows rightward, y grows downward. A frame reads the pad the
player holds when it starts.

---

## 1. What the player can tell about Samus (State)

Every quantity here is something the player sees or can measure frame by frame. Nothing else is
State.

| Quantity | Units, type | Why the player can tell | Read by |
| --- | --- | --- | --- |
| `position` | x, y: `P.S`; wraps mod 2^32 | where she is drawn | every move; output |
| `run_speed` | `P.S` ≥ 0 | how fast she walks, runs or drifts | `SpeedingUp`, `Braking`, `TurnSkid`, `AirDrift`, `SpinDrift`, `Landing`, `PoseChangeEffects` |
| `dash` | `P.S` ≥ 0 | the extra speed from holding run; it carries into a jump or the ball (the mockball) | `Dash`, `DashCap`, `JumpStarts`, `TurnSkid` |
| `dash_kept` | `bool` | whether the dash survives her next movement: after a halt it is gone the first time she moves again; a kept dash carries on | `Dash`, `LosingTheDash` |
| `drive` | `Drive { Driven, Coasting(Along) }`, `Along { Facing, Behind }` | whether her horizontal motion follows the pad (builds while held; in the air ends when let go) or carries on by itself: sliding to a stop, carried through a jump, or skidding backwards through a turn (`Behind`: moving against her facing) | `SpeedingUp`, `Braking`, `TurnSkid`, `AirDrift`, `CarriedDrift`, `SpinDrift`, `BallRolls` |
| `vertical` | `Vertical { Rising, Falling, Settled }` | rising, falling, or neither (on the ground, or the `Moonfall`) | `Gravity`, `RiseEnds`, `FallingSpeedLimit`, `SpaceJump`, `StartingToFall`, `Landing` |
| `vertical_velocity` | signed `P.S`, down positive | how fast she rises or falls | `Gravity`, `RiseEnds`, `SpaceJump`, `BallBounce` |
| `bounces` | count 0..=2 | how many times the ball has bounced since it last rested | `BallBounce` |
| `pose` | `Pose { body, action, facing, aim }` (§2) | the sprite | every behaviour; output |

`dash_kept` and `drive` are not measures, so they are argued: from the same pad, a halted Samus
loses her dash on her next step while a running one keeps it into a mockball; a running jump keeps
moving with the pad let go while a standing jump stops dead; a turning Samus visibly moves backwards
while she faces forwards. Each value is an outcome the player can tell apart.

### 1.1 What she reads from the world and the player (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| `pad` | held: run, jump, shoot, left, right, up, down, aim; newly pressed: jump | most behaviours |
| `equipment` | `hi_jump`, `space_jump`: `bool` | `JumpSpeeds`, `SpaceJump` |
| `world` | the room's blocks and solid enemies, answered as §8 says | §8 |
| `spin_turned_far` | `bool`: the spin's sprite has turned far enough to kick off a wall | `WallJumpKick` |
| `was_spinning` | `bool`: her previous two poses included a spin or a wall jump | `WallJumpArming`, `SpinReverse` |
| `elevator_moving` | `bool` | `RidingTheElevator` |

---

## 2. Her pose as the player sees it

| Field | Values |
| --- | --- |
| `body` | `Upright`, `Crouched`, `Ball`, `SpringBall` |
| `action` | `Still`, `Running`, `Moonwalking`, `Turning`, `Changing` (crouching, standing up, morphing or unmorphing), `RanIntoWall`, `Rolling`, `TakingOff`, `Jumping`, `Spinning`, `Falling`, `Rising` (the spring ball's hop), `WallJumping`, `TurningInAir` |
| `facing` | `Left`, `Right`, `Screen` (facing the player; counts as `Right` wherever a side is needed) |
| `aim` | the arm's direction; it changes only the sprite, except for `ArmPumpStep` |

The ball `Still` is the ball at rest. A pose chosen in a frame takes effect in her motion the next
frame (`MovesAsShePosed`).

---

## 3. On the ground

### ActivitySpeeds
- Parameter. How fast each activity speeds up and its top speed; every activity slides to a stop
  at 0.8000 a frame.

| Activity | Speeds up by | Top speed |
| --- | --- | --- |
| Running | 0.3000 | 2.C000 |
| the ball rolling | 0.C000 | 3.4000 |
| Moonwalking | 0.C000 | 0.8000 |
| Jumping, Changing, the spring ball's hop | 0.C000 | 1.4000 |
| Spinning, WallJumping | 0.C000 | 1.6000 |
| Falling, the ball falling | 0.C000 | 1.0000 |
| Still, Turning, RanIntoWall, TurningInAir | 0.C000 | 0.0000 |

- checked: oracle (the `SpeedingUp` and air vectors).

### SpeedingUp
- Mechanic. While driven she gains the activity's step each frame, up to its top speed
  (`WordwiseTopSpeed`), and moves by run speed plus dash along her facing. Applies running,
  moonwalking, the rolling ball, turning, and in the air as §6 says. checked: oracle.

### Braking
- Mechanic. Coasting on the ground she loses 0.8000 a frame instead; when that would take her
  below zero she stops (run speed 0) and is driven again. checked: oracle (`TurnSkid` vectors).

### StandingStill
- Mechanic. Standing, crouching, changing pose and after running into a wall, the pad doesn't move
  her: she moves by her dash only. At the frame's end standing and running into a wall halt her
  (run speed 0, dash 0 and lost, driven); crouching does the same but leaves `dash_kept` as it was;
  a changing pose keeps both speeds. Facing the screen she doesn't move at all and keeps her speeds.
  checked: oracle.

### Dash
- Mechanic. Running with run held she gains 0.1000 dash a frame up to 2.0000 (`DashCap`), and the
  dash is kept. In any other moving activity a dash that isn't kept is gone (0). checked: oracle
  (running vectors with run held; samus-mockball for the carry).

### DashCap
- Quirk (invisible in vanilla). The cap tests the dash from before the step: it grows if its pixel
  part is `below` 2 or its subpixel part is `below` 0 (§10), else it becomes 2.0000. So 2.F000 grows
  to 3.0000 and then snaps to 2.0000. checked: original only (C-dashcap-a, C-dashcap-b).

### LosingTheDash
- Mechanic. A halt (`StandingStill`, `WallStop`, a turn's end, `AirDrift`, a pose change's
  *slide* or *drop dash*) loses the dash: 0 and not kept. A pose change's *halt* only marks it not
  kept, so it goes the next frame she moves. checked: oracle.

### TurnSkid
- Mechanic. Turning round, on the ground or in the air, she folds her dash into her run speed (run
  speed += dash, dash 0) and coasts behind her new facing: she skids backwards, braking, until the
  speed is spent, then builds the new way (Turning's top speed is 0, so only `TurnCreep`). When the
  turn ends she loses her dash. checked: oracle.

### TurnCreep
- Quirk. Turning from a standstill slides her 0.C000 every other frame the new way: the speed goes
  0, 0.C000, 0, because Turning's top speed is 0 and `WordwiseTopSpeed` keeps 0.C000 over 0 once.
  checked: oracle (samus-input-poses turns).

### TurnIgnoresFall
- Quirk. Turning on the ground over a drop, she sinks by her total speed plus 1 px a frame instead
  of falling. checked: original only.

### ArmPumpStep
- Quirk. Changing aim while running moves her 1 px forward, without a speed change; a block or a
  solid enemy in the way puts her in `RanIntoWall` instead, as does a `WallStop` that frame. Not
  when the new pose came from an animation ending. checked: original only (open question 3).

### StandingShotRestartsAnimation
- Mechanic (output). Standing, unaimed and not landing, holding shoot restarts the standing
  animation. checked: original only.

### RidingTheElevator
- Mechanic. Facing the screen on a moving elevator, she is carried 1 px a frame, tested upward and
  ignoring solid enemies; nothing else of hers changes. checked: original only.

---

## 4. The morph ball on the ground

### BallRolls
- Mechanic. The rolling ball speeds up and brakes like running, with its own speeds
  (`ActivitySpeeds`), and keeps a kept dash (the mockball). The ball at rest and driven moves by
  its dash only and halts at the frame's end, unless it is in the air; braking or skidding, it
  rolls. checked: oracle (samus-mockball, samus-bomb-jump-vertical).

---

## 5. Leaving the ground

### JumpSpeeds
- Parameter. Jump 4.E000 upward (hi-jump 6.0000); wall jump 4.A000 (hi-jump 5.8000); ball bounce
  1.0000; gravity 0.1C00; the falling limit at pixel part 5; the crouch lift 10 px; the wall reach
  8 px. checked: oracle.

### TakingOff
- Mechanic. The first poses of a normal jump don't steer: she moves by her dash only, is driven,
  and `Dash` still applies; she doesn't move vertically. On the first of them she starts rising at
  the jump speed; from a crouch without aim she is also lifted 10 px. checked: oracle (y open:
  question 6).

### JumpStarts
- Mechanic. Starting a spin jump, or the spring ball's hop from the ground, she rises at the jump
  speed. Starting a normal jump or a fall she coasts along her facing if she has a dash, else she
  is driven, so a running jump keeps its drift. checked: oracle.

### FollowingTheGround
- Mechanic. On the ground she is pulled down after her horizontal move, by her total speed plus
  1 px (1 px on a slope), and the floor stops her flush (§8). Walking off a ledge the pull finds no
  floor and she starts to fall (`StartingToFall`). checked: oracle (every ground vector).

### StartingToFall
- Mechanic. When the ground goes from under her and she isn't rising, she falls from rest:
  vertical speed 0, `Falling`, bounces 0. Turning or changing pose, she doesn't (`TurnIgnoresFall`).
  checked: original only.

### MovesAsShePosed
- Timing. On the frame she changes pose she still moves as her old pose did: activity, speeds, and
  what a landing or a fall does, come from the pose at the frame's start. checked: oracle.

---

## 6. In the air

### Gravity
- Mechanic. Each frame she moves vertically by her vertical velocity, then it grows by 0.1C00
  downward, except at the falling limit. checked: oracle (every air vector).

### FallingSpeedLimit
- Quirk. Falling, her speed stops growing only when its pixel part is exactly 5, so it settles at
  5.1400 (4.F800 + 0.1C00); a speed set above that keeps its value. checked: original only
  (C-fallcap-a, C-fallcap-b).

### RiseEnds
- Mechanic. Rising, she starts to fall (vertical speed 0, `Falling`) once the upward speed is
  spent, and in a normal or spin jump or the spring ball's hop as soon as jump is let go. The frame
  it ends she doesn't move vertically. checked: oracle.

### AirDrift
- Mechanic. In a normal jump, falling and in the ball in the air, she builds towards the
  activity's top speed. Driven, with neither left nor right held, she stops drifting at once (run
  speed 0, no horizontal move); the falling ball instead halts (run speed 0, dash 0 and lost) but still moves by its dash that
  frame. checked: oracle.

### CarriedDrift
- Quirk. In the air, coasting doesn't brake: she still gains the activity's step, so a running jump
  keeps or gains its drift with the pad let go. checked: oracle.

### SpinDrift
- Mechanic. A spin jump drifts only while she holds the side she faces, or is skidding, or reached
  her top speed this frame: then she coasts (a skid stays a skid) and moves. Otherwise she stops
  drifting: run speed 0, driven, no move. checked: oracle.

### SpinReverse
- Mechanic. Reversing a spin (a spin facing the other way right after a spin or wall jump), or the
  ball turning round, folds the dash into her run speed, loses the dash and skids. checked: oracle
  (samus-screw-attack).

### AirTurn
- Mechanic. Turning in the air moves like `TurnSkid`, under gravity while airborne. checked: oracle.

### SpaceJump
- Mechanic. With space jump, pressing jump while falling in a spin jumps again, when her fall speed
  read as 8.8 (its pixel part's low byte and subpixel's high byte) is from 2.80 up to, not
  including, 5.00. It happens before the frame's horizontal move. checked: original only.

### Moonfall
- Quirk. A spin jump reached when a turn's animation ends, not by pressing jump, starts with no
  rise: she drops under gravity with no falling limit and jump does nothing, until she lands.
  checked: oracle (chain-0014-96BA), constructed (C-moonfall-a, C-moonfall-b).

---

## 7. Wall jumps, landings and pose changes

### WallJumpArming
- Mechanic. Spinning right after a spin or wall jump, holding away from a wall within 8 px (left
  held: a wall on her right; otherwise right held: on her left) readies the wall jump, which the
  sprite shows. checked: oracle (samus-walljump-16, -22, -28).

### WallJumpKick
- Mechanic. Once the spin has turned far enough, pressing jump with the wall closer than 8 px kicks
  her off: no vertical move that frame; she starts the wall jump rising at the wall-jump speed, run
  speed 0, driven. Through the wall jump she always drifts, coasting along her facing.
  checked: oracle (samus-walljump-*, samus-grapple-walljump).

### WallProbeSubpixel
- Quirk. Looking for a wall writes her x subpixel as a stop against it would (FFFF looking right, 0
  looking left), without moving her. checked: oracle (item-morph-ball's leftward spin jumps).

### Landing
- Mechanic. Falling onto a floor upright she lands: vertical velocity 0, `Settled`, bounces 0, run
  speed 0, driven; the dash keeps its value and `dash_kept` (standing then loses it, §3).
  Landing during a changing pose stops only her vertical motion; both speeds and the drive are
  kept. checked: oracle (chain-0097-ACF0 for the changing pose).

### BallBounce
- Mechanic. The ball bounces twice before it rests: landing at 3 px or more with no bounce yet, it
  rises at the bounce speed (bounces 1); after bounce 1 it rises at the bounce speed less 1 px
  (bounces 2); after bounce 2, or slower than 3 px, it lands as `Landing`. checked: original only
  (C-bounce). The spring ball landing with jump held hops again at the jump speed instead, bounces 0.

### HitCeiling
- Mechanic. Rising into a ceiling ends the rise: vertical speed 0, `Falling`. checked: original
  only.

### PoseChangeEffects
- Mechanic. Some pose changes change her speeds as they start; which change carries which effect
  belongs to the pose chart (§11). The effects:
  - *slide*: with run speed 0, a *halt*; else fold the dash into run speed, coast along her
    facing, lose the dash, and fall from rest if the new pose is a fall and the old one wasn't;
  - *halt*: driven; the dash is marked not kept;
  - *drop run speed*: run speed 0, driven, and the fall rule above;
  - *drop dash*: dash 0 and lost, and the fall rule above.
- A pose an animation hands over to (a turn's end into a spin) takes none of these and no jump
  start, and clears the bounces. A pose swapped for another (every turn, and the space-jump and
  screw-attack spins) skips the effect of the pose it replaced. checked: oracle
  (samus-walljump-16 frame 58 keeps its speed through the screw-attack swap); open question 2.

---

## 8. The world she moves through

There is no physics here: the world is data she meets. The room is a grid of 16 px blocks, each of
a kind: air, solid, a slope (with its shape), or a kind out of scope (§11). Solid enemies are boxes
at positions. Her body is a box of half-width `radius.x` and half-height `radius.y` around her
centre. Each move asks what is at the cells her body's leading edge reaches, and each kind answers
as below. These rules are Samus's; other actors get their own when their domains are written.

### FloorsStopFlush
- Mechanic. A move into a solid block stops her with her leading edge against its face: moving
  right or down she ends with that axis's subpixel FFFF, moving left or up with 0. Her leading edge
  at the target is `x + d + rx − 1` right, `x + d − rx` left, `y + d + ry − 1` down, `y + d − ry` up,
  from the pixel parts. checked: oracle (every ground vector's floor).

### WallStop
- Mechanic. A horizontal move stopped by a wall, on the ground or in the air, halts her: run speed
  0, dash 0 and lost, driven. The ground pull that frame still uses the speed from before the stop.
  checked: original only (C-wall).

### CeilingsStop
- Mechanic. An upward move into a block stops flush and ends the rise (`HitCeiling`). checked:
  original only.

### FastMovesTunnel
- Quirk. A move tests only the row or column of her leading edge at the target, so a move of 16 px
  or more can pass a floor. checked: original only.

### MoveLimit
- Mechanic. A horizontal move is limited to 15 px either way, its subpixel kept. Unreachable in
  vanilla motion. checked: original only (C-limit).

### ZeroMovesStillTouch
- Mechanic. A horizontal move of 0 still meets solid enemies on her heading. checked: original
  only.

### SolidEnemyTouch
- Quirk. A solid enemy she already touches on the tested side stops a move with no distance and
  sets her **y** subpixel to 0, on either axis; one ahead stops her flush against it. Looking for a
  wall to kick off (§7) meets solid enemies first. checked: original only.

### SlopesCarry
- Mechanic. On a slope the ground pull is 1 px. A horizontal move that meets solid enemies ends by
  seating her on the slope, or under the sloped ceiling, she reached: a whole-pixel shift of y
  before the vertical move. checked: oracle (samus-walljump-16/22/28 frame 44); no vector yet.

---

## 9. Order that matters

Visible at the frame's end:
1. The pose at the frame's start decides how she moves (`MovesAsShePosed`).
2. The space jump, then the horizontal move with this frame's speeds, then the wall-jump test,
   then the vertical move.
3. The vertical move uses the velocity from before this frame's gravity.
4. The ground pull uses this frame's total speed, from before a wall stop.
5. A halt at the frame's end (`StandingStill`, `BallRolls`, a turn's end) follows the moves.
6. A new pose's effects (§7) follow the moves; the arm-pump step follows them.

## 10. Numbers

- `below(a, b)` on 16-bit parts: bit 15 of (a − b) mod 2^16 is set. 1000 is not below C000; C000 is
  below 4000 and below 0.
- Adding `P.S` carries the subpixel into the pixel part, mod 2^16 on the pixel part.

### WordwiseTopSpeed
- Quirk, **Toggle**. The top speed is tested part by part. `Wordwise` (vanilla): she keeps her
  speed if its pixel part is `below` the top's; else if the pixel parts differ she takes the top
  speed; else she keeps it if the subpixels are equal or hers is `below`, else takes the top. So
  2.1000 snaps up to 2.C000, 3.C000 is kept over 3.4000, 0.C000 is kept over 0, and 2.2000 snaps
  down to 1.4000. `Exact`: the smaller of the two. checked: oracle (`Wordwise`, the running
  vectors); constructed (C-cap, C-cap-exact).

## 11. Out of scope, and what remains

- **The pose chart:** which pose follows which pad and world event, and which change carries which
  effect (`PoseChangeEffects`). It is ROM data; it needs its own observer lane, written as records
  keyed by what the player does.
- **Slopes:** the sideways speed change on a slope, and square-slope corners.
- **Liquids and the speed booster:** other speeds and gravity in water and lava; the booster's dash
  cap of 7 and its higher jump.
- **Pushes:** room pushes, quicksand, knockback, bomb jumps, grapple, shinespark.

## 12. Open questions

1. `drive` and `dash_kept` are argued as perceivable (§1); a reviewer may prefer a derivation.
2. `PoseChangeEffects` needs vectors at pose changes (take-offs, turns' ends, landings other than
   the changing one); the extractor takes only steady frames and landings so far.
3. `ArmPumpStep`, `SlopesCarry`, `SolidEnemyTouch` and `HitCeiling` have no vectors yet.
4. The water rows, terrain-quicksand and samus-ceres-push gave no vectors: their liquid and push rules are out of
   scope.
5. Landing on a solid enemy's top (item-morph-ball frame 797) clears the y subpixel before she
   touches it: a collision question.
6. The crouch take-off's lift appears a frame apart from pose fit (item-morph-ball 1311).
7. A skid at exactly 0.8000 ends driven in the records (samus-input-poses:120 and the vector with
   `drive = "Skidding"` and run speed 0.8000), though braking by 0.8000 lands on 0, not below. Either
   the turn's slide is larger than 0.8000 or something else ends the skid; not yet traced.
