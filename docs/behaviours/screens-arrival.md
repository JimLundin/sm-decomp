# Her arrival at Ceres as a new game begins: behaviour spec

What the player sees as a new game starts: the screen stays dark while the station is made
ready, then Samus is standing at the top of the station's shaft, facing the screen, and the
picture comes up from black to full brightness over one second while she waits on the
elevator's pad; once it is fully up, play begins. Written behaviour first, from the oracle's
records over time. Vectors:
[`scenarios/vectors/screens-arrival.toml`](../../scenarios/vectors/screens-arrival.toml).

## Header

**Scope: the first piece of the cinematics (the plan's F4), Samus's arrival.** This piece is the
arrival of a new game at Ceres: the dark wait while the game makes the start ready, the moment she arrives,
and the fade in up to play. It is the arrival's own State (where the arrival has got
to, the screen's brightness and the fade's count) and what the arrival hands the other domains
on the frame she arrives. Later pieces of F4 add the flights, the station's explosion, the
ending and the credits.

**What it reads.**
- **Whether the start is ready** (the start's loading, R2 and C1, with the platform's lag, T1;
  borrowed): the frame on which the game has made the start ready (the start's room in place
  with its enemies, the HUD up, the room's music loaded). How long that takes, its fixed waits
  included, is theirs, so it is an input, true on one frame.
- **The start** (content, the new game's start in our format): the room the game starts in and
  her place across it, x in pixels. Its room and its enemies are the room loading's (R2, with
  the enemy framework's placing, C1).
- **Her suit's colours** (samus-appearance's content): the sixteen colours of her suit as it
  stands, the power suit's at a new game.
- **The screen's goal colours** (the door passage's palette, world-doors), as the start's
  loading leaves them: her block, 192 to 207, holds the start's own goal colours (content of the
  new game's start), the rest the room's loading's (R2).

**What it hands the others, on the frame she arrives** (§1.4): her place and her pose, to
samus-movement and samus-appearance; the moment the arrival places the elevator's pad, with her
place then, to the station (boss-ceres-ridley §7.3, `PadPlaced`); the screen's colours; the
mini-map shown, to the HUD (interface-hud). Each frame of the fade in, the screen's
brightness, to the renderer.

**What it leaves.** The station's pad, which then holds her and lowers her, is the station's
(boss-ceres-ridley part 2). What her frame does while the pad holds her is samus-movement's and
samus-appearance's. Bringing in the room, placing its enemies and putting up the HUD are R2's, C1's and
the HUD's. Starting the file (her energy, items, play time and the station's stage) is the new
file's (P2, run-progress). What plays in the fade in (her frame, the enemies, the
projectiles, the HUD, the random number, play time) runs as in play (§4); this domain only adds
the fade's step and says when play begins.

**What it replaces:** nothing. The engine has no arrival: the played runner refuses a seed
outside ordinary play.

