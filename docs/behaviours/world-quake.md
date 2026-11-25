# Room quakes

A room stays steady when no shake is under way, and shakes while one is: the
room's layers jump back and forth by a few pixels, and the actors in it shake
with them. Vectors:
[world-quake.toml](../../scenarios/vectors/world-quake.toml).

## Header

**Scope.** The shake under way in a room: its state (its kind and the frames it has
left), how a shake another part of the game starts or stops replaces it, how
each update counts it down, which layers it moves, by how much and which way,
the actors it asks to shake, and the offset Drawing gives an actor's
projectile while it lasts. The quiet update with no shake under way is part of
it. There is no old production owner of the shake to replace.

**Reads.**
- The scheduler supplies each update: an ordinary opportunity, or none (a
  door's passage has none; the frames of play, and of play's ending at the
  station's elevator, have one).
- **Time held** (borrowed): whether time is stopped this frame, as the
  reserve tanks' refill and her death (H1) and the x-ray scope (its owner)
  hold it. None of the Ceres rows holds it.
- **Starts** (borrowed): another part of the game starts a shake by naming
  its kind and its frames. In the Ceres rows these are the doors' escape
  shakes (boss-ceres-ridley's `EscapeShakes`, `Whole` or `Back`, 2 or 4
  frames) and Ceres Ridley's getaway (a later part of boss-ceres-ridley:
  `Back`, 3 px, diagonal, 64 frames). Who starts a shake, when, and of which
  kind are theirs.
- **Stops** (borrowed): another part ends the shake under way. In the Ceres
  rows the next room's building in a door's passage (world-doors' `BuildRoom`,
  the effects room loading prepares) stops it, and so does Ceres Ridley's
  placing (boss-ceres-ridley).

**Leaves to others.**
- Where each layer is before this frame's offset: the camera places the front
  layer and the room's rates place the back layer. On an axis the room holds a
  layer still, its place is the last frame's, offsets included, so the offsets
  of a shake add up there frame after frame; a whole shake returns it to where
  it was, and a shake a passage cuts short leaves it moved until the next room
  places it. The layers' places are their owners'; the shake gives only the
  offset.
