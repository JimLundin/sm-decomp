# Samus on the ground: behaviour spec

Observer's spec for MODEL §4. It covers standing, running, turning on the ground,
crouching, the transition poses, the morph ball on the ground, the shared speed rules, the
step down, and one ground move against blocks. The air is out of scope: jumps, falling, the
wall jump. So is pose selection, except where it decides which motion row runs.

Notation. A position or speed is `P.S`: a 16-bit pixel word and a 16-bit subpixel word, so
`2.C000` is 2 + 0xC000/65536 = 2.75 px. Hex throughout. "Frame N" means the oracle record N.
A record holds the state at the frame's end, and its `pad` is the pad read at vblank N, which
the game's frame N+1 consumes.

---

## 1. Model in one page

**Quantities Samus has on the ground**

| Quantity | Units | Notes |
| --- | --- | --- |
| `position` (x, y) | pixel word + subpixel word each | x, y are her centre. Moves add 32-bit with carry, and wrap modulo 2^32. |
| `radius` (x, y) | pixels | Comes from the pose. In slice 0: x = 5; y = 0x15 standing/running/turning, 0x10 crouching, 7 morph ball, 0x18 the elevator/landing pose, 0x0C spin, 0x13 jumping. |
| `facing` | Left / Right | Comes from the pose. |
| `base_speed` | P.S, never negative | Speed from the movement type's speed-table row. |
| `extra_run` | P.S, never negative | Speed from holding run. |
| `momentum` | bool | "Running momentum": while it is set, extra_run survives a frame without running. |
| `mode` | `Accelerating` / `TurningAround` / `Decelerating` | ROM values 0 / 1 / 2. |
| `total_speed` | P.S | base + extra_run, stored. It is RAM-observable and goes stale on frames that do no horizontal step. |
| `extra_x_displacement`, `extra_y_displacement` | signed P.S | Pushes from other systems. Zero in slice 0's ground frames. |
| `speed_divisor` | 0..4+ | It shifts total_speed right (see `SpeedDivisor`). It was 0 in every slice-0 record. |
| `slope_adjusted` | bool | Set when this frame's horizontal step re-seated her on a slope. |
| outcome fields | | Solid-collision kind, X-speed-killed direction, vertical result, and distances moved left/right/up/down. All are RAM-observable (§4). |

**How a frame's ground motion works**

1. **Pick the row** from the movement type *as it stands at the frame's start* (the pose set by the
   previous frame). That type indexes the motion table (§3) and the speed table
   (`data::movement::SpeedTable`, entry = movement type).
2. **Horizontal speed.** The row says one of three things:
   - *Hold*: move with base speed taken as 0, so only extra_run moves her.
   - *Speed rules*: update extra_run (`ExtraRunSpeed`), then base_speed (`BaseSpeedAccelerate` or
     `BaseSpeedDecelerate` by `mode`).
   - *None*: no horizontal step at all.
3. **Horizontal step.** total = (base + extra_run) >> divisor. The direction is the facing, reversed
   when mode is TurningAround. The displacement is ±total + extra_x_displacement, with the pixel word
   clamped to ±15 (`DisplacementClamp`). One horizontal sweep (§4) moves her. If it hit something,
   speeds are killed (`WallStop`). Then she is re-seated on a non-square slope (`SlopeSeat`).
4. **Vertical step ("keep to ground").** She moves *down* by total + 1.0 px, or by 1.0 px if
   `slope_adjusted` (`StepDown`). One vertical sweep (§4) does it. If the sweep hits floor, she lands
   flush and the result is `Landed`. If not, she has moved down that far and the result is `Falling`;
   the pose layer turns that into a fall.
5. **Row epilogue**: the row's `After` (zero the speeds, cancel speed boosting, and so on).
6. **Later in the frame** the pose update runs. If it chooses a running pose, Samus takes one extra
   1-px horizontal step forward (`ArmPumpStep`). This is outside the row but is ground motion.

---

## 2. Behaviours

Each entry: id, kind, summary; the rule; parameters; switch; source.

