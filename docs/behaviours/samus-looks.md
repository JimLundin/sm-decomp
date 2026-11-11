# How Samus looks, part 1, her animation's timing: behaviour spec

What the player sees of Samus's animation from one frame to the next: which step of her pose's
clip shows and for how long, how a clip loops, branches and ends, where the clip of the pose she
takes starts, what her moves do to it (a shot from standing, readying a wall jump, a fast fall, a
straight jump's rise), and her run: its pace while she dashes and the speed booster's stages up
to boosting. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-looks.toml`](../../scenarios/vectors/samus-looks.toml).

## Header

**Scope.** Part 1 of the domain "how Samus looks": her animation's timing. One domain with her
sprite and colours (part 2), because the player sees one thing, her figure; this part is the
clock that the sprite reads. It covers the step her clip shows, the time left on it, the end of
a clip (which [samus-motion](samus-motion.md) reads as `AnimationEnds`), whether her spin has
turned far enough to kick off a wall (samus-motion's input "spin turned far"), and the boost: her
run's stages up to boosting, which the shinespark, the echoes, her glow, the blocks that only a
boost breaks, and the enemies she touches read.

**What it reads** (§1.3, each from the domain that owns it): her pose at the frame's start and at
its end, and what her moves did this frame (samus-motion); her clips and the run's paces (the
importer, §1.4); her energy and the items switched on (progress); the pad's run button; the
slowdown and whether she is submerged (the liquids); how many sounds are waiting (the sound
domain).

**What it leaves to others**: which pose follows a clip's end (samus-motion, `AnimationEnds`);
which picture each step shows, where it is drawn and in which colours, the flicker while she is
invincible, the splashes and dust (part 2, §7); the poses other domains drive (§7); whether a
sound it asks for is heard (the sound domain).

**Notation.** A clip's steps are numbered from 0. "The clip moves on" means it goes to the step
after the one it is on. A hold is a number of frames.

---

## 1. What the player can tell about her animation

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| her step | a step of her clip, from 0 | the picture she shows (part 2) | `ClipPlays`, `StartingClip`, `SpinTurnedFar`, `FastFall`, `RisingHold`; output (her sprite) |
| the time left | frames, 0 to 255: how long her step still shows | when the picture changes | `ClipPlays`, `RisingHold`, `StartingClip` |
| the boost stage | 0 to 4; 4 is boosting | her run's pace, and once boosting her glow and echoes | `RunPace`, `BoostStages`; output (the shinespark, the echoes, her glow, the blocks a boost breaks, her contact with enemies) |
| the laps left | 0 to 2: the laps of her run the stage still lasts | when the pace next quickens | `BoostStages` |

These four are all the domain remembers; each is State a caller can set to any value the spec
allows and read back. A caller keeps them from one frame to the next and gives them back as they
were left: nothing else carries over, since the end, where the clip of the pose she takes starts,
the slowdowns and what her moves did are each worked out within the frame. A vector gives them
as `before` and compares them as `after`.

### 1.1 Outputs

| Output | Type | Read by |
| --- | --- | --- |
| end | the end her clip reaches this frame (`ClipEnds`): none, an end, or an end unless a jump; known from the frame's start, before samus-motion picks her pose (§6) | samus-motion (`AnimationEnds`, where an end unless a jump gives way when its pad chart's pose that frame is a take-off aimed forward or a spin) |
| turned far | `bool`, at the frame's end (`SpinTurnedFar`) | samus-motion (`WallJump`), the next frame |
| sounds | the sound asked for this frame, at most one (a frame passes at most one control): `PlainSpin`, `SpaceJump`, `ScrewAttack` (`WallJumpBranch`), `Boosting` (`BoostStages`) | the sound domain |
| her step and the boost stage | as §1 | part 2 (her sprite, her glow), the shinespark, the echoes, the blocks, the enemies |

### 1.2 Her pose

Her pose is samus-motion's (§2 there: body, action, facing, aim, gun, legs) with the sprite
`HowSheIsDrawn` chooses for it (plain, the screw attack's, the space jump's, the other suit's):
each pose and sprite has its own clip. She takes another pose when either part differs.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| her pose | §1.2, at the frame's start | every entry |
| her pose after | §1.2: the pose she ends the frame in (samus-motion's choice) | `StartingClip`, `SpinTurnedFar` |
| her clips | content, by pose (§1.4) | every entry |
| the run's paces | content (§1.4) | `RunPace`, `BoostStages` |
| energy | a count | `Breathing` |
| items switched on | `SpeedBooster`, `SpaceJump`, `ScrewAttack`: progress's own output | `RunPace`, `BoostStages`, `WallJumpBranch` |
| submerged | `bool`: she is under a liquid's surface, the liquid slows her, and she lacks the gravity suit (the liquids) | `WallJumpBranch` |
| slowdown | frames, 0 to 3: added to the holds her clip moves on to: 3 in water, whatever her suit; 2 in acid, whatever her suit; 2 in lava without the gravity suit, 0 with it; 0 out of a liquid (the liquids) | `ClipPlays`, `ClipControls`, `Breathing`, `WallJumpBranch`, `HandOverSlowdown`, `RunPace`, `BoostStages` |
| the start's slowdown | frames, 0 to 3: added to the first hold of the pose she takes: 3 with the feet of that pose under water, 2 under lava or acid; 0 with the gravity suit, out of a liquid, or where the liquid doesn't slow her (the liquids) | `StartingClip`, `HandOverSlowdown` |
| run held | `bool`: the pad holds run this frame | `RunPace`, `BoostStages` |
| handed over | `bool`: samus-motion took the pose her clip's end handed over this frame (samus-motion's `handed over`, its §1.4) | `HandOverSlowdown` |
| dashing | `bool`: she is dashing as the clip moves on (samus-motion's `dashing after her moves`, its §1.4) | `RunPace`, `BoostStages` |
| dash began, dash lost | `bool` each: a dash began this frame, before the clip moves on; it was lost this frame (samus-motion's `dash began` and `dash lost`) | `BoostStages` |
| standing restarts | `bool`: samus-motion's "the standing animation restarts" (`AimingAndShooting`) | `StandingRestart` |
| wall jump readied | `bool`: samus-motion's near-wall test readied a wall jump (`WallJump`) | `WallJumpReady` |
| her vertical speed | signed `P.S`, pixels a frame, downward positive: samus-motion's `vertical speed after her moves` (its §1.4), as her moves leave her, before what they met acts and before the pose change. An input of its own, apart from heading down: she can rise fast in a fall's pose, and the moonfall and a rise a ceiling stops are not heading down whatever the speed | `FastFall` |
| heading down | `bool`: samus-motion's `heading down` (its §1.4), as her moves leave her, before what they met acts and before the pose change: she falls, a fall at 0 included, and not in the moonfall; a rise a ceiling stops this frame is not heading down | `RisingHold` |
| sounds waiting | 0 to 15: the sounds waiting in the queue the speed booster's sound joins, before it (the sound domain) | `BoostingLap` |

### 1.4 Her clips and the run's paces

Clips and paces are content: the importer supplies them in this format, and the vectors use
invented ones in the same format. A clip is a list of steps. A shown step has a hold, 1 frame or
more; every other step is a control, which the clip passes through in the frame it reaches
it, unless it says it rests there.

Cells: show, rest, again, breathe, end, end unless a jump, branch

| Step | Written | What it does when the clip reaches it |
| --- | --- | --- |
| show | `n` | shows its picture for its hold, `n` frames (`ClipPlays`) |
| rest | `rest` | she rests on it for one frame with no time left (`ClipControls`) |
| again | `again s` | the clip goes on from step `s`, a shown step (`ClipControls`) |
| breathe | `breathe` | back to step 0 while her energy is high, else on to the next step (`Breathing`) |
| end | `end` | the clip ends (`ClipEnds`) |
| end unless a jump | `end-unless-jump` | the clip ends, unless the pad asks for a jump (`ClipEnds`) |
| branch | `branch x y z` | goes on from step `x` (the spin), `y` (the space jump) or `z` (the screw attack) (`WallJumpBranch`) |

A clip may also mark one step each as its **kick step** (`kick`, a spin's: `WallJumpReady`,
`SpinTurnedFar`), its **fast fall step** (`fast_fall`: `FastFall`) and its **rising step**
(`rising`: `RisingHold`).

The run's paces: the **plain pace**, a hold for every shown step of her running clips, and five
**boosted paces**, one per boost stage 0 to 4, each a hold for the same steps.

### 1.5 The inputs' domain

The spec defines her animation on these inputs; outside them the game's behaviour is outside
this spec (stated simplifications, docs/LEGACY.md):

- An `again` or a `branch` goes to a shown step, a `rest` is followed by a shown step, and no
  clip's last step is a shown step, so a clip never moves on past its end.
- **Whether her clip ends depends only on the frame's start.** No clip with an `end` or an
  `end-unless-jump` has a kick, fast fall or rising step, or belongs to her run, to her standing
  unaimed (the standing restart's), or to a "kept" row of `StartingClip`. So
  the end follows from her step, the time left and her clip alone, and no move of the
  frame changes it (§6).
- When her clip ends (`ClipEnds`) she takes another pose that frame, and when a turn's or a
  landing's end gives way, she takes the jump the pad asked for.
- A hold is at most 127 frames (the game's are at most 80), so a hold with both slowdowns stays
  within the time left's 0 to 255.
- Not 5 sounds waiting when she starts boosting (`BoostingLap`): with 5 the game's behaviour is
  outside this spec (LEGACY).
- The laps left are 1 or 2 with the speed booster, 0 without it.

---

## 2. Her clip plays

### ClipPlays
- kind: mechanic.
- summary: each frame the step's time runs down by 1; when it runs out the clip moves on, and
  a shown step shows for its hold.
- params: none.
- switch: none.
- The time left, less 1, above 0 → that is the time left, on the same step [C-count, R-count]
- The time left, less 1, at 0 or below → the clip moves on to `step + 1` [C-next, C-after-rest]
- The clip moving on to a shown step → its hold plus the slowdown is the time left,
  `hold + slowdown` [C-next, C-after-rest, C-slow-show]
- So a hold of `n` shows its step for `n` frames, and a step with no time left (after a rest)
  moves on the next frame.
- checked: oracle (every recorded frame of play); constructed (C-count, C-next, C-after-rest,
  C-slow-show); recorded (R-count).

### ClipControls
- kind: mechanic.
- summary: an `again` loops the clip back to a step, and a `rest` holds it one frame.
- params: none.
- switch: none.
- The clip moving on to `again s` → it goes on from step `s`, with that step's hold plus the
  slowdown as the time left [C-again, C-slow-again]
- The clip moving on to a `rest` → she rests on it with no time left, 0 [C-rest]
- So a rest shows its picture for exactly one frame, however slow the liquid, and the clip moves
  on from it the next frame (C-after-rest).
- checked: oracle (500 loops and 6 rests on record); constructed (C-again, C-slow-again, C-rest,
  C-after-rest).

### Breathing
- kind: mechanic.
- summary: standing or crouched still, she breathes in a loop while her energy is high, and goes
  on to breathe hard once it is low.
- params: the threshold, 30 energy.
- switch: none.
- The clip moving on to `breathe`, her energy at 30 or more → back to step 0, with its hold plus
  the slowdown [C-breathe-loop]
- The clip moving on to `breathe`, her energy below 30 → on to the next step, with its hold plus
  the slowdown [C-breathe-on]
- checked: oracle (31 loops and 2 on to breathe hard, on record); constructed (C-breathe-loop,
  C-breathe-on).

### ClipEnds
- kind: output.
- summary: a clip that reaches its end reports which end it is, and samus-motion hands over her
  next pose; at a turn's or a landing's end it gives way to a jump its pad chart asks for.
- params: none.
- switch: none.
- The clip moving on to `end` → the end is an end [C-end, C-slow-handover]
- The clip moving on to `end-unless-jump` → the end is an end unless a jump [C-end-unless-jump,
  C-end-gives-way, C-jump-up-from-turn]
- So with time left on the step before an end, the frame reports no end (C-end-later); the
  frame an end is reported is the frame her next pose shows (`StartingClip`), and whether an end
  unless a jump hands over is samus-motion's (C-end-gives-way: its pad's jump taken).
- checked: oracle (350 ends on record: 151 ends, 199 ends unless a jump, of which 9 gave way;
  each is reported from the frame's start); constructed (C-end, C-end-unless-jump,
  C-end-gives-way, C-end-later).

### WallJumpBranch
- kind: mechanic.
- summary: a wall jump spins as the screw attack with it, else as the space jump with it, else
  plainly, and plainly when submerged; each asks for its own spin's sound.
- params: none.
- switch: none.
- The clip moving on to `branch x y z`, submerged → step `x`, asking for `PlainSpin`
  [C-branch-submerged]
- Not submerged, with the screw attack → step `z`, asking for `ScrewAttack` [C-branch-screw,
  C-branch-screw-and-space]
- Not submerged, with the space jump and not the screw attack → step `y`, asking for
  `SpaceJump` [C-branch-space]
- Not submerged, with neither → step `x`, asking for `PlainSpin` [C-branch-spin]
- The step it goes to → its hold plus the slowdown is the time left [C-branch-spin,
  C-branch-space]
- checked: oracle (13 wall jumps on record: 9 plain, 4 with the screw attack); constructed
  (C-branch-spin, C-branch-space, C-branch-screw, C-branch-screw-and-space,
  C-branch-submerged).

### HandOverSlowdown
- kind: quirk.
- summary: in a liquid, the first step of a pose a clip's end handed over is slowed by the start's
  slowdown and by the clip's slowdown both.
- params: none.
- switch: none.
- Handed over: she takes the pose her clip's end handed over → the first step's time left is
  longer by the slowdown too, `hold + start + slowdown` with `start` the start's slowdown
  [C-slow-handover, C-slow-gravity-handover, C-slow-acid-gravity]
- So another pose that no clip's end brought takes the start's slowdown alone (C-slow-start,
  C-slow-gravity-start), as does the pad's jump after an end unless a jump (C-end-gives-way), and
  out of a liquid both are 0.
- checked: constructed (C-slow-handover, C-slow-gravity-handover, C-slow-acid-gravity,
  C-slow-start, C-slow-gravity-start, C-end-gives-way); no recorded frame of play is in a liquid
  that slows her.

---

## 3. Taking another pose

### StartingClip
- kind: mechanic.
- summary: the clip of the pose she takes starts at its first step, at its second when she
  was already doing what the first step shows, and keeps going when her run or her ball goes on
  in another pose.
- params: the table.
- switch: none.
- She takes another pose, the pair in a row of "kept" below → her step and the time left stay as
  they are [C-kept-run, C-kept-ball, C-kept-spring]
- She takes another pose, the pair in a row of "step 1" below → its clip from step 1, with its
  hold plus the start's slowdown, `hold + start` [C-spin-from-spin, C-spin-from-walljump,
  C-stand-up-from-up, C-landing-up-from-up, C-jump-up-from-takeoff, C-crouch-up-from-crouching,
  C-spin-sprite-only]
- She takes another pose, any other pair → its clip from step 0, with its hold plus the start's
  slowdown [C-start, C-not-kept-morph, C-stand-up-from-turn, C-stand-up-from-forward,
  C-jump-up-from-turn, C-crouch-up-from-turn, C-suit-sprite-only,
  C-start-after-count, C-slow-start, C-slow-gravity-start, C-slow-surface]

Cells: kept, step 1

| She takes | From | Starts |
| --- | --- | --- |
| `Running` | `Running` | kept |
| the ball `Still`, `Rolling` or `Falling` | the ball `Still`, `Rolling` or `Falling` | kept |
| the spring ball, any action | the spring ball, any action | kept |
| `Spinning` | `Spinning` or `WallJumping` | step 1: the spin goes on past its first step, the jump from the floor |
| upright `Still` or `Landing`, aimed `Up` | aimed `Up`, any action but `Turning` | step 1: her arm is already up |
| `Jumping` aimed `Up` | `TakingOff` aimed `Up` | step 1 |
| crouched `Still` aimed `Up` | `Crouching` aimed `Up` | step 1 |

- So the ball `Morphing` is not one of the ball's rows (C-not-kept-morph), and a turn aimed up
  doesn't shoot, so it doesn't count as aimed up (C-stand-up-from-turn).
- checked: oracle (1,964 kept, 102 from step 1 and 2,139 from step 0 on record); constructed
  (the vectors above).

---

## 4. What her moves do to it

### StandingRestart
- kind: mechanic.
- summary: holding shoot while she stands unaimed starts her standing clip over.
- params: the restart's hold, 16 frames, which no liquid slows.
- switch: none.
- Standing restarts → step 0 with 16 frames left, before the time runs down this frame
  [C-restart, R-restart-a, R-restart-b]
- So the frame ends on step 0 with 15 frames left (R-restart-a, from 1 frame left).
- checked: oracle (29 restarts on record); constructed (C-restart); recorded (R-restart-a,
  R-restart-b).

### WallJumpReady
- kind: mechanic.
- summary: readying a wall jump turns her spin straight to its kick step.
- params: none.
- switch: none.
- A wall jump readied, her clip with a kick step → the step before the kick step, with 1 frame
  left, before the time runs down this frame [C-ready, C-ready-walljump]
- So the clip moves on the same frame and shows the kick step for its hold (C-ready), or, in a
  wall jump's clip whose kick step is an `again`, goes where that leads (C-ready-walljump).
- checked: oracle (39 readied on record); constructed (C-ready, C-ready-walljump).

### SpinTurnedFar
- kind: output.
- summary: her spin has turned far enough to kick off a wall once it reaches its kick step.
- params: none.
- switch: none.
- The clip she ends the frame on with a kick step, her step at it or past it → turned far
  [C-far, C-ready, C-far-new-clip]
- Her clip with a kick step, her step before it → not turned far [C-not-far]
- Her clip with no kick step → not turned far [C-far-no-kick]
- checked: oracle (every frame of a spin on record, through samus-motion's wall jumps);
  constructed (C-far, C-not-far, C-far-no-kick, C-ready, C-far-new-clip).

### FastFall
- kind: mechanic.
- summary: moving fast in a fall's pose, her fall jumps to its fast fall picture, whichever way
  she moves.
- params: the speed, 5 pixels a frame; the hold, 8 frames, which no liquid slows.
- switch: none.
- Her clip with a fast fall step, her step on any step before it, whatever the time left, her
  vertical speed 5 pixels a frame or more either way (its size, rising or falling) → the fast
  fall step with 8 frames left, before the time runs down this frame [C-fast-fall,
  C-fast-fall-early, C-fast-fall-liquid, C-fast-fall-rising]
- So short of 5 pixels a frame either way (C-fast-fall-slow, C-fast-fall-rising-slow), or on the
  fast fall step or past it (C-fast-fall-on, C-fast-fall-past), the clip plays as it would.
- So a fall's pose rising at 5 pixels a frame or more (an unmorph's end that carries such a rise
  into the crouch, then a walk off a ledge) shows the fast fall picture too (C-fast-fall-rising);
  no record rises so fast in a fall's pose (the fastest is 2.4000).
- checked: oracle (4 on record); constructed (C-fast-fall, C-fast-fall-early, C-fast-fall-liquid,
  C-fast-fall-slow, C-fast-fall-rising, C-fast-fall-rising-slow, C-fast-fall-on,
  C-fast-fall-past).

### RisingHold
- kind: mechanic.
- summary: a straight jump holds its rising picture for as long as she rises.
- params: the hold, 4 frames, which no liquid slows.
- switch: none.
- Her clip with a rising step, on it with 1 frame left, not heading down → 4 frames left, before
  the time runs down this frame [C-rising]
- So with more time left on it, it counts down as usual (C-rising-early).
- So she stays on the rising step 3 frames at a time until she heads down, then the clip goes on
  (C-rising-down).
- checked: oracle (20 on record); constructed (C-rising, C-rising-early, C-rising-down).

---

## 5. Her run

### RunPace
- kind: mechanic.
- summary: running while dashing, her stride takes the run's pace, plain without the speed
  booster and quickening with its stages; with run held each lap of her run starts over on the
  pace.
- params: none (the paces are content, §1.4).
- switch: none.
- Running and dashing, the clip moving on to a shown step → its hold is the pace's for that
  step, plus the slowdown: the plain pace without the speed booster, the boosted pace of her stage
  with it [C-pace-plain, C-pace-boost, C-dash-lost-moving-on]
- So dashing in any other action, a jump or a fall, her clip's own holds show (C-dash-jump).
- Running and dashing, run held, without the speed booster, the clip moving on to a control
  → the lap starts over: step 0, with the plain pace's first hold plus the slowdown [C-lap-plain,
  C-lap-again-mid, C-lap-at-rest]
- So dashing with run let go, a lap ends as the clip says, on the clip's own hold
  (C-lap-not-held), and not dashing her run shows the clip's holds (C-not-dashing).
- checked: oracle (760 plain and 715 boosted holds, 58 laps on record); constructed (C-pace-plain,
  C-pace-boost, C-lap-plain, C-lap-not-held, C-not-dashing, C-dash-jump).

### BoostStages
- kind: mechanic.
- summary: with the speed booster, every lap run with run held counts towards the next stage;
  four stages on she is boosting, and laps go on at the boosting pace.
- params: the laps each stage lasts, 1 at stages 0 to 3 and 2 at stage 4.
- switch: parameter (the laps per stage).
- A dash began → stage 0, with stage 0's laps left with the speed booster, 0 without it
  [C-began-booster, C-began-plain, R-began]
- Running and dashing, run held, with the speed booster, the clip moving on to a control,
  the laps left less 1 above 0 → that is the laps left, and the control does as the clip says
  [C-lap-count]
- So with run let go the laps left stay as they are, no stage goes up, and the control does as
  the clip says (C-lap-not-held-booster, C-stage-not-held).
- The same, the laps left less 1 at 0, the stage below 4 → the stage goes up by 1, the laps left
  are the new stage's, and the lap starts over: step 0, with the new stage's pace's first hold
  plus the slowdown [C-stage-up]
- The same at stage 4 → the laps left are stage 4's, and the lap starts over: step 0, with the
  boosting pace's first hold plus the slowdown [C-boosting-lap]
- The stage going up to 4 → asks for `Boosting` [C-boost-quirk-0]
- A dash lost → stage 0, with 0 laps left, after the clip moves on [C-lost, C-dash-lost-moving-on,
  R-lost-a, R-lost-b]
- checked: oracle (96 began, 50 stages up, 16 laps counted, 109 lost on record); constructed
  (C-began-booster, C-began-plain, C-lap-count, C-lap-not-held-booster, C-stage-not-held,
  C-stage-up, C-boosting-lap, C-lost); recorded (R-began, R-lost-a, R-lost-b).

### BoostingLap
- kind: quirk.
- summary: the lap on which she starts boosting starts on the first hold of the stage numbered by
  the sounds waiting, usually the slowest, and lasts that stage's laps; the rest of the lap runs
  at the boosting pace.
- params: none.
- switch: none.
- The stage going up to 4, `w` sounds waiting, `w` at most 4 → the lap's first hold is stage
  `w`'s pace's, and the laps left are stage `w`'s [C-boost-quirk-0, C-boost-quirk-2,
  C-boost-quirk-4]
- The same, 6 or more waiting → stage 3's [C-boost-quirk-6]
- So with no sound waiting, as on record, the first boosting lap starts on stage 0's first hold
  with stage 0's 1 lap; its other steps show the boosting pace, since her stage is 4
  (`RunPace`; C-boosting-first-lap), and its lap end starts the next at the boosting pace.
- checked: oracle (5 on record, each with 0 waiting); constructed (C-boost-quirk-0,
  C-boost-quirk-2, C-boost-quirk-4, C-boost-quirk-6, C-boosting-first-lap).

---

## 6. Order that matters

A frame has two halves, because samus-motion reads the end before it picks her pose, and this
domain reads the pose it picks:

- **First, before samus-motion's step:** the end (`ClipEnds`), from her step, the time left and
  her clip at the frame's start. §1.5 makes this exact: no move of the frame can change
  whether her clip ends.
- **Then, with samus-motion's results and the pose she takes**, in this order, as the State at
  the frame's end shows:

1. Her moves' changes: a dash's beginning, a standing restart, a readied wall jump, a fast
   fall.
2. The rising hold.
3. The time runs down and the clip moves on (§2, §5), from her pose at the frame's start.
4. The start of the clip of the pose she takes (`StartingClip`, `HandOverSlowdown`).
5. A dash's loss.
6. Turned far, from the step and clip the frame ends on.

So a restart from 1 frame left ends the frame on 15 (R-restart-a), a readied spin shows its kick
step the same frame (C-ready), and a clip that ends hands over in the frame it ends (C-end).

## 7. Out of scope, and who supplies it

- **Part 2 of this domain:** her sprite (the picture each step shows, where its halves sit, what
  it uploads), her palettes (her suits, the visor, the charge's flash, the speed booster's glow
  and the screw attack's, the flash when she is hurt), the flicker while she is invincible, the
  splashes and dust, and the echoes' pictures. They read her step and the boost stage.
- **Poses other domains drive**, whose clips those domains step or set: the x-ray scope, the
  grapple beam, the shinespark (its windup sets the stage to 4 and its end clears it; the crouch's
  shine keeps it), knockback and her hurt, a metroid draining her, the crystal flash, her
  appearance at the start, the death, and an item switched on or off mid-pose by the pause screen.
- **The liquids:** the slowdown and whether she is submerged.

## 8. Open questions

1. The laps left at 0 with the speed booster (a dash begun without it, then the booster switched
   on mid-run) is outside the inputs' domain (§1.5); the pause screen's equipment change is out of
   scope.
