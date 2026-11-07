# Samus moving through the world: behaviour spec

What the player sees Samus do when she runs, stops, turns, crouches, rolls, jumps, spins, falls,
wall-jumps and lands, and how the world's blocks, slopes, solid enemies, floors, walls and
ceilings stop her. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-motion.toml`](../../scenarios/vectors/samus-motion.toml).

## Header

**Scope (Jim, 2026-11-07).** One domain, because the player sees one thing: Samus moving through
the world. It covers her speeds and position, how her pose follows from input and the world where
that changes her speeds, and her collision with blocks, slopes and solid enemies.

**Not in this spec yet** (each an input where it feeds us; §10): which exact sprite pose follows
which input (the pose chart), pose fit (the push when her body grows), animation timing, liquids,
the speed booster's extra speed, knockback, bomb jumps, grapple, shinespark, pushes from the room
(Ceres, conveyors, quicksand), doors, spikes and the other blocks that act when she touches them, and slopes' sideways speed change.

**Baseline: about 3,600 lines**, counted with `wc -l`, tests excluded.

**Target: 1,600 lines** for the new model, tests and data tables excluded: motion about 500,
landings and turns about 150, the pose chart about 450 (as data records where it can be),
collision with blocks and solid enemies about 400, slack 100. Over the target is a review stop;
over the baseline a strict failure.

**Notation.** A quantity written `P.S` is a pixel part and a subpixel part, each 16 bits, in hex:
`2.C000` is 2.75 px. A signed one has a sign in front: `-4.E000` is 4.875 px a frame upward or
leftward. Positions are her body's centre; x grows rightward, y grows downward. A frame reads the
pad the player holds when it starts.

---

## 1. What the player can tell about Samus

Every quantity here is something the player sees or can measure frame by frame.

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| position | x, y: `P.S`; wraps mod 2^32 | where she is drawn | every move; output |
| horizontal speed | signed `P.S`, rightward positive | how fast and which way she walks, runs or drifts, before the dash | `SpeedBuildsAndBrakes`, `TurningRound`, `DriftInTheAir`, `Landing`, `WallsAndCeilings` |
| dash | `P.S` ≥ 0 | the extra speed from holding run, added the way she is moving (her facing when her horizontal speed is 0); it carries into a jump or the ball (the mockball) | `Dash`, `TurningRound`, `Jumping` |
| vertical speed | signed `P.S`, downward positive | how fast she rises or falls | `RisingAndFalling`, `Jumping`, `Landing`, `Moonfall` |
| bounces | count 0..=2 | how many times the ball has bounced since it last rested | `Landing` |
| pose | §2 | the sprite | every behaviour; output |

Each frame she moves horizontally by her horizontal speed plus her dash, the dash taken the way the horizontal speed
points.

### 1.2 What she remembers

Some of what she does depends on what came before. These are behaviours, told by what the player
sees; how the model keeps them is the implementer's choice. A vector shows them by its lead-in
frames (the vectors' format), never by a value in its start.

- **Carrying on.** Her horizontal motion either follows the pad (it builds while she holds a side,
  and in the air it stops when she lets go) or carries on by itself (it brakes on the ground and
  keeps building in the air with the pad let go). It carries on after a jump or a fall she started
  with a dash, while a spin jump drifts (`DriftInTheAir`), through a wall jump, and while she
  skids. It follows the pad again once she stops braking, halts, lands, hits a wall or ceiling, or
  takes off.
- **Skidding.** A skid is her moving against her facing: she turned round, or reversed a spin, and
  still travels the old way. A turn from a standstill is a skid of zero: its first frame brakes, so
  she doesn't creep until the frame after (`TurningRound`).
- **Her dash.** The dash she builds running stays hers, through jumps and into the ball, until a
  halt takes it. Where a pose change halts her but leaves the dash's value on her, the next frame
  she moves the dash is gone.
- **The moonfall** (`Moonfall`).
- **A spin carried on.** A spin she was already in two frames earlier, or that followed a wall
  jump, can ready a wall jump or be reversed (`WallJump`, `TurningRound`).

A vector starts at a moment where none of this is in play: her motion follows the pad unless she
is skidding, any dash she has is hers to keep, and a vertical speed of 0 means she stands on the ground,
except in a jump, a spin, a fall or a wall jump, where it means she has just started to fall.

### 1.3 What she reads from the world and the player (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| pad | held: run, jump, shoot, left, right, up, down, aim up, aim down; just pressed: jump | most behaviours |
| equipment | hi-jump, space jump: `bool` | `Jumping` |
| world | the room's blocks and solid enemies, answered as §8 says | §8 |
| spin turned far | `bool`: the spin's sprite has turned far enough to kick off a wall | `WallJump` |
| elevator moving | `bool` | `RidingTheElevator` |

---

## 2. Her pose as the player sees it

| Part | Values |
| --- | --- |
| body | `Upright`, `Crouched`, `Ball`, `SpringBall` |
| action | `AgainstWall` (pressed to the wall she ran into), `Crouching`, `Falling`, `Jumping` (a straight jump, or the spring ball's hop), `Moonwalking`, `Morphing`, `Rolling`, `Running`, `Spinning`, `StandingUp`, `Still`, `TakingOff`, `Turning` (on the ground or in the air), `Unmorphing`, `WallJumping` |
| facing | `Left`, `Right`, `Screen` (facing the player; counts as `Right` wherever a side is needed) |
| aim | the arm's direction; it changes only the sprite, except for `AimingAndShooting` |

The ball `Still` is the ball at rest. While her body grows or shrinks, she moves as
`SpeedBuildsAndBrakes` says.

---

## 3. On the ground

### SpeedBuildsAndBrakes
- Parameter and mechanic. How fast her speed builds and its top, by what she is doing:

| She is | Speeds up by | Top speed |
| --- | --- | --- |
| running | 0.3000 | 2.C000 |
| rolling as a ball | 0.C000 | 3.4000 |
| moonwalking | 0.C000 | 0.8000 |
| in a straight jump or the spring ball's hop | 0.C000 | 1.4000 |
| in a spin jump or a wall jump | 0.C000 | 1.6000 |
| falling, upright or as a ball | 0.C000 | 1.0000 |
| turning round | 0.C000 | 0.0000 |

- Following the pad she gains the step each frame, up to the top speed (`WordwiseTopSpeed`), and
  moves along her facing. This applies running, moonwalking, rolling, turning, and in the air as §5
  says. Carrying on along the ground she loses 0.8000 a frame instead; when that would take her
  below zero she stops (horizontal speed 0) and follows the pad again.
- Standing, crouched, against the wall and while her body changes shape, the pad doesn't move her:
  she moves by her dash only. After that move, standing and against the wall halt her (horizontal speed 0,
  dash 0, following the pad); crouched does the same; while her body changes shape she keeps both
  speeds. Facing the screen she
  doesn't move at all and keeps her speeds.
- The ball at rest, following the pad, moves by its dash only and halts after that move, unless it
  is in the air; braking or skidding, it rolls like running, with its own speeds, and keeps its
  dash (the mockball).
- checked: oracle (running, ground and turning vectors; samus-mockball, samus-bomb-jump-vertical).

### Dash
- Mechanic. Running with run held she gains 0.1000 dash a frame up to 2.0000. The cap tests the
  dash from before the step: it grows if its pixel part is `below` 2 or its subpixel part is
  `below` 0 (§9), else it becomes 2.0000, so 2.F000 grows to 3.0000 and then snaps to 2.0000
  (invisible in vanilla). In any other moving activity a dash she is to lose (§1.2) is gone (0).
  Standing, against the wall, a wall stop, a turn's end, the falling ball letting go and a spin
  reversed each take it (0).
- checked: oracle (running vectors with run held; samus-mockball for the carry); original only
  (C-dashcap-a, C-dashcap-b).

### TurningRound
- Mechanic. Turning round, on the ground or in the air, or reversing a spin (a spin facing the
  other way, §1.2), or the ball turning round, she adds her dash to her horizontal speed (dash 0) and keeps
  travelling the old way, now against her new facing: a skid. She brakes until the speed is spent,
  then builds the new way (turning's top speed is 0, so only the creep below). When the turn ends
  she loses her dash.
- Turning from a standstill slides her 0.C000 every other frame the new way: the speed goes 0,
  0.C000, 0, because turning's top speed is 0 and `WordwiseTopSpeed` keeps 0.C000 over 0 once.
- Turning on the ground over a drop, she sinks by her whole speed plus 1 px a frame instead of
  falling. In the air she turns under gravity (`RisingAndFalling`).
- checked: oracle (samus-input-poses turns, samus-screw-attack); constructed (the sinking).

### AimingAndShooting
- Quirk and output. Changing aim while running moves her 1 px forward on top of that frame's move,
  without a speed change; a block or a solid enemy in the way, or a wall stop that frame, puts her
  against the wall instead. Standing, unaimed
  and not landing, holding shoot restarts the standing animation.
- checked: constructed (open question 3).

### RidingTheElevator
- Mechanic. Facing the screen on a moving elevator, she moves down 1 px a frame; nothing else of
  hers changes. checked: constructed.

---

## 4. Leaving the ground

### Jumping
- Parameter and mechanic. Jump 4.E000 upward (hi-jump 6.0000); wall jump 4.A000 (hi-jump 5.8000);
  ball bounce 1.0000; the crouch lift 10 px.
- The first poses of a straight jump (taking off) don't steer: she moves by her dash only, follows
  the pad and still gains dash (`Dash`); she doesn't move vertically. On the first of them she
  starts rising at the jump speed; from a crouch without aim she is also lifted 10 px.
- Starting a spin jump, or the spring ball's hop from the ground, she rises at the jump speed.
  Starting a straight jump or a fall she carries on along her facing if she has a dash, else she
  follows the pad, so a running jump keeps its drift.
- With space jump, pressing jump while falling in a spin jumps again, when her fall speed read as
  8.8 (its pixel part's low byte and subpixel's high byte) is from 2.80 up to, not including, 5.00.
  She rises at the new speed that same frame.
- checked: oracle (take-off and spin vectors; y open: question 6); constructed (space jump).

### FollowingTheGround
- Mechanic. On the ground she is pulled down after her horizontal move, by that frame's whole speed
  plus 1 px (1 px on a slope), and the floor stops her flush (§8). Walking off a ledge the pull
  finds no floor: unless she is rising, turning or her body is changing shape, she falls from
  rest (vertical speed 0, falling, bounces 0). checked: oracle (every ground vector).

### MovesAsShePosed
- Timing. On the frame she changes pose she still moves as the pose at the frame's start says: what she is doing,
  her speeds, and what a landing or a fall does, come from the pose at the frame's start. checked:
  oracle.

---

## 5. In the air

### RisingAndFalling
- Mechanic. Each frame she moves vertically by her vertical speed, then it grows by 0.1C00 downward.
- Falling, her speed stops growing only when its pixel part is exactly 5, so it settles at 5.1400
  (4.F800 + 0.1C00); a speed set above that keeps its value.
- Rising, she starts to fall (vertical speed 0) once the upward speed is spent, and in a normal or spin
  jump or the spring ball's hop as soon as jump is let go. The frame it ends she doesn't move
  vertically.
- checked: oracle (every air vector); constructed (C-fallcap-a, C-fallcap-b).

### DriftInTheAir
- Mechanic. In a straight jump, falling and in the ball in the air, she builds towards the top speed
  (`SpeedBuildsAndBrakes`). Following the pad, with neither left nor right held, she stops
  drifting at once (horizontal speed 0, no horizontal move); the falling ball instead halts (horizontal speed 0, dash
  0) but still moves by its dash that frame.
- Carrying on in the air doesn't brake: she still gains the step, so a running jump keeps or gains
  its drift with the pad let go. A skid in the air brakes, as on the ground.
- A spin jump drifts only while she holds the side she faces, or is skidding, or reached her top
  speed this frame: then she carries on (a skid stays a skid) and moves. Otherwise she stops
  drifting: horizontal speed 0, following the pad, no move.
- Through a wall jump she always drifts, carrying on along her facing.
- checked: oracle.

### Moonfall
- Quirk. A spin she enters when a turn ends, with jump not just pressed, starts with no rise: she
  drops under gravity with no falling limit and jump does nothing, until she lands. checked: oracle
  (r03); no vector yet (open question 8).

---

## 6. Wall jumps and landings

### WallJump
- Mechanic. Spinning in a spin carried on (§1.2), holding away from a wall within 8 px (left held: a
  wall on her right; otherwise right held: on her left) readies the wall jump, which the sprite
  shows. The wall is looked for where her x is after this frame's move.
- Once the spin has turned far enough, pressing jump with the wall closer than 8 px kicks her off:
  she doesn't move vertically that frame; she starts the wall jump rising at the wall-jump speed, x
  speed 0, following the pad.
- Looking for a wall writes her x subpixel as a stop against it would (FFFF looking right, 0
  looking left), without moving her. It meets solid enemies first.
- checked: oracle (samus-walljump-16, -22, -28, samus-grapple-walljump, item-morph-ball's leftward
  spin jumps).

### Landing
- Mechanic. Falling onto a floor upright she lands: vertical speed 0, bounces 0, horizontal speed 0, following the
  pad; the dash keeps its value.
  Landing while her body changes shape stops only her vertical motion; both speeds, and whether
  she carries on, are kept.
- The ball bounces twice before it rests: landing at 3 px or more with no bounce yet, it rises at
  the bounce speed (bounces 1); after bounce 1 it rises at the bounce speed less 1 px (bounces 2);
  after bounce 2, or slower than 3 px, it lands as above. The spring ball landing with jump held
  hops again at the jump speed instead, bounces 0.
- checked: oracle (r06 for the changing body); constructed (C-bounce).

---

## 7. Pose changes

How a pose change changes her speeds comes with the **pose chart**, a separate later spec, as data
keyed by what the player does. Until then a vector gives the pose a frame changes to, and the
recorded vectors avoid changes the chart decides. One fact the chart must carry, found now: where a
pose change stops her horizontal speed dead, it takes her dash with it (dash 0).

---

## 8. The world she moves through

There is no physics here: the world is data she meets. The room is a grid of 16 px blocks, each of
a kind: air, solid, a slope (with its shape), or a kind out of scope (§10). Solid enemies are boxes
at positions. Her body is a box of half-width `radius.x` and half-height `radius.y` around her
centre. Each move asks what is at the cells her body's leading edge reaches, and each kind answers
as below. These rules are Samus's; other actors get their own when their domains are written.

### BlocksStopFlush
- Mechanic. A move into a solid block stops her with her leading edge against its face: moving
  right or down she ends with that axis's subpixel FFFF, moving left or up with 0. Her leading edge
  at the target is `x + d + rx − 1` right, `x + d − rx` left, `y + d + ry − 1` down, `y + d − ry`
  up, from the pixel parts.
- A move tests only the row or column of her leading edge at the target, so a move of 16 px or
  more can pass a floor.
- A horizontal move is limited to 15 px either way, its subpixel kept (unreachable in vanilla
  motion).
- checked: oracle (every ground vector's floor); constructed (C-limit).

### WallsAndCeilings
- Mechanic. A horizontal move stopped by a wall, on the ground or in the air, halts her: horizontal speed 0,
  dash 0, following the pad. The ground pull that frame still uses the speed from before
  the stop. An upward move into a block stops flush and ends the rise: vertical speed 0, falling.
- checked: constructed (C-wall).

### SolidEnemies
- Quirk. A horizontal move of 0 still meets solid enemies on her heading. A solid enemy she
  already touches on the tested side stops a move with no distance and sets her **y** subpixel to
  0, on either axis; one ahead stops her flush against it. checked: constructed.

### SlopesCarry
- Mechanic. On a slope the ground pull is 1 px. A horizontal move that meets solid enemies ends by
  seating her on the slope, or under the sloped ceiling, she reached: a whole-pixel shift of y
  before the vertical move. checked: oracle (samus-walljump-16/22/28 frame 44); no vector yet.

---

## 9. Numbers

- `below(a, b)` on 16-bit parts: bit 15 of (a − b) mod 2^16 is set. 1000 is not below C000; C000 is
  below 4000 and below 0.
- Adding `P.S` carries the subpixel into the pixel part, mod 2^16 on the pixel part.
- A vertical move uses the vertical speed from before that frame's gravity; a halt's speeds are those
  after that frame's moves.

### WordwiseTopSpeed
- Quirk, **Toggle**. The top speed is tested part by part, on the speed's size. `Wordwise`
  (vanilla): she keeps her speed if its pixel part is `below` the top's; else if the pixel parts
  differ she takes the top speed; else she keeps it if the subpixels are equal or hers is `below`,
  else takes the top. So 2.1000 snaps up to 2.C000, 3.C000 is kept over 3.4000, 0.C000 is kept
  over 0, and 2.2000 snaps down to 1.4000. `Exact`: the smaller of the two. checked: oracle
  (`Wordwise`, the running vectors); constructed (C-cap, C-cap-exact).

## 10. Out of scope, and what remains

- **The pose chart:** which pose follows which pad and world event, and what each change does to
  her speeds (§7). It needs its own observer lane, written as records keyed by what the player
  does.
- **Slopes:** the sideways speed change on a slope, and the corners of steep blocks.
- **Liquids and the speed booster:** other speeds and gravity in water and lava; the booster's dash
  cap of 7 and its higher jump.
- **Pushes:** room pushes, quicksand, knockback, bomb jumps, grapple, shinespark.

## 11. Open questions

- Whether a pose change the chart makes at an animation's end moves her the extra pixel of
  `AimingAndShooting`: for the pose-chart spec.
- Whether the elevator's descent differs from an ordinary 1 px move anywhere she can see it.

1. "A spin she was already in two frames earlier" (§1.2) is not yet traced against a record: it
   may count pose changes rather than frames.
2. The pose chart's vectors (take-offs, turns' ends, landings other than the changing body) wait
   for its lane; the vectors so far change pose only by a jump, a fall or a turn from the ground.
3. `AimingAndShooting`, `SlopesCarry`, `SolidEnemies` and the ceiling half of `WallsAndCeilings`
   have no vectors yet.
4. The water rows, terrain-quicksand and samus-ceres-push gave no vectors: their liquid and push
   rules are out of scope.
5. Landing on a solid enemy's top (item-morph-ball frame 797) clears the y subpixel before she
   touches it: a collision question.
6. The crouch take-off's lift appears a frame apart from pose fit (item-morph-ball 1311).
7. A skid at exactly 0.8000 ends following the pad in the records (samus-input-poses:120), though
   braking by 0.8000 lands on 0, not below. Either the turn's slide is larger than 0.8000 or
   something else ends the skid; not yet traced.
8. `Moonfall` needs a vector with a lead-in through the turn's end; r03 shows one in a straight jump's
   pose as well as a spin, so the entry's "a spin" may be too narrow.
9. 31 of the 149 recorded frames were dropped because no lead-in from a plain start reaches them
   through frames the spec covers (mostly after a wall jump's kick or a body change in the air).