### MovementRowByStartOfFrameType
- kind: Timing. summary: on the frame Samus changes pose, she still moves the way her old pose moved.
- Rule: the motion row (and the speed-table entry) comes from the movement type at the frame's
  start. The pose update later in the frame does not re-run motion. Example: frame 687 ends
  *Standing*, but it moved by the *turning* row (0.C000 creep). Frame 1310 ends *Crouching*, but the
  transition row ran, so base_speed 3.4000 was left untouched until frame 1311.
- Params: none. switch: Fixed.
- Source: `$90:A337` (movement handler dispatch on movement type).

### SpeedTableRow
- kind: Mechanic. summary: each way of moving has its own acceleration, top speed and braking.
- Rule: entry = table[movement type] with (accel, max, decel), each P.S. The normal table is used
  unless Samus is submerged in water (with water physics on) or in lava/acid, and has no gravity
  suit. Then the water or lava table is used.
- Params (normal table, `data::movement::SpeedTable`, `$90:9F55`), as accel / max / decel:

  | Type | accel | max | decel |
  | --- | --- | --- | --- |
  | 0 Standing | 0.C000 | 0.0000 | 0.8000 |
  | 1 Running | 0.3000 | 2.C000 | 0.8000 |
  | 4 Morph ball on ground | 0.C000 | 3.4000 | 0.8000 |
  | 5 Crouching | 0.C000 | 0.0000 | 0.8000 |
  | E Turning on ground | 0.C000 | 0.0000 | 0.8000 |
  | F Crouch/stand/morph/unmorph transition | 0.C000 | 1.4000 | 0.8000 |
  | 10 Moonwalking | 0.C000 | 0.8000 | 0.8000 |
  | 11 Spring ball on ground | 0.C000 | 3.4000 | 0.8000 |
  | 15 Ran into a wall | 0.C000 | 0.0000 | 0.8000 |

  The water table (`$90:A08D`): running and ball accel 0.0400, max 2.C000, decel 0.0800.
- switch: Parameter (per-entry values).
- Source: `$90:9BD1` (determine speed table entry).

### BaseSpeedAccelerate
- kind: Mechanic. summary: Samus speeds up by the row's acceleration each frame, up to the row's top speed.
- Rule (when mode = Accelerating): base += accel, then the cap rule (`SpeedCap`, below) is applied.
- Params: accel and max from `SpeedTableRow`. switch: Fixed (the values are Parameters).
- Source: `$90:9A7E` (base speed, deceleration allowed).

### SpeedCap  (the word-by-word cap: Quirk)
- kind: Quirk. summary: speed briefly overshoots, or snaps *up* to, the top speed.
- Rule, vanilla `Wordwise`: after adding, compare the pixel words, then the subpixel words, each
  as "a is below b iff (a − b) mod 2^16 has bit 15 set":
  - pixel below max.pixel: keep.
  - pixel above max.pixel: set base = max.
  - pixels equal: if the subpixel equals max.sub or is below it (by the bit-15 test), keep;
    otherwise set base = max.

  Consequences seen in the oracle:
  - Running: 1.E000 + 0.3000 = **2.1000**. Sub 0x1000 vs 0xC000 gives 0x5000, bit 15 clear,
    so it counts as *past* the cap and becomes **2.C000**. Speed jumps up, a free 0.B000 (frames
    699 and 752).
  - Morph ball: 3.0000 + 0.C000 = **3.C000**. Sub 0xC000 vs 0x4000 gives 0x8000, bit 15 set, so it
    counts as *below* the cap and is kept for one frame above the max of 3.4000. The next frame,
    4.8000 is over the pixel and caps to 3.4000 (frames 1271 to 1272).
  - Turning (max 0.0000): 0 + 0.C000 = 0.C000. Sub 0xC000 vs 0 gives 0xC000, bit 15 set, so it is
    kept. The next frame, 1.8000 caps to 0.0000. So a turning Samus creeps 0.C000 every other frame
    (`TurnCreep`, frames 683 to 687).
