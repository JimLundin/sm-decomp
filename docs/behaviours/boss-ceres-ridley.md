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

## 13. Part 4a, Ceres Ridley wakes

His waiting over (`RidleyWaits`), Ridley's eye lights up out of the dark: over about a second
it brightens to its full colour and holds. Then his body fades in from black, a little every
other frame, for about half a second. As his body reaches its full colours he turns tangible and
the fight's music starts. A moment later he opens his jaws and roars, his jaws wide for over a
second and a half, then shuts them and sits a while before he takes off. The baby goes on as
part 3 says, held, fidgeting and now and then crying.

**What this part reads and leaves.** It reads nothing beyond part 3's inputs (§12.3), the random
number for the baby, and the content: his eye's steps and colour sets and his body's colour sets
(§13.1). It hands the screen his colours, the enemy framework (C1) that he is tangible, audio
(U1) the fight's music and the roar, and his next part the moment he takes off. Left to others:
his tail and wings, which move from his body's fade on, and the hit his tail gives her (part 4b,
with health's `TailHit`, H1); her knockback (samus-movement) and her hurt colours
(samus-appearance); her shots against him and his flash when they hit (C1 and a later part);
his flight (part 4c); drawing his figures (C1, as in §12).

**Running.** As in §10: the caller steps him on each frame he runs; on a frame he doesn't run,
nothing of him or the baby changes. His position doesn't change in this part. Runs below are
the frames he runs.

### 13.1 What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| his phase | `RidleyPhase`, part 3's and, listed alphabetically: `BodyFading`, `EyeFading`, `ReadyToRoar`, `Roaring`, `TakingOff` (his roar over: part 4c runs him). Part 3's `Awake` is his eye's fade with no step shown | which colours change, when he is tangible, when he roars | every entry of this part |
| the eye's steps shown | count, 1 to the eye's steps' count, while `EyeFading` | how bright his eye is | `EyeFadesIn` |
| the body's sets shown | count, 0 to the body's sets' count less 1, while `BodyFading` | how far his body has faded in | `BodyFadesIn` |
| the body's runs | runs since his body last took a set, or since its fade began, 0 to `p` − 1, while `BodyFading` | when it takes its next set | `BodyFadesIn` |
| his wait | runs, 0 to 4 while `ReadyToRoar`, 0 to 252 while `Roaring` | when he roars, when he takes off | `ReadyToRoar`, `Roars` |
| the roar's figure | `RoarFigure`, 1 to 5 (`Roars`), with its runs: how many runs it has been shown, 1 to its length; the fifth has no length and no runs | his look, whether he is roaring | `Roars` |
| his look | `RidleyLook`: part 3's `Rest`, and `Open` (his jaws open) and `Wide` (his jaws wide, roaring), all facing left | which figure is drawn | output (drawing) |
| the eye's steps | content: a list of at least one step, each naming one of the eye's colour sets; the game's has 64 | how his eye brightens | `EyeFadesIn` |
| the eye's colour sets | content: sets of three colours | his eye's colours | output |
| the body's colour sets | content: a list of at least one set of eleven colours, in order; the game's has 16 | how his body fades in | `BodyFadesIn` |

### 13.2 What he remembers: state a caller can hold

His phase with its count, runs, wait, or roar's figure and that figure's runs; his look and
position; and the baby as part 3 says (§12.2): nothing else. Whether he is tangible, moving or
roaring follows from his phase (§13.4). A caller can build him in any state this part allows and
read it back.

### 13.3 What he reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| the random number | 16-bit, as part 3 (§12.3) | the baby (`BabyCries`) |
| his content | the eye's steps, the eye's colour sets and the body's colour sets (§13.1), imported | `EyeFadesIn`, `BodyFadesIn` |

### 13.4 What he reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| the eye's colours | none, or one of the eye's colour sets: the screen shows it at once on colours 12 to 14 of the palette he is drawn in (the doors', §12.4) | the screen's colours (the renderer) |
| the body's colours | none, or one of the body's colour sets: the screen shows it at once on colours 1 to 11 of the palette he is drawn in and of his second palette, which none of his figures in this part uses | the screen's colours (the renderer) |
| tangible | `bool`: phase `ReadyToRoar`, `Roaring` or `TakingOff`; before, she, her touch and her shots go through him (§12) | the enemy framework (C1): her touch and her shots; his tail's hit (part 4b) |
| moving | `bool`: any phase from `BodyFading` on: his tail and wings move, and the baby is drawn with them | his tail and wings (part 4b); drawing (C1) |
| roaring | `bool`: phase `Roaring` with the roar's figure 2, 3 or 4 | his tail's whip sound and his fireballs (parts 4b and 4c) |
| the fight's music | a request, on the run he turns tangible | audio (U1) |
| the roar | a sound, with a queue limit of 6, on the run the roar's second figure starts | audio (U1) |
| taking off | phase `TakingOff`: from that run on part 4c runs him | part 4c |
| his drawing | his look at his position, and the baby as in §12.4 | drawing (C1) |

### EyeFadesIn
- kind: mechanic.
- summary: once awake his eye lights up out of the dark, one step of its fade each run.
- params: the eye's steps and colour sets (content, §13.1).
- switch: Fixed.
- A run, `Awake` → `EyeFading`, 1 step shown; the eye's colours: the set the first step names
  [F-eye-first, F-eye-one-step]
- A run, `EyeFading`, `n` steps shown, `n` under the steps' count → `n + 1` steps shown; the
  eye's colours: the set step `n + 1` names, the same set as the last run's or not [F-eye-step,
  F-eye-same-set, F-eye-last]
- A run, `EyeFading`, every step shown → `BodyFading`, 0 sets shown, 0 runs; no colours; moving
  from this run on [F-eye-done, F-eye-one-step-done]
- Every run before `Roaring` → his look `Rest` [F-eye-first, F-body-last, F-ready-counts]
- So with the game's 64 steps his body's fade begins on the 65th run after he wakes.
- checked: oracle (recorded cases F1 to F4, every run of the fade in the cached rows);
  constructed.

### BodyFadesIn
- kind: mechanic and parameter.
- summary: then his body fades in from black, one colour set every `p` runs; with its last set
  he turns tangible and the fight's music starts.
- params: the pace, `p` = 2 runs; a rule from 1 to 255. The body's colour sets (content).
- switch: **Parameter**, the pace (params).
- A run, `BodyFading`, its runs + 1 under `p` → its runs one more; no colours [F-body-wait,
  F-body-adjusted-wait]
- A run, `BodyFading`, its runs + 1 = `p`, `s` sets shown, `s + 1` under the sets' count →
  `s + 1` sets shown, 0 runs; the body's colours: set `s + 1` [F-body-first, F-body-set,
  F-body-adjusted-set, F-body-pace-one]
- A run, `BodyFading`, its runs + 1 = `p`, `s + 1` the sets' count → `ReadyToRoar`, wait 4; the
  body's colours: the last set; tangible from this run on; the fight's music [F-body-last,
  F-body-adjusted-last]
- So the game's 16 sets at the pace 2 take 32 runs: the first shows on the body's 2nd run, the
  last, with the music, on its 32nd.
- checked: oracle (recorded cases F5 to F8, every run of the fade in the cached rows);
  constructed (the adjusted pace).

### ReadyToRoar
- kind: mechanic.
- summary: tangible, he holds still a moment before he roars.
- params: the wait, 4 runs (set by `BodyFadesIn`); the roar's wait, 252 runs.
- switch: Fixed.
- A run, `ReadyToRoar`, wait over 0 → wait one less [F-ready-counts, F-ready-last]
- A run, `ReadyToRoar`, wait 0 → `Roaring`, wait 252, the roar's figure 1, its first run
  [F-ready-roars]
- So he starts his roar on the 5th run after he turns tangible.
- checked: oracle (recorded cases F9 to F11); constructed.

### Roars
- kind: mechanic.
- summary: he opens his jaws and roars, shuts them, and sits until his roar's wait runs out;
  then he takes off.
- params: the roar's figures, in order, with their lengths in runs: 1 `Rest` 6, 2 `Open` 8,
  3 `Wide` 96, 4 `Open` 8, 5 `Rest`, held for the rest of the roar.
- switch: Fixed.
- A run, `Roaring`, wait over 0 → wait one less, and its figure moves on as the next arms say
  [F-roar-counts, F-roar-last]
- The same, its figure shown fewer runs than its length → one run more [F-roar-counts,
  F-roar-wide-counts]
