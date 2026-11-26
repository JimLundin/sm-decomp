# How Samus looks: behaviour spec

What the player sees of Samus's animation from one frame to the next: which step of her pose's
clip shows and for how long, how a clip loops, branches and ends, where the clip of the pose she
takes starts, what her moves do to it (a shot from standing, readying a wall jump, a fast fall, a
straight jump's rise), and her run: its pace while she dashes and the speed booster's stages up
to boosting. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/samus-appearance.toml`](../../scenarios/vectors/samus-appearance.toml).

## Header

**Scope.** Her animation's timing, its sound opportunities (§9), when her cannon may follow
the selected item (§10), the ordinary dry palette opportunity (§11), her hurt colours and the
push's poses (§12), her standing picture during the elevator hold and fade (§13), and her
body's protection flicker during ordinary play in Ceres (§14).
Her sprite and colours belong to the same domain because the player sees one figure.
The animation contract covers the step her clip shows, the time left on it, the end of
a clip (which [samus-movement](samus-movement.md) reads as `AnimationEnds`), whether her spin has
turned far enough to kick off a wall (samus-movement's input "spin turned far"), and the boost: her
run's stages up to boosting, which the shinespark, the echoes, her glow, the blocks that only a
boost breaks, and the enemies she touches read. Cannon readiness is a separate visible delay;
it does not choose a weapon, fire a shot or animate how her gun changes shape.

**What it reads** (§1.3, each from the domain that owns it, each once a frame): her pose at the
frame's start and at its end, and what her moves did this frame (samus-movement); her clips and
the run's paces (the importer, §1.4); her current energy ([health](samus-health.md)) and the
items switched on (progress); the pad's run and shoot buttons; the two slowdowns and whether she is
submerged (the liquids); how many sounds are waiting (the sound domain); whether a cinematic or
a boss is active; her charge (weapons).

**Palette neighbours.** Movement supplies her pose as the frame ends (§1.3). Equipment,
room effects, Environment, Grapple and the charge and effect owners supply current facts
(§11). Appearance owns the ordinary visor and charge-glow reset. Room palette objects
supply their own palette changes, and video owns delivery of the current palette.

**What it leaves to others**: which pose follows a clip's end (samus-movement, `AnimationEnds`);
delivery of her picture and tiles (video); body placement and art selection outside §13,
splashes and dust (remaining part 2, §7); the poses other domains drive (§7); whether a
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

These four, with the cannon delay (§10), the three colour-sequence quantities (§11.1) and her
kept action (§9.1), are her appearance State: all she remembers from one frame to the next.
Each is State a caller can set to any value the spec allows and read back. A caller keeps the
State from one frame to the next and gives it back as it was left: nothing else carries over,
since the end, whether her spin has turned far, where the clip of the pose she takes starts, the
slowdowns and what her moves did are each worked out within the frame. §1.6 gives the State she
starts with. A vector gives it as `before` and compares it as `after`.

### 1.1 Outputs

| Output | Type | Read by |
| --- | --- | --- |
| end | the end her clip reaches this frame (`ClipEnds`): none, an end, or an end unless a jump; known from the frame's start, before samus-movement picks her pose (§6) | samus-movement (`AnimationEnds`, where an end unless a jump gives way when its pad chart's pose that frame is a take-off aimed forward or a spin) |
| turned far | `bool`: whether her spin has turned far enough to kick off a wall (`SpinTurnedFar`); known from the frame's start, before samus-movement's step (§6) | samus-movement (`WallJump`), that frame |
| her step and the boost stage | as §1 | part 2 (her sprite, her glow), the shinespark, the echoes, the blocks, the enemies |
| the frame's sounds | one list of the sounds of §9.1, in the order asked (§9.2): the stride's, the animation update's (at most one, since an update passes at most one control), the spin start's, a passage entry's, drawing's | the sound domain |
| graphical contact | the stride's contact opportunity (`FootContact`) | part 2 (splashes and dust), that frame |
| cannon may follow | readiness as the frame's item controls leave it (§10) | the picture of her gun, that frame |
| her colour sequences | the visor delay, visor position and charge-glow position as the frame's palette opportunity leaves them (§11) | drawing and palette upload, that frame |

The end and turned far are the **read-ahead**: one query of her State before the frame, not a
part of the frame. It reads the State as the frame starts and the clip of her pose then, and
nothing else; it changes nothing, and a frame with her movement gives the same answer. A caller
asks it before the frame, as samus-movement does before its step, because the frame needs her
pose at its end, which that step picks. The frame itself then tells only what it alone can: the
State it leaves and the other outputs above, each as the whole frame leaves it.

### 1.2 Her pose

Her pose comes from samus-movement (§2 there: body, action, facing, aim, gun, legs) with the sprite
`HowSheIsDrawn` chooses for it (plain, the screw attack's, the space jump's, the other suit's):
each pose and sprite has its own clip. She takes another pose when either part differs.

### 1.3 What a frame brings (Inputs)

Each fact of the frame is one input, read by every rule that needs it. Her pose is two inputs,
as the frame starts and as it ends, because samus-movement picks the pose she takes part-way
through the frame: her stride and the preparation of her work come before the pick, the clip of
the first pose plays out before the clip of the second starts (§6), and the spin start compares
the two (§9.2). On a frame she doesn't move, her pose after is her pose.

| Input | Type | Read by |
| --- | --- | --- |
| her pose | §1.2, as the frame starts: the pose whose clip the State is on | every entry of §2 to §5; `FootContact` (her action and her clip's contact steps), `SpinStarts` (the pose she is selected from), `SpinEnds` (the action her preparation keeps) |
| her pose after | §1.2: the pose she ends the frame in (samus-movement's choice) | `StartingClip`; `SpinStarts` (the pose selected); `SpinEnds` and `AtTheDoor` (her action); `OrdinaryDryColours` (her body and action) |
| her clips | content, by pose (§1.4) | every entry of §2 to §5, `FootContact` |
| the run's paces | content (§1.4) | `RunPace`, `BoostStages` |
| energy | a count | `Breathing` |
| items switched on | `SpeedBooster`, `SpaceJump`, `ScrewAttack`: progress's own output | `RunPace`, `BoostStages`, `WallJumpBranch`, `SpinStarts`, `OrdinaryDryColours` |
| submerged | `bool`: she is under a liquid's surface, the liquid slows her, and she lacks the gravity suit (the liquids) | `WallJumpBranch`, `SpinStarts` |
| the step slowdown | frames, 0 to 3: added to the hold of each step her clip moves on to: 3 in water, whatever her suit; 2 in acid, whatever her suit; 2 in lava without the gravity suit, 0 with it; 0 out of a liquid (the liquids) | `ClipPlays`, `ClipControls`, `Breathing`, `WallJumpBranch`, `HandOverSlowdown`, `RunPace`, `BoostStages` |
| the start slowdown | frames, 0 to 3: added to the hold of the step the clip of the pose she takes starts on: 3 with the feet of that pose under water, 2 under lava or acid; 0 with the gravity suit, out of a liquid, or where the liquid doesn't slow her (the liquids) | `StartingClip`, `HandOverSlowdown` |
| run held | `bool`: the pad holds run this frame | `RunPace`, `BoostStages` |
| handed over | `bool`: samus-movement took the pose her clip's end handed over this frame (samus-movement's `handed over`, its §1.4) | `HandOverSlowdown` |
| the dash | one of the six below: her dash this frame, as samus-movement's `dashing after her moves`, `dash began` and `dash lost` (its §1.4) together give it | `RunPace`, `BoostStages` |
| standing restarts | `bool`: samus-movement's "the standing animation restarts" (`AimingAndShooting`) | `StandingRestart` |
| wall jump readied | `bool`: samus-movement's near-wall test readied a wall jump (`WallJump`) | `WallJumpReady` |
| her vertical speed | signed `P.S`, pixels a frame, downward positive: samus-movement's `vertical speed after her moves` (its §1.4), as her moves leave her, before what they met acts and before the pose change. An input of its own, apart from heading down: she can rise fast in a fall's pose, and the moonfall and a rise a ceiling stops are not heading down whatever the speed | `FastFall` |
| heading down | `bool`: samus-movement's `heading down` (its §1.4), as her moves leave her, before what they met acts and before the pose change: she falls, a fall at 0 included, and not in the moonfall (samus-movement's `Moonfall`: the spin she enters as a turn ends, which drops with no rise); a rise a ceiling stops this frame is not heading down | `RisingHold` |
| sounds waiting | 0 to 15: the sounds waiting in the queue the speed booster's sound joins, before it (the sound domain). The caller supplies it on every frame; it is no part of her State and has no starting value | `BoostingLap` |
| cinematic active | `bool` (the cinematic owner) | `FootContact`, `SpinStarts` |
| boss active | `bool` (the boss owner) | `FootContact` |
| special colours active | `bool`: special colour time remains (appearance's colour owner, part 2); not §11.2's special colouring | `FootContact` |
| shoot held | `bool`: the pad holds shoot | `AtTheDoor` |
| charge | frames, 0 to 32767: how long her shot has charged (weapons) | `AtTheDoor` |

**The slowdowns and submerged** are three facts of the liquids, none following from another.
The step slowdown applies to each step's time as the clip moves on (§2, §5); the start slowdown
to the step the clip of a pose she takes starts on (`StartingClip`). The gravity suit zeroes the
start slowdown in every liquid, and the step slowdown only in lava. Submerged reads neither.

**The dash.** Its beginning and its loss are one lifecycle, and a frame carries one of these
values. The clip moves on at §6's step 3, and a dash is lost at its step 5.

| The dash | Dashing as the clip moves on | Then |
| --- | --- | --- |
| off | no | nothing |
| on | yes | nothing |
| began | yes: it began this frame, before the clip moves on (§6, 1) | nothing |
| lost | yes | lost by the frame's end |
| began and lost | yes: it began this frame, before the clip moves on | lost by the frame's end |
| stopped | no: her moves stopped it before the clip moves on (a wall halts her, say) | lost by the frame's end |

Began and lost can fall in one frame: running with no dash, run held and her direction let go,
her moves begin a dash and she loses it as she slows down in the same pose, so the clip moves on
at the pace [C-dash-began-and-lost]. A dash that begins and stops before the clip moves on is
stopped.

Which parts the frame has is an input too. A frame may have any of them, each at most once:

| Part | When | Read by |
| --- | --- | --- |
| her initialization | first in its frame: it sets her whole appearance State to §1.6's, whatever State the frame starts with. The game's frame of her initialization has none of her other parts | every quantity of the State (§1.6), `CannonReadinessReset` |
| her movement | samus-movement's step, with the animation update on its results; her stride is part of it. A frozen frame, at a passage's door or while paused, has none | every entry of §2 to §5, `FootContact`, `SpinStarts` |
| the preparation of her work | before her movement (§9.2). A frame whose work is locked is still prepared; a paused frame, or a passage's frame that bypasses her work, is not | `SpinEnds` |
| her item controls | §10 | `CannonItemDelay` |
| the palette opportunity, with its observations (§11.2) | after her movement and the suit's selection | `OrdinaryDryColours` |
| her first entry into a passage | before the passage holds its sounds | `AtTheDoor` |
| normal drawing | the frame's last | `SpinEnds` |

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
  within the time left's 0 to 255. The restart's, the fast fall's and the rising frames (§4) are
  1 to 126.
- Not 5 sounds waiting when she starts boosting (`BoostingLap`): with 5 the game's behaviour is
  outside this spec (LEGACY).
- The laps left are 1 to the stage's laps with the speed booster, 0 without it or before her
  first dash since initialization (§1.6), when no rule reads them; each stage lasts 1 lap or
  more.
- Within the poses this domain plays (§7, which leaves the x-ray scope to another domain),
  nothing changes her pose between the frame's start and the preparation of her work: her
  preparation keeps the action of her pose as the frame starts (`SpinEnds`).
- She is handed over a pose only on a frame whose end (`ClipEnds`) is an end, or an end unless
  a jump that doesn't give way.

### 1.6 Her appearance at initialization

When Samus is initialized (her arrival at Ceres or Zebes, or a demo's loading), her appearance
State is the same whatever came before. Her initialization is a part of a frame of its own
(§1.3), not a part of her item controls: it sets every quantity of her appearance State to the
table below, whatever State the frame starts with [C-initialize, C-cannon-initialize]. The game
runs it while it loads, on a frame with none of her other parts; a frame given other parts too
plays them on from this State. So a caller either starts from this State or plays a frame with
her initialization from any State, alike:

| Quantity | At initialization |
| --- | --- |
| her step and the time left | 0 and 0 |
| the boost stage and the laps left | 0 and 0, whatever her items: a dash's beginning sets the laps (`BoostStages`) |
| the cannon delay | 2 (`CannonReadinessReset`) |
| the visor delay and visor sequence position | 1 eligible palette tick and the fourth entry (zero-based 3), the values the ordinary reset sets (`OrdinaryDryColours`) |
| the charge-glow position | the reset position |
| her kept action | `Still`: no spin, so drawing asks for no spin's end until a preparation keeps one (`SpinEnds`) |

The pose she takes at the start, and a demo's own setup of her, are other domains' (§7), and
play on from this State. The vector file's `[initialization]` table holds it.

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
- The clip moving on to a shown step → its hold plus the step slowdown is the time left
  [C-next, C-after-rest, C-slow-show]
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
  step slowdown as the time left [C-again, C-slow-again]
- The clip moving on to a `rest` → she rests on it with no time left, 0 [C-rest]
- So a rest shows its picture for exactly one frame, however slow the liquid, and the clip moves
  on from it the next frame [C-after-rest].
- checked: oracle (500 loops and 6 rests on record); constructed (C-again, C-slow-again, C-rest,
  C-after-rest).

### Breathing
- kind: mechanic.
- summary: standing or crouched still, she breathes in a loop while her energy is high, and goes
  on to breathe hard once it is low.
- params: the threshold, 30 energy.
- switch: parameter (the threshold).
- The clip moving on to `breathe`, her energy at the threshold or more → back to step 0, with
  its hold plus the step slowdown [C-breathe-loop]
- The clip moving on to `breathe`, her energy below the threshold → on to the next step, with its
  hold plus the step slowdown [C-breathe-on, C-breathe-adjusted]
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
- So with time left on the step before an end, the frame reports no end [C-end-later]; the
  frame an end is reported is the frame her next pose shows (`StartingClip`), and whether an end
  unless a jump hands over is samus-movement's, by the jump its pad asks for [C-end-gives-way].
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
- The step it goes to → its hold plus the step slowdown is the time left [C-branch-spin,
  C-branch-space]
- checked: oracle (13 wall jumps on record: 9 plain, 4 with the screw attack); constructed
  (C-branch-spin, C-branch-space, C-branch-screw, C-branch-screw-and-space,
  C-branch-submerged).

### HandOverSlowdown
- kind: quirk.
- summary: in a liquid, the first step of a pose a clip's end handed over is slowed by the start
  slowdown and by the step slowdown both.
- params: none.
- switch: none.
- Handed over: she takes the pose her clip's end handed over → the first step's time left is
  longer by the step slowdown too: its hold plus both slowdowns [C-slow-handover,
  C-slow-gravity-handover, C-slow-acid-gravity]
- So another pose that no clip's end brought takes the start slowdown alone, as does the pad's
  jump after an end unless a jump, and out of a liquid both are 0 [C-slow-start,
  C-slow-gravity-start, C-end-gives-way].
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
  they are [C-kept-run, C-kept-run-advance, C-kept-ball, C-kept-spring]
- She takes another pose, the pair in a row of "step 1" below → its clip from step 1, with its
  hold plus the start slowdown [C-spin-from-spin, C-spin-from-walljump,
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

- So the ball `Morphing` is not one of the ball's rows, and a turn aimed up doesn't shoot, so it
  doesn't count as aimed up [C-not-kept-morph, C-stand-up-from-turn].
- So, as the time runs down in the clip of her pose at the frame's start (§6), a kept row keeps
  the hold that clip gave as it moved on [C-kept-ball, C-kept-run-advance].
- checked: oracle (1,964 kept, 102 from step 1 and 2,139 from step 0 on record); constructed
  (the vectors above).

---

## 4. What her moves do to it

### StandingRestart
- kind: mechanic.
- summary: holding shoot while she stands unaimed starts her standing clip over.
- params: the restart's frames, 15: the frames step 0 shows after a restart, the restart's own
  frame the first of them; no liquid slows it.
- switch: parameter (the restart's frames).
- Standing restarts → step 0, and the frame ends with the restart's frames left, whatever time
  was left: the time left is the restart's frames plus 1 before the time runs down this frame
  [C-restart, R-restart-a, R-restart-b, C-restart-adjusted]
- checked: oracle (29 restarts on record); constructed (C-restart); recorded (R-restart-a,
  R-restart-b); adjusted, the restart's frames at 24 (C-restart-adjusted).

### WallJumpReady
- kind: mechanic.
- summary: readying a wall jump turns her spin straight to its kick step.
- params: none.
- switch: none.
- A wall jump readied, her clip with a kick step → the step before the kick step, with 1 frame
  left, before the time runs down this frame [C-ready, C-ready-walljump]
- So the clip moves on the same frame and shows the kick step for its hold, or, in a clip
  whose kick step is an `again`, goes where that leads [C-ready, C-ready-walljump].
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
  shows as turned far the frame after, as does a clip she takes on its kick step, and the pose
  she takes doesn't undo it that frame [C-ready, C-far-at-kick, C-far-taken-clip, C-far-leaving].
- So "past it" goes by step number: in a clip that branches, the steps of a branch numbered after
  the kick step are past it [C-far-later-branch].
- checked: oracle (every frame of a spin on record, through samus-movement, which reads it when she wall jumps);
  constructed (C-far, C-far-at-kick, C-not-far, C-far-no-kick, C-ready, C-far-taken-clip,
  C-far-leaving, C-far-later-branch).

### FastFall
- kind: mechanic.
- summary: moving fast in a fall's pose, her fall jumps to its fast fall picture, whichever way
  she moves.
- params: the speed, 5 pixels a frame; the fast fall's frames, 7: the frames the fast fall step
  shows, the frame she reaches it the first of them; no liquid slows it.
- switch: parameter (the speed and the fast fall's frames).
- Her clip with a fast fall step, her step on any step before it, whatever the time left, her
  vertical speed at the speed or more either way (its size, rising or falling) → the fast fall
  step, and the frame ends with the fast fall's frames left: the time left is those frames plus 1
  before the time runs down this frame [C-fast-fall, C-fast-fall-early, C-fast-fall-liquid,
  C-fast-fall-rising, C-fast-fall-adjusted]
- So short of the speed either way, or on the fast fall step or past it, the clip plays as it
  would [C-fast-fall-slow, C-fast-fall-rising-slow, C-fast-fall-on, C-fast-fall-past].
- So a fall's pose rising at the speed or more (an unmorph's end that carries such a rise into
  the crouch, then a walk off a ledge) shows the fast fall picture too [C-fast-fall-rising]; no
  record rises so fast in a fall's pose (the fastest is 2.4000).
- checked: oracle (4 on record); constructed (C-fast-fall, C-fast-fall-early, C-fast-fall-liquid,
  C-fast-fall-slow, C-fast-fall-rising, C-fast-fall-rising-slow, C-fast-fall-on,
  C-fast-fall-past); adjusted, the speed at 3 pixels a frame and the fast fall's frames at 11
  (C-fast-fall-adjusted).

### RisingHold
- kind: mechanic.
- summary: a straight jump holds its rising picture for as long as she rises.
- params: the rising frames, 3: the frames the rising step shows again each time she still
  rises on its last frame, that frame the first of them; no liquid slows it.
- switch: parameter (the rising frames).
- Her clip with a rising step, on it with 1 frame left, not heading down → the frame ends on it
  with the rising frames left: the time left is those frames plus 1 before the time runs down
  this frame [C-rising, C-rising-adjusted]
- So with more time left on it, it counts down as usual [C-rising-early].
- So she stays on the rising step the rising frames at a time until she heads down, then the
  clip goes on [C-rising-down].
- checked: oracle (20 on record); constructed (C-rising, C-rising-early, C-rising-down);
  adjusted, the rising frames at 6 (C-rising-adjusted).

---

## 5. Her run

### RunPace
- kind: mechanic.
- summary: running while dashing, her stride takes the run's pace, plain without the speed
  booster and quickening with its stages; with run held each lap of her run starts over on the
  pace.
- params: none (the paces are content, §1.4).
- switch: none.
- "Dashing" here is the dash as the clip moves on (§1.3): on, began, lost, or began and lost.
- Running and dashing, the clip moving on to a shown step → its hold is the pace's for that
  step, plus the step slowdown: the plain pace without the speed booster, the boosted pace of
  her stage with it [C-pace-plain, C-pace-boost, C-dash-lost-moving-on, C-dash-began-and-lost]
- So dashing in any other action, a jump or a fall, her clip's own holds show, as they do when
  her dash is off or stopped [C-dash-jump, C-not-dashing, C-dash-stopped].
- A lap is her running clip from step 0 to the first control the clip moves on to. Any control
  ends it: a rest, an `again` to any step, a `breathe`, an end or a branch [C-lap-at-rest,
  C-lap-again-mid, C-lap-count-rest].
- Running and dashing, run held, without the speed booster, at a lap's end → the lap starts
  over: step 0, with the plain pace's first hold plus the step slowdown [C-lap-plain,
  C-lap-again-mid, C-lap-at-rest]
- So dashing with run let go, a lap ends as the clip says, on the clip's own hold
  [C-lap-not-held].
- checked: oracle (760 plain and 715 boosted holds, 58 laps on record); constructed (C-pace-plain,
  C-pace-boost, C-lap-plain, C-lap-not-held, C-not-dashing, C-dash-jump, C-dash-stopped,
  C-dash-began-and-lost).

### BoostStages
- kind: mechanic.
- summary: with the speed booster, every lap run with run held counts towards the next stage;
  four stages on she is boosting, and laps go on at the boosting pace.
- params: the laps each stage lasts, 1 at stages 0 to 3 and 2 at stage 4; an adjusted game may
  give each stage 1 lap or more.
- switch: parameter (the laps per stage).
- A dash began (began, or began and lost) → stage 0, with stage 0's laps left with the speed
  booster, 0 without it, before the clip moves on [C-began-booster, C-began-plain, R-began,
  C-laps-adjusted-began, C-dash-began-and-lost]
- Running and dashing, run held, with the speed booster, at a lap's end (`RunPace`), the laps
  left less 1 above 0 → that is the laps left, and the control does as the clip says
  (`CountedLapHold`) [C-lap-count, C-lap-count-rest]
- So with run let go the laps left stay as they are, no stage goes up, and the control does as
  the clip says [C-lap-not-held-booster, C-stage-not-held].
- The same, the laps left less 1 at 0, the stage below 4 → the stage goes up by 1, the laps left
  are the new stage's, and the lap starts over: step 0, with the new stage's pace's first hold
  plus the step slowdown [C-stage-up, C-laps-adjusted-up]
- The same at stage 4 → the laps left are stage 4's, and the lap starts over: step 0, with the
  boosting pace's first hold plus the step slowdown [C-boosting-lap, C-laps-adjusted-boosting]
- The stage going up to 4 → asks for `Boosting` [C-boost-quirk-0]
- A dash lost (lost, began and lost, or stopped) → stage 0, with 0 laps left, as the frame ends
  [C-lost, C-dash-lost-moving-on, C-dash-began-and-lost, C-dash-stopped, R-lost-a, R-lost-b]
- checked: oracle (96 began, 50 stages up, 16 laps counted, 109 lost on record); constructed
  (C-began-booster, C-began-plain, C-lap-count, C-lap-count-rest, C-lap-not-held-booster,
  C-stage-not-held, C-stage-up, C-boosting-lap, C-lost, C-dash-began-and-lost, C-dash-stopped);
  adjusted, the laps per stage at 3, 2, 4, 1 and 3 (C-laps-adjusted-began, C-laps-adjusted-up,
  C-laps-adjusted-boosting); recorded (R-began, R-lost-a, R-lost-b).

### CountedLapHold
- kind: quirk.
- summary: a boosted lap that only counts down her laps goes on at the clip's own hold, not the
  pace's, until the next shown step.
- params: none.
- switch: none.
- A lap's end that only counts (`BoostStages`: laps left less 1 above 0) → the control does as
  the clip says, and the step it leads to takes the clip's own hold plus the step slowdown, not
  the boosted pace's [C-lap-count]; a rest leaves no time left, as it does anywhere
  [C-lap-count-rest]
- So the pace's hold returns with the next shown step the clip moves on to (`RunPace`), and a
  lap's end that raises the stage, or that restarts a lap at stage 4 or without the speed
  booster, starts over on the pace instead.
- checked: oracle (16 laps counted on record, each on the clip's own hold); constructed
  (C-lap-count, C-lap-count-rest).

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
  (`RunPace`), and its lap end starts the next at the boosting pace [C-boosting-first-lap].
- checked: oracle (5 on record, each with 0 waiting); constructed (C-boost-quirk-0,
  C-boost-quirk-2, C-boost-quirk-4, C-boost-quirk-6, C-boosting-first-lap).

---

## 6. Order that matters

A frame has two halves, because samus-movement reads the end before it picks her pose, and this
domain reads the pose it picks:

- **First, before samus-movement's step:** the read-ahead (§1.1): the end (`ClipEnds`), from her
  step, the time left and her clip at the frame's start, and whether her spin has turned far
  (`SpinTurnedFar`), from her step and that clip. §1.5 makes the end exact: no move of the frame can change whether her
  clip ends; turned far reads nothing the frame changes.
- **Then, with samus-movement's results and the pose she takes**, in this order, as the State at
  the frame's end shows:

1. Her moves' changes: a dash's beginning (began, or began and lost), a standing restart, a
   readied wall jump, a fast fall.
2. The rising hold.
3. The time runs down and the clip moves on (§2, §5), from her pose at the frame's start.
4. The start of the clip of the pose she takes (`StartingClip`, `HandOverSlowdown`).
5. A dash's loss (lost, began and lost, or stopped, §1.3).

So the restart's, the fast fall's and the rising frames count the frame that sets them (step 3
runs that frame's time down), a readied spin shows its kick step the same frame, and a clip that
ends hands over in the frame it ends [R-restart-a, C-ready, C-end].

## 7. Out of scope, and who supplies it

- **Part 2 of this domain:** her sprite (the picture each step shows, where its halves sit, what
  it uploads), her suits and active palette effects (visor, charge, booster glow, screw attack,
  the colour effects that own her colours outright), splashes and dust, and the echoes'
  pictures. They read her step and the boost stage.
- **Poses other domains drive**, whose clips those domains step or set: the x-ray scope, the
  grapple beam, the shinespark (its windup sets the stage to 4 and its end clears it; the crouch's
  shine keeps it), a metroid draining her, the crystal flash, her
  appearance at the start, the death, and an item switched on or off mid-pose by the pause screen.
- **The liquids:** the two slowdowns and whether she is submerged.

## 8. Open questions

1. The laps left at 0 with the speed booster (a dash begun without it, then the booster switched
   on mid-run) is outside the inputs' domain (§1.5); the pause screen's equipment change is out of
   scope.


## 9. Sounds at her stride, pose selection and drawing

These opportunities belong to how Samus looks: a foot contacting the ground is timed by the
picture still showing, a spin starts with the figure she takes, and a spin ends while that
figure is drawn. Each is a rule of the frame that reads the frame's inputs (§1.3); the
implementer chooses the interface form. None advances the animation, changes its four
quantities, or consumes a sound acknowledgement. Each applies at most once a frame. Several may
ask for a sound in the same frame; the animation update's at-most-one sound does not limit them
(§9.2).

### 9.1 Content, State and sounds

| Quantity | Type and owner | Read by |
| --- | --- | --- |
| contact steps | a set of shown step indices in each clip, content supplied by the importer; any number of contacts, independent of holds and the kick mark | `FootContact` |
| her step, the time left and the boost stage, as the frame starts | the State (§1): her running movement doesn't change them, and the frame's animation update comes after the stride; at stage 4 she is boosting | `FootContact` |
| her kept action | State: the action her last prepared frame kept (§9.2), an action of §1.2; independent of pose-change history and of her pose as the frame starts | `SpinEnds` |

A contact opportunity returns a graphical contact opportunity separately from its optional sound.
Part 2 reads the graphical opportunity with the room and liquid context to decide the splash
or dust. Cinematic, boss and special colours suppress its sound only. This amendment does not
replace the existing atmospheric drawing or add liquid graphics. Its bounded dry Ceres
admission has no boosting, so the graphical opportunity leaves atmospheric slots and sprite output unchanged;
it must still remain available to part 2.

The domain's sounds are semantic requests, one set whichever rule asks for them:

| Sound | Offered to, with its allowance | Asked for by |
| --- | --- | --- |
| `Footstep` | the footstep sounds, 6 | `FootContact` |
| `Boosting` | the footstep sounds, 6 | `BoostStages` |
| `PlainSpin`, `SpaceJump`, `ScrewAttack` | Samus sounds, 6 | `WallJumpBranch` (a wall jump's branch) and `SpinStarts` (the spin she takes): each is the same request whichever asks |
| `SpinEnded` | Samus sounds, 15 | `SpinEnds`, `AtTheDoor` |
| `SamusSilence` | Samus sounds, 15 | `AtTheDoor` |
| `Hurt` | Samus sounds, 6 | `HurtColours` (§12) |

Appearance only reports each request, with its queue and allowance as the table gives them. Whether
a request is admitted is the sound domain's rule ([audio-transport](audio-transport.md)
`OfferAcceptance`: a queue takes an offer while it holds fewer sounds than the allowance), as are
sending, retries and driver replies; no rule here reads whether a request was admitted. An
outstanding retry is not another appearance sound. Callers keep the frame's requests in the order
of §9.2.

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

A qualifying selection means her pose after differs from her pose (§1.2, the sprite included),
its action is `Spinning`, and she is not submerged. "Already spinning" means her pose's action
is `Spinning` or `WallJumping`. A qualifying selection is decided in this order, the first that
applies deciding; the space jump comes before "already spinning", so it alone is asked for on
a spin that goes on:

1. With ScrewAttack → `ScrewAttack` unless already spinning, whatever the cinematic.
2. Else with SpaceJump → `SpaceJump`, already spinning or not, whatever the cinematic.
3. Else → `PlainSpin` unless already spinning or a cinematic is active.

- Qualifying selection, with SpaceJump and without ScrewAttack → ask for `SpaceJump`
  [C-start-space, C-start-space-continued, C-start-space-cinematic]
- Qualifying selection, with ScrewAttack, not already spinning → ask for `ScrewAttack`
  [C-start-screw, C-start-both, C-start-screw-cinematic]
- Qualifying selection, with ScrewAttack, already spinning → no sound
  [C-start-screw-continued]
- Qualifying selection, with neither item, not already spinning, cinematic inactive →
  ask for `PlainSpin` [C-start-plain, R-start]
- Qualifying selection, with neither item, already spinning or cinematic active → no sound
  [C-start-continued, C-start-wall, C-start-cinematic, C-start-continued-cinematic]
- Her pose after the same as her pose, no spinning selection, or submerged → no sound
  [C-start-same, C-start-other, C-start-submerged, C-start-submerged-both]
- checked: constructed (the cases above); recorded (R-start). `WallJumpBranch` asks for the
  same spin sounds at a different moment of a frame, a wall jump's branch in the clip of her
  pose as the frame starts.

### SpinEnds
- kind: output.
- summary: her prepared frames keep her action, and when she has left a kept spin or wall
  jump, normal drawing ends its sound, even while her animation is frozen.
- params: none.
- switch: none.
- The frame's work prepared → her kept action becomes her pose's action, before normal drawing
  reads it [C-end-prepared, C-end-prepared-replaces, R-unspin]
- So a frame without preparation leaves her kept action as it is, as a paused frame does
  [C-end-frozen, C-end-spin, C-end-wall].
- Normal drawing, her kept action `Spinning` or `WallJumping`, her pose after's action
  neither → ask for `SpinEnded` [C-end-spin, C-end-wall, C-end-frozen, C-end-prepared, R-unspin]
- Her kept action neither, or her pose after's action `Spinning` or `WallJumping` → no sound
  [C-end-neither, C-end-spin-spin, C-end-spin-wall, C-end-wall-spin, C-end-wall-wall,
  C-end-prepared-replaces]
- checked: constructed (the cases above); recorded (R-unspin).

### AtTheDoor
- kind: output.
- summary: on first entering a passage, Samus stops her spin sound or her quiet uncharged
  sound before sounds are held.
- params: none.
- switch: none.
- First passage opportunity, her pose after's action `Spinning` or `WallJumping` → ask for
  `SpinEnded`, whatever shoot and charge [C-door-spin, C-door-wall]
- Her action neither, shoot released, charge below 16 frames → ask for `SamusSilence`
  [C-door-quiet, C-door-below, R-door]
- Her action neither, shoot held or charge 16 frames or more → no request
  [C-door-held, C-door-threshold, C-door-above]
- checked: constructed (the cases above); recorded (R-door).

### 9.2 The frame's sounds, in order

A frame of play gives this domain's sounds as one list, in the order they are asked for, with
the graphical contact beside it:

1. the stride's (`FootContact`), from the frame's start, after her running movement and before
   the animation update;
2. the animation update's own, at most one (`WallJumpBranch`, `BoostStages`);
3. the spin start of the pose she takes (`SpinStarts`), after the clip of her pose at the
   frame's start has played (§6, 3) and before the clip of the pose she takes starts (§6, 4);
4. normal drawing's spin end (`SpinEnds`), with her kept action and her action as the frame
   ends.

A frame in which she first enters a passage asks for `AtTheDoor`'s request once, before the
passage holds its sounds, and then normal drawing's. Animation and motion may stay frozen in
such a frame: it has no animation update, and normal drawing still compares her actions. Sound
retries are the sound domain's. None of these needs another motion step or another animation
update [C-end-frozen, C-contact, C-door-quiet].

An invented frame combines the wall-jump branch sound with drawing's spin end; another
combines a contact and a spin start. Each is one frame with one set of inputs (§1.3), keeps both
requests in this order and the graphical contact, and compares the State it leaves, with one
animation update [C-branch-and-end, C-contact-and-start].

Her kept action is her appearance State. The frame's preparation of her work keeps her action
as the frame starts, before her moves and the pose she takes; normal drawing reads it later in
the frame without refreshing or consuming it. It is independent of pose-change history. Her
pose as the frame starts is not a substitute for it: a frame without preparation keeps the
earlier action, so drawing compares her action as the frame ends with the action of the last
prepared frame.

Frozen animation alone does not decide whether the kept action changes. A frame whose work is
locked but still prepared keeps her action; a paused frame without that preparation keeps the
earlier one. The extra inanimate body draw keeps no action and asks for no second sound. If
preparation is skipped and normal drawing repeats on later frames, each normal drawing reads
the same kept action, without another motion or animation update.

## 10. When the cannon may follow her selected item

The HUD owns the selected item. Her look owns the delay before her cannon may follow that
item: a selection change can leave the cannon showing its previous open or closed state for
one more frame of item controls. This is independent of the animation playhead, boost and sounds.
Opening or closing the cannon, its pictures, and which selected items need it open remain
downstream appearance rules; readiness does not assert that the cannon has finished moving.

| Quantity | Units and domain type | Why the player can tell | Read by |
| --- | --- | --- | --- |
| cannon selection delay | `CannonDelay`, 0 to 2 frames of eligible item controls remaining | her cannon can keep its previous open or closed state while the newly selected item is already highlighted | `CannonItemDelay`, `CannonReadinessReset`; output (the picture of her gun) |
| cannon may follow the selection | readiness: the delay, as the frame's item controls leave it, is zero | the cannon may begin responding to the currently selected item | output, that frame (the picture of her gun) |

The delay is separate persistent appearance memory. A caller can construct it at every value
in its range, read it, and save and restore it with the rest of the look; a caller carrying the
whole look also carries cannon readiness. It is not reconstructed from the selected item,
whether her gun is open, a frame number, or an event retained by the HUD.

**Inputs and neighbours.** A frame whose item controls run supplies whether controls are
eligible (the HUD's `can switch`) and the selected item identities immediately before and
after those controls. An identity is none, missiles, super missiles, power bombs, the x-ray
scope or the grapple beam. The HUD's one-shot mode is independent: `Item(item)` and
`OneShot(item)` have the same identity. The event `Switched` also is independent: item cancel
on none and cycling back to the same available identity still report an event.

These identities belong to the item controls' own answer and the selection they start from.
They are not the displayed highlight, an animation figure, or a selection read after a later
weapon drop. Controls that cannot switch do not advance this clock. Its unit is a frame whose
eligible item controls run, not a video interrupt, a lag frame, an animation update or a
drawing. Frozen time alone does not stop eligible controls. A frame whose controls are skipped,
including facing the screen or being controlled by another domain, leaves it alone.

### CannonItemDelay
- kind: mechanic.
- summary: after her selected item changes, the cannon waits for one more frame of eligible item controls before it may follow that item.
- params: none.
- switch: none.
- Eligible controls run with different selected item identities before and after them → the delay becomes 1, including a change to none and another change during a delay [C-cannon-change, C-cannon-deselect, C-cannon-rechange, C-cannon-drop-after-change, R-cannon-select, R-cannon-deselect].
- Eligible controls run with the same selected item identity → the delay becomes `max(0, delay - 1)`, irrespective of a same-item switch event or a change in one-shot mode [C-cannon-change, C-cannon-same-event, C-cannon-mode-only, C-cannon-ready, R-cannon-follow, R-cannon-same-item].
- Controls are ineligible or are skipped, or the selection changes outside item controls → the delay stays as it is [C-cannon-disabled, C-cannon-skipped, C-cannon-frozen, C-cannon-external-drop, C-cannon-drop-after-change, R-cannon-lag].
- checked: oracle (the R cases); constructed (the C cases).

### CannonReadinessReset
- kind: mechanic.
- summary: a newly initialized Samus needs two frames of eligible unchanged item controls before her cannon may follow the selection.
- params: none.
- switch: none.
- Her initialization (§1.3's part of its own, §1.6, which sets her whole State) → the delay becomes 2 [C-cannon-initialize, C-cannon-reset-during-wait, C-initialize].
- A pose or clip changes, drawing repeats, the game lags or a passage only moves her to another room without reinitializing Samus → keep this delay; only `CannonItemDelay` and actual initialization change it [C-cannon-unrelated, C-cannon-skipped, R-cannon-lag].
- checked: original only (initialization); constructed (the C cases). No recorded initialization-delay history is claimed.

**Order that matters.** A frame's eligible item controls change the delay before her gun's
picture reads it that frame. A changed selection has delay 1 in the frame of those controls. At
the next frame of eligible unchanged controls it reaches zero, so that frame's picture may follow
the item. A selection drop outside item controls, before or after them in the frame, does not
itself age or restart this clock; drawing reads the current selected item with the delay as it
stands [C-cannon-external-drop, C-cannon-drop-after-change]. All cannon animation, weapon
selection, weapon-charge effects and suit colour changes and rendering outputs remain their
neighbouring contracts.

The `cannon_readiness` vectors in the appearance vector file hold histories of these frames.
They supply item identities and one-shot mode separately and compare the delay after every
operation. The implementation chooses the interface and shape; no source counter, callback,
method list or original state encoding is specified.

## 11. Samus's colours during ordinary dry movement

Appearance owns the visor sequence and charge-glow position, alongside animation and
cannon readiness. The frame's palette state includes both ordered palettes.
Keep a single current palette and a single palette for target colours shared by their producers and
consumers. Appearance's ordinary reset needs no reads or writes of either palette. Room
palette objects and other producers make their own changes to those same palettes, in the
reset's frame as in any other. Video reads the current colours for presentation, and colour
transitions can read the target colours. The reset keeps whatever colours the frame's other
producers leave, at their exact slots, and asks for no colour replacement, graphics, sound or
hardware request.

### 11.1 Persistent quantities

| Quantity | Type and units | Read by |
| --- | --- | --- |
| visor delay | `VisorDelay`, 0 to 255 eligible palette ticks remaining | `OrdinaryDryColours`; output, that frame (the visor sequence) |
| visor sequence position | `VisorStep`, entry in the visor's colour sequence, numbered from zero | `OrdinaryDryColours`; output, that frame (the visor sequence) |
| charge-glow position | `ChargeGlowPosition`, a picture in the charge glow's sequence, numbered from zero, with zero the reset position | `OrdinaryDryColours`; output, that frame (the charge-glow sequence) |
| current colours | `Palette`, 256 ordered RGB colours with 0 to 31 levels per component | output (video and colour transitions) |
| target colours | `Palette`, 256 ordered RGB colours with 0 to 31 levels per component | output (colour transitions) |

A caller can supply and read back the entire palette state, including when the palettes
are stored in a shared owner. A save and restore keeps both palettes at their current values
along with Appearance's three sequence quantities. A reset charge-glow position describes the
sequence's first picture. The current beam and effect facts govern whether a glow is active.
A frame applies the reset at most once; whether it applied is an output of the frame, and
applying it can leave the three quantities as they were.

### 11.2 Current observations

The rule reads two of the frame's inputs (§1.3): her pose after, the pose this frame's movement
selected (its body and action), and the items switched on (the screw attack). Its other
observations are its own: each is its owner's value for the frame's palette opportunity, the
suit selection made for this frame, and the others as their owners have them by then. Where
she is doesn't enter: the water and lava levels say whether a liquid is present.

| Observation | Type and owner | Read by |
| --- | --- | --- |
| current suit selection | `Suit` (Power, Varia or Gravity), the actual equipment-dependent selection supplied for current use | `OrdinaryDryColours` |
| room colour treatment | `RoomColours` (standard, red background or orange background), from room effects | `OrdinaryDryColours` |
| water and lava levels | independently present or absent, from room effects and the current Environment; the lava level is the surface of lava or acid | `OrdinaryDryColours` |
| grapple activity | dormant or busy, from Grapple | `OrdinaryDryColours` |
| lingering glow after firing and weapon charging | independently inactive or active, from the shot/charge owners | `OrdinaryDryColours` |
| appearance override and special colouring | independently inactive or active, from the current colour-effect owners | `OrdinaryDryColours` |
| speed glow and hit colour pulse | independently inactive or active, from current boost activity and the hurt count (§12, active above 0) | `OrdinaryDryColours` |

Use the frame's suit selection. With Gravity enabled it is Gravity, otherwise with Varia
enabled it is Varia, otherwise it is Power. A suit value from before this frame's selection
cannot stand for it, even if it describes the same suit. Similar-looking
facts such as a camera's tether-related scrolling status or which layers haze blends do not establish
the grapple or room effects observations above.

### OrdinaryDryColours

- kind: mechanic.
- summary: during ordinary dry movement the visor and charge-glow sequences are readied while the present and target colours stay as they were.
- params: none.
- switch: none.
- The frame's palette opportunity sees her pose after upright and running, standing still, moonwalking, in a straight jump, turning in a take-off or a straight jump (`Turning(Jump)`), landing upright from a straight jump or a fall (`Landing(Jump)`), or spinning while the screw attack is switched off; in the Power suit, standard room colours, neither water nor lava present, dormant grapple, and all six current activity observations inactive → set visor delay to one eligible palette tick, visor position to the fourth entry (zero-based entry 3), and charge-glow position to reset. The input sequence positions and delay may differ from these values. Leave all 256 current and 256 target colours at their positions as the frame's other producers leave them, even with different valid poses of those actions or initial palettes. Every admitted action except the spin is admitted with the screw attack on or off. A spin without the screw attack readies the sequences exactly as a run does: neither the spin nor its direction changes the three quantities or either palette [R-ordinary-run-colours, R-ordinary-spin-colours, R-ordinary-jump-colours, R-ordinary-jump-turn-colours, R-ordinary-landing-colours, R-ordinary-still-colours, R-ordinary-moonwalk-colours, C-ordinary-stale, C-ordinary-other-stride, C-ordinary-already-ready, C-ordinary-spin-stale, C-ordinary-spin-other-way, C-ordinary-jump-stale, C-ordinary-jump-turn-stale, C-ordinary-landing-stale, C-ordinary-still-stale, C-ordinary-moonwalk-stale, C-ordinary-screw-jump, C-ordinary-shared-colours].
- Every fact read, a premise of that arm not holding → refused, an ordinary result: nothing
  applied [C-ordinary-boundary, C-ordinary-spin-boundary]
- A fact the rule can't read → unrecognized: nothing applied, and the frame is outside what this
  rule decides [C-ordinary-unreadable]
- checked: oracle (recorded application and preservation for each admitted action, R-ordinary-run-colours to R-ordinary-moonwalk-colours); original and constructed (stale reset, varied inputs, refusals and unreadable facts).

**Two ways a frame applies nothing.** A frame's palette opportunity ends one of three ways:
applied (the arm above), refused, or unrecognized.

- **Refused**, an ordinary result of the frame: every fact is read and the arm's premises don't
  all hold. Among them: other actions (falling, crouching, the morph and spring balls, taking
  off, standing up, wall jumps, turning on the ground or in a fall, landing from a spin or a
  wall jump, and any action of a crouched or ball figure), a spin while the screw attack is
  switched on, other suits, red or orange room backgrounds (which can advance the visor
  sequence's clock themselves), water or lava present, busy grapple, active stored glow, weapon
  charging, special colouring, appearance override, speed glow or hit colour pulse
  [C-ordinary-boundary, C-ordinary-spin-boundary].
- **Unrecognized**: a current fact the rule can't read, an unavailable or unrecognized
  observation, or colours it can't represent. The frame is outside what this rule decides
  [C-ordinary-unreadable].

Either leaves the three quantities and both palettes as they were and applies nothing. A frame
without a palette opportunity, or drawing alone, does not apply this rule.

**Order that matters.** The frame's palette opportunity comes after movement has selected her
pose and the suit has been selected, and before health maintenance and the
frame's later drawing and palette upload, which read the three quantities as the reset leaves
them. It does not advance animation, cannon readiness, movement, room palette objects or
health. The model chooses its API and representation. The palette histories in the vector file
specify the visible outcomes.

## 12. Her hurt colours and the push's poses

When her energy falls (Ridley's tail, [samus-health](samus-health.md) `TailHit`) or a push
starts ([samus-movement](samus-movement.md) §12, `PushStarts`), her colours pulse: three frames
in her hurt colours, each followed by a frame in her suit's, then her suit's colours hold while a
count runs out. The second frame asks for the hurt sound. The push's two poses, `Recoiling` and
`Leaping`, are ordinary poses here: their clips are content, and they start and play as every
clip does (§2, §3).

**What it reads.** Her current energy as the frame leaves Health and the energy the frame before
left (`EnergyAmount`, whole energy; samus-health's output, as `Breathing` reads it); whether a
push started this frame (samus-movement §12's output "push started"); whether a cinematic is
active; the current suit selection (§11.2); her pose after, and whether she spins (§1.2); and
whether the grapple is busy. Content (the importer): her
hurt colours, each suit's colours and the cinematic's grey colours, 16 colours each, the colours
of her own row of colours (the fifth of the eight sixteen-colour rows sprites draw from).

**What it keeps.** The hurt count (`HurtCount`, 0 to 59 frames): State, given and read back by a
caller with the rest of appearance's State. It is the "hit colour pulse" §11.2 reads: active
while the count is above 0.

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| hurt count | `HurtCount`, frames | which frames show her hurt colours, and when the pulse is over | `HurtColours`; §11.2's hit colour pulse; output |
| her sprite's colours | 16 colours of the current colours (§11.1), her row | what she is drawn in | output (video) |

### HurtColours
- kind: timing and output.
- summary: a fall in energy or a push's start pulses her colours three times and asks for the
  hurt sound.
- params: none; the colours are content.
- switch: none.
- A push started this frame → the count is 1; else her energy fell this frame and the count is 0
  → the count is 1; her energy falling with the count above 0 changes nothing [RH1,
  C-hurt-fall-only, C-hurt-pushed-again, C-hurt-fall-mid-pulse]
- Then, the count above 0 → by the count: 1, 3 or 5 → her row of the current colours takes her
  hurt colours; 2, 4 or 6 → it takes her suit's colours (the frame's suit selection), or, with a
  cinematic active, the cinematic's grey colours; 7 or more → it is left as it is. The target
  colours are left as they are [RH1, RH2, RH3, RH6, RH7, RH8, C-hurt-cinematic]
- The count 2, with no cinematic active → ask for `Hurt` [RH2, C-hurt-cinematic]
- Then the count goes up by 1; reaching 40, the grapple dormant, spinning (her pose after a spin
  or a wall jump) → ask for her spin's sound again (`PlainSpin`, `SpaceJump` or `ScrewAttack`, as
  `SpinStarts` chooses it); reaching 60 → the count is 0 [RH4, C-hurt-spin-again, RH5]
- The count 0 and no fall and no push → nothing changes [C-hurt-idle]
- Out of scope (§7), none of them in Ceres: a frame whose colours another of her effects owns
  outright (the shinespark's, the crystal flash's, the final weapon's), which leaves the count as
  it is; at 2, the `Hurt` request skipped while the final boss drains her; at 40, a powered shot's
  hum coming back, and the tether's sound while the grapple is busy.
- checked: oracle (0006's five hits, all alike: RH1-RH8); constructed (C-hurt-fall-only,
  C-hurt-cinematic, C-hurt-spin-again, C-hurt-pushed-again, C-hurt-idle, C-hurt-fall-mid-pulse).

The sound `Hurt` joins §9.1's table: Samus sounds, allowance 6.


## 13. Her standing picture on the elevator

The elevator scene supplies each locked room drawing opportunity. Movement supplies her
standing pose and whole room centre. Her clip supplies its current displayed step.
The shaft supplies the view carried into the room turn. The camera supplies the layer origin
at drawing. Imported content provides a picture and tiles for each supported standing look.
This section covers the ordinary Power suit with no active damage flicker. The timer and room
actors supply their own sprites. Video owns the pending and published pictures, their tile transfers and the
object buffer. The station's drawing replaces the room picture as its own setup finishes.

| Quantity | Domain type and units | Read by |
| --- | --- | --- |
| Standing pose | `Pose` from [samus-movement §2](samus-movement.md) | ElevatorStandingPicture |
| Standing look | Side (`Left` or `Right`) and displayed clip step counted from zero | ElevatorStandingPicture |
| Room centre, pivot and camera origin | Pairs of whole pixels in the room | ElevatorStandingPicture |
| Carried shaft view | Signed 8.8 diagonal and cross terms for the rotating floor | ElevatorStandingPicture |
| Vertical art displacement | Signed whole pixels from pose content, positive when the art origin is below her centre | ElevatorStandingPicture |
| Body pieces | Two ordered lists of sprite offsets in whole pixels, with tile, palette, size, priority and flips | ElevatorStandingPicture |
| Body tiles | Imported tile strips for the upper and lower body rows. Each half has a first strip and an optional second strip. | ElevatorStandingPicture |
| Body placements and graphics | Ordered object placements in screen pixels and graphics for the object's reserved tiles | output (video) |

### ElevatorStandingPicture

- kind: mechanic.
- summary: Samus keeps the standing picture for her current pose and clip step as the shaft tilts behind her.
- params: none.
- switch: Fixed.
- A locked drawing opportunity with an upright `Still` pose facing left or right, aim
  `Forward`, legs `Together` and cannon `Stowed` → use that facing and displayed step to
  select the art displacement, upper and lower pieces and body tiles from content. Write the
  upper pieces first and then the lower pieces, each at its own offset from the same projected
  origin. Carry each piece's tile, palette, size, priority and flips into its placement. Request
  the tile strips for their respective reserved object tiles [C-elevator-body-left,
  C-elevator-body-right, R-elevator-body].
- Any other pose at a new locked drawing opportunity → report an unsupported pose at the
  drawing boundary [C-elevator-body-outside-pose].
- An admitted look with no art at its exact facing and step → report missing art at the drawing
  boundary [C-elevator-body-missing-step].
- A cut that resumes suspended work without a new room drawing opportunity → submit no new
  body placements or tile strips [C-elevator-body-suspended, R-elevator-body-suspended].
- checked: oracle (shaft cut 165 and subsequent locked turns through final publication at cut 241), constructed.

With room centre `(x, y)`, shaft pivot `(p, q)`, diagonal `d` and cross term `s`, let
`u = x - p`, `v = q - y`. The projected room point is
`(p + floor(d*u/256) + floor(s*v/256), q - floor(-s*u/256) - floor(d*v/256))`.
Subtract the camera origin, then add the signed vertical art displacement to the vertical position.
The projection changes the drawing alone. The room centre still governs movement and the mini-map.

**Order that matters.** The body follows the frozen countdown and precedes room actors
in the drawn picture. It reads the view before the shaft takes its late tilt step.
An elevator reach during ordinary play supplies the pose for the next drawing opportunity.
Drawing reads that pose and the displayed clip step without advancing its hold.

Video keeps the live picture and tiles while room work is pending. Its next requested
publication installs the prepared pieces and present tile strips. A strip's destination is
reserved for its own row of its own half. An absent strip leaves that row unchanged, and
the other strips still write to their reserved rows. The constructed suspended case follows
the prepared and live values across those video opportunities.


## 14. Her body flickers during protection

An ordinary room drawing opportunity in Ceres can omit Samus's body for one frame.
This decision applies to her whole body, including both halves when her pose has them.
The room still uses her current pose and displayed clip step to request her body tiles.
The room's other sprites have their own drawing rules.

| Quantity | Domain type and units | Read by |
| --- | --- | --- |
| Protected while drawn | `bool`, whether [Health](samus-health.md) has any protection frames left at the draw | `ProtectionFlicker` |
| Push time left | `bool`, whether [Movement](samus-movement.md) has any push frames left at the draw | `ProtectionFlicker` |
| Special suit colour effect is timed | `bool`, whether a special suit colour effect has time left at the draw | `ProtectionFlicker` |
| Parity of finished frames | `Parity`, even or odd at the start of ordinary gameplay work | `ProtectionFlicker` |
| Body drawn | `bool`, whether to submit the body placements for this opportunity | output (room picture) |

### ProtectionFlicker

- kind: timing and output.
- summary: after a hit's push time runs out, protection hides the body on alternate draws.
- params: none.
- switch: Fixed.
- Protection above zero, zero push time, no timed special suit colour effect, and odd
  finished-frame parity → omit the body placements [C-protection-odd, R-protection-first-hidden,
  R-protection-hidden].
- Otherwise → submit the usual body placements [C-protection-even, C-protection-spent,
  C-protection-push, C-protection-colour-effect, R-protection-new-push, R-protection-last-push,
  R-protection-first-visible, R-protection-visible].
- checked: oracle (ordinary Ridley fight cuts with a body drawing opportunity), constructed.

**Order that matters.** Drawing reads protection and push time after any hit this frame and
before the frame spends one of each. One frame left can still affect the current picture.
Parity comes from the count as the frame's gameplay work starts. Push time is separate from
her pose. The hurt colour pulse has its own timing (§12). The decision adds no State.
