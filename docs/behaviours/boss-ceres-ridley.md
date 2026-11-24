# Ceres Ridley and the escape, parts 1 to 3, the Ceres doors, the shaft as she arrives and Ridley's room as she enters: behaviour spec

What the player sees the doors of the Ceres station do: a closed door that slides open as Samus
comes near and shuts behind her once she has gone, the hiss it makes as it opens, the door in the
shaft that becomes a wall once Ridley has fled, and the station shaking during the escape, which the
doors start; and, as a new game begins, the elevator's pad lowering her down the shaft and
letting her go, the shaft's beacon pulsing and its platform turning; and, as she enters Ridley's
room, its door slamming shut behind her and staying locked until Ridley is beaten, the two
walls his getaway puts up, and Ridley waiting in the dark with the baby, which now and then
cries and glows. Written behaviour first,
from the oracle's records over time. Vectors:
[`scenarios/vectors/boss-ceres-ridley.toml`](../../scenarios/vectors/boss-ceres-ridley.toml).

## Header

**Scope: the first of the domain's pieces.** The plan's B8, Ceres Ridley and the escape, is
specced in pieces, each verifiable on its own. This piece is the Ceres doors: the doors between
Ceres's rooms (each faces left or right), the shaft's door (a door before the getaway, a wall
after it), and the shaking the doors start during the escape. The second piece (§7 to §9) is the
shaft as she arrives: the elevator's pad that holds her, lowers her and lets her go, the cover
over the shaft's floor below it, the beacon's colours and the platform's turning look before the
escape. The third piece (§10 to §12) is Ridley's room as she enters it: the room's door, which
slams shut behind her and stays locked until Ridley is beaten; the two walls his getaway puts
up; and Ceres Ridley placed and waiting, with the baby he holds, until the moment he wakes.
Later pieces add the rest: Ceres Ridley's fight from his waking on, and his getaway; the
shaft's rumble before the station blows up and its beacon during the escape; the escape's
steam; the falling debris and the dust.

**What the third piece reads and leaves.** Its door and walls read what a door reads (§1.3) and
whether Ridley is beaten (progress, P1). Ridley reads whether a door's passage is under way
(world-doors) and the random number. It hands the HUD (interface-hud) that a boss is
encountered, Ridley in Ceres, as he is placed; the rule that turns the mini-map off for it,
`BossBlanksTheMap`, is the HUD's own. It hands audio (U1) the music's stop and the baby's cry,
the screen's shaking (R3) the stop of any shake and progress (P1) the stage. Ridley's fight,
his drawing once he wakes and the getaway that places the walls are later pieces (§5).

**What the second piece reads and leaves.** It reads her position from samus-movement (§7.3),
the moment the arrival places the pad, which frames the pad and the cover run and which the
shaft's overlay runs (the enemy framework, C1; the caller steps each piece on those frames,
§7.3), the count of finished frames (the coordinator of time, T1), how far a door's fade has
got (the door passage, world-doors) and the station's stage. It hands samus-movement her height
while the pad carries her and whether she is held (§7.4); her pose as she arrives, what her frame does while
she is held, and the arrival itself are not this domain's (§5).

**What it reads from samus-movement:** her position, in whole pixels, as it was at the end of the
last frame (§1.3). Nothing else of her.

**What it reads from progress (P1) and the escape:** the station's stage (`Stage`, §1.3): calm,
Ridley's getaway, or the escape.

**What it reads from the random numbers:** the frame's random number as the doors read it (§1.3).

**What it reads from the screen's shaking (the room's effects, R3):** how many frames the shake
has left when the door runs (§1.3).

**What the enemy framework (C1) must supply.** A door is one of the room's enemies, and four
things it needs belong to every enemy, so they are the framework's, not this spec's:
1. **Running.** Each frame the enemies run, the framework runs every door once, in the room's
   order of its enemies, and counts the run on the enemy's own frame count. A door runs whether it
   is in view or not. On a frame the enemies don't run (the room loading, part of a door's
   passage), nothing of a door changes (`Waiting`).
2. **Drawing.** While a door is shown, the framework draws its figure (§1.4) at its position,
   relative to the camera, in the doors' palette (`TheDoorsColours`), among the other enemies; a
   hidden door is never shown. The figures are imported content: which sprites, where, flipped how.