- The same, figure 1 shown its 6 runs → figure 2, its first run; the roar [F-roar-opens]
- The same, figure 2 or 3 shown its length → the next figure, its first run [F-roar-wide,
  F-roar-shuts]
- The same, figure 4 shown its 8 runs → figure 5 [F-roar-rests]
- The same, figure 5 → figure 5 [F-roar-held]
- A run, `Roaring`, wait 0 → `TakingOff`, whatever the figure; his figure and all he does on this
  run and after are part 4c's [F-roar-flies, F-roar-flies-early]
- Every run, `Roaring` → his look, the figure's [F-roar-counts, F-roar-opens, F-roar-wide,
  F-roar-shuts, F-roar-rests, F-roar-held]
- So the roar is heard on the roar's 7th run, his jaws are wide from its 15th run for 96 runs,
  he stops roaring on its 119th and takes off on its 254th.
- checked: oracle (recorded cases F12 to F21, every run of the roar in the cached rows);
  constructed.

**The baby goes on.** On every run of this part the baby runs as part 3's `BabyHeld`,
`BabyFidgets` and `BabyCries` say, and its colours and cry are reported as in §12.4; every F-
vector carries it [F-baby-cries].

## 14. Part 4b, his tail at rest and its hit

From the moment he starts moving (§13.4) until he takes off, Ridley's tail hangs still behind
him, its tip a little right of him and below. When he is tangible and her protection has run
out, she is hit as soon as her body comes within reach of the tip: she loses energy and is pushed
back, away from the tip. In the movie she stands at the tip as he turns tangible and again each
time her protection runs out, so she is hit three times before he takes off.

**What this part reads and leaves.** It reads his phase and position (§13), her position, her body
and her protection as the frame starts (samus-movement, samus-health) and the random number
(§12.3). It hands the frame's hit to her energy (samus-health) and to her motion (samus-movement),
and the tail's place to drawing. Ridley reports the hit's worth. Samus-health applies it with
the reduction for her suit and gives her protection. Samus-movement supplies the push, its
leap and its poses. Samus-appearance supplies her hurt colours. Part 4c-1b (§17) supplies the
moving tip for `TailHits` and the whip's sound. The enemy framework (C1) draws the wings and
tail segments. His fake retreat, after her 100th hit on him, ends the tail's hits.

**Running.** As in §13: on each run that ends in `BodyFading`, `ReadyToRoar` or `Roaring`, this
part's entries run after §13's, so the run he turns tangible can hit. A run that ends before
he moves leaves the tail as it was, and the run he takes off is part 4c's, tail included.

### 14.1 What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the tail's place | x, y: px, 16-bit: where the tip and all seven of its segments are while it rests; none before he moves | where the tail is drawn; where it reaches her | `TailHits`; output (drawing) |
| the tail's curl | count, 8 to 23; 16 when he is placed | how tightly the tail curls once it moves (part 4c) | `TailCurls`; part 4c |
| the tail's reach | px, `t` | how close she can come to the tip unhurt | `TailHits` |
| the frame's hit | none, or a hit with its side (`Side`: `Left` or `Right`, the way it pushes her) and its push time (`PushFrames`, frames: 5) | she loses energy and is pushed back | output |

### 14.2 What he remembers: state a caller can hold

§13.2's, and the tail's curl: nothing else. The tail's place follows from his position while it
rests. A caller can build him in any state this part allows and read it back.

### 14.3 What he reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| her position | x, y: px, 16-bit, as the frame starts: where the last frame left her, before her moves this frame (samus-movement) | `TailHits` |
| her body | her half-width and half-height, px, of the body she started the frame in, read from her movement's content (its body size for the pose she began the frame in); not an output of her movement | `TailHits` |
| her protection | frames, as the frame starts (samus-health's remaining protection) | `TailHits` |
| the random number | 16-bit, the one the baby reads on the same run (§12.3) | `TailCurls` |

### 14.4 What he reports (Outputs)

The hit is this part's alone: neither reader judges the reach, the side or the push time again.
The frame runs in this order: his run reads her position, her body and her protection as the
frame starts, and reports the frame's hit; Health applies the hit; then her moves, and her
movement's reaction in the same frame starts the push.

| Output | Type | Read by |
| --- | --- | --- |
| the frame's hit | none, or a hit with its side and push time (§14.1), made on his run, before her moves that frame | samus-health, in the same frame (the frame's `Impact`, worth 15 whole energy); samus-movement, in the same frame, in its reaction step after her moves (its §12 input, the frame's hit: `PushTime` and the push that follows) |
| the tail's place | as in §14.1 | drawing (C1) |

### TailAtRest
- kind: mechanic.
- summary: from the moment he starts moving until he takes off, his tail hangs still: its tip and
  all its segments at one place behind him.
- params: the place, 32 px right of his position and 16 px down (he faces left).
- switch: Fixed.
- A run that ends in `BodyFading`, `ReadyToRoar` or `Roaring` → the tail's place
  `((x + 32) mod 2^16, (y + 16) mod 2^16)`, with `(x, y)` his position [T-rest-first,
  T-rest-intangible, T-hit-left, T-wrap, T-wrap-across, T-wrap-y]
- A run that ends before he moves reports no tail's place [T-curl-still].
- So the tail's place doesn't move in this part: his position doesn't.
- checked: oracle (recorded cases T2 to T7, every run 577-866 of the cached rows); constructed
  (the wrap).

### TailCurls
- kind: mechanic.
- summary: now and then, at random, the tail's curl changes; it shows only once the tail moves
  (part 4c).
- params: the change, at a random number of 65,280 or more, to 8 + (the random number mod 16).
- switch: Fixed.
- He is placed → curl 16 [T-placed]
- A run that ends in `BodyFading`, `ReadyToRoar`, `Roaring` or runs the tail in flight (§17),
  the random number `n` 65,280 or more → curl `8 + (n mod 16)` [T-curl-least, T-curl-most,
  T-curl-recorded, FT-curl-flight]
- Otherwise the curl stays: a random number under 65,280, or a run that ends before he moves
  [T-curl-below, T-rest-first, T-curl-still].
- checked: oracle (recorded cases T1 and T8; every run of the cached rows from his placing to
  the escape); constructed.

### TailHits
- kind: mechanic and parameter.
- summary: tangible, he hits her when her body comes within reach of his tail's tip and her
  protection has run out; she is pushed away from the tip.
- params: the tail's reach, `t` = 14 px (rule `tail_reach`); a rule from 0 to 255. The push
  time, 5 frames. The hit is worth 15 whole energy, his damage in Ceres. Samus-health takes it off (`TailHit`).
- switch: **Parameter**, the tail's reach (params).
- Here `dx` and `dy` are her x less the tip's x and her y less the tip's y, each taken mod 2^16
  as a signed number from −32,768 to 32,767, her position as the frame starts, and `w` and `h`
  the half-width and half-height of the body she started the frame in.
- A run, tangible, her protection 0 as the frame starts, `|dx| < w + t` and `|dy| < h + t` →
  the frame's hit, worth 15, push time 5; side `Right` when `dx` ≥ 0, else `Left` [T-hit-left,
  T-hit-right, T-hit-jumping, T-hit-ready, T-reach-x-inside, T-reach-y-inside, T-side-left,
  T-wrap, T-wrap-across, T-wrap-y, T-reach-adjusted, FT-hit-rising, FT-hit-hovering, FT-reach-reference]
- Otherwise there is no hit: she out of reach on either axis [T-reach-x-edge, T-reach-y-edge,
  T-reach-adjusted-edge, T-curl-least, FT-reach-adjusted], her protection over 0 as the frame starts
  [T-protected, FT-protected], or he not tangible [T-rest-first, T-rest-intangible, T-curl-still].
- So her side is judged on where the last frame left her: at the tip's x or right of it she is
  pushed right. With her protection of 96 frames she is hit every 96 frames while she
  stays in reach.
- checked: oracle (recorded cases T2 to T7: every run 577-866 of the cached rows, three hits; W6, W7, W21 in flight);
  constructed (reach edges and adjusted reach at rest, the flight y edge with and without
  adjusted reach, and wraps on both axes).

## 15. Part 4c-1a, his take-off and flight

His roar over, Ridley takes off: he crouches and hops twice, up and to the left, then springs
into the air and rises fast, slows, and hovers near the top right of the room, drifting to a
point of his own above her, until she is down to her last energy and he gives up (part 4c-2) or
he turns to an attack (later parts). In the movie he rises for 20 frames, slows for 16 and
hovers for 79 before he gives up.

