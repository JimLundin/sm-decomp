# Status: how to measure it

Acceptance is indexed production replay (MODEL §1 decisions 32 and 33). The committed
[index](../scenarios/replay/index.toml), [baseline](../scenarios/replay/baseline.toml) and
[transition ledger](../scenarios/replay/transition.toml) define witnesses, exact obligations and
reviewed replacement/loss dispositions. What counts as progress toward the headline milestone is
[MODEL §6](architecture/MODEL.md#6-how-progress-is-measured). `tools report` prints the generated
numbers, and the README shows the progress; [BUILDING](BUILDING.md#the-readme-and-the-report)
describes both. The record cache the cases read is made as [ORACLE §6](ORACLE.md#6-commands)
shows.

## Running indexed replay

What the index and baseline oblige is
[ARCHITECTURE §11](architecture/ARCHITECTURE.md#11-the-engine--oracle-comparison-contract):
behaviour obligations require completed comparisons, and boundary obligations the exact named
Unsupported result.

```sh
TOOLS=target/verify/tools
INDEX=scenarios/replay/index.toml
$TOOLS replay list "$INDEX" --rule samus-movement/TravelledDistances
$TOOLS replay run "$INDEX" --case movement/D-inherited-cut \
  --cache "$CACHE" --bundle "$BUNDLE" --rom "$ROM" --native "$NATIVE"
$TOOLS replay check "$INDEX" --all --baseline scenarios/replay/baseline.toml \
  --cache "$CACHE" --bundle "$BUNDLE" --rom "$ROM" --native "$NATIVE"
```

`run` reports selected outcomes; `check` compares them with the reviewed baseline. `--write`
writes safe TOML receipts containing IDs, counts, field names and hashes, not ROM-derived values.
Local case selection is useful diagnosis; required acceptance runs the complete index, with no
automatic reseed after a failure.

## Frames played alone

The rung measure is the number of cuts a room plays alone from its seed.
[`verify::replay::played`](../crates/verify/src/replay/played.rs) admits the whole-frame
owner [`game::production::played`](../crates/game/src/production/played.rs) once from the seed
record. Each frame of play it steps her clip's look ahead, `Samus::step`, the animation update,
the camera and the doors, once each, in any pose with imported content. Each frame of play
runs the room's enemies before her moves, her health (samus-health's whole frame, reading the
hit of Ridley's tail) between her moves and her reaction, and ends with the HUD's frame of play. The HUD reads her position after
her moves, the energy Health leaves and its held neighbours. While a passage runs, the HUD takes
its step outside play, turned on when the doors turn the mini-map on. The run also compares
Health's State (energy, pending loss, warning, protection) and the HUD's State (the mini-map's
window, the status bar, the explored cells, the values last shown, the selection and the
mini-map's off), each part by name: Health at every cut, the HUD at every cut but the lag cuts
of the contents' fetch, from its first frame to the slide's end, which can find the fetch
part-done (Ridley's placing blanks the mini-map within it, below). The game's count of finished frames, which the
HUD reads as a frame begins (her cell's blink), is the played owner's too: a finished frame
counts it, and each frame on which the doors restart it (world-doors' `FinishedFrames`) counts
from 0. It is compared as `frame-count` at every cut. A frame whose moves
reach a door to a room plays out whole; from the next, while the passage runs, the doors step
alone and move her position, the view, where the camera last saw her and the colours. As each
of the passage's calls is done, the room's loading
([`game::production::room_loading`](../crates/game/src/production/room_loading.rs)) loads its
part of the next room from the imported world, tilesets, species and sprite palettes, in the
version her progress chooses. Building the room gives what play reads of her room from the
version's imported level and scroll areas: what her moves meet (its blocks, with those its
scroll PLMs and extensions place, and no actor's box), its doors to other rooms by cell, and
the camera's room (its screens, size and scrollers; the scroll areas past a preset room's
screens stay as they were), and turns the camera's lead back to ahead, as the original's build
does. The blocks PLMs place are set as the room is built, where the original sets them as its
contents are fetched; nothing reads her room between, and a version placing PLMs whose blocks
no owner places is refused at the build. Fetching the tiles heads the colours for the tileset's. Fetching
the contents heads the sprite rows for her suit's, the species' the version loads, the common
sprites' and the flashing enemies' palettes, then Ridley's room's colours where it places him
and the Ceres doors' colour set for the station's stage in the palette each door's placing names
(boss-ceres-ridley's `TheDoorsColours`), and refuses a version placing any other actor, whose
box no owner here places. The
room's setup and the door's own code are not run: the world names each by what it does
(`RoomSetup`, `DoorRule`), and fetching the contents refuses each that no owner models as a
named Unsupported (`LoadFault::RoomSetup`, `LoadFault::DoorRule`). It lets through only what
changes nothing play reads: no code; the station's haze (`RoomSetup::CeresHaze`), the setup
of the rooms on 0001's and 0003's paths, which only blends a backdrop colour over the screen;
the haze of Ridley's room (`RoomSetup::CeresRidleyHaze`), which also sets the tile bases and
the screen mode its floor is drawn with, which the loop doesn't draw; and the way out of the station's elevator room (`DoorRule::FromCeresElevatorShaft`), which
only turns the screen out of its rotating view: it sets the mode 7 flag and the PPU's fake mode,
which no loop owner produces and the run does not compare. The way into it is still refused. Fetching
the contents also shows her shots' palette for the beams she has switched on at once. Each later
cut's only new input is the movie's pad from the preceding vblank. The controller starts as the
seed leaves it, with the game's last read and the buttons that went down at it, so a button
held through the seed is not pressed again on the first cut. Her kept action, which the
record keeps as a movement type, is decoded as the one action her own poses (the map's) of
that type share, and refused where they share none; poses outside her map are not counted, a
limit of the decode, though the kept action is carried and no frame these runs play reads it. The run compares those
owners with the record at every NMI return and stops at the first difference or the first
unsupported pose or content. Room facts with no selected producer yet are read once from the
seed and held; `SEED_HELD` names them, and each is a missing owner. Her choices of her pose,
whether her latest and the one before it left her in a spin or a wall jump, are read from the
record's pose history (the old and old-old poses' movement types, which the game shifts only on
a frame that chooses) and compared at every cut as `movement.choices`. The comparison is at frame
cuts; it does not compare intra-frame bus timing. While she follows the pad the game keeps no way for her run of its own, so her run's way
is compared only while she carries on, and otherwise through her position.

The escape countdown (interface-hud §5.1) is the HUD's State, carried with it. Its frame of
play assumes main gameplay runs the countdown, a constant the runner gives rather than the
scene's choice read from the game (every main-gameplay frame of the escape rows has the running
handler); its start; and the count of finished frames as the last frame left it, the played
State's sixteen-bit count. The start is
borrowed from the records until the station's boss raises it (`BORROWED`): the timer set between
two records, at the time the second holds. A seed's countdown is decoded from the timer's words,
a start not yet taken refused. The doors read whether it has begun from the HUD, to draw it and
keep its colours lit through a passage. The run compares it at every cut it compares the HUD
(`hud.countdown.begun`, `.time` in hundredths, `.age`, and `.x` and `.y` in 1/256ths of a
pixel) and, on each frame of play, the expiry answer against the game state turning to the
time-up's (`hud.countdown.expiry`): a run-out reported early is caught, but a missed one stops
as Unsupported at the time-up's game state, which the runner refuses before it compares; no row
of the movie runs out. A seed in the escape is admitted, but not one in the
rotating shaft. `chain-any-0007-E06B` holds a begun countdown through its hold, the slide from
cut 72, the passage and room 4; there a count of finished frames one ahead differs at cut 2 in
`hud.countdown.time`, and the doors reading the countdown dormant through a passage at cut 97 in
`doors.colours.goal`. The doors reading it begun through a passage differ at
`chain-any-0003-DFD7` cut 183 in `doors.colours.goal`, a start borrowed on every frame at cut 1
in `hud.countdown.begun`, and the expiry answer inverted at cut 1 in `hud.countdown.expiry`.

The room's Ceres doors (boss-ceres-ridley's `CeresDoor`) play in the loop too. Their State is
decoded once from the seed record, each door in the room's order of its enemies, and carried
with nothing fed back. Each frame of play runs every door once through `CeresDoor::run`, before
her moves, reading where she was at the end of the last frame and the station's stage. Her
motion meets the boxes of the doors that were solid as the frame began. The original meets the
boxes in slot order and the held boxes keep no slots, so the doors' boxes come last and the
runner refuses a seed holding an actor box in a slot after a door's. In a passage the doors run on the frames of work the passage says the enemies run.
Fetching the next room's contents lets the last room's doors go and places the next room's
from the imported world: the version her progress chooses lists the Ceres doors its enemy
population places. The run compares how many doors the room holds, and each door's phase, wait, figure,
shown and solid, by its place (`ceres.0.phase`), at every cut whose frame of work finished,
since a lag cut can find the room's enemies part-built, with whether Ridley draws it
(`ceres.0.drawn-with-ridley`). In the escape each door reads the room quake's frames of shake
left as the doors before it left them, and the shake it starts, if any, starts the quake before
the next door runs (below). No record checks that door-by-door order: the doors in these rooms
start the same shake from the same random number, so no played row tells it from every door
reading the shake left as the frame began. Only a unit test checks the fold. A door's
request for its opening sound is produced but not compared: it belongs to the audio boundary
(MODEL decision 37), whose owner is not in the loop yet.

The Ceres shaft as she arrives (boss-ceres-ridley part 2) plays in room 1. The elevator's pad
and the cover below it (`PadAndCover`) are decoded once from the seed record, from their enemy
projectile slots, and carried. A seed whose lock on her disagrees with the pad's presence is
refused. They run once each frame of play, and on each frame of work a passage runs the
projectiles, after her own frame. While the pad holds her, her moves and her animation don't
run, and its run sets her height; the camera then reads her where the pad leaves her, with
nothing placing her, and the HUD's mini-map where her own frame left her. She is let go after
her frame on the pad's last run. The run compares the pad (its wait, position and look) and the
cover's position at every cut (`shaft.pad`, `shaft.cover`); their figures are drawing and are
not compared. The shaft's overlay is the actor of the doors' species the seed record or the
next room's contents place with first parameter 2. On each frame the room's enemies run, it
steps the lights (`Lights::step`) with the count of finished frames, the doors' fade's steps
taken and the station's stage; the beacon's step sets the six screen colours its imported
colour set names, compared as `doors.colours.screen`. The platform's look is the planes', which
the loop doesn't draw: it is produced and not compared. As the colours come up the lights run before
the fade's step, and read its steps taken without it; no record yet shows that order or that
case in a played run, since room 1 is not entered by a door before the escape, so unit tests
alone check them. In the escape a later piece owns the
overlay, so a room holding it refuses the frame (`Fault::Escape`).

The room's quake (world-quake) is the played State's, decoded once from the seed record (the
frames of shake left, the kind while some are left, a kind code past the 36 refused, and the
phase) and carried. The Ceres doors start its shakes as the enemies run, a door's shake moving
what its kind's shakes move by its strength, diagonally, for its length. Building the next room
in a passage stops the shake (room loading's effects set-up, inside the building), and so does
Ridley's placing. Each frame of play, and of its fade in, updates it once, last, with time
running (a seed that holds time is refused); a passage's frames have no update. Its offsets
and the actors' request are produced and compared, not drawn: no loop owner draws the layers or
the enemies, and Drawing's query for the actors' projectiles is not asked. The run compares the
frames left (`quake.remaining`), the phase (`quake.phase`) and, while frames are left, the kind
(`quake.kind`) at every cut whose frame of work finished. On a frame the record shows the update
ran (the last cut in main gameplay or its fade in, both cuts finished) it compares each layer's
offset, the scroll past where its owner placed the layer (`quake.front`, `quake.back`, x and
y); on an axis whose back layer the room holds still the game writes no place, so there it is
the scroll's change since the last cut. The front offset is not compared at
`chain-any-0006-E0B5` cuts 1,158 to 1,270, where the getaway draws its picture through the
front layer's scroll. Where the record shows a shake reaching the actors under way as the update
began, it compares that every Ceres door's slot was asked to shake (`quake.actors`); a slot reads
asked for a frame after its request too, so where none is made none is compared, and only the
constructed vectors check that no request is made. A shake a passage cuts short leaves the held
axis moved until the next room places it; that place is the layer's owner's. A sign flipped
differs at `chain-any-0007-E06B` cut 1 in `quake.front`, the update run before the doors' starts
at cut 2 in `quake.remaining`, the phase held while shaking at cut 1 in `quake.phase`, and the
doors reading no shake left at cut 1 in `quake.remaining`. The count of finished frames
in sixteen bits, which nothing restarts, is the played owner's: carried from the seed's value,
it counts each frame the doors report finished or restarting, so the borrowed lag decides it.
It is compared as `finished-frames` at every cut, and her parity is its parity as the frame
begins.

Lag is borrowed until the platform's costs are modelled (MODEL decision 35); `BORROWED` names
it. The runner reads one fact per cut from the record: whether the NMI found the frame's work
finished. A lag NMI finds it part-done, so the game commits no new picture (the screen keeps the
previous snapshot) and the work runs on into the next frame. In a passage this is the doors'
"work done" input: the phase holds, and only the placing and the slide move. The contents'
fetch is the exception, borrowed as done on its first frame. Its work waits on its own uploads,
so every NMI through it is a lag NMI, and none says when its colours were set. The record shows
them set within the slide and the visor lit as the slide ends, and only a fetch done before
the slide's end does that. Its first frame is the earliest such frame. The colours are
compared only at cuts whose work finished, since a lag cut finds a fade's step part-done. A
lag cut in play is still a named stop: the M0 row has none, so no record yet shows how it plays.

More facts are borrowed, the audio ones and the passage's random number only through a passage.
Audio's answer to the passage, whether a track waits in
the queue of tracks as the frame of work begins, is read from the record's music queue as the
main loop's start steps it once (world-doors' decode); the passage into Ridley's room waits on it
for 68 frames as his room's music loads; the earlier passages of these rows play as ready.
Whether every sound request has been sent as the frame of work begins is read from the record at
the preceding cut, since the original sends the sounds after the frame's work: each of the three
sound libraries' next index at its start. The escape's passage out of room 4 waits on it, so
`chain-any-0008-E021` holds all 263 cuts and room 5 played on holds 659, to her pose at cut 660;
the sounds always taken as sent differ at `chain-any-0008-E021` cut 105 in
`doors.colours.screen`. Every
frame draws the game's random number once (game-random's `AdvanceRandom`) as its work begins,
whenever the last NMI found the last frame's work finished. In play the played State carries the
number from the seed and every draw is the loop's; it is compared as `random` at every finished
cut in play. Through a passage the number before the frame's draw is borrowed, the record's at
the preceding cut: no owner here models a passage's draws, whose own frames draw it on some
frames and not others, so a number carried through one falls out of step at the first passage
(0002's cut 238). The loop carries on from the passage's last borrowed draw.

Ridley's room as she enters (boss-ceres-ridley part 3) plays from room 5's passage into it.
Fetching the contents of a version whose population places Ceres Ridley places him first
(`CeresRidley::place`, his own place whatever the placement says, unless progress counts the
station's boss beaten), then the door of his room (`RidleyDoor`, the first parameter 3). The
importer names the door actors' first parameters 3, 5 and 6 as the door of his room and the
getaway's two walls; the walls are placed by his getaway, a later piece, so no population of
these rows places them. His placing hands on a boss encountered, which the HUD meets outside a
frame of play (interface-hud's `BossBlanksTheMap`, Ridley in Ceres: no screens explored), its
one producer here; the stage, which the played State carries from the seed and his placing sets
(calm); his room's colours, which the contents' fetch writes as the fade's goal (the baby's and
the door's own palette from his room's imported colours, the doors' palette black but for its
first colour, then the door's calm set in its own palette); the music's stop, which is produced
and not compared (audio's boundary is not in the loop); and the shake's stop, which stops the
room's quake. Each frame the room's enemies run, his run comes first, reading whether a passage is
under way and the frame's random number; the doors run after him reading whether he is beaten
(progress's, held: no run here beats him). The baby's colour sets go to the screen's sprite row 3
at once, compared as `doors.colours.screen`, and its cry is produced and not compared. His
drawing, his look, the baby's and the doors he draws closed, is produced for drawing and not
compared. The run compares whether he is placed and his phase, wait, look, position and the
baby's routine, runs and position (`ridley.phase`, `ridley.baby.routine`) at every cut whose
frame of work finished. His waking (boss-ceres-ridley §13) runs in the played frame. His content,
the eye's 64 steps and the body's 16 colour sets, is imported and checked once. The eye's set
goes to the screen at once on colours 12 to 14 of sprite row 7 (the body's would go to colours 1
to 11 of rows 7 and 1), compared as `doors.colours.screen`; the run also compares his count,
runs, the roar's figure and its runs, his look, and whether he is tangible, moving and roaring,
read from his slot's properties and his words. What is played is his eye's fade, all 64 steps, and
his body's fade, and his turning tangible at cut 609. From his body's fade on his tail rests
behind him (part 4b): the run compares its curl and its seven segments' places
(`ridley.tail.curl`, `ridley.tail.0` to `ridley.tail.6`), read from his tail's words while it
rests and none before. Tangible, his tail hits her at cuts 609 and 705: his run reads her position,
the body of the pose she started the frame in and her protection, all as the frame starts, and
reports the hit, which Health takes (worth 15 energy, his tail's worth imported from
the player's ROM as his content, and 96 frames of protection) and her movement's reaction takes
in the same frame (her push, `Recoiling`, then her backward leap).
The hit itself is compared where it lands: `health.*`, `movement.knockback` and her hurt colours
in `doors.colours.screen`. His wait, his roar, moving and roaring, and the refusal of a run
that starts in an attack or as he gives up (`Fault::RidleyAttacks`, `Fault::RidleyGivesUp`, later
parts') are reached by no played cut: they hold by the vectors and the production tests only.
His take-off and flight (part 4c-1a) play from cut 867, and the played cuts, which stop at 896,
reach only his take-off and his rise: he lifts off at cut 879 and rises to 896. His slowing (from
cut 899), his hover and his pull to his point (from 916) are reached by no played cut, nor are
his reading of her energy as the frame starts it, his hits taken held at 0 (an input no owner
produces yet; the game has 11, under the 50 and 100 his rules read) and the shake he starts at
the room's left side, which goes to the room's quake before the doors run: they hold by the
vectors and the production tests only. The bound he meets on a run (`7E783E` in the harness
note) is a run output and isn't compared. The fight's music and the roar are produced and not
compared. Her going backwards, which the camera reads, is her skid, her moonwalk, or exactly
one of a hit's push and her backward leap (world-camera's `LeadingHer`). Room 6's moonwalk at its
left edge is the first played frame that tells the moonwalk apart, at cut 352; her leap once the
push is over, at cut 621, tells the push and the leap apart. No played cut has the push and the
leap together where the camera tells exactly one of them from either of them; the production
test of the camera's going backwards holds that.

```sh
$TOOLS replay played chain-any-0003-DFD7 0 344 \
  --cache "$CACHE" --rom "$ROM" --lock scenarios/locks/lsnes.toml
```

It prints one `PLAYED_ALONE` line: the cuts held and where the run stopped, by cut and field or
dependency name only. An end past the row's last cut plays on through the rows that continue it,
the same capture's next seeds, without a new seed: each row's cut 0 is the video frame after
the last cut of the row before, and the run's cuts count on through it. The seed room's
geometry, door cells and camera room are held from the seed; each room a passage brings her
into is built from the imported world, with its map room and its Ceres doors. Played alone,
`chain-any-0001-DF45` holds all 411 cuts from her arrival: the pad lowers her from cut 29 and
lets her go at cut 101, and she leaves by room 1's door. `chain-any-0002-DF8D` alone holds all
281 cuts, `chain-any-0004-E021` all 263, and `chain-any-0006-E0B5`, Ridley's room, 896 from its
seed. Played back to back, the Ceres rows from `chain-any-0001-DF45` hold 2,520 cuts, through
rooms 2 to 5 and on to cut 896 of `chain-any-0006-E0B5`, and from `chain-any-0003-DFD7` 1,826;
from her arrival (`any-ceres-arrival`, below) they hold 2,743, to movie frame 11,058. In the
escape, `chain-any-0007-E06B`, room 5, alone holds all 258 cuts, and played on through room 4
and its passage out holds 659, stopping at cut 660 in `movement.pose`; `chain-any-0008-E021`,
room 4, alone holds all 263; `chain-any-0009-DFD7`, room 3, holds 136 and stops at cut 137 in
`movement.pose`; and `chain-any-0010-DF8D`, room 2, holds 65 and stops at cut 66 in
`movement.pose`. Her frame
is two steps, her moves and then her reaction to the frame's hit, with the station and Health
between; her knockback (samus-movement §12) and her hurt colours (samus-appearance) run in it,
and her crouched turn, begun at room 6's cut 535, ends at the record's frame. Both hits of his
tail that the runs reach play. At room 6's cut 609 she loses 15 energy, takes `Recoiling` with her
hurt colours, is pushed left and leaps backward, and the push ends at cut 614; at cut 705 she
loses 15 more, the push ends at cut 710, and she leaps again. The camera reads her as going
backwards in exactly one of the push and the leap (world-camera §1.3), so in the leap it leads
the way she travels once the push is over. World-camera's harness cases for the leap and the
edges' subpixel (R36 to R40, at room 6's cuts 229, 615, 621 and 623 and room 3's cut 136) are
compared by these played runs and are not wired as indexed cases. At cut 728 she lets the second
leap go, holding forward without jump: she follows the pad and stops dashing, and the dash she
keeps is gone the next frame (samus-movement's backward leap let go). He takes off at cut 867 and
flies; each run stops at room 6's cut 897 in `movement.pose`, where his tail hits her in flight
(boss-ceres-ridley part 4c-1b), which isn't played yet.
CI runs it for the M0 row from its seed (cut 0) and from cut 6 when dispatched with the target
`played`. It is a diagnostic, not yet an indexed obligation.

Sixty-two planted faults in the fault matrix, those of the responsibility `played`, stop
that run, or the run of another Ceres row from its seed, at a recorded cut. For example, a pad one frame stale fails the runner's pad alignment
check at cut 2, and a camera State held rather than carried differs at cut 1 in
`camera.view.x.fraction`. Every button held at the seed taken as newly pressed differs at
cut 17 in `movement.jump-held`, played from the M0 row's cut 16. A frame on which the doors
restart the frame count counted on instead differs at cut 238 in `frame-count`. The last room's
geometry, doors and camera room kept after a passage builds the next differ, played back to back
from the M0 row's seed, at cut 348 (the next row's cut 3) in `movement.position`. Of the Ceres doors' faults,
their State held rather than carried differs at cut 1 in `ceres.0.wait`, and the doors reading her position after her moves rather
than before differs at cut 7 in `ceres.0.phase`. Of her frame's, her reaction run on a copy, so
she keeps the State her moves started from, differs at cut 1 in `movement.pose`, and the energy
the frame before left read one above the frame's, a fall that starts her hurt colours, at cut 1
in `appearance.hurt-count`. In room 1, the pad's carry dropped from her
height differs at cut 29 in `movement.position`; the beacon reading the count as the frame
leaves it differs at cut 7, and reading no fade's steps at cut 257, in `doors.colours.screen`;
and a frame whose work overran counted as finished differs at cut 253 in `finished-frames`. The
beacon reading the HUD's count instead holds all 411 cuts: it reads bits 3 to 5 of the count,
which the two counts share until a passage restarts the HUD's, and no frame of the overlay
follows a restart before the escape. In Ridley's room, played from room 5's seed
(`chain-any-0005-E06B`), the slam's hold a frame longer differs at cut 288 in `ceres.0.wait`;
the passage into his room taken as finding the music always ready differs at cut 216 in
`ceres.0.phase`; Ridley read as no passage under way, so waiting from the passage's fade in, at cut 282 in
`ridley.phase`; his room's colours a colour out of place at cut 211, the passage's first cut
whose work finished after the fetch, in `doors.colours.goal`; and the HUD meeting Ridley in
Norfair rather than in Ceres at cut 211 in `hud.explored`. Played from room 6's seed, the baby's
colour sets not shown differ at cut 151 in `doors.colours.screen` (its first set, at 145, holds
the colours the row holds), the random number not drawn as the frame's work begins at cut 1
in `random`, the camera reading her as going backwards by the push alone, the leap left out,
at cut 621 in `camera.view.x.fraction`, and jump fed to her as held while she leaps dashing,
which keeps jump held through her let-go, so at cut 728 she takes the striding jump, in `movement.pose`.

The M0 row never shows a door blocking her: a fault that drops the doors' boxes from what her
motion meets holds all 344 cuts, and rows 0004 and 0005 as far as they play; the door of
Ridley's room locks behind her, out of her way. Only the unit
test of a door that opens in the frame still blocking her catches it. Retiring the indexed
case `movement-ceres/carried-position-proximity` lost its gating obligation, which the
diagnostic played run compares but does not gate. That loss is accepted until the played run
is indexed.

### The intro's opening and page 1

`any-intro-0568`, the intro's opening and page 1, plays alone from its seed, cut 0, the frame end
the options hand on to the intro at: the run admits the State the hand-on gives (the stage
`Begins`, the screen black and blanked, the fade's count 0, the rest at rest) and reads nothing
else of the seed but the options' language. Page 1's script, the eyes' and the caret's are
imported from the player's ROM. Each cut borrows whether its frame of work finished and, at
finished cuts, whether a track waited in audio's queue as the frame began (the flight's
producer). It holds all 1,650 cuts to page 1's wait: at each finished cut the stage, the stage's
wait while the opening holds, the dissolve's count from page 1's making ready on, the height,
each script's name, step, hold and repeat count, the caret's place and whether it shows, the
glows, the click turn, the clicks against the sound queue's index, the music's requests against
the music queue's, the brightness, the fade's count in the slow fades and the equipment
returned; at the lag cuts (1-53, 71-104, 412-435, 739-773) the brightness alone. Masked, with
their writers: VRAM (E2), CGRAM, the intro's palettes as its setup loads them (content through
T1), OAM (E1), the PPU mirrors (E3, E4), and what the intro's first frame sets outside it (her
State and drawing, the bindings, the area, the room's size, the door, layer 1's place, the line
program and the colour effects): their owners', none in the loop before the battle on Zebes.
Limits: the equipment the model lends is reported but compared by neither run, and applying it
has no owner in the loop; the tunes' start delays (8 and 14 frames) are checked by neither run,
because this run counts the music's requests, the power-on run's queue code is the same for
every tune played, and whether a track waits is read from the record, so a wrong delay never
feeds back. The press while page 1 waits starts the battle, part (b), which the
intro's adapter refuses (`BattleBegins`).

Planted faults: the adapter ignoring audio's reply (none waits) differs
at cut 55 in `intro.stage` (from power-on, cut 623); page 1's end imported as page 5's wait
differs at cut 1,650 in `intro.stage` (2,218); the pages' tune coded `FF37h` in verify's played
runner differs from power-on at cut 1,290 in `front-end.sounds` (the intro row's own run counts
the requests, so it holds): the first two fail on both runs, the third from power-on alone.

### The intro's last pages

`any-intro-6012`, the intro's last pages, plays alone from cut 1, inside page 4's turn: the
dissolve to the scene, the only stage a run is admitted at. The intro's State is admitted from
the seed's intro work RAM (the stage, the dissolve's count, the colours' levels and steps and the
colours shown, the picture's height, the baby, the page, the eyes, the caret, the glows, the
click turn and the brightness), with the language from the options' setting; its content (the
pages', the eyes', the baby's and the caret's scripts, the full colours, the dissolves' sets, the
caret's colours and home) is imported from the player's ROM, and nothing of room play is read.
It holds 1,498 cuts, 1,496 in full: at each finished cut the stage, the dissolve's count, the
colours and the colours shown, the height, each script's name, step, hold and repeat count, the
caret's place and whether it shows, the glows, the click turn, the clicks against the sound
queue's index, the music's stop against the music queue's, the brightness, the fade's count and
the equipment returned; at the 2 lag cuts, 129 and 403, which are borrowed, the brightness
alone. VRAM (E2), OAM (E1), the colour blend's bands (E3) and the PPU mirrors (E3) are masked.
The intro's end, the blank, `IntroEnds` and the equipment returned, falls in the next frame,
the flight's seed, outside the row: the played run does not witness it, its vectors and unit
test do, and it becomes live at the join into the flight. A model that never moves differs at
cut 2 in `intro.dissolve`; a press dropped by the runner differs at cut 1186 in `intro.stage`.

### The flight to Ceres

A row seeded at the flight to Ceres's first frame plays the flight alone
(`game::production::played::flight`, the screens-flight model), from the model's start with the
scene imported from the player's ROM and nothing of room play read from the seed. It steps once
per frame of work that finished, borrowing the lag and whether a track waits in audio's queue as
the frame began. `any-fly-to-ceres` holds all 802 cuts: the stage, the plane's angle, zoom and
position, its transformation and scroll, the flash, the brightness and the fade's count, the
label's step and hold, the sprites by the slot each kind takes, the places the sprite drawing is
told and whether it is done, and the letters the label types, compared from the setup's end at
cut 32 (before it, the brightness alone). Not compared: the colour cycles (R4), CGRAM's setup load
at cut 32, OAM, mode 7 VRAM, the BG1 tilemap's emptying, the letters' palette, priority and flips,
and the PPU mirrors the stages write. A lagged frame stepped as finished differs at cut 1 in
`flight.stage`. At each flight cut the run compares the appended music requests and their
delays in queue order.
On cut 32 the flight asks to load the Ceres tune with a delay of 8 frames, then to play it with
a delay of 14. The other 801 cuts issue no requests. Reversing the two requests first differs
at cut 32 in `flight.music`. Audio supplies the reply about whether a track is waiting.

At cut 803, movie frame 8,315, the flight turns the dark screen over to arrival during a
lag cut. Production carries both frame counts, brightness and the random sequence into
preparation. The run compares both counts at every cut and the random number at finished
flight cuts and every preparation cut. The first room and its actors come from room loading
and fresh-file Progress. The same run plays through the fade, the first room and the following
rooms up to movie frame 11,058. It holds 3,546 cuts from the flight seed and first differs
at 3,547 in `movement.pose`, when Ridley's tail hits Samus in flight. The cartridge save
is outside the comparison. Delaying the flight handoff until the work finishes first differs
at cut 803 in `flight.active`.

### Her arrival at Ceres

The standalone `any-ceres-arrival` run holds all 222 arrival cuts, then plays on through
movie frame 11,058 without a second seed. The start loads the shaft, its Ceres doors,
its overlay and the colours the fade heads for, with the start's own palette row for Samus.
Fresh-file Progress supplies the room version and Samus's energy and items. The preparation
carries the frame clock and random number. It borrows the loading path's entry into the
final music service and the platform's wait completions. After twenty waits, room setup
publishes the HUD at arrival cut 184, movie frame 8,499, with the imported opening frame,
the preview from the load station and room graphics recipe, and the clock from cut 183.

The comparison checks the publication cut, every HUD State field and all 128 HUD tilemap
cells through cut 222. Arrival at cut 192 turns the mini-map on and compares the carried
HUD, the room's actor roster and palette, and fresh-file Progress. The arrival still borrows
readiness from the room's loading. The first ordinary update at 193 explores the first cell
and replaces the preview. The fade advances after each frame of play until cut 222.

### From power-on

From the console's power-on, `any-power-on` plays the front end alone
(`game::production::played::front_end`, the screens-boot model). There is no seed record: the
run starts from the model's power-on State and reads nothing of cut 0. Cut 0's work RAM is the
emulator's power-on fill; its save memory is the movie's starting save memory, which P2 will
read as the game settles (cuts 263 to 270). Each cut borrows two inputs, which the run's receipt
lists (a power-on run holds nothing from a seed). One is whether its frame of work finished,
with a vblank without NMI counted as not finished. The other is whether the boot's driver
upload ended in its interval (CPU and audio timing until T1): a CPU write of 0 to APU port 1
straight after ports 2-3, looked for at every cut. An end outside the sound's load is bad data,
so a run that reaches the title found exactly one. Before the logo only the screen's blank is
compared, because the level under the blank first comes from the platform's power-on value.
After that the run compares the screen and its level, and the hold's frames left while the
logo is held. It compares the count of finished frames at finished cuts before the load (until
the clear in cut 4's interval the count is the emulator's fill) and at every cut while the
sound loads. While the game settles, the count is compared from the third cut after the
upload's end: the settle's clear runs through that cut and the two after it. The boot holds
its 270 cuts; at cut 271 the title begins and the run plays on (below). On the boot alone, a
step that does nothing holds 17. The upload's end moved to cut 252 differs at cut 255 in
`power-on.finished-frames`. A hold of 119 differs at cut 33 in `power-on.hold`. By the root's
ruling, recorded on 2026-11-24 in session #758, the lit cuts 18 to 166 count as played on the
comparisons this spec makes, with the logo's picture listed as not compared until its spec
lands. Limits:
- an end one cut late (254) is not caught, because the count is 150 on both sides at 254 and
  0 on both sides from 257;
- the logo's picture is not compared: OAM, CGRAM and VRAM at the lit cuts 18 to 166;
- the other display registers set at cut 5 are not compared: `obsel`, `bgmode` and `tm`;
- `$05B8` (every NMI counted) and the lag counts `$05BA`/`$05BB` are not compared;
- the APU port contents are not compared; only the upload's end is borrowed;
- the pad is not compared: the game reads the movie's pad from cut 18, and no boot rule acts on it;
- the settle's save memory and random number are not compared before cut 271.

### The title, file select and options

`any-power-on` plays on past the boot through the title, file select and the options menu
(the screens-title, screens-file-select and screens-options models, one step per file under
`game::production::played::front_end`), to the options' hand-on to the intro, and on through
the intro's opening and page 1 (screens-intro, through the intro's adapter) with no new seed.
Cut 568 is the first cut of the next row, `any-intro-0568`, which the run reaches as the row that
follows a power-on row: the frame end the options hand on at, where the front end begins the
intro in the State `Begins`. The run holds 2,219 cuts, through `any-intro-2219`'s cut 0, and
stops at cut 2,220 with `Intro(BattleBegins)`: the press while page 1 waits starts the battle on
Zebes, part (b) of the intro, not yet owned. From cut 568 the intro's own fields are compared
beside the front end's, as the intro's run compares them, but the report's: the front end's
sounds compare the intro's clicks and music requests (but the stops, which the queue holds as
empty entries), and whether the caret shows is not compared from power-on.

From cut 271 it compares the screen, the game state and the count of finished frames at every
cut; at finished cuts, the random number, the sounds against the queues (accumulated over each
finished cut, because a sound can appear at a lag cut), the title's countdown, file select's
and the options' cursors, and the options' block size and settings (Japanese text,
weapon-reset, moonwalk). Borrowed, as the run's receipt lists: the frames of work finished in
each interval (from the cut kind, and 2 wherever the game's count steps by 2 at a `Requested`
cut), the random number as the game settles, and P2's save-memory answers.

Planted faults: a title fast step of 3 differs at cut 308 in `front-end.screen`; a file
select menu pace of 3 at cut 403 in `front-end.screen`; file select drawing a random number on a
frame of work that goes on at cut 395 in `front-end.random`; ChoiceMade's sound coded 2Bh at cut
433 in `front-end.sounds`; Up moving the options' cursor down at cut 488 in `options.cursor`; the
options keeping the settings they were handed at cut 519 in `options.moonwalk`. The boot's two
faults fail where they did on the longer run. Limits:
- from the title on, the count of finished frames is the sum of the borrowed steps, which are
  read from the game's own count, so comparing it is not an independent check; the screen and
  the random number check those cuts. A step of 2 is taken at any `Requested` cut; this run
  sees one at cut 308 alone;
- how the file begins is the harness's P2 answer for a new file (the intro), not observed; the
  intro's language is the options' setting;
- from the intro on, the music's stops are not compared: the queue holds a stop as an empty
  entry;
- the order of the sounds asked for within one interval is not compared, within a queue or
  across queues: expected and played are each sorted before they are compared;
- save memory (SRAM) is not compared;
- the pictures are not compared: OAM, CGRAM and VRAM;
- the queues' heads and timers are not compared, only the sounds in them;
- the sub-stage indexes are not compared.

## Intra-frame timing and the played-alone run

Intra-frame timing is T1's (MODEL decision 35): absolute master-tick clocks, ordered hardware
callbacks, ordered CPU port access, the Contact latches at post-RTI clocks and Random's ordered
device transfers. No indexed case compares them.

The played-alone run carries Movement, Appearance, Camera, Doors, the colours, Health and the
HUD from the M0 seed, in room 1 the shaft's pad, cover and lights, and in Ridley's room Ridley and
the baby. The frame's draw of the random number is in the loop, and the number before it is
carried in play, where it is compared, and borrowed only through a passage; the Haze fields, and the Random fields through a passage, are covered only by their models' vectors
until their owners join the loop. Weapons, Contact,
the sound queue and the Ceres door actor at the opening shot wait for the loop in the same way.
The [transition](../scenarios/replay/transition.toml) records each retired frame case, the
owners its cut-level fields wait for and the indexed cases that still witness part of it.

The played-alone run, once indexed for M0, accounts for the row's earlier claims. It covers
42 transition claims on `chain-any-0003-DFD7`: the floor and free-run floor of each of
`samus-movement`, `samus-appearance`, `world-camera`, `interface-hud`, `game-time`,
`boss-ceres-ridley`, `weapons-projectiles`, `enemy-interactions` and `audio-transport`; the
fourteen camera receipt floors; the three lockstep entries; and the seven CI requirements,
`ordinary/*` at 183 cuts and `movement/M0-*` at 181. It also covers the seven matching
prior-claim requirements. Each is accounted for once the run holds through the cuts it claims.
Claims past cut 184 need the lag cuts as borrowed inputs: `interface-hud` at 225, and
`game-time`, `boss-ceres-ridley`, `weapons-projectiles` and the receipt frame count at 341 or
more. The row's `run-progress`, `world-blocks`, `world-doors`, `uncompared` and stream-gate
entries don't depend on the played-alone run.

## The models' gain over the null model

A witness must exercise a rule, not merely compare unchanged fields. Null-model and harness
figures from before indexed replay are labelled as historical evidence, never as acceptance.

[`tools/mutants.toml`](../tools/mutants.toml) pins one production wiring fault per supported
responsibility to its exact recorded Difference cut and field. With `SM_CACHE`, `SM_ROM` and
`SM_AUDIO_NATIVE` set, `python3 tools/check_mutants.py` first requires a Pass, temporarily
applies each fault, restores the source byte for byte, then requires the original complete
receipt again, including its digest and counts. Each fault's `before` text must match its
file exactly once, so a refactor that moves or duplicates a patched line fails Quality's
fault-runner test and the fault plan before any build. Full pull-request, scheduled and manual CI
run every fault. A fast-forward to main reuses a successful pull-request verdict for the
same head and tested tree. Missing evidence runs every fault. The supported responsibility
inventory comes from the index and reviewed baseline, so omitting an owner fails the check.
Weapons and contact have separate faults in their shared case. The played run's faults are
listed under Frames played alone.

These faults exercise movement scheduling, the audio frame's cadence, cannon permission, contact
permission, camera following, HUD clock input, animation scheduling, event delivery and Ceres scheduling. They
prove the named comparisons detect those errors within the existing scoped cases and the
played run. They do not prove all rules,
borrowed producers, omitted outputs or pixels. Unsupported
cases remain held boundaries and require no fault credit until they have reviewed Pass cases.

## Scoped Ceres tail contact

The indexed `health/ceres-tail-drawing-gauge` case starts from the authenticated native cut
992 and plays the next two frames whole. Each frame's borrowed inputs are Ridley's tail impact
(an eligible hit on the first frame, worth the energy the record observes it deliver, and none
on the second), the clock and equipment that admit
the frame, and the HUD's neighbours. Each frame plays Health once
(`samus_health::State::play_frame`), commits Drawing's energy history from the frame's Health,
and plays the HUD's one frame of play, its status bar reading the frame's whole energy. Each
cut compares Health's complete State, Drawing's energy history, the gauge's shown energy and
redraw, the Warning request, the protection Drawing reads and all sixteen CPU gauge words
against independent observations. Reads inside the frame are not compared. A protected repeat
of the hit and a stale gauge with current shown energy are separate constructed controls.

The actor's geometry and hit selection, the Warning receiver, and Drawing's hurt-flash and
rendering producers require separate admission. The scope includes two auxiliary cuts and
adds no played-alone frames.

## Scoped sender offers

Install the encrypted [sender offer fixture](../tools/ci/fetch_sender_offers.sh)
with `bash tools/ci/fetch_sender_offers.sh "$ROM" "$NATIVE"`, then select
`--rule audio-transport/OfferAcceptance --native "$NATIVE"` in the shared replay command.

Each recorded case admits one group's independent starting queue and progress and the
borrowed offer from `inputs.toml`. It plays one frame through production's audio owner with that
offer and a cadence of no audio turn and no service: the window's bracket holds no service of the
group between the seed and the comparison. The rest of the transport is a silent seed such a
frame never reads. The case compares the full oldest-first queue and progress against
`expected.toml` at the following selected-group service entry, before that service. The index
uses the source video cut, and the ingress identity distinguishes the two offers at cuts 157 and
183. Source chronology qualifies the borrowed cross-group order. Live producer order and the
sender service are outside this scope.

The constructed cases exercise same-group order, duplicates, capacity, reduced allowance
and three suppressors. The saved final cut checks restoration of the complete transport State.
Each native case has one independent seed.

## Native audio frames

`audio/native-control` (cuts 0 to 183) and `audio/native-expire-prefix` (cuts 0 to 64) play
the opening room's audio one frame at a time from the ROM-imported resident bank, the capture's
loaded audio side and the load's three ready senders. The native capture's port log supplies each
frame's borrowed inputs: the cues the CPU newly presents in the frame are its offers, and its
cadence is the effect turns completed before and after its first CPU write and its completed
music turns. A turn is complete at its Contact reply. The cases compare the song and release
fields, the four replies and the three request lines at every cut; the model's sender service
writes the request lines, so they witness RequestDelivery and RequestRelease. A cut can fall
inside an effect turn, after its Action and Ambient replies and before Contact's; the frame
form completes that turn in the next frame, and those replies are not compared (12 in the
control arm, 2 in the expiry prefix). The capture records no CPU memory, so the senders' queues
and progress are not compared. The expiry arm's post-intervention seed carries no sender State,
so its cuts 65 to 183 are not indexed.

## Live soft touchdown

The two Magnet Stairs windows start at their own locked predecessors. Each decodes Movement's
complete prior state and committed figure and supplies the observed controls, prior terrain and
two solid actors in their current Movement-entry order. The scoped caller plays Movement's whole
frame. It sends the actual Land request to the Contact FIFO with the producer's allowance and
current suppression, then passes the separate graphics opportunity and selected figure to the
dry Appearance arm. The replay compares her landing position and vertical speed, her downward
travel after the floor clips it, her position after her moves (where the pose fit starts), the
accepted response, the Contact offer and Appearance clear publication. The requested downward
distance, the fit's result and its clearance tests happen inside the frame, have no whole-frame
counterpart and are not compared. The Contact offer plays one frame of the carried transport with a cadence of no audio turn
and no service, and the next native service-entry snapshot independently corroborates its queue.
The pinned bracket contains no intervening Contact offer or service. The offer and clear
command count as two scoped outputs.

Install the [touchdown fixture](../tools/ci/fetch_movement_touchdown.sh) and the sender-offer
fixture under the same native source directory. Use the common replay command with
`--case movement/soft-touchdown-57 --case movement/soft-touchdown-124`. The checkpoints
restore each disjoint window from its own starting owners.

## Finding the code for a replay segment

The read-only `map` command joins the typed index and validated reviewed baseline with
[rule-to-code metadata](../scenarios/replay/code-map.toml). It executes no replay and needs no
ROM or cache. Each rule declares model and adapter files; navigation targets point to its
catalogue or specification. Targets are existing repository-relative files, without line or
symbol claims. The complete mapping is validated before filtering: missing, duplicate,
unknown and dangling entries fail even when the query selects another rule.

```mermaid
flowchart LR
    C[Indexed case and segment contract] <--> R[Named behaviour rule]
    R <--> F[Declared model and adapter files]
    B[Validated reviewed baseline] --> C
    E[Borrowed stimuli and scope limits] --> C
```

Run from the repository root, using the `TOOLS` and `INDEX` above:

```sh
$TOOLS replay map "$INDEX"
$TOOLS replay map "$INDEX" --case movement/W15
$TOOLS replay map "$INDEX" --rule samus-movement/TravelledDistances
$TOOLS replay map "$INDEX" --path crates/clean/samus_movement/src/travel.rs
$TOOLS replay map "$INDEX" --mapping scenarios/replay/code-map.toml \
  --baseline scenarios/replay/baseline.toml --repo . --case audio/native-control
```

Mapping and baseline paths default to `code-map.toml` and `baseline.toml` beside the index.
`--repo` defaults to the working directory and sets the root for target validation; other
argument paths resolve from the working directory. `--path` requires the exact declared
forward-slash repository-relative filename. Multiple `--case` values select their union;
case, rule and path filters intersect. An unknown selector or empty result fails.

The TOML report keeps `declaration` from the index, `reviewed_requirement` from the baseline
and `declared_relevance` from the mapping separate. Requirements include exact expected
outcomes, counts and observation digests; they are reviewed obligations, not fresh results.
Borrowed providers, phases, fields and limits stay in the declaration and do not become rule
proof. File links express declared relevance, not exhaustive impact or measured coverage.

The default query shows all indexed cases, including three empty-rule boundaries with empty code
links. A travel-file query includes the scoped witness and the integrated and movie held
Unsupported boundaries. A boundary's rule references do not make it a passing behaviour proof.
Mapping queries never select or skip landing checks; the complete index remains required.

Cache cuts are local to the named record row: W15 starts at cut 814 and compares cuts 815
through 817 of `item-morph-ball`. Native seeds and records use native fixture coordinates:
`audio/native-control` compares fixture cuts 0 through 183, and the Ceres tail record's case
compares cuts 993 and 994 after its cut-992 origin. Power-on starts have their own coordinate
family. The report exposes
the index's lock path and source declarations; it supplies no movie-global offset conversion.

## Carried ordinary movement, appearance and camera

The connected ordinary witness starts once at cut6 and compares nine actual opportunities
through cut15 in one production owner, which steps each model once per frame in the played
owner's order. Complete Movement, the four Appearance quantities, the playing Figure and all
nine Camera components are observed, along with four directional travels per frame. The
read-ahead asks the retained appearance State before her moves; Appearance plays a frame of
play from one set of inputs and movement's outcome (her work prepared, her stride, her clip,
the pose she takes and her drawing); Camera follows the finished movement and retains its
history. The owner carries her whole appearance State, decoded once from the start's record:
her animation, kept action, cannon delay and colour sequences. Her item controls and the
palette opportunity have no producer in the loop, so the delay and the sequences carry as
decoded; only the four animation quantities are compared.

The separately checked current source has five clip holds, four advances, its first wrap at
cut14 and one marked Contact at cut10, with its graphical contact and Footstep offer. The
indexed case checks each cut's continuation from a checkpoint.

Controls, equipment, liquid, energy, occupancy, the owners active at her stride, the charge,
current terrain and actors, and room settings are authenticated borrowed stimuli. Actor lifecycle, receiver execution,
health arithmetic and played-alone production remain outside this witness. Its package is
fetched by [the pinned connected fixture script](../tools/ci/fetch_connected_ordinary.sh); the declaration,
reviewed outcome and code navigation are in the existing replay index, baseline and mapping.

The second ordinary witness admits once at cut126 and carries the same selected owners through
cut142. It exercises current Run release and resumed growth, alternating Forward/DownForward
aim selection and its extra horizontal move, and reached non-square Floor/Surface terrain.
Its floor factors, vertical surface responses and unseated support checks are bounded witnesses;
stepped and ceiling terrain remain unsupported. The stale-Run fault is caught at cut141's complete movement State.
The [pinned second fixture](../tools/ci/fetch_connected_run_slope.sh) keeps its admission and source
identity separate from the first witness. The same borrowed producers and omitted consumers apply.

The scoped early mini-map witness starts once at cut 78 of the room-seeded Ceres row.
At 79 and 80 the HUD plays its whole frame, its mini-map read at movement's position after her
moves, before the aim step and pose change, from the frame's outcome. It compares both exact early axes, the complete explored
bitmap and the fifteen mini-map cells. Cut 79 also compares complete Movement, her animation's four
quantities with Figure, and Camera against the independent carried record; the rest of her
appearance State is carried and not compared. Cut 80 plays its whole frame, into
the later spinning pose, and compares only the map. The ordinary and restored executions
compare both frames, and a frame without the map's inputs is rejected before any change.

The [early mini-map fixture](../tools/ci/fetch_movement_early_map.sh) supplies the independently
qualified current collision roster, control, room, clock and other borrowed inputs; the record
pairs supply her stock as the frame begins and the values the status bar draws, which are not
compared. VRAM and audio delivery are outside this scope.

## Scoped ordinary power shot and drawing

`weapons-contact/M0-power-shot-drawing` admits the vacant shot pool at cut 156 of
`chain-any-0003-DFD7`. The same weapon owner advances through the spawn, trail request and
actor 1 consuming shot slot 0 at cut 160. W1 carries the marked body until the next update
removes it. The exact sprite encoder receives the actual flying pictures on cuts 157 and
159. The witness compares the attributed low entry and high pair at each insertion, and
the absence of pictures at the other cuts. The pre-draw cursor, camera and phase come
from the independent watched fixture. The earlier drawing producers, full OAM contents
and hardware service are outside the scope.

The [drawing fixture installer](../tools/ci/fetch_shot_contact.sh) installs its hash-pinned
files.

