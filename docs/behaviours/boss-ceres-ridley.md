# Ceres Ridley and the escape, part 1, the Ceres doors: behaviour spec

What the player sees the doors of the Ceres station do: a closed door that slides open as Samus
comes near and shuts behind her once she has gone, the hiss it makes as it opens, the door in the
shaft that becomes a wall once Ridley has fled, and the station shaking during the escape, which the
doors start. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/boss-ceres-ridley.toml`](../../scenarios/vectors/boss-ceres-ridley.toml).

## Header

**Scope: the first of the domain's pieces.** The plan's B8, Ceres Ridley and the escape, is
specced in pieces, each verifiable on its own. This piece is the Ceres doors: the doors between
Ceres's rooms (each faces left or right), the shaft's door (a door before the getaway, a wall
after it), and the shaking the doors start during the escape. Later pieces add the rest of the
station's own actors: the door of Ridley's room (a later piece); the shaft's
cover and its rumble before the station blows up; the getaway's two walls; Ceres Ridley himself and
his getaway; the escape's steam; the falling debris and the dust.

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
   happens (`Solid`). Each frame, her motion, her touch and her shots meet a door as blocking if it
   was solid when the frame began, before its run; it is drawn as it is after its run (§4;
   original only).
4. **Placing.** When a room starts, the framework places each door from the room's content: its
   kind, its facing and its position, in the state `RoomStart` names.

**Not in this spec** (§6 names who supplies each): the framework's four parts above; what the
shake does to the view and how it counts down (R3); the sound queue and what happens to a request
it can't hold (audio, U1); fading the doors' colours in with the room (the palettes, R4); the
room's other enemies, and the door of Ridley's room, the shaft's cover and the getaway's walls
(later pieces).

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
| the doors' colours | when the room places a door: which of the doors' two colour sets, `Calm` or `Escape`, the palette they are drawn with takes (`TheDoorsColours`); the 15 colours of each set are the doors' content | output: the palettes (R4) |

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
- checked: oracle (whether it is solid, every frame of the cached rows); original only (her touch
  and her shots: no cached row has her meet a door while it blocks).

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
- Her motion, her touch and her shots meet a door as blocking if it was solid when the frame began,
  before its run: the frame a door's `Opening(4)` steps to `Open` it still blocks her, and the
  frame its `Open` steps to `Closing(1)` it doesn't block her yet. It is drawn as it is after its
  run (`Solid`, `Shown`; original only: no cached row has her meet a door those frames).
- The doors run in the room's order of its enemies, and a shake one starts is in the frames left
  the next one reads (`EscapeShakes`).
- The opening sound is asked for in the run that takes `Opening(1)`'s figure.

## 5. Out of scope, and who supplies it

- **The enemy framework (C1):** running each door once a frame the enemies run, in order, and
  counting its frames; drawing its figure; solid contact, her touch and her shots; placing it at
  the room's start (Header).
- **The screen's shaking (R3):** moving the view and the enemies with each shake, and counting its
  frames down.
- **Audio (U1):** the sound queue and its limit.
- **The palettes (R4):** fading the doors' colours in with the room.
- **The random numbers:** the number and when it steps.
- **Progress and the escape (P1, B8's later pieces):** the station's stage.
- **Later pieces of B8:** the door of Ridley's room; the shaft's
  cover and its rumble; the getaway's walls.

## 6. Open questions

1. Her touch and her shots on a door while it blocks: no cached row has her meet one (any% never touches a
   closed door), so `Solid`'s touch and shot arms are the original's only. A short trick movie
   that walks into a closing door and shoots one would check them.
2. `Getaway` with a door of this piece placed: no row places one in that stage (the getaway happens
   in Ridley's room, whose door is a later piece). The arms that read it follow the original.
