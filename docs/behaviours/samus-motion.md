# Samus moving through the world: behaviour spec

What the player sees Samus do when she runs, stops, turns, crouches, rolls, jumps, spins, falls,
wall-jumps and lands, and how the world's blocks, slopes, solid enemies, floors, walls and
ceilings stop her. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-motion.toml`](../../scenarios/vectors/samus-motion.toml).

## Header

**Scope (Jim, 2026-11-07).** One domain, because the player sees one thing: Samus moving through
the world. It covers her speeds and position, how her pose follows from input and the world where
that changes her speeds, and her collision with blocks, slopes and solid enemies.

**Not in this spec yet** (each an input where it feeds us; §10): animation timing, liquids, the
speed booster's extra speed, knockback, bomb jumps, grapple, shinespark, pushes from the room
(Ceres, conveyors, quicksand), and what a cell she reaches then does (`TouchedCells` says which
she reaches; `OtherBlockKinds` how each kind meets her motion).

Size is counted in logical tokens: each identifier, keyword, literal and operator is one, and
so is each bracket pair; comments, layout and tests are free. The values of constants and statics
(tables of the game's data, such as the chart's rows) are counted apart as data and reported;
the target is on everything else, the logic.

**Baseline: about 15,100 tokens of logic** in the old code this domain replaces, tests excluded
(pose fit, now in the domain, included).

**Target: 7,300 tokens of logic** for the new model: motion, landings and turns about 2,000, the
pose chart's reader and effects about 1,500, collision with blocks and solid enemies about 1,500,
pose fit about 800, the shared types and slack 1,500. Over the target, the implementer keeps reshaping and reports
why the rest can't go, and the root reviews that report; over the baseline is a strict failure.

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
  travels against her new facing (the old way, unless she was already skidding, `TurningRound`). A
  turn from a standstill is a skid of zero: its first frame brakes, so she doesn't creep until the
  frame after (`TurningRound`).
- **Her dash.** The dash she builds running stays hers, through jumps and into the ball, until a
  halt takes it. Where a pose change halts her but leaves the dash's value on her, the next frame
  she moves the dash is gone.
- **The moonfall** (`Moonfall`).
- **A spin carried on.** A spin she was already in two frames earlier, or that followed a wall
  jump, can ready a wall jump (`WallJump`). Reversing needs no such memory: any spin can be
  reversed from its first frame, and the reversal always skids (`TurningRound`).

A vector starts at a moment where none of this is in play: her motion follows the pad unless she
is skidding, any dash she has is hers to keep, and a vertical speed of 0 means she stands on the ground,
except in a jump, a spin, a fall or a wall jump, where it means she has just started to fall.

### 1.3 What she reads from the world and the player (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| pad | held: run, jump, shoot, left, right, up, down, aim up, aim down; just pressed: jump | most behaviours |
| equipment | hi-jump, space jump: `bool` | `Jumping` |
| equipment seen only in the sprite | screw attack, spring ball, a suit other than the first (varia or gravity): `bool` | output (§2's last paragraph); spring ball also `AnimationEnds` |
| world | the room's blocks and solid enemies, answered as §8 says | §8 |
| spin turned far | `bool`: the spin's sprite has turned far enough to kick off a wall | `WallJump` |
| elevator moving | `bool` | `RidingTheElevator` |
| frame parity | `Even` or `Odd`: the game's frame count, alternating every frame | `BlocksStopFlush` |
| body size | each pose's half-width and half-height in px (`radius`): the game's data, for any pose, the one the step chooses included | §8, `PoseFitAfterMoves` |

---

## 2. Her pose as the player sees it

| Part | Values |
| --- | --- |
| body | `Upright`, `Crouched`, `Ball`, `SpringBall` |
| action | `AgainstWall` (pressed to the wall she ran into), `Crouching`, `Falling`, `Jumping` (a straight jump, or the spring ball's hop), `Landing` (the moment she touches down from a jump, a fall or a spin, before she stands still), `Moonwalking`, `Morphing`, `Rolling`, `Running`, `Spinning`, `StandingUp`, `Still`, `TakingOff`, `Turning` (on the ground or in the air), `Unmorphing`, `WallJumping` |
| facing | `Left`, `Right`, `Screen` (facing the player; counts as `Right` wherever a side is needed) |
| aim | the arm's direction: `Forward`, `Up`, `UpForward`, `DownForward`, `Down`; it changes only the sprite, except for `AimingAndShooting` |
| turn | for `Turning` only, the turn she is in: `Fall` (in a fall), `Ground` (on the ground, upright or crouched), `Jump` (in a take-off or a straight jump), `MoonwalkJump` (the moonwalk's jump, still on the ground). The player sees it as what she was doing when she turned; `AnimationEnds` hands over a pose by it, and the pad chart reads `MoonwalkJump` as a turn on the ground |
| landed from | for `Landing` only, what she touched down from: `Jump` (a straight jump or a fall, upright), `Spin` (a spin or a wall jump). The two look different, and one lasts longer than the other; both move her as standing still |
| gun | `Drawn` (her gun held out in front, a sprite of its own), `Stowed`; only upright `Running`, `Jumping` and `Falling` aimed `Forward`, and `Landing` from `Jump` aimed `Forward` (drawn firing), show it `Drawn`; every other pose has it `Stowed`. It changes only the sprite and the pad chart's rows (`PadChart` says what draws and stows it) |
| legs | `Striding` (drawn moving forward), `Together`; only an upright `Jumping` aimed `Forward` with the gun `Stowed` can be `Striding`, every other pose has them `Together`. It changes only the sprite (`PadChart` and `AnimationEnds` say which) |

The `turn`, `landed from`, `gun` and `legs` values are listed alphabetically; their order means
nothing.

**Forward** is the side she faces, **back** the other side; holding forward means holding the pad
towards her facing.

**Aim in a turn.** While `Turning` she keeps the aim she had just before. A turn in a jump or
a fall has one sprite for `DownForward` and `Down`, and it is `Down`: a turn begun aimed
`DownForward` in the air is aimed `Down`, and ends so (`AnimationEnds`). Crouched, the same sprite
is `DownForward`; no grounded pose aims `Down`.

**Her body while it changes shape** is the body she changes into: crouched while `Crouching` or
`Unmorphing`, a ball while `Morphing` (whether it is the spring ball is decided when the animation
ends, `AnimationEnds`), upright while `StandingUp`.

**What the sprite shows beyond these parts** follows from the inputs, and moves her the same:
- the spin (`Spinning`, and the spin of `WallJumping`) is drawn as the screw attack with screw
  attack, else as the space jump with space jump, else plain (in a liquid without the gravity suit
  it is always plain; liquids are out of scope, §10);
- facing the screen, she is drawn in her first suit, or in the other suit's sprite with varia or
  gravity;
- how far her animation has run (out of scope, §10).

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
  (invisible in vanilla). In any other moving activity a dash she is to lose (§1.2) is gone (0),
  and none is gained, taking off included (`Jumping`).
  Standing, against the wall, a wall stop, a turn's end, the falling ball letting go and a spin
  reversed each take it (0).
- checked: oracle (running vectors with run held; samus-mockball for the carry); original only
  (C-dashcap-a, C-dashcap-b).

### TurningRound
- Mechanic. Turning round, on the ground or in the air, or reversing a spin (a spin facing the
  other way: any spin, from its first frame on, §1.2), or the ball turning round, she adds her dash to her horizontal speed (dash 0) and keeps
  travelling the old way, now against her new facing: a skid. She brakes until the speed is spent,
  then builds the new way (turning's top speed is 0, so only the creep below). When the turn ends
  she loses her dash.
- Every turn and every spin reversed leaves her skidding, whatever she was doing before: her
  speed, the dash added, then points against her new facing. So a spin reversed while she is
  already skidding (travelling against her old facing, which is the way she now faces) turns her
  travel round: that frame she moves the old way, as `MovesAsShePosed` says, and from the next
  frame the same speed points the other way, back along her old facing, and goes on braking as a
  skid.
- Turning from a standstill slides her 0.C000 every other frame the new way: the speed goes 0,
  0.C000, 0, because turning's top speed is 0 and `WordwiseTopSpeed` keeps 0.C000 over 0 once.
- Turning with no button held at all, on the ground or in the air, her skid ends after that
  frame's move: she keeps her horizontal speed, which now follows the pad and points along her new
  facing. So a skid braked to exactly 0 ends there too. On the ground the same happens when she
  holds only buttons that bring no pose change (run, shoot or jump alone): no other row of the pad
  chart fits, and the chart's last row for turning on the ground ends the skid (`PadChart`). In the
  air, holding any such button, she keeps skidding: the speed still points the old way, and the skid ends only as
  `SpeedBuildsAndBrakes` says, when braking would take it below zero.
- Turning on the ground over a drop, she sinks by the ground pull (`FollowingTheGround`: her
  speed, subpixel part included, plus 1 px) a frame instead of falling. In the air she turns under
  gravity (`RisingAndFalling`).
- checked: oracle (samus-input-poses turns, samus-screw-attack; R133 for letting go, R11 and R12
  (samus-input-poses:120-121) for run held on the ground: braked to 0 at 120, she creeps 0.C000 at
  121, which a skid kept at 0 would not; R141 for run held in the air; SR1 for a spin reversed in
  its first frame, which skids; TL4 for the sinking onto a floor; W11 for the sinking over air,
  subpixel and all; W14 for a spin reversed while skidding); constructed (the sinking).

### AimingAndShooting
- Quirk and output. A frame whose pose is chosen as a running pose other than the one she is in
  moves her 1 px forward on top of that frame's move, without a speed change: running with another
  aim, but also from standing, from a landing, or from any pose whose pad chart row leads into
  running. Forward is the facing she had at the frame's start. The choice that counts is the one
  the pad chart and what her moves met make (`PoseFollows` items 2 and 3): a landing or a fall that
  replaces the pad's running pose brings no step, and an animation's end handing over a pose
  neither brings nor cancels it. A block or a solid enemy in the way of the step puts her against
  the wall instead (`WhatHerMovesMet`), so a standing start next to a wall with the pad asking to
  run leaves her against it. A wall stop that frame while running puts her against the wall too,
  without a step. Standing, unaimed and not landing, holding shoot restarts the standing
  animation.
- The step comes after that frame's moves and the ground pull. A solid enemy within its reach
  (`SolidEnemies`) stops it with no move at all; one she already touches still clears her y
  subpixel. A block in the way stops it as any horizontal move (`BlocksStopFlush`): by whole pixels
  up to the face, at most the 1 px, with her x subpixel written FFFF or 0, so a face less than a
  pixel ahead moves her that part of a pixel. The floor and ceiling checks of `SlopesCarry` don't
  run after the step, so it can leave her bottom edge in a sloped floor's solid part.
- checked: oracle (A1, the step; W12 standing into running, W17 and W18 a landing into
  running, W19 standing into running against a wall); constructed (C-aim-short, C-aim-enemy,
  C-aim-slope; the shot).

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
- The first poses of a straight jump (taking off) don't steer: she moves by her dash only and
  follows the pad; she gains no dash (only running does, `Dash`), and a dash she is to lose (§1.2)
  is gone (0) before that move, so taking off from a crouch she doesn't move at all and the jump
  that follows follows the pad. She doesn't move vertically and the pull doesn't act. The
  frame she changes into them she starts rising at the jump speed, without moving vertically that
  frame; from a crouch without aim she is also lifted 10 px.
- Starting a spin jump, or the spring ball's hop from the ground, she rises at the jump speed.
  Starting a straight jump or a fall she carries on along her facing if she has a dash, else she
  follows the pad, so a running jump keeps its drift.
- With space jump, pressing jump while falling in a spin jumps again, when her fall speed read as
  8.8 (its pixel part's low byte and subpixel's high byte) is from 2.80 up to, not including, 5.00.
  She rises at the new speed that same frame.
- checked: oracle (J1, J2, J3, and the take-off and spin vectors; W16, a take-off from a crouch
  with run held; the crouch lift open: question 6); constructed (space jump).

### FollowingTheGround
- Mechanic. On the ground she is pulled down after her horizontal move, by that frame's speed
  (her horizontal speed plus her dash, subpixel part included) plus 1 px, or by 1 px on a slope,
  and the floor stops her flush (§8). So 0.C000 pulls her 1.C000: with no floor in reach she
  moves all of it, her y subpixel included (02A9.0000 becomes 02AA.C000); on a floor the flush stop
  hides the subpixel part. Walking off a ledge the pull finds no floor: unless she is rising,
  turning or her body is changing shape, she falls from rest (vertical speed 0, falling, bounces
  0). checked: oracle (every ground vector; W11, a turn on the ground over air).

### MovesAsShePosed
- Timing. On the frame she changes pose she still moves as the pose at the frame's start says: what she is doing,
  her speeds, and what a landing or a fall does, come from the pose at the frame's start. checked:
  oracle (J1, J2, A1).

### PoseFitAfterMoves
- Timing. A frame ends in this order: her moves (horizontal, the ground pull or her vertical move,
  then the aim change's step), then what they met (a landing, a fall, a ceiling, a wall), then the
  pose change. Within the pose change, pose fit's push comes first: when the new pose's body is
  taller, she is pushed clear of what is above or below her from the position her moves left
  (the rules below). Then come the new pose's speeds (a jump's rise, `Jumping`), then the crouch
  take-off's 10 px lift, from the pushed position.
- So the push sees her before the lift: her y at the frame's end is the moves' y, plus the push
  found there, less the 10 px. A push found after the lift would differ wherever a block is within
  10 px above her.
- Pose fit is part of the frame's step: it follows from the pose the step chose, the pose she had,
  their bodies' sizes (§1.3) and the world. It runs only when the chosen pose differs from the one
  she had, doesn't face the screen, and has a body taller by g px (half-height against
  half-height); otherwise there is no push.
- Mechanic. From where her moves left her, at her old size, she looks g px up and g px down: each
  look is a vertical test move of g whole px, her subpixel as it is, stopped as a move is, solid
  enemies first (`SolidEnemies`), then blocks (`BlocksStopFlush`, its `Even`/`Odd` rule included);
  against blocks, a look of 8 px or more is tested at 8 px first, and the first test stopped
  decides. The room on a side is how far its look got: g where nothing stopped it. The cells the
  looks reach are their own, not `TouchedCells`' reports, and no cell's reaction
  (`OtherBlockKinds`) starts from them.
  - Nothing in reach on either side: no push; she grows about her centre.
  - Something on one side only: she is pushed away from it by g less the room on that side, in
    whole px, her subpixel kept: up from what is below her (on a floor with nothing above, her feet
    stay where they were: a body that grows by n px raises her centre by n px), down from what is
    above her. The far side is looked at once more for that distance (blocks at her old size; for
    a push from a solid enemy, solid enemies at her new size); if that look is stopped, the change
    is refused.
  - Something above her and something below her (blocks or solid enemies, in any mix): with an old body under 8 px
    half-height (the ball) the change is refused. Otherwise she takes crouched `Still` instead, aim
    `Forward`, facing as before the change; if her old body was shorter than the crouch, she rises
    by the difference, her feet staying where they were, and there is no other push. The crouch
    has no take-off: no rise and no 10 px lift.
  - A refused change leaves her in the pose she had: the frame ends in it, and the chosen pose's
    speeds (`Jumping`) don't come.
- checked: oracle (PF1, item-morph-ball:1311: crouched to the take-off, the body grows 3 px and
  she rises 3 px, then the 10 px lift, in one frame; U1, crouched to standing, 5 px; PF2, a
  landing from a jump, 2 px; PF3, a landing from a spin, 9 px; the push up from a floor in every
  recorded landing on a flat floor; area-norfair:38, a body growing 9 px in open air, no push);
  constructed (C-fit-ceiling, pushed down from a ceiling; C-fit-tunnel, a take-off between a floor
  and a ceiling crouches instead; C-fit-ball-tunnel, unmorphing there is
  refused); original only (the second look's refusal, solid enemies in pose fit, and what else a
  refused change leaves undone).

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
  wall on her right; otherwise right held: on her left) readies the wall jump. Readying is not a
  part of her pose (§2): the pose stays the spin it was. Only the spin's animation shows it,
  skipping ahead to the point where the spin has turned far enough, so the input "spin turned far"
  (§1.3) holds from the next frame. How far an animation has run is out of scope (§10), so the
  step reports the readying for the animation to show.
- Output. Each frame reports one of three outcomes of the near-wall test: readied (a wall near her
  while the spin hasn't turned far: the spin's animation skips ahead), kicked (the kick below, also
  `WhatHerMovesMet`'s wall jump), or neither (no test that frame, no wall found, or a wall near
  her once the spin has turned far without jump pressed). A wall found by its block test and one
  found by its solid enemy test ready her alike.
- A wall is near her when a test move of 8 px towards it, from where her x is after this frame's
  move, would be stopped (`BlocksStopFlush`, `SolidEnemies`): fewer than 8 px of air between her
  edge and the wall. Readying and the kick use this same test.
- The test needs a spin carried on and a side held, and runs on every frame that has both, looking
  away from the side held as above: on a frame that readies, on every frame after the spin has
  turned far whether or not she presses jump, and on a frame that finds no wall. Without a spin
  carried on, or holding no side, there is no test: nothing is readied, nothing
  is written and no kick comes.
- Once the spin has turned far enough, still in a spin carried on and holding away from a wall
  near her, pressing jump kicks her off:
  she doesn't move vertically that frame; she starts the wall jump rising at the wall-jump speed, x
  speed 0, following the pad.
- Each frame the test runs, readying frames included, finding a block wall writes her x subpixel
  as a stop against it would (FFFF looking right, 0 looking left), without moving her. The test
  meets solid enemies first. Finding a solid enemy instead, it writes her x nothing: an enemy she
  already touches on that side clears her y subpixel to 0, as any test against one does
  (`SolidEnemies`), and one ahead of her writes nothing.
- The test reaches cells as a move of 8 px would (`TouchedCells`), and cells that react when she
  reaches them react to it: in item-morph-ball the test reaches an item's cell 8 px past her edge
  and she picks the item up.
- checked: oracle (samus-walljump-16, -22, -28, samus-grapple-walljump, item-morph-ball's leftward
  spin jumps; samus-walljump-16:46-47: readied in one frame, kicked off in the next; RW1: the
  readying frame writes her x subpixel 0 against the wall on her left, and so does the frame
  before, which reverses the spin, and the frame reports the readying; W15, the test's cells
  reached); original only (no test without a spin carried on or without a
  side held: no vector checks a spin near a wall lacking either).

### Landing
- Mechanic. Falling onto a floor upright she lands: vertical speed 0, bounces 0, horizontal speed 0, following the
  pad; the dash keeps its value.
  Landing while her body changes shape stops only her vertical motion (vertical speed 0); both
  speeds, and whether she carries on, are kept.
- Landing while turning (any `Turning`: in a jump, in a fall, or on the ground as she sinks over a
  drop) stops only her vertical motion: the floor stops her move flush (`BlocksStopFlush`), and
  nothing else changes. Unlike the change of shape, it leaves her vertical speed as it is: in a
  turn in a jump or a fall the speed goes on growing by the pull each frame as in the air
  (`RisingAndFalling`) while the floor holds her, until the turn ends (`AnimationEnds`); then the
  next downward move that meets the floor lands her as above. Her horizontal motion goes on as
  `TurningRound` says.
- The ball bounces twice before it rests: landing at 3 px or more with no bounce yet, it rises at
  the bounce speed (bounces 1); after bounce 1 it rises at the bounce speed less 1 px (bounces 2);
  after bounce 2, or slower than 3 px, it lands as above. The spring ball landing with jump held
  hops again at the jump speed instead, bounces 0.
- checked: oracle (r06 for the changing body, LB1, LB2; TL1-TL4 for a turn: TL1 a turn in a fall
  meets the floor, TL2 the next frame on it, its vertical speed still growing, TL3 a turn in a
  jump, TL4 the ground turn sinking; r02:37-38 and r07:263-264, where the turn's end hands over a
  fall or a jump and the next frame lands her); constructed (C-bounce).

---

## 7. Pose changes: the pose chart

Which pose she shows next, from the pad, her equipment and what her moves met, and what the change
does to her speeds. Written as data: a few tables keyed by what she is doing and what the player
holds or has just pressed. Vectors: the `[[pose_step]]` entries of the vector file.

**Size.** Inside the header's 1,500 tokens of logic for the chart: the chart's reader and the
effects. The tables below are data records (about 120 rows), counted as data.

### PoseFollows
- Mechanic and timing. Her pose changes at most once a frame. What decides, the first that applies:
  1. an animation ending hands over its pose (`AnimationEnds`);
  2. what her moves met (`WhatHerMovesMet`, which says which wins when several apply);
  3. the pad chart (`PadChart`), from the pose and the pad at the frame's start; its last row
     for her situation covers nothing fitting.
- The frame runs in this order, and this is what the player can tell apart at its end: she moves as
  the pose at the frame's start says (`MovesAsShePosed`); what those moves met is known; the pose
  is chosen as above; then the change's effects act on her speeds and dash. So a change's effects
  never alter that frame's moves, and the frame ends in the pose chosen, with its effects applied.
- One that comes lower in the list loses its pose and its effects to a higher one: a landing
  replaces the pad chart's outcome and its effect, and an animation's end replaces a landing's pose
  (the landing still stops her as `Landing` says).
- Where a change stops her horizontal speed dead, it takes her dash with it (dash 0).
- checked: oracle (the `[[pose_step]]` vectors).

### PadChart
- Mechanic. A row fits when every button it names as just pressed was just pressed this
  frame and every button it names as held is held; other buttons don't matter. Where several
  rows of her situation fit: a row that needs a just-pressed button beats one that doesn't,
  and between two rows of the same kind, the one that names more buttons wins. Where that
  leaves two rows tied, or where her situation goes against it, the Pad cell says which wins
  ("beats ..."). The rows' order in the table carries no meaning. "Aim" in a next pose means
  the aim the `Aiming` rule gives. "kept" means her pose stays as it is, apart from an aim
  the cell names; the Effect cell says whatever else happens, and a fitting row that keeps her
  pose leaves the last row unused.
- Forward and back are by her facing at the frame's start (a turn already faces its new way).
  The chart covers pads with at most one of left and right, and at most one of up and down, held:
  the pad can't hold both, and no record does.
- The gun. A next pose has the gun `Stowed` unless its cell says "gun drawn" (she draws it) or
  "gun as it was" (drawn if it was drawn); "kept" keeps it as it was. So she draws it by holding
  shoot while running forward, jumping, falling or wall jumping, or by pressing shoot in a spin,
  where the ranks make that row the winner; any other outcome stows it: an aim, a turn, holding
  forward in a jump, a landing (except one that lands firing, `WhatHerMovesMet`), walking off a
  ledge, the end of an animation (a take-off ends in a jump with the gun stowed, whatever she
  holds). A Pad cell that starts "gun stowed:" fits only while her gun is stowed.
- The legs. A next pose has its legs `Together` unless its cell says "striding"; "kept" keeps them
  as they were. So she strides by holding forward in a take-off or a straight jump, and holding
  jump alone, or shoot, brings her legs together again.
- Rows are listed by rank (just pressed first, then by how many buttons they name), and inside a
  rank by next pose (action, then the first aim the cell names, alphabetically; "kept" last);
  where two rows still tie, the one that beats the other comes first. Each situation's last row
  is what happens when no other row fits: its pose and its effect are her let-go outcome.

| She is | Pad | Next pose | Effect |
| --- | --- | --- | --- |
| standing still, or landing | down pressed | crouched `Crouching`, crouch aim | none |
| | jump pressed; beats down pressed | upright `TakingOff`, take-off aim | `Jumping`'s take-off |
| | back, shoot and an aim button held; beats back and both aim buttons held | `Moonwalking` facing back, `DownForward` with aim down, else `UpForward` | none |
| | back and both aim buttons held | `Turning` | `TurningRound` |
| | back and shoot held | `Moonwalking` facing back, `Forward` | none |
| | forward and an aim button held; beats forward with up or down held | `Running`, `UpForward` with aim up, else `DownForward` | none |
| | forward with up or down held | `Running`, `UpForward` with up, else `DownForward` | none |
| | both aim buttons held; beats every other row naming two buttons | `Still`, `Up` | none |
| | forward held | `Running`, `Forward` | none |
| | up held; beats aim up or aim down held | `Still`, `Up` | none |
| | aim up or aim down held | `Still`, `UpForward` with aim up, else `DownForward` | none |
| | back held; beats up held and aim up or aim down held | `Turning` | `TurningRound` |
| | anything else, or nothing held | kept, aim `Forward`; landing, kept as she is | she follows the pad again (a skid ends) |
| running | down pressed; beats jump pressed | crouched `Crouching`, `Forward` | none |
| | jump pressed | `Spinning` | `Jumping`'s spin start |
| | forward and shoot held | `Running`, `Forward`, gun drawn | none |
| | forward and an aim button held; beats forward with up or down held, and forward and shoot held | `Running`, `UpForward` with aim up, else `DownForward` | none |
| | forward with up or down held; beats forward and shoot held | `Running`, `UpForward` with up, else `DownForward` | none |
| | forward held | `Running`, `Forward`, gun as it was | none |
| | up held; beats aim up or aim down held | upright `Still`, `Up` | none |
| | aim up or aim down held | upright `Still`, `UpForward` with aim up, else `DownForward` | none |
| | back held; beats up held and aim up or aim down held | `Turning` | `TurningRound` |
| | anything else, or nothing held | upright `Still`, `Forward` | the dash joins her horizontal speed (dash 0) and she carries on, braking (`SpeedBuildsAndBrakes`); with a horizontal speed of 0 she only follows the pad again |
| taking off | jump and forward held | `Jumping`, `Forward`, striding | none |
| | jump with up, down or an aim button held; beats jump and forward held | `Jumping`, aim `Up` with up, else `Down` with down, else `UpForward` with aim up, else `DownForward` | none |
| | jump and back held; beats jump with up, down or an aim button held | `Turning` in a jump | `TurningRound` |
| | shoot held | `Jumping`, `Forward`, gun drawn | none |
| | anything else, or nothing held (jump alone too) | kept | as running |
| in a straight jump | aiming `Down`: down pressed | ball `Morphing` | none |
| | aiming `Down`, she faces `Right`: forward, jump and shoot held; forward with up, down or an aim button held beats it | `Jumping`, `Forward`, gun drawn | none |
| | forward with up, down or an aim button held | `Jumping`, air aim | none |
| | forward held; beats shoot held and jump held | `Jumping`, `Forward`, striding | none |
| | shoot held; beats jump held | `Jumping`, `Forward`, gun drawn | none |
| | gun stowed: jump held | `Jumping`, `Forward` | none |
| | up, down or an aim button held; beats forward held, shoot held and jump held | `Jumping`, air aim | none |
| | back held; beats every other row naming one button | `Turning` in a jump | `TurningRound` |
| | aiming `Down`: jump held, and no other button a row names; beats jump held | kept | none |
| | anything else, or nothing held (with the gun drawn, jump alone too) | kept | as running |
| spinning | shoot pressed | `Jumping`, `Forward`, gun drawn | none |
| | shoot with up, down or an aim button held; beats forward and jump held | `Jumping`, aim `Up` with up, else `Down` with down, else `UpForward` with aim up, else `DownForward` | none |
| | forward and jump held | kept | none |
| | up, down or an aim button held; beats forward held and back held | `Jumping`, aim `Up` with up, else `UpForward` with aim up, else `DownForward` with aim down, else `Down` | none |
| | back held | `Spinning`, facing back: the spin reversed | `TurningRound` |
| | forward held | kept | none |
| | anything else, or nothing held | kept | none |
| falling (upright) | aiming `Down`: down pressed | ball `Morphing` | none |
| | forward with up or down held | `Falling`, `UpForward` with up, else `DownForward` | none |
| | shoot held; beats forward held | `Falling`, `Forward`, gun drawn | none |
| | forward held | `Falling`, `Forward`, gun as it was | none |
| | up, down or an aim button held; beats shoot held and forward held | `Falling`, air aim | none |
| | back held; beats shoot held, forward held, and up, down or an aim button held; aiming `Down` or with the gun drawn, up or down held beats it | `Turning` in a fall | `TurningRound` |
| | anything else, or nothing held | kept, aim `Forward` | dash 0 |
| crouched | down pressed; beats jump pressed | ball `Morphing` | none |
| | up pressed; beats every other row naming a pressed button | `StandingUp`, crouch aim | none |
| | jump pressed | upright `TakingOff`, `Forward` | `Jumping`'s take-off |
| | back pressed; beats down pressed and jump pressed | crouched `Turning` | none |
| | forward and an aim button held | upright `Still`, `Forward` | she stands at once; her feet stay where they were, 5 px up on a floor, by pose fit's push (`PoseFitAfterMoves`), not an effect of its own |
| | both aim buttons held; beats forward and an aim button held | crouched `Still`, `Up` | none |
| | forward held | upright `Still`, `Forward` | she stands at once; her feet stay where they were, 5 px up on a floor, by pose fit's push (`PoseFitAfterMoves`), not an effect of its own |
| | aim up or aim down held | crouched `Still`, `UpForward` with aim up, else `DownForward` | none |
| | anything else, or nothing held | kept, aim `Forward` | she follows the pad again (a skid ends) |
| a ball, on the ground or in the air | jump pressed, with the spring ball, on the ground | the ball off the ground, rising | `Jumping`'s spring-ball hop |
| | up pressed, or without the spring ball jump pressed; beats the spring ball's hop | crouched `Unmorphing` | none |
| | left or right held | the same ball facing that side, `Rolling` on the ground | none |
| | anything else, or nothing held: on the ground | ball `Still` | horizontal speed 0, dash 0, following the pad |
| | in the air, without the spring ball | kept | as running |
| | off the ground, with the spring ball | kept | horizontal speed 0, dash 0, following the pad |
| turning on the ground or from the moonwalk's jump (not aimed `UpForward`) | aimed `Up` or `DownForward`: forward held and jump pressed | `Spinning`, the new facing | `Jumping`'s spin start |
| | jump pressed | upright `TakingOff`, the new facing | `Jumping`'s take-off |
| | aimed `Forward`: forward and jump held; beats jump pressed | `Spinning`, the new facing | `Jumping`'s spin start |
| | forward held | kept | none |
| | anything else, or nothing held | kept | she follows the pad again (`TurningRound`) |
| turning in a jump or a fall, crouched turning, or turning on the ground or from the moonwalk's jump aimed `UpForward` | any button held | kept | none: she keeps turning (`TurningRound`) |
| | nothing held | kept | she follows the pad again (`TurningRound`) |
| changing body: `Crouching`, `StandingUp`, `Morphing`, `Unmorphing` | anything, or nothing held | kept | none: the pad has no say until the animation ends |
| moonwalking | down pressed; beats jump pressed | crouched `Crouching`, facing the way she looks, `Forward` | none |
| | jump pressed | `Turning` from the moonwalk's jump, aim kept, then a spin the way she walks (`AnimationEnds`) | `TurningRound` |
| | the way she walks, shoot and an aim button held | `Moonwalking`, `UpForward` with aim up, else `DownForward`; with both aim buttons held, `UpForward` facing left and `DownForward` facing right | none |
| | the way she walks and shoot held | `Moonwalking`, `Forward` | none |
| | the way she looks held | `Running` that way, `Forward` | none |
| | the way she walks held | `Turning` | `TurningRound` |
| | anything else, or nothing held | upright `Still`, facing the way she looks | she follows the pad again |
| wall jumping | down pressed | ball `Morphing` | none |
| | shoot held; beats jump held | `Jumping`, `Forward`, gun drawn | none |
| | aim up or aim down held; beats shoot held and jump held | `Jumping`, `UpForward` with aim up, else `DownForward` | none |
| | back held; beats every other row naming one button | `Spinning`, facing back | `TurningRound` |
| | jump held | kept | none |
| | anything else, or nothing held | `Spinning` | horizontal speed 0, dash 0, following the pad |
| against the wall | down pressed | crouched `Crouching`, `Forward` | none |
| | jump pressed; beats down pressed | upright `TakingOff`, `Forward` | `Jumping`'s take-off |
| | back and an aim button held | `Moonwalking` facing back, `DownForward` with aim down, else `UpForward` | none |
| | forward with up or down held; beats down pressed | `Running`, `UpForward` with up, else `DownForward` | none |
| | forward held | `Running`, `Forward` | none |
| | up held; beats aim up or aim down held, and back held | upright `Still`, `Up` | none |
| | aim up or aim down held; beats forward held and back held | upright `Still`, `UpForward` with aim up, else `DownForward` | none |
| | back held | `Turning` | `TurningRound` |
| | anything else, or nothing held | kept, aim `Forward` | she follows the pad again (a skid ends) |

- In a straight jump aimed `Down`, the row for forward, jump and shoot held fits only when she
  faces `Right`, and nothing takes its place the other way: there those three held are decided as
  in any straight jump, forward held winning (`Jumping`, `Forward`, striding; C-chart-j against
  C-chart-i).
- A `Turning` cell gives the turn she is in (§2): `Ground` unless it says "in a jump", "in a
  fall" or "from the moonwalk's jump". Facing is kept unless the row says otherwise; `Turning`
  faces back (the turn shows her new facing, and `TurningRound` turns her), and keeps the aim she
  had (§2, aim in a turn). A moonwalk faces the
  way she walks, which is backwards: her sprite looks the other way.
- Body follows the action: `Running`, `Moonwalking`, `TakingOff`, `Jumping`, `Spinning`, `Falling`,
  `WallJumping`, `AgainstWall`, `Landing` and `StandingUp` are upright; `Crouching` and `Unmorphing`
  are crouched; `Morphing` is a ball (§2).
- A moonwalk's jump turns with the moonwalk's own aim, whatever aim button is held.
- The let-go outcomes "as running" mean: the dash joins her horizontal speed (dash 0) and she
  carries on, braking (`SpeedBuildsAndBrakes`); with a horizontal speed of 0 she only follows the
  pad again.
- checked: oracle (P1, P3, P13 to P18, P20 to P22, P24, P25, G1 to G9, L1, L2, U1, and R133 and
  R141 of `TurningRound`); constructed (C-chart-a to C-chart-j).

### Aiming
- Mechanic. The aim a next pose takes from the held buttons, by the rule its row names:

| Rule | Both aim buttons | Up | Down | Aim up | Aim down | Forward and up / down | Else |
| --- | --- | --- | --- | --- | --- | --- | --- |
| take-off | (as aim up) | `Up` | `Forward` | `UpForward` | `DownForward` | as up / `Forward` | `Forward` |
| standing | `Up` | `Up` | `Forward` | `UpForward` | `DownForward` | (running rows) | `Forward` |
| crouch | `Up` | (stands) | `Forward` | `UpForward` | `DownForward` | (stands) | `Forward` |
| air | `UpForward` | `Up` | `Down` | `UpForward` | `DownForward` | `UpForward` / `DownForward` | `Forward` |

- Where up and an aim button are both held, up wins; in the air rule down wins over an aim button
  too; where up or down and forward are held, the forward diagonal wins. checked: oracle (P2, P9,
  P24).

### WhatHerMovesMet
- Mechanic. What this frame's moves met chooses her pose over the pad:

| Her moves met | She was | Next pose | Effect |
| --- | --- | --- | --- |
| a wall (a wall stop) | running | `AgainstWall`, facing kept; aim `UpForward` or `DownForward` kept, else `Forward` (`Up` stays `Still`, `Up`) | `WallsAndCeilings` |
| a block or solid enemy in the way of the aim change's step | in any pose, a running pose other than hers chosen (`AimingAndShooting`) | `AgainstWall`, facing kept, with the aim of the running pose chosen | `AimingAndShooting` |
| a wall near her as she presses jump (`WallJump`) | spinning | `WallJumping`, facing kept | `WallJump`'s kick |
| a floor (landing) | upright, in a straight jump or falling | `Landing` from `Jump`, facing kept, aim kept (`Down` becomes `Forward`); aimed `Forward`, not `Down`, with shoot held, gun drawn (she lands firing) | `Landing` |
| | spinning or wall jumping | `Landing` from `Spin`, facing kept, `Forward` | `Landing` |
| | a ball, bouncing (`Landing`) | kept | `Landing`'s bounce |
| | a ball, coming to rest | ball `Still`, facing kept | `Landing` |
| | changing body: `Crouching`, `StandingUp`, `Morphing` or `Unmorphing` | kept: the ball rows above are not for `Morphing`, and the pad has no say (`PadChart`) | `Landing`: only her vertical motion stops |
| | turning: any `Turning`, upright or crouched (in a jump, in a fall, or on the ground as she sinks over a drop) | kept: the upright and spinning rows above are not for a turn, and the landing adds no pad row; the turn's own rows apply as on any frame of it (`PadChart`) | `Landing`: only her vertical motion stops; her vertical speed is kept |
| no floor (walking off a ledge, `FollowingTheGround`) | upright on the ground | upright `Falling`, facing kept, `Forward` | `FollowingTheGround`'s fall |
| | a ball | ball `Falling`, facing kept | `FollowingTheGround`'s fall |
| a ceiling | rising | kept | `WallsAndCeilings` |

- Where two of these meet in one frame: a wall stop while running wins over walking off a ledge.
  The aim change's step comes only when the pose chosen is a running one, so a landing or walking
  off a ledge, which choose a pose of their own, leave no step to block. The wall-jump kick, a
  landing, walking off a ledge and a ceiling each come from her one vertical move or her kick, so
  no two of them meet her in the same frame.
- A landing while her body changes shape leaves the change to run on: its animation's end hands over
  the next pose (`AnimationEnds`, on the ground now), and when that end comes in the landing's own
  frame it decides the pose, as `PoseFollows` says. A landing while turning leaves the turn to run
  on the same way: its end hands over what `AnimationEnds` names for the turn, and a fall or jump
  handed over lands when a downward move next meets the floor (`Landing`).
- checked: oracle (P4, P5, P6, P7, P8; LB1 landing while `Morphing`, LB2 while `Unmorphing`;
  TP1 landing in a turn in a fall, TP2 in the ground turn as it sinks, TP3 in a turn in a jump;
  item-morph-ball:1264 and r06:16, where the landing and the animation's end share a frame);
  original only (`Crouching` and `StandingUp`, and the crouched and moonwalk turns: she never
  lands in them in a record).

### AnimationEnds
- Mechanic. Some poses last only as long as their animation, and its end hands over the next pose
  whatever the pad (animation timing is out of scope, §10: a vector says on which frame it ends):

| Ending pose | Next pose | Effect |
| --- | --- | --- |
| `TakingOff` | `Jumping`, aim kept | none: her rise speed came with the take-off (`Jumping`) |
| `Turning` on the ground (turn `Ground`) | upright `Still`, the new facing, aim kept | dash 0 (`Dash`) |
| `Turning` in a jump (turn `Jump`) | `Jumping`, the new facing, aim kept; aimed `Forward`, striding | dash 0 |
| `Turning` in a fall (turn `Fall`) | `Falling`, the new facing, aim kept | dash 0 |
| crouched `Turning` | crouched `Still`, the new facing, aim kept | dash 0 |
| `Turning` from the moonwalk's jump (turn `MoonwalkJump`) | `Spinning`, the new facing | `Moonfall` |
| `Crouching` | crouched `Still` | none |
| `StandingUp` | upright `Still` | none |
| `Morphing` | ball `Still` on the ground, ball `Falling` in the air; with the spring ball, the spring ball's | none |
| `Landing` | upright `Still`, aim kept, gun stowed | none |
| `Unmorphing` | crouched `Still` on the ground, upright `Falling` in the air | none |

- checked: oracle (P10, P11, P12, P19, P23, M1).

### Out of the chart's scope
- The spring ball's rows, the grapple, knockback, the boost after damage, x-ray, shinespark, the
  crystal flash, the drain, liquids, and equipment that changes mid-pose.

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
  side: the enemy's near side less her near side, the sides being the boxes' edges at centre ±
  half-size, so boxes whose pixels are next to each other have a gap of 0 (downward, her bottom
  side `y + ry` against the enemy's top side `ey − ery`; rightward, her right side `x + rx`
  against its left side `ex − erx`). A gap of 0 is touching: she doesn't move, and her **y**
  subpixel is set to 0, on either axis; a larger gap stops her by that many whole pixels, her
  subpixel kept; a negative one (already overlapping) never stops her: the move goes as if that
  enemy were not there, so another solid enemy in its path decides it, or with none, blocks do
  (`BlocksStopFlush`; C-enemy-overlap). So a move of 0 is never shortened by one.
- Standing on a solid enemy, with no gap between them, each frame's ground pull reaches it and
  stops at once, so she stays on it (a landing, `Landing`), her y subpixel 0. A landing on one stops her
  with her subpixel kept, and when the landing pose's taller body is then fitted
  (`PoseFitAfterMoves`), its look down is a test like any move's, so it finds no gap below her,
  and her y subpixel becomes 0 before the push.
- A stop against an enemy is a wall, a ceiling or a floor as the move's direction says: sideways it
  halts her (`WallsAndCeilings`), upward it ends the rise, downward she lands (`Landing`).
- A solid enemy in a move's path decides that move alone: she stops against it and no block
  stops her, even a nearer one, so the move can end with her edge overlapping a block. Blocks
  stop her only on a path free of solid enemies (`BlocksStopFlush`).
- checked: oracle (W13, a landing on one 2 px below and the fit's look touching it; W20,
  running on one); constructed (C-enemy-ahead, C-enemy-adjacent, C-enemy-down, C-enemy-overlap,
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
- **An empty column.** A column of a sloped surface can hold none of the surface: the whole
  column is air in that block. It counts as a surface just past the block's far edge:
  - as a floor, the last air row above it is the block's bottom row. A downward move whose bottom
    edge at the target is on that row stops and lands her there, as above; one that ends higher
    passes. The check after a horizontal move never lifts her.
  - as a ceiling, the first air row under it is the block's top row. A rise whose top edge at the
    target is on that row doesn't move her and ends the rise, as above; one whose top edge ends
    on any lower row of the block moves on. The check after a horizontal move, with her top
    edge on that row, pushes her 0 px and seats her.
  - The vectors write such a column as 16, for a floor and a ceiling alike.
- **A column far past.** A column's surface can also lie more than one row past the block's far
  edge. Such a column is air in that block, as an empty one is, but it never meets her, in either
  direction: as a floor it never stops a downward move, whatever row of the block her bottom edge
  reaches, and the check after a horizontal move never lifts her; as a ceiling it never stops a
  rise, whatever row of the block her top edge reaches, and the check after a horizontal move
  never pushes her. It never seats her, so that frame's pull is her speed plus 1 px, unless something else
  seated her. A sloped
  shape that rooms use has such columns across half its width. The vectors write such a column as
  17, for a floor and a ceiling alike.
- checked: oracle (samus-walljump-16/22/28 frame 44; samus-walljump-28:62 and
  samus-screw-attack:66 for the sloped ceiling; chain r01:677, samus-walljump-22:93 and
  samus-input-poses:426 for the landing that keeps the subpixel); constructed (C-slope-land,
  C-slope-edge, C-slope-ceiling-edge, C-seat-lift, C-seat-none, C-step-down, C-down-odd,
  C-aim-slope, and for the empty column C-slope-ceiling-empty, C-slope-ceiling-empty-on,
  C-slope-floor-empty, and for the column far past C-slope-floor-far, C-slope-ceiling-far).

### SlopesSlowTheWalk
- Mechanic. A horizontal move made while her vertical speed is exactly 0 (standing, running,
  turning, rolling, the first frame of a fall from rest, or after a rise ends at the frame's
  start), whose leading edge's column meets a sloped floor, is shortened by that floor's walk
  factor. A sloped ceiling never shortens a move, and with any vertical speed but 0 no slope does.
- The cells that count are those `TouchedCells` names for the move: the leading edge's column at
  the move's unshortened target, in each row her body spans. Each sloped floor among them
  shortens it again, top row first. A solid block below it in that column still stops her as
  `BlocksStopFlush` says.
- The walk factor is world data, a multiple of 1/256 for each sloped floor: one of 1.00, 0.F0,
  0.D8, 0.C0, 0.B0, 0.80 and 0.50. Vectors give it
  as `world.slope_floor.factor`.
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
- **The cells her moves reach.** For each move of the frame (the horizontal move, the near-wall
  test of `WallJump`, the ground pull or her vertical move, the aim change's step, the elevator's
  1 px), in the order they happen: the
  cells of her leading edge's line at the move's target, the line `BlocksStopFlush` names. For a
  horizontal move that is the column of her leading edge at the target, in each row her body
  spans where she stands before the move, top row first. For a vertical move it is the row of her
  leading edge at the target, in each column her body spans where she stands before the move, left
  to right on an `Even` frame and right to left on an `Odd` one. Each cell comes with the move's
  direction, one of four ways: down, left, right or up (listed alphabetically; the order means
  nothing), because some reactions depend on it.
- The near-wall test is a look, not a move: its cells are those of a horizontal move of 8 px
  towards the wall from where her horizontal move left her, and they come with the way `Look`
  instead of a direction. A reaction that depends on the way she moves (a floor that crumbles, a
  station she enters from one side) doesn't answer a look; the others (an item, a cell that
  changes the scrolling) do, as for a move. The world answers a reacting cell's solid-or-air for
  a look as for a move (`OtherBlockKinds`).
- The report ends at the first cell that stops her: the cells after it on the line are not
  reached. A move of exactly 0 reaches no cells (`BlocksStopFlush`), and neither does a move or a
  look that a solid enemy decides (`SolidEnemies`). Pose fit's looks (`PoseFitAfterMoves`) reach
  cells of their own, which belong to them, not to these reports.
- A cell that stands for another (`OtherBlockKinds`) is reported as the cell it stands for.
- **The cells she is inside.** Once each frame, before her moves, from her position at the frame's
  start: the cell in her centre column at her bottom edge's row; then at her centre's row, when it
  differs from her bottom edge's row; then at her top edge's row, when it differs from her bottom
  edge's row and the three rows' numbers combined by exclusive-or are not 0. So when the rows of
  her top edge and her centre are one, and her bottom edge's another, that cell is reported twice.
- checked: oracle (W15: the near-wall test reaches item-morph-ball's item cell, and the game picks
  the item up); original only (area-brinstar's scrolling cells); constructed
  (C-touch-run, C-touch-wall, C-touch-fall-odd, C-touch-inside-twice, and the C-kind vectors).

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
| air her shots act on | air | shots |
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
- checked: original only; constructed (C-kind-spikes, C-kind-door, C-kind-reacting-air,
  C-kind-reacting-solid, C-kind-stand-in).

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
- **Animation timing:** how far an animation has run and on which frame it ends (a vector says,
  `AnimationEnds`); its frames are drawn by the sprite, and readying a wall jump shows only there
  (`WallJump`).

## 11. Open questions

- With more than one solid enemy in a move's reach, which one decides is not yet specified (the
  vectors have one). It is not always the nearest, and one she already overlaps never decides
  (`SolidEnemies`).

- Answered: an animation's end neither brings nor cancels the extra pixel of
  `AimingAndShooting`; the step follows the pose the pad chart and her moves chose.
- The elevator's descent: in area-brinstar she moves down 1 px and 2 px on alternate frames while
  she faces the screen, so the elevator carries her on top of `RidingTheElevator`'s 1 px. That
  carry belongs to the elevator's own lane; C-elevator checks her own 1 px alone.

1. "A spin she was already in two frames earlier" (§1.2) is not yet traced against a record: it
   may count pose changes rather than frames.
2. Answered: the pose chart's steps are §7's `pose_step` vectors. The motion vectors change pose
   by a jump, a fall, a turn from the ground or a spin reversed, and, for pose fit, by a landing,
   a take-off, standing up and unmorphing (PF1-PF3, U1, the C-fit vectors).
3. `RidingTheElevator`, stepped blocks and most of `SlopesCarry` have constructed vectors only:
   no recorded frame is clean for them (`SolidEnemies` now has W13 and W20). The slope landings in r01 and samus-walljump-22 need lead-ins through a
   spin's drift and a sloped floor's effect on a move's length (`SlopesSlowTheWalk`).
4. The water rows, terrain-quicksand and samus-ceres-push gave no vectors: their liquid and push
   rules are out of scope.
5. Answered again: landing on a solid enemy's top (item-morph-ball frame 797) clears the y
   subpixel because the landing pose's fit looks down and finds her touching the enemy
   (`SolidEnemies`, W13). The earlier reading (the gap less 1, so she touches a pixel early) was
   wrong: a gap of 0 is boxes whose pixels are next to each other.
6. Answered: the crouch take-off's push and lift come in one frame (PF1, `PoseFitAfterMoves`).
   Which comes first shows only with a block within 10 px above her, which no record has.
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
11. Crouched, with down and jump both just pressed in one frame, the morph wins (`PadChart`): original
   only. No record presses both in one frame; open until one does.
12. Answered: a turn in a jump or a fall that meets the floor before its animation ends stays the
   turn, its vertical speed still growing, and lands only after the turn ends (`Landing`,
   `WhatHerMovesMet`; TL1-TL3, TP1, TP3). The ground turn sinking onto a floor is the same (TL4, TP2).
13. A moonwalk's jump turn with the moonwalk aimed (§7, the moonwalk's own aim): no record has one,
   and the chart's rows for that turn (`PadChart`, "turning ... from the moonwalk's jump") are
   checked for the unaimed turn only.
14. The let-go row while `Landing` (kept as she is, an aimed landing included) is original only:
   no vector checks it yet. Landing firing is on record (landings with shoot held) but has no
   vector.
15. The empty column and the column far past (`SlopesCarry`) have constructed vectors only. A floor's empty column stops a
   downward move that ends on the block's bottom row; whether any room's floor meets her that way
   is not measured.
16. No vector yet covers the check after a horizontal move on a column far past: a ceiling column
    just past pushes her 0 px and seats her, one far past does neither. Constructed vectors would
    settle it; until then it rests on the original.