- `Exact`: compare the full 32-bit values; base = min(base, max).
- Params: none beyond the table. **switch: Toggle (`SpeedCap::Wordwise` vanilla | `SpeedCap::Exact`)**.
- Source: `$90:9A99` (cap compare in "Calculate Samus X base speed").

### TurnCreep
- kind: Quirk. summary: during the on-ground turn, Samus slides 0.C000 px forward every other frame.
- Rule: no rule of its own. It is `SpeedCap::Wordwise` applied to the turning row (accel 0.C000,
  max 0). Base alternates 0 → 0.C000 → 0 (via the 1.8000 → max snap). She moves in the
  facing direction, which is the *new* facing once mode has returned to Accelerating. Under
  `Exact` the turn has no creep.
- Params: none. switch: follows `SpeedCap`.
- Source: `$90:A67C` (turning around on ground).

### BaseSpeedDecelerate
- kind: Mechanic. summary: when Samus brakes or reverses, her speed falls by the row's deceleration.
- Rule (when mode = Decelerating or TurningAround): step = decel, or decel × multiplier / 256 when
  the deceleration multiplier is non-zero. The multiplier product uses only the low byte of
  each decel word: sub = (decel.sub high byte × m) as a 16-bit result, and pixel = (decel.pixel low
  byte × m) high byte. base −= step. If the result's pixel word has bit 15 set (it went negative),
  base = 0 and mode = Accelerating. No cap is applied while decelerating.
- Params: decel from the table; `decel_multiplier` (byte, normally 0, set elsewhere).
  switch: Parameter.
- Source: `$90:9ABA` (decelerating branch).

### ExtraRunSpeed
- kind: Mechanic. summary: holding run while running adds speed up to 2 px/frame (7 with speed booster).
- Rule, on every row that applies the speed rules:
  - *Not eligible* if Samus is submerged without the gravity suit: in water with water physics on,
    or in lava/acid. "Submerged" means the liquid surface is above her bottom edge.
  - *Running* = eligible, movement type is Running, and run is held. Then:
    - set momentum (and, with the speed booster, start the boost timer; out of scope);
    - if extra_run is at or past the cap, by the word-by-word test (pixel not below, and sub not
      below; "below" is the bit-15 test as in `SpeedCap`), set extra_run = cap;
    - otherwise extra_run += 0.1000.
  - *Not running*: if momentum is clear, extra_run = 0; otherwise extra_run is kept.
- Quirk inside: the cap test is word-by-word, so a value like 2.8000 (sub ≥ 0x8000) counts as
  below 2.0000 and keeps growing until it reaches 3.0000, then snaps to 2.0000. From 0 in steps of
  0x1000 she lands exactly on 2.0000 or 7.0000, so this never shows in vanilla play. Constructed
  vector V11.
- Quirk inside: the water and lava columns of the table (accel 0.0400, caps 4/1/0) are never used,
  because submerged means "not running" before the column is chosen.
- Params (`data::movement::ExtraRun`, `$90:9F01`): accel 0.1000; cap 2.0000 without the speed
  booster, 7.0000 with it. switch: Parameter (accel, caps); the cap-compare style follows `SpeedCap`.
- Source: `$90:973E` (handle extra run speed).

### Momentum
- kind: Mechanic. summary: run speed carries through a jump and is lost when she stops.
- Rule: momentum is set by `ExtraRunSpeed` while running. "Cancel speed boosting" clears it, together
  with the boost counter and the boost palette. Cancel runs on the standing row, the ball-at-rest
  row, the turn row, `WallStop`, and the pose-change commands Decelerate and Stop. The crouching row
  zeroes the speeds but does *not* cancel.
- switch: Fixed. Source: `$91:DE53` (cancel speed boosting).

