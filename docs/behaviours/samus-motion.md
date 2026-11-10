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
| vertical speed | signed `P.S`, downward positive; at 0, also whether she is falling (§1.2) | how fast she rises or falls; on the ground, a speed she keeps without moving by it, except as the ball or while her body changes shape | `RisingAndFalling`, `Jumping`, `Landing`, `Moonfall`, `FollowingTheGround` |
| bounces | count 0..=2 | how many times the ball has bounced since it last rested | `Landing` |
| the y a placement put her at, if any | y: `P.S`, or none | the screen scrolls after her vertical move as measured from where pose fit or a change of shape placed her this frame (`PoseFitAfterMoves`, `ShrinkingLowersHer`), a placement of 0 px included, so the scroll doesn't chase those placements; with none, it measures from where it last saw her. The camera reads only its pixel part and keeps its own subpixel; the subpixel given is hers at the frame's start (`PoseFitAfterMoves`) | output (the camera's domain reads its pixel part; each frame works it out, so it isn't State) |
| rising | `bool`, at the frame's end | the view aims lower while she rises, so it shows more above her; her vertical speed can't tell it at a rise at 0, or on the frame whose pull spends the rise, where the speed already points down (`RisingAndFalling`) | output (the camera's domain reads it) |
| pose | §2 | the sprite | every behaviour; output |

### 1.2 What she remembers

Some of what she does depends on what came before. These are behaviours, told by what the player
sees; how the model keeps them is the implementer's choice. A vector shows them by its lead-in
frames (the vectors' format), never by a value in its start, with two exceptions: falling at 0
on the ground (below), which a start gives as `falling`, and jump held through (below), which a
start may give.

- **Carrying on.** Her horizontal motion either follows the pad (it builds while she holds a side,
  and in the air it stops when she lets go) or carries on by itself (it brakes on the ground and
  keeps building in the air with the pad let go). It carries on after a jump or a fall she started
  with a dash (`Jumping`), while a spin jump drifts (`DriftInTheAir`), through a wall jump
  (`DriftInTheAir`), and while she skids (`TurningRound`). It follows the pad again once she stops
  braking (`SpeedBuildsAndBrakes`), halts, lands (`Landing`), hits a wall or ceiling
  (`WallsAndCeilings`), or takes off (`Jumping`), or stops steering during a turn
  (`TurningRound`).
- **Skidding.** A skid is her moving against her facing: she turned round, or reversed a spin, and
  travels against her new facing (the old way, unless she was already skidding, `TurningRound`). A
  turn from a standstill is a skid of zero: its first frame brakes, so she doesn't creep until the
  frame after (`TurningRound`). Whether she skids is output: the camera leads her backwards while
  she does, a skid of zero included, which a speed of 0 can't show (MT2).
- **Her dash.** The dash she builds running stays hers, through jumps and into the ball, until a
  halt takes it, and a dash a pose change left on her is lost the next frame she moves (`Dash`).
- **The moonfall** (`Moonfall`).
- **How long jump has stayed held** since it was pressed: a count of frames, 0 to 65,535, which a
  landing or a turn's end can make a press. Read by `Jumping` (jump held through).
- **Landed or ended the frame before**: `bool`, whether, the frame before, she landed or a turn's
  or a landing's animation ended and handed over its pose; it lasts one frame. Read by `Jumping`
  (jump held through), whose press comes the frame after.
- **Jump held through is state a caller gives and reads back.** A caller starting mid-play (a
  saved moment, a frame from a record) gives both parts: any count from 0 to 65,535, and either
  value of the frame before, in any pose. At the end of every frame it reads both back, as the
  next frame starts from them. Nothing else in her start limits them: standing still with a count
  of 5 and the frame before `true`, jump held counts as a press on her first frame and she takes
  off (C-held-built), and the counts past the window act as `Jumping` says (C-held-long,
  C-held-edge, C-held-wrap).
  A vector gives them as `before.jump_held` and `before.landed_or_ended`, and compares them as
  `after.jump_held` and `after.landed_or_ended` (C-held-landed).
- **A spin carried on**, which can ready a wall jump (`WallJump`).

A vector starts at a moment where none of this is in play: her motion follows the pad unless she
is skidding, any dash she has is hers to keep, and a vertical speed of 0 means she stands on the ground,
except in a jump, a spin, a fall or a wall jump, where it means she has just started to fall. On the
ground a vertical speed other than 0, downward or upward, is one she keeps there, or that the ball
moves by (`FollowingTheGround`): the speed itself shows it, so a start may have one. An upward
one there is a rise she keeps (`RisingAndFalling`, C-ground-rise-kept).

**Falling at 0 on the ground.** The ball, or her body while it changes shape, can be on the ground
and falling at a vertical speed of 0, which moves her unlike no vertical speed
(`FollowingTheGround`). In any other pose on the ground she keeps it and moves as with none, until
she leaves the ground or changes shape (`FollowingTheGround`, BG3). Only a push beyond this spec
hands her back so (a bomb jump's end, at its peak, or a knockback's end, §10): in scope, a landing
leaves her no vertical motion, the ball falling at 0 walks off the same frame, and a change of
shape falling at 0 grows the speed the same frame. So no lead-in in scope reaches it, and a start
says it outright: the vectors' `before.falling`. A vertical motion, on the ground, is a vertical speed other than 0
or falling at 0.

### 1.3 What she reads from the world and the player (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| pad | held: run, jump, shoot, left, right, up, down, aim up, aim down; just pressed: each of them, held this frame and not the frame before | most behaviours |
| items switched on | the `Item`s switched on: progress's own output (progress §1.1), never the ones merely hers | below |
| world | the room's blocks and solid enemies, answered as §8 says | §8 |
| spin turned far | `bool`: the spin's sprite has turned far enough to kick off a wall | `WallJump` |
| elevator moving | `bool` | `RidingTheElevator` |
| frame parity | `Even` or `Odd`: the game's frame count, alternating every frame | `BlocksStopFlush` |
| body size | each pose's half-width and half-height in px (`radius`): the game's data, for any pose, the one the step chooses included | §8, `PoseFitAfterMoves` |

**The items she reads**, of progress's `Item`s, and what each changes. "With" an item, here and
below, means with it switched on:

| Item | What it changes here |
| --- | --- |
| `HiJumpBoots` | her jump and wall jump speeds (`Jumping`) |
| `SpaceJump` | the jump again in a spin (`Jumping`), and the spin's sprite (`HowSheIsDrawn`) |
| `MorphBall` | without it, a change into `Morphing` is refused (`PoseFollows`) |
| `SpringBall` | the ball's hop (`PadChart`) and which ball a morph ends in (`AnimationEnds`) |
| `ScrewAttack`, `VariaSuit`, `GravitySuit` | only her sprite (`HowSheIsDrawn`); `GravitySuit` also in liquids, out of scope (§10) |
| `SpeedBooster` | its dash cap and higher jump, out of scope (§10): the recorded vectors that switch it on come from frames neither shows on |
| `Bombs`, `GrappleBeam`, `XRayScope` | none of her motion: what they do is other domains' (§10) |

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

**Aim in a turn.** While `Turning` she keeps the aim she had just before (`PadChart`), apart from
one case `TurningRound` gives.

**Her body while it changes shape** is the body she changes into: crouched while `Crouching` or
`Unmorphing`, a ball while `Morphing` (whether it is the spring ball is decided when the animation
ends, `AnimationEnds`), upright while `StandingUp`.

**What the sprite shows beyond these parts** follows from the inputs, and moves her the same
(`HowSheIsDrawn`).

The ball `Still` is the ball at rest. While her body grows or shrinks, she moves as
`SpeedBuildsAndBrakes` says. The frame a crouch or a morph starts, she is lowered
(`ShrinkingLowersHer`); the frame a taller pose starts, pose fit places her (`PoseFitAfterMoves`).

### HowSheIsDrawn
- kind: output.
- summary: what her sprite shows beyond her pose's parts, from the items switched on.
- params: none.
- switch: none.
- The spin (`Spinning`, and the spin of `WallJumping`) → it is drawn as the screw attack with
  screw attack, else as the space jump with space jump, else plain [original only]
- In a liquid without the gravity suit the spin is always plain; liquids are out of scope (§10).
- Facing the screen → she is drawn in her first suit, or in the other suit's sprite with varia or
  gravity [original only]
- How far her animation has run is out of scope (§10).
- checked: original only (the sprite isn't compared: the recorded vectors switch these items on
  as their records do, and compare none of it).

---

## 3. On the ground

### SpeedBuildsAndBrakes
- kind: mechanic and parameter.
- summary: how her horizontal speed builds towards a top speed, brakes and halts, by what she is
  doing.
- params: the speed-up step and the top speed for each activity (the table); the brake, 0.8000 a
  frame.
- switch: none (the top speed's test is `WordwiseTopSpeed`'s).

| She is | Speeds up by | Top speed |
| --- | --- | --- |
| running | 0.3000 | 2.C000 |
| rolling as a ball | 0.C000 | 3.4000 |
| moonwalking | 0.C000 | 0.8000 |
| in a straight jump or the spring ball's hop | 0.C000 | 1.4000 |
| in a spin jump or a wall jump | 0.C000 | 1.6000 |
| falling, upright or as a ball | 0.C000 | 1.0000 |
| turning round | 0.C000 | 0.0000 |

- Cells: running; rolling as a ball; moonwalking; in a straight jump or the spring ball's hop; in
  a spin jump or a wall jump; falling, upright or as a ball; turning round.
- A frame whose pose moves her → she moves horizontally by `speed + dash × way`, `way` being +1
  or −1 as her horizontal speed points (her facing when it is 0) [R2, R61, R144, J1]
- Left and right both held → her motion reads it as the side she faces held, never as back held
  [original only]
- Following the pad while running, moonwalking, rolling or turning, or in the air as §5 says →
  each frame she gains the step, `speed + step`, up to the top speed (`WordwiseTopSpeed`), and
  moves along her facing [R2, R61, R76, R95, R97, C-cap]
- Carrying on along the ground → she brakes instead, `speed − 0.8000` a frame [R133, R1, R87]
- Carrying on along the ground, braking would take her below zero → she stops (horizontal speed
  0) and follows the pad again [original only]
- Standing, crouched or against the wall → the pad doesn't move her: she moves by her dash only,
  then halts (horizontal speed 0, dash 0, following the pad) [R77, R47, R49, R107, R18]
- While her body changes shape, or as the ball at rest on the ground following the pad with a
  vertical motion (a vertical speed, or falling at 0: §1.2, `FollowingTheGround`) → she moves by
  her dash only and keeps both speeds [R48, R89, R128, R129, BG1, BG2]
- Facing the screen → she doesn't move at all and keeps her speeds [C-elevator]
- The ball at rest on the ground, following the pad, with no vertical motion → it moves by its
  dash only and halts after that move [R43]
- The ball at rest, braking or skidding → it rolls like running, with its own speeds, and keeps
  its dash (the mockball) [original only]
- checked: oracle (the running, ground and turning vectors; samus-mockball,
  samus-bomb-jump-vertical; BG2, the ball at rest falling at 0); original only (braking below
  zero: R98 and R123 start `Still` following the pad, so they halt as standing does; left and
  right both held).

### Dash
- kind: mechanic.
- summary: the extra speed she builds running with run held, and its cap.
- params: the gain, 0.1000 a frame; the cap, 2.0000.
- switch: none.
- Running with run held, the dash before the step has `below(pixel, 2) or below(subpixel, 0)`
  (`below` as `WordwiseTopSpeed` gives it) → it grows, `dash + 0.1000` [R2, R61, R97, R144,
  C-dashcap-a]
- Running with run held, neither part below → it becomes 2.0000 [C-dashcap-b]
- So 2.F000 grows to 3.0000 and then snaps to 2.0000 (invisible in vanilla).
- No other moving activity gains dash, taking off included (`Jumping`; W16, R95, R133).
- A pose change halted her but left the dash's value on her (a dash she is to lose, §1.2) → the
  next frame she moves, the dash is gone (0) before that move [W16]
- A dash she is to lose is gone (0) in any other moving activity. Standing and against the
  wall (`SpeedBuildsAndBrakes`), a wall stop (`WallsAndCeilings`), a turn's end
  (`AnimationEnds`), the falling ball letting go (`DriftInTheAir`) and a spin reversed
  (`TurningRound`) each take it (0).
- checked: oracle (running vectors with run held; samus-mockball for the carry; W16 for a dash
  lost); original only
  (C-dashcap-a, C-dashcap-b).

### TurningRound
- kind: mechanic.
- summary: turning round, or reversing a spin, makes her skid the old way and brake before she
  goes the new way.
- params: none of its own (turning's speeds are `SpeedBuildsAndBrakes`').
- switch: none.
- Turning round, on the ground or in the air, reversing a spin (a spin facing the other way: any
  spin, from its first frame on, §1.2), or the ball turning round → her dash joins her horizontal
  speed, `speed + dash` (dash 0), and she keeps travelling the old way, now against her new
  facing: a skid [R1, R80, R87, SR1]
- She brakes until the speed is spent (`SpeedBuildsAndBrakes`, `DriftInTheAir`), then builds the
  new way (turning's top speed is 0, so only the creep below). When the turn ends she loses her
  dash (`AnimationEnds`).
- Every turn and every spin reversed leaves her skidding, whatever she was doing before: her
  speed, the dash added, then points against her new facing.
- The moonwalk's jump, the one turn that keeps her facing (`PadChart`) → it leaves her skidding
  too: her speed points against her facing, back the way her sprite looks, though the moonwalk
  carried her along her facing [MT2]
- So the turn's first frame brakes that speed to 0 (the moonwalk's top speed, 0.8000, is one
  frame's brake), and only the camera, which reads the skid, shows which way it pointed.
- A spin reversed while she is already skidding (travelling against her old facing, which is the
  way she now faces) → that frame she moves the old way (`MovesAsShePosed`), and from the next
  frame the same speed points the other way, back along her old facing, and goes on braking as a
  skid [W14]
- So turning from a standstill slides her 0.C000 every other frame the new way: the speed goes 0,
  0.C000, 0, because turning's top speed is 0 and `WordwiseTopSpeed` keeps 0.C000 over 0 once
  (R93, R94).
- Turning with no button held at all, on the ground or in the air, or on the ground holding only
  buttons that bring no pose change (run, shoot or jump alone: the chart's last row for turning
  on the ground, `PadChart`) → her skid ends after that frame's move: she keeps her horizontal
  speed, which now follows the pad and points along her new facing [R133, R64, R11, R12]
- So a skid braked to exactly 0 ends there too.
- Turning in the air holding any such button → she keeps skidding: the speed still points the old
  way, and the skid ends only as `SpeedBuildsAndBrakes` says, when braking would take it below
  zero [R141, R142]
- Turning on the ground over a drop → she sinks by the ground pull (`FollowingTheGround`: her
  speed, subpixel part included, plus 1 px) a frame instead of falling [W11, TL4]
- In the air she turns under gravity (`RisingAndFalling`), except in the moonfall, where a turn
  sinks as this one on the ground does (`Moonfall`).
- A turn in a jump or a fall begun aimed `DownForward` → it is aimed `Down`, and ends so
  (`AnimationEnds`) [original only]
- A turn in a jump or a fall has one sprite for `DownForward` and `Down`, and it is `Down`.
  Crouched, the same sprite is `DownForward`; no grounded pose aims `Down`.
- checked: oracle (samus-input-poses turns, samus-screw-attack; R133 for letting go, R11 and R12
  (samus-input-poses:120-121) for run held on the ground: braked to 0 at 120, she creeps 0.C000 at
  121, which a skid kept at 0 would not; R141 for run held in the air; SR1 for a spin reversed in
  its first frame, which skids; TL4 for the sinking onto a floor; W11 for the sinking over air,
  subpixel and all; W14 for a spin reversed while skidding; MT2 for the moonwalk's jump, chain-0014-96BA,
  as chain-0232-D5A7:18 and samus-input-poses:119 show it too); constructed (the sinking); original
  only (the aim of a turn begun aimed `DownForward`).

### AimingAndShooting
- kind: quirk and output.
- summary: changing into another running pose steps her 1 px forward.
- params: the step, 1 px.
- switch: none.
- A frame whose pose is chosen, by the pad chart and not replaced by what her moves met
  (`PoseFollows` items 2 and 3), as a running pose other than the one she is in (running with
  another aim, but also from standing, from a landing, or from any pose whose pad chart row leads
  into running) → she moves 1 px forward, by the facing she had at the frame's start, after that
  frame's moves and the ground pull, without a speed change [A1, W12, W17, W18]
- Running, the pad chart's let-go row keeping her running pose (no other row fits and her
  horizontal speed at the frame's start isn't 0, `PadChart`) → the kept pose counts as chosen: she
  moves the 1 px forward on top of the braking move, every frame she brakes so, the frame her dash
  joins her speed and the frame braking takes the speed to 0 included [AS1, AS2, AS3]
- So a row whose next pose is the one she is in already (forward held, running forward) brings
  no step (C-cap), and a landing or a fall that replaces the pad's running pose brings no step, and an animation's
  end handing over a pose neither brings nor cancels it.
- A block or a solid enemy in the way of the step puts her against the wall instead
  (`WhatHerMovesMet`), so a standing start next to a wall with the pad asking to run leaves her
  against it. Against the wall with forward held is the same: the row into running brings the
  step, the wall she is against stops it, and she stays against the wall where she is (R18, R88,
  R117, R149). A wall stop that frame while running puts her against the wall too, without a step
  (`WhatHerMovesMet`).
- A solid enemy within the step's reach (`SolidEnemies`) → it stops the step with no move at all
  [C-aim-enemy]
- One she already touches still clears her y subpixel, as any test against one does
  (`SolidEnemies`; C-aim-enemy). A block in the way stops it as any horizontal move
  (`BlocksStopFlush`): by whole pixels up to the face, at most the 1 px, with her x subpixel
  written FFFF or 0, so a face less than a pixel ahead moves her that part of a pixel
  (C-aim-short).
- After the step → the floor and ceiling checks of `SlopesCarry` don't run, so it can leave her
  bottom edge in a sloped floor's solid part [C-aim-slope]
- Standing, unaimed and not landing, holding shoot → the standing animation restarts [original
  only]
- checked: oracle (A1, the step; W12 standing into running, W17 and W18 a landing into
  running, W19 standing into running against a wall; R18, R88, R117 and R149 against the wall
  with forward held; AS1 the let-go frame her dash joins her speed, AS2 the next, braking, AS3 the
  frame braking takes her speed to 0); constructed (C-aim-short, C-aim-enemy, C-aim-enemy-stop,
  C-aim-slope, C-cap); original only (the shot).

### RidingTheElevator
- kind: mechanic.
- summary: facing the screen on a moving elevator, she goes down with it.
- params: the descent, 1 px a frame.
- switch: none.
- Facing the screen on a moving elevator → she moves down 1 px a frame, and nothing else of hers
  changes, her speeds and dash included [C-elevator]
- A solid or stepped block stops that move as any downward move (`BlocksStopFlush`); a sloped
  surface under her there is out of scope.
- checked: constructed (C-elevator).

---

## 4. Leaving the ground

### Jumping
- kind: mechanic and parameter.
- summary: how she takes off, starts a spin or a hop, jumps again with space jump, and takes off
  again with jump held through a landing.
- params: jump 4.E000 upward (hi-jump 6.0000); wall jump 4.A000 (hi-jump 5.8000); ball bounce
  1.0000, and an adjusted one at least 1.0000 (the rise after bounce 1 is it less 1 px,
  `Landing`); the crouch lift 10 px; space jump's window, 2.80 to 5.00; jump held through, a
  count of 1 to 8 frames (and 32,777 to 65,535).
- switch: none.
- The first poses of a straight jump (taking off) → she doesn't steer: she moves by her dash only
  and follows the pad; she gains no dash (`Dash`); she doesn't move vertically and the pull
  doesn't act [J3, W16]
- A dash she is to lose (§1.2) is gone (0) before that move, so taking off from a crouch she
  doesn't move at all and the jump that follows follows the pad (W16).
- The frame she changes into a take-off → she starts rising at the jump speed, without moving
  vertically that frame [J2, PF1]
- The frame she changes into a take-off from a crouch without aim → she is also lifted, `y − 10`
  (`PoseFitAfterMoves` says when) [PF1]
- Starting a spin jump, or the spring ball's hop from the ground → she rises at the jump speed
  [J1]
- Starting a straight jump or a fall with a dash → she carries on along her facing, so a running
  jump keeps its drift [original only]
- Starting a straight jump or a fall without a dash → she follows the pad [W16, J3]
- With space jump, falling in a spin, jump pressed, her fall speed read as 8.8 `v` (its pixel
  part's low byte and its subpixel's high byte) with `2.80 ≤ v < 5.00` → she jumps again, rising
  at the new speed that same frame [SJ1]
- **Jump held through** (§1.2): a count of the frames jump has stayed held since its press.
- At a frame's end, jump held and not just pressed → the count goes up by 1 [JH1, JH2, JH3, JH4]
- At a frame's end, jump just pressed, or not held → the count is 0 [JH1, JH2, JH3, JH4]
- The count goes from 65,535 to 0, and a count of 32,777 to 65,535 (jump held about 9 minutes or
  more) acts as one of 1 to 8 does below; 32,776 doesn't [C-held-wrap, C-held-long, C-held-edge]
- The frame after one on which she landed from a straight jump, a fall, a spin or a wall jump
  (`WhatHerMovesMet`'s rows into `Landing`, whatever pose fit makes of the change), or on which
  the animation of a turn or of a landing ended and handed over its pose (`AnimationEnds`), with
  the count at 1 to 8 → jump counts as just pressed for the pad chart that frame (`PadChart`), and
  the count is 0 before that frame's end adds to it [JH1, JH2, JH3, JH4, C-held-9, C-held-built,
  C-held-landed]
- Jump let go on that frame → it still counts as just pressed for the pad chart; only the frame's
  end sees it let go, and puts the count at 0 [C-held-let-go]
- So JH3 with jump let go on the landing's next frame takes off all the same (C-held-let-go).
- So with jump held through a landing she takes off from it the next frame (JH1-JH3), unless the
  count has passed 8 (C-held-9: 9). A turn's end on the way, at a count of 1 to 8, starts it
  again (JH4: pressed at 96, it is 2 at a turn's end, 7 at the next, and 1 at the landing, not 10;
  frames numbered by their start).
- The press counts only where a row names jump pressed: the fall's rows don't, and a landing
  outranks the pad (`PoseFollows`; JH4). The other rules that read a press see the pad as it is.
- checked: oracle (J1, J2, J3, and the take-off and spin vectors; W16, a take-off from a crouch
  with run held; SJ1, space jump's jump again with hi-jump; the crouch lift open: question 6;
  JH1-JH4, jump held through a landing from a straight jump and a fall, JH4 through two turns'
  ends); constructed (C-held-9, a count of 9; C-held-let-go, jump let go on the landing's next
  frame; C-held-built, C-held-long, C-held-edge and C-held-wrap, counts a caller gives at the
  start, and C-held-landed, the two parts read back after a landing).

### FollowingTheGround
- kind: mechanic.
- summary: on the ground she is pulled down onto the floor after each horizontal move.
- params: the extra 1 px.
- switch: none.
- On the ground, after her horizontal move → she is pulled down by that frame's speed plus 1 px,
  `|speed| + dash + 1`, subpixel part included (1 px on a slope, `SlopesCarry`), and the floor
  stops her flush (§8; a sloped floor as `SlopesCarry` says, C-slope-roll-carry,
  C-slope-roll-carry-next, C-slope-run-off) [R2, R61, R144, W11]
- So 0.C000 pulls her 1.C000: with no floor in reach she moves all of it, her y subpixel included
  (02A9.0000 becomes 02AA.C000, W11); on a floor the flush stop hides the subpixel part.
- Walking off a ledge (the pull finds no floor) with no upward vertical speed, unless she is
  turning or her body is changing shape → she moves the whole pull, subpixel part included, and
  falls from rest from there: vertical speed 0, falling, bounces 0 [P6, C-slope-run-off]
- Walking off so with an upward vertical speed, one she keeps on the ground → she keeps that speed
  instead and rises on from there, as in the air (`RisingAndFalling`); bounces 0 [C-ground-rise]
- So a rise that a change of shape carries into a pose on the ground (the unmorph's end in the air,
  `AnimationEnds`) lasts: she keeps it there and doesn't move by it, and walking off she rises on
  in the fall's pose until the pull spends it (record only: item-charge-beam 7-16, the crouch
  rising at 2.4000 walks off and rises on in the fall; item-reserve-tank 520-521, the crouch
  rising at 2.2400 pulled 1.0000 into the fall, as C-ground-rise).
- On the ground (standing, running, crouched, turning on the ground, moonwalking, against the
  wall) → her vertical speed is kept as it is, whatever it is, and so is falling at 0 (§1.2):
  she doesn't move by it (the pull above is her only vertical move), the pull of
  `RisingAndFalling` doesn't grow it, and the floor under her doesn't land her, so it never halts
  her [HV1, HV2, HV3, AS3, AS4, C-slope-carry-kept, BG3]
- So falling at 0 changes nothing in these poses: she moves exactly as with no vertical speed, a
  walk off a ledge included (she falls from rest either way), and nothing on the ground ends it.
  Only leaving the ground does (a jump or a spin start sets the jump speed), or a change of shape
  shows it: crouching or morphing from it, she moves by it under the pull (below), 0 px, and the
  speed grows to 0.1C00 where with no vertical speed she would be pulled onto the floor (BG3).
- The kept speed plays no part in the pull either: on a sloped floor she is pulled and stopped
  exactly as with no vertical speed (`SlopesCarry`; C-slope-carry-kept). What it does change on a
  slope is the walk: a sloped floor shortens a move only at a vertical speed of exactly 0
  (`SlopesSlowTheWalk`), so a run with a kept speed covers its whole speed down or up a slope.
- While her body changes shape, or as the ball on the ground (the spring ball too), with a
  vertical motion (a vertical speed, or falling at 0: §1.2) → she moves by it under the pull
  (`RisingAndFalling`) instead of being pulled down onto the floor, and a floor she meets stops
  her as `Landing` says [HV4, BG1, BG2]
- So the ball falling at 0 moves 0 px, and a move of 0 is never stopped by blocks
  (`RisingAndFalling`): it is a walk off the ledge, as below, and she falls from rest where she
  was (BG2).
- So a change of shape that ends before a floor stops it leaves her the speed it reached (HV3),
  and the ball keeps its speed on the floor, still growing it (BG1); a move by it that finds no
  floor is a walk off the ledge, as above. With no vertical motion (no vertical speed, and not
  falling at 0) both are pulled onto the floor as standing is.
- So crouching from a run she comes to keep a downward speed: +0.E000 grows to +1.3400 in the
  crouch (samus-shinespark-left 151-155), and she keeps it crouched, through a turn on the ground
  and into the take-off at 196 (HV3). The kept speed ends when she leaves the ground: a jump or a
  spin start sets the jump speed (`Jumping`), and walking off a ledge she falls from rest, as
  above.
- checked: oracle (every ground vector; W11, a turn on the ground over air; the kept speed: HV1
  standing, HV2 turning on the ground, HV3 crouched, HV4 the crouch moving her by it, AS3 running
  and braking; samus-mockball and
  the four shinespark rows run with +0.E000 from their first frame, and samus-space-jump-chain
  and the three wall-jump rows run, brake, stand and turn with it; BG2, the ball on the ground
  falling at 0, samus-bomb-jump-vertical:1037); constructed (BG1, the ball at rest moving by its
  vertical speed: no record has the ball on the ground with one; BG3, standing falling at 0, kept
  into a crouch: no record has her upright on the ground and falling at 0).

### MovesAsShePosed
- kind: timing.
- summary: a frame's moves follow the pose she started it in.
- params: none.
- switch: none.
- The frame she changes pose → she still moves as the pose at the frame's start says: what she is
  doing, her speeds, and what a landing or a fall does, come from the pose at the frame's start
  [J1, J2, A1, W12, W14]
- checked: oracle (J1, J2, A1).

### PoseFitAfterMoves
- kind: timing and mechanic.
- summary: when her new pose is taller, she is pushed clear of what is above or below her, or the
  change is refused or becomes a crouch.
- params: the 8 px look.
- switch: none.
- A frame ends in this order: her moves (horizontal, the ground pull or her vertical move, then
  the aim change's step), then what they met (a landing, a fall, a ceiling, a wall), then the pose
  change. Within the pose change, pose fit's push comes first, then the new pose's speeds (a
  jump's rise, `Jumping`), then the crouch take-off's 10 px lift.
- A crouch take-off's frame → the push is found from where her moves left her, before the lift:
  her y is the moves' y plus the push, less the lift, `y + push − 10` [PF1]
- A push found after the lift would differ wherever a block is within 10 px above her.
- Pose fit is part of the frame's step: it follows from the pose the step chose, the pose she had,
  their bodies' sizes (§1.3) and the world.
- The chosen pose differs from the one she had, doesn't face the screen, and has a body taller by
  g px, `g = new half-height − old half-height > 0` → from where her moves left her, at her old
  size, she looks g px up and g px down; otherwise there is no push [PF1, PF2, PF3, U1]
- Each look is a vertical test move of g whole px from her position, subpixel included, stopped as
  a move is, against solid enemies (`SolidEnemies`) or against blocks (`BlocksStopFlush`, its
  `Even`/`Odd` rule included, and the sloped floors of `SlopesCarry` for a look down).
- A look up → it passes every sloped ceiling [original only]
- What a sloped floor does to a look up is open (§11, question 18).
- The looks → run one after another, each seeing what the ones before it wrote: solid enemies up,
  solid enemies down, then the enemy's far-side look (below; only with an enemy on one side
  alone); then blocks up, blocks down, then the block's far-side look (only with a block on one
  side alone) [C-fit-tunnel, C-fit-land-tunnel]
- So the block look up writes her y subpixel before the block look down does, and the enemy looks
  all run before a block look writes it.
- A look refuses the change → the looks after it don't run [original only]
- Against blocks, a look of 8 px or more (the far-side look and `ShrinkingLowersHer`'s included)
  → it is stopped by whatever a look of 8 px the same way would stop, with the room that look
  found; only when nothing stops the 8 px look is it the look of its own distance
  [C-fit-step-ceiling, C-fit-step-floor, C-fit-step-enemy]
- Against blocks, a look of under 8 px → it is that look alone, however much her body grows
  [C-fit-ceiling]
- So a stepped block decides by the half that 8 px reaches: a ball unmorphing (g = 9) with her top
  edge against a stepped block's solid lower half finds no room above, though 9 px would reach its
  air upper half; and in C-fit-ceiling the far-side look of 6 px is one look, with no 8 px look
  before it. The room on a side is how far its look got: its whole distance where nothing stopped
  it.
- A look stopped → it doesn't move her, but what stops it writes to her as a stop would (a block
  her y subpixel, FFFF looking down and 0 looking up; a solid enemy right against her clears it to
  0; a sloped floor leaves it), and the write stays whatever pose fit then decides, a refusal
  included [C-fit-ceiling, C-fit-tunnel, W13]
- A look down that a stepped block stops seats her on a slope (`SlopesCarry`).
- With nothing in reach on either side there is no push: she grows about her centre (record only:
  area-norfair:38).
- Something on one side only → the far side is looked at once more for `g − room` (blocks at her
  old size; for a push from a solid enemy, solid enemies at her new size), and unstopped, she is
  pushed away from it by `g − room` in whole px, keeping the subpixel her looks left: up from what
  is below her, down from what is above her [PF1, PF2, PF3, U1, C-fit-ceiling]
- So on a floor with nothing above, her feet stay where they were: a body that grows by n px
  raises her centre by n px.
- A block and a solid enemy on the same side → the enemy's far-side look comes first and can
  refuse the change; then the push is by the block's room, as if the enemy were not there
  [original only]
- That far-side look stopped, or something above her and something below her (blocks or solid
  enemies, in any mix) with her old body under 8 px half-height (the ball) → the change is
  refused [C-fit-ball-tunnel]
- Something above her and something below her, her old body 8 px half-height or more → she takes
  crouched `Still` instead, aim `Forward`, facing as before the change; with an old body shorter
  than the crouch she rises by the difference, her feet staying where they were, and there is no
  other push [C-fit-tunnel, C-fit-land-tunnel]
- A block on one side and a solid enemy on the other → each side's far-side look runs first, as
  in the one-side case (the enemy's, then the block's), and either one stopped refuses the change
  before the crouch or the ball's refusal is reached [C-fit-step-enemy]
- A refused change, or the crouch taken instead → the frame ends in the pose she had or in the
  crouch, and none of the chosen pose's effects on her speeds and dash come: not its pad chart
  row's (a rise, a halt, a brake, the dash joining her speed), not what her moves met (a landing's
  stop of her speeds: her horizontal and vertical speeds stay as her moves left them), and the
  crouch has no take-off (no rise and no 10 px lift) [C-fit-land-tunnel, C-fit-tunnel,
  C-fit-ball-tunnel]