**Part 4c-1 is specced in two pieces, by mechanism.** This piece, 4c-1a, is his body's flight:
his take-off's figures and hops, his rise, his slowing, his hover and the pull to his point,
his move by his speed within the room's bounds, and where the flight hands over. The second,
4c-1b, is his tail in flight: its motion, its whips and their sound, and the place of its tip,
on which part 4b's `TailHits` judges the two hits the movie shows in his flight (cuts 897 and
993, both pushing her left). The tail in flight is §17.

**What this part reads and leaves.** It reads his phase and position (§13), her energy
(samus-health), the hits he has taken from her shots (a later part counts them) and the random
number (§12.3). It hands drawing his look and position, her touch, her shots and his tail
that he stays tangible and moving (§15.4), the screen's shaking (R3) a shake when he meets the
room's left side fast, and the parts after it the bound he met and the run he gives up or
attacks. Left to others: his tail in flight (4c-1b); his wings' flapping, which
follows his speed and is drawing only (C1); his retreat (4c-2) and his fake retreat, after her
100th hit on him (a later part); his attacks (later parts); his colours as her hits on him add
up (a later part: with fewer than 50 hits nothing changes, and his colours on a hovering run
with 50 or more are that part's); her knockback and hurt colours at the tail's hits
(samus-movement, samus-appearance).

**Running.** As in §13: every frame he runs is a run, and on a frame he doesn't run nothing of
him changes. A run from his take-off on goes in this order: his phase's rule (`TakesOff`,
`Rises`, `SlowsDown`, `Hovers`); his move by his speed (`Flies`); his tail (4c-1b); the baby,
held at his position as his move leaves it (`BabyHeld`); then his take-off's figure steps on,
with its hops and the lift-off (`TakesOff`). So a hop shows in the baby's place one run late.

### 15.1 What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| his phase | `RidleyPhase`, part 4a's and, listed alphabetically: `FakeRetreat`, `Fireballs`, `Lunge`, `Swoop` (later parts run him), `Hovering`, `Retreating` (part 4c-2 runs him), `Rising`, `Slowing`; `TakingOff` is his take-off | how he moves | every entry of this part |
| his position | x, y: px, 16-bit, each with a fraction, 0 to 255 in 1/256 px | where he is drawn and the baby held | `Flies`, `Rises`, `SlowsDown`, `Hovers`, `PulledToHisPoint`; output |
| his speed | x, y: 1/256 px per run, each a signed 16-bit number, −32,768 to 32,767; x rightward, y downward | how fast he moves | `Flies`, `Rises`, `SlowsDown`, `PulledToHisPoint` |
| the take-off's figure | `TakeOffFigure`, 1 to 5 (`TakesOff`), with its runs, 1 to its length; the fifth has no length and no runs | his look, his hops, his lift-off | `TakesOff` |
| his look | `RidleyLook`: part 4a's, and `TakeOff1` and `TakeOff2`, his take-off's two crouches, facing left | which figure is drawn | output (drawing) |
| his hover's runs | count, 0 to 123: the runs he has hovered without reaching his point; 0 from his placing | when he turns to an attack | `Hovers` |
| the bound he met | none, or `Bound`, listed alphabetically: `Bottom`, `Left`, `Right`, `Top`, on this run | which side of the room stopped him | output (later parts) |

### 15.2 What he remembers: state a caller can hold

§14.2's, and his speed, his position's fractions, the take-off's figure with its runs and his
hover's runs: nothing else. The bound he met is the run's alone. Any speed from −32,768 to 32,767
is allowed in every phase (`PulledToHisPoint` states its cap for each). A caller can build him in
any state this part allows and read it back.

### 15.3 What he reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| her energy | energy, as it stands when he runs: any hit of an earlier frame taken off (samus-health) | `Hovers` |
| his hits taken | count, 0 to 65,535, as it stands when he runs: how many of her shots have hit him since he was placed, a shot of the same frame included (counted by a later part, her shots against him) | `Hovers` |
| the random number | 16-bit, the one the baby reads on the same run (§12.3) | `Hovers` |

### 15.4 What he reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| his drawing | his look at his position (whole px), and the baby as in §12.4 | drawing (C1) |
| tangible | `bool`: true on every run of this part, in every phase from `TakingOff` through `Hovering` and on the run he turns to an attack (§13.4's `tangible`, which this part extends to `Rising`, `Slowing`, `Hovering` and the attacks' first run) | as in §13.4: her touch and her shots (C1); his tail's hits (part 4b's `TailHits`, on 4c-1b's tip) |
| moving | `bool`: true on every run of this part, as in §13.4 | as in §13.4 |
| roaring | `bool`: false on every run of this part; nor does any run of it ask for the fight's music or the roar | as in §13.4 |
| the bound he met | as in §15.1 | his attacks (later parts) |
| a shake | none, or a shake of the `Back` layers (§1.4), moving sideways, 3 px for 12 frames (`Flies`) | the screen's shaking (R3) |
| giving up | phase `Retreating` or `FakeRetreat`: all he does on that run and after is part 4c-2's or a later part's | part 4c-2, a later part |
| an attack | phase `Fireballs`, `Lunge` or `Swoop`: from the next run on a later part runs him | later parts |

### TakesOff
- kind: mechanic.
- summary: he crouches, hops up and to the left twice, then springs into the air.
- params: the take-off's figures, in order, with their lengths in runs: 1 `Rest` 3, 2 `TakeOff1`
  4, 3 `TakeOff2` 5, 4 `TakeOff1` 17, 5 `Rest`, held from then on. The hops, at the start of
  figure 2 (+1, −12) px and of figure 3 (−4, −8) px. The lift-off speed, (0, −352).
- switch: Fixed.
- The run he takes off (§13, `Roars`: `Roaring`, wait 0) → `TakingOff`; the take-off's figure 1,
  its first run; his speed (0, 0) [L-takeoff]
- A run, `TakingOff` → his speed (0, 0) before he moves (`Flies`) [L-figure-counts, L-hop-first]
- At the end of every run after the one he takes off: the figure shown fewer runs than its
  length → one run more [L-figure-counts, L-rise-counts]
- The same, figure 1 shown its 3 runs → figure 2, its first run; he hops (+1, −12) [L-hop-first]
- The same, figure 2 shown its 4 runs → figure 3, its first run; he hops (−4, −8)
  [L-hop-second]
- The same, figure 3 shown its 5 runs → figure 4, its first run; `Rising`, his speed
  (0, −352) [L-lift-off]
- The same, figure 4 shown its 17 runs → figure 5 [L-figure-rest]
- The same, figure 5 → figure 5 [L-slow-counts, L-hover-pull]
- A hop adds to his whole px, each mod 2^16, his fractions kept; it is not held to the room's
  bounds until his next move [L-hop-first, L-hop-wrap].
- Every run from his take-off on → his look, the figure's [L-hop-first, L-hop-second,
  L-lift-off, L-figure-rest]
- So he hops on his 4th and 8th runs, springs on his 13th and rests from his 30th.
- checked: oracle (recorded cases L1 to L6, every run 867-994 of the cached rows); constructed.

### Rises
- kind: mechanic.
- summary: he rises faster and faster until he is high enough, then starts to slow.
- params: the rise's pull, 16 (1/256 px per run, upward, each run); the height, y 112.
- switch: Fixed.
- A run, `Rising` → his y speed 16 less, mod 2^16 [L-rise-counts]
- Then, his y less 112, as a signed 16-bit number, negative → `Slowing`, and `SlowsDown` runs on
  this run too [L-rise-slows, L-rise-hovers]
- His y here is his whole px as the run starts, before he moves.
- checked: oracle (recorded cases L5, L7); constructed.

### SlowsDown
- kind: mechanic.
- summary: rising, he slows until he is high enough, then hovers.
- params: the slowing, 20 (1/256 px per run, downward, each run); the height, y 80.
- switch: Fixed.
- A run, `Slowing` → his y speed 20 more, mod 2^16 [L-slow-counts, L-rise-slows]
- Then, his y less 80, as a signed 16-bit number, negative → `Hovering`, from the next run
  [L-slow-hovers, L-rise-hovers]
- checked: oracle (recorded cases L7 to L9); constructed.

### Hovers
- kind: mechanic.
- summary: he hovers, drifting to his point; once she is nearly out of energy he gives up,
  and once he reaches his point, or has hovered a while, he attacks.