3. **Solid contact.** While a door blocks, its box (§1) stops her as a wall does: her motion
   treats it as it treats a solid enemy (samus-movement), her touch does nothing to
   her or to it, and her shots that reach it end there (a plasma beam's go through, as with any
   enemy that doesn't stop plasma) and do nothing to it. While it is passable none of this
   happens (`Solid`). Her motion reads the solid interaction membership selected when the frame
   began. Shots use the separate processing membership and the current door contact state
   before that door runs, as [enemy-interactions](enemy-interactions.md) specifies. Her touch
   has no response. Drawing reads the resulting door state (§4).
4. **Placing.** When a room starts, the framework places each door from the room's content: its
   kind, its facing and its position, in the state `RoomStart` names. The getaway's walls are
   the exception: Ridley's getaway places them (a later piece), in the same state.

The walls' content, unlike the doors', doesn't make them solid to her: a wall that is solid
(only ever `Placed`) meets her shots as a blocking door does, and never stops her motion.

**Not in this spec** (§6 names who supplies each): the framework's four parts above; what the
shake does to the view and how it counts down (R3); the sound queue and what happens to a request
it can't hold (audio, U1); fading the doors' colours in with the room (the door passage,
world-doors); the room's other enemies but Ridley's waiting (§12). The shaft's overlay is part 2's
(§7).

**Notation.** Positions are whole pixels, x rightward and y downward, each a 16-bit number written
in hex (`0123`). A door's position is the centre of its box. Frames are frames the door runs
(Header, the framework's first part).

---

## 1. What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| its kind | `DoorKind`: `Door`, `GetawayWall`, `RidleyDoor` or `ShaftDoor` (below) | whether it becomes a wall, slams shut or stands in for the room's sides | `RoomStart`, `ShaftWall`, `RidleyRoomStart`, `GetawayWalls`, `EscapeShakes`, `TheDoorsColours` |
| its facing | `Facing`: `Left` or `Right`: the way it opens, the side its figures are drawn for | which figures it shows | every entry that takes a figure |
| its position | x, y: px, 16-bit | where it is drawn and where its box is; how near she is | `SheIsNear`; output (drawing, solid contact) |
| its phase | `Phase` (below) | whether it is shown, solid, opening or closing | every entry |
| its wait | frames, 1 to the largest of the poll, the hold, the slam's hold and 2: the frames before its next step; no run ends with a wait of 0 | when it next looks for her or shows its next figure | `Waiting` |

Its box is 16 px wide and 64 px tall, centred on its position: 8 px to its left and right, 32 px
up and down. The box is fixed (the doors' content); only whether it is solid changes (`Solid`).

**Kinds**, listed alphabetically (the order means nothing):
- `Door`: a door between two rooms.
- `GetawayWall`: one of the two walls Ridley's getaway puts up at the sides of his room, `Left`
  or `Right`: shown during the getaway only, and never in her way (`GetawayWalls`).
- `RidleyDoor`: the door of Ridley's room, facing `Right`: it slams shut behind her as the room
  starts and stays locked until Ridley is beaten; then it is a `Door` (`RidleyRoomStart`,
  `SlamShut`, `LockedIn`).
- `ShaftDoor`: the shaft's door: a left-facing door until Ridley's getaway, a wall from then on
  (`ShaftWall`).

**Phases**, listed alphabetically (the order means nothing):
- `Closed`: shown, solid, its closed figure; it looks for her each time its wait runs out.
- `Closing(n)`, `n` from 1 to 4: shown, solid, showing the n-th figure of its closing.
- `Lingering`: the door of Ridley's room just after the room starts: hidden, passable, open.
- `Locked`: the door of Ridley's room, shut while he lives: hidden, solid; Ridley draws it.
- `Open`: hidden, passable; it looks for her each time its wait runs out.
- `Opening(n)`, `n` from 1 to 4: shown, solid, showing the n-th figure of its opening.
- `Placed`: as the room placed it: shown and solid, with no figure; its first run starts it.
- `Slammed`: the door of Ridley's room the frame it is shut: shown, solid; Ridley draws it too.
- `Slamming(n)`, `n` from 1 to 4: shown, solid, showing the n-th figure of its slam.
- `Standing`: a getaway wall during the getaway: shown, passable.
- `Starting`: hidden, passable; it looks for her once, when its wait runs out, to choose its first
  phase.
- `Unseen`: a getaway wall outside the getaway: hidden, passable.
- `Wall`: the shaft's door once Ridley has fled: hidden, solid.

A getaway wall only ever holds `Placed`, `Standing` or `Unseen`; `Lingering`, `Slamming(n)`,
`Slammed` and `Locked` belong to the door of Ridley's room alone. Any other pairing of kind and
phase is not one this spec allows (§1.2).

**Figures**, by look and facing: `Closed`, `Ajar1`, `Ajar2`, `Ajar3` and `Open` (from closed to
open), and `Side` (a getaway wall), each facing `Left` or `Right`, so twelve figures. Which sprites each is belongs to the doors'
content; this domain only names which one a door holds. Each phase holds one figure:

Cells: phase, look, facing

| Phase | Its figure |
| --- | --- |
| `Placed` | none |
| `Starting` | `Closed`, facing `Right`, whichever way the door faces |
| `Closed` | `Closed` |
| `Opening(1)` to `Opening(4)` | `Ajar1`, `Ajar2`, `Ajar3`, `Open` |
| `Open` | `Open` when it faces right; `Closed` when it faces left |
| `Closing(1)` to `Closing(4)` | `Open`, `Ajar3`, `Ajar2`, `Ajar1` |
| `Wall` | `Closed`, facing `Left` |
| `Lingering` | `Open` |
| `Slamming(1)` to `Slamming(4)` | `Open`, `Ajar3`, `Ajar2`, `Ajar1` |
| `Slammed`, `Locked` | `Closed` |
| `Standing`, `Unseen` | `Side` |

The figure faces the door's way unless the table says otherwise. The figure of a hidden phase
(`Lingering`, `Locked`, `Open`, `Starting`, `Unseen`, `Wall`) is never drawn by the framework, but the door holds it all the same, and it is
compared: a vector's `after` names it.

### 1.2 What it remembers: state a caller can hold

A door's kind, facing and position, which never change, its phase and its wait: nothing else. Its
figure, whether it is shown and whether it is solid follow from its phase and facing. This is state
the engine carries from one frame to the next, saves and restores, and sets when play starts
part-way, so a caller can build a door in any phase with any wait this spec allows and read both
back: a vector's `before` builds it and its `after` reads it.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| where she was | x, y: px, 16-bit: her position at the end of the last frame, before anything moves her this frame (samus-movement) | `SheIsNear` |
| the station's stage | `Stage`, listed alphabetically: `Calm` (before Ridley's getaway), `Escape` (from the escape's start to the end of Ceres), `Getaway` (Ridley flees, the escape not yet started) | `RoomStart`, `ShaftWall`, `EscapeShakes`, `TheDoorsColours` |
| the random number | 16-bit: the frame's random number as it stands when the doors run (the random numbers' owner) | `EscapeShakes` |
| the shake's frames left | frames, 0 to 65,535: how long the screen's current shake has left when this door runs, after any earlier door's start this frame (R3) | `EscapeShakes` |
| Ridley beaten | `bool`: the station's boss counts as beaten, from the escape's start on (progress, P1), as it stands when the door runs | `LockedIn` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| shown | `bool`, from its phase | output: the framework draws its figure (Header) |
| solid | `bool`, from its phase | output: the framework's solid contact (Header) |
| its figure | a figure (§1), or none, from its phase and facing | output: drawing |
| a sound | the doors' opening sound, with a queue limit of 6, or none (`Opening`) | output: audio (U1) |
| a shake | none, or a shake moving diagonally, 1 px for 2 frames or 3 px for 4 frames, of `ShakeLayers` (listed alphabetically): `Back` (the second background and the enemies, not the first background) or `Whole` (both backgrounds and the enemies) (`EscapeShakes`) | output: the screen's shaking (R3) |
| drawn with Ridley | `bool`, from its phase: Ridley's drawing draws it, closed (`DrawnWithRidley`) | output: Ridley's drawing (§12.4) |
| the doors' colours | when the room places a door: which of the doors' two colour sets, `Calm` or `Escape`, the palette it is drawn with takes, and which palette, `Doors` (the one the doors share) or `Own` (a palette of its own) (`TheDoorsColours`); the 15 colours of each set are the doors' content | output: the passage's fade brings the screen to them (world-doors) |

---

## 2. A door's run

### Waiting
- kind: mechanic.
- summary: a door counts its wait off by one each run, and steps when it runs out.
- params: none.
- switch: Fixed.
- A run with a wait over 1 → the wait drops by one, and nothing else changes: no sound, and the
  door looks for nobody [D-start-wait, D-closed-wait, D-opening-wait,
  D-open-wait]
- A run with a wait of 1 → the door steps, by its phase's entry below, takes the figure of its new
  phase and starts its new wait [D-closed-far-stays,
  D-open-near-stays, D-shaft-wall-again]
- So a door that steps into the phase it was in, as a closed door that doesn't see her does,
  starts a whole new wait (D-closed-far-stays).
- On a frame the framework doesn't run the door, nothing of it changes (Header, Running): the
  harness's frames of the room loading and the door's passage.
- checked: oracle (the harness, every run of a door in the cached rows); constructed (every vector).

### SheIsNear
- kind: mechanic and parameter.
- summary: a door sees her when she was within a set distance of its position on both axes at the
  end of the last frame.
- params: the reach, `r` = 48 px.
- switch: **Parameter**, the reach (params).
- On each axis, the gap is the difference of the two 16-bit positions, `(door − her) mod 2^16`,
  taken as its negation `mod 2^16` when its bit 15 is set.
- Both gaps under `r` → she is near [D-closed-near-opens, D-closed-left-of, D-closed-edge-y,
  D-wrap-near, D-start-near-opens, D-reach-adjusted, D-reach-adjusted-both]
- Either gap `r` or more → she is not near [D-closed-edge-x, D-closed-far-y, D-wrap-far,
  D-start-far-closes]
- So a gap of `8000` is not near: its negation is itself (D-wrap-far); and her position left of
  the room (`FFF0`) is 24 px from a door at `0008` (D-wrap-near).
- So her position is the one before she moves this frame: the door that sees her at 47 px opens
  even if this frame's move takes her 50 px away (§4).
- checked: oracle (every look in the cached rows); constructed (the edges, wrapping, the adjusted
  reach).

### RoomStart
- kind: mechanic.
- summary: a door placed by a room's start is shown and solid with no figure for its first frame,
  then hidden and passable until it chooses, two frames later, to stand open or closed.
- params: the poll, `Polling`'s.
- switch: Fixed.
- The room places it → `Placed`, wait 1 [D-placed-calm, D-placed-escape]
- `Placed` steps, a `Door`, or a `ShaftDoor` with the stage `Calm` → `Starting`, its wait the poll
  [D-start-first-run, D-start-left-first-run, D-shaft-before-getaway, D-poll-adjusted-start]
- `Placed` steps, a `ShaftDoor` with the stage `Escape` or `Getaway` → `Wall` (`ShaftWall`)
  [D-shaft-wall, D-shaft-wall-getaway]
- `Starting` steps, she is near → `Open`, its wait the poll, no sound [D-start-near-opens,
  D-start-left-near, D-poll-adjusted-started]
- `Starting` steps, she isn't near → `Closed`, its wait the poll [D-start-far-closes]
- So a door she comes through, near it as the room starts, is open already when she can see it,
  and makes no sound; the far door is closed (D-start-near-opens, D-start-far-closes).
- checked: oracle (every room start in the cached rows); constructed.

### Polling
- kind: parameter.
- summary: a door that is closed or open looks for her every few frames.
- params: the poll, `p` = 2 frames.
- switch: **Parameter**, the poll (params).
- `Closed` steps, she isn't near → `Closed`, its wait `p` [D-closed-far-stays, D-poll-adjusted]
- `Open` steps, she is near → `Open`, its wait `p` [D-open-near-stays, D-poll-adjusted-open]
- checked: oracle (every poll in the cached rows); constructed (the adjusted poll).

### Opening
- kind: mechanic and parameter.
- summary: a closed door that sees her hisses and slides open over four figures, staying solid
  until it is open; then it is hidden and she can pass.
- params: the hold, `h` = 5 frames for each figure.
- switch: **Parameter**, the hold (params).
- `Closed` steps, she is near → `Opening(1)`, its wait `h`, and the doors' opening sound with a
  queue limit of 6 [D-closed-near-opens, D-closed-left-of, D-closed-edge-y, D-wrap-near,
  D-hold-adjusted-opening]
- `Opening(n)` steps, `n` under 4 → `Opening(n + 1)`, its wait `h`, whether she is near or not
  [D-opening-step, D-opening-ignores-her, D-hold-adjusted-opening-step]
- `Opening(4)` steps → `Open`, its wait the poll [D-opening-last, D-opening-last-left,
  D-poll-adjusted-opened]
- So the door takes `4 × h` frames, 20, from the hiss to being passable, solid and shown all that
  while, then is hidden: the open door is the room's own picture behind it.
- checked: oracle (the nine openings in the cached rows); constructed.

### Closing
- kind: mechanic and parameter.
- summary: an open door that no longer sees her shows itself again and slides shut over four
  figures, without a sound and without looking for her until it is closed.
- params: the hold, `h` = 5 frames for each figure.
- switch: **Parameter**, the hold (params).
- `Open` steps, she isn't near → `Closing(1)`, its wait `h`, no sound [D-open-far-closes,
  D-hold-adjusted-closing]
- `Closing(n)` steps, `n` under 4 → `Closing(n + 1)`, its wait `h`, whether she is near or not
  [D-closing-step, D-closing-ignores-her, D-hold-adjusted-closing-step]
- `Closing(4)` steps → `Closed`, its wait the poll [D-closing-last, D-poll-adjusted-closed]
- So a door she turns back to while it closes finishes closing, then opens again at its next look
  (D-closing-ignores-her, then D-closed-near-opens).
- checked: oracle (the closings in the cached rows); constructed.

### ShaftWall
- kind: mechanic.
- summary: once Ridley has fled, the shaft's door is an unseen wall.
- params: none.
- switch: Fixed.
- `Wall` steps → `Wall`, wait 1, whether she is near or not [D-shaft-wall, D-shaft-wall-again]
- So a shaft door that started before the getaway goes on as a left-facing door: only the
  room's start turns it into the wall (`RoomStart`, D-shaft-before-getaway).
- checked: oracle (the shaft's wall in the escape's last two rows, recorded cases R15 and R16);
  constructed.

### Solid
- kind: mechanic.
- summary: a door blocks in the phases that show it, and as the wall; hidden doors that are open
  or starting let her through.
- params: the box, 8 px to the left and right of its position and 32 px up and down.
- switch: Fixed.
- Phase `Closed`, `Opening(n)`, `Closing(n)`, `Placed` or `Wall` → solid; the framework's solid
  contact applies to its box (Header) [D-closed-far-stays, D-opening-step, D-closing-step,
  D-start-first-run, D-shaft-wall]
- Phase `Slamming(n)`, `Slammed` or `Locked` → solid [R-slam-start, R-slam-last, R-locked]
- Phase `Open`, `Starting`, `Lingering`, `Standing` or `Unseen` → passable: she, her touch and
  her shots go through it as through air [D-open-near-stays, D-start-wait, R-first-run,
  W-first-run, W-hides]
- Her touch while it is solid does nothing to her or to it (Header, Solid contact; original
  only).
- checked: oracle (whether it is solid, every frame of the cached rows); original only (her touch); shot selection and consumption are checked by
  [enemy-interactions](enemy-interactions.md).

### Shown
- kind: mechanic.
- summary: a door is shown in the phases it is solid in, except the wall; it is drawn with its
  phase's figure.
- params: none.
- switch: Fixed.
- Phase `Closed`, `Opening(n)`, `Closing(n)` or `Placed` → shown: its figure is drawn (Header,
  Drawing; `Placed` has none, so nothing is drawn) [D-closed-far-stays, D-opening-step,
  D-closing-step, D-start-first-run]
- Phase `Slamming(n)`, `Slammed` or `Standing` → shown [R-slam-start, R-slam-last,
  W-first-run]
- Phase `Open`, `Starting` or `Wall` → hidden: nothing is drawn [D-open-near-stays, D-start-wait,
  D-shaft-wall]
- Phase `Lingering`, `Locked` or `Unseen` → hidden [R-first-run, R-locked, W-hides]
- checked: oracle (whether it is shown, and its figure, every frame of the cached rows).

## 3. The station

### EscapeShakes
- kind: mechanic and parameter.
- summary: during the escape, each door keeps the station shaking: whenever the screen is still,
  a door's run starts a small shake, and now and then a strong one.
- params: the chance of a strong shake, `c` = 128 in 4,096; the small shake, 1 px for 2 frames;
  the strong shake, 3 px for 4 frames.
- switch: **Parameter**, the chance and the two shakes (params).
- Every run of a door, whether it steps or not, with the stage `Escape`, and the shake's frames
  left 0, and the random number's low 12 bits, `random mod 4096`, under `c` → a strong shake
  [D-shake-strong, D-shake-high-bits, D-shake-adjusted]
- The same, with `random mod 4096` `c` or more → a small shake [D-shake-small, D-shake-small-bit8,
  D-shake-wait, D-shake-adjusted-small, D-shake-adjusted-small-pixels]
- The shake's frames left over 0, or the stage `Calm` or `Getaway` → no shake [D-shake-busy,
  D-shake-calm, D-shake-getaway]
