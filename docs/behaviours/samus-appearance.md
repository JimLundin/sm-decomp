# How Samus looks, part 1, her animation's timing: behaviour spec

What the player sees of Samus's animation from one frame to the next: which step of her pose's
clip shows and for how long, how a clip loops, branches and ends, where the clip of the pose she
takes starts, what her moves do to it (a shot from standing, readying a wall jump, a fast fall, a
straight jump's rise), and her run: its pace while she dashes and the speed booster's stages up
to boosting. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-appearance.toml`](../../scenarios/vectors/samus-appearance.toml).

## Header

**Scope.** Part 1 of the domain "how Samus looks": her animation's timing and its sound opportunities (§9). One domain with her
sprite and colours (part 2), because the player sees one thing, her figure; this part is the
clock that the sprite reads. It covers the step her clip shows, the time left on it, the end of
a clip (which [samus-movement](samus-movement.md) reads as `AnimationEnds`), whether her spin has
turned far enough to kick off a wall (samus-movement's input "spin turned far"), and the boost: her
run's stages up to boosting, which the shinespark, the echoes, her glow, the blocks that only a
boost breaks, and the enemies she touches read.

**What it reads** (§1.3, each from the domain that owns it): her pose at the frame's start and at
its end, and what her moves did this frame (samus-movement); her clips and the run's paces (the
importer, §1.4); her energy and the items switched on (progress); the pad's run button; the
slowdown and whether she is submerged (the liquids); how many sounds are waiting (the sound
domain).

**What it leaves to others**: which pose follows a clip's end (samus-movement, `AnimationEnds`);
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
| the laps left | 0 to the most laps a stage lasts (2 in the game): the laps of her run the stage still lasts, the lap she is running included, so at 1 that lap is the stage's last | when the pace next quickens | `BoostStages` |

These four are all the domain remembers; each is State a caller can set to any value the spec
allows and read back. A caller keeps them from one frame to the next and gives them back as they
were left: nothing else carries over, since the end, whether her spin has turned far, where the
clip of the pose she takes starts, the slowdowns and what her moves did are each worked out
within the frame. A vector gives them as `before` and compares them as `after`.

### 1.1 Outputs

| Output | Type | Read by |
| --- | --- | --- |
| end | the end her clip reaches this frame (`ClipEnds`): none, an end, or an end unless a jump; known from the frame's start, before samus-movement picks her pose (§6) | samus-movement (`AnimationEnds`, where an end unless a jump gives way when its pad chart's pose that frame is a take-off aimed forward or a spin) |
| turned far | `bool`: whether her spin has turned far enough to kick off a wall (`SpinTurnedFar`); known from the frame's start, before samus-movement's step (§6) | samus-movement (`WallJump`), that frame |
| sounds | the sound asked for by the animation update, at most one (an update passes at most one control): `PlainSpin`, `SpaceJump`, `ScrewAttack` (`WallJumpBranch`), `Boosting` (`BoostStages`) | the sound domain |
| her step and the boost stage | as §1 | part 2 (her sprite, her glow), the shinespark, the echoes, the blocks, the enemies |

The end and turned far are given ahead: each follows from the State and the clip of her pose at
the frame's start alone, so a caller reads them before the frame, as samus-movement does. The frame
itself then tells only what it alone can: the State it leaves and the sound it asks for.

### 1.2 Her pose

Her pose comes from samus-movement (§2 there: body, action, facing, aim, gun, legs) with the sprite
`HowSheIsDrawn` chooses for it (plain, the screw attack's, the space jump's, the other suit's):
each pose and sprite has its own clip. She takes another pose when either part differs.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| her pose | §1.2, at the frame's start | every entry |
| her pose after | §1.2: the pose she ends the frame in (samus-movement's choice) | `StartingClip` |
| her clips | content, by pose (§1.4) | every entry |
| the run's paces | content (§1.4) | `RunPace`, `BoostStages` |
| energy | a count | `Breathing` |
| items switched on | `SpeedBooster`, `SpaceJump`, `ScrewAttack`: progress's own output | `RunPace`, `BoostStages`, `WallJumpBranch` |
| submerged | `bool`: she is under a liquid's surface, the liquid slows her, and she lacks the gravity suit (the liquids) | `WallJumpBranch` |
| slowdown | frames, 0 to 3: added to the holds her clip moves on to: 3 in water, whatever her suit; 2 in acid, whatever her suit; 2 in lava without the gravity suit, 0 with it; 0 out of a liquid (the liquids) | `ClipPlays`, `ClipControls`, `Breathing`, `WallJumpBranch`, `HandOverSlowdown`, `RunPace`, `BoostStages` |
| the start's slowdown | frames, 0 to 3: added to the first hold of the pose she takes: 3 with the feet of that pose under water, 2 under lava or acid; 0 with the gravity suit, out of a liquid, or where the liquid doesn't slow her (the liquids) | `StartingClip`, `HandOverSlowdown` |
| run held | `bool`: the pad holds run this frame | `RunPace`, `BoostStages` |
| handed over | `bool`: samus-movement took the pose her clip's end handed over this frame (samus-movement's `handed over`, its §1.4) | `HandOverSlowdown` |
| dashing | `bool`: she is dashing as the clip moves on (samus-movement's `dashing after her moves`, its §1.4) | `RunPace`, `BoostStages` |
| dash began, dash lost | `bool` each: a dash began this frame, before the clip moves on; it was lost this frame (samus-movement's `dash began` and `dash lost`) | `BoostStages` |
| standing restarts | `bool`: samus-movement's "the standing animation restarts" (`AimingAndShooting`) | `StandingRestart` |
| wall jump readied | `bool`: samus-movement's near-wall test readied a wall jump (`WallJump`) | `WallJumpReady` |
| her vertical speed | signed `P.S`, pixels a frame, downward positive: samus-movement's `vertical speed after her moves` (its §1.4), as her moves leave her, before what they met acts and before the pose change. An input of its own, apart from heading down: she can rise fast in a fall's pose, and the moonfall and a rise a ceiling stops are not heading down whatever the speed | `FastFall` |
| heading down | `bool`: samus-movement's `heading down` (its §1.4), as her moves leave her, before what they met acts and before the pose change: she falls, a fall at 0 included, and not in the moonfall (samus-movement's `Moonfall`: the spin she enters as a turn ends, which drops with no rise); a rise a ceiling stops this frame is not heading down | `RisingHold` |
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
**boosted paces**, one per boost stage 0 to 4, each a hold for the same steps. A pace's holds go
by step number: its hold `n` is for step `n`, which her running clips number alike.

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
- The laps left are 1 to the stage's laps with the speed booster, 0 without it; each stage
  lasts 1 lap or more.
- A dash that begins this frame is dashing, and she is handed over a pose only on a frame whose
  end (`ClipEnds`) is an end, or an end unless a jump that doesn't give way.

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
  moves on the next frame: 1 frame left and none move on alike.
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
- switch: parameter (the threshold).
- The clip moving on to `breathe`, her energy at the threshold or more → back to step 0, with
  its hold plus the slowdown [C-breathe-loop]
- The clip moving on to `breathe`, her energy below the threshold → on to the next step, with its
  hold plus the slowdown [C-breathe-on, C-breathe-adjusted]
- checked: oracle (31 loops and 2 on to breathe hard, on record); constructed (C-breathe-loop,
  C-breathe-on); adjusted, the threshold at 50 (C-breathe-adjusted).

### ClipEnds
- kind: output.
- summary: a clip that reaches its end reports which end it is, and samus-movement hands over her
  next pose; at a turn's or a landing's end it gives way to a jump its pad chart asks for.
- params: none.
- switch: none.
- The clip moving on to `end` → the end is an end [C-end, C-slow-handover]
- The clip moving on to `end-unless-jump` → the end is an end unless a jump [C-end-unless-jump,
  C-end-gives-way, C-jump-up-from-turn]
- So with time left on the step before an end, the frame reports no end (C-end-later); the
  frame an end is reported is the frame her next pose shows (`StartingClip`), and whether an end
  unless a jump hands over is samus-movement's (C-end-gives-way: its pad's jump taken).
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
- So, as the time runs down in the clip of her pose at the frame's start (§6), a kept row keeps
  the hold that clip gave as it moved on (C-kept-ball: the ball's 9).
- checked: oracle (1,964 kept, 102 from step 1 and 2,139 from step 0 on record); constructed
  (the vectors above).

---

## 4. What her moves do to it

### StandingRestart
- kind: mechanic.
- summary: holding shoot while she stands unaimed starts her standing clip over.
- params: the restart's hold, 16 frames, which no liquid slows.
- switch: parameter (the restart's hold).
- Standing restarts → step 0 with the restart's hold left, before the time runs down this frame
  [C-restart, R-restart-a, R-restart-b, C-restart-adjusted]
- So the frame ends on step 0 with 15 frames left (R-restart-a, from 1 frame left).
- checked: oracle (29 restarts on record); constructed (C-restart); recorded (R-restart-a,
  R-restart-b); adjusted, the restart's hold at 25 (C-restart-adjusted).

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
- summary: her spin has turned far enough to kick off a wall once it is on its kick step or past
  it, known as the frame starts.
- params: none.
- switch: none.
- The clip of her pose at the frame's start with a kick step, her step at it or past it →
  turned far [C-far, C-far-at-kick, C-far-leaving, C-far-later-branch]
- Her clip with a kick step, her step before it → not turned far [C-not-far, C-ready]
- Her clip with no kick step → not turned far [C-far-no-kick, C-far-taken-clip]
- So it reads only the State and her pose's clip as the frame starts, as the end does (§6): the
  frame's moves and the pose she takes change it from the next frame on. A readied wall jump
  shows as turned far the frame after (C-ready, then C-far-at-kick), as does a clip she takes on
  its kick step (C-far-taken-clip), and the pose she takes doesn't undo it that frame
  (C-far-leaving).
- So "past it" goes by step number: in a clip that branches, the steps of a branch numbered after
  the kick step are past it (C-far-later-branch: the space jump's steps in a wall jump).
- checked: oracle (every frame of a spin on record, through samus-movement, which reads it when she wall jumps);
  constructed (C-far, C-far-at-kick, C-not-far, C-far-no-kick, C-ready, C-far-taken-clip,
  C-far-leaving, C-far-later-branch).

### FastFall
- kind: mechanic.
- summary: moving fast in a fall's pose, her fall jumps to its fast fall picture, whichever way
  she moves.
- params: the speed, 5 pixels a frame; the hold, 8 frames, which no liquid slows.
- switch: parameter (the speed and the hold).
- Her clip with a fast fall step, her step on any step before it, whatever the time left, her
  vertical speed at the speed or more either way (its size, rising or falling) → the fast fall
  step with the hold left, before the time runs down this frame [C-fast-fall,
  C-fast-fall-early, C-fast-fall-liquid, C-fast-fall-rising, C-fast-fall-adjusted]
- So short of 5 pixels a frame either way (C-fast-fall-slow, C-fast-fall-rising-slow), or on the
  fast fall step or past it (C-fast-fall-on, C-fast-fall-past), the clip plays as it would.
- So a fall's pose rising at 5 pixels a frame or more (an unmorph's end that carries such a rise
  into the crouch, then a walk off a ledge) shows the fast fall picture too (C-fast-fall-rising);
  no record rises so fast in a fall's pose (the fastest is 2.4000).
- checked: oracle (4 on record); constructed (C-fast-fall, C-fast-fall-early, C-fast-fall-liquid,
  C-fast-fall-slow, C-fast-fall-rising, C-fast-fall-rising-slow, C-fast-fall-on,
  C-fast-fall-past); adjusted, the speed at 3 pixels a frame and the hold at 12
  (C-fast-fall-adjusted).

### RisingHold
- kind: mechanic.
- summary: a straight jump holds its rising picture for as long as she rises.
- params: the hold, 4 frames, which no liquid slows.
- switch: parameter (the hold).
- Her clip with a rising step, on it with 1 frame left, not heading down → the hold left, before
  the time runs down this frame [C-rising, C-rising-adjusted]
- So with more time left on it, it counts down as usual (C-rising-early).
- So she stays on the rising step 3 frames at a time until she heads down, then the clip goes on
  (C-rising-down).
- checked: oracle (20 on record); constructed (C-rising, C-rising-early, C-rising-down); adjusted, the hold at 7 (C-rising-adjusted).

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
- So a lap is her running clip from step 0 to the first control the clip moves on to, whatever
  control it is: a rest, or an `again` to any step, ends it alike (C-lap-at-rest,
  C-lap-again-mid).
- So dashing with run let go, a lap ends as the clip says, on the clip's own hold
  (C-lap-not-held), and not dashing her run shows the clip's holds (C-not-dashing).
- checked: oracle (760 plain and 715 boosted holds, 58 laps on record); constructed (C-pace-plain,
  C-pace-boost, C-lap-plain, C-lap-not-held, C-not-dashing, C-dash-jump).

### BoostStages
- kind: mechanic.
- summary: with the speed booster, every lap run with run held counts towards the next stage;
  four stages on she is boosting, and laps go on at the boosting pace.
- params: the laps each stage lasts, 1 at stages 0 to 3 and 2 at stage 4; an adjusted game may
  give each stage 1 lap or more.
- switch: parameter (the laps per stage).
- A dash began → stage 0, with stage 0's laps left with the speed booster, 0 without it
  [C-began-booster, C-began-plain, R-began, C-laps-adjusted-began]
- Running and dashing, run held, with the speed booster, the clip moving on to a control,
  the laps left less 1 above 0 → that is the laps left, and the control does as the clip says
  [C-lap-count]
- So a lap only counted goes on with the clip's own hold for the step the control leads to, not
  the pace's (C-lap-count: the clip's 6); the pace's hold returns with the next shown step.
- So with run let go the laps left stay as they are, no stage goes up, and the control does as
  the clip says (C-lap-not-held-booster, C-stage-not-held).
- The same, the laps left less 1 at 0, the stage below 4 → the stage goes up by 1, the laps left
  are the new stage's, and the lap starts over: step 0, with the new stage's pace's first hold
  plus the slowdown [C-stage-up, C-laps-adjusted-up]
- The same at stage 4 → the laps left are stage 4's, and the lap starts over: step 0, with the
  boosting pace's first hold plus the slowdown [C-boosting-lap, C-laps-adjusted-boosting]
- The stage going up to 4 → asks for `Boosting` [C-boost-quirk-0]
- A dash lost → stage 0, with 0 laps left, after the clip moves on [C-lost, C-dash-lost-moving-on,
  R-lost-a, R-lost-b]
- checked: oracle (96 began, 50 stages up, 16 laps counted, 109 lost on record); constructed
  (C-began-booster, C-began-plain, C-lap-count, C-lap-not-held-booster, C-stage-not-held,
  C-stage-up, C-boosting-lap, C-lost); adjusted, the laps per stage at 3, 2, 4, 1 and 3
  (C-laps-adjusted-began, C-laps-adjusted-up, C-laps-adjusted-boosting); recorded (R-began, R-lost-a, R-lost-b).

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
- So 5 waiting is outside the inputs' domain, as §1.5 states (LEGACY).
- So with no sound waiting, as on record, the first boosting lap starts on stage 0's first hold
  with stage 0's 1 lap; its other steps show the boosting pace, since her stage is 4
  (`RunPace`; C-boosting-first-lap), and its lap end starts the next at the boosting pace.
- checked: oracle (5 on record, each with 0 waiting); constructed (C-boost-quirk-0,
  C-boost-quirk-2, C-boost-quirk-4, C-boost-quirk-6, C-boosting-first-lap).

---

## 6. Order that matters

A frame has two halves, because samus-movement reads the end before it picks her pose, and this
domain reads the pose it picks:

- **First, before samus-movement's step:** the end (`ClipEnds`), from her step, the time left and
  her clip at the frame's start, and whether her spin has turned far (`SpinTurnedFar`), from her
  step and that clip. §1.5 makes the end exact: no move of the frame can change whether her
  clip ends; turned far reads nothing the frame changes.
- **Then, with samus-movement's results and the pose she takes**, in this order, as the State at
  the frame's end shows:

1. Her moves' changes: a dash's beginning, a standing restart, a readied wall jump, a fast
   fall.
2. The rising hold.
3. The time runs down and the clip moves on (§2, §5), from her pose at the frame's start.
4. The start of the clip of the pose she takes (`StartingClip`, `HandOverSlowdown`).
5. A dash's loss.

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


## 9. Sounds at her stride, pose selection and drawing

These opportunities belong to how Samus looks: a foot contacting the ground is timed by the
picture still showing, a spin starts with the figure she takes, and a spin ends while that
figure is drawn. Each is an observable opportunity with its own inputs; the implementer chooses the
interface form. None advances the animation, changes
its four State quantities, or consumes a sound acknowledgement. The caller runs each at its
named opportunity once. Multiple opportunities in one work may each ask for sound; the
animation update's at-most-one output does not limit their combined outputs.

### 9.1 Inputs, content and outputs

| Quantity | Type and owner | Read by |
| --- | --- | --- |
| contact steps | a set of shown step indices in each clip, content supplied by the importer; any number of contacts, independent of holds and the kick mark | `FootContact` |
| stride figure and playhead | her figure, step and frames left at the running movement opportunity, before the animation update; samus-movement and this domain | `FootContact` |
| cinematic active | bool, the cinematic owner | `FootContact`, `SpinStarts` |
| boss active | bool, the boss owner | `FootContact` |
| special colours active | bool, appearance's colour owner: special colour time remains | `FootContact` |
| boosting | the boost stage already in this domain's State, at the stride opportunity | `FootContact` |
| figure selected from and figure selected | her figure immediately before and after a pose selection, including its sprite; samus-movement | `SpinStarts` |
| submerged and items | the existing liquid input and items switched on, at pose selection | `SpinStarts` |
| previous and current movement | typed actions: spin, wall jump, or any named other action; previous is the caller-retained action captured at the latest actual work preparation, current is samus-movement's action at normal drawing. Previous is independent of pose-change history | `SpinEnds` |
| current action, shoot held, charge | current action from samus-movement, bool from the pad, charge frames from weapons, a count from 0 to 32767 | `AtTheDoor` |

A contact opportunity returns a graphical contact opportunity separately from its optional sound.
Part 2 reads the graphical opportunity with the room and liquid context to decide the splash
or dust. Cinematic, boss and special colours suppress its sound only. This amendment does not
replace the existing atmospheric drawing or add liquid graphics. Its bounded dry Ceres
admission has no boosting, so the graphical opportunity leaves atmospheric slots and sprite output unchanged;
it must still remain available to part 2.

Sound outputs are semantic requests: `Footstep` to the footstep sounds, with at most 6 waiting;
`PlainSpin`, `SpaceJump` and `ScrewAttack` to Samus sounds, with at most 6 waiting;
`SpinEnded` and `SamusSilence` to Samus sounds, with at most 15 waiting. Sound admission,
sending, retries and driver replies belong to the sound domain. An outstanding retry is not
another appearance opportunity. Callers preserve requests from distinct opportunities in the
order those opportunities occur.

**Bounded engine admission for the other drawing effects.** Charge is zero, a charge sound is not retained
for resumption, the echoes are quiet, debug is inactive, and the countdown has not expired.
The owning domains must establish these predicates from the native starting State and preserve
them; a caller may not manufacture them as false or read them from a later oracle cut. A
violation requires the missing charge, echo or countdown owner before the engine can claim complete drawing work. These are engine capability limits, not guards on `SpinEnds`: the appearance model
returns its spin-end decision for every supplied pair of movements. It does not model
charge resumption, echo cancellation or countdown expiry.
Foot contact itself accepts every combination of its sound suppressors. The broader sprite,
colour and liquid effects remain part 2.

### FootContact
- kind: output.
- summary: the last frame of a running contact picture asks for the stride's graphics, and
  for its footstep sound unless an active sound suppressor prevents it.
- params: none; contacts are content.
- switch: none.
- Running, exactly 1 frame left, her shown step marked as a contact → graphical contact
  opportunity [C-contact, C-contact-cinematic, C-contact-boss, C-contact-colours, C-contact-boost]
- That contact, cinematic inactive, boss inactive, special colours inactive, not boosting →
  ask for `Footstep` [C-contact, C-contact-stage3, R-contact]
- That contact with any of those four active → no sound, graphical contact still present
  [C-contact-cinematic, C-contact-boss, C-contact-colours, C-contact-boost]
- Any other movement, a different time left (0 included), or an unmarked step → neither
  contact nor sound [C-contact-other, C-contact-early, C-contact-zero, C-contact-unmarked]
- checked: constructed (the cases above); recorded (R-contact, the native prefix contacts).

### SpinStarts
- kind: output.
- summary: the pose she takes starts its spin's sound, while a plain continuing spin stays
  quiet; this does not add a second animation update.
- params: none.
- switch: none.

For the item branches below, a qualifying selection means a different figure is selected,
its action is `Spinning`, and she is not submerged. This common condition says nothing
about her items, prior action or cinematic activity.

- Qualifying selection, with SpaceJump and without ScrewAttack → ask for `SpaceJump`,
  even from spin or wall jump and even during a cinematic [C-start-space, C-start-space-continued, C-start-space-cinematic]
- Qualifying selection, with ScrewAttack, not from spin or wall jump → ask for `ScrewAttack`,
  even during a cinematic [C-start-screw, C-start-both, C-start-screw-cinematic]
- Qualifying selection, with ScrewAttack, from spin or wall jump → no sound
  [C-start-screw-continued]
- Qualifying selection, with neither item, not from spin or wall jump, cinematic inactive →
  ask for `PlainSpin` [C-start-plain, R-start]
- Qualifying selection, with neither item, from spin or wall jump or cinematic active → no sound
  [C-start-continued, C-start-wall, C-start-cinematic, C-start-continued-cinematic]
- No different figure, no spinning selection, or submerged → no sound
  [C-start-same, C-start-other, C-start-submerged, C-start-submerged-both]
- checked: constructed (the cases above); recorded (R-start). The existing `WallJumpBranch`
  remains a wall jump's later animation branch sound, a distinct opportunity.

### SpinEnds
- kind: output.
- summary: when she leaves a spin or wall jump, normal drawing ends its sound, even while
  her animation is frozen.
- params: none.
- switch: none.
- Normal drawing, prior action `Spinning` or `WallJumping`, current action
  neither → ask for `SpinEnded` [C-end-spin, C-end-wall, C-end-frozen, R-unspin]
- Prior action neither, or current action `Spinning` or `WallJumping` → no sound
  [C-end-neither, C-end-spin-spin, C-end-spin-wall, C-end-wall-spin, C-end-wall-wall]
- checked: constructed (the cases above); recorded (R-unspin).

### AtTheDoor
- kind: output.
- summary: on first entering a passage, Samus stops her spin sound or her quiet uncharged
  sound before sounds are held.
- params: none.
- switch: none.
- First passage opportunity, current action `Spinning` or `WallJumping` → ask for `SpinEnded`,
  whatever shoot and charge [C-door-spin, C-door-wall]
- Current movement neither, shoot released, charge below 16 frames → ask for `SamusSilence`
  [C-door-quiet, C-door-below, R-door]
- Current movement neither, shoot held or charge 16 frames or more → no request
  [C-door-held, C-door-threshold, C-door-above]
- checked: constructed (the cases above); recorded (R-door).

### 9.2 Opportunities and paused work

The running contact read follows that running movement's position changes and precedes the
animation update; pose selection follows. The selection read belongs to each actual pose selection;
it uses the selected figures, not next-cut sound State. Normal drawing reads the remembered
prepared action and the current action after pose selection, including the normal drawing
performed during a passage's quieting; animation and motion may remain frozen there.
The first passage read happens once on entry before the sound hold, separately from normal
drawing. Sound retries are serviced by sound work alone. These reads require neither another
motion step nor another animation update [C-end-frozen, C-contact, C-door-quiet].


An invented work combines the existing wall-jump branch sound with drawing's spin end;
another combines a contact and a spin selection. Each retains both requests in opportunity
order, retain graphical contact, and compare all four animation quantities while animation
runs at most once [C-branch-and-end, C-contact-and-start].


The caller captures her current action when work preparation actually runs, after earlier
effects that may have changed her action, before ordinary movement; pose selection follows. It retains this
snapshot independently of pose-change history, and restores it with the rest of the caller's
State. Normal drawing reads the snapshot without refreshing or consuming it. It is a supplied
input, outside this model's four animation quantities; the appearance model stores no extra
history. A frame-start figure is not a substitute for the actual preparation snapshot.

Frozen animation alone does not decide whether the snapshot changes. Locked work that still
performs preparation captures the current action; paused drawing that bypasses preparation
keeps the earlier snapshot. The extra inanimate body draw does not capture an action or run a
second sound opportunity. If preparation is bypassed and normal drawing repeats, each actual
normal drawing reads the same retained snapshot. These rules preserve both the last prepared
spin and a current action that has left it, without another motion or animation update.

## 10. Observing and suspending the ordinary animation update

The two halves and order of section6 still hold. These observations let timing see the existing
animation decision before its effects, without another animation update. The implementation
chooses its interface and retention mechanism; no concrete method, stage list or enum is required.

| Quantity | Units and domain meaning | Read by |
| --- | --- | --- |
| pending animation decision | named existing animation rule, current figure/clip, playhead, stage/laps, relevant inputs and its chosen answer; steps as clip indices, holds in frames | output (timing); completion of that decision |
| completed playing result | actual playhead, boost stage and laps after the current clip plays, plus its optional semantic sound | StartingClip, BoostStages; output |
| actual selection | figure immediately before and actually accepted figure after selection, not an anticipated frame-end figure | StartingClip, HandOverSlowdown, SpinStarts |
| interrupted animation update | all four animation quantities, pending answer and still-needed actual inputs/figures, completed outputs and consumed input position | output (caller save/restore); continuation of this update |

These are update quantities, not a fifth persistent animation memory. The caller retains complete
interrupted work alongside the four State quantities and caller-owned opportunity/input progress.
An already chosen answer is retained unchanged. Inputs for a decision not yet made are supplied
at its actual opportunity; different future figures cannot choose how the earlier current clip plays.

### PerformedAnimationDecisions
- kind: timing.
- summary: the actual current clip decision and selected clip initialization are observable before their effects.
- params: none; the observed rules retain their existing parameters and content.
- switch: none.
- An animation update reaches a move-driven playhead change, current-clip play decision, selected-clip start or dash-loss decision → offer its actual named rule, current quantities and chosen answer before applying it; report its completed State and sound before any later decision consumes them [C-op-kept-clip, C-op-current-play, C-op-restart-play, C-op-restart-adjusted, C-op-kept-advance]
- checked: constructed (the cases above); adjusted (C-op-restart-adjusted).

Clip-playing reads the actual playhead after earlier movement-driven changes and rising hold,
and the current figure's clip, pace and stage. Its answer does not require a future selected
figure. Selected-clip initialization instead reads the actual accepted selection and the already
completed playing result. A kept Running clip keeps that result, including a hold newly read
from the old clip, rather than restoring its earlier playhead or reading a new clip's hold.
Dash loss follows these decisions as section6 defines.

FootContact and SpinStarts remain independent read-only opportunities of section9, with their
actual current quantities at the named opportunity. A contact reads after running position
changes and before animation advances. Selection reads actual selected figures. Their sound and
graphical outputs keep opportunity order with any animation-control output; none advances the
playhead, repeats movement or consumes a sound acknowledgement. Sound suppression leaves contact
graphics available. NormalDraw and PassageEntry keep their existing frozen-work meanings.

### SuspendedAppearance
- kind: timing.
- summary: interrupted animation and appearance opportunities continue once with complete State and ordered outputs.
- params: none.
- switch: none.
- Suspend before an offered animation effect or after a completed animation/appearance opportunity → serialize all four State quantities, pending answer and still-needed actual inputs/figures, completed outputs and consumed opportunity/input progress; discard the original interrupted instance and deserialize a fresh instance to restore to identical final State, graphics and ordered sounds as uninterrupted updating [C-op-kept-clip, C-op-current-play, C-op-restart-play, C-op-restart-adjusted, C-op-contact-order, C-op-contact-suppressed]
- The actual selection changes after current-clip playing but before selected-clip initialization is decided → preserve the already played current clip and decide initialization from the new actual selection [C-resume-selection-before]
- Selection changes after the local initialization answer was chosen, before its effect → apply the retained answer to its retained actual selection once, without recomputing it from the later figure [C-resume-selection-after]
- A contact suppressor changes before contact is read → its fresh value decides that sound, leaving graphics available [C-resume-contact-before]
- A contact suppressor changes after contact was completed → retain its completed sound and graphics without another contact read or publication [C-resume-contact-after]
- checked: constructed (the opportunity cases above), using every offered/completed opportunity as a save point.

The ordinary animation entry completes the same rules without suspension. Read-only Ahead and
appearance opportunities remain read-only; retaining their completed receipt prevents a resumed
update from asking them again. Saving observes work without advancing it. The caller's prepared
action snapshot remains independently retained as section9.2 specifies. A completed update has
no effects left to resume. All existing ordinary and frozen-work vectors retain their results.

Opportunity vectors use the additive `suspended_update` format documented in the existing
[samus-appearance vectors](../../scenarios/vectors/samus-appearance.toml). Animation rules and
content determine the answers; timing assigns no animation result or future selection. These
observations introduce no liquid, palette, echo, sound-transport or dispatch producer.

A future input is not cached merely because an update has begun. The changed-selection cases
play the old running clip to step2 with hold5 first. Before the selected initialization decision,
a new actual falling selection starts hold9. After a KeepPlayhead answer was already chosen for
the running selection, a later supplied figure cannot change that answer or its selected identity.
The paired contact cases likewise distinguish an unmade read from a completed publication.
These are observations of existing total rules, not new capability guards.

Every all-opportunity save test serializes the complete interrupted work and caller-held inputs,
opportunity/query and output progress, discards the original objects, then deserializes fresh
objects. A clone or memory-only checkpoint is insufficient. All four animation quantities,
actual figures, pending answers and completed sound/graphic progress survive; each actual effect
and read-only opportunity publishes once. Compare complete State and all outputs, beyond the
listed observation subsequence. No storage layout, method list or stage enum is prescribed.
