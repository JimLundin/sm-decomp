# Samus moving through the world: behaviour spec

What the player sees Samus do when she runs, stops, turns, crouches, rolls, jumps, spins, falls,
wall-jumps and lands, and how the world's blocks, slopes, solid enemies, floors, walls and
ceilings stop her. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-motion.toml`](../../scenarios/vectors/samus-motion.toml).

## Header

**Scope (Jim, 2026-11-07).** One domain, because the player sees one thing: Samus moving through
the world. It covers her speeds and position, how her pose follows from input and the world where
that changes her speeds, and her collision with blocks, slopes and solid enemies.

**Not in this spec yet** (each an input where it feeds us; §10): pose fit (the push when her
body grows; its place in the frame is `PoseFitAfterMoves`), animation timing, liquids,
the speed booster's extra speed, knockback, bomb jumps, grapple, shinespark, pushes from the room
(Ceres, conveyors, quicksand), and what a cell she reaches then does (`TouchedCells` says which
she reaches; `OtherBlockKinds` how each kind meets her motion).

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
  takes off, or stops steering during a turn (`TurningRound`).
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
| frame parity | `Even` or `Odd`: the game's frame count, alternating every frame | `BlocksStopFlush` |

---

## 2. Her pose as the player sees it

| Part | Values |
| --- | --- |
| body | `Upright`, `Crouched`, `Ball`, `SpringBall` |
| action | `AgainstWall` (pressed to the wall she ran into), `Crouching`, `Falling`, `Jumping` (a straight jump, or the spring ball's hop), `Moonwalking`, `Morphing`, `Rolling`, `Running`, `Spinning`, `StandingUp`, `Still`, `TakingOff`, `Turning` (on the ground or in the air), `Unmorphing`, `WallJumping` |
| facing | `Left`, `Right`, `Screen` (facing the player; counts as `Right` wherever a side is needed) |
| aim | the arm's direction: `Forward`, `Up`, `UpForward`, `DownForward`, `Down`; it changes only the sprite, except for `AimingAndShooting` |

**Forward** is the side she faces, **back** the other side; holding forward means holding the pad
towards her facing. The sprite has more variants than these parts tell apart (the arm drawn out
while shoot is held, the landing sprites, the spin drawn as the space jump or the screw attack);
they move her the same, so the chart (§7) doesn't tell them apart either.

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
- Turning with no button held at all, on the ground or in the air, her skid ends after that
  frame's move: she keeps her horizontal speed, which now follows the pad and points along her new
  facing. So a skid braked to exactly 0 ends there too. On the ground the same happens when she
  holds only buttons that bring no pose change (run, shoot or jump alone): no row of the pad chart
  fits, and her let-go outcome ends the skid (`LettingGo`). In the air, holding any such button, she
  keeps skidding: the speed still points the old way, and the skid ends only as
  `SpeedBuildsAndBrakes` says, when braking would take it below zero.
- Turning on the ground over a drop, she sinks by her whole speed plus 1 px a frame instead of
  falling. In the air she turns under gravity (`RisingAndFalling`).
- checked: oracle (samus-input-poses turns, samus-screw-attack; R133 for letting go, R11 and R12
  (samus-input-poses:120-121) for run held on the ground: braked to 0 at 120, she creeps 0.C000 at
  121, which a skid kept at 0 would not; R141 for run held in the air); constructed (the sinking).

### AimingAndShooting
- Quirk and output. Changing aim while running moves her 1 px forward on top of that frame's move,
  without a speed change; a block or a solid enemy in the way, or a wall stop that frame, puts her
  against the wall instead. Standing, unaimed
  and not landing, holding shoot restarts the standing animation.
- The step comes after that frame's moves and the ground pull. A solid enemy within its reach
  (`SolidEnemies`) stops it with no move at all; one she already touches still clears her y
  subpixel. A block in the way stops it as any horizontal move (`BlocksStopFlush`): by whole pixels
  up to the face, at most the 1 px, with her x subpixel written FFFF or 0, so a face less than a
  pixel ahead moves her that part of a pixel. The floor and ceiling checks of `SlopesCarry` don't
  run after the step, so it can leave her bottom edge in a sloped floor's solid part.
- checked: oracle (A1, the step); constructed (C-aim-short, C-aim-enemy, C-aim-slope; the shot).

### RidingTheElevator
- Mechanic. Facing the screen on a moving elevator, she moves down 1 px a frame, a move a solid or
  stepped block stops as any downward move does (`BlocksStopFlush`); a sloped surface under her
  there is out of scope. Nothing else of hers changes, her speeds and dash included. checked:
  constructed (C-elevator).

---

## 4. Leaving the ground

### Jumping
- Parameter and mechanic. Jump 4.E000 upward (hi-jump 6.0000); wall jump 4.A000 (hi-jump 5.8000);
  ball bounce 1.0000; the crouch lift 10 px.
- The first poses of a straight jump (taking off) don't steer: she moves by her dash only, follows
  the pad and still gains dash (`Dash`); she doesn't move vertically and the pull doesn't act. The
  frame she changes into them she starts rising at the jump speed, without moving vertically that
  frame; from a crouch without aim she is also lifted 10 px.
- Starting a spin jump, or the spring ball's hop from the ground, she rises at the jump speed.
  Starting a straight jump or a fall she carries on along her facing if she has a dash, else she
  follows the pad, so a running jump keeps its drift.
- With space jump, pressing jump while falling in a spin jumps again, when her fall speed read as
  8.8 (its pixel part's low byte and subpixel's high byte) is from 2.80 up to, not including, 5.00.
  She rises at the new speed that same frame.
- checked: oracle (J1, J2, J3, and the take-off and spin vectors; the crouch lift open: question
  6); constructed (space jump).

### FollowingTheGround
- Mechanic. On the ground she is pulled down after her horizontal move, by that frame's whole speed
  plus 1 px (1 px on a slope), and the floor stops her flush (§8). Walking off a ledge the pull
  finds no floor: unless she is rising, turning or her body is changing shape, she falls from
  rest (vertical speed 0, falling, bounces 0). checked: oracle (every ground vector).

### MovesAsShePosed
- Timing. On the frame she changes pose she still moves as the pose at the frame's start says: what she is doing,
  her speeds, and what a landing or a fall does, come from the pose at the frame's start. checked:
  oracle (J1, J2, A1).

### PoseFitAfterMoves
- Timing. A frame ends in this order: her moves (horizontal, the ground pull or her vertical move,
  then the aim change's step), then what they met (a landing, a fall, a ceiling, a wall), then the
  pose change. Within the pose change, pose fit's push comes first: when the new pose's body is
  taller, she is pushed clear of what is above or below her from the position her moves left
  (the push itself is out of scope, the header). Then come the new pose's speeds (a jump's rise,
  `Jumping`), then the crouch take-off's 10 px lift, from the pushed position.
- So the push sees her before the lift: her y at the frame's end is the moves' y, plus the push
  found there, less the 10 px. A push found after the lift would differ wherever a block is within
  10 px above her.
- A pose that doesn't change, or a change to facing the screen, gets no push.
- checked: original only (item-morph-ball 1311 shows the lift and pose fit a frame apart, §11
  question 6).

---

## 5. In the air

### RisingAndFalling
- Mechanic. Each frame she moves vertically by her vertical speed, then it grows by 0.1C00 downward.
- Falling, her speed stops growing only when its pixel part is exactly 5, so it settles at 5.1400
  (4.F800 + 0.1C00); a speed set above that keeps its value.
- Rising, she moves by her upward speed and then the pull takes 0.1C00 off it. The upward speed is
  spent once the pull has taken it past zero, so that it points downward by even a subpixel; a
  speed of exactly 0 is not spent, and she rises by 0 for one more frame. The rise ends at the
  start of the frame after it is spent, and in a normal or spin jump or the spring ball's hop at
  the start of any frame with jump not held: vertical speed 0, falling, and she doesn't move
  vertically that frame; the pull then acts as in a fall, so the frame ends at +0.1C00.
- A frame's vertical move of exactly 0 (a fall from rest, a rise at 0) is never stopped by
  blocks, even with her bottom or top edge already inside one (`BlocksStopFlush`): it never lands
  her or ends her rise.
- An upward speed of exactly one pull is never met from a jump: no rise speed in `Jumping` is a
  whole number of pulls. The ball rises at exactly 0 when it bounces the second time, so it
  rises by 0 for one frame before the rise ends.
- checked: oracle (every air vector; E1 for the speed past zero); constructed (C-fallcap-a,
  C-fallcap-b, E2, E3).

### DriftInTheAir
- Mechanic. In a straight jump, falling and in the ball in the air, she builds towards the top speed
  (`SpeedBuildsAndBrakes`). Following the pad, with neither left nor right held, she stops
  drifting at once (horizontal speed 0, no horizontal move); the falling ball instead halts (horizontal speed 0, dash
  0) but still moves by its dash that frame.
- Carrying on in the air doesn't brake: she still gains the step, so a running jump keeps or gains
  its drift with the pad let go. A skid in the air brakes, as on the ground.
- A spin jump drifts only while she holds the side she faces, or is skidding, or reached her top
  speed this frame: then she carries on (a skid stays a skid) and moves. Otherwise she stops
  drifting: horizontal speed 0, following the pad, and she moves by her dash alone.
- Through a wall jump she always drifts, carrying on along her facing.
- checked: oracle.

### Moonfall
- Quirk. A spin she enters when a turn ends, with jump not just pressed, starts with no rise: her
  vertical speed is 0 and she drops under gravity (`RisingAndFalling`), but her fall speed keeps
  growing by 0.1C00 past the falling limit, and pressing jump starts no rise (not even with space
  jump), until she lands. A wall jump is not blocked: the spin readies and kicks off a wall near
  her as any spin carried on does (`WallJump`), and the kick, rising at the wall-jump speed, ends
  the moonfall. checked: oracle (r03: M1); constructed (C-moonfall-jump,
  C-moonfall-nolimit, both from r03's start, with its vertical motion).

---

## 6. Wall jumps and landings

### WallJump
- Mechanic. Spinning in a spin carried on (§1.2), holding away from a wall near her (left held: a
  wall on her right; otherwise right held: on her left) readies the wall jump, which the sprite
  shows.
- A wall is near her when a test move of 8 px towards it, from where her x is after this frame's
  move, would be stopped (`BlocksStopFlush`, `SolidEnemies`): fewer than 8 px of air between her
  edge and the wall. Readying and the kick use this same test.
- Once the spin has turned far enough, pressing jump with a wall near her kicks her off:
  she doesn't move vertically that frame; she starts the wall jump rising at the wall-jump speed, x
  speed 0, following the pad.
- Finding a block wall writes her x subpixel as a stop against it would (FFFF looking right, 0
  looking left), without moving her. The test meets solid enemies first. Finding a solid enemy
  instead, it writes her x nothing: an enemy she already touches on that side clears her y subpixel
  to 0, as any test against one does (`SolidEnemies`), and one ahead of her writes nothing.
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

## 7. Pose changes: the pose chart

Which pose she shows next, from the pad, her equipment and what her moves met, and what the change
does to her speeds. Written as data: a few tables keyed by what she is doing and what the player
holds or has just pressed. Vectors: the `[[pose_step]]` entries of the vector file.

**Size.** The pose selection it replaces is about 1,100 lines of old code. **Target: 200 lines**
of model (the chart's reader and the effects), plus the tables below as data records (about 120
rows); inside the header's 450 for the chart.

### PoseFollows
- Mechanic and timing. Her pose changes at most once a frame. What decides, the first that applies:
  1. an animation ending hands over its pose (`AnimationEnds`);
  2. a wall stop while she runs puts her against the wall (`WhatHerMovesMet`);
  3. what her moves met: a landing, walking off a ledge, the wall-jump kick (`WhatHerMovesMet`);
  4. the pad (`PadChart`), from the pose at the frame's start and the pad at the frame's start;
  5. when no row of the pad chart fits, her let-go outcome (`LettingGo`).
- The frame runs in this order, and this is what the player can tell apart at its end: she moves as
  the pose at the frame's start says (`MovesAsShePosed`); what those moves met is known; the pose
  is chosen as above; then the change's effects act on her speeds and dash. So a change's effects
  never alter that frame's moves, and the frame ends in the pose chosen, with its effects applied.
- One that comes lower in the list loses its pose and its effects to a higher one: a landing
  replaces a let-go outcome's effect, and an animation's end replaces a landing's pose (the landing
  still stops her as `Landing` says).
- Where a change stops her horizontal speed dead, it takes her dash with it (dash 0).
- checked: oracle (the `[[pose_step]]` vectors).

### PadChart
- Mechanic. A row fits when every button it names as just pressed was just pressed this frame and
  every button it names as held is held; other buttons don't matter. Where two rows of her situation
  fit, the higher one wins. "Aim" in a next pose means the aim the `Aiming` rule gives. A row whose
  next pose is the one she already has leaves her as she is.

| She is | Pad | Next pose | Effect |
| --- | --- | --- | --- |
| standing still (upright, or just landed) | jump pressed | upright `TakingOff`, take-off aim | `Jumping`'s take-off |
| | down pressed | crouched `Crouching`, crouch aim | none |
| | back, shoot, an aim button held | `Moonwalking` facing back, `DownForward` with aim down, else `UpForward` | none |
| | back and both aim buttons held | `Turning` | `TurningRound` |
| | both aim buttons held | `Still`, `Up` | none |
| | forward with aim up or up held | `Running`, `UpForward` | none |
| | forward with aim down or down held | `Running`, `DownForward` | none |
| | back and shoot held | `Moonwalking` facing back, `Forward` | none |
| | back held | `Turning` | `TurningRound` |
| | up, aim up or aim down held | `Still`, standing aim | none |
| | forward held | `Running`, `Forward` | none |
| running | down pressed | crouched `Crouching`, `Forward` | none |
| | jump pressed | `Spinning` | `Jumping`'s spin start |
| | forward with aim up or up held | `Running`, `UpForward` | none |
| | forward with aim down or down held | `Running`, `DownForward` | none |
| | forward held | `Running`, `Forward` | none |
| | back held | `Turning` | `TurningRound` |
| | up, aim up or aim down held | upright `Still`, standing aim | none |
| taking off | jump and back held | `Turning` in the air | `TurningRound` |
| | jump held, or shoot held | `Jumping`, air aim | none |
| in a straight jump | aiming `Down`, down pressed | ball `Morphing` | none |
| | back held | `Turning` in the air | `TurningRound` |
| | anything else held | `Jumping`, air aim | none |
| spinning | shoot pressed | `Jumping`, `Forward` | none |
| | forward and jump held | `Spinning` | none |
| | shoot with up, down or an aim button held, or up, down or an aim button held | `Jumping`, air aim | none |
| | forward held | `Spinning` | none |
| | back held | `Spinning`, facing back: the spin reversed | `TurningRound` |
| falling (upright) | back held | `Turning` in the air | `TurningRound` |
| | anything else held | `Falling`, air aim | none |
| crouched | up pressed | `StandingUp`, crouch aim | none |
| | back pressed | crouched `Turning` | none |
| | down pressed | ball `Morphing` | none |
| | jump pressed | upright `TakingOff`, `Forward` | `Jumping`'s take-off |
| | both aim buttons held | `Still`, `Up` | none |
| | forward held | upright `Still`, `Forward` | she stands at once: 5 px up |
| | aim up or aim down held | `Still`, crouch aim | none |
| a ball, on the ground or in the air | up or jump pressed | crouched `Unmorphing` | none |
| | left or right held | the same ball facing that side, `Rolling` on the ground | none |
| turning on the ground | the new forward and jump held | `Spinning`, the new facing | `Jumping`'s spin start |
| | jump pressed | upright `TakingOff`, the new facing | `Jumping`'s take-off |
| | the new forward held | `Turning` (kept) | none |
| turning in the air | any button held | `Turning` (kept) | none |
| moonwalking | down pressed | crouched `Crouching`, facing the way she looks, `Forward` | none |
| | jump pressed | `Turning`, then a spin the way she walks (`AnimationEnds`), aimed by aim up or aim down | `TurningRound` |
| | the way she walks, shoot and an aim button held | `Moonwalking`, `UpForward` with aim up, else `DownForward` | none |
| | the way she walks and shoot held | `Moonwalking`, `Forward` | none |
| | the way she looks held | `Running` that way, `Forward` | none |
| | the way she walks held | `Turning` | `TurningRound` |
| wall jumping | down pressed | ball `Morphing` | none |
| | back held | `Spinning`, facing back | `TurningRound` |
| | jump held | `WallJumping` (kept) | none |
| against the wall | jump pressed | upright `TakingOff`, `Forward` | `Jumping`'s take-off |
| | down pressed | crouched `Crouching`, `Forward` | none |
| | back and an aim button held | `Moonwalking` facing back, `DownForward` with aim down, else `UpForward` | none |
| | back held | `Turning` | `TurningRound` |
| | forward held | `Running`, `Forward` | none |

- Facing is kept unless the row says otherwise; `Turning` faces back (the turn shows her new
  facing, and `TurningRound` turns her). A moonwalk faces the way she walks, which is backwards:
  her sprite looks the other way.
- Body follows the action: `Running`, `Moonwalking`, `TakingOff`, `Jumping`, `Spinning`, `Falling`,
  `WallJumping`, `AgainstWall` and `StandingUp` are upright; `Crouching` and `Unmorphing` end
  crouched; `Morphing` ends a ball.
- checked: oracle (P1, P13 to P18, P20 to P22, P24).

### Aiming
- Mechanic. The aim a next pose takes from the held buttons, by the rule its row names:

| Rule | Both aim buttons | Up | Down | Aim up | Aim down | Forward and up / down | Else |
| --- | --- | --- | --- | --- | --- | --- | --- |
| take-off | (as aim up) | `Up` | `Forward` | `UpForward` | `DownForward` | as up / `Forward` | `Forward` |
| standing | `Up` | `Up` | `Forward` | `UpForward` | `DownForward` | (running rows) | `Forward` |
| crouch | `Up` | (stands) | `Forward` | `UpForward` | `DownForward` | (stands) | `Forward` |
| air | `UpForward` | `Up` | `Down` | `UpForward` | `DownForward` | `UpForward` / `DownForward` | `Forward` |

- Where up and an aim button are both held, up wins; where up or down and forward are held, the
  forward diagonal wins. checked: oracle (P2, P9, P24).

### LettingGo
- Mechanic. With no row of the pad chart fitting, and also with no button held at all, her pose and
  speeds change as her situation says. Turning in the air is the one exception: there it happens
  only with no button held at all, and with any button held she keeps turning as she is
  (`TurningRound`).

| She is | Next pose | Effect |
| --- | --- | --- |
| standing, crouched, against the wall | kept, aim `Forward` | she follows the pad again (a skid ends) |
| running | upright `Still`, `Forward` | the dash joins her horizontal speed (dash 0) and she carries on, braking (`SpeedBuildsAndBrakes`); with a horizontal speed of 0 she only follows the pad again |
| in a straight jump or taking off, a ball in the air | kept | as running |
| moonwalking | upright `Still`, facing the way she looks | she follows the pad again |
| turning, on the ground or in the air | kept | she follows the pad again (`TurningRound`) |
| spinning | kept | none |
| falling (upright) | kept, aim `Forward` | dash 0 |
| a ball on the ground | `Still` | horizontal speed 0, dash 0, following the pad |
| wall jumping | `Spinning` | horizontal speed 0, dash 0, following the pad |

- checked: oracle (P3, and R133 and R141 of `TurningRound`).

### WhatHerMovesMet
- Mechanic. What this frame's moves met chooses her pose over the pad:

| Her moves met | She was | Next pose | Effect |
| --- | --- | --- | --- |
| a floor (landing) | upright, in a straight jump or falling | upright `Still`, facing kept, aim kept (`Down` becomes `Forward`) | `Landing` |
| | spinning or wall jumping | upright `Still`, facing kept, `Forward` | `Landing` |
| | a ball, bouncing (`Landing`) | kept | `Landing`'s bounce |
| | a ball, coming to rest | ball `Still`, facing kept | `Landing` |
| no floor (walking off a ledge, `FollowingTheGround`) | upright on the ground | upright `Falling`, facing kept, `Forward` | `FollowingTheGround`'s fall |
| | a ball | ball `Falling`, facing kept | `FollowingTheGround`'s fall |
| a ceiling | rising | kept | `WallsAndCeilings` |
| a wall near her as she presses jump (`WallJump`) | spinning | `WallJumping`, facing kept | `WallJump`'s kick |
| a wall (a wall stop) | running | `AgainstWall`, facing kept; aim `UpForward` or `DownForward` kept, else `Forward` (`Up` stays `Still`, `Up`) | `WallsAndCeilings` |
| a block or solid enemy in the way of the aim change's step | running into another running pose | `AgainstWall`, with the aim of the running pose chosen | `AimingAndShooting` |

- A wall stop while running wins over walking off a ledge in the same frame.
- checked: oracle (P4, P5, P6, P7, P8).

### AnimationEnds
- Mechanic. Some poses last only as long as their animation, and its end hands over the next pose
  whatever the pad (animation timing is out of scope, §10: a vector says on which frame it ends):

| Ending pose | Next pose | Effect |
| --- | --- | --- |
| `TakingOff` | `Jumping`, aim kept | none: her rise speed came with the take-off (`Jumping`) |
| `Turning` on the ground | upright `Still`, the new facing, aim kept | dash 0 (`Dash`) |
| `Turning` in a jump | `Jumping`, the new facing | dash 0 |
| `Turning` in a fall | `Falling`, the new facing | dash 0 |
| crouched `Turning` | crouched `Still`, the new facing | dash 0 |
| `Turning` from the moonwalk's jump | `Spinning`, the new facing | `Moonfall` |
| `Crouching` | crouched `Still` | none |
| `StandingUp` | upright `Still` | none |
| `Morphing` | ball `Still` on the ground, ball `Falling` in the air | none |
| `Unmorphing` | crouched `Still` on the ground, upright `Falling` in the air | none |

- checked: oracle (P10, P11, P12, P19, P23, M1).

### Out of the chart's scope
- The spring ball's rows, the grapple, knockback, the boost after damage, x-ray, shinespark, the
  crystal flash, the drain, liquids, and equipment that changes mid-pose.
- Whether her arm's step of `AimingAndShooting` also comes with a running sprite that an
  animation's end hands over.

---

## 8. The world she moves through

There is no physics here: the world is data she meets. The room is a grid of 16 px blocks, each
of a kind: air, solid, a stepped block, a sloped surface, or a kind out of scope (§10). A stepped
block is solid in some of its four 8 px quarters (a half, a quarter, three quarters); a sloped
surface has a floor or a ceiling whose height changes column by column (its shape). Solid enemies are boxes
at positions. Her body is a box of half-width `radius.x` and half-height `radius.y` around her
centre. Each move meets the cells her body's leading edge reaches (`TouchedCells`), and each kind
acts on her motion as below. These rules are Samus's; other actors get their own when their domains are written.

### BlocksStopFlush
- Mechanic. A move into a solid block stops her with her leading edge against its face: moving
  right or down she ends with that axis's subpixel FFFF, moving left or up with 0. Her leading edge
  at the target is `x + d + rx − 1` right, `x + d − rx` left, `y + d + ry − 1` down, `y + d − ry`
  up, from the pixel parts.
- A move tests only the row or column of her leading edge at the target, so a move of 16 px or
  more can pass a floor.
- A horizontal move is limited to 15 px either way, its subpixel kept (unreachable in vanilla
  motion).
- A stepped block stops a move on either axis the same way, with the same subpixels (FFFF or 0),
  but against the face of the 8 px quarter her leading edge reaches, not the block's face. The
  quarters that count are those in the half her leading edge reaches (a side's half for a sideways
  move, an upper or lower half for a vertical one) that her body overlaps across it. So
  reaching the far half of a block with both halves solid, she stops at the far half's face, her
  body overlapping the near half.
- A stop never moves her backwards: when her edge at the start is already past the face it stops
  at, she moves 0 px, and only the subpixel is written (FFFF or 0), which is forward. So a move of
  any length above 0, even 0.0001, whose leading edge starts inside a solid block or quarter is
  stopped there: a downward one lands her.
- Blocks never stop a move of exactly 0.0000, on either axis: her position, subpixel included,
  is unchanged, and it doesn't end her seat on a slope (`SlopesCarry`). This holds also with
  her leading edge already inside a solid block, quarter or sloped surface, as in the first frame
  of a fall from rest (C-step-far, C-step-noback, C-span-step: she falls on, ending at +0.1C00).
- When blocks across her body would stop a move at different places, one of them decides, and it
  need not be the nearest: for a horizontal move, the topmost of them; for a vertical move, the
  leftmost on an `Even` frame and the rightmost on an `Odd` one. Air, and a sloped surface whose
  stop she doesn't reach, never decide.
- checked: oracle (every ground vector's floor); constructed (C-limit, C-step-far, C-step-noback,
  C-step-left, C-step-down, C-step-up, C-span-step, C-down-even, C-down-odd).

### WallsAndCeilings
- Mechanic. A horizontal move stopped by a wall, on the ground or in the air, halts her: horizontal speed 0,
  dash 0, following the pad. The ground pull that frame still uses the speed from before
  the stop. An upward move into a block stops flush and ends the rise: vertical speed 0, falling,
  with no pull that frame.
- checked: constructed (C-wall, C-ceiling).

### SolidEnemies
- Quirk. Her box and an enemy's are each centre ± half-size. Her reach is her box at the move's
  end pushed 1 px further on: moving left or up unless the end's subpixel is 0, moving right or
  down unless both her start's subpixel and the move's are 0. She reaches an enemy when, there,
  the centres are closer than the two half-sizes summed on both axes.
- An enemy she reaches is judged from where she starts, by the gap between the boxes on that
  side (the enemy's near side less her near side, less 1): a gap of 0 is touching: she doesn't
  move, and her **y** subpixel is set to 0, on either axis; a larger gap stops her by that many
  whole pixels, her subpixel kept; a negative one (already overlapping) never stops her: the move
  goes as if that enemy were not there, so another solid enemy in its path decides it, or with
  none, blocks do (`BlocksStopFlush`; C-enemy-overlap). So a move of 0 is never stopped by one.
- A stop against an enemy is a wall, a ceiling or a floor as the move's direction says: sideways it
  halts her (`WallsAndCeilings`), upward it ends the rise, downward she lands (`Landing`).
- A solid enemy in a move's path decides that move alone: she stops against it and no block
  stops her, even a nearer one, so the move can end with her edge overlapping a block. Blocks
  stop her only on a path free of solid enemies (`BlocksStopFlush`).
- checked: constructed (C-enemy-ahead, C-enemy-adjacent, C-enemy-down, C-enemy-overlap,
  C-aim-enemy).

### SlopesCarry
- Mechanic. On a slope the ground pull is 1 px instead of her speed plus 1 px.
- She is on a slope from the moment something seats her on one until her next vertical move
  of more than 0 starts (a move of exactly 0 leaves her seated, `BlocksStopFlush`). Two things
  seat her: the check after every horizontal move (below), and a downward move
  stopped by a stepped block. So a frame's pull is 1 px when that frame's horizontal move seated
  her, or the previous frame's pull stopped on a stepped block.
- After every horizontal move, whether or not it met solid enemies, two checks run, each asking just
  one cell and only on a sloped surface (a stepped block takes no part in them):
  - **Floor.** The block at her centre column and her bottom edge's row, if it is a sloped floor.
    Her bottom edge is in its solid part when its row there is the surface's top row at her
    centre column or lower. Then she is lifted by whole pixels until her bottom edge is on
    the last air row above the surface, and that lift seats her. With her bottom edge already on
    that last air row, or higher, nothing happens and she is not seated, so that frame's pull is
    her speed plus 1 px (C-seat-lift against C-seat-none).
  - **Ceiling.** Then the block at her centre column and her top edge's row, if it is a sloped
    ceiling. When her top edge is in its solid part, or on the first air row under it, she is
    pushed down by whole pixels until her top edge is on that first air row, and the push seats
    her, also when it moves her 0 px because her top edge was already there.
- A sloped surface never stops a horizontal move (how a sloped floor changes the move's length is
  `SlopesSlowTheWalk`).
- A downward move meets a sloped floor only in the block at her centre column; one in the other
  columns her body spans doesn't stop her. When her bottom edge at the target reaches the last air
  row above the surface's top there, or lower, the move stops with her bottom edge on that last
  air row: so a move that would end exactly on it already stops and lands her there. The stop
  never moves her up: with her bottom edge already below that last air row (inside the surface's
  solid part), she moves 0 px, her y subpixel kept, and lands. She moves by whole pixels only:
  the move's subpixel part is lost and her y subpixel is kept, not set to FFFF, whatever the
  surface's shape. With a solid or stepped block in another column of that
  row, which of the two decides is the order of `BlocksStopFlush`. This stop
  doesn't seat her; it lands her as any floor does (`Landing`). A stepped block, like a solid one,
  is met in every column her body spans and stops her flush with subpixel FFFF
  (`BlocksStopFlush`), and that stop seats her.
- Rising so that her top edge at the target reaches a sloped ceiling's solid part at her centre
  column, or the first air row under it, she doesn't move up at all that frame (her subpixel
  kept), and the rise ends as at a flat ceiling (`WallsAndCeilings`).
- The surface's height at each column is world data; the vectors give it as `world.slope_floor`
  and `world.slope_ceiling`.
- checked: oracle (samus-walljump-16/22/28 frame 44; samus-walljump-28:62 and
  samus-screw-attack:66 for the sloped ceiling; chain r01:677, samus-walljump-22:93 and
  samus-input-poses:426 for the landing that keeps the subpixel); constructed (C-slope-land,
  C-slope-edge, C-slope-ceiling-edge, C-seat-lift, C-seat-none, C-step-down, C-down-odd,
  C-aim-slope).

### SlopesSlowTheWalk
- Mechanic. A horizontal move made while her vertical speed is exactly 0 (standing, running,
  turning, rolling, the first frame of a fall from rest, or after a rise ends at the frame's
  start), whose leading edge's column meets a sloped floor, is shortened by that floor's walk
  factor. A sloped ceiling never shortens a move, and with any vertical speed but 0 no slope does.
- The cells that count are those `TouchedCells` names for the move: the leading edge's column at
  the move's unshortened target, in each row her body spans. Each sloped floor among them
  shortens it again, top row first. A solid block below it in that column still stops her as
  `BlocksStopFlush` says.
- The walk factor is world data, a multiple of 1/256 for each shape: 1.00 (a flat-topped shape),
  0.F0, 0.D8 (a half-height triangle), 0.C0 (a triangle, a quarter or a three-quarter triangle),
  0.B0 (a bumpy triangle), 0.80 (a half-width triangle), 0.50. Vectors give it as
  `world.slope_floor.factor`.
- The arithmetic, exactly. The move is first cut to a multiple of 0.0100 (1/256 px), rounding
  towards minus infinity, so a leftward move's length rounds up; only the pixel part's value mod
  256 is kept. Its length is then multiplied by the factor exactly (no bits are lost), and the
  direction put back. So 1.7000 at 0.C0 is 1.1400, 4.C000 at 0.D8 is 4.0200, and 0.C000 at 0.C0 is
  0.9000; a factor of 1.00 still cuts the move to a multiple of 0.0100.
- Her speeds don't change: only that frame's move is shorter. The aim change's step
  (`AimingAndShooting`) is a horizontal move too: on a 0.C0 floor it moves her 0.C000.
- checked: oracle (S1 samus-input-poses:512, S2 r01:717; the moves of r01:625-630 at 0.C0 and of
  r06:123-126 at 0.D8, aim steps included); constructed (C-slope-walk-left, C-slope-walk-air,
  C-slope-walk-ceiling).

### TouchedCells
- Output. Some cells react when she reaches them (one that changes the scrolling, an item, a door,
  a floor that crumbles); what they then do belongs to their own domains. Motion reports which cells she reaches,
  in order, so the world can raise those reactions. Two reports come out of a frame.
- **The cells her moves reach.** For each move of the frame (the horizontal move, the ground pull
  or her vertical move, the aim change's step, the elevator's 1 px), in the order they happen: the
  cells of her leading edge's line at the move's target, the line `BlocksStopFlush` names. For a
  horizontal move that is the column of her leading edge at the target, in each row her body
  spans where she stands before the move, top row first. For a vertical move it is the row of her
  leading edge at the target, in each column her body spans where she stands before the move, left
  to right on an `Even` frame and right to left on an `Odd` one. Each cell comes with the move's
  direction (left, right, up, down), because some reactions depend on it.
- The report ends at the first cell that stops her: the cells after it on the line are not
  reached. A move of exactly 0 reaches no cells (`BlocksStopFlush`), and neither does a move that
  a solid enemy decides (`SolidEnemies`). The wall-jump test (`WallJump`) and pose fit's push
  reach cells of their own, which belong to them, not to these reports.
- A cell that stands for another (`OtherBlockKinds`) is reported as the cell it stands for.
- **The cells she is inside.** Once each frame, before her moves, from her position at the frame's
  start: the cell in her centre column at her bottom edge's row; then at her centre's row, when it
  differs from her bottom edge's row; then at her top edge's row, when it differs from her bottom
  edge's row and the three rows' numbers combined by exclusive-or are not 0. So when the rows of
  her top edge and her centre are one, and her bottom edge's another, that cell is reported twice.
- checked: original only (area-brinstar's scrolling cells, item-morph-ball's pickup).

### OtherBlockKinds
- Mechanic. How the remaining kinds of cell meet her motion. Each is solid (it stops her as a
  solid block does, `BlocksStopFlush`), air (it never stops her), or decided by the cell's own
  reaction this frame. Each is reported as `TouchedCells` says, whatever it does.

| Kind | To her motion | Its own domain |
| --- | --- | --- |
| a door that leads to a room | air: the move goes on, and her passage to that room starts | doors |
| a door that leads nowhere (an elevator's) | solid | the elevator |
| spikes | solid | damage and knockback, which start no sooner than the next frame |
| air with spikes in it | air | damage, from the cells she is inside |
| a block her shots break, a block her grapple holds | solid | shots, grapple |
| air her shots act on, and the one other kind of plain air | air | shots |
| a block her bombs break | solid, unless its reaction lets her through this frame | bombs |
| air her bombs act on | air | bombs |
| a reacting block (one that crumbles, one only speed breaks, an item's) | solid or air, as its reaction answers this frame | those blocks, items |
| reacting air (a cell that changes the scrolling, a moving floor, an item) | air | scrolling, room pushes, items |
| a cell that stands for another | like the cell it names, in its own place; one that names nothing is air | the room's data |

- A cell that stands for another names a cell some number of cells away along its row or its
  column. It meets her exactly as that cell's kind and shape would if it were here: the stop and
  the slope's shape are at this cell's position. A chain of such cells is followed to its end.
- The world answers a reacting cell's solid-or-air for each touch; the answer may change from one
  touch to the next (a floor that crumbles holds her, then falls away).
- checked: original only.

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

- **Slopes:** the corners of steep blocks.
- **Liquids and the speed booster:** other speeds and gravity in water and lava; the booster's dash
  cap of 7 and its higher jump.
- **Pushes:** room pushes, quicksand, knockback, bomb jumps, grapple, shinespark.

## 11. Open questions

- With more than one solid enemy in a move's reach, which one decides is not yet specified (the
  vectors have one). It is not always the nearest, and one she already overlaps never decides
  (`SolidEnemies`).

- Whether a pose change the chart makes at an animation's end moves her the extra pixel of
  `AimingAndShooting`: open, listed in §7's scope.
- The elevator's descent: in area-brinstar she moves down 1 px and 2 px on alternate frames while
  she faces the screen, so the elevator carries her on top of `RidingTheElevator`'s 1 px. That
  carry belongs to the elevator's own lane; C-elevator checks her own 1 px alone.

1. "A spin she was already in two frames earlier" (§1.2) is not yet traced against a record: it
   may count pose changes rather than frames.
2. Answered: the pose chart's steps are §7's `pose_step` vectors. The motion vectors still change
   pose only by a jump, a fall or a turn from the ground.
3. `SolidEnemies`, `RidingTheElevator`, stepped blocks and most of `SlopesCarry` have constructed
   vectors only: no recorded frame is clean for them. The slope landings in r01 and samus-walljump-22 need lead-ins through a
   spin's drift and a sloped floor's effect on a move's length (`SlopesSlowTheWalk`).
4. The water rows, terrain-quicksand and samus-ceres-push gave no vectors: their liquid and push
   rules are out of scope.
5. Answered: landing on a solid enemy's top (item-morph-ball frame 797) clears the y subpixel
   because she reaches the enemy a pixel early and is touching it (`SolidEnemies`).
6. The crouch take-off's lift appears a frame apart from pose fit (item-morph-ball 1311). The
   order within a frame is `PoseFitAfterMoves`; a recorded vector for it waits for pose fit's own
   lane.
7. Answered: holding no button during a turn ends the skid; holding only buttons that change
   nothing ends it on the ground and keeps it in the air (`TurningRound`, R11, R12, R141).
8. `Moonfall` has M1 (r03), whose lead-in runs through the turn's end. Whether a turn's end into a
   straight jump's pose (§7) also starts one is not yet traced against a record.
9. 31 of the 149 recorded frames were dropped because no lead-in from a plain start reaches them
   through frames the spec covers (mostly after a wall jump's kick or a body change in the air).
10. Dropped after the first implementation: two frames that rise into a sloped ceiling
   (samus-walljump-28:62, samus-screw-attack:66; `SlopesCarry`), two from a bomb jump
   (samus-bomb-jump-vertical:1009, 1010) and two from a grapple release (samus-grapple-release:322,
   323), both out of scope (§10).