- How an actor shakes once asked (the enemies' drawing: C1), and drawing the
  actors' projectiles (C1).
- The station's stage, the random number and the doors' choice of shake
  (boss-ceres-ridley's `EscapeShakes`); the drawing of the door of Ridley's
  room, which reads the frames left.
- The room quake and the player's saved play time advance independently.

**Notation.** x grows rightward, y grows downward. An offset is in whole
pixels, signed, and is added to where the view sits on a layer: an offset of
+1 on x shows the layer's picture 1 px further left. An actor's drawn offset is
added to where the projectile is drawn on screen: +1 on x draws it 1 px
further right.

## Memory and boundary

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| phase | `QuakePhase`, 0..65535 updates counted around a cycle | QuietQuakePhase, ShakeCountsDown, output |
| remaining | `QuakeDuration`, 0..65535 frames of shake left | QuietQuakePhase, ShakeCountsDown, ShakenLayers, ShakenActors, boss-ceres-ridley's `EscapeShakes` (the shake's frames left) and its drawing of the door of Ridley's room, output |
| kind | `QuakeKind`: what shakes (`QuakeReach`), how far (`QuakeSize`) and which way (`QuakeDirection`); present while remaining is over 0 | ShakenLayers, ShakenActors, output |

| Type | Values |
| --- | --- |
| `QuakeReach` | `Whole`: both layers and the actors; `Back`: the back layer and the actors; `Front`: the front layer alone; `Scenery`: both layers, not the actors |
| `QuakeSize` | 1, 2 or 3 px |
| `QuakeDirection` | `Diagonal`: x and y; `Sideways`: x alone; `UpAndDown`: y alone |

The front layer is the room's blocks; the back layer is the room's
background. The actors are the room's enemies and their projectiles.

The three quantities are complete State for the quake. Set and read all
three, carry them and restore them into fresh State. A kind is present exactly
while remaining is over 0: a start always names one, and with remaining 0 no
rule reads it, so a State with remaining 0 holds none. The State's shape keeps
the two together: a State with frames left and no kind, or a kind and no
frames left, cannot be built, so no rule meets one. The phase is a wrapping
count of the room's updates, shaking or not, and earns no
position change by itself.

What an update publishes, none of it State:

| Effect | Units | Read by |
| --- | --- | --- |
| front offset, back offset | each a pair of whole pixels (x, y), −3..3 | output: added to where the view sits on each layer as its owner placed it |
| actors asked to shake | yes or no: every actor the room runs this frame is asked to shake for its next 2 frames | the enemies' drawing (C1) |

Drawing's query, also not State: **the drawn offset**, a pair of whole pixels
(x, y), −3..3, added to where each actor's projectile is drawn. Read by the
actors' projectiles' drawing (C1).

A requested update without an established opportunity is Unsupported before
changing anything or publishing an effect. A start names a kind and 1..65535
frames; a start of 0 frames is a stop.

## Rules

### QuietQuakePhase

- kind: mechanic
- summary: the quake phase advances while the room stays steady.
- params: none
- switch: fixed
- Ordinary opportunity with remaining zero, whether time runs or is held →
  set phase to `(phase + 1) mod 65536`, keep remaining zero, and contribute
  no quake displacement or actor shake request [R-quiet-frame, C-other-phase,
  C-mid-cycle, C-wrap, C-after-wrap, C-restored-next, C-quiet-time-held].
- checked: oracle (R-quiet-frame), with independent constructed starts and wrap.

### ShakeStarts

- kind: mechanic
- summary: a shake another part of the game starts replaces the one under
  way, and a stop ends it.
- params: none
- switch: fixed
- A start of kind `k` for `n` frames, `n` 1 or more, in play or in a passage, whatever is
  under way → kind `k`, remaining `n`, phase unchanged; the frame's update, if
  it has one, then runs on the new shake [R-small-starts, R-strong-starts,
  R-getaway-starts, R-back-small-starts, R-start-in-passage,
  C-start-replaces, C-start-long].
- A stop, or a start of 0 frames → remaining 0 and no kind, phase unchanged
  [R-stop-building, C-stop-mid-shake, C-start-zero].
- checked: oracle (the frames the shake started or stopped in the Ceres rows);
  constructed.

### ShakeCountsDown

- kind: mechanic
- summary: each update of a shaking room spends one of its frames.
- params: none
- switch: fixed
- Ordinary opportunity, remaining `r` over 0, time running → remaining
  `r − 1`, kind kept while `r − 1` is over 0, phase `(phase + 1) mod 65536`
  [R-small-starts, R-small-ends, R-strong-starts, R-strong-second,
  R-strong-third, R-strong-ends, R-getaway-starts, R-back-ends,
  R-back-small-starts, C-sideways-front, C-up-and-down-scenery,
  C-whole-sideways, C-back-up-and-down, C-drawn-strong, C-start-replaces,
  C-start-long, C-restored-long, C-continued].
- Ordinary opportunity, remaining over 0, time held → remaining and kind
  kept, phase `(phase + 1) mod 65536`, no offset and no actor asked to shake
  [C-held-shaking].
- No opportunity → phase, remaining and kind kept, nothing published
  [C-absent-shaking, R-start-in-passage, R-stop-building].
- checked: oracle (every frame of play the Ceres rows hold); constructed.

### ShakenLayers

- kind: mechanic
- summary: while a shake lasts, the layers it reaches jump back and forth
  along its direction by its size, two frames each way.
- params: none
- switch: fixed

| Reach | Front layer | Back layer |
| --- | --- | --- |
| `Whole` | shaken | shaken |
| `Scenery` | shaken | shaken |
| `Front` | shaken | still |
| `Back` | still | shaken |

Cells: shaken, still.

| Direction | x | y |
| --- | --- | --- |
| `Diagonal` | moved | moved |
| `Sideways` | moved | 0 |
| `UpAndDown` | 0 | moved |

Cells: moved, 0.

- Ordinary opportunity, remaining `r` over 0 as the update begins, time
  running, `r mod 4` 0 or 1 → each shaken layer's offset is `+size` on each
  moved axis, 0 on the other; a still layer's offset is (0, 0)
  [R-small-ends, R-strong-starts, R-strong-ends, R-getaway-starts,
  R-back-ends, C-sideways-front, C-back-up-and-down, C-continued].
- The same with `r mod 4` 2 or 3 → `−size` on each moved axis
  [R-small-starts, R-strong-second, R-strong-third, R-back-small-starts,
  C-up-and-down-scenery, C-whole-sideways, C-start-replaces, C-start-long,
  C-restored-long, C-drawn-strong].
- checked: oracle (the layers' places in every frame of play the Ceres rows
  hold, but the front layer's during the getaway's picture); constructed for
  the reaches, sizes and directions no record shows.

### ShakenActors

- kind: mechanic
- summary: a shake that reaches the actors asks each of them to shake, and
  moves their projectiles as they are drawn, the opposite way to the layers'
  picture.
- params: none
- switch: fixed
- Ordinary opportunity, remaining over 0, time running, reach `Whole` or
  `Back` → every actor the room runs this frame is asked to shake for its next
  2 frames [R-small-starts, R-small-ends, R-strong-starts, R-strong-second,
  R-strong-third, R-strong-ends, R-getaway-starts, R-back-ends,
  R-back-small-starts, C-whole-sideways, C-back-up-and-down, C-drawn-strong,
  C-start-replaces, C-continued].
- A shake of reach `Front` or `Scenery` asks no actor to shake
  [C-sideways-front, C-up-and-down-scenery, C-start-long].
- Drawing's query with remaining `r` over 0, time running, reach `Whole` or
  `Back` → the drawn offset is `+size` on each moved axis when `r mod 4` is 0
  or 1, `−size` when it is 2 or 3, and 0 on the other axis: the same number as
  the layers' offset, which moves the projectile the opposite way to the
  layers' picture (Notation)
  [C-whole-sideways, C-back-up-and-down, C-start-replaces, C-absent-shaking,
  C-continued, C-drawn-strong].
- Drawing's query with remaining 0, time held, or reach `Front` or `Scenery`
  → the drawn offset is (0, 0) [C-held-shaking, C-sideways-front,
  C-up-and-down-scenery, C-quiet-time-held].
- checked: oracle for the actors asked (every frame of play the Ceres rows
  hold); constructed for the drawn offset, which no record decodes.

## Order that matters

Within a frame, the starts and stops of the Ceres rows come first, each as
its owner runs (a start made by a room's own code, which no Ceres row has,
comes after Drawing's query and before the update; the round that brings such
a room places it): a
part that reads the frames left reads them as the starts before it left them,
so of two doors in a room the first to run starts a shake and the second
finds one under way. Drawing's query comes next and reads the shake those
starts left. The ordinary update comes last: the offsets and the actors'
request are made from the remaining as the update begins, and the countdown
follows. So the frame that starts a shake also moves the layers and spends its
first frame. A passage skips the update, but its starts and stops still change
the shake.

The ordinary opportunity advances the phase once before the completed room
State for that frame is observed. A continuation uses the full previous
result. A fresh restoration of that result gives the same next phase,
remaining, kind and effects. A steady picture alone cannot distinguish a
missed or duplicated phase advance.