### SpeedDivisor
- kind: Mechanic. summary: (an effect slows all of Samus's horizontal motion by a power of two)
- Rule: n = min(divisor, 4). total = (base + extra_run) >> n, computed as follows. The two subpixel
  words are added with carry into the pixels. The pixel sum's low byte is shifted, and the bits
  shifted out of it go into the subpixel's top bits. The subpixel is shifted too, and its low bits
  are lost. This is exact only for pixel sums below 256. With n = 0 it is the plain sum.
- Params: divisor (0 in every slice-0 record). switch: Fixed.
- Source: `$90:E4E6` (calculate Samus X speed).

### DisplacementClamp
- kind: Mechanic. summary: Samus never moves more than 15 px sideways in a frame.
- Rule: d = ±total + extra_x_displacement (minus when moving left). The pixel word is clamped to
  [−15, +15]; when it is clamped, the subpixel word is *kept* (so +15.F000 is possible).
- Params: 15 (constant). switch: Parameter.
- Source: `$90:E4AD` (displacement for moving right).

### HorizontalDirection
- kind: Mechanic. summary: during a turn, Samus keeps sliding the way she was going.
- Rule: if mode = TurningAround, the step goes opposite to the facing; otherwise it goes along the
  facing.
- switch: Fixed. Source: `$90:8EA9` (move Samus horizontally).

### WallStop
- kind: Mechanic. summary: running into a wall stops her dead, flush against it.
- Rule: after the horizontal sweep, if it collided (with a block or a solid enemy):
  - X-speed-killed = 4 if the collision direction was left, else 8;
  - cancel speed boosting; extra_run = 0, base = 0, mode = Accelerating.

  With no collision: X-speed-killed = 0 and the vertical result is cleared. Quirk: if the step's
  sign disagrees with the collision direction (only possible through extra_x_displacement), the
  collision flag is cleared and nothing is killed. If the row was running, the pose update then
  turns X-speed-killed into the "ran into a wall" pose.
- switch: Fixed. Source: `$90:E5CE` (kill Samus X speed if collision detected).

### StepDown  (keep to ground)
- kind: Mechanic. summary: walking off a ledge or down a slope, Samus sticks to the ground beneath her.
- Rule: the ground rows move her down by:
  - extra_y_displacement + 1.0, when extra_y_displacement ≠ 0 and its pixel word ≥ 0;
  - up by |extra_y_displacement|, when its pixel word < 0;
  - otherwise 1.0, if `slope_adjusted`;
  - otherwise total_speed + 1.0 (1 is added to the pixel word, and the subpixel is kept).

  The move is one vertical sweep. A hit means `Landed`, flush to the surface. A miss means she moves
  down the whole amount and the result is `Falling`.
- Params: `step_down_extra` = 1.0 px (constant). switch: Parameter.
- Source: `$90:923F` (Samus Y movement, no speed calculations).

### FloorFlushSubpixel
- kind: Quirk. summary: (invisible) after landing on a flat solid floor her vertical subpixel is FFFF.
- Rule: a downward sweep that stops on a solid block sets the y subpixel to FFFF, and the pixel to
  sit flush. So on flat ground y.sub = FFFF every frame. A non-square slope floor leaves the
  subpixel unchanged. An upward stop sets the sub to 0. Horizontal stops are the same: moving right,
  x.sub = FFFF; moving left, x.sub = 0 (frame 1333: X 053B.FFFF).
- switch: Fixed. Source: `$94:8F82` (vertical solid block reaction).

### SlopeSeat
- kind: Mechanic. summary: walking across a slope keeps her feet on its surface.
- Rule: after every horizontal step, when horizontal slope detection is on (vanilla: on, value 3):
  - Feet: take the block under her centre x at her bottom edge (y + ry − 1). If it is a non-square
    slope (shape ≥ 5) and not upside down: let A = surface(column) − (bottom mod 16) − 1, where
    the column is x mod 16, mirrored if the slope is x-flipped. If A < 0: y += A (raise her), and
    set `slope_adjusted`.
  - Head: take the block under her centre x at her top edge (y − ry). If it is a non-square slope
    and upside down, the mirror rule pushes her down when she is inside it, and sets
    `slope_adjusted`.
- Params: `data::level::slope::Slopes` (surfaces `$94:8B2B`). switch: Fixed.
- Source: `$94:87F4` (align Samus Y position with non-square slope).

### SlopeSlowdown
- kind: Mechanic. summary: running up or down a steep slope covers less ground.
- Rule: in the horizontal sweep, each non-square slope block (not upside down) met in the leading
  column, while her vertical speed is 0, scales the step: d = d × factor / 256. The 16.16 step is
  reduced to 8.8 first (its subpixel low byte is lost). The result is negated back for a leftward
  step. The block never stops her, and the scan continues, so two slope rows in the column scale
  twice. The factors' first column (named "speed modifiers") has no effect.
- Params: `data::level::slope` FACTORS (`$94:8586`): shape 12/14/15 → 0xC0; 16/17 → 0xD8;
  1B/1C → 0x80; E/F → 0xB0; others 0x100. switch: Parameter.
- Source: `$94:84D6` (horizontal non-square slope reaction).

### VerticalScanAlternates
- kind: Quirk. summary: which block she lands on, when two differ under her, flips every frame.
- Rule: the vertical sweep scans her columns left to right on even frame counters and right to left
  on odd ones. The first block that collides decides the distance. This is observable only when
  the blocks under her give different answers (solid next to slope, door, spike).
- switch: Toggle (vanilla alternating | always left-to-right). Source: `$94:9763`.

### ArmPumpStep
- kind: Quirk. summary: changing aim while running moves Samus an extra pixel forward ("arm pump").
- Rule: in the pose update, when the transition table yields a new prospective pose whose movement
  type is Running (standing → running counts too):
  - first test 1.0 px forward against solid enemies; a hit sets the pose to "ran into a wall";
  - otherwise *move* 1.0 px forward through the horizontal block sweep (§4), with no slope seat and
    no step down; a block hit sets the pose to "ran into a wall" (with the matching aim).

  Before that: if the frame's `WallStop` fired and the type is running, the pose becomes "ran into
  a wall" with no step. The step does not touch speeds or total_speed. Because there is no step
  down, it can lift her off a downward slope.
- Seen: every running frame whose pose changes (frames 688 to 705 alternate aim; each moves
  total + 1.0); frames with an unchanged pose move total only (1230, 1237).
- Params: 1.0 px. switch: Toggle (vanilla on).
- Source: `$91:EADE` (check if prospective pose runs into a wall).

### ElevatorStanding
- kind: Mechanic. summary: on a moving elevator, Samus facing the screen rides it down.
- Rule: the standing row with the facing-forward pose (0 or 9B) does no horizontal step. If an
  elevator is active, she moves down 1.0 with no solid-enemy check. Quirk: the move is flagged as
  *upward* for slope reactions. The vertical result is then cleared.
- switch: Fixed. Source: `$90:A383` (standing).

---

## 3. Motion table

`H`: horizontal speed. `ER`: extra run. `V`: vertical. `After`: the epilogue. "Speed rules" =
`ExtraRunSpeed` then `BaseSpeedAccelerate`/`Decelerate` by mode, then the step. "Hold" = step with
base taken as 0 (extra_run still moves her), base unchanged. "KeepToGround" = `StepDown`. "Air"
= the airborne vertical routine (out of scope) when the Y direction is not none.

| Row (movement type) | H | ER | V | After |
| --- | --- | --- | --- | --- |
| Standing (0), pose facing forward | None | untouched | elevator ride (`ElevatorStanding`) | clear vertical result |
| Standing (0), other poses | Hold | untouched by the step | KeepToGround | cancel boost; extra_run = 0; base = 0; mode = Accelerating |
| Running (1) | Speed rules | ExtraRunSpeed | KeepToGround | footstep sound on frames 2 and 7 of the run animation (out of scope) |
| Morph ball on ground (4), mode Accelerating and pose at rest (1D right / 41 left) | Hold | untouched | Air if airborne (and then **no** After), else KeepToGround | cancel boost; extra_run = 0; base = 0; mode = Accelerating |
| Morph ball on ground (4), otherwise (rolling, or decelerating) | Speed rules | ExtraRunSpeed (not running, so it is zeroed unless momentum) | Air if airborne, else KeepToGround | nothing |
| Crouching (5) | Hold | untouched | KeepToGround | extra_run = 0; base = 0; mode = Accelerating (no cancel) |
| Turning on ground (E) | Speed rules | ExtraRunSpeed (not running) | KeepToGround | cancel boost; extra_run = 0; clear vertical result. The turn ends when the pose update leaves the turn pose. |
| Transition (F), poses 35-38, 3B-3E | Hold | untouched | Air if airborne, else KeepToGround | if the vertical result is Landed (and the per-type landing kind is "no change"): Y speed = 0, Y direction = none, bounce = none; then clear vertical result |
| Transition (F), poses ≥ DB below F1 | None | untouched | None | as above |
| Moonwalking (10) | Speed rules | ExtraRunSpeed | KeepToGround | nothing |
| Spring ball on ground (11) | as morph ball, at-rest poses 79 right / 7A left | | | |
| Ran into a wall (15) | (not read; the row exists in `$90:A75F`) | | | |

Notes:
- `mode` values come from pose-change commands outside this domain. *Decelerate* folds extra_run
  into base (base += extra_run, extra_run = 0), sets Decelerating and cancels the boost; with
  base = 0 it acts as *Stop*. *Turn* folds extra_run into base and sets TurningAround. *Kill X
  speed* zeroes base and extra_run and sets Accelerating.
- Every "Hold" and "Speed rules" step writes total_speed. "None" rows leave it stale.

---

## 4. Collision: one sweep

A sweep moves Samus by a signed P.S distance `d` along one axis, against the room's blocks. Inputs:
position, radii, `d`, and the frame-counter parity (vertical). Output: `Contacts`.

**Solid enemies come first, and are outside this domain.** Before each block sweep, the solid-enemy
test runs with the same `d`. If an enemy blocks, the enemy code adjusts `d`, she moves with no block
test, and the collision flag = Enemy (FFFF).

**Horizontal sweep**
- Target x = x + d. Leading edge: x + d + rx − 1 when moving right (d.pixel ≥ 0); x + d − rx when
  moving left. One column: edge >> 4.
- Rows from her top row ((y − ry) >> 4) down to her bottom row ((y + ry − 1) >> 4), at the
  *current* y, in that order. The first block that collides ends the scan.
- Reactions by block type:
  - air, shootable air, unused air: pass.
  - solid, shootable, grapple:
    - right: d = max(0, (edge & ~F) − rx − x).0000 and x.sub := FFFF;
    - left: d = min(0, (edge | F) + 1 + rx − x).0000 and x.sub := 0.
  - non-square slope: pass, possibly scaling d (`SlopeSlowdown`).
  - square slope (shape < 5): the quadrant table (`data::level::slope` Square). Only the half of
    the block that the edge is in is tested, by row half. When it is solid, she stops as for solid
    blocks but aligned to 8 px. At 8+ px/frame she can therefore stop inside the far half.
  - door, spike, special, bombable, extensions: their own reactions (out of scope).
- Then position.x += d (with carry from the subpixel).
- Contacts: collided (Block), d actually moved (written as "distance moved left/right").

**Vertical sweep**
- Target y = y + d. Edge: y + d + ry − 1 (down) or y + d − ry (up). One row: edge >> 4.
- Columns from (x − rx) >> 4 to (x + rx − 1) >> 4: left to right on even frame counters, right to
  left on odd ones (`VerticalScanAlternates`). The first collision ends the scan.
- Reactions:
  - solid (and door blocks that lead nowhere special):
    - down: d = max(0, (edge & ~F) − ry − y).0000 and y.sub := FFFF;
    - up: d = min(0, (edge | F) + 1 + ry − y).0000 and y.sub := 0.
  - non-square slope: only if this column is the one under her centre x. Going down, the slope
    must not be upside down (going up, it must be). Let A = surface(column) − (edge mod 16) − 1,
    where the column is x mod 16, mirrored when x-flipped. If A ≤ 0: d = max(0, d.pixel + A).0000;
    the subpixel of y is *not* changed.
  - square slope: quadrants as above; aligned to 8 px; sets y.sub (FFFF down, 0 up) and
    `slope_adjusted`.
- Then position.y += d. Contacts: collided (Block); the vertical result: down → `Landed` or `Falling`
  (plus a per-movement-type landing kind: ground / ball / spring ball / no change); up → `HitCeiling`
  or none; "distance moved down/up".

**Slopes and the step down together.**
1. The horizontal sweep scales d on slopes.
2. `SlopeSeat` raises her onto an upward slope and marks `slope_adjusted`.
3. `StepDown` then moves her down only 1.0, which finds the surface she was just seated on.
4. Going down a slope, she is not seated (A ≥ 0), so the step down of total + 1.0 px pulls her
   onto the lower surface. That covers slopes up to 45°. On steeper slopes, or a drop deeper than
   total + 1.0, the step misses and the result is `Falling`.

Slice 0's oracle records never exercise a slope seat (`slope_adjusted` was never set at a frame's
end, and no frame shows one). The slope rules are from the original alone.

