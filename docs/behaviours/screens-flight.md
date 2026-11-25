# The gunship's flight to Ceres: behaviour spec

What the player sees after the intro's last page: the screen is dark while the flight is made
ready and its music loads; then the gunship appears, seen from the front and turned, and rushes
toward the camera past streaking stars; with a white flash the view cuts to the gunship from
behind, flying away toward the station among drifting asteroids while the flash fades; then the
picture changes to the station's view and the words "SPACE COLONY" are typed out letter by
letter; after a pause the picture fades to black and the new game begins. Written behaviour
first, from the oracle's records over time. Vectors:
[`scenarios/vectors/screens-flight.toml`](../../scenarios/vectors/screens-flight.toml).

## Header

**Scope: a piece of the cinematics (the plan's F4), the flight to Ceres.** The flight's own
State: where the scene has got to, the turning plane that shows the gunship (its angle, its zoom
and its position across the screen), the sprites (their places and motion), the white flash,
the label's step, the screen's brightness and the fade's count; and what it hands the renderer
each frame (the plane's transformation, the scroll, the flash, the sprites' places, the label's
letters), and the music requests it hands to audio (U1).

**The plane's transformation** (`PlaneTransforms`) is the rule every cinematic plane shares
(this flight, the flight to Zebes and the station's explosion use the same one): it is
specified here for its first use and is to be lifted to the platform's plane drawing (E4) when
the second scene needs it.

**What it reads.**
- **Whether a track waits to play in audio's queue of tracks** (U1, borrowed): the flight waits in the dark
  while one does.
- **The scene** (content, the flight's scene in our format): how it is made ready (the plane's
  angle, zoom and place, the rushing stars' place and speed), how it turns (the plane's angle
  and place, and the sprites placed, each with its kind, place, drift and whether it wraps),
  and the label (its steps in order, each a letter or none with its place on the label's layer
  and its hold in frames). Which label a language shows is the content's, chosen by the caller
  from the game's options.
- **The sprites' looks** (content): which picture each sprite shows, and how it animates, are the
  scene's content, drawn by the sprite drawing; this spec gives only their places.

**What it hands the others, each frame:** the plane's transformation (four values) and its
scroll, to the renderer (the plane's drawing is the platform's); the flash, to the renderer's colour blend; the sprites'
places, to the sprite drawing; each letter as its step begins, to the tile writing; the screen's
brightness, to the renderer. On the first finished frame it asks audio to load and play the
Ceres flight tune. The flight reports when it is done. The next frame starts the new game
(screens-arrival).

**What it leaves.** The colours that cycle while it plays (the gunship's engine, the station's
lights) are the colour cycles' (the plan's R4). The pictures on the plane (the gunship from the
front, then from behind), how the picture's layers mix as each stage sets them, and the
sprites' pictures are content, drawn by the platform, and not compared here. Loading the
flight's content (its colours included), its length and the lag frames are the platform's (T1).
Loading and playing the music are audio's (U1). The flight borrows audio's reply about whether
a track waits. Saving the file as the new game begins is P2's.

**What it extends.** The flight's frame report carries the music requests to audio.

**Notation.** Positions are pixels across the screen, x rightward and y downward, signed 16-bit
whole pixels with a fraction in 65,536ths (`-72.0`); a fraction written in hex is in
65,536ths (`.2000` is 1/8 px). Angles are in 256ths of a turn, written in hex (`20` is an eighth
of a turn, clockwise as the picture turns). Zoom is in 256ths, 0 to 32,767: 256 shows the plane
at its size, a larger zoom shows it smaller. Colours of the flash are 5-bit levels of red, green and blue, 0
to 31. Frames are frames of the flight; a frame here is one the game finishes (a lag frame is
the platform's and carries nothing of this domain over).

---

## 1. What the player can tell

### 1.1 Quantities

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the flight's stage | `FlightStage`: `Starting`, `WaitingForMusic`, `RushingIn`, `FlyingAway`, `Label`, `FadingOut` or `Done` | what the screen shows | every entry |
| the plane's angle | 256ths of a turn, 0 to 255 | how the gunship is turned | `PlaneTransforms`; set by `FlightIsMadeReady`, `ViewTurnsAway`, `StationComesIntoView`, `FlightEnds` |
| the plane's zoom | 256ths, 0 to 32,767 | how large the gunship looks | `GunshipRushesIn`, `GunshipFliesAway`, `PlaneTransforms` |
| the plane's position | `(x, y)`, px with a fraction in 65,536ths, each signed 16-bit whole px | where the gunship stands on the screen | `StarsRushPast`, `GunshipFliesAway`, `PlaneTransforms` |
| the sprites | a set of sprites, each of a `FlightSprite` kind (`RushingStars`, `Station`, `SmallAsteroids`, `LargeAsteroids`, `Vortex`, `DistantStars`) with its place `(x, y)`, px with a fraction in 65,536ths on each; the rushing stars with their speed, 256ths of a px per frame, signed 16-bit; every other with its drift, 65,536ths of a px per frame, signed, and whether its x wraps across 0 to 511 | where the stars, the asteroids, the vortex and the station are | `StarsRushPast`, `SpritesDrift`; output (the sprite drawing) |
| the flash | `(r, g, b)`, 0 to 31 each; black as the flight begins | how white the screen's backdrop glows | `FlashFades`; output (the renderer) |
| the label's step | the index of the label's current step, and its hold left in frames, 0 to 65,535 | when the next letter appears | `LabelIsTyped` |
| the screen's brightness | `Brightness`: a level, 0 to 15 (0 black, 15 full), and whether the screen is `Blanked` (shows nothing at any level) or `Shown` | how bright the picture is | `PictureAppears`, `PictureDarkens`, `FlightEnds`; output (the renderer) |
| the fade's count | frames, 0 to 32,768 | when the picture next darkens | `PictureDarkens` |
| the fade's rhythm | frames, `d` (`PictureDarkens`): a rule, not State | how many frames each level holds | `LabelIsTyped`, `PictureDarkens` |

### 1.2 What it remembers: state a caller can hold

The stage, the plane's angle, zoom and position, the sprites (their places, and the rushing
stars' speed or each other sprite's drift and wrap), the flash, the label's step and hold, the brightness and the fade's count: nothing
else. A caller can build any of them in any value this spec allows, read them back, carry them
and restore them; a vector's `before` builds them and its `after` reads them back.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| a track waits | `bool`: a track waits to play in audio's queue of tracks as the frame's step runs (U1; borrowed) | `PictureAppears` |
| the scene | content: made ready, the angle, the zoom (0 to 32,767), the plane's place (whole px) and the rushing stars' place and speed; the turn, the angle, the plane's place and the sprites placed (each a kind, a place, a drift and a wrap); the label, its steps in order, each a letter or none, its place (column, row) on the label's layer, and its hold, frames, 1 to 65,535 (the importer refuses a hold of 0) | `FlightIsMadeReady`, `ViewTurnsAway`, `StationComesIntoView`, `LabelIsTyped` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| the plane's transformation | four signed 16-bit values `(a, b, c, d)`, in 256ths (`PlaneTransforms`) | output (the renderer's plane) |
| the scroll | `(x, y)`, the whole pixels of the plane's position, 16-bit | output (the renderer) |
| the flash | `(r, g, b)` as it stands | output (the renderer's colour blend) |
| the sprites' places | each sprite's kind and whole-pixel place | output (the sprite drawing) |
| a letter typed | an event: the letter and its place, as its step begins | output (the tile writing) |
| the screen's brightness | `Brightness`, every frame | output (the renderer) |
| the music requests | an ordered list of `MusicRequest` events, `LoadCeresFlightTune` and `PlayCeresFlightTune`, each with an unsigned 16-bit delay in frames | output (U1) |
| the flight done | `bool`: the stage is `Done` | the caller: the next frame starts the new game |

---

## 2. In the dark, then the gunship appears

### FlightIsMadeReady
- kind: mechanic.
- summary: the flight's first frame makes it ready as the scene says: the plane shows the
  gunship from the front, turned and small, and the stars are placed to rush; the screen is
  dark and the flash black. The flight asks audio to load and play its tune.
- params: the load delay of 8 frames and the play delay of 14 frames. The angle, zoom and places
  come from the scene.
- switch: Fixed.
- `Starting`, the scene made ready with the angle `a`, the zoom `z`, the plane's place `(x, y)`
  and the rushing stars at `(p, q)` with speed `v` → the stage `WaitingForMusic`; the angle `a`,
  the zoom `z`, the plane's position `(x.0, y.0)`, fractions 0; the sprites: the rushing stars
  alone at `(p.0, q.0)` with speed `v`; the flash `(0, 0, 0)`; the brightness `Blanked` at level
  0. The frame reports `LoadCeresFlightTune` with delay 8, then `PlayCeresFlightTune` with delay
  14 [R-setup, C-setup-lit]
- Any stage other than `Starting` → an empty list of music requests [R-wait, R-appear, C-done]
- checked: oracle (recorded cases S1, S2, S3); constructed.

### PictureAppears
- kind: mechanic.
- summary: the flight waits in the dark while its music is still queued; on the first frame
  none is, the picture comes on at full brightness and the gunship starts to rush in.
- params: none.
- switch: Fixed.
- While a track waits, nothing of the flight changes and the screen stays dark (R-wait).
- `WaitingForMusic`, no track waits → the stage `RushingIn`, the brightness level 15, `Shown`
  [R-appear, C-appear-from-lit]
- So the stars take their first rush on that same frame (§7): the picture appears with them
  already moving (R-appear).
- checked: oracle (recorded cases S2, S3); constructed.

## 3. The rush toward the camera, and the turn

### GunshipRushesIn
- kind: mechanic.
- summary: the gunship grows as it rushes at the camera, its zoom falling by 16 each frame
  until it would be too large, and then the view turns to follow it away.
- params: the zoom's step 16; the zoom it turns at, under 32.
- switch: Fixed.
- `RushingIn`, the zoom `z` at least 32 → the zoom `z − 16` [R-rush-first, R-rush-mid]
- `RushingIn`, the zoom under 32 → the turn (`ViewTurnsAway`) [R-turn]
- checked: oracle (recorded cases R1, R2, R3); constructed.

### StarsRushPast
- kind: mechanic.
- summary: while the gunship rushes in, the stars streak past ever faster: their speed rises by
  half a pixel per frame each frame, and the stars and the plane both move by it, down and to
  the right alike.
- params: the speed's rise, 128 (in 256ths of a px per frame, each frame).
- switch: Fixed.
- `RushingIn`, the rushing stars placed with speed `v` → their speed `v + 128`; their place and
  the plane's position each move by `(v + 128) / 256` px on x and on y alike, fractions carried
  `(v + 128) × 256` in 65,536ths [R-appear, R-rush-first, R-rush-mid]
- In any other stage, or with no rushing stars, nothing rushes (R-wait, R-turn).
- checked: oracle (recorded cases S3, R1, R2); constructed.

### ViewTurnsAway
- kind: mechanic.
- summary: the view cuts to the gunship from behind, turned the other way, with a white flash;
  the rushing stars go, and the sprites of the scene's turn appear (in the game, the station,
  the asteroids, a vortex and distant stars).
