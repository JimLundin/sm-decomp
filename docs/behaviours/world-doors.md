# Doors between rooms, part 1, the passage: behaviour spec

What the player sees happen when Samus walks into a door that leads to another room: the room
goes quiet and dark around the HUD, the view slides across into the next room with her carried
along, she lands at the door's mouth, the new room's colours come up, and play goes on with her
moving as she was. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/world-doors.toml`](../../scenarios/vectors/world-doors.toml).

## Header

**Scope: the first of the domain's pieces.** The plan's R1, doors and elevators between rooms,
is specced in pieces. This piece is the passage from one room to the next by a door: from the
frame after she touches the door until the frame play goes on in the new room. Its doors lead
left, right, up or down. Later pieces add the elevators (the ride, and the door that leads
nowhere), the door that takes her into another area, and the passage's music and sound beyond
the requests it makes here.

**What it reads from samus-movement (S1).** That she touched a door to a room this frame, and
which: its doorway (`DoorwayId`, from our `World` records; samus-movement's `TouchedCells`
reports the door cell, whose content names the doorway). Her position, `P.S` on both axes, as
the frame of the touch left it.

**What it reads from the world (our `World` records, AF1).** The doorway's arrival (§1.3): the
way she leaves by, the view's place in the next room, and how fast the passage carries her, if
the doorway sets it; and the next room, which the room's loading (R2) loads. Whether either
room keeps the items dark (a room's own flag, §1.3).

**What it reads from the camera (CAM).** The camera's position, `P.S`, as the frame of the touch
left it. The passage then sets it (§1.4); the camera doesn't run during the passage.

**What it reads from the escape (B8).** Whether the escape's timer is running (`TheFades`,
`WhatRuns`).

**What it reads from audio (U1).** Whether the sound requests it waits on have all been sent
(`Quieting`), and whether the new room's music is ready (`Landing`).

**What it reads from the platform (T1).** The cost of each frame of work the passage does, by its
cost event (`PassageTime`): T1 answers, each frame, whether the work of the frame of work in
progress is done. The passage owns the rest of its timing: the counted frames of work, the waits,
the slide's wait and the finished frames (`PassageTime`, `FinishedFrames`). Until T1 models the
costs, the lock's recorded timing gives them, as it does for every domain the free run plays.

**What the other domains must carry unchanged across the passage.** Samus's State in
samus-movement, but for her position, which the passage moves (§1.4): her pose, facing, speeds,
momentum, and every timer of hers; she resumes as she was (`SheKeepsHerMotion`). The camera's
State, but for the two quantities the passage sets. The HUD's and progress's State. Each enemy's
State on the frames the enemies don't run (`WhatRuns`).

**Not in this spec** (§4 names who supplies each): building the next room: its level, tiles,
tilesets, enemies, objects, effects and music (R2, through the calls of §1.5); layer 2's
position over the arrival view (the backgrounds); the tile layers' drawing and their scroll
registers (the video encoder); the colours each domain writes while the screen is dark (the
enemies' and the room's, R2 and C1); what each running domain does on the frames it runs; which
sound she makes at the door and at the end (samus-appearance); the save station's mark of its
last use, cleared on leaving (P2).

**Notation.** A quantity written `P.S` is a pixel part and a subpixel part, each 16 bits, in hex,
as in samus-movement: `0.C800` is 0.78125 px. x grows rightward, y grows downward. A room is a
grid of screens, each 256 px square. A **frame** is a video frame; a **frame of work** is one
pass of the game's work, which ends on a frame (`PassageTime`). A **finished frame** is one whose
picture, sprites and colours are sent to the screen, whose pad is read and whose frame counts
step; a frame the work overruns is not one (`FinishedFrames`). A colour is 15 bits, three
channels of 5 bits: red in bits 0 to 4, green 5 to 9, blue 10 to 14.

---

## 1. What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the phase | `Phase` (below), with its data | what the screen shows and how long until play goes on | every entry |
| the doorway's arrival | `Arrival` (§1.3), held from the frame it is taken | where the view and she land | `Squaring`, `SettingOff`, `Sliding`, `Carrying`, `WhereSheLands` |
| the slide's frames | count, 0 to the slide's length | where the view is in the slide | `Sliding`, `Carrying`, `Arriving` |

**Phases**, listed alphabetically (the order means nothing; `PassageTime` gives the course):
- `Arriving { placed, fetching }`: she is placed in the next room and the view slides in while
  the room's tiles and then its contents are fetched; `placed` once she is; `fetching` is
  `Tiles`, `Contents`, or `Done` (the contents fetched, the frame of work held until the slide
  ends).
- `BeforeTheSlide(n)`, `n` from 0 to 2, the frames of work since the view was squared: the dark
  screen before the view sets off.
- `Brightening(t)`, `t` from 0 to 14: the new room's colours coming up; play goes on after it.
- `Darkening(t)`, `t` from 0 to 14: the room's colours going dark.
- `Landing(n)`, `n` from 0 to 1, the frames of work since the slide ended.
- `Music { started }`: the dark new room waiting for its track; `started` once the track has
  started.
- `Quieting { asked }`: the room falling silent; `asked` once the silence is asked for.
- `Squaring`: in the dark, the doorway's arrival taken and the view moved onto a screen's edge.

### 1.2 What it remembers: state a caller can hold

The phase with its data, the arrival once taken and the slide's frames: nothing else. Her
position, the camera's, the screen's colours and the colours they head for belong to their
domains; the passage reads and sets them (§1.3, §1.4). This is state the engine carries from
frame to frame, saves and restores, so a caller can build a passage in any phase this spec
allows and read it back: a vector's `before` builds it and its `after` reads it.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| the door touched | `DoorwayId`, on the frame of the touch (samus-movement) | `TheTouch` |
| the arrival | `Arrival`: the way she leaves, `Way` (`Down`, `Left`, `Right`, `Up`, listed alphabetically), the way the view and she move; the view's place in the next room, x and y in px, each a whole number of screens; her speed in the slide, `P.S` per frame, or none (the doorway's, from our `World` records) | `Squaring` |
| her position | x, y: `P.S` (samus-movement), as the frame before left it | `WhereSheLands`, `Carrying` |
| the camera | x, y: `P.S` (the camera), as the frame before left it | `Squaring`, `SettingOff` |
| the screen's colours | 256 colours as they stand (the palettes), each frame of the fade | `TheFades` |
| the new room's colours | 256 colours: what the room's loading set the screen to head for (R2) | `TheFades` |
| the HUD's colours | which of the 256 are the HUD's (content, `TheFades`): those always lit, the icon colours, and the escape timer's four | `TheFades` |
| either room keeps the items dark | `bool`: the room she leaves or the next one is marked to (each room's content) | `TheFades`, `FinishedFrames` |
| the escape's timer running | `bool` (B8) | `TheFades`, `WhatRuns` |
| the sounds sent | `bool`: every sound request so far has been sent (U1) | `Quieting` |
| the music ready | `bool`: no track waits in the queue of tracks, as this frame of work finds it (U1) | `Landing` |
| the work done | `bool`: the work of the frame of work in progress, of the cost event the passage names, is done by this frame's end (T1, or the lock's recorded timing) | `PassageTime` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| passing | `bool`: from the frame after the touch to the last frame of `Brightening`; while it holds, her motion, the camera, the blocks' run (world-blocks' `TheScriptsRun` is paused), the HUD and play time don't run | every domain; output |
| what runs | per frame of work: the enemies, their projectiles, the animated tiles, what is drawn, the escape's timer drawn, each a `bool` (`WhatRuns`) | C1, R4, samus-appearance, B8 |
| the cost event | per frame: `CostEvent` (`BuildRoom`, `FadeStep`, `FetchContents`, `FetchTiles`, `Other`, listed alphabetically), or none while the passage waits on its own (`PassageTime`) | T1 |
| a finished frame | `bool`, per frame (`FinishedFrames`) | the Tick, T1 |
| the frame count restarts | `bool`, per frame: the HUD's frame count is 0 before this frame counts (`FinishedFrames`) | H2 |
| the slide's screen | on the frames that restart the count: what is shown below the HUD, `FrontAndSprites` or `Sprites` (`FinishedFrames`) | the video encoder (E2) |
| her position | x, y: `P.S`, set by `WhereSheLands` and `Carrying` | samus-movement; output |
| the camera | x, y: the pixel parts, set by `Squaring`, `SettingOff` and `Sliding`; the subpixel parts are kept | the camera; output |
| where the camera last saw her | x, y: the pixel parts, set to hers at each placement and slide step; the subpixel parts are kept | the camera (`Remembering`) |
| layer 2 | moved by the slide's steps with the view (`Sliding`) | the backgrounds; output |
| the screen's colours | 256 colours, each frame of the fade, and her visor's (`Arriving`) | the palettes; output |
| the colours it heads for | 256: the dark colours, set on the first frame of work | the palettes; output |
| the room's calls | `RoomCall` (§1.5), each on its frame of work | R2, and each family it loads |
| the room's sound requests | the silence on the first frame of work; sounds held off from then until play goes on, and let through when it does | U1 |
| her sound requests | `AtTheDoor` on the first frame of work, `SpinAgain` on the last (`Quieting`, `TheFades`) | samus-appearance |
| the mini-map on | `true` on the first frame of work (H2's input `the mini-map turned on`) | H2 |

### 1.5 The room's calls

What the passage asks the room's loading (R2) to do, each on the frame of work its phase names.
R2 owns what each does and in what order; the passage owns when.

| Phase | Call |
| --- | --- |
| `Squaring`, taking the arrival | `LeaveRoom`: the room's objects that hold the line effects are let go |
| `BeforeTheSlide(0)` | `DrawTopRow`: for a way `Up`, the next room's top row is drawn ahead of the slide |
| `BeforeTheSlide(1)` | `BuildRoom(doorway)`: the next room, its version chosen by progress (P1): its effects, level and scrolls, as R2 orders them |
| `Arriving`, fetching `Tiles` | `FetchTiles`: the tileset's tiles and colours |
| `Arriving`, fetching `Contents` | `FetchContents`: the room's contents, a set whose order is R2's, each list alphabetical by what it concerns: what it fetches (her beam's tiles and colours, the cap closing behind her (W2), the common sprites' and her suit's colours, the effects and their layer, the enemies placed and started, the enemies' graphics, the room's music asked for, the room's objects with the door's and the room's own start) and what it clears (the animated tiles, the enemies' projectiles, the old room's objects, the palette effects, her shots in flight (W1)) |
| `Landing(1)` | `AnimateTiles`: one run of the animated tiles |
| `Music { started: false }` | `PlayMusic`: the next room's track, once the music is ready |
| `Music { started: true }` | `ReadLiquid`: her surroundings' liquid for her motion |

---

## 2. The course of a passage

### TheTouch
- kind: mechanic.
- summary: touching a door to a room starts the passage on the next frame.
- params: none.
- switch: Fixed.
- A frame she touches a door to a room → that frame plays out as usual, and the passage is in
  `Quieting { asked: false }` for the next [touch]
- A touch while passing → nothing: nothing she touches is read while play doesn't run
  [touch-passing]
- checked: constructed. The cached rows hold 18 touches, but the touch shows only in the
  record after it, so the harness can't compare it as matched.

### PassageTime
- kind: timing.
- summary: the passage takes a counted number of frames of work in each phase, a frame of work
  ends when T1 says its cost is spent, and the slide runs one step every frame whatever the work.
- params: none (the costs are T1's).
- switch: Fixed.
- Each frame of work has a cost event, the passage's report to T1: a frame of work of `Darkening`
  or `Brightening` → `FadeStep`; of `BeforeTheSlide(1)` → `BuildRoom`; of `Arriving` fetching
  `Tiles` → `FetchTiles`, fetching `Contents` → `FetchContents`; any other → `Other`
  [work-pending, building, place-right, loading-waits, settle]
- A frame whose work isn't done → the phase and its data stay, and nothing but the placing and
  the slide moves [work-pending, place-pending, work-pending-slide]
- A frame whose work is done → the phase's entry below acts, in that frame [silence-sent,
  dark-step, prepare, building, settle]
- `Arriving`, fetching `Done` → no cost: T1 is asked nothing, and the frame of work ends on the
  frame the slide ends (`Arriving`) [slide-left, slide-end]
- The course, in frames of work:

Cells: phase, frames of work, next phase

| Phase | Frames of work | Next |
| --- | --- | --- |
| `Quieting` | 1, then until the sounds are sent | `Darkening(0)` |
| `Darkening(t)` | `d + 3`, 15: `t` 0 to 14 | `Squaring` |
| `Squaring` | 1 to take the arrival, then 1 per px the view moves, then 1 | `BeforeTheSlide(0)` |
| `BeforeTheSlide(n)` | 3: `n` 0 to 2 | `Arriving` |
| `Arriving` | 1 for the tiles, 1 for the contents and the wait for the slide's end | `Landing(0)` |
| `Landing(n)` | 2: `n` 0 to 1 | `Music { started: false }` |
| `Music { started }` | until the music is ready, then 1 | `Brightening(0)` |
| `Brightening(t)` | `d + 3`, 15: `t` 0 to 14 | play |

- So with every frame of work done in its frame, she lands 65 frames after the view sets off
  for a slide left or right, and the rest of the course is 37 frames of work, plus the waits,
  each at least one: the sounds, the squaring and the music.
- T1's costs are long: in the cached rows each fade takes 37 to 45 frames, the room's building 7
  to 18, the tiles' fetch 21 to 51, and the whole passage 161 to 226 frames, of which the slide
  is 64. The contents' fetch always ended before the slide did.
- checked: oracle (every frame of 18 passages in the cached rows); constructed.

### FinishedFrames
- kind: timing.
- summary: from the slide's own first step until she steps clear, every frame is a finished
  frame, whatever the work, and the HUD's frame count starts again at each.
- params: none.
- switch: Fixed.
- A frame of `Arriving` once she is placed, of `Landing` or of `Music` → a finished frame, its
  work done or not; the HUD's frame count is 0 before the frame counts it, so 1 after; below the
  HUD the screen shows the front layer and the sprites, or only the sprites when either room
  keeps the items dark [slide-right, work-pending-slide, forced-wait, clear-right, slide-dark-items]
- Any other frame of the passage → a finished frame only when its work is done [place-pending,
  work-pending, bright-pending, dark-step]
- So the first such frame is the one after she is placed (the slide's first step that the view
  takes on its own), and the last the one she steps clear in; the next, the fade in's first,
  counts on from 1.
- checked: oracle (the finished frames and the HUD's frame count at every frame of the 18 cached
  passages); constructed.

### Quieting
- kind: mechanic.
- summary: the room falls silent before the screen darkens.
- params: none.
- switch: Fixed.
- `Quieting { asked: false }` → the silence is asked for, sounds are held off until play goes on,
  her sound at the door is asked for (`AtTheDoor`), the mini-map is turned on; `Quieting {
  asked: true }` [leaving]
- `Quieting { asked: true }`, the sounds sent → `Darkening(0)` [silence-sent]
- `Quieting { asked: true }`, sounds still to send → it stays [silence-waits]
- Which sound she makes at the door is samus-appearance's: the passage only asks.
- checked: oracle (the silence, the held sounds, her sound at the door and the mini-map at each
  cached passage; the sound waits of the cached rows: 1 to 4 frames of work); constructed.

### TheFades
- kind: mechanic.
- summary: the room's colours go to black but for the HUD's, and the new room's come up the same
  way; play goes on when they are up.
- params: the fade's steps, `Fading`'s.
- switch: Fixed.
- `Quieting { asked: false }` → the dark colours: each colour black, but the HUD's always-lit
  colours, which keep their colour [leaving, dark-items]
- `Quieting { asked: false }`, and neither room keeps the items dark → the icon colours keep
  theirs too, and so do the escape timer's four while it runs [leaving, dark-timer]
- `Quieting { asked: false }`, either room keeps the items dark → the icon colours and the
  timer's darken as well [dark-items, dark-timer-dark]
- `Darkening(t)` → every colour steps toward its dark colour by `Fading`'s step `t`;
  `Darkening(t + 1)`, or `Squaring` after `t` = `d + 2` [dark-step, dark-first, dark-last,
  dark-end]
- `Brightening(t)` → every colour steps toward the new room's colours by `Fading`'s step `t`;
  `Brightening(t + 1)` [bright-step, bright-first]
- `Brightening(t)`, `t` = `d + 2` → play goes on: passing ends, sounds are let through again,
  and her spin's sound is asked for again (`SpinAgain`; whether she makes one is
  samus-appearance's) [bright-end]
- Which colours are the HUD's is the HUD's content, imported with it, in three groups: those
  always lit (the mini-map and the text), the icon colours (the icons and the empty tanks), and
  the escape timer's four; the vectors name invented ones.
- checked: oracle (the 256 colours at every frame of work of the cached fades, the HUD's kept
  through every fade out; the sounds let through and her spin's sound at each cached end);
  constructed.

### Fading
- kind: mechanic and parameter.
- summary: each colour of a fade moves to its goal in even steps, each channel on its own.
- params: the steps, `d` = 12.
- switch: **Parameter**, the steps (params).
- A step with `t` 0 → no colour changes [dark-first]
- A step with `t` from 1 to `d` → each channel `c` of a colour not yet at its goal `g` becomes
  `c + q`, where `q = ⌊|g − c| × 256 / (d + 1 − t)⌋`, negated if `g < c`, and the sum
  `c × 256 + q` is taken `mod 2^16` and its high byte kept (`⌊(c × 256 + q) / 256⌋`, which
  rounds down) [dark-step, bright-step, fade-half, steps-adjusted]
- A step with `t` = `d + 1` → each colour is its goal [dark-last, steps-adjusted-last]
- A step with `t` = `d + 2` → no colour changes; the fade ends [dark-end, bright-end,
  steps-adjusted-end, bright-steps-adjusted-end]
- A colour already at its goal → it stays [dark-step]
- So a fade takes `d + 3` frames of work, 15, and a colour reaches its goal at the `d + 1`-th, 13th
  (the phases' `t` 0 to 14 are for `d` = 12; with another `d`, `t` runs 0 to `d + 2`).
- checked: oracle (every colour of the cached fades); constructed (the adjusted steps).

### Squaring
- kind: mechanic.
- summary: in the dark, the doorway's arrival is taken, then the view creeps to the nearest edge
  of a screen across the way she leaves.
- params: none.
- switch: Fixed.
- `Squaring`, the arrival not yet taken → the doorway's arrival is taken and held; the line
  effects' objects are let go (`LeaveRoom`) [reading]
- `Squaring`, the way `Left` or `Right`: the view's y on a screen's edge (`y mod 256` = 0) →
  `BeforeTheSlide(0)`, the view stays [square-done, touch-passing]
- `Squaring`, the way `Left` or `Right`, `y mod 256` 128 or more → y + 1, it stays
  [square-down]
- `Squaring`, the way `Left` or `Right`, `y mod 256` from 1 to 127 → y − 1, it stays
  [square-up, timer-drawn]
- `Squaring`, the way `Up` or `Down` → the same, on x [square-x]
- checked: oracle (18 squarings in the cached rows, one of 14 px); constructed.

### Building
- kind: mechanic.
- summary: the view sets off on the third frame of work after the squaring, and the next room
  is built on the second.
- params: none.
- switch: Fixed.
- `BeforeTheSlide(0)`, the way `Up` → the next room's top row is drawn (`DrawTopRow`) and the
  slide's still frame 0 is made; `BeforeTheSlide(1)` [prepare-up]
- `BeforeTheSlide(0)`, another way → a frame of work that does nothing; `BeforeTheSlide(1)`
  [prepare]
- `BeforeTheSlide(1)` → the next room is built (`BuildRoom`, cost `BuildRoom`);
  `BeforeTheSlide(2)` [building]
- `BeforeTheSlide(2)` → the view sets off (`SettingOff`) [set-off-right]
- checked: oracle (18 buildings in the cached rows, each one frame of work, 7 to 18 frames);
  constructed.

### SettingOff
- kind: mechanic.
- summary: the view is put one screen back from its arrival, along the way she leaves, and makes
  the slide's first step.
- params: the slide's, `Sliding`'s.
- switch: Fixed.
- `BeforeTheSlide(2)`, the way `Right` → the camera is the arrival, its x less 256 [set-off-right]
- `BeforeTheSlide(2)`, the way `Left` → the arrival, its x plus 256 [set-off-left]
- `BeforeTheSlide(2)`, the way `Down` → the arrival, its y less 224 [set-off-down]
- `BeforeTheSlide(2)`, the way `Up` → the arrival, its y plus 255, and the arrival's y is taken
  as 32 px lower from then on [set-off-up]
- Then, in the same frame, a slide frame (`Sliding`), and `Arriving { placed: false, fetching:
  Tiles }` [set-off-right, set-off-left, set-off-down, set-off-up]
- For `Up`, the slide's frame 0 was made in `BeforeTheSlide(0)`, so this one is its frame 1
  [prepare-up, set-off-up]
- checked: oracle (the ways `Left` and `Right`, every cached passage); original only (`Up` and
  `Down`: no cached row passes a door up or down); constructed.

### Sliding
- kind: mechanic and parameter.
- summary: the view slides across into the next room by a fixed step every frame, whatever the
  game's work, and stops on its arrival.
- params: the step, `s` = 4 px; the slide's length, `n` = 64 frames left or right, 57 up or down.
- switch: **Parameter**, the step and the lengths (params).
- A slide frame, the way `Left` or `Right` → the slide's frames + 1; the view's x moves `s` px
  the way she leaves (layer 2 with it); she is carried (`Carrying`) [slide-right, slide-left,
  slide-adjusted]
- A slide frame, the way `Down`, the slide's frames 0 → only + 1: no move [slide-down-first]
- A slide frame, the way `Down` or `Up`, the slide's frames 1 or more → + 1; the view's y moves
  `s` px the way she leaves (layer 2 with it); she is carried [slide-down, slide-up,
  carry-adjusted-up]
- The slide's frames reaching `n` → the camera is the arrival, wherever the steps left it: the
  slide has ended [slide-end, slide-up-end, slide-adjusted-end, slide-adjusted-down-end,
  slide-adjusted-up-end, slide-adjusted-left-end]
- `SettingOff`'s slide frame is the slide's first (for `Up`, its second); the next runs on the
  frame after the one she is placed in, and the rest one each frame, at the frame's start,
  every frame, until it ends [slide-right, work-pending-slide, place-right]
- So left or right, 64 steps of 4 px bring the view the 256 px back onto the arrival; down, 56
  of them the 224; up, 56 of them leave it 31 px below the arrival, and the end puts it on the
  arrival taken 32 px lower (slide-up-end).
- checked: oracle (every frame of the 17 cached slides); original only (`Up`, `Down`);
  constructed (the adjusted step and lengths).

### Carrying
- kind: mechanic and parameter.
- summary: she is carried along with the slide at the doorway's speed, or the passage's own.
- params: the speeds when the doorway gives none: `0.C800` px a frame left or right, `1.8000` up
  or down.
- switch: **Parameter**, the two speeds (params).
- A slide frame that moves the view → her position along the way she leaves moves by the speed,
  `P.S` with the subpixel's carry, the way she leaves; where the camera last saw her becomes her
  pixel parts [slide-right, slide-left, slide-down, slide-up, carry-adjusted]
- The arrival with a speed → that speed [carry-own]
- The arrival with none → the passage's own speed for the way [carry-adjusted,
  carry-adjusted-down, carry-adjusted-up, slide-adjusted-left-end, slide-right]
- So she travels `64 × 0.C800` = 50 px left or right, and `56 × 1.8000` = 84 px up or down,
  over the slide.
- checked: oracle (every frame of the 17 cached slides, all at `0.C800`); original only (up,
  down, a doorway's own speed); constructed.

### Arriving
- kind: mechanic.
- summary: she is placed in the next room as the slide begins; the room's tiles and then its
  contents are fetched; she lands on the frame the slide ends.
- params: none.
- switch: Fixed.
- The first frame of `Arriving`, its work done or not → she is placed (`WhereSheLands`);
  `placed` [place-right, place-pending]
- `Arriving`, fetching `Tiles`, its work done → the tileset is fetched (`FetchTiles`); fetching
  `Contents` [place-right]
- `Arriving`, fetching `Contents`, its work done → the room's contents are fetched
  (`FetchContents`); fetching `Done` [loading-waits]
- `Arriving`, fetching `Done`, the slide not yet ended → it stays [slide-left]
- `Arriving`, fetching `Done`, the slide ended this frame → the blocks' run runs once
  (`WhatRuns`); her visor's colour becomes `(0, 31, 14)` (red, green, blue) at once;
  `Landing(0)` [loading-done, slide-end, slide-adjusted-end]
- checked: oracle (17 arrivals in the cached rows); constructed.

### WhereSheLands
- kind: mechanic.
- summary: she is moved into the next room's place, keeping where she was on the screen; after
  the slide her x settles on the far edge of its 8 px column, the way she went; then she is
  stepped away from a screen's second column or second-last column, and from a screen's second
  row, into the middle of the next.
- params: none.
- switch: Fixed.
- Placed, at the start of `Arriving`'s first frame → on each axis her pixel part becomes the
  camera's pixel part plus her old pixel part `mod 256`; her subpixel parts stay; where the
  camera last saw her becomes her pixel parts [place-right, place-left, place-pending]
- `Landing(0)`, the way `Right` → her x's pixel part `x | 7` [land-right]
- `Landing(0)`, the way `Left` → `x & ~7`, the 16-bit pixel part [land-left]
- `Landing(0)`, the way `Up` or `Down` → she stays [land-down]
- Where the camera last saw her stays as the slide left it [land-right]
- `Music { started: true }`, her x's pixel part with `x mod 256` from 16 to 31 →
  `((x | 15) + 8)`, the middle of the next 16 px column rightward [clear-right]
- `Music { started: true }`, `x mod 256` from 224 to 239 → `(x & ~15) − 8`, the middle of the
  next column leftward [clear-left]
- `Music { started: true }`, `y mod 256` from 16 to 31 → `((y | 15) + 8)`, the middle of the row
  below [clear-down]
- Any other x or y → it stays: a screen's second-last row is never stepped from [clear-none,
  clear-bottom]
- Where the camera last saw her stays [clear-right]
- checked: oracle (17 placings and landings: one steps left); original only (the ways up and
  down); constructed.

### Landing
- kind: mechanic.
- summary: she settles across on the first frame of work after the slide and the animated tiles
  run once on the second; the next room's track starts on the first frame of work after those
  two that finds the music ready; she steps clear on the frame of work after it starts, and the
  fade in begins on the next.
- params: none.
- switch: Fixed.
- `Landing(0)` → she settles across (`WhereSheLands`); `Landing(1)` [land-right]
- `Landing(1)` → one run of the animated tiles (`AnimateTiles`); `Music { started: false }`
  [settle]
- `Music { started: false }`, the music ready → the next room's track starts (`PlayMusic`);
  `Music { started: true }` [music-ready]
- `Music { started: false }`, the music not ready → it stays [music-waits, forced-wait]
- `Music { started: true }` → she steps clear (`WhereSheLands`), her liquid is read
  (`ReadLiquid`); `Brightening(0)` [clear-right]
- checked: oracle (17 landings; the music waits: one frame of work in 16 passages, 25 frames of
  work over 66 frames in one); constructed.

### SheKeepsHerMotion
- kind: mechanic.
- summary: nothing of her but her position changes across the passage.
- params: none.
- switch: Fixed.
- Every frame of the passage → her pose, facing, speeds and momentum are as the touch left them;
  play goes on with them (a harness case only: the vectors hold no pose)
- checked: oracle, harness only (her pose, speeds and momentum at every frame of 18 passages).

### WhatRuns
- kind: mechanic.
- summary: the enemies go on while the room is lit; nothing else of play runs.
- params: none.
- switch: Fixed.
- A frame of work of `Quieting`, or of `Darkening(t)` with `t` up to `d + 1`, 13 → the enemies
  run, and she, her shots, the enemies and their projectiles are drawn, still, where they are [leaving,
  silence-waits, dark-step]
- A frame of work of `Brightening(t)` → the animated tiles, the enemies and their projectiles
  run, and all are drawn [bright-step]
- A frame of work of `Landing(1)` → the animated tiles run [settle]
- The frame of work `Arriving` ends in → the blocks' run runs once (world-blocks'
  `TheScriptsRun`), before the colours come up [loading-done]
- Every frame of work but the first and the contents' → the escape's timer is drawn, as it
  stands, while it runs [timer-drawn, land-right, loading-done]
- The first frame of work → the timer is drawn while it runs and neither room keeps the items
  dark [dark-timer, dark-timer-dark]
- Any other frame of the passage → none of them runs [square-done, dark-end]
- checked: oracle (the enemies' frame counts at every frame of work of the 18 cached passages);
  constructed (the tiles, the projectiles, the timer and what is drawn).

## 3. Order that matters

- The slide's frame runs at each frame's start, before the passage's frame of work, so the frame
  of work that waits for the slide ends in the frame the slide does.
- She is placed before the slide's next frame carries her.
- The touch's frame plays out whole (her motion, the camera, the enemies); the passage starts on
  the next.
- Where the camera last saw her is set by the placing and every slide frame, not by `Landing`:
  after the passage the camera measures from her position at the slide's end.
- The queue of tracks takes its step at the start of the frame of work, before
  `Music { started: false }` reads it (U1).

## 4. Out of scope, and who supplies it

- **The room's loading (R2):** what each of the room's calls (§1.5) loads, and in what order, and
  the new room's colours; the colours it and the enemies write straight to the screen while it
  is dark; the HUD's `a boss encountered` (interface-hud), on entering the room of a boss.
- **The platform (T1):** the cost of each frame of work by its cost event (`PassageTime`); until
  it is modelled, the lock's recorded timing.
- **The backgrounds:** layer 2's place over the arrival view, by the room's rates; the passage
  only moves it with the slide.
- **The video encoder (E2):** the tile layers drawn as the view slides, and their scroll
  registers; the slide's screen (`FinishedFrames`).
- **Audio (U1):** the sound and music requests, when they are sent, the queue of tracks and the
  music's loading.
- **samus-appearance:** her sound at the door (`AtTheDoor`) and her spin's sound at the end
  (`SpinAgain`): which sound, if any, by her movement, the pad and her charge.
- **The weapons (W1):** her shots in flight, cleared by `FetchContents`.
- **Saving (P2):** the save station's mark of its last use, cleared on the first frame of work.
- **The enemies (C1) and their families:** what each does on the frames it runs; which run when
  out of view.
- **Later pieces of R1:** the elevators, and the door that takes her into another area.

## 5. Open questions

1. No cached row passes a door up or down, or one whose doorway gives her speed: their arms are
   the original's only. A short movie through a vertical door would check them.
2. The colours that the room's loading and some enemies write straight to the screen while it is
   dark (Kraid's, the beacon of the elevator's room in Ceres) are inputs here; R2, C1 and B8 must name them.