---

## 5. Test vectors

All from scenario `item-morph-ball` unless marked constructed. The before state is the end of frame
N−1, the after state is the end of frame N, and the input is record N−1's pad.

| # | Frame | Situation | Before | After |
| --- | --- | --- | --- | --- |
| V1 | 1230 | Running right, pose 09 → 09 (no arm pump); pad B+Right | X 047D.3FFF; base 0; ER 0; mom 0; mode Acc | base 0.3000; ER 0.1000; mom 1; total 0.4000; X 047D.7FFF |
| V2 | 1231 | Running right, aim change 09 → 11 (`ArmPumpStep`) | X 047D.7FFF; base 0.3000; ER 0.1000 | base 0.6000; ER 0.2000; total 0.8000; X 047E.FFFF (= +0.8000 + 1.0) |
| V3 | 752 | **Speed-cap quirk**, running left, pose 12 → 0A | X 04D6.7FFF; base 1.E000; ER 0.A000 | base **2.C000** (1.E000 + 0.3000 = 2.1000 snaps up); ER 0.B000; total 3.7000; X 04D2.0FFF (= −3.7000 − 1.0). Under `Exact`: base 2.1000, total 2.B000, X 04D2.CFFF |
| V4 | 753 | At the cap, running left, 0A → 12 | X 04D2.0FFF; base 2.C000; ER 0.B000 | base 2.C000 (2.F000 capped); ER 0.C000; total 3.8000; X 04CD.8FFF |
| V5 | 1271, 1272 | **Ball overshoot quirk**, rolling right, pose 1E | X 04B4.3FFF; base 3.0000 | 1271: base **3.C000** (kept above max 3.4000); X 04B7.FFFF. 1272: base 3.4000; X 04BB.3FFF |
| V6 | 682 to 687 | **TurnCreep**, turning left on the ground (mode TurningAround at 681, base 0) | X 0580.1000 | base per frame 0, .C000, 0, .C000, 0, .C000; X 0580.1000, 057F.5000, 057F.5000, 057E.9000, 057E.9000, 057D.D000 (687 ends Standing but ran the turn row) |
| V7 | 682 to 686 | **StepDown** to the floor after the radius shrinks 0x18 → 0x15; floor top at y 0x2C0 | Y 02A8.0000 (681, ry 0x18) | 682 Y 02A9.0000 (+1.0); 683 02AA.C000 (+1.C000 = total 0.C000 + 1); 684 02AB.C000 (+1.0); 685 **02AB.FFFF** (flush: bottom 0x2BF, sub FFFF); 686 02AB.FFFF |
| V8 | 1333 | **WallStop**, running right into a wall at x 0x540 | X 053B.BFFF; base 0.C000; ER 0.4000; mom 1 | target X 053C.FFFF, right edge 0x540 → d = 0.0000; X **053B.FFFF**; base 0; ER 0; mom 0; mode Acc; total 1.4000 (stale value of this frame); pose → D1 (ran into a wall) |
| V9 | 688 | Standing row and arm pump; standing → running left (prospective 12) | X 057D.D000; base 0.C000 | base 0; ER 0; total 0; X 057C.D000 (−1.0 from `ArmPumpStep` only) |
| V10 | 1266, 1267 | Ball at rest, then rolling | 1265: pose 1D; base 0.C000 (stale); X 04AC.BFFF | 1266: base 0; X 04AC.BFFF. 1267 (pose 1E): base 0.C000; X 04AD.7FFF |
| V11 | constructed | ExtraRun word-wise cap | ER 2.8000, running, no booster | ER 2.9000 (counts as below 2.0000). From ER 2.F000: 3.0000, then the next frame snaps to 2.0000. `Exact`: stays 2.0000 |
| V12 | constructed | DisplacementClamp | total 0x10.4000 right, extra disp 0 | step 0x0F.4000 (pixel clamped, sub kept) |
| V13 | 1310, 1311 | Transition row keeps base | 1309: base 3.4000, type F | 1310 (ends Crouching): base 3.4000. 1311 (crouch row ran): base 0 |