- The block looks reach cells as a vertical move of their distance would, reported with the way
  `Look` (`TouchedCells`). Solid enemies decide their looks and leave no cells. What a reacting
  cell does with a look is `TouchedCells`' (§11, question 17).
- **Where a placement put her** (§1, an output of the step): the camera's scroll measures her
  vertical move from it, so pose fit's push is not a move the camera chases.
- Pose fit moves her this frame (a push, or the crouch taken instead raising her) → she is placed
  at her y pixel just after that, with the subpixel her y had at the frame's start [PF1, PF2,
  PF3, U1, W13, C-fit-ceiling, C-fit-land-tunnel]
- So the crouch taken instead from an old body already the crouch's height (crouched, aimed or
  not) raises her 0 px and leaves her unplaced, though her pose changes (C-fit-tunnel-aimed).
- So the push's pixels and the frame's moves before it are not followed, the subpixel part of
  those moves is, and what comes after the push is: the crouch take-off's 10 px lift (PF1: from
  02C0.FFFF, pushed 3 px to 02BD, lifted to 02B3.FFFF, placed at 02BD.FFFF).
- So any other frame, a growing body that finds nothing or a refused change included (and not a
  change of shape, `ShrinkingLowersHer`), has no placement (W11, C-fit-step-enemy).
