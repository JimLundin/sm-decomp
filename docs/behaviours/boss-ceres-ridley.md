# Ceres Ridley and the escape, parts 1 and 2, the Ceres doors and the shaft as she arrives: behaviour spec

What the player sees the doors of the Ceres station do: a closed door that slides open as Samus
comes near and shuts behind her once she has gone, the hiss it makes as it opens, the door in the
shaft that becomes a wall once Ridley has fled, and the station shaking during the escape, which the
doors start; and, as a new game begins, the elevator's pad lowering her down the shaft and
letting her go, the shaft's beacon pulsing and its platform turning. Written behaviour first,
from the oracle's records over time. Vectors:
[`scenarios/vectors/boss-ceres-ridley.toml`](../../scenarios/vectors/boss-ceres-ridley.toml).

## Header

**Scope: the first of the domain's pieces.** The plan's B8, Ceres Ridley and the escape, is
specced in pieces, each verifiable on its own. This piece is the Ceres doors: the doors between
Ceres's rooms (each faces left or right), the shaft's door (a door before the getaway, a wall
after it), and the shaking the doors start during the escape. The second piece (§7 to §9) is the
shaft as she arrives: the elevator's pad that holds her, lowers her and lets her go, the cover
over the shaft's floor below it, the beacon's colours and the platform's turning look before the
escape. Later pieces add the rest of the station's own actors: the door of Ridley's room; the
shaft's rumble before the station blows up and its beacon during the escape; the getaway's two
walls; Ceres Ridley himself and his getaway; the escape's steam; the falling debris and the dust.

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
   kind, its facing and its position, in the state `RoomStart` names.

**Not in this spec** (§6 names who supplies each): the framework's four parts above; what the
shake does to the view and how it counts down (R3); the sound queue and what happens to a request
it can't hold (audio, U1); fading the doors' colours in with the room (the door passage,
world-doors); the room's other enemies, and the door of Ridley's room and the getaway's walls
(later pieces). The shaft's overlay is part 2's (§7).

**Notation.** Positions are whole pixels, x rightward and y downward, each a 16-bit number written
in hex (`0123`). A door's position is the centre of its box. Frames are frames the door runs
(Header, the framework's first part).

---

## 1. What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| its kind | `DoorKind`: `Door` or `ShaftDoor` (below) | whether it becomes a wall | `RoomStart`, `ShaftWall` |
| its facing | `Facing`: `Left` or `Right`: the way it opens, the side its figures are drawn for | which figures it shows | every entry that takes a figure |
| its position | x, y: px, 16-bit | where it is drawn and where its box is; how near she is | `SheIsNear`; output (drawing, solid contact) |
| its phase | `Phase` (below) | whether it is shown, solid, opening or closing | every entry |
| its wait | frames, 1 to the larger of the poll and the hold: the frames before its next step; no run ends with a wait of 0 | when it next looks for her or shows its next figure | `Waiting` |

Its box is 16 px wide and 64 px tall, centred on its position: 8 px to its left and right, 32 px
up and down. The box is fixed (the doors' content); only whether it is solid changes (`Solid`).

**Kinds**, listed alphabetically (the order means nothing):
- `Door`: a door between two rooms.
- `ShaftDoor`: the shaft's door: a left-facing door until Ridley's getaway, a wall from then on
  (`ShaftWall`).

**Phases**, listed alphabetically (the order means nothing):
- `Closed`: shown, solid, its closed figure; it looks for her each time its wait runs out.
- `Closing(n)`, `n` from 1 to 4: shown, solid, showing the n-th figure of its closing.
- `Open`: hidden, passable; it looks for her each time its wait runs out.
- `Opening(n)`, `n` from 1 to 4: shown, solid, showing the n-th figure of its opening.
- `Placed`: as the room placed it: shown and solid, with no figure; its first run starts it.
- `Starting`: hidden, passable; it looks for her once, when its wait runs out, to choose its first
  phase.
- `Wall`: the shaft's door once Ridley has fled: hidden, solid.

**Figures**, by look and facing: `Closed`, `Ajar1`, `Ajar2`, `Ajar3` and `Open` (from closed to
open), each facing `Left` or `Right`, so ten figures. Which sprites each is belongs to the doors'
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

The figure faces the door's way unless the table says otherwise. The figure of a hidden phase
(`Starting`, `Open`, `Wall`) is never drawn, but the door holds it all the same, and it is
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

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| shown | `bool`, from its phase | output: the framework draws its figure (Header) |
| solid | `bool`, from its phase | output: the framework's solid contact (Header) |
| its figure | a figure (§1), or none, from its phase and facing | output: drawing |
| a sound | the doors' opening sound, with a queue limit of 6, or none (`Opening`) | output: audio (U1) |
| a shake | none, or a shake of the whole view, both backgrounds and the enemies, moving diagonally: 1 px for 2 frames, or 3 px for 4 frames (`EscapeShakes`) | output: the screen's shaking (R3) |
| the doors' colours | when the room places a door: which of the doors' two colour sets, `Calm` or `Escape`, the palette they are drawn with takes (`TheDoorsColours`); the 15 colours of each set are the doors' content | output: the passage's fade brings the screen to them (world-doors) |

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
- Phase `Open` or `Starting` → passable: she, her touch and her shots go through it as through air
  [D-open-near-stays, D-start-wait]
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
- Phase `Open`, `Starting` or `Wall` → hidden: nothing is drawn [D-open-near-stays, D-start-wait,
  D-shaft-wall]
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
- So with two doors in a room, the first to run starts the shake and the second, which reads the
  frames left after it, doesn't (§4).
- checked: oracle (the 860 frames the doors ran in the escape in the cached rows); constructed.

### TheDoorsColours
- kind: mechanic.
- summary: when a room with doors starts, the doors' palette takes the doors' colours, another
  set during the escape.
- params: none (the two colour sets are the doors' content).
- switch: Fixed.
- A door placed, the stage `Calm` → the doors' colours are the `Calm` set [D-placed-calm]
- A door placed, the stage `Escape` or `Getaway` → they are the `Escape` set [D-placed-escape,
  D-placed-getaway]
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
- **Later pieces of B8:** the door of Ridley's room; the shaft's rumble and its beacon during
  the escape; the getaway's walls.
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
2. `Getaway` with a door of this piece placed: no row places one in that stage (the getaway happens
   in Ridley's room, whose door is a later piece). The arms that read it follow the original.
3. Part 2: the pad's placing is recorded once (the arrival); her height there is 0. Heights below
   the floor at placing, and a pad placed in a room other than the shaft, are no row's: the arms
   that read them follow the original's arithmetic.
4. Part 2: the shaft's rumble, a later piece, owns the beacon and the platform in the escape;
   this piece shows no step and no look in the escape (`BeaconCycles`, `PlatformTurns`) and
   states them for the stages `Calm` and `Getaway` only. Answered for that piece from the records:
   when the shaft starts in the escape (she comes back to it), the overlay's first run steps the
   beacon and the platform once, by the counts of `BeaconCycles` and `PlatformTurns`, and from its next run
   the rumble takes over from them.

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