Vertical cross-check: on flat solid ground, y.sub stays FFFF (frames 685 to 705, Y 02AB.FFFF).
Frames 1229 to 1237 have y.sub 0000 and stand on a solid enemy, not a block, so use them for
horizontal checks only.

---

## 6. Order that matters

Everything else inside the frame is free (MODEL §1). These orders are visible at the frame's end:

1. **The row is chosen before the pose update** (`MovementRowByStartOfFrameType`).
2. **The horizontal step comes before the vertical step.** The step down uses this frame's
   total_speed and the `slope_adjusted` mark from this frame's horizontal step.
3. **`ExtraRunSpeed` comes before the base speed, and both come before the step.** total_speed is
   the post-update sum, and the extra-run update reads the movement type and the run button.
4. **Zeroing epilogues run after the step.** A standing or crouching row still moves by the old
   extra_run this frame (Hold), then zeroes it. The turn row moves with whatever extra_run survived,
   then zeroes it.
5. **`ArmPumpStep` comes after all motion**, and so after `WallStop`. Its 1-px move is not seated
   on slopes and not followed by a step down.
6. **The first block that collides wins**, in scan order: horizontal top to bottom; vertical by
   frame parity.
7. **The subpixel snap comes before the add.** The solid reaction sets the subpixel (FFFF or 0),
   then d (sub 0) is added, so the subpixel stays at the snapped value.