- checked: oracle (PF1, item-morph-ball:1311: crouched to the take-off, the body grows 3 px and
  she rises 3 px, then the 10 px lift, in one frame; U1, crouched to standing, 5 px; PF2, a
  landing from a jump, 2 px; PF3, a landing from a spin, 9 px; the push up from a floor in every
  recorded landing on a flat floor; area-norfair:38, a body growing 9 px in open air, no push);
  constructed (C-fit-ceiling, pushed down from a ceiling, the look's subpixel write kept, and the
  looks' cells with a far-side look under 8 px; C-fit-tunnel, a take-off between a floor and a
  ceiling crouches instead; C-fit-ball-tunnel, unmorphing there is refused; C-fit-step-ceiling and
  C-fit-step-floor, the 8 px look against a stepped block, and in C-fit-step-ceiling the cells of
  a 9 px far-side look and its 8 px look; C-fit-step-enemy, a stepped block above and a solid
  enemy below; C-fit-land-tunnel, a landing crouched instead keeps her speeds; C-fit-tunnel-aimed,
  the crouch taken instead from a body already the crouch's height: no placement); original only
  (the looks' cells of the harness case W15 and of C-near-wall-item, a body growing 7 px in open
  air, which the record doesn't show; the second
  look's refusal, the mixed and same-side cases beyond C-fit-step-enemy, a look up passing a sloped
  ceiling, the seat, and the cells' reactions); the placement: how far the records' screen may
  move up or down the next frame (PF1, PF2, PF3, U1, W13 after a push, W11 on a frame without
  one); constructed for the cases no record has: C-fit-ceiling (a push down), C-fit-land-tunnel
  (the crouch taken instead, rising 4 px), C-fit-tunnel-aimed (the crouch taken instead, no rise:
  none), C-fit-step-enemy (a refusal: none).

### ShrinkingLowersHer
- kind: mechanic.
- summary: the frame she starts to crouch or morph, she is lowered.
- params: the crouch's 5 px; the morph's 9 px.
- switch: none.
- The frame she starts to crouch (`Crouching`, aimed or not) or to morph (`Morphing`) → after her
  moves, as the pose change's effect, she looks down the pose's own distance (crouch 5 px, morph
  9 px) at her new size, a block look as pose fit's are (`PoseFitAfterMoves`, the 8 px rule
  included; solid enemies don't stop it), and is lowered by the room it finds, `y + room`, in
  whole pixels, her subpixel as it is [SH1, SH2, SH3, C-shrink-floor]
- Where nothing stops it, that is the whole distance. Where a block is within reach of
  her new feet, she is lowered only as far as the block's face, and the look writes as a stop
  would: her y subpixel FFFF, or a seat on a stepped block (original only).
- On the ground this keeps her feet where they were (an upright body of half-height 21 becomes a
  crouched one of 16; a crouched one of 16 becomes the ball's 7): the look reaches only the row her
  feet are in, which is air, so she is lowered the whole distance.
- The distance is the pose's own, not the shrink: morphing from a straight jump or a fall aimed
  `Down`, whose body is shorter than the crouch, lowers her 9 px all the same, so in the air her
  feet drop below where they were, unless a floor close under her feet ends it sooner
  (C-shrink-floor).
- The look reaches cells as pose fit's do, reported in `TouchedCells` with the way `Look` after
  the frame's moves. A frame has either this look or pose fit's, never both (original only): one
  body shrinks, the other grows.
- Starting to crouch or morph leaves her speeds and dash as they are: the rows into `Crouching`
  and `Morphing` have no other effect (SH3, R48, R89).
- A crouch or a morph starting this frame, aimed or not, and also standing up aimed `Forward`
  (`StandingUp`) or unmorphing (`Unmorphing`), which lower her 0 px → she is placed at her y pixel
  just after the lowering (and after pose fit's push, for a body that grows), with the subpixel
  her y had at the frame's start (`PoseFitAfterMoves`) [SH1, SH2, SH3, C-fit-ceiling, C-place-0]
- So in the air the morph's 9 px are not followed (SH3: from 0094.B7FF, lowered to 009D.B7FF and
  placed there), and a placement of 0 px is a placement still: unmorphing in open air she is
  placed at her start y (C-place-0).
- A change refused, or the crouch taken instead, starts no change of shape, so it has no
  placement of its own (`PoseFitAfterMoves`, C-fit-step-enemy).
- Standing up aimed `Up`, `UpForward` or `DownForward` has no lowering, so only pose fit places
  her; on a floor its look down always finds the floor, so it pushes her or refuses the change.
- checked: oracle (SH1, r02:95, standing to crouching, 5 px; SH2, samus-bomb-jump-vertical:849,
  crouched to morphing, 9 px; SH3, r04:312, a straight jump aimed `Down` to morphing in the air,
  9 px on top of her fall, her speeds and dash kept; SH1-SH3 also for the placement,
  by how far the records' screen may move); constructed (C-shrink-floor, a morph in the
  air 2 px above a floor: the 8 px look stops on it, she is lowered 5 px, her y subpixel FFFF,
  and the look's one cell; C-fit-ceiling, unmorphing, for the placement; C-place-0, a placement of
  0 px at her start y); original only (a seat on a stepped block; the placement on a frame she
  starts standing up).

---

## 5. In the air

### RisingAndFalling
- kind: mechanic.
- summary: gravity, the fall's limit, and when a rise ends.
- params: the pull, 0.1C00 a frame; the fall's limit, pixel part 5.
- switch: none.
- Each frame in the air → she moves vertically by her vertical speed, then it grows downward,
  `vy + 0.1C00` (rising, the pull takes 0.1C00 off her upward speed) [R7, R53, R70, R92, R109,
  R3]
- Falling, her speed's pixel part `= 5` → it stops growing [C-fallcap-b]
- So it settles at 5.1400 (4.F800 + 0.1C00, C-fallcap-a); a speed set above that keeps its value.
- Rising, at the start of a frame after her upward speed was spent (the pull took it past zero, so
  that it points downward by even a subpixel), or, in a normal or spin jump or the spring ball's
  hop, at the start of any frame with jump not held → the rise ends: vertical speed 0, falling,
  and she doesn't move vertically that frame; the pull then acts as in a fall, so the frame ends
  at +0.1C00 [E1, R91, R108, R6]
- Rising, the pull leaves her upward speed exactly 0 → it is not spent: she rises by 0 for one
  more frame [E2, E3]
- **Whether she is rising** (§1, an output of the step), at the frame's end.
- A rise holds at the frame's end (from its start, a take-off, a jump or spin start, the spring
  ball's hop, a ball's bounce, a wall jump's kick or space jump's jump again, through the frame
  whose pull spends it, a rise at 0 included) → `rising` [J2, E1, E3, R4, WK1, SJ1, C-rise-spent,
  C-rise-at-0, C-bounce-second]
- On the ground with an upward vertical speed, one she keeps there (`FollowingTheGround`), at the
  frame's end → `rising` [C-ground-rise-kept, C-ground-rise]
- So `rising` is false on the frame a rise ends (its speed spent at the frame's start, jump let
  go, a ceiling) and on any frame without a rise: falling, the moonfall, on the ground with no
  upward speed (E2, R91, C-ceiling, R32, MT1).
- On the ground she has an upward speed only from a rise that a change of shape carried into a
  pose on the ground (`FollowingTheGround`): the original holds it so outside any push, and the
  camera, which reads `rising`, aims as for a rise (record only: item-charge-beam 7,
  item-reserve-tank 520).
- So `rising` is not the sign of her vertical speed: at a rise at 0 the speed is 0 (C-rise-at-0,
  C-bounce-second), and on the frame that spends the rise it points down (E1: +0.0C00, rising)
  where the next frame's +0.1C00 is a fall (E2).
- A frame's vertical move of exactly 0 (a fall from rest, a rise at 0) is never stopped by
  blocks, even with her bottom or top edge already inside one (`BlocksStopFlush`): it never lands
  her or ends her rise.
- An upward speed of exactly one pull is never met from a jump: no rise speed in `Jumping` is a
  whole number of pulls. The ball rises at exactly 0 when it bounces the second time, so it
  rises by 0 for one frame before the rise ends.
- checked: oracle (every air vector; E1 for the speed past zero; `rising`: J2 a take-off, WK1 a
  wall jump's kick, R4 the rise after it, SJ1 space jump's jump again, E1 the frame that spends
  the rise, R91 the rise ended by jump let go, R32 a fall, MT1 the moonfall); constructed (C-fallcap-a,
  C-fallcap-b, E2, E3; `rising`: C-rise-spent and C-rise-at-0, the camera's cases, E3 and
  C-bounce-second at 0, E2 the rise's end, C-ceiling a ceiling, C-ground-rise-kept and
  C-ground-rise an upward speed on the ground).

### DriftInTheAir
- kind: mechanic.
- summary: how she steers, keeps or loses her drift in the air.
- params: none of its own (the speeds are `SpeedBuildsAndBrakes`').
- switch: none.
- In a straight jump, falling and in the ball in the air she builds towards the top speed
  (`SpeedBuildsAndBrakes`).
- In a straight jump or falling upright, following the pad, with neither side held →
  she stops drifting at once: horizontal speed 0, no horizontal move [R53, R91, R92]
- The falling ball, following the pad, with neither side held → it halts (horizontal
  speed 0, dash 0) but still moves by its dash that frame [R52]
- Carrying on in the air → she doesn't brake: she still gains the step, so a running jump keeps
  or gains its drift with the pad let go [original only]
- A skid in the air → it brakes, as on the ground [R80, R82, R141]
- A spin jump while she holds the side she faces, or is skidding, or reached her top speed this
  frame → she carries on (a skid stays a skid) and moves [R9, R10, R100, R101, R102, W14]
- A spin jump otherwise → she stops drifting: horizontal speed 0, following the pad, and she moves
  by her dash alone [R125, R7]
- "Reached her top speed this frame" means the step was cut to the top speed: `WordwiseTopSpeed`
  gave her the top speed instead of the sum (`Exact`: the sum was over it). A step that lands
  exactly on the top speed keeps her speed, so it hasn't reached it, and with neither forward held
  nor a skid she stops drifting (original only).
- Through a wall jump → she always drifts, carrying on along her facing [R4, R30, R38, R73]
- Left and right both held → her drift reads it as the side she faces held, never as back held,
  so a spin carries on [R102]
- checked: oracle.

### Moonfall
- kind: quirk.
- summary: a spin entered as a turn ends falls without the limit and can't jump.
- params: none.
- switch: none.
- A spin she enters when a turn ends, with jump not just pressed → it starts with no rise: her
  vertical speed is 0 and she drops under gravity (`RisingAndFalling`) [M1]
- In the moonfall, until she lands → her fall speed keeps growing by 0.1C00 past the falling limit
  [C-moonfall-nolimit]
- In the moonfall, until she lands, pressing jump → it starts no rise, not even with space jump
  [C-moonfall-jump]
- The moonfall is in her vertical motion, not her pose: it lasts through a spin reversed and
  into a straight jump the pad makes of the spin, and it goes on growing there (MT1's lead-in).
- In the moonfall, turning in a jump or a fall → she moves vertically as a turn on the ground
  over a drop does (`TurningRound`): no gravity and no move by her vertical speed, which she
  keeps exactly as it is, and each frame the ground pull, her speed plus dash plus 1 px
  (`FollowingTheGround`; 1 px on a slope), which a floor stops flush; when the turn ends the
  moonfall goes on from that kept speed [MT1]
- So in MT1 she turns with her moonfall at +1.8800: each frame of the turn she sinks by her
  braking speed plus 1 px (1.4000 at the checked frame), and from the turn's end she falls by
  +1.8800 again, the speed growing by 0.1C00 a frame from there.
- A wall jump is not blocked: the spin readies and kicks off a wall near her as any spin carried
  on does (`WallJump`).
- The wall jump's kick, rising at the wall-jump speed → it ends the moonfall [original only]
- checked: oracle (r03: M1; chain-0232-D5A7: MT1, a turn in a jump in the moonfall, frames 38-43
  of the turn and 44-46 after it); constructed (C-moonfall-jump, C-moonfall-nolimit, both from
  r03's start, with its vertical motion).

---

## 6. Wall jumps and landings

### WallJump
- kind: mechanic and output.
- summary: a spin near a wall readies, then kicks off it.
- params: the near-wall test's 8 px.
- switch: none.
- Readying is not a part of her pose (§2): the pose stays the spin it was. Only the spin's
  animation shows it, skipping ahead to the point where the spin has turned far enough, so the
  input "spin turned far" (§1.3) holds from the next frame. How far an animation has run is out of
  scope (§10), so the step reports the readying for the animation to show. Each frame reports one
  of three outcomes of the near-wall test: readied, kicked (also `WhatHerMovesMet`'s wall jump) or
  neither.
- A test move of 8 px towards a wall, from where her x is after this frame's move, would be
  stopped (`BlocksStopFlush`, `SolidEnemies`) → the wall is near her: fewer than 8 px of air
  between her edge and the wall; readying and the kick use this same test [RW1, C-near-wall-item]
- A spin she was already in two frames earlier, or one that followed a wall jump, facing either
  way (a reversal keeps it) → it is carried on (§1.2) [RW1, P8]
- RW1 readies in the frame after a reversal. Reversing needs no such memory: any spin can be
  reversed from its first frame, and the reversal always skids (`TurningRound`).
- Spinning in a spin carried on, holding a side, the test looks away from the side held
  (left held: for a wall on her right; otherwise right held: on her left) → the test runs, on
  every frame that has both: on a frame that readies, on every frame after the spin has turned
  far whether or not she presses jump, and on a frame that finds no wall [RW1, C-near-wall-item]
- Without a spin carried on, or holding no side, there is no test: nothing is readied, nothing
  is written and no kick comes.
- The test finds a wall near her, by its block test or its solid enemy test, while the spin hasn't
  turned far → the wall jump is readied: the spin's animation skips ahead, and the frame reports
  `Readied` [RW1]
- Once the spin has turned far, still in a spin carried on and holding away from a wall near her,
  pressing jump → she kicks off: she doesn't move vertically that frame, and starts the wall jump
  rising at the wall-jump speed, x speed 0, following the pad; the frame reports `Kicked` [P8, WK1]
- No test that frame, no wall found, or a wall near her once the spin has turned far without jump
  pressed → the frame reports `Neither` [C-near-wall-item]
- The test finds a block wall, readying frames included → it writes her x subpixel as a stop
  against it would (FFFF looking right, 0 looking left), without moving her [RW1]
- The test meets solid enemies first, and finds a solid enemy → it writes her x nothing [original
  only]
- An enemy she already touches on that side clears her y subpixel to 0, as any test against one
  does (`SolidEnemies`), and one ahead of her writes nothing.
- The test reaches cells as a move of 8 px would (`TouchedCells`), and cells that react when she
  reaches them react to it: in item-morph-ball the test reaches an item's cell 8 px past her edge
  and she picks the item up (W15).
- checked: oracle (samus-walljump-16, -22, -28, samus-grapple-walljump, item-morph-ball's leftward
  spin jumps; samus-walljump-16:46-47: readied in one frame, kicked off in the next; RW1: the
  readying frame writes her x subpixel 0 against the wall on her left, in the frame after the
  spin is reversed, and the frame reports the readying); harness (W15, item-morph-ball:817, the
  test's cells reached in the room); constructed (C-near-wall-item, the test's cells reached over
  an invented room); original
  only (no test without a spin carried on or without a
  side held: no vector checks a spin near a wall lacking either).

### Landing
- kind: mechanic.
- summary: what touching down does to her speeds, by what she is doing, and the ball's bounces.
- params: the bounce's 3 px threshold; the bounce speed (`Jumping`).
- switch: none.
- The ball's bounce reads her vertical speed after that frame's pull (`RisingAndFalling`), not
  the speed her move used: a fall that met the floor at 2.F000 has 3.0C00 when it lands. Only
  its pixel part counts: 3 px or more.
- Falling onto a floor upright, or the ball landing after bounce 2 at any speed, or with no
  bounce yet at `vy < 3.0000` → she lands: vertical speed 0, bounces 0, horizontal speed 0,
  following the pad; the dash keeps its value [PF2, PF3, W13, C-enemy-down, C-slope-land,
  C-bounce-third]
- Only a move by her vertical speed lands her, in the air or while her body changes shape. On the
  ground the floor under her never lands her, whatever vertical speed she keeps or the ball moves
  by (`FollowingTheGround`).
- Landing while her body changes shape → only her vertical motion stops (vertical speed 0); both
  speeds, and whether she carries on, are kept [LB1, LB2]
- Landing while turning (any `Turning`: in a jump, in a fall, or on the ground as she sinks over a
  drop), or the ball on the ground meeting the floor by its vertical speed → only her vertical
  motion stops: the floor stops her move flush (`BlocksStopFlush`), her vertical speed is left as
  it is, and nothing else changes [TL1, TL2, TL3, TL4, BG1]
- Unlike the change of shape, in a turn in a jump or a fall the speed goes on growing by the pull
  each frame as in the air (`RisingAndFalling`) while the floor holds her, until the turn ends
  (`AnimationEnds`), and the ball's on the ground does so while it stays on the ground. Her
  horizontal motion goes on as `TurningRound` says.
- The turn's end handed over a fall or a jump → the next downward move that meets the floor lands
  her as above [record only: r02:37-38, r07:263-264]
- The ball landing at `vy ≥ 3.0000` with no bounce yet → it rises at the bounce speed (bounces 1)
  [C-bounce, C-bounce-pulled]
- The ball landing after bounce 1, at any speed (the 3 px gate is only bounce 1's) → it
  rises at the bounce speed less 1 px, `bounce − 1.0000`, `1.0000 − 1 = 0.0000` (bounces 2)
  [C-bounce-second]
- So bounce 1, rising at 1.0000, comes back at about 1 px a frame and still bounces again; bounce
  2 rises at exactly 0, so it rises by 0 for one frame before the rise ends
  (`RisingAndFalling`), and the landing after it brings her to rest at any speed.
- The spring ball landing with jump held → it hops again at the jump speed instead, bounces 0
  [original only]
- checked: oracle (r06 for the changing body, LB1, LB2; TL1-TL4 for a turn: TL1 a turn in a fall
  meets the floor, TL2 the next frame on it, its vertical speed still growing, TL3 a turn in a
  jump, TL4 the ground turn sinking; r02:37-38 and r07:263-264, where the turn's end hands over a
  fall or a jump and the next frame lands her); constructed (C-bounce, C-bounce-pulled, bounce 1
  read after the pull; C-bounce-second, bounce 2 at under 3 px; C-bounce-third,
  the landing after it; BG1, the ball on the ground held by the floor).

---

## 7. Pose changes: the pose chart

Which pose she shows next, from the pad, the items switched on and what her moves met, and what the change
does to her speeds. Written as data: a few tables keyed by what she is doing and what the player
holds or has just pressed. Vectors: the `[[pose_step]]` entries of the vector file.

### PoseFollows
- kind: mechanic and timing.
- summary: what decides her next pose, and when the change acts.
- params: none.
- switch: none.
- Her pose changes at most once a frame. What decides is the first that applies:
- An animation ending this frame → it hands over its pose (`AnimationEnds`) [P10, P11, P12, P19,
  P23]
- No animation ending, her moves met something that chooses a pose → that pose
  (`WhatHerMovesMet`, which says which wins when several apply) [P4, P5, P6, P7, P8]
- Neither → the pad chart's pose (`PadChart`), from the pose and the pad at the frame's start; its
  last row for her situation covers nothing fitting [P1, P3, P13]
- The frame runs in this order, and this is what the player can tell apart at its end: she moves as
  the pose at the frame's start says (`MovesAsShePosed`); what those moves met is known; the pose
  is chosen as above; then the change's effects act on her speeds and dash.
- A pose change → its effects act on her speeds and dash after that frame's moves, so they never
  alter them, and the frame ends in the pose chosen, with its effects applied [J1, J2, A1]
- One that comes lower in the list loses its pose and its effects to a higher one: a landing
  replaces the pad chart's outcome and its effect.
- An animation's end and a landing in the same frame → the end's pose, and the landing still stops
  her as `Landing` says [P23]
- A change that stops her horizontal speed dead → it takes her dash with it (dash 0) [original
  only]
- A change into `Morphing` (from a crouch or in the air) without `MorphBall` → it is refused: the
  frame ends in the pose she had, aim included, the pad's other rows don't step in, and none of
  the change's effects come: no lowering and no placement (`ShrinkingLowersHer`) [NM1,
  C-morph-no-ball-air]
- So crouched without the morph ball, down and forward pressed together, she stays crouched that
  frame, though forward alone would stand her up (NM1).
- checked: oracle (the `[[pose_step]]` vectors; NM1, a morph refused crouched); constructed
  (C-morph-no-ball-air, a morph refused in a straight jump).

### PadChart
- kind: mechanic.
- summary: the pad's rows, by what she is doing, and how a row is chosen.
- params: none (the rows are data).
- switch: none.
- Each row → fits when every button it names as just pressed was just pressed this frame and
  every button it names as held is held; other buttons don't matter [P1, P13, P21]
- Several rows of her situation fit, one needing a just-pressed button and one not → the one that
  needs it wins [P13, P17]
- Several fit, of the same kind → the one that names more buttons wins [P2, C-chart-a]
- A row whose Pad cell says "beats ..." fits along with the rows it names → it wins, where the
  ranks would leave two rows tied or where her situation goes against them [C-chart-a, C-chart-b,
  C-chart-c, C-chart-d, G7]
- No other row of her situation fits → its last row: its pose and its effect are her let-go
  outcome [P3]
- A fitting row whose next pose is "kept" → her pose stays as it is, apart from an aim the cell
  names, the Effect cell says whatever else happens, and the last row is not used [P15, R102]
- The rows' order in the table carries no meaning. "Aim" in a next pose means the aim the `Aiming`
  rule gives.
- Forward and back are by her facing at the frame's start (a turn already faces its new way).
  The chart covers pads with at most one of up and down held (no record holds both).
- A recorded pad can hold left and right together (r04:307-308). Rows naming forward and rows
  naming back then both fit, and the ranks and "beats" decide, with the further ranks below; how
  her motion reads both held is `SpeedBuildsAndBrakes`' and `DriftInTheAir`'s.
- Both held, standing still, landing or against the wall → back held beats forward held, and
  forward with up or down held beats the moonwalk's two-button row (back and shoot held; against
  the wall, back and an aim button held) [original only]
- Both held, running or spinning → forward held beats back held: she runs on, or keeps spinning
  [R102]
- Both held, taking off → jump and back held beats jump and forward held [original only]
- Both held, in a straight jump (aimed `Down` and with the gun drawn too) → back held beats
  forward with an aim button held, unless jump is held too [original only]
- Both held, moonwalking → the way she looks held beats the way she walks held: she runs
  [original only]
- Both held, a ball on the ground, with or without the spring ball → she rolls rightward
  [original only]
- Both held, falling as the spring ball → it turns round [original only]
- Both held, the spring ball on its hop → it turns rightward [original only]
- "Falling" and "on its hop" are its pose (`Falling`, `Jumping`), not the way it moves: a bounce
  keeps the pose it landed in (`WhatHerMovesMet`), so a spring ball rising from a bounce after a
  fall is still falling and turns round, and one bouncing after its hop is still on its hop.
- Both held, the ball in the air without the spring ball → it keeps its facing: forward held wins
  [original only]
- Both held, everywhere else, the ranks and "beats" decide alone: the outcome is always the one
  of one side held alone.
- A next pose → its gun is `Stowed`, unless its cell says "gun drawn" (she draws it) or "gun as it
  was" (drawn if it was drawn); "kept" keeps it as it was [G1, G2, G3, G4, G5, G6, G7, G9]
- So she draws it by holding shoot while running forward, jumping, falling or wall jumping, or by
  pressing shoot in a spin, where the ranks make that row the winner; any other outcome stows it:
  an aim, a turn, holding forward in a jump, a landing (except one that lands firing,
  `WhatHerMovesMet`), walking off a ledge, the end of an animation (a take-off ends in a jump with
  the gun stowed, whatever she holds). A Pad cell that starts "gun stowed:" fits only while her
  gun is stowed.
- A next pose → its legs are `Together` unless its cell says "striding"; "kept" keeps them as
  they were [L1, L2, P9]
- So she strides by holding forward in a take-off or a straight jump, and holding jump alone, or
  shoot, brings her legs together again.
- Rows are listed by rank (just pressed first, then by how many buttons they name), and inside a
  rank by next pose (action, then the first aim the cell names, alphabetically; "kept" last);
  where two rows still tie, the one that beats the other comes first. Each situation's last row
  is what happens when no other row fits.

| She is | Pad | Next pose | Effect |
| --- | --- | --- | --- |
| standing still, or landing | down pressed | crouched `Crouching`, crouch aim | she is lowered 5 px (`ShrinkingLowersHer`) |
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
| running | down pressed; beats jump pressed | crouched `Crouching`, `Forward` | she is lowered 5 px (`ShrinkingLowersHer`) |
| | jump pressed | `Spinning` | `Jumping`'s spin start |
| | forward and shoot held | `Running`, `Forward`, gun drawn | none |
| | forward and an aim button held; beats forward with up or down held, and forward and shoot held | `Running`, `UpForward` with aim up, else `DownForward` | none |
| | forward with up or down held; beats forward and shoot held | `Running`, `UpForward` with up, else `DownForward` | none |
| | forward held | `Running`, `Forward`, gun as it was | none |
| | up held; beats aim up or aim down held | upright `Still`, `Up` | none |
| | aim up or aim down held | upright `Still`, `UpForward` with aim up, else `DownForward` | none |
| | back held; beats up held and aim up or aim down held | `Turning` | `TurningRound` |
| | anything else, or nothing held | kept while her horizontal speed at the frame's start isn't 0, a choice that brings `AimingAndShooting`'s step; upright `Still`, `Forward` when it is 0 | as running (below) |
| taking off | jump and forward held | `Jumping`, `Forward`, striding | none |
| | jump with up, down or an aim button held; beats jump and forward held | `Jumping`, aim `Up` with up, else `Down` with down, else `UpForward` with aim up, else `DownForward` | none |
| | jump and back held; beats jump with up, down or an aim button held | `Turning` in a jump | `TurningRound` |
| | shoot held | `Jumping`, `Forward`, gun drawn | none |
| | anything else, or nothing held (jump alone too) | kept | as running |
| in a straight jump | aiming `Down`: down pressed | ball `Morphing` | she is lowered 9 px (`ShrinkingLowersHer`) |
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
| falling (upright) | aiming `Down`: down pressed | ball `Morphing` | she is lowered 9 px (`ShrinkingLowersHer`) |
| | forward with up or down held | `Falling`, `UpForward` with up, else `DownForward` | none |
| | shoot held; beats forward held | `Falling`, `Forward`, gun drawn | none |
| | forward held | `Falling`, `Forward`, gun as it was | none |
| | up, down or an aim button held; beats shoot held and forward held | `Falling`, air aim | none |
| | back held; beats shoot held, forward held, and up, down or an aim button held; aiming `Down` or with the gun drawn, up or down held beats it | `Turning` in a fall | `TurningRound` |
| | anything else, or nothing held | kept, aim `Forward` | dash 0 |
| crouched | down pressed; beats jump pressed | ball `Morphing` | she is lowered 9 px (`ShrinkingLowersHer`) |
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
| moonwalking | down pressed; beats jump pressed | crouched `Crouching`, facing the way she looks, `Forward` | she is lowered 5 px (`ShrinkingLowersHer`) |
| | jump pressed | `Turning` from the moonwalk's jump, facing kept (the way she walks), aim kept, then a spin the way she walks (`AnimationEnds`) | `TurningRound` |
| | the way she walks, shoot and an aim button held | `Moonwalking`, `UpForward` with aim up, else `DownForward`; with both aim buttons held, `UpForward` facing left and `DownForward` facing right | none |
| | the way she walks and shoot held | `Moonwalking`, `Forward` | none |
| | the way she looks held | `Running` that way, `Forward` | none |
| | the way she walks held | `Turning` | `TurningRound` |
| | anything else, or nothing held | upright `Still`, facing the way she looks | she follows the pad again |
| wall jumping | down pressed | ball `Morphing` | she is lowered 9 px (`ShrinkingLowersHer`) |
| | shoot held; beats jump held | `Jumping`, `Forward`, gun drawn | none |
| | aim up or aim down held; beats shoot held and jump held | `Jumping`, `UpForward` with aim up, else `DownForward` | none |
| | back held; beats every other row naming one button | `Spinning`, facing back | `TurningRound` |
| | jump held | kept | none |
| | anything else, or nothing held | `Spinning` | horizontal speed 0, dash 0, following the pad |
| against the wall | down pressed | crouched `Crouching`, `Forward` | she is lowered 5 px (`ShrinkingLowersHer`) |
| | jump pressed; beats down pressed | upright `TakingOff`, `Forward` | `Jumping`'s take-off |
| | back and an aim button held | `Moonwalking` facing back, `DownForward` with aim down, else `UpForward` | none |
| | forward with up or down held; beats down pressed | `Running`, `UpForward` with up, else `DownForward` | none |
| | forward held | `Running`, `Forward` | none |
| | up held; beats aim up or aim down held, and back held | upright `Still`, `Up` | none |
| | aim up or aim down held; beats forward held and back held | upright `Still`, `UpForward` with aim up, else `DownForward` | none |
| | back held | `Turning` | `TurningRound` |
| | anything else, or nothing held | kept, aim `Forward` | she follows the pad again (a skid ends) |


- Cells: standing still, or landing; running; taking off; in a straight jump; spinning; falling
  (upright); crouched; a ball, on the ground or in the air; turning on the ground or from the
  moonwalk's jump (not aimed `UpForward`); turning in a jump or a fall, crouched turning, or
  turning on the ground or from the moonwalk's jump aimed `UpForward`; changing body; moonwalking;
  wall jumping; against the wall; pressed; held; beats; anything else, or nothing held; an aim
  button; both aim buttons; up or down; aiming `Down`:; she faces `Right`:; aimed `Up` or
  `DownForward`:; aimed `Forward`:; gun stowed:; and no other button a row names; with the spring
  ball; on the ground; in the air; off the ground; the way she walks; the way she looks; any
  button held; kept; kept while her horizontal speed at the frame's start isn't 0; kept, aim
  `Forward`; landing, kept as she is; gun drawn; gun as it was; striding; facing back; the new
  facing; facing the way she looks; crouch aim; take-off
  aim; air aim; with, else; then a spin the way she walks; aim kept; the same ball facing that
  side; the ball off the ground, rising; she is lowered (`ShrinkingLowersHer`); `Jumping`'s
  take-off; `Jumping`'s spin start; `Jumping`'s spring-ball hop; `TurningRound`; none; she follows
  the pad again; as running; horizontal speed 0, dash 0, following the pad; dash 0.
- In a straight jump aimed `Down`, while she faces `Left`, forward, jump and shoot held →
  decided as in any straight jump, forward held winning (`Jumping`, `Forward`, striding): the
  row for those three fits only when she faces `Right`, and nothing takes its place the other way
  [C-chart-j, C-chart-i]
- A `Turning` cell gives the turn she is in (§2): `Ground` unless it says "in a jump", "in a
  fall" or "from the moonwalk's jump". Facing is kept unless the row says otherwise; `Turning`
  faces back (the turn shows her new facing, and `TurningRound` turns her), and keeps the aim she
  had (§2, aim in a turn). A moonwalk faces the way she walks, which is backwards: her sprite
  looks the other way. So the moonwalk's jump is the one turn that keeps her facing: it already
  points the way she walks, the way her spin then goes, and its turn shows her looking that way
  (MT1's first frame: the moonwalk and its turn face the same side).
- Body follows the action: `Running`, `Moonwalking`, `TakingOff`, `Jumping`, `Spinning`, `Falling`,
  `WallJumping`, `AgainstWall`, `Landing` and `StandingUp` are upright; `Crouching` and `Unmorphing`
  are crouched; `Morphing` is a ball (§2).
- A moonwalk's jump, whatever aim button is held → it turns with the moonwalk's own aim [original
  only]
- The let-go effect "as running" (running, taking off, a straight jump) is decided by her
  horizontal speed at the frame's start, like the running row's pose, and acts after the frame's
  move (`PoseFollows`).
- "As running", her horizontal speed at the frame's start not 0, and still not 0 after the
  frame's move → the dash joins her horizontal speed (dash 0) and she carries on, braking
  (`SpeedBuildsAndBrakes`) [AS1, AS2]
- "As running", her horizontal speed 0 at the frame's start, or brought to 0 by the frame's move
  → she follows the pad again, keeping the speed the move left her: running at 0 with nothing
  held, the frame's run gains its step (0.3000) and she stands with it, following the pad, not
  carrying on [AS3, AS4]
- checked: oracle (P1, P3, P13 to P18, P20 to P22, P24, P25, G1 to G9, L1, L2, U1, and R133 and
  R141 of `TurningRound`; R102 for left and right both held in a spin; AS1 to AS4 for the
  let-go effect, AS4 the run at 0 that stands following the pad); constructed (C-chart-a to
  C-chart-j); original only (the other rows with left and right both held).

### Aiming
- kind: mechanic.
- summary: the aim a next pose takes from the held buttons.
- params: none (the table is data).
- switch: none.

| Rule | Both aim buttons | Up | Down | Aim up | Aim down | Forward and up / down | Else |
| --- | --- | --- | --- | --- | --- | --- | --- |
| take-off | (as aim up) | `Up` | `Forward` | `UpForward` | `DownForward` | as up / `Forward` | `Forward` |
| standing | `Up` | `Up` | `Forward` | `UpForward` | `DownForward` | (running rows) | `Forward` |
| crouch | `Up` | (stands) | `Forward` | `UpForward` | `DownForward` | (stands) | `Forward` |
| air | `UpForward` | `Up` | `Down` | `UpForward` | `DownForward` | `UpForward` / `DownForward` | `Forward` |


- Cells: take-off; standing; crouch; air; up; down; aim up; aim down; forward and up / down;
  else; (as aim up); (running rows); (stands).
- Up and an aim button both held → up wins [P24]
- In the air rule, down and an aim button both held → down wins [original only]
- Up or down held with forward → the forward diagonal wins [P9, P20]
- checked: oracle (P2, P9, P24).

### WhatHerMovesMet
- kind: mechanic.
- summary: the poses that what her moves met chooses over the pad.
- params: none (the table is data).
- switch: none.

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
| no floor (walking off a ledge, `FollowingTheGround`) | upright or crouched on the ground | upright `Falling`, facing kept, `Forward` | `FollowingTheGround`'s fall |
| | a ball | ball `Falling`, facing kept | `FollowingTheGround`'s fall |
| a ceiling | rising | kept | `WallsAndCeilings` |


- Cells: a wall (a wall stop); a block or solid enemy in the way of the aim change's step; a wall
  near her as she presses jump; a floor (landing); no floor (walking off a ledge); a ceiling; in
  any pose, a running pose other than hers chosen; upright, in a straight jump or falling;
  spinning or wall jumping; a ball, bouncing; a ball, coming to rest; turning; upright or crouched
  on the ground; a ball; rising; facing kept; with the aim of the running pose chosen; (`Down` becomes
  `Forward`); `WallsAndCeilings`; `AimingAndShooting`; `WallJump`'s kick;
  `Landing`; `Landing`'s bounce; `Landing`: only her vertical motion stops; her vertical speed is
  kept; `FollowingTheGround`'s fall.
- A wall stop while running and walking off a ledge in one frame → the wall stop wins [original
  only]
- The aim change's step comes only when the pose chosen is a running one, so a landing or walking
  off a ledge, which choose a pose of their own, leave no step to block (`AimingAndShooting`). The
  wall-jump kick, a landing, walking off a ledge and a ceiling each come from her one vertical
  move or her kick, so no two of them meet her in the same frame.
- A landing while her body changes shape keeps the change running (the table's "kept"): its
  animation's end hands over the next pose (`AnimationEnds`, on the ground now; LB1, LB2, P23).
  When that end comes in the landing's own frame it decides the pose, as `PoseFollows` says.
- A landing while turning keeps the turn running the same way: its end hands over what
  `AnimationEnds` names for the turn (TP1, TP2, TP3).
- A fall or jump handed over then lands when a downward move next meets the floor (`Landing`).
- checked: oracle (P4, P5, P6, P7, P8; LB1 landing while `Morphing`, LB2 while `Unmorphing`;
  TP1 landing in a turn in a fall, TP2 in the ground turn as it sinks, TP3 in a turn in a jump;
  item-morph-ball:1264 and r06:16, where the landing and the animation's end share a frame);
  original only (`Crouching` and `StandingUp`, and the crouched and moonwalk turns: she never
  lands in them in a record).

### AnimationEnds
- kind: mechanic.
- summary: the poses that last as long as their animation, and what their end hands over.
- params: none (the table is data).
- switch: none.
- Animation timing is out of scope (§10): a vector says on which frame it ends.
- A pose that lasts only as long as its animation, its animation ending → its end hands over the
  next pose and effect below, whatever the pad [P10, P11, P12, P19, P23, M1]

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
| `Unmorphing` | crouched `Still`, on the ground or in the air | none |


- Cells: aim kept; the new facing; aimed `Forward`, striding; on the ground; in the air; on the
  ground or in the air; with the spring ball, the spring ball's; gun stowed; dash 0; `Moonfall`.
- The unmorph ending in the air → she is crouched in the air, a pose on the ground: her next frame
  is the crouch's, the ground's pull with her vertical speed kept, and, with no floor within the
  pull, it walks her off a ledge (`FollowingTheGround`) [C-unmorph-air-end, C-ground-rise]
- So an unmorph that ends in a fall has one crouched frame and then falls from rest (record only:
  boss-kraid-defeated 779-782), and one that ends in a rise keeps the rise (item-charge-beam 5-8,
  item-reserve-tank 519-521).
- checked: oracle (P10, P11, P12, P19, P23, M1); constructed (C-unmorph-air-end).

### Out of the chart's scope
- The spring ball's rows, the grapple, knockback, the boost after damage, x-ray, shinespark, the
  crystal flash, the drain, liquids, and items switched on or off mid-pose.

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
- kind: mechanic.
- summary: a move into a solid block stops her flush against its face.
- params: the horizontal move's limit, 15 px.
- switch: none.
- Her leading edge at the target, on the move's axis (x and `rx`, or y and `ry`): `x + d + rx − 1`
  moving right or down, `x + d − rx` moving left or up, `x + d` being the pixel part of her
  position plus the whole move, subpixels added, so a subpixel part that carries into a pixel
  moves the edge a pixel on.
- A move right or down into a solid block → it stops her with her leading edge against its face,
  that axis's subpixel FFFF [C-wall, C-step-down, PF2, R2]
- A move left or up into a solid block → it stops her with her leading edge against its face,
  that axis's subpixel 0 [C-ceiling, C-step-left, C-step-up]
- A move tests only the row or column of her leading edge at the target (`TouchedCells`), so a
  move of 16 px or more can pass a floor.
- A horizontal move → it is limited to 15 px either way, `min(|d|, 15)`, its subpixel kept
  (unreachable in vanilla motion) [C-limit]
- A move into a stepped block, on either axis → it stops her the same way, with the same
  subpixels (FFFF or 0), but against the face of the 8 px quarter her leading edge reaches, not
  the block's face: the quarters that count are those in the half her leading edge reaches (a
  side's half for a sideways move, an upper or lower half for a vertical one) that her body
  overlaps across it [C-step-far, C-step-left, C-step-down, C-step-up, C-span-step]
- So reaching the far half of a block with both halves solid, she stops at the far half's face,
  her body overlapping the near half.
- A stop whose face is behind her edge at the start → it never moves her backwards: she moves 0
  px, and only the subpixel is written (FFFF or 0), which is forward [C-step-noback]
- So a move of any length above 0, even 0.0001, whose leading edge starts inside a solid block or
  quarter is stopped there: a downward one lands her.
- A move of exactly 0.0000, on either axis → blocks never stop it: her position, subpixel
  included, is unchanged, and it doesn't end her seat on a slope (`SlopesCarry`), also with her
  leading edge already inside a solid block, quarter or sloped surface [C-step-far, C-step-noback,
  C-span-step]
- As in the first frame of a fall from rest: she falls on, ending at +0.1C00.
- Blocks across her body would stop a horizontal move at different places → the topmost of them
  decides, which need not be the nearest [C-span-step]
- Blocks across her body would stop a vertical move at different places → the leftmost decides on
  an `Even` frame and the rightmost on an `Odd` one [C-down-even, C-down-odd]
- Air, and a sloped surface whose stop she doesn't reach, never decide.
- checked: oracle (every ground vector's floor); constructed (C-limit, C-step-far, C-step-noback,
  C-step-left, C-step-down, C-step-up, C-span-step, C-down-even, C-down-odd).

### WallsAndCeilings
- kind: mechanic.
- summary: a wall halts her; a ceiling ends her rise.
- params: none.
- switch: none.
- A horizontal move stopped by a wall, on the ground or in the air → it halts her: horizontal
  speed 0, dash 0, following the pad [C-wall, C-step-far, C-step-left]
- A wall stop on the ground → the ground pull that frame still uses the speed from before the
  stop [C-wall]
- An upward move into a block → it stops flush and ends the rise: vertical speed 0, falling, with
  no pull that frame [C-ceiling, C-step-up]
- checked: constructed (C-wall, C-ceiling).

### SolidEnemies
- kind: quirk.
- summary: how solid enemies stop her, judged by the gap between the boxes.
- params: the reach's 1 px.
- switch: none.
- Her box and an enemy's are each centre ± half-size.
- Moving left or up with the move's end subpixel not 0, or moving right or down with her start's
  subpixel or the move's not 0 → her reach is her box at the move's end pushed 1 px further on;
  otherwise it is her box at the move's end [C-enemy-ahead, C-enemy-down, C-enemy-adjacent]
- There, the centres closer than the two half-sizes summed on both axes, `|x − ex| < rx + erx`
  and the same for y → she reaches the enemy [C-enemy-ahead, C-enemy-down]
- An enemy she reaches is judged from where she starts, by the gap between the boxes on that
  side: the enemy's near side less her near side, the sides being the boxes' edges at centre ±
  half-size, so boxes whose pixels are next to each other have a gap of 0 (downward, her bottom
  side against the enemy's top side, `gap = (ey − ery) − (y + ry)`; the other ways alike).
- A gap of 0 → she is touching: she doesn't move, and her **y** subpixel is set to 0, on either
  axis [C-enemy-adjacent, W13, W20]
- A larger gap → it stops her by that many whole pixels, her subpixel kept [C-enemy-ahead,
  C-enemy-down]
- A negative gap (already overlapping) → it never stops her: the move goes as if that enemy were
  not there, so another solid enemy in its path decides it, or with none, blocks do
  (`BlocksStopFlush`) [C-enemy-overlap]
- So a move of 0 is never shortened by one.
- So standing on a solid enemy, with no gap between them, each frame's ground pull reaches it and
  stops at once, so she stays on it (a landing, `Landing`), her y subpixel 0 (W20). A landing on
  one stops her with her subpixel kept, and when the landing pose's taller body is then fitted
  (`PoseFitAfterMoves`), its look down is a test like any move's, so it finds no gap below her,
  and her y subpixel becomes 0 before the push (W13).
- A stop against an enemy → it is a wall, a ceiling or a floor as the move's direction says:
  sideways it halts her (`WallsAndCeilings`), upward it ends the rise, downward she lands
  (`Landing`) [C-enemy-ahead, C-enemy-down]
- A solid enemy in a move's path → it decides that move alone: she stops against it and no block
  stops her, even a nearer one, so the move can end with her edge overlapping a block; blocks stop
  her only on a path free of solid enemies (`BlocksStopFlush`) [original only]
- checked: oracle (W13, a landing on one 2 px below and the fit's look touching it; W20,
  running on one); constructed (C-enemy-ahead, C-enemy-adjacent, C-enemy-down, C-enemy-overlap,
  C-aim-enemy, C-aim-enemy-stop).

### SlopesCarry
- kind: mechanic.
- summary: how sloped floors and ceilings seat her, lift her, push her down and stop her.
- params: none (the shapes are world data).
- switch: none.
- On a slope → the ground pull is 1 px instead of her speed plus 1 px (`FollowingTheGround`)
  [C-slope-turn-walk, C-seat-lift]
- Seated, her next vertical move of more than 0 starts → she is no longer on a slope [C-seat-lift,
  C-seat-none]
- A move of exactly 0 leaves her seated (`BlocksStopFlush`). Three things seat her: the check
  after every horizontal move (below), a downward move stopped by a stepped block, and a look down
  that a stepped block stops. So a frame's pull is 1 px when that frame's horizontal move seated
  her, or the previous frame's pull or look down stopped on a stepped block.
- A downward move, or a look down (`PoseFitAfterMoves`, `ShrinkingLowersHer`), stopped by a
  stepped block → it seats her [C-step-down, C-down-odd]
- A stepped block, like a solid one, is met in every column her body spans and stops her flush
  with subpixel FFFF (`BlocksStopFlush`).
- After every horizontal move, whether or not it met solid enemies, two checks run, each asking
  just one cell and only on a sloped surface (a stepped block takes no part in them).
- **Floor.** The block at her centre column and her bottom edge's row is a sloped floor, and her
  bottom edge is in its solid part, `bottom ≥ top`, the surface's top row at her centre column →
  she is lifted by whole pixels until her bottom edge is on the last air row above the surface,
  `top − 1`, and that lift seats her [C-seat-lift]
- With her bottom edge already on that last air row, or higher, nothing happens and she is not
  seated, so that frame's pull is her speed plus 1 px (C-seat-lift against C-seat-none).
- **Ceiling.** Then, the block at her centre column and her top edge's row is a sloped ceiling,
  and her top edge is in its solid part or on the first air row under it, `top edge ≤ bottom + 1`
  → she is pushed down by whole pixels until her top edge is on that first air row, and the push
  seats her, also when it moves her 0 px because her top edge was already there, as in
  samus-walljump-16:44, -22:44 and -28:44 [C-slope-ceiling-push]
- A horizontal move meeting a sloped surface → it is never stopped by it (how a sloped floor
  changes the move's length is `SlopesSlowTheWalk`) [C-slope-turn-walk, C-slope-walk-left]
- A vertical move meeting a sloped surface that can't stop it → it passes as through air: a
  sloped floor or ceiling in another column her body spans than her centre's (only the block at
  her centre column counts), a sloped floor on a rise, a sloped ceiling on a downward move, or a
  sloped floor at her centre column whose last air row her bottom edge at the target doesn't
  reach, b < t − 1 (below) [C-slope-edge, C-slope-ceiling-side, C-slope-roll-carry,
  C-slope-run-off]
- **A downward move onto a sloped floor.** Where it ends is found from her position plus the
  whole move, subpixels added (`BlocksStopFlush`); `b` is the row in the block of her bottom edge
  there, `t` the surface's top row at her centre column (t − 1 is the last air row above it), and
  `d` the move's pixel part.
- Her bottom edge at the target on the last air row or lower, `b ≥ t − 1` → the stop is by whole
  pixels: she moves `max(0, d + t − 1 − b)` px, the move's subpixel part lost and her y subpixel
  kept, not set to FFFF, whatever the surface's shape; she lands (`Landing`, or on the ground the
  pull's flush stop), and this stop doesn't seat her [C-slope-land, C-slope-edge,
  C-slope-roll-carry, C-slope-roll-carry-next, C-slope-carry-kept]
- So with her y subpixel and the move's subpixel part adding to less than a pixel she stops with
  her bottom edge on the last air row (C-slope-land), and a move that would end exactly on it
  already stops there.
- So when they carry into a pixel, the target's row is one lower and she stops 1 px higher, her
  bottom edge on the row above the last air row: the carried pixel moved her target's row, not her
  (C-slope-roll-carry, C-slope-roll-carry-next, C-slope-carry-kept, C-slope-walk-left). A run or
  a roll down a slope from y subpixel
  FFFF, with a pull that has a subpixel part, stops so: samus-mockball 91 (the spring ball's pull of 9.4000 from
  04B9.FFFF stops at 04BD.FFFF, its bottom edge on row 3 above a surface whose top row is 5), and
  the same in the runs of samus-mockball and the four samus-shinespark rows with a kept vertical
  speed, and in chain-0220-D017 39-42 with none. Seated, the pull is 1.0000, which never carries.
- So with her bottom edge already below the last air row (inside the surface's solid part) the
  move never takes her up: she moves 0 px and lands. That needs a move with no horizontal move
  before it in the frame (the elevator's 1 px, a look): after a horizontal move the floor check
  above has already lifted her out of the surface.
- A sloped floor at her centre column and a solid or stepped block in another column of that row
  → which of the two decides is the order of `BlocksStopFlush` [C-down-odd, C-down-even]
- Rising so that her top edge at the target reaches a sloped ceiling's solid part at her centre
  column, or the first air row under it → she doesn't move up at all that frame (her subpixel
  kept), and the rise ends as at a flat ceiling (`WallsAndCeilings`) [C-slope-ceiling-edge]
- The surface's height at each column is world data; the vectors give it as `world.slope_floor`
  and `world.slope_ceiling`.
- How a model holds it is a representation, and either side will do. A surface can as well be
  held as its depth from the block's open side: a floor's top row as it is, a ceiling's bottom
  row counted up from the block's bottom row (15 less it), with 16 (an empty column) and 17 (a
  column far past) the same for both. The last air row above a floor is then one less than the
  depth, counted from the top, and the first air row under a ceiling one less than the depth,
  counted from the bottom, so every rule here reads the same, the empty and far columns
  included; nothing changes but where the vectors' numbers are converted.
- **An empty column.** A column of a sloped surface holding none of the surface (the whole column
  is air in that block; the vectors write it as 16, for a floor and a ceiling alike) → it counts
  as a surface just past the block's far edge [C-slope-floor-empty, C-slope-ceiling-empty,
  C-slope-ceiling-empty-on]
  - So, as a floor, the last air row above it is the block's bottom row. A downward move whose
    bottom edge at the target is on that row stops and lands her there, as above; one that ends
    higher passes. The check after a horizontal move never lifts her.
  - As a ceiling, the first air row under it is the block's top row. A rise whose top edge at the
    target is on that row doesn't move her and ends the rise, as above; one whose top edge ends
    on any lower row of the block moves on. The check after a horizontal move, with her top
    edge on that row, pushes her 0 px and seats her.
- **A column far past.** A column whose surface lies more than one row past the block's far edge
  (air in that block, as an empty one is; the vectors write it as 17, for a floor and a ceiling
  alike) → it never meets her, in either direction: as a floor it never stops a downward move,
  whatever row of the block her bottom edge reaches, and the check after a horizontal move never
  lifts her; as a ceiling it never stops a rise, whatever row of the block her top edge reaches,
  and the check after a horizontal move never pushes her; it never seats her [C-slope-floor-far,
  C-slope-ceiling-far]
- So that frame's pull is her speed plus 1 px, unless something else seated her. A sloped shape
  that rooms use has such columns across half its width.
- checked: oracle (samus-walljump-16/22/28 frame 44; samus-walljump-28:62 and
  samus-screw-attack:66 for the sloped ceiling; chain r01:677, samus-walljump-22:93 and
  samus-input-poses:426 for the landing that keeps the subpixel); harness (SL1 and SL2,
  samus-mockball 91 and 92, the stop a pixel above after a carry, and SL1 a rectangle in another
  column passed; SL3, chain-0220-D017 43, a pull that ends above the surface's last air row passes
  and she walks off); constructed (C-slope-roll-carry and C-slope-roll-carry-next, SL1's and
  SL2's rules over an invented room, C-slope-run-off, SL3's, C-slope-land, C-slope-edge,
  C-slope-ceiling-edge, C-slope-ceiling-side, C-slope-ceiling-push, C-slope-carry-kept, C-seat-lift, C-seat-none, C-step-down, C-down-odd,
  C-aim-slope, and for the empty column C-slope-ceiling-empty, C-slope-ceiling-empty-on,
  C-slope-floor-empty, and for the column far past C-slope-floor-far, C-slope-ceiling-far).

### SlopesSlowTheWalk
- kind: mechanic.
- summary: a sloped floor shortens the move she makes along it.
- params: the walk factor, world data: one of 1.00, 0.F0, 0.D8, 0.C0, 0.B0, 0.80 and 0.50 for
  each sloped floor; the vectors give it as `world.slope_floor.factor`.
- switch: none.
- A horizontal move made while her vertical speed is exactly 0 (standing, running, turning,
  rolling, the first frame of a fall from rest, or after a rise ends at the frame's start), whose
  leading edge's column meets a sloped floor → it is shortened by that floor's walk factor `f`,
  exactly: the move is cut to a multiple of 0.0100 (1/256 px), rounding towards minus infinity,
  only the pixel part's value mod 256 kept, then its length multiplied by the factor (no bits
  lost) and the direction put back, `sign(d) × (|cut(d)| mod 100.0000) × f` [C-slope-turn-walk,
  C-slope-fall-walk, C-slope-walk-left]
- So a leftward move's length rounds up. 1.7000 at 0.C0 is 1.1400, 4.C000 at 0.D8 is 4.0200, and
  0.C000 at 0.C0 is 0.9000; a factor of 1.00 still cuts the move to a multiple of 0.0100.
- A sloped ceiling never shortens the move, nor does a sloped floor with any vertical speed but 0
  (C-slope-walk-ceiling, C-slope-walk-air).
- The cells that count are those `TouchedCells` names for the move: the leading edge's column at
  the move's unshortened target, in each row her body spans; each sloped floor among them → it
  shortens the move again, top row first [original only]
- A solid block below it in that column still stops her as `BlocksStopFlush` says.
- Her speeds stay as they are: only that frame's move is shorter (C-slope-turn-walk,
  C-slope-fall-walk).
- The aim change's step (`AimingAndShooting`) is a horizontal move too: on a 0.C0 floor it moves
  her 0.C000.
- checked: harness (S1 samus-input-poses:512, S2 r01:717; the moves of r01:625-630 at 0.C0 and of
  r06:123-126 at 0.D8, aim steps included); constructed (C-slope-turn-walk and C-slope-fall-walk,
  S1's and S2's rules over an invented room, C-slope-walk-left, C-slope-walk-air,
  C-slope-walk-ceiling).

### TouchedCells
- kind: output.
- summary: which cells her moves and looks reach, and which she is inside, in order.
- params: none.
- switch: none.
- Some cells react when she reaches them (one that changes the scrolling, an item, a door, a floor
  that crumbles); what they then do belongs to their own domains. Motion reports which cells she
  reaches, in order, so the world can raise those reactions. Two reports come out of a frame.
- **The cells her moves and looks reach.**
- Each move of the frame (the horizontal move, the near-wall test of `WallJump`, the ground pull or
  her vertical move, the aim change's step, the elevator's 1 px), then each look of the pose change
  (pose fit's block looks, `PoseFitAfterMoves`, in the order given there, or the look of
  `ShrinkingLowersHer`) → in the order they happen, the cells of her leading edge's line at the
  move's target, the line `BlocksStopFlush` names [C-touch-run, C-touch-wall, C-near-wall-item]
- A horizontal move → the column of her leading edge at the target, in each row her body spans
  where she stands before the move, top row first [C-touch-run, C-touch-wall]
- A vertical move → the row of her leading edge at the target, in each column her body spans
  where she stands before the move, left to right on an `Even` frame and right to left on an
  `Odd` one [C-touch-fall-odd, C-touch-run]
- Each cell → it comes with its way: one of four directions for a move (down, left, right or up),
  or `Look` for a look, because some reactions depend on it [C-touch-run, C-fit-ceiling,
  C-shrink-floor, C-near-wall-item]
- The ways are listed alphabetically; the order means nothing. The near-wall test, pose fit's
  looks and the shrink's look are looks, not moves: they don't move her.
- The near-wall test → its cells are those of a horizontal move of 8 px towards the wall from
  where her horizontal move left her [C-near-wall-item]
- A vertical look → its cells are those of a vertical move of its distance, an 8 px look's cells
  before its own look's where the 8 px rule applies (`PoseFitAfterMoves`) [C-fit-ceiling,
  C-fit-step-ceiling, C-shrink-floor]
- A reaction that depends on the way she moves (a floor that crumbles, a station she enters from
  one side, a save point's access) → it doesn't answer a look; the others (an item, a
  cell that changes the scrolling, a door to a room, spikes) do, as for a move [C-near-wall-item]
- The world answers a reacting cell's solid-or-air for a look as for a move (`OtherBlockKinds`).
- The first cell that stops her → the report ends there: the cells after it on the line are not
  reached [C-touch-wall, C-kind-spikes, C-kind-reacting-solid]
- A move of exactly 0, or a move or a look that a solid enemy decides (`SolidEnemies`) → it
  reaches no cells: pose fit's enemy looks reach none [C-touch-inside-twice]
- A cell that stands for another (`OtherBlockKinds`) → it is reported as the cell it stands for
  [C-kind-stand-in]
- **The cells she is inside.** Once each frame, before her moves, from her position at the frame's
  start:
- Each frame → first the cell in her centre column at her bottom edge's row [C-touch-run,
  C-touch-inside-twice]
- Her centre's row `≠` her bottom edge's row → then the cell at her centre's row
  [C-touch-inside-twice, C-touch-wall]
- Her top edge's row `≠` her bottom edge's row, and the three rows' numbers combined
  `top xor centre xor bottom ≠ 0` → then the cell at her top edge's row [C-touch-inside-twice,
  C-touch-wall]
- So when the rows of her top edge and her centre are one, and her bottom edge's another, that
  cell is reported twice.
- checked: harness (W15: the near-wall test reaches item-morph-ball's item cell, and the game
  picks the item up); original only (area-brinstar's scrolling cells; W15's pose-fit looks);
  constructed (C-near-wall-item, W15's cells over an invented room, C-touch-run, C-touch-wall,
  C-touch-fall-odd, C-touch-inside-twice, the C-kind vectors, and the looks' cells in
  C-fit-ceiling, C-fit-step-ceiling and C-shrink-floor).

### OtherBlockKinds
- kind: mechanic.
- summary: how the remaining kinds of cell meet her motion.
- params: none (the table is data).
- switch: none.
- A cell of one of the kinds below → it is solid (it stops her as a solid block does,
  `BlocksStopFlush`), air (it never stops her), or decided by the cell's own reaction this frame,
  as the table says, and is reported as `TouchedCells` says, whatever it does [C-kind-spikes,
  C-kind-door, C-kind-reacting-air, C-kind-reacting-solid]

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


- Cells: air; solid; air: the move goes on, and her passage to that room starts; solid, unless
  its reaction lets her through this frame; solid or air, as its reaction answers this frame;
  like the cell it names, in its own place; one that names nothing is air.
- A cell that stands for another → it names a cell some number of cells away along its row or its
  column, and meets her exactly as that cell's kind and shape would if it were here: the stop and
  the slope's shape are at this cell's position; a chain of such cells is followed to its end
  [C-kind-stand-in]
- A reacting cell → the world answers its solid-or-air for each touch, and the answer may change
  from one touch to the next (a floor that crumbles holds her, then falls away)
  [C-kind-reacting-air, C-kind-reacting-solid]
- checked: original only; constructed (C-kind-spikes, C-kind-door, C-kind-reacting-air,
  C-kind-reacting-solid, C-kind-stand-in).

---

## 9. Numbers

- `below`, on 16-bit parts, is defined in `WordwiseTopSpeed`.
- Adding `P.S` carries the subpixel into the pixel part, mod 2^16 on the pixel part.
- A vertical move uses the vertical speed from before that frame's gravity (`RisingAndFalling`);
  a halt's speeds are those after that frame's moves.

### WordwiseTopSpeed
- kind: quirk.
- summary: the top speed is tested part by part.
- params: none.
- switch: **Toggle**, `Wordwise` (vanilla) or `Exact`.
- `below(a, b)` on 16-bit parts: bit 15 of `(a − b) mod 2^16` is set. 1000 is not below C000;
  C000 is below 4000 and below 0. `Dash` uses it too.
- The test is on the speed's size: `p.s` hers, `tp.ts` the top's.
- `Wordwise`, `below(p, tp) or (p = tp and (s = ts or below(s, ts)))` → she keeps her speed
  [R2, R61, C-cap-exact, R94]
- `Wordwise`, any other speed → she takes the top speed [C-cap, R100, R134]
- So 2.1000 and 2.0000 snap up to 2.C000, 3.C000 is kept over 3.4000, 0.C000 is kept over 0, and
  2.2000 snaps down to 1.4000.
- `Exact` → the smaller of the two, `min(speed, top)` [C-cap-exact]
- checked: oracle (`Wordwise`, the running vectors); constructed (C-cap, C-cap-exact,
  C-aim-enemy-stop).

## 10. Out of scope, and what remains

- **Slopes:** the corners of steep blocks.
- **Liquids and the speed booster:** other speeds and gravity in water and lava; the booster's dash
  cap of 7 and its higher jump.
- **Pushes:** room pushes, quicksand, knockback, bomb jumps, grapple, shinespark.
- **Her outputs under a push.** Where a push moves her in a frame, pose fit and the shrink still
  place her as `PoseFitAfterMoves` and `ShrinkingLowersHer` say, and the placement is
  still theirs, but from where the push left her, and the push decides what they meet: in
  terrain-quicksand the sand carries her 1.C000 down, and the look down meets the sand at once, so a
  body 7 px taller is pushed up 7 px (62, 75), and it holds her up so that the crouch she starts
  lowers her 5 px (151, 389). Without the push she meets open air there: the body grows about its
  centre, or she walks off. So her y and the placement are the push's on those
  frames, with the push.
- **A knockback's end.** After a hit's push back she falls in a jump pose with no falling limit, as
  in the moonfall, while the knockback's time runs on. Those frames are her own motion, in scope:
  `Moonfall`'s rule for her fall, `RisingAndFalling` for whether she rises. The first frame after
  its time has run out ends the knockback, wherever she is, and that frame is the knockback's,
  outputs and all: whatever her motion did, she keeps her pose and is handed back falling at 0
  (§1.2), with no change of body to fit (r05:400, where the floor meets her on that frame and a
  landing would raise her 2 px). The knockback's own push, and her rise on the frame it starts (the
  records show her rising there), are the knockback (damage) domain's too, as her going backwards
  is.
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

1. "A spin she was already in two frames earlier" (`WallJump`) is not yet traced against a record: it
   may count pose changes rather than frames.
2. Answered: the pose chart's steps are §7's `pose_step` vectors. In the motion vectors her pose
   changes by a jump, a fall, a turn from the ground or a spin reversed; for pose fit, by a
   landing, a take-off, standing up and unmorphing (PF1-PF3, U1, the C-fit vectors); and for
   `ShrinkingLowersHer`, by starting to crouch or morph (SH1-SH3, C-shrink-floor).
3. `RidingTheElevator` and stepped blocks have constructed vectors only: no recorded frame is
   clean for them (`SolidEnemies` now has W13 and W20). `SlopesCarry`'s ground pull onto and
   past a sloped floor has harness cases (SL1-SL3) and constructed vectors over invented rooms
   (C-slope-roll-carry, C-slope-roll-carry-next, C-slope-run-off); its other rules are
   constructed. The slope
   landings in r01 and samus-walljump-22 need lead-ins through a spin's drift and a sloped floor's
   effect on a move's length (`SlopesSlowTheWalk`).
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
17. The looks' cells (the near-wall test's, pose fit's and the look of `ShrinkingLowersHer`) are
    reported in `TouchedCells` with the way `Look`. Open is only how the doors' and damage's
    domains answer a `Look` cell: in the original a door to a room starts her passage and spikes
    hurt her from one, as from a move, she takes an item and a scrolling cell acts, while
    reactions that depend on the way she moves do nothing. The motion vectors check the cells
    (C-near-wall-item, C-fit-ceiling, C-fit-step-ceiling, C-shrink-floor), not what the doors and
    damage do.
18. A look up of pose fit whose row holds a sloped floor (a ledge's slope above her head): the
    original tests it as it tests a sloped floor below her, which can give a room no look down
    would. No record or vector meets one; open.
19. Left and right held together (`PadChart`): only a spin is on record (R102, r04:308). The other
    situations' ranks with both held rest on the original; no vector holds both there yet.
20. The morph's end (`AnimationEnds`): the original hands over the ball `Still` when her vertical
    speed is 0 and the ball `Falling` at any other, wherever she is. The table's "on the ground"
    and "in the air" give the same pose except at a vertical speed of 0 in the air (the morph
    ending on a rise at 0 or on a fall's first frame from rest), where the original's ball is
    `Still` and then walks off. No record or vector has one; open.