- params: his point, (192, 100); her energy's limit, 30; his hits' limit, 100; his patience,
  124 runs; the attacks, by the random number mod 16: 0 to 3, 9 and 15 `Fireballs`, 4 to 8
  `Lunge`, 10 to 14 `Swoop`.
- switch: Fixed.
- A run, `Hovering`, his hits taken 100 or more → `FakeRetreat`, whatever her energy: all he
  does on this run is a later part's [L-fake-retreat, L-fake-retreat-low]
- A run, `Hovering`, his hits taken under 100, her energy under 30 → `Retreating`: all he does
  on this run is part 4c-2's [L-gives-up, L-energy-edge-low]
- Otherwise he is pulled to his point (`PulledToHisPoint`) and then:
  - he is near it, his x within 16 px of 192 and his y within 16 px of 100, each difference
    taken mod 2^16 as a signed number → an attack, by the random number; his hover's runs 0
    [L-near, L-near-x-edge, L-attack-lunge-first, L-attack-lunge-last, L-attack-fireballs-9,
    L-attack-swoop-last, L-attack-fireballs-15]
  - not near, his hover's runs + 1 under 124 → his hover's runs one more, `Hovering`
    [L-hover-pull, L-energy-edge, L-hits-edge, L-not-near-edge, L-not-near-y-edge]
  - not near, his hover's runs + 1 of 124 or more → an attack, by the random number; his hover's
    runs 0 [L-patience]
- On a run he turns to an attack he still moves (`Flies`); from the next run on a later part
  runs him.
- Near is judged on his position as the run starts, before he moves.
- So in the movie he hovers 79 runs, never near his point, and gives up the run after the
  tail's second hit in his flight leaves her at 24.
- checked: oracle (recorded cases L10 to L14, every run 916-994 of the cached rows); constructed.

### PulledToHisPoint
- kind: mechanic.
- summary: each run he hovers, his speed on each axis is pulled to his point: harder the
  farther he is, three times as hard while he moves away from it.
- params: the pull's divisor, 16; the top speed, 1,280 (1/256 px per run) each way.
- switch: Fixed.
- Each axis on its own. Here `p` is his whole px on the axis, `q` his point's, `v` his
  speed on it. `D` is `(p − q) mod 2^16`; the point is ahead (right of him, or below) when `D`
  is 32,768 or more, behind when it is 1 to 32,767. The pull `d` is
  `max(1, floor(g / 16))`, with `g` = 65,536 − `D` when the point is ahead and `D` when it is
  behind. The odd unit `c` is 1 when `p` ≥ `q` as unsigned 16-bit numbers, else 0: 0 when the
  point is ahead and 1 when behind, except far apart as unsigned numbers: ahead with `p` 32,768
  or more above `q` (as when he is above the room's top) it is 1, and behind with `q` 32,769 or
  more above `p` it is 0. Each new speed is taken mod 2^16 as a signed number.
- `p` = `q` → his speed on that axis stays [L-pull-at-point]
- Ahead, `v` ≥ 0 → `v + d + c` [L-hover-pull, L-pull-apart]
- Ahead, `v` < 0 → `v + 3d + c + e`, with `e` = 1 when `v + 2d + c` ≥ 0, else 0
  [L-pull-turns, L-pull-away, L-pull-turns-early, L-pull-turns-late]
- Then, ahead, the new speed less 1,280, taken mod 2^16 as a signed number, not negative →
  1,280 [L-pull-top, L-pull-wrap]
- Behind, `v` < 0 → `v − d − (1 − c)` [L-pull-behind]
- Behind, `v` ≥ 0 → `v − 3d − (1 − c) − e`, with `e` = 1 when `v − 2d − (1 − c)` < 0, else 0
  [L-pull-back, L-pull-back-turns]
- Then, behind, the new speed plus 1,280, taken mod 2^16 as a signed number, negative → −1,280
  [L-pull-bottom]
- So turning back to his point costs one more than three pulls, on the run he turns, and any
  speed from −1,280 to 1,280 stays within it.
- checked: oracle (recorded cases L10, L11, L13: every run 916-993 of the cached rows,
  one turn at 963); constructed (each branch, the caps, the odd unit far apart).

### Flies
- kind: mechanic.
- summary: he moves by his speed, kept within the room's bounds; meeting the left side fast, he
  shakes the room.
- params: the bounds, x 40 to 224 and y −32 to 176 (px); the shake's speed, 640; the shake,
  `Back`, sideways, 3 px for 12 frames.
- switch: Fixed.
- A run from his take-off on, after his phase's rule: his x, in 1/256 px (his whole px × 256 +
  his fraction), gains his x speed, mod 2^24 [L-hover-pull, L-fraction-carry]; then
  - his whole x less 40, as a signed 16-bit number, negative → x 40, his fraction kept, his x
    speed 0, the bound `Left`; and a shake when the larger of his two speeds' sizes, taken
    before this stop, is 640 or more [L-bound-left, L-bound-left-slow]
  - otherwise his whole x less 224 not negative → x 224, his fraction kept, his x speed 0, the
    bound `Right` [L-bound-right]
- Then the same for y, against −32 (`Top`, no shake) and 176 (`Bottom`) [L-bound-top,
  L-bound-bottom, L-rise-counts]