8. **total_speed is stored by every row that steps**, and is stale otherwise (V8 keeps 1.4000 after
   the wall). It is RAM-observable.

---

## 7. Open questions

1. **The speed-table pointer.** When she is not submerged, or has the gravity suit, the original
   doesn't *set* the table pointer back to normal. It keeps the last one, which other code
   (block-inside reactions, `$94:97D3` and friends) resets to normal. In slice 0 it was always the
   normal table. Is a stale water table reachable on land?
2. **Who writes `speed_divisor`?** No write was found in banks 90, 91 or 94. It was 0 in all
   records, so treat it as 0 for slice 0.
3. **The deceleration multiplier** (`$0B4C`) is set outside these banks; assume 0 on the ground.
4. **The "ran into a wall" row (15)** and the spring ball were not traced in detail.
5. **Horizontal slope detection flag.** The vanilla value is 3 (on) in every record. A wall-jump
   path sets it to 0 or 1 (`$91:F30E`), and it was not traced when it is restored.
6. **Morph-ball turning momentum** (`$91:FA0F`, "apply momentum if turning in morph ball") belongs
   to pose selection and was not read. The ball's on-ground deceleration (mode Decelerating while
   rolling) never occurs in the records.
7. **Door blocks as floor.** In the elevator room (frames 682 to 705) the floor row under her is a
   door-type block, and it behaved as solid (sub FFFF). The door reaction's full rule (`$94:93CE`)
   is in the door domain.
8. **Speed booster** (cap 7.0, boost counter, contact damage at counter 4) and **liquids** (water
   and lava tables) are not exercised in slice 0, so they were not checked against the oracle.