**Notation.** Positions are whole pixels, x rightward and y downward, 16-bit, written in hex
(`0080`). Colours are 15-bit colours in hex, each named by its index among the screen's 256
colours (0 to 255). Frames are frames of the arrival; a frame here is one the game finishes (a
lag frame is the platform's and carries nothing of this domain over).

---

## 1. What the player can tell

### 1.1 Quantities

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the arrival's stage | `ArrivalStage`: `Preparing`, `FadingIn` or `Playing` | whether the screen is dark, coming up, or she plays | every entry |
| the screen's brightness | `Brightness`: a level, 0 to 15, and whether the screen is `Blanked` or `Shown` | how bright the picture is; a blanked screen shows nothing at any level | `PictureBrightens`, `PlayBegins`; output (the renderer) |
| the fade's count | frames, 0 to 32,768: the frames left before the fade's next step | when the picture next brightens | `PictureBrightens`; set by `ArrivesAtTheTop` |
| the fade's rhythm | frames, `d` (`PictureBrightens`): a rule, not State | how many frames each level holds | `ArrivesAtTheTop`, `PictureBrightens` |

### 1.2 What it remembers: state a caller can hold

The stage, the brightness (its level and whether it is blanked) and the fade's count: nothing
else. A caller can build any of them in any value this spec allows, read them back, carry them
and restore them; a vector's `before` builds them and its `after` reads them back.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| the start ready | `bool`: the game has made the start ready this frame (the start's loading, R2 and C1, with T1's lag; borrowed) | `StartsInTheDark`, `ArrivesAtTheTop` |
| the start | content: her x, px, 16-bit (the room is R2's) | `ArrivesAtTheTop` |
| her suit's colours | sixteen 15-bit colours (samus-appearance's content, her suit as it stands) | `ArrivesAtTheTop` |
| the screen's goal colours | 256 15-bit colours (world-doors' palette): 192 to 207 the start's goal colours for her block (content), the rest as the room's loading left them (R2) | `ArrivesAtTheTop` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| she arrives | an event, on the arrival's frame only, with her place `(x, y)`, px, 16-bit, and her pose | samus-movement (her place; she stands still, facing the screen, with no speed); samus-appearance (her pose's look); boss-ceres-ridley (`PadPlaced`, with her place) |
| the screen's colours | the 256 colours as they stand, on the arrival's frame (`ArrivesAtTheTop`); their goal is left as it is | output (the renderer); world-doors (the colours a later fade starts from) |
| the mini-map shown | an event, on the arrival's frame | interface-hud |
| the screen's brightness | `Brightness` (§1.1), every frame | output (the renderer) |
| play begins | `bool`: the stage is `Playing` | the caller: from the next frame the game plays as in any room |

---

## 2. The preparation and the arrival

### StartsInTheDark
- kind: mechanic.
- summary: while the game makes the start ready, nothing of the arrival moves and the screen
  stays as dark as it was.
- params: none.
- switch: Fixed.
- `Preparing`, the start not ready → the stage, the brightness and the fade's count are left as
  they are; no output but the brightness [R-setup, C-setup-lit]
- checked: oracle (recorded case S1); constructed.

### ArrivesAtTheTop
- kind: mechanic.
- summary: on the frame the preparation is done she appears at the top of the shaft, facing the
  screen, the room's colours go up on the screen and the mini-map shows; the fade in starts on
  the next frame.
- params: the arrival's height, 0 px (the top of the room); the colour the arrival turns
  black, 223; her colours, 192 to 207.
- switch: Fixed.
- `Preparing`, the start ready, the start's x `x` → the stage `FadingIn`, the fade's count `d`,
  the brightness left as it is; she arrives at `(x, 0000)`, standing still, facing the screen;
  the arrival places the pad with her at `(x, 0000)`; the mini-map is shown [R-arrival,
  C-arrival-adjusted, C-arrival-lit]
- The same arrival → every one of the screen's colours takes its goal, then colours 192 to 207
  take her suit's sixteen, in order, and colour 223 is black (`0000`); the goal colours are left
  as they are [C-arrival-colours]
- So her colours on the screen are her suit's even where the goal holds others, and colour 223
  is black on the screen while its goal holds what the room gave it: a later fade heads for the
  goal (world-doors).
- checked: oracle (recorded cases S2, S3) for her place, the pad and the colours; constructed
  only for the mini-map shown: no row shows it change, since the map is already on as the
  start is made ready (§6).

## 3. The fade in

### PictureBrightens
- kind: mechanic and parameter.
- summary: each frame of the fade in counts down; when the count runs out the picture comes up
  one level and the count starts again, so each level holds `d + 1` frames.
- params: the rhythm, `d` = 1 frame; a rule from 0 to 32,768 frames.
- switch: **Parameter**, the rhythm (params).
- `FadingIn`, the fade's count `n` over 0 → the count `n − 1`, the brightness left as it is
  [R-first-frame, C-count-adjusted]