- params: the flash `(31, 31, 31)` (the angle, the place and the sprites are the scene's content).
- switch: Fixed.
- The turn (from `GunshipRushesIn`), the scene's turn with the angle `a`, the plane's place
  `(x, y)` and its sprites → the stage `FlyingAway`; the angle `a`; the plane's whole pixels
  `(x, y)` with their fractions left; the zoom left as it is; the flash `(31, 31, 31)`; the
  rushing stars gone and each of the turn's sprites placed at its place with fractions 0, with
  its drift and wrap [R-turn, C-turn-fraction]
- checked: oracle (recorded case R3); constructed.

## 4. Flying away toward the station

### GunshipFliesAway
- kind: mechanic.
- summary: the gunship shrinks as it flies away, slowly at first and then twice as fast, and
  drifts left by an eighth of a pixel each frame; once small enough the view changes to the
  station.
- params: the zoom's steps, 16 under 3,072 and 32 under 8,192; the zoom it ends at, 8,192; the
  plane's drift, −8,192 in 65,536ths (−1/8 px) on x.
- switch: Fixed.
- `FlyingAway`, the zoom `z` under 3,072 → the zoom `z + 16`; the plane's x `− .2000` [R-away-first]
- `FlyingAway`, the zoom from 3,072 to under 8,192 → the zoom `z + 32`; the plane's x `− .2000`
  [R-away-fast]
- `FlyingAway`, the zoom at least 8,192 → the station's view (`StationComesIntoView`) [R-station]
- checked: oracle (recorded cases A1, A2, A3); constructed.

### FlashFades
- kind: mechanic.
- summary: the white flash fades one level per frame on each of its colours, to black.
- params: the step, 1 level per frame.
- switch: Fixed.
- `FlyingAway`, the flash `(r, g, b)` → `(max(r − 1, 0), max(g − 1, 0), max(b − 1, 0))`
  [R-away-first, C-flash-floor]
- checked: oracle (recorded cases A1, A2); constructed.

### SpritesDrift
- kind: mechanic.
- summary: each placed sprite drifts sideways by its own speed every frame, whatever the stage,
  those that wrap coming back in on the other side of 512 px.
- params: none (each sprite's drift and wrap are its own).
- switch: Fixed.
- A drifting sprite at x `p` (with its fraction), its drift `s`, wrapping → x `(p + s)` with the
  whole pixels taken modulo 512 (0 to 511) [R-drift, C-drift-wrap]
- A drifting sprite that does not wrap → x `p + s`, signed 16-bit whole pixels [R-drift,
  C-drift-negative]
- The sprites' y never changes but in the rush (`StarsRushPast`).
- checked: oracle (recorded case D1, and the row's cuts 116-801 with the game's scene, harness
  case H1); constructed.

## 5. The station's view and the label

### StationComesIntoView
- kind: mechanic.
- summary: the picture changes from the gunship's plane to the station's view, on which the
  label will be typed; the plane is put back square and at its size.
- params: the zoom 256; the angle 0; the plane's place `(0, 0)`.
- switch: Fixed.
- From `GunshipFliesAway` at the zoom's end → the stage `Label`; the zoom 256, the angle 0, the
  plane's position `(0.0, 0.0)` with fractions 0; the label's layer shown and emptied; the
  label's first step begins: its letter typed, its hold left its hold [R-station,
  C-label-content]
- checked: oracle (recorded case A3); constructed.

### LabelIsTyped
- kind: mechanic.
- summary: the label is typed one step at a time: each step shows its letter as it begins and
  holds for its frames; after the last step the picture starts to fade out.
- params: none (the steps and their holds are the label's content).
- switch: Fixed.
- `Label`, the hold left `h` over 1 → `h − 1` [R-label-hold]
- `Label`, the hold left 1, a next step → that step begins: its letter (if any) typed at its
  place, the hold left its hold [R-label-letter, C-label-content]
- `Label`, the hold left 1, no next step → the stage `FadingOut`, the fade's count `d`
  [R-label-end, C-label-end-adjusted]
- checked: oracle (recorded cases L1, L2, L3); constructed.

## 6. The fade out and the end

### PictureDarkens
- kind: mechanic and parameter.
- summary: the picture darkens one level every `d + 1` frames; at level 1 the next step blanks
  it.
- params: the rhythm, `d` = 1 frame; a rule from 0 to 32,768 frames.
- switch: **Parameter**, the rhythm (params).
- `FadingOut`, the count `n` over 0 → the count `n − 1`, the brightness left [R-fade-count]
- `FadingOut`, the count 0, the level `l` over 1 → the count `d`, the level `l − 1`, `Shown`
  [R-fade-step, C-fade-adjusted]
- `FadingOut`, the count 0, the level 1 → the count `d`, the level 0, `Blanked` [R-fade-last]
- `FadingOut`, the count 0, the level 0 → the count `d`, the brightness left, `Blanked` or `Shown` [C-fade-at-zero]
- checked: oracle (recorded cases O1, O2, O3); constructed.

### FlightEnds
- kind: mechanic.
- summary: once the picture is black and blanked, the flight is over: its plane and its sprites
  are cleared, and the next frame starts the new game.
- params: none.
- switch: Fixed.
- `FadingOut`, after the fade's step, the brightness `Blanked` at level 0 → the stage `Done`; the
  angle 0, the zoom 0, the plane's position `(0.0, 0.0)`, no sprites, no label, the fade's count
  0; the flash left [R-fade-last]
- `FadingOut`, after the fade's step, any other brightness → the stage stays `FadingOut`
  [R-fade-step, C-fade-at-zero]
- Once `Done`, nothing of the flight changes but the sprites' drift, and it stays done (C-done).
- checked: oracle (recorded case O3); constructed.

## 7. The plane's transformation

### PlaneTransforms
- kind: mechanic.
- summary: every frame the plane is drawn turned by its angle and scaled by its zoom, scrolled to
  its position.
- params: the sine `sin(a)`, for an angle `a` in 256ths of a turn: `trunc(256 × sin(2π a / 256))`,
  a whole number from −256 to 256 (truncated toward 0); `cos(a)` is `sin(a + 64)`. The quarter
  turns are exact: `sin(0) = 0`, `sin(64) = 256`, `sin(128) = 0`, `sin(192) = −256`. These are the
  game's own values for all 256 angles; an implementation may compute them once.
- switch: Fixed.
- Any stage, the angle `a`, the zoom `z`, the position `(x, y)` → the transformation
  `a = d = floor(cos(a) × z / 256)`, `b = floor(sin(a) × z / 256)`, `c = −b`, each within
  −32,767 to 32,767 for a zoom in its range; the scroll, the whole pixels of `(x, y)` [R-setup,
  R-rush-mid, R-turn, R-away-fast, R-station, C-done]
- So at the angle `E0` the cosine is 181 and the sine −181, and at `20` both are 181: a zoom of
  496 gives `(350, −351, 351, 350)` at `E0` (R-rush-first).
- checked: oracle (recorded cases at every cut of the row); constructed.

---

## 8. Order that matters

- **Within a frame:** first the stage's step (§2 to §6), then the stars' rush (`StarsRushPast`)
  if the stage is then `RushingIn`, then every placed sprite's drift (`SpritesDrift`), then the
  transformation and the scroll (`PlaneTransforms`) from what the frame left.
- So on the frame the picture appears the stars already rush (R-appear); on the turn the
  rushing stars are gone before they would rush, and the new sprites drift on the frame they
  are placed (R-turn); on the station's frame the transformation is the square one at 256.
- **The turn's frame** changes neither the zoom nor the flash's fade: the zoom falls no further
  and the flash is first dimmed on the next frame.
- **The station's frame** does fade the flash and drift the plane before the plane is put back,
  so the plane's position ends at `(0.0, 0.0)` on it.
- **The label's last step** ends on the frame its hold runs out; the fade's first count is the
  next frame.

## 9. Out of scope, and who supplies it

- **The flight's loading and its lag (T1):** how long the first frame takes, and the frames that
  lag while the music loads.
- **Audio (U1)** handles the requests, loads and plays the tune and supplies the borrowed reply
  about whether a track waits.
- **The plane's drawing (E4, the platform):** the picture on the plane and its swap at the turn,
  how the picture's layers mix as each stage sets them; the transformation rule (§7) is to be
  lifted there when a second scene uses it.
- **The colours that cycle (R4):** the gunship engine's flicker and the station's lights, from
  the turn on.
- **The pictures (content, drawn by the platform):** the gunship from the front and from behind
  on the plane, the sprites' pictures and their animation, the label's letters, and the
  flight's colours as its content loads them.
- **The new game (P2, screens-arrival):** saving the file and the arrival, from the frame after
  the flight is done.

## 10. Open questions

1. The flight is recorded once (the any% movie's new game): every frame of it is the same in
   every run, as no input reaches it.
2. The fade out is the mirror of the fade in other screens use, and the label is typed as the
   intro's pages are; a shared screen brightness and a shared typed script come in a later
   coherence round.
3. The label's letters for another language are other content: the steps a language skips are
   the content's choice, not a rule here.