- The bound he met is y's when he met one, else x's, else none [L-bound-corner].
- So a speed of 256 moves him 1 px a run.
- checked: oracle (every run 867-994 of the cached rows: no bound met); constructed (each
  bound, the shake's edge, the corner).

**Where the tail's hits read her.** His runs come after her pose has been read from her
controls and before her moves: the hits his tail gives her in his flight (part 4b's `TailHits`
on 4c-1b's tip) read her position as the frame starts, where the last frame left her, and the
body she started the frame in, as part 4b's §14.3 says.

**The baby in his flight.** `BabyHeld` holds as part 3 says, at his position as his move leaves
it: on a hop's run, before the hop (recorded case L2) [L-hop-first, L-hop-second].

## Part 4c-2, Ridley leaves the room

When her energy falls below the limit in `Hovers`, he changes course and flies above the room.
He still carries the baby, drifts sideways, and swings his tail as he leaves her behind.
He can pass his point before he stops pulling toward it. He coasts farther up, until the
room's upper limit stops him. His getaway picture and the walls take their colours while the
station is still calm.

This part runs from the real retreat selected by `Hovers` through the placing of the walls.
The next run starts the getaway picture. His retreat after her hundredth shot takes the
`FakeRetreat` handoff in §15. A caller can hold and restore the body, baby and tail in §§12–17,
including each of these phases of the real retreat.

| Quantity | Type and units | Read by |
| --- | --- | --- |
| his retreat | `RidleyPhase` continues with `CoastingAbove`, `GetawayReady` and `Retreating`. In `CoastingAbove` he has 0 through 63 more runs to coast before he stops | `FliesOutOfReach`, `GetawayBackdrop`, the getaway |
| the person he can hit | her whole-pixel position, body half-sizes and protection at the start of the frame (§14.3) | `TailHits` |
| placing the walls | a pair of `GetawayWall` placements, left and right, at the room's wall positions in its content | the actor framework, `GetawayWalls`, `TheDoorsColours` |
| preparing the colours | the three current colour selections in `GetawayBackdrop`, with the colours supplied by the room's content | drawing |

His position, fractions and speed use §15.1's units. The tail and baby read the same random
number as in §12.3. He stays tangible and moving on each run, including the run that places
the walls. `BabyHeld` uses his whole-pixel position after his move. The baby's own fidgets and
cries continue. The tail can still swish or hit her. A hit passes its side and five-frame push
to her movement and its worth of 15 whole energy to her health. He neither roars nor requests
music here. The door still reads `Calm`. The station's stage, her progress and the countdown
keep their incoming values through these runs.

### FliesOutOfReach

- kind: mechanic.
- summary: he pulls away from her, then coasts once he is high enough above the room.
- params: his point is (192, −128) px. The pull's divisor is 15. His upper bound is −192 px.
  His coast lasts 64 runs including its first run.
- switch: Fixed.
- `Hovers` selects the real retreat, or he starts the run `Retreating` → pull each speed by
  `PulledToHisPoint` using the retreat's point and divisor, including its odd unit, turn and caps.
  Move by `Flies` with the retreat's upper bound [RT-first-pull, RT-pull-divisor, RT-near-her]
- His whole y before that move is at or below the height of his point
  (`signed16(y + 128) >= 0`, y increasing down) → stay `Retreating`, even when this move
  crosses the point [RT-at-height, RT-crosses]
- His whole y before that move is above the point (`signed16(y + 128) < 0`) → this run's pull
  is the last. He ends it `CoastingAbove` with 63 more runs to coast [RT-coasts]
- `CoastingAbove` with more runs to coast → take one run off. Carry the speed into `Flies`
  with the same retreat bounds. A bound can stop him on one axis while he keeps moving on the
  other [RT-momentum, RT-ceiling, RT-last-coast]
- On these rising and coasting runs → swing the tail at the position left by his move, as in
  §17. He cannot aim a new whip in either phase. A whip already in motion can finish, and the
  usual curl, recovery, swish, stretch and hit still apply [RT-first-pull, RT-near-her]
- `CoastingAbove` with zero runs left at the start → stop both speeds before his move and
  prepare the room by `GetawayBackdrop` [RT-walls]
- checked: oracle (real retreat, crossing the point, coasting, upper bound and stop). Constructed
  at the exact height and with her in reach on a chance to whip.

### GetawayBackdrop

- kind: mechanic.
- summary: when he finishes coasting, the two wall actors are placed and the picture takes its colours.
- params: the wall positions, the getaway body's 15 colours and the walls' eight colours are content.
- switch: Fixed.
- His coast ends → place the left and right getaway walls in `Placed` (§11). Select the body's
  colours for `BackgroundBody` and the same wall colours for `BackgroundWalls` and
  `SpriteWalls`. Each selection starts after the transparent colour of its palette. He ends
  this run `GetawayReady`, with the station still `Calm` [RT-walls, RT-walls-and-tip]
- The tail on this last run → keep its swing directions, angles, lag, marks, pace and offsets.
  It keeps each segment's swinging state. Relink its places from the stopped body with those
  offsets (`TailBends`), and apply `TailStretches` with the stretching each segment brought
  in. `TailCurls`, `TailSwishes` and `TailHits` still apply [RT-walls, RT-walls-and-tip]
- checked: oracle (body stops, walls placed, colours reach the screen). Constructed with a
  fast, stretching tail and her within reach of the last tip.

## Part 4c-3, Ridley flies past with the baby

When his offscreen retreat leaves him `GetawayReady`, Ridley appears against the
room as a turning picture. The baby moves in his grasp and his wings beat. As he
flies past, the scenery shakes and Samus is pushed aside. When the
picture has gone the room is ready to show its warning.

| Quantity | Type and units | Read by |
| --- | --- | --- |
| his getaway | `GetawayReady` from part 4c-2, `FlyingPast` with the number of pictures already shown, or `WarningReady` | `GetawayPass`, the self-destruct warning |
| where the picture is and how far it has turned | a pair of signed whole pixels and a turn in 65,536ths, carried in `FlyingPast` | `GetawayPass`, `GetawayPicture` |
| the baby's beat and the wings' beat | a place in the baby's four-beat cycle and one in the wings' two-beat cycle | `GetawayPicture` |
| Ridley's path and art | room content. Each picture supplies a signed scroll change in whole pixels and a zoom in 256ths, 0 through 2,048. The room supplies nine sets of body colours, the baby's three looks and the wings' two looks | `GetawayPass`, `GetawayPicture` |
| finished frames | the game clock's count at the start of this frame | `GetawayPicture` |
| the station's stage, his sound, the push and the quake | a change to `Getaway`, the getaway sound, a request to move her aside, a diagonal shake of the back layer and the actors | the walls, sound transport, her movement and the room quake |
| the picture | the room view or the flyby plane, its scroll and pivot in whole pixels, its four drawing coefficients in 256ths, the selected body colours and any changes of look | drawing |

The caller can set, read and restore his getaway. A path has at least one
picture. In `FlyingPast` the count runs from one through the length of that
path. `GetawayReady` includes the first picture on the run that starts it. The animation beats count from zero. The room
starts them at zero and a restored getaway carries both. His airborne body, tail and held-baby motion end when the flyby begins. The path in Ceres has
112 pictures. Its motion is independent of her controls, position and health.
The quake owns its remaining time and displacements. Her movement owns the
direction, pose, travel and release of the push. His pass deals no damage.

### GetawayPass

- kind: mechanic.
- summary: Ridley flies across the room and turns as he passes her. His pass
  shakes the scenery and pushes her aside.
- params: the path is content. Positive scroll changes move the view right
  and down on the picture.
- switch: Fixed.
- `GetawayReady` on his next run → set the stage to `Getaway` and request
  the getaway sound once, on the second effects channel with a six-sound limit. Start the path at scroll (−128, 32) px with a turn of 1,
  keeping both animation beats. Show its first picture this run
  [G-first, G-first-between-beats]
- A picture remains → add its scroll change to the incoming scroll, wrapping
  each signed coordinate at 16 bits. Add 48 to the turn modulo 65,536.
  Show this picture by `GetawayPicture` at its own zoom. Count it as shown,
  including the last picture [G-before-pass, G-pass, G-last, G-wrap]
- On the 105th run of his pass, counting the first picture as one → ask her movement to move her aside.
  Start a diagonal 3 px shake of the back layer and the actors for 64 frames.
  On the surrounding pictures he makes neither request. If a shorter path has just
  shown 104 pictures, the same push and shake accompany its return to the room
  [G-before-pass, G-pass, G-after-pass, G-pass-on-return]
- `FlyingPast` with the whole path already shown → return to the room view
  with zero scroll, pivot and drawing coefficients. Hand off `WarningReady`.
  The stage stays `Getaway`. Only the room is shown on this run
  [G-room, G-short-room, G-pass-on-return]
- checked: oracle (both paths through his room, including their different pushes).
  Constructed for a restored turn, colour and scroll.

### GetawayPicture

- kind: output.
- summary: his picture turns and grows. The baby's look and his wings change
  to the beat of the game clock.
- params: the flyby pivots about (64, 64) px. Beyond its picture the plane is
  transparent. Drawing uses the whole 256th of a turn, `a = floor(turn / 256)`.
  For angle `a`, let `s(a) = trunc(256 × sin(2πa / 256))`, with exact
  quarter turns. Truncation is toward zero.
- switch: Fixed.
- A picture at zoom `z` → show it at the new scroll about the flyby pivot.
  Its drawing coefficients are `(c, s, −s, c)`, where
  `c = trunc(s(a + 64) × z / 256)` and `s = trunc(s(a) × z / 256)`.
  Select the body's set `floor(z / 256)`, counted from zero, for the 15
  colours after transparency in `BackgroundBody`
  [G-first, G-negative-turn, G-pass]
- A picture on a count of finished frames divisible by four → advance the baby
  one beat, wrapping after the fourth. Show the look of the new beat in the
  cycle `First, Middle, Third, Middle`. Otherwise keep her beat and picture
  [G-first, G-baby-alone, G-first-between-beats, G-wrap]
- A picture on a count divisible by eight → advance the wings one beat,
  alternating `First` and `Second`, and show the new look. Otherwise keep
  their beat and picture [G-first, G-baby-alone, G-first-between-beats, G-negative-turn]
- checked: oracle (the turn, zoom, body colours, baby and wing pictures).
  Constructed for a negative drawing coefficient and wrapping scroll.

**Order that matters.** The walls see the new stage on their first run. The
first sound and picture occur that same frame. The pass comes after her moves
and before the quake's update. Her movement can show her new pose on the
following frame. The quake spends its first frame on the pass itself.
The run after the last picture restores the room. The warning takes its first
step on the following run.

## 16. Part 5, the escape's steam

As she runs through the third and second rooms on her way out, steam vents in the station's
floors, walls and ceilings blow puffs. From the room's start each vent waits a moment, each its
own, then blows a puff that grows out of it and thins away over 21 frames, rests unseen for 64
frames, and puffs again; it does all this only while it is in view. A puff that touches her
throws her back as a hit does: she recoils, pushed away from the vent, is protected for a while
and flashes, but loses no energy. In the movie a puff touches her twice: once in room 3, once in room 2.

**What this part reads and leaves.** It reads each vent's facing and position (the room's content,
placed as the room starts, R2), the random number each vent's placing draws (the random numbers,
G2; where the vents' draws fall among the room's other placing draws is R2's to give), the camera
(world-camera), her position and body (samus-movement) and her protection (samus-health). It hands
the frame's hit to her energy (samus-health) and to her motion (samus-movement), and each vent's
puff to drawing. Left to others: the push, its poses and its leap (samus-movement §12); the
protection a hit grants (samus-health's `TailHit`, on the frame's impact, `Impact`, here worth 0
whole energy); her hurt colours (samus-appearance); drawing the puffs, whose sprites are content
(C1); her shots meeting a puff (C1); the shaft's vents in the escape, drawn where the turning room
puts them (part 7).

**Running.** The caller runs the room's vents together, in the room's order of its enemies,
once on each frame the enemies run (the enemy framework, C1); on a frame they don't (the room
loading, a door's passage) nothing of a vent changes. Runs below are those frames. On a run, a
vent in view first meets her with the phase it started the run in (`PuffTouches`), then steps
(`VentWaits`, `VentPuffs`, `VentRests`); a vent out of view does neither (`VentInView`). Like
every hitter, the vents read her position, body and protection as the frame starts, before she
moves this frame, as Ridley's tail does (§14.3): the frame's hit is made before her moves, and
her movement's reaction later in the frame starts the push.

### 16.1 What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| its facing | `VentFacing`, listed alphabetically: `Down`, `Left`, `Right`, `Up`: the way its puff blows | which way the puff grows; where it reaches | `PuffReach`; output (drawing) |
| its position | x, y: px, 16-bit | where the puff is drawn and reaches | `VentInView`, `PuffReach`, `PuffTouches`; output |
| its phase | `VentPhase` (below) | whether a puff is out, which figure, when the next comes | every entry of this part |
| the puff's reach | a box: its left, top, right and bottom, px, 16-bit, each edge included, from its position, facing and phase (`PuffReach`) | where the puff touches her | `PuffTouches` |
| the frame's hit | none, or the shared hit (§14.1): its side (`Side`: `Left` or `Right`, the way it pushes her) and its push time (`PushFrames`, frames: 5) | she recoils and flashes, her energy unchanged | output |

**Phases**, listed alphabetically (the order means nothing):
- `Placed(w)`: as the room placed it, never run; `w` from 1 to 32, its wait. Nothing is drawn,
  but it reaches one pixel (`PuffReach`).
- `Puffing(k, n)`: its puff out, showing figure `k`, 1 to 7, for `n` more runs, 1 to 3.
- `Resting(n)`: no puff, for `n` more runs, 1 to 255 (the rest's range, `VentRests`).
- `Waiting(w)`: no puff yet; `w` from 1 to 32, the runs before its first.

The puff's seven figures, from a small puff at the vent to a thin one far from it, are drawn
from the content for its facing; this part only names which one a vent shows.

### 16.2 What it remembers: state a caller can hold

Each vent's facing and position, which never change, and its phase with its count: nothing
else. Whether a puff is shown, its figure and its reach follow from them. A caller can build a
vent in any phase this part allows and read it back.

### 16.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| the random number | 16-bit: the number the vent's placing draws, one draw per vent, in the room's order (G2) | `VentPlaced` |
| the camera | x, y: px, 16-bit: the view's top left as the last frame left it (world-camera) | `VentInView` |
| her position | x, y: px, 16-bit: where she is as the frame starts, before she moves this frame (samus-movement) | `PuffTouches` |
| her body | her half-width and half-height, px, of the body she started the frame in, read from her movement's content | `PuffTouches` |
| her protection | frames, as the frame starts (samus-health's remaining protection) | `PuffTouches` |

### 16.4 What it reports (Outputs)

The hit is this part's alone: neither reader judges the reach, the side or the push time again.
It is the same shared hit Ridley's tail gives (§14.4), holding only its side and push time. Its
worth is each hitter's to state: a puff's is 0 whole energy (`PuffTouches`), and production
hands that worth to samus-health with the hit.

| Output | Type | Read by |
| --- | --- | --- |
| the frame's hit | none, or a hit with its side and push time (§16.1) | samus-health, in the same frame, as the frame's impact (`Impact`), an eligible hit worth 0 whole energy: she keeps her energy and gains its protection; samus-movement, in the same frame, in its reaction step (its §12 input, the frame's hit) |
| shown | `bool`, per vent: phase `Puffing` | drawing (C1) |
| its figure | none, or the puff's figure 1 to 7 with the vent's facing: phase `Puffing(k, n)` shows `k` | drawing (C1) |

### VentPlaced
- kind: mechanic.
- summary: placed with the room, each vent waits a random 1 to 32 runs before its first puff.
- params: none.
- switch: Fixed.
- The room places a vent, its placing's random number `r` → `Placed(1 + (r mod 32))`
  [ST-placed-least, ST-placed-most, ST-placed-high]
- checked: constructed; original only (no replay row reaches this entry: every row starts after
  its room has placed its vents, and their waits, 1 to 32, agree with the rule). It stays so
  until R2 places the enemies of rooms 3 and 2.

### VentInView
- kind: mechanic.
- summary: a vent runs only while it is in view, or nearly: out of view its puff stops where it
  is and touches nothing.
- params: the window: from 8 px left of the view's left edge to 264 px right of it, and from
  8 px above the view's top to 248 px below it, each end included.
- switch: Fixed.
- Here `(x, y)` is the vent's position and `(cx, cy)` the camera; each difference is taken mod
  2^16 as a signed number from −32,768 to 32,767.
- `x + 8 − cx ≥ 0`, `cx + 264 − x ≥ 0`, `y + 8 − cy ≥ 0` and `cy + 248 − y ≥ 0` → the vent
  runs: it meets her and steps [ST-view-left-edge, ST-view-right-edge, ST-view-top-edge,
  ST-view-bottom-edge, ST-view-wrap]
- Otherwise → nothing of it changes, and it gives no hit, whatever it reaches [ST-out-left,
  ST-out-right, ST-out-top, ST-out-bottom, ST-out-touching]
- checked: oracle (recorded cases ST14 to ST18; every frame of rooms 3 and 2: 814 vent-runs
  out of view, the left, right and bottom edges met at 0 and 1 px past); constructed (the top
  edge, the wrap).

### VentWaits
- kind: mechanic.
- summary: its first run starts its wait; when the wait runs out its first puff comes.
- params: none.
- switch: Fixed.
- A run, `Placed(w)` → `Waiting(w)` [ST-first-run, ST-placed-touched]
- A run, `Waiting(w)`, `w` over 1 → `Waiting(w − 1)` [ST-wait]
- A run, `Waiting(1)` → `Puffing(1, 3)` [ST-wait-ends]
- So a vent placed with the wait `w` shows its first puff on its `w + 1`th run.
- checked: oracle (recorded cases ST1 to ST3: 7 first runs, 116 waiting runs and 10 ends in the
  rows); constructed.

### VentPuffs
- kind: mechanic.
- summary: a puff shows its seven figures, three runs each, then the vent rests.
- params: the figures, 7; their runs, 3 each.
- switch: Fixed.
- A run, `Puffing(k, n)`, `n` over 1 → `Puffing(k, n − 1)` [ST-puff-counts, ST-hit-left]
- A run, `Puffing(k, 1)`, `k` under 7 → `Puffing(k + 1, 3)` [ST-puff-next, ST-hit-right,
  ST-touch-before-step]
- A run, `Puffing(7, 1)` → `Resting(p)`, `p` the rest (`VentRests`) [ST-puff-last,
  ST-rest-adjusted, ST-rest-adjusted-one]
- Every run that ends in `Puffing(k, n)` → shown, figure `k` [ST-wait-ends, ST-puff-next]
- Every run that ends in another phase → not shown, no figure [ST-puff-last, ST-rest]
- So a puff is out for 21 runs.
- checked: oracle (recorded cases ST4 to ST6: 261 counting runs, 113 figure changes and 20 ends
  in the rows); constructed.

### VentRests
- kind: mechanic and parameter.
- summary: between puffs the vent rests unseen, then puffs again.
- params: the rest, `p` = 64 runs (rule `steam_rest`); a rule from 1 to 255.
- switch: **Parameter**, the rest (params).
- A run, `Resting(n)`, `n` over 1 → `Resting(n − 1)` [ST-rest, ST-rest-first, ST-rest-adjusted-counts]
- A run, `Resting(1)` → `Puffing(1, 3)` [ST-rest-ends, ST-rest-ends-untouched]
- So with the rest of 64 a vent puffs every 85 runs: 21 out, 64 resting.
- checked: oracle (recorded cases ST7 and ST8: 20 first rest runs, 695 resting runs and 5 ends
  in the rows); constructed (the adjusted rests).

### PuffReach
- kind: mechanic.
- summary: a puff reaches a box that grows out of the vent the way it faces; its last two
  figures, thinned away, reach nothing, nor does a vent with no puff out.
- params: the boxes below.
- switch: Fixed.
- Cells: facing; figure; the box's left, top, right and bottom, px from the vent's position,
  x rightward, y downward, each edge included.

| Facing | Figure 1 | Figure 2 | Figure 3 | Figure 4 | Figure 5 |
| --- | --- | --- | --- | --- | --- |
| `Down` | −8, 0, 7, 14 | −8, 0, 7, 23 | −8, 8, 7, 31 | −8, 15, 7, 39 | −8, 23, 6, 38 |
| `Left` | −16, −8, −1, 7 | −24, −8, −2, 7 | −32, −7, −9, 7 | −40, −9, −17, 5 | −40, −11, −26, 2 |
| `Right` | 0, −8, 15, 7 | 1, −8, 23, 7 | 9, −8, 31, 7 | 18, −9, 38, 5 | 25, −11, 40, 3 |
| `Up` | −8, −16, 7, −1 | −8, −23, 7, −2 | −8, −32, 7, −8 | −8, −40, 7, −16 | −8, −40, 6, −24 |

- `Puffing(k, n)`, `k` 1 to 5 → the box of its facing and figure `k`, each edge its position's
  x or y plus the table's offset, mod 2^16 [ST-hit-left, ST-hit-right, ST-edge-left-in,
  ST-edge-right-in, ST-edge-top-in, ST-edge-bottom-in, ST-wrap]
- `Placed(w)` → the one pixel at its position: left and right its x, top and bottom its y
  [ST-placed-touched, ST-placed-missed]
- `Puffing(6, n)`, `Puffing(7, n)`, `Waiting(w)` or `Resting(n)` → no reach [ST-figure-6,
  ST-figure-7, ST-waiting-untouched, ST-rest-ends-untouched]
- checked: oracle (recorded cases ST9 to ST13, every in-view run of the rows: 276 vent-runs with
  a reach out); constructed (each edge, the wrap); original only (the boxes of figures no row
  brings her into).

### PuffTouches
- kind: mechanic.
- summary: a puff that touches her once her protection is spent hits her: she is pushed away
  from the vent and loses no energy; the frame has at most one hit.
- params: the push time, 5 frames; the hit's worth, 0 whole energy (handed with the hit to
  samus-health's `Impact`).
- switch: Fixed.
- Here `(x, y)` is the vent's position, `(hx, hy)` her position, `w` and `h` her half-width and
  half-height, and `L`, `T`, `R`, `B` the reach's edges, with the phase the vent started the run
  in; each difference is taken mod 2^16 as a signed number.
- A run of a vent in view, its reach not none, her protection 0 as the frame starts, no earlier
  vent's hit this frame, `L − (hx + w) < 0`, `R − (hx − w) ≥ 0`, `T − (hy + h) < 0` and
  `B − (hy − h) ≥ 0` → the frame's hit, push time 5, worth 0 whole energy; side `Right` when
  `hx − x ≥ 0`, else `Left` [ST-hit-left, ST-hit-right, ST-edge-left-in, ST-edge-right-in,
  ST-edge-top-in, ST-edge-bottom-in, ST-side-level, ST-wrap, ST-placed-touched,
  ST-touch-before-step, ST-second-vent]
- Otherwise this vent gives no hit: she out of reach on either axis [ST-near-miss,
  ST-edge-left-out, ST-edge-right-out, ST-edge-top-out, ST-edge-bottom-out, ST-placed-missed],
  her protection over 0 as the frame starts [ST-protected-room-2, ST-protected-after-hit,
  ST-protected-one], an earlier vent's hit this frame [ST-first-vent], or no reach
  [ST-figure-6, ST-figure-7, ST-waiting-untouched, ST-rest-ends-untouched].
- So a puff touches her when their boxes share a pixel column and row, her box running from
  `hx − w` to `hx + w − 1` across and `hy − h` to `hy + h − 1` down; the vent's position, not
  its puff's, decides the side.
- checked: oracle (recorded cases ST9 to ST13: the two hits, room 3 at the left and room 2 at
  the right, and every other in-view run of the rows with no hit); constructed (each edge, the
  order of two vents, the wrap).
## 17. Part 4c-1b, his tail in flight

As Ridley takes off his tail swings loose. The swing travels from its base to its tip.
Once he hovers he can whip the tail toward Samus. Its segments stretch in turn, then
draw in as they settle. The tip hits her twice in the movie, once as he rises and once
as he hovers.

**What this part reads and leaves.** It reads the phase and position his flight supplies (§15),
her position, body and protection as the frame starts (§14.3) and the random number (§12.3).
Aiming assumes the ordinary Ceres loadout, with no missiles. Aiming when missiles
are available belongs to the round that covers that equipment.
It supplies the seven segment positions to drawing (C1), a swish request for audio (U1), and
the hit from the tip to her health and movement (§14.4). Production runs these rules
from take-off through the last hover. C1 owns the segment sprites. The retreat and
the fake retreat own their changes to the tail and to his ability to hit. The tail
reports no hit while he holds her or carries the baby off in his fake retreat.

### 17.1 What a caller can hold

A caller can set and read back the tail state in the supported phases. Before take-off,
§14.2 holds its curl. From take-off through hovering, the state also holds the following
quantities in a chain of seven segments, ordered from base to tip.

| Quantity | Type and units | Read by |
| --- | --- | --- |
| pace | `SwingPace`, 1, 2 or 8 angle units per run | `TailFollows`, `TailSwings`, `TailSwishes` |
| mark | optional `TailAngle`, the angle the segments aim for ahead | `TailSwings` |
| swish count | `SwishRuns`, 0 to 15 runs | `TailSwishes` |
| swinging | bool per segment, whether the swing is active | `TailFollows`, `TailWhips`, `TailRecovers`, `TailSwings` |
| lag | per segment, `Counting` with a nonnegative count of angle units, or `Passed` once it has passed the swing along | `TailFollows` |
| way | per segment, `Ahead` or `Back`, the swing's direction | `TailFollows`, `TailSwings` |
| angle | `TailAngle` per segment, a 16-bit count of 256ths of a turn, increasing toward the back around the resting angle of 16,384 | `TailWhips`, `TailSwings`, `TailBends` |
| length | `TailLength` per segment, in 1/256 px, 2 px at the base, elsewhere from 0 to its cap in `TailStretches` | `TailBends`, `TailStretches` |
| stretching | bool per segment, whether it extends toward the length threshold | `TailStretches` |
| offset | per segment, x and y in signed 16-bit whole px | `TailBends` |
| place | per segment, x and y in unsigned 16-bit whole px | base only, `TailWhips`; all, output (drawing) |

A present mark is `16,128 + d` or `16,384 + d`, with `d` in 0 through 23 or
232 through 255 as in `TailWhips`. A caller supplies a mark in that range or supplies
no mark. The lag count reaches at most 30 before passing the swing. A segment can be held or
swinging while counting. A segment that passed the swing stays swinging until it settles
and starts counting again. Her position uses the same whole-pixel units as his and the
tail's places. The tail uses his position from `Flies`, before any hop. The run that selects a retreat stays
tangible and moving. Part 4c-2 owns that run, including its tail. This part uses his
phase after the phase rule, including any attack selected on that run.

### 17.2 Outputs

The seven places go to drawing in base-to-tip order. The swish is an audio request.
The hit is the shared event from §14.4. Her health applies the energy loss and
protection, and her movement applies the push.

### 17.3 Order that matters

On the take-off run `TailComesLoose` supplies the initial shape. The curl and swish count
use this run's random number (§12.3) and the pace before any change in pace, respectively.
A whip aims from the last run's base place. A new mark or recovery pace applies to the
same run. Segments pass the swing along and bend in base-to-tip order. Their places use
the lengths as the run starts. Their new lengths affect places on the next run. `TailHits`
uses the resulting tip and her position, body and protection as the frame starts. Its
worth is 15 whole energy and its push lasts 5 frames, as at rest.

### TailComesLoose

- kind: mechanic.
- summary: his tail starts swinging along its entire length as he takes off.
- params: starting pace 1, no mark, swish count 0. Base to tip, the initial lengths are
  2, 8, 8, 8, 8, 8 and 5 px. Angle `16,384 + 16i` for segment `i` numbered from the base at 0.
- switch: Fixed.
- He takes off → all seven segments swing `Ahead`, each counting lag from 17. Each starts
  at the resting place (§14) with offset (0, 0) and stretching false [FT-takeoff, FT-takeoff-tight]
- checked: oracle (W1); constructed (take-off with a tighter curl).

### TailFollows

- kind: mechanic.
- summary: each segment waits for the swing to reach it, then waits for its curl before moving.
- params: the curl from `TailCurls` and this run's swing pace.
- switch: Fixed.
- Swinging and counting below the curl → add the pace to its lag and keep its angle and
  offset this run [FT-lag-counts, FT-lag-edge, FT-takeoff-tight]
- Swinging and counting at or above the curl → its lag becomes `Passed` and it can
  move this run. If there is a following segment, it starts swinging this run in the
  passing segment's way [FT-takeoff, FT-passes-on, FT-lag-edge]
- Swinging and already `Passed` → it can move on this run without passing the swing again
  [FT-swing-ahead, FT-turn-ahead]
- checked: oracle (W1, W2, W16 to W18); constructed (curl and lag edges).

### TailSwings

- kind: mechanic.
- summary: each segment swings between its limits. A whip can pull a segment to a mark
  ahead, where it settles once the segment before it has settled.
- params: forward limit 16,368, back limit 16,448. Let `a` be the segment's angle and `p`
  the pace. Compare the angle differences below as signed 16-bit values after wrapping.
- switch: Fixed.
- Ready to move `Back`, `a + p - 16,448` negative → increase angle by `p`, wrapping
  at 16 bits [FT-swing-back]
- Ready to move `Back`, `a + p - 16,448` zero or positive → stop at 16,448 and turn
  `Ahead` [FT-turn-back]
- Ready to move `Ahead` → use the mark as the forward limit if there is one, else
  16,368. With a mark it starts stretching
  [FT-whip-stretch]
- Ready to move `Ahead`, `a - p - 1 - limit` zero or positive → decrease angle by `p`,
  wrapping at 16 bits [FT-swing-ahead, FT-ahead-exact]
- Ready to move `Ahead`, `a - p - 1 - limit` negative, with no mark or the preceding
  segment still swinging → set angle to 16,368 and turn `Back` [FT-turn-ahead, FT-aim]
- Ready to move `Ahead`, `a - p - 1 - limit` negative, with a mark and the segment is the
  base or the preceding segment has settled → set angle to the mark, stop swinging,
  turn `Back`, set lag to `Counting(0)` and keep the previous offset this run
  [FT-whip-settles, FT-swish]
- checked: oracle (W2, W4, W8, W9, W13, W14, W20); constructed (exact forward step and back swing).

### TailWhips

- kind: mechanic.
- summary: hovering or on an attack, he can whip his swinging tail at a spot below
  her. A chance to whip can occur even if she is far away.
- params: 128 px of horizontal reach from him, 24 px below her for his aim, and a chance
  on 16 out of 256 random values.
- switch: Fixed.
- All seven segments swinging, and he is `Hovering`, `Fireballs`, `Lunge` or `Swoop`:
  - `n mod 256` at least 240 → try mark `16,384 + d` [FT-aim-chance, FT-aim-rejected]
  - `n mod 256` below 240 and `abs(signed16(her x - his x)) < 128` → try mark
    `16,128 + d` [FT-aim, FT-reaim, FT-aim-near, FT-aim-attack]
- A mark tried and strictly below the base's angle as an unsigned number → set the
  mark to that angle and set pace 8 [FT-aim, FT-aim-chance, FT-aim-attack]
- A mark tried at or above the base's angle → keep mark and pace [FT-aim-rejected]
- No mark tried → the mark and pace stay as they were unless `TailRecovers` changes
  them [FT-chance-rising, FT-hit-rising, FT-aim-far, FT-restart, FT-lag-counts]

To obtain `d`, measure `dx` right and `dy` down from the base's last place to the point
24 px below where she began the frame. Take each as a wrapping signed 16-bit difference.
Let `u` be the integer quotient of 256 times the low eight bits of the lesser magnitude
divided by the low eight bits of the greater magnitude, then divided by 8 again, discarding
the remainder at both divisions. With zero divisor, the first quotient is 65,535.
Compute the bearing below mod 256. Directly down has bearing 0, directly right 64.

Cells: `x` and `y` denote `dx` and `dy`; `u` is the quotient above.

| Magnitudes | x ≥ 0, y ≥ 0 | x ≥ 0, y < 0 | x < 0, y ≥ 0 | x < 0, y < 0 |
| --- | --- | --- | --- | --- |
| `abs(y) >= abs(x)` | `u` | `128 - u` | `-u` | `128 + u` |
| `abs(y) < abs(x)` | `64 - u` | `64 + u` | `192 + u` | `192 - u` |

- Bearing 24 through 231 inclusive → use `d = 232` [FT-clamp-24]
- Otherwise → use the bearing as `d` [FT-bearing-0, FT-bearing-1, FT-bearing-2, FT-bearing-3, FT-bearing-4,
  FT-bearing-5, FT-bearing-6, FT-bearing-7, FT-bearing-zero, FT-clamp-23]
- checked: oracle (W5, W6, W8, W9, W12); constructed (chance, aim edges, bearing rows and
  zero divisor, attack hand-over).

### TailRecovers

- kind: mechanic.
- summary: when all segments settle, a new swing starts from the base.
- params: her x threshold, 112 px.
- switch: Fixed.
- Every segment held as the run starts → the base swings in its current way, clear the mark,
  and set pace to 1 if her unsigned x is below 112, otherwise 2 [FT-restart, FT-restart-slow]
- checked: oracle (W15); constructed (the lower side of the threshold).

### TailStretches

- kind: mechanic.
- summary: a whip stretches the tail, then the segments draw in again until they reach
  their resting lengths.
- params: growth 240 and shrinkage 128, in 1/256 px. The stretch threshold is 12 px. Base
  length is fixed. The remaining segments' caps are 24, 24, 22, 22, 18 and 5 px. Their
  resting thresholds are 8, 8, 8, 8, 8 and 5 px.
- switch: Fixed.
- A segment after the base, stretching after its swing → add 240 to length in 1/256 px, stopping at
  the segment's cap [FT-whip-stretch, FT-stretch-caps]
- A segment after the base, stretching after its swing and already longer than 12 px → clear stretching
  for future runs, with this run's growth still applied [FT-stretch-past, FT-stretch-edge]
- A segment after the base, without stretching and above its resting threshold
  → subtract 128, even when that takes it below the threshold [FT-shrink, FT-shrink-edge]
- A segment after the base, without stretching and at or below its resting threshold →
  keep length [FT-shrink-stops, FT-shrink-edge]
- The base's length update → keep both its length and the stretching state left by its
  swing [FT-whip-stretch, FT-stretch-caps]
- checked: oracle (W9 to W11, W19); constructed (caps, threshold and undershoot).

### TailBends

- kind: mechanic.
- summary: the curve of the tail comes from linking the offsets of its seven segments.
- params: the tail's anchor is 32 px right and 16 px down from his position before a hop.
- switch: Fixed.
- A segment outside the lag wait and not settling this run → replace its offset with
  `(floor(S(b) * l / 256), floor(S(b + 64) * l / 256))`. Here `l` is the whole part
  of the length in px, `b` is its new angle plus the preceding segment's new angle
  (base uses its own angle only), and `S(k) = trunc(256 * sin(2π * (k mod 256) / 256))`,
  truncated toward zero [FT-takeoff, FT-swing-ahead, FT-hop, FT-hit-rising]
- A run → put the base at the anchor plus its offset, and each following segment at
  the preceding segment's place plus its own offset. Both coordinates wrap mod 2^16
  [FT-hop, FT-whip-settles, FT-lag-counts, FT-hit-hovering, FT-place-wrap]
- checked: oracle (W1 to W3, W6, W13, W16, W21); constructed (coordinate wrapping).

### TailSwishes

- kind: mechanic.
- summary: while the tail swings fast, it makes a swish every sixteen runs.
- params: pace threshold 8, period 16 runs. Audio handles the swish request in
  its third effects channel with a queue limit of six.
- switch: Fixed.
- Pace below 8 as the run starts → set count 0 [FT-lag-counts]
- Pace at least 8 as the run starts and count below 15 → add 1 [FT-whip-stretch,
  FT-restart, FT-swish-next]
- Pace at least 8 as the run starts and count 15 → request the whip's swish and set
  count 0 [FT-swish]
- checked: oracle (W9, W14 to W16); constructed (the run before the swish).