- `FadingIn`, the count 0, the level `l` under 15 → the count `d`, the level `l + 1`, `Shown`
  [R-first-step, R-step-mid, R-last-step, C-step-adjusted, C-step-from-shown]
- `FadingIn`, the count 0, the level 15 → the count `d`, the brightness left as it is, blanked
  or shown [C-full-blanked, C-full-shown]
- So from a blanked screen at level 0 the first step shows the picture at level 1, two frames
  after the arrival with `d` = 1, and level 15 is reached `2 × 15` = 30 frames after it
  (R-first-step, R-last-step).
- checked: oracle (recorded cases F1 to F4, every frame of the fade in in the cached rows);
  constructed (the adjusted rhythm, a fade from a lit or a full screen).

### PlayBegins
- kind: mechanic.
- summary: once the picture is fully up the fade in ends and she plays.
- params: none.
- switch: Fixed.
- `FadingIn`, after the fade's step, the brightness `Shown` at level 15 → the stage `Playing`,
  the fade's count 0 [R-last-step, C-full-shown]
- `FadingIn`, after the fade's step, any other brightness → the stage stays `FadingIn`
  [R-first-frame, R-first-step, R-step-mid, C-full-blanked]
- `Playing` → the stage stays `Playing`, the brightness and the count kept [C-playing]
- So a screen blanked at level 15 never comes up and the fade in never ends (C-full-blanked):
  no start leaves it that way.
- checked: oracle (recorded case F4); constructed.

---

## 4. Order that matters

- **The arrival's frame.** On the frame the preparation is done the game plays no frame of play: she
  arrives, the pad is placed and the colours go up, and none of the room's enemies or
  projectiles runs. The fade's first frame is the next one, so the pad's first run is the
  fade's first frame too (boss-ceres-ridley §7, Running).
- **A frame of the fade in** is a frame of play in every other respect, in play's order: her
  frame (held by the pad), the enemies, the projectiles, the HUD, the random number and play
  time, as on any frame of play. Then the fade takes its step (`PictureBrightens`), and then
  `PlayBegins` reads the brightness that step left. The colours the station's beacon sets on
  such a frame (boss-ceres-ridley `BeaconCycles`) are not touched by the fade: the fade changes
  brightness alone.
- **The fade's last frame** is the one whose step reaches level 15; the next frame is play, with
  nothing of the arrival left to run.

## 5. Out of scope, and who supplies it

- **The preparation's length (the start's loading, R2 and C1, with T1's lag):** which frame the
  start is ready on, its fixed waits and its lag frames included.
- **Loading the start room and placing its enemies (R2, C1):** the station's shaft, its two
  doors and the shaft's overlay; the room's colours as the goal, and the start's goal colours
  for her block (192 to 207), content of the new game's start.
- **The new file (P2, run-progress):** her energy, ammunition and items, play time from zero, the
  station's stage.
- **Her frame (samus-movement, samus-appearance):** standing still facing the screen from the
  arrival; held by the pad (boss-ceres-ridley §7.4) until it lets her go.
- **The pad and the cover (boss-ceres-ridley part 2):** placed with her place at the arrival.
- **The HUD (interface-hud):** putting it up for the new file; the mini-map following her once shown.
- **Audio (U1):** the room's music: its upload while the start is made ready, and asking for it
  on the arrival's frame (silence first, then the start room's track, through U1's queue).
- **The renderer:** showing the brightness and the colours.

## 6. Open questions

1. The start is recorded once (the any% movie's new game, a power suit, no items): a start in
   another room or with a suit is no row's; the arms state the arithmetic for any.
2. The fade in from other screens (the title, the intro's pages, the flights) is the same
   brightness fade in the records; whether those scenes share `PictureBrightens` is for their
   pieces (F2, F4) to decide.
3. The mini-map shown at the arrival has no recorded witness: the map is on in every cut of
   the row, so the HUD's round does not count it as evidence.
4. A new game that starts on Zebes (a file saved after Ceres) arrives by the gunship instead,
   a later F4 piece.