- A shake a `RidleyDoor` starts, in any phase, is `Back`; any other door's is `Whole`
  [R-released, R-shake-strong-back, D-shake-strong]
- A `GetawayWall` never starts a shake [W-hides]
- So with two doors in a room, the first to run starts the shake and the second, which reads the
  frames left after it, doesn't (§4).
- checked: oracle (the 860 frames the doors ran in the escape in the cached rows); constructed.

### TheDoorsColours
- kind: mechanic.
- summary: when a room with doors starts, the doors' palette takes the doors' colours, another
  set during the escape.
- params: none (the two colour sets are the doors' content).
- switch: Fixed.
- A door placed, the stage `Calm` → the doors' colours are the `Calm` set, in the `Doors`
  palette [D-placed-calm, W-placed]
- A door placed, the stage `Escape` or `Getaway` → they are the `Escape` set, in the `Doors`
  palette [D-placed-escape, D-placed-getaway, W-placed-getaway, R-placed-escape]
- A `RidleyDoor` placed, the stage `Calm` → the `Calm` set, in its `Own` palette: while Ridley is
  in the room his colours hold the doors' [R-placed-calm]
- checked: oracle (two harness cases, H1 and H2, whose colours are the game's content);
  constructed (which set).

## 4. Order that matters

- The doors run before she moves: the position they read is hers at the end of the last frame
  (`SheIsNear`).
- Her motion uses the solid interaction membership selected when the frame began: a door
  opening this frame still blocks her movement, and a door starting to close is not yet in that
  membership. Shots instead meet each processing door with its current contact state before
  that door runs; earlier actors' changes remain visible. Drawing reads the resulting door state
  (`Solid`, `Shown`, [enemy-interactions](enemy-interactions.md)).
- The doors run in the room's order of its enemies, and a shake one starts is in the frames left
  the next one reads (`EscapeShakes`).
- The opening sound is asked for in the run that takes `Opening(1)`'s figure.
- Part 2: the pad and the cover run after her own frame (§7.3). So on the pad's last run she is
  let go after her frame, and her own motion first runs the next frame; the view and the HUD
  that frame see the height the pad left her at.
- Part 2: the pad runs before the cover, so on the frame the pad sets her on the floor the cover
  sees her there and is gone the same frame (`CoverHides`).
- Part 3: in his room Ridley is the first of the enemies, so the door and the walls run after
  him: the stage and whether he is beaten are as his run that frame leaves them. On the frame the
  escape starts the door is let go and the walls hide (R-released, W-hides).

## 5. Out of scope, and who supplies it

- **The enemy framework (C1):** running each door once a frame the enemies run, in order, and
  counting its frames; drawing its figure; solid contact, her touch and her shots; placing it at
  the room's start (Header).
- **The screen's shaking (R3):** moving the view and the enemies with each shake, and counting its
  frames down.
- **Audio (U1):** the sound queue and its limit.
- **The door passage (world-doors):** fading the doors' colours in with the room; its fade's
  steps, which the beacon reads (§7.3); which of a passage's frames run the enemies, and so the
  shaft's overlay (`WhatRuns`).
- **The random numbers:** the number and when it steps.
- **Progress and the escape (P1, B8's later pieces):** the station's stage.
- **Later pieces of B8:** Ceres Ridley from his waking on (§12): his fading in, his fight, his
  getaway, which places the getaway's walls and sets the stage, and his drawing of the door and
  the baby then; the shaft's rumble and its beacon during the escape.
- **Progress (P1):** whether Ridley is beaten.
- **The HUD (interface-hud):** what a boss encountered does to the mini-map (`BossBlanksTheMap`).
- **The arrival (the new game, P2, and Samus's arrival, F4):** starting the game in the shaft,
  setting her where she stands and her pose facing the screen, the room's music; the moment it
  places the pad (§7.3). Before that the room is loaded (R2); after it the screen's brightness
  comes up, a step every two frames, with no colour changed: that is not a door's fade, and the
  beacon steps through it (§7.3).
- **Her frame while she is held (samus-movement, samus-appearance):** while the pad holds her
  (§7.4), nothing of her moves or changes but what the pad does to her height; the mini-map
  still follows her (the HUD, H2).
- **Drawing the pad and the cover (C1, AF3):** their figures at their positions, relative to the
  camera; which sprites each figure is (content).
- **The shaft's overlay itself (C1, AF3):** placing it with the shaft's other enemies, from the
  shaft's content (no other room has it); running it (§7); drawing its one figure, which never
  changes, every frame at its place; it has no contact: nothing she does touches it. Which
  sprites its figure is, and bringing them in with the shaft, are content.
- **The beacon's colours and the platform's two looks (content):** the eight sets of six colours
  and the two pictures are the shaft's content; this domain names which one shows. Showing a
  colour set on the screen is the renderer's; putting the platform's look on the screen is
  the line and plane effects' (E4).
- **The count of finished frames (the coordinator of time, T1):** counting the frames the game
  finishes, and which frames finish (lag). It is not the HUD's frame count (interface-hud), which
  counts mod 256 and starts again at each door (world-doors' `FinishedFrames`).

## 6. Open questions

1. Her touch on a blocking door remains original-only. The recorded ordinary power-shot contact
   is covered by [enemy-interactions](enemy-interactions.md);
   other beam families remain outside that first piece.
2. `Getaway` with a door placed: no row places one in that stage (Ridley's getaway places its
   walls the frame before the getaway starts, still `Calm`). The arms that read it follow the
   original.
3. Part 2: the pad's placing is recorded once (the arrival); her height there is 0. Heights below
   the floor at placing, and a pad placed in a room other than the shaft, are no row's: the arms
   that read them follow the original's arithmetic.
4. Part 2: the shaft's rumble, a later piece, owns the beacon and the platform in the escape;
   this piece shows no step and no look in the escape (`BeaconCycles`, `PlatformTurns`) and
   states them for the stages `Calm` and `Getaway` only. Answered for that piece from the records:
   when the shaft starts in the escape (she comes back to it), the overlay's first run steps the
   beacon and the platform once, by the counts of `BeaconCycles` and `PlatformTurns`, and from its next run
   the rumble takes over from them.
5. Part 3: no row enters Ridley's room once Ridley is beaten, or in the getaway or the escape: the
   arms for the door placed then (`RidleyRoomStart`, `TheDoorsColours`) and for Ridley placed
   once beaten (`RidleyPlaced`) follow the original.
6. Part 3: where Ridley's drawing puts the door it draws, and how the screen's shake moves it, are
   his drawing's, a later piece with his fight; nothing here compares a picture.

---

## 7. Part 2, the shaft as she arrives: what the player can tell

A new game starts with her at the top of the shaft, on the elevator's pad, facing the screen.
For a second nothing moves but the shaft's lights. Then the pad lowers her one pixel a frame
to the shaft's floor, 72 px down, and lets her go: from the next frame she plays. Below the pad,
a cover hides the shaft's floor until she stands on it. Before the escape the beacon's lights
pulse through a cycle of colours and the platform below turns.

**Notation.** Part 2 writes its positions, heights and counts in decimal; y is downward, so
the floor at 72 px is below her start at 0.

**Running.** Part 2 is two pieces, and the caller steps each on its own schedule (§7.3).
- **The pad and the cover** (§8) hold state. They run once on each frame of play, and on each
  frame of the room's fade-in, the pad first; running them is the enemy framework's projectile
  runner (C1, AF3), as running the doors is its enemy runner (Header). The caller steps them on
  the frame the arrival places them and on each frame they run. On a frame they don't run it
  doesn't step them, and nothing of them changes. The fade-in starts on the frame after the
  arrival and runs them on each of its frames, so their first run is always the next frame's.
- **The beacon and the platform** (§9) remember nothing. They are the shaft's overlay's: one of
  the shaft's enemies, neither a door nor a wall, which the shaft's content alone places. The
  framework runs it on each frame the enemies run (Header, Running); during a door's passage
  those are only the passage's frames of work that run the enemies (world-doors, `WhatRuns`),
  every third frame in the recorded passages. The caller steps them on each frame the overlay
  runs. On a frame it doesn't run, and in every other room, the caller doesn't step them: no
  beacon step and no platform look, and the screen keeps what it shows.

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the pad | present or gone | whether she is held and the pad is drawn | every entry of §8 |
| the pad's wait | frames, 0 to the wait `w` (`PadWaits`): the runs before it starts lowering her; 0 once it has | when she starts down | `PadWaits` |
| the pad's position | x, y: px, 16-bit | where the pad is drawn | output (drawing) |
| the pad's look | `PadLook`: `First` or `Second`, or none before its first run | which of its two figures is drawn | `PadFlickers`; output (drawing) |
| the cover | present or gone | whether the cover is drawn | `CoverHides` |
| the cover's position | x, y: px, 16-bit | where the cover is drawn | output (drawing) |
| the beacon's step | 0 to 7: the place in its cycle of eight colour sets, or none | the beacon's colours | output (the screen's colours) |
| the platform's look | `PlatformLook`: `First` or `Second`, or none (`PlatformTurns`) | which of its two pictures shows | output (the planes, E4) |

### 7.2 What it remembers: state a caller can hold

The pad (present or gone, its wait, its position and its look) and the cover (present or gone,
its position): nothing else. A caller can build either in any state this spec allows and read it
back; a vector's `before` builds them and its `after` reads them. Whether she is held follows
from the pad. The beacon's step and the platform's look have no memory: each follows from the
count of finished frames on the frame it is shown.

### 7.3 What it reads (Inputs)

**The pad and the cover.** The caller steps them on the frame the arrival places them and on each
frame they run (§7, Running); a step reads one of these two.

| Input | Type | Read by |
| --- | --- | --- |
| the arrival places the pad | an event, with her position then: x, y, px, 16-bit (the arrival, §5) | `PadPlaced` |
| her height | y: px, 16-bit: on a frame they run, her position's y after her own frame (samus-movement) | `PadCarries`, `CoverHides` |

**The beacon and the platform.** The caller steps them on each frame the shaft's overlay runs (§7,
Running); a step reads all three.

| Input | Type | Read by |
| --- | --- | --- |
| the count of finished frames | frames, 16-bit, wrapping: the frames the game has finished (a lag frame isn't one) as the last frame left it; it never starts again, at a door or anywhere, unlike the HUD's frame count (T1, §5) | `BeaconCycles`, `PlatformTurns` |
| the fade's steps taken | steps, 0 to 14: how many steps of a door's fade (world-doors' `Fading`, `t` 0 to `d + 2`) the passage has taken when the overlay runs. 0 outside a door's fade, including the screen's brightness coming up at the arrival (§5), and on a passage's frames before its fade; in `Darkening(t)` the passage takes its step before the enemies run, so `t + 1`; in `Brightening(t)` after them, so `t`, 0 on the fade in's first frame of work | `BeaconCycles` |
| the station's stage | `Stage` (§1.3) | `BeaconCycles`, `PlatformTurns` |

### 7.4 What it reports (Outputs)

A step of the pad and the cover reports the first three; a step of the beacon and the platform,
the last two.

| Output | Type | Read by |
| --- | --- | --- |
| she is held | `bool`: the pad is present | samus-movement and samus-appearance (§5); output |
| her height | y: px, 16-bit, as the pad leaves it | samus-movement, as her position; the view and the HUD that frame |
| the pad's and the cover's figures and positions | the pad's look, or the cover, at its position; none while gone, and the cover none on the frame it is placed, until its first run (`PadPlaced`) | output: drawing (C1) |
| the beacon's step | 0 to 7, or none (`BeaconCycles`) | output: the screen's colours take the step's colour set (the renderer); none leaves them as they are |
| the platform's look | `PlatformLook`, or none (`PlatformTurns`) | output: the planes (E4); none leaves the picture as it is |

## 8. The pad and the cover

### PadPlaced
- kind: mechanic.
- summary: the arrival puts the pad under her and the cover below it, and from then on she is
  held.
- params: the wait `w` (`PadWaits`); the pad 28 px below her; the cover 97 px down the room.
- switch: Fixed.
- The arrival places the pad, with her at `(x, y)` → the pad present, its wait `w`, its position
  `(x, (y + 28) mod 2^16)`, no look; the cover present at `(x, 97)`, with no figure until its
  first run; she is held [E-placed, E-placed-low, E-placed-adjusted, E-placed-drawn]
- So neither runs on the frame it is placed, and that frame draws neither. Their first run is the
  next frame's (§7, Running): it draws the pad's first look and the cover at its position
  (E-first-run, E-first-drawn).
- checked: oracle (the arrival, recorded case A1); constructed.

### PadWaits
- kind: parameter.
- summary: the pad holds her still for a second before it starts to lower her.
- params: the wait, `w` = 60 frames; a rule from 0 to 32,768 frames.
- switch: **Parameter**, the wait (params).
- A run with the pad's wait over 1 → the wait drops by one; her height and the pad's position
  stay as they are [E-first-run, E-first-drawn, E-wait-counts, E-wait-two]
- A run with the pad's wait 1 or 0 → the wait is 0, and the pad lowers her in this run
  (`PadCarries`) [E-wait-last, E-carry]
- So her first pixel down is on the pad's `w`-th run, `w` frames after its placing
  (E-placed-adjusted, E-wait-last).
- checked: oracle (recorded cases A2 to A5); constructed (the adjusted wait).

### PadCarries
- kind: mechanic and parameter.
- summary: each run once its wait is over, the pad lowers her one pixel and is drawn 28 px below
  where she stood, until she stands on the shaft's floor; then it lets her go.
- params: the floor, `f` = 72 px.
- switch: **Parameter**, the floor (params).
- A run that lowers her, with her height `y` and `(y − f) mod 2^16` 32,768 or more → the pad's
  position becomes `(x, (y + 28) mod 2^16)`, her height `y + 1` [E-wait-last, E-carry,
  E-carry-to-floor, E-wrap-above, E-floor-adjusted-above]
- A run that lowers her, with `(y − f) mod 2^16` under 32,768 → her height becomes `f`, the pad is
  gone and she is no longer held [E-floor, E-below-floor, E-wrap-below, E-floor-adjusted]
- So she reaches the floor on one run and is let go on the next: from 71 px a run sets her at
  72, and the run after leaves her there and ends the pad (E-carry-to-floor, E-floor).
- So a `y` past the floor's (lower on the screen) by less than 32,768 puts her on the floor at
  once, and one past it by 32,768 or more counts as above it and is lowered as any other
  (E-below-floor, E-wrap-below, E-wrap-above).
- The pad's x never changes.
- checked: oracle (recorded cases A5 to A8, every run of the pad in the cached rows);
  constructed (the wrap, the heights past the floor, the adjusted floor).

### PadFlickers
- kind: mechanic.
- summary: the pad's two figures take turns, one each run.
- params: none.
- switch: Fixed.
- A run that leaves the pad present, its look `First` → its look `Second` [E-wait-counts,
  E-wait-last, E-floor-adjusted-above]
- A run that leaves the pad present, its look `Second` or none → its look `First`
  [E-first-run, E-first-drawn, E-wait-two, E-carry, E-carry-to-floor, E-wrap-above]
- checked: oracle (every run of the pad in the cached rows).

### CoverHides
- kind: mechanic.
- summary: the cover hides the shaft's floor below the pad until she stands on the floor.
- params: the floor `f` (`PadCarries`).
- switch: Fixed.
- A run of the cover with her height `f` → the cover is gone [E-carry-to-floor, E-cover-alone,
  E-cover-adjusted, E-below-floor, E-floor-adjusted-above]
- A run of the cover with any other height → the cover stays where it is [E-carry,
  E-wait-last, E-cover-stays]
- So the cover goes on the frame she reaches the floor, one frame before the pad (§4).
- checked: oracle (every run of the cover in the cached rows); constructed.

## 9. The beacon and the platform

### BeaconCycles
- kind: mechanic and parameter.
- summary: before the escape, the shaft's beacon steps through its cycle of eight colour sets,
  one step every few frames, except once a door's fade has taken a step.
- params: the rhythm, `r` = 8 frames a step; a rule from 1 to 65,535 frames.
- switch: **Parameter**, the rhythm (params).
- A step with the stage `Escape` → no step: a later piece of the station owns the shaft in the
  escape (open question 4) [B-escape]
- A step with the stage `Calm` or `Getaway`, the fade's steps taken 0 → the beacon shows
  step `(count / r) mod 8` of its cycle, with `count` the count of finished frames and `/`
  dropping the remainder [B-step, B-step-next, B-step-wrap, B-getaway, B-rhythm-adjusted,
  B-arrival, B-count-not-hud]
- The same with the fade's steps taken 1 or more → no step: the beacon's colours are left as
  the fade makes them [B-fade, B-fade-last]
- So the beacon moves on every `r` frames; where its content repeats a colour set, the screen
  shows no change at that step.
- checked: oracle (every frame of the shaft before the escape in the cached rows); constructed
  (the wrap, the fade, the adjusted rhythm, the escape).

### PlatformTurns
- kind: mechanic.
- summary: before the escape, the platform below the shaft shows its two looks in turn, two
  frames each.
- params: none.
- switch: Fixed.
- A step with the stage `Escape` → no look: a later piece of the station owns the shaft in the
  escape (open question 4) [B-escape]
- A step with the stage `Calm` or `Getaway`, `(count / 2) mod 2` 0 → the platform's look
  `First` [P-first, B-fade]
- The same with `(count / 2) mod 2` 1 → the platform's look `Second` [P-second, B-getaway,
  B-fade-last]
- So the platform turns while a door's fade is under way too (B-fade, B-fade-last).
- checked: oracle (every frame of the shaft before the escape in the cached rows); constructed
  (the escape).

---

## 10. Part 3, Ridley's room as she enters: what the player can tell

She comes into Ridley's room by its door. Behind her the door stands open a moment, then
slams shut in four quick figures and locks: no longer drawn as a door of its own, but drawn by
Ridley's drawing, and still in her way. The room is dark. Ridley is at his place, but his
colours are black, so only the baby he holds can be seen; every so often it cries and glows.
Ridley waits a fixed while from the end of the door's passage, then wakes (a later piece). Once
he is beaten, as the escape starts, the door is an ordinary door again and lets her out. During
his getaway two walls stand at the room's sides.

**Running.** The door and the walls are Ceres doors (Header, Running), and in his room they
run after Ridley, the room's first enemy (§4). The framework runs Ridley on each frame the
enemies run, the passage's frames of work included; the caller steps him (§12) on the frame the
room places him and on each frame he runs. On a frame he doesn't run, nothing of him or the baby
changes.

Part 3's door and walls are the doors' kinds `RidleyDoor` and `GetawayWall`, with the phases
`Lingering`, `Slamming(n)`, `Slammed`, `Locked`, `Standing` and `Unseen` (§1); they read whether
Ridley is beaten (§1.3) and report whether Ridley draws the door (§1.4). Part 3 writes its
positions in hex, as part 1 does.

## 11. The door of Ridley's room and the getaway's walls

### RidleyRoomStart
- kind: mechanic.
- summary: the door of Ridley's room, placed as the room starts, stands open behind her a moment.
- params: the pause, 2 frames.
- switch: Fixed.
- `Placed` steps, a `RidleyDoor` → `Lingering`, wait 2, whatever the stage, whether Ridley is
  beaten and wherever she is [R-first-run, R-first-run-escape]
- So the door she has come through is hidden and lets her by for its first two frames.
- checked: oracle (recorded case N2, the room start in the cached rows); constructed.

### SlamShut
- kind: mechanic and parameter.
- summary: then it slams shut over four figures, quickly and silently, near her or not.
- params: the slam's hold, `s` = 2 frames for each figure.
- switch: **Parameter**, the slam's hold (params).
- `Lingering` steps → `Slamming(1)`, its wait `s`, no sound [R-slam-start, R-slam-adjusted]
- `Slamming(n)` steps, `n` under 4 → `Slamming(n + 1)`, its wait `s`, near her or not
  [R-slam-step, R-slam-near, R-slam-adjusted-step]
- `Slamming(4)` steps → `Slammed`, wait 1 [R-slam-last, R-slam-adjusted-last]
- So the door is shut `4 × s` frames after its slam starts, 8, solid from the slam's first
  figure.
- checked: oracle (recorded cases N4 to N6, every run of the slam in the cached rows);
  constructed (the adjusted hold).

### LockedIn
- kind: mechanic.
- summary: the shut door stays locked while Ridley lives, looking every 2 frames whether he is
  beaten; then it is a closed door like any other, which opens as she comes near.
- params: the lock's look, every 2 frames; the poll `p` (`Polling`).
- switch: Fixed.
- `Slammed` steps → `Locked`, wait 2, whether Ridley is beaten or not [R-locked, R-slammed-beaten]
- `Locked` steps, Ridley not beaten → `Locked`, wait 2, wherever she is [R-locked-again,
  R-locked-near]
- `Locked` steps, Ridley beaten → `Closed`, its wait the poll [R-released, R-released-poll]
- From then on it steps by part 1's arms as a `Door` that faces right: it opens as she comes near,
  with the opening's sound [R-released-opens]
- checked: oracle (recorded cases N7 to N11, every run of the lock in the cached rows);
  constructed.

### DrawnWithRidley
- kind: mechanic.
- summary: while the door of Ridley's room is shut, Ridley's drawing draws it.
- params: none.
- switch: Fixed.
- Phase `Slammed` or `Locked` → drawn with Ridley [R-slam-last, R-locked]
- Any other phase → Ridley doesn't draw it [R-slam-start, R-released]
- So on the frame it is shut both the framework and Ridley draw it; from the next, he alone.
- checked: oracle (every run in the cached rows).

### GetawayWalls
- kind: mechanic.
- summary: during Ridley's getaway two walls stand at his room's sides; they look for nobody.
- params: none.
- switch: Fixed.
- A `GetawayWall` steps on every run, as its wait is always 1; the stage `Getaway` → `Standing`,
  wait 1 [W-first-run, W-right, W-standing, W-near]
- The same, the stage `Calm` or `Escape` → `Unseen`, wait 1 [W-calm, W-hides]
- So the walls show from their first run in the getaway until the escape starts, and are never
  in her way.
- checked: oracle (recorded cases N12 to N15, every run of the walls in the cached rows);
  constructed.

## 12. Ceres Ridley as she enters

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| his phase | `RidleyPhase`, listed alphabetically: `Awake` (his waiting is over: a later piece runs him), `Holding` (placed, the door's passage still under way), `Waiting` | when he wakes | `RidleyWaits` |
| his wait | frames, 0 to `w` − 1, while `Waiting` | when he wakes | `RidleyWaits` |
| his look | `RidleyLook`: `Rest`, facing left; none before his first run | which figure is drawn | output (drawing) |
| his position | x, y: px, 16-bit | where he is drawn and the baby held | `BabyHeld` |
| the baby's routine | `BabyRoutine`: `New` (before its first run), `Fidget(i)`, `i` 0 to 7, or `Glow(j)`, `j` 0 to 11, with its runs: how many runs it has shown its figure, 1 to the figure's length | its look, when it cries | `BabyFidgets`, `BabyCries` |
| the baby's look | `BabyLook`: `First`, `Second` or `Third`; none while `New` | which of its figures is drawn | output (drawing) |
| the baby's position | x, y: px, 16-bit; none before its first run | where it is drawn | output (drawing) |

He is shown and passable: the framework draws his look at his position, in colours that stay
black until he wakes; she, her touch and her shots go through him, and he doesn't hurt her.

### 12.2 What he remembers: state a caller can hold

His phase, wait, look and position, and the baby's routine with its runs and its position:
nothing else. The baby's look follows from its routine. A caller can build him in any state
this spec allows and read it back.

### 12.3 What he reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| Ridley beaten | `bool` (§1.3), as the room places him | `RidleyPlaced` |
| a passage under way | `bool`: a door's passage is under way as he runs (world-doors' `passing`) | `RidleyWaits` |
| the random number | 16-bit, as the doors read it (§1.3) | `BabyCries` |

### 12.4 What he reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| a boss encountered | Ridley in Ceres, as he is placed | interface-hud (`BossBlanksTheMap`) |
| the music's stop | a request, as he is placed | audio (U1) |
| the shake's stop | the screen's shake ends, as he is placed | the screen's shaking (R3) |
| the stage | `Calm`, as he is placed | progress (P1) |
| his room's colours | as he is placed, these goal colours: the baby's palette and the `Own` palette of his room's door take what his room's content holds for them; the palette he is drawn in, the doors', turns black, all but its first colour | the passage's fade (world-doors) |
| the baby's colours | none, or one of the baby's colour sets, listed alphabetically: `Glow1`, `Glow2`, `Glow3`, `Rest`; the screen shows them in the baby's palette at once; the sets are content | the screen's colours (the renderer) |
| the baby's cry | a sound, with a queue limit of 6, or none | audio (U1) |
| his drawing | his look at his position, the baby's look at its position, and the door of his room while it is drawn with Ridley (`DrawnWithRidley`) | drawing (C1) |

### RidleyPlaced
- kind: mechanic.
- summary: as she enters his room he is placed at his place, in the dark, the baby in his hold;
  the mini-map goes blank, the music stops and the station stops shaking.
- params: his place, x 186, y 169 (px, decimal).
- switch: Fixed.
- The room places him, Ridley not beaten → `Holding`, no look, at his place whatever the room's
  content says; the baby `New`; shown and passable; a boss encountered, the music's stop, the
  shake's stop, the stage `Calm` and the room's colours [K-placed]
- The room places him, Ridley beaten → he isn't placed, and none of those happen
  [K-placed-beaten]
- So he is placed before the door of his room, whose colours follow his (`TheDoorsColours`).
- checked: oracle (recorded case K1); constructed.

### RidleyWaits
- kind: mechanic and parameter.
- summary: once the door's passage into his room is over he waits a fixed while in the dark, then
  wakes.
- params: the wait, `w` = 512 frames; a rule from 1 to 32,768.
- switch: **Parameter**, the wait (params).
- A run, `Holding`, a passage under way → `Holding` [K-first-run, K-held-elsewhere]
- A run, `Holding`, no passage under way → `Waiting`, wait `w` − 1 [K-first-wait,
  K-wait-adjusted, K-wait-adjusted-wake]
- A run, `Waiting`, wait over 0 → wait one less [K-wait-counts, K-fidget-next]
- A run, `Waiting`, wait 0 → `Awake`: from this run on a later piece runs him [K-wake]
- Every run → his look `Rest` [K-first-run]
- So he wakes on the `w + 1`-th frame he runs after the passage, 512 frames after the first.
- checked: oracle (recorded cases K2 to K5, every run of him in the cached rows until he
  wakes); constructed (the adjusted wait).

### BabyHeld
- kind: mechanic.
- summary: the baby stays in his hold.
- params: the hold, 16 px left of him and 22 px down.
- switch: Fixed.
- Every run → the baby at `((x − 16) mod 2^16, (y + 22) mod 2^16)`, with `(x, y)` his position
  [K-first-run, K-held-elsewhere]
- checked: oracle (every run of him in the cached rows); constructed.

### BabyFidgets
- kind: mechanic.
- summary: the baby fidgets through its figures, ten runs each.
- params: eight figures of 10 runs each, `Fidget(0)` to `Fidget(7)`, their looks `First`,
  `Second`, `Third`, `Second`, twice over.
- switch: Fixed.
- The baby's first run, `New` → `Fidget(0)`, 1 run [K-first-run]
- A run, its figure shown fewer runs than its length → one run more [K-first-wait,
  K-wait-counts, K-glow-counts, K-held-elsewhere]
- A run, `Fidget(i)` shown its 10 runs, `i` under 7 → `Fidget(i + 1)`, 1 run [K-fidget-next]
- A run, `Fidget(7)` shown its 10 runs → as `BabyCries` says [K-calm, K-cries]
- checked: oracle (every run of the baby in the cached rows).

### BabyCries
- kind: mechanic.
- summary: at the end of each fidget the baby, one time in two, cries and glows.
- params: the glow, twelve figures `Glow(0)` to `Glow(11)`, lasting 6, 5, 4, 3, 2, 3, 4, 5, 6, 7,
  8 and 9 runs, their looks `First`, `Second`, `Third`, `Second` three times over and their
  colour sets `Glow1`, `Glow2`, `Glow3`, `Glow2` three times over.
- switch: Fixed.
- `Fidget(7)` shown its 10 runs, the random number odd → `Fidget(0)`, 1 run, no cry [K-calm]
- `Fidget(7)` shown its 10 runs, the random number even → `Glow(0)`, 1 run; the baby's cry and
  its colours `Glow1` [K-cries]
- `Glow(j)` shown its length, `j` under 11 → `Glow(j + 1)`, 1 run, and its colour set
  [K-glow-next]
- `Glow(11)` shown its 9 runs → `Fidget(0)`, 1 run, its colours `Rest` [K-glow-last]
- So every 80 runs, one time in two, it cries and glows for 62 runs.
- checked: oracle (recorded cases K6 to K9, every chance in the cached rows); constructed.
