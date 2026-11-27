# Room quakes: decode and harness

Companion to the [room quake spec](world-quake.md). Keep these notes outside
confined exports.

## Sources

### QuietQuakePhase

Source: [Bank 86](https://patrickjohnston.org/bank/86), `$86:8427..846A`
for the earlier enemy projectile Drawing's quake displacement query. The
zero-duration branch returns zero displacement on both axes. Its positive
duration path can derive displacement from duration and content. The phase
has no read in Drawing here.

Source: [Bank A0](https://patrickjohnston.org/bank/A0), `$A0:8687..8711` for
the room update. At zero duration it skips the countdown, displacement edits,
and enemy shake requests. It still increments the phase with full-width wrap,
whether or not time is frozen (`$0A78` is tested only after the zero test).

The accepted final-history v4 source packet pins checker SHA-256
`754f50ef1458ed80b4d3ec22234895a55e05a0904e049d1d9ff386f4ada55d48`.
Its fixed physical reference is `feb96d3dd93c1bf49ba859cb622568ebd7c65787`.

### ShakeStarts

The Ceres producers, each a plain store of type and timer (the spec's start
replaces whatever is under way):
- `$A6:F773..F7A4` (the Ceres doors in the escape, `EscapeShakes`): timer 2
  with the door's 1 px type, or timer 4 with type + 6 (3 px). Types `14h`
  (doors) and `1Dh` (the door of Ridley's room, `$A6:F770`).
- `$A6:AADD..AAE6` (Ceres Ridley's getaway, at its table index `D0h`, the
  105th frame): type `23h`, timer `40h`.

A store of timer 0 is a stop: the room update and the query test the timer
first, so the stored type is never read (the spec's "a start of 0 frames is a
stop"; no Ceres producer stores 0).

Stops: `$88:8350` (the effects' set-up for a new room, `$88:82C1`, inside
world-doors' `BuildRoom`) zeroes the timer and leaves the type;
`$A6:A150..A153` (Ceres Ridley's placing) zeroes both.

Outside Ceres, for later rounds: `$A0:9CB4`, `$A0:A1E6`, `$84:D539`,
`$86:C407`, `$86:C753`, `$8F:919F`, `$8F:91C0`, `$8F:C929..C96A`,
`$8F:E59D`, `$8F:E5CB`, `$A6:B756`, `$A6:D94B` store as the Ceres
producers do; `$86:B7FE`, `$88:B373`, `$88:B38E`, `$88:DC2F`, `$88:DC75`
OR `20h` into the timer (`TSB`), which is not a replacing start: those rounds
add an arm. `$84:B803` and `$8F:C12D` also write the timer.

### ShakeCountsDown

`$A0:868F..8697` (timer 0, or time frozen: skip to the phase),
`$A0:86FF` (decrement), `$A0:870D` (phase). Called once per frame from
`$82:8BAF` in game state 8, after the room's main code and the game time, and
before the hurt timers; the station's end (game states `20h`, `21h`) runs it
too (`chain-any-0011-DF45` cuts 164-239 count down). The door passage's game
states (9 to `0Bh`) don't call it: through a passage the timer moves only by
the doors' starts and the building's stop.

### ShakenLayers

`$A0:86A1..86FD` and the table `$A0:872D` (8 bytes per type: BG1 X, BG1 Y,
BG2 X, BG2 Y). Timer bit 1 set (`r mod 4` 2 or 3) subtracts the table's
values from the scroll words `$B1`, `$B3`, `$B5`, `$B7`; clear adds them.
The scroll words are written earlier in the same frame by `$80:A3AB` from
the layer positions; BG2's are written only on an axis whose layer 2 scroll
rate (`$091B` X, `$091C` Y) is not 1 (`$80:A2F9`, `$80:A33A` return carry
set), so on a held axis the quake's edits add up frame after frame.

### ShakenActors

`$A0:8702..870A`, `$A0:8712..872C`: types `12h` and up set every active
enemy's shake timer (`$0FA2`, via the active list `$17AC`) to 2. The enemy
drawing (`$A0:947B..9494`) shakes x by ±1 on the enemy's frame counter bit 1
and decrements it: C1's.

Drawing's query: `$86:8427..846A` and the table `$86:846B` (4 bytes per
type, zero for types below `12h`); bit 1 set negates. `$86:83E9..8408` adds
the result to the projectile's screen position. So with timer bit 1 clear
(`r mod 4` 0 or 1) both the scroll and the projectile get `+size`: the same
number as the layers' offset, which moves the projectile right while the
layers' picture moves left (a scroll increase moves the picture left). Called by `$86:8390` and `$86:83B2` from `$A0:884D`, before
`$A0:8687` in the frame.

The door drawn by Ceres Ridley (`$A6:A2FF..A306`) reads `$1840 mod 4`
directly: boss-ceres-ridley's drawing, named in the spec's Read by.

## RAM mapping

| Quantity | Source | Decode |
| --- | --- | --- |
| remaining | `$7E:1840` | whole little-endian count |
| kind | `$7E:183E`, read only while `$1840` is over 0 | `code = 9 × reach + 3 × (size − 1) + direction`, reach codes 0 `Front` (BG1 only), 1 `Scenery` (BG1 and BG2), 2 `Whole` (BG1, BG2 and enemies), 3 `Back` (BG2 and enemies); direction codes 0 `Sideways`, 1 `UpAndDown`, 2 `Diagonal`. Codes `24h` and up are no kind: the room update then neither shakes nor counts down and the query gives (0, 0) (`$86:8437` branches to the zeroing at `$86:8466`), so the decode refuses such a seed and the importer such content |
| phase | `$7E:1842` | whole little-endian count; never read by the game but its own increment |
| time held | `$7E:0A78` non-zero | set by `$82:DB83` (reserve tanks, auto), `$82:DBA2` (death), `$91:CAFE`/`$91:E21A` (x-ray), `$90:E7C1` (debug); never set in the Ceres rows |
| front offset | `$B1 − ($0911 + $091D)`, `$B3 − ($0915 + $091F)` | signed 16 bits |
| back offset | X: `$B5 − ($0917 + $0921)` where `$091B` ≠ 1, else `$B5` minus its value at the last cut; Y: the same with `$B7`, `$0919`, `$0923`, `$091C` | signed 16 bits |
| actors asked | `$0FA2 + 40h × slot` = 2 for every slot the room ran (`$17AC`, cleared after the update, so at a cut: every running door's slot) | |

The kind's code order is the original's, kept here only: the spec's reach
and direction lists are in another order.

The quiet interval: corrected native callback coordinates; the game clock
opportunity precedes the room update; hurt timers and actor-list termination
follow it. The v4 bind proves that the actual guard uses the admitted
duration and that both IRQ histories preserve both quantities.

## Recorded vectors

| Vector | Row:cut | Note |
| --- | --- | --- |
| R-quiet-frame | `chain-any-0003-DFD7`, the `postRTI9232 -> postRTI9233` interval | as landed |
| R-small-starts, R-small-ends | `chain-any-0007-E06B`:2, 3 | a door's small shake: started at remaining 0, offsets −1 then +1 |
| R-strong-starts .. R-strong-ends | `chain-any-0007-E06B`:6, 7, 8, 9 | a door's strong shake: +3, −3, −3, +3 |
| R-stop-building | `chain-any-0007-E06B`:142 | the passage into room 4: `BuildRoom` zeroes 1 |
| R-start-in-passage | `chain-any-0007-E06B`:219 | room 4's door starts a shake during the fade-in; no update |
| R-getaway-starts | `chain-any-0006-E0B5`:1262 | the getaway's start, the type before it code 0 (Ridley's placing); BG1 is the getaway's picture, so the front offset is the rule's, not observed |
| R-back-ends | `chain-any-0006-E0B5`:1325 | the getaway shake's last frame |
| R-back-small-starts | `chain-any-0006-E0B5`:1606 | the door of Ridley's room in the escape |

Each recorded vector's before is the preceding cut and its after the named
cut. Both are finished cuts for the vectors of play. The two passage vectors
are taken at overrun cuts: R-stop-building's after (row 0007 cut 142) and
R-start-in-passage's (cut 219) are `O` cuts; their befores (141, 218) are
finished, and the next finished cuts (149, 220) hold the same remaining and
phase. A later round comparing passages cites finished cuts. The start in a vector is inferred where the after state
is not the countdown of the before (its producer's output, which the doors'
own vectors check).

## Findings behind the spec's choices

- **The sign follows the timer before the decrement.** The first frame of a
  2-frame shake subtracts, the second adds; a 4-frame shake adds, subtracts,
  subtracts, adds. The spec states it as `r mod 4`.
- **The start's frame shakes.** A door starts in the enemies' run, earlier in
  the frame than the room update, so the frame that starts a shake also moves
  the layers and spends one frame: a door's 2-frame shake shows at two cuts
  (`chain-any-0007-E06B` 2 and 3) and the door can start the next at the
  third.
- **The held axis.** Rooms 5 and 4 hold layer 2 still vertically
  (`$091C` = 1): `$B7` keeps the last frame's value, so the record's back y
  offset is the change since the last cut. The touch of a door at
  `chain-any-0007-E06B`:96 starts a shake and steps it once (−1); the passage
  then has no update and `BuildRoom` stops it, so `$B7` stays at −1 through
  the passage until the next room's scroll is set. The spec leaves the
  layers' places to their owners and says so.
- **The getaway's picture.** `chain-any-0006-E0B5` cuts 1158-1270: the
  getaway writes `$B1`/`$B3` for its mode 7 picture (B8 part 4c-3, E4), so
  the front offset isn't observable there.
- **The phase** counts every room update, quiet or shaking, time held or not.

## Reference model

`$TEMP/collab/quake-ref/` (local only): `model.py` (the spec), `vecrun.py`
(every vector, with the spec's citations cross-checked both ways),
`reccheck.py` (each recorded vector against its row and cut), `perframe.py`
(per frame over the cached rows), `carry.py` (State carried from a seed, the
starts borrowed from the record).

- **Vectors:** 31 of 31 agree (12 recorded, 19 constructed), and the
  admission case.
- **Per frame:** over the 1,465 finished frames of play from
  `chain-any-0006-E0B5` cut 860 to `chain-any-0011-DF45` (game states 8,
  `20h`, `21h`), remaining, kind, phase and both offsets agree on 1,465, the
  front offset masked on the getaway's 113 cuts. The actors' request is
  checked one way only: on the 804 frames that make one, a running slot reads
  2. Where none is made a slot can still read 2 from an earlier request (260
  of the 661 quiet frames, row 0006 cut 1326 among them), so "no request" is
  checked by the constructed vectors alone. A
  do-nothing model agrees on 0 (the phase always moves); the landed quiet
  model on 661, the quiet frames; the 804 shaking frames are the gain. 358 of
  them start a shake.
- **Carried from the seed:** `chain-any-0007-E06B` 0-95 and
  `chain-any-0008-E021` 0-99 hold every cut; all 95 and 99 frames of play
  shake.
- **Planted faults** (vectors failed; then the records): the sign flipped
  (19; carried, cut 1 of both rows in both offsets), the sign read after the
  countdown (9; cut 2), no countdown (18; cut 1 in remaining and kind), the
  phase held while shaking (19; cut 1 in the phase), the back reach shaking
  the front (4; 89 frames of room 6, none of rooms 5 and 4), time held
  ignored (1; no record), the scenery reach asking the actors (1; no record),
  the drawn offset's sign inverted (6, every vector with a non-zero drawn
  offset; no record), a start not replacing a shake under way (3; no record),
  a start of 0 frames ignored rather than a stop (1, C-start-zero; no
  record). The update run before the
  frame's start is an order the vectors fix by their form (start, then
  update); carried from the seeds it differs at cut 2 of both rows in
  remaining and the offsets, so the played run is its check.

## Harness note

- **Compared:** remaining and phase at every finished cut; the kind while
  remaining is over 0; the front and back offsets (the mapping above) on
  every finished cut whose frame had an ordinary update and whose last cut
  finished too; the actors' request where one is made (a running slot reads
  2; a slot can read 2 without one, from an earlier request).
- **Skipped, and why:** the kind at remaining 0 (no rule reads it, and the
  stops differ in whether they clear it); Drawing's query (no record decodes
  the projectiles' sprites; constructed vectors only); the enemies' own
  shaking after the request (C1's drawing).
- **Masks:** the front offset at `chain-any-0006-E0B5` cuts 1158-1270, its
  writer the getaway's picture (B8 part 4c-3, E4). The offsets on a passage's
  frames (no update; the layers' places are the passage's).

## Harness contract: the escape's rooms 5 and 4 played alone

With interface-hud's countdown (the same round of the escape survey), this
amendment lets `chain-any-0007-E06B` cuts 0-95 and `chain-any-0008-E021`
cuts 0-99 play from their own seeds. What the integrator wires for the quake:

1. **The seed.** The played State carries the quake's State, decoded once
   from the seed record (remaining `$1840`, kind `$183E` while remaining is
   over 0, phase `$1842`); a kind code `24h` and up is a refused seed. The
   seed's stage, now refused as "played seed in the Ceres escape", is read as
   `Escape` for `093F` = 2 or `8000h` (borrowed: held from the seed, its
   owner the open question below).
2. **The doors.** Each frame the enemies run, each Ceres door reads the
   quake's remaining as the doors before it left it (`shake_frames_left`, now
   0) and its shake, if any, starts the quake before the next door runs. This
   replaces `Fault::Escape` for a room with Ceres doors and no shaft overlay;
   the overlay (room 1 in the escape) stays refused, B8 part 7's.
3. **The update.** One ordinary update per frame of play, after her moves,
   the camera and the HUD's frame (the original runs it after the room's code
   and the game time, before the hurt timers), with time running (a seed
   that holds time is already refused, and no frame of these rows holds it).
   Its offsets and the actors' request are produced and compared, not fed to
   a picture: no loop owner draws the layers or the enemies yet.
4. **The passage.** No update on a passage's frames; the doors' runs on its
   frames of work still start shakes (row 0007 cut 219), and production
   issues the stop at the `BuildRoom` call, room loading's effects set-up
   (row 0007 cut 142, row 0008 cut 147, both overrun cuts). The doors'
   model and boss-ceres-ridley's spec are unchanged, and so are world-doors'
   model and spec: only their inputs change. `Inputs::shake_frames_left`,
   today the constant 0, gets the quake's remaining, and production's door
   run becomes a door-by-door fold that starts the quake between doors;
   world-doors gains no output.
5. **Compared fields,** at every cut whose frame of work finished:
   `quake.remaining`, `quake.phase`, `quake.kind` (while remaining is over
   0), and on frames of play `quake.front`, `quake.back` (x and y, from the
   mapping's offsets, the back's held axis by its change since the last cut)
   and `quake.actors` (every Ceres door's slot shake timer reads 2). Rows 0007
   0-95 and 0008 0-99 have no lag cut.
6. **Faults to plant** (the reference's first differences, carried from each
   seed): the sign flipped differs at cut 1 in `quake.front`; the update run
   before the doors' starts at cut 2 in `quake.remaining`; the phase held
   while shaking at cut 1 in `quake.phase`; the doors reading 0 frames left
   (today's input) starts a shake on every door run and differs at the first
   frame whose shake was under way (row 0007 cut 1 in `quake.remaining`).

**Borrowed inputs and their owners.** The station's stage (who owns it is
open: P1, survey question 1; held from the seed); the countdown's running and
drawing (interface-hud, round 2 of the survey: `Fault::Hud` until it lands);
time held (H1 and the x-ray scope: never in these rows); the random number
the doors read (game-random, already carried); the starts' choices
(boss-ceres-ridley's `EscapeShakes`, already modelled); the stop at
`BuildRoom` (world-doors and room loading).
