# Status: how to measure it

Current acceptance is indexed production replay. The committed
[index](../scenarios/replay/index.toml), [baseline](../scenarios/replay/baseline.toml) and
[transition ledger](../scenarios/replay/transition.toml) define witnesses, exact obligations and
reviewed replacement/loss dispositions. Generated numbers live in [REPORT](REPORT.md) and the
README's progress; regenerate them as [BUILDING](BUILDING.md#the-status-and-the-report) describes.

```mermaid
flowchart LR
    A[Pick a behaviour] --> B[Load a short indexed case]
    B --> C[Run selected production systems]
    C --> D[Compare independent oracle observations]
```

## The headline milestone

The goal remains power-on through Ceres using movie inputs only, every compared cut and output
byte matching. Scoped and integrated witnesses are useful progress but do not substitute for
whole-movie execution. Required missing boot, clock, controller, actor or passage owners remain
explicit unsupported boundaries. Borrowed facts never count as modeled producers.

## Frames played alone

The rung measure is the number of cuts a room plays alone from its seed.
[`verify::replay::played`](../crates/verify/src/replay/played.rs) admits the whole-frame
owner [`game::production::played`](../crates/game/src/production/played.rs) once from the seed
record. Each frame of play it steps her clip's look ahead, `Samus::step`, the animation update,
the camera and the doors, once each, in any pose with imported content. Each frame of play
starts with her health (samus-health's whole frame, with no tail impact: Ridley's fight is not
in the loop) and ends, after her moves, with the HUD's frame of play. The HUD reads her position after
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
(`ceres.0.drawn-with-ridley`). The escape refuses the frame: its shakes read the screen's
shaking, which is not in the loop. A door's
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
overlay, so a room holding it refuses the frame (`Fault::Escape`). The count of finished frames
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

Two more facts are borrowed, the second only through a passage. Audio's answer to the passage, whether a track waits in
the queue of tracks as the frame of work begins, is read from the record's music queue as the
main loop's start steps it once (world-doors' decode); the passage into Ridley's room waits on it
for 68 frames as his room's music loads; the earlier passages of these rows play as ready. Every
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
first colour, then the door's calm set in its own palette); and the music's stop and the shake's
stop, which are produced and not compared (audio's boundary and the screen's shaking are not in
the loop). Each frame the room's enemies run, his run comes first, reading whether a passage is
under way and the frame's random number; the doors run after him reading whether he is beaten
(progress's, held: no run here beats him). The baby's colour sets go to the screen's sprite row 3
at once, compared as `doors.colours.screen`, and its cry is produced and not compared. His
drawing, his look, the baby's and the doors he draws closed, is produced for drawing and not
compared. The run compares whether he is placed and his phase, wait, look, position and the
baby's routine, runs and position (`ridley.phase`, `ridley.baby.routine`) at every cut whose
frame of work finished. His waking is a later piece's: a frame he would run awake stops the run
(`Fault::RidleyAwake`). Her going backwards, which the camera reads, is her skid or her moonwalk
(world-camera's `LeadingHer`); room 6's moonwalk at its left edge is the first played frame
that tells the moonwalk apart, at cut 352.

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
281 cuts, `chain-any-0004-E021` all 263, and `chain-any-0006-E0B5`, Ridley's room, 512 from its
seed. Played back to back, the Ceres rows from `chain-any-0001-DF45` hold 2,136 cuts, through
rooms 2 to 5 and on to cut 512 of `chain-any-0006-E0B5`; from `chain-any-0002-DF8D` they hold
1,724 cuts, from `chain-any-0003-DFD7` 1,442, from `chain-any-0004-E021` 1,097 and from
`chain-any-0005-E06B` 833, each to the same stop. That stop is Ridley waking at room 6's cut 512:
the next frame he runs is his fight's, a later piece (`RidleyAwake`).
CI runs it for the M0 row from its seed (cut 0) and from cut 6 when dispatched with the target
`played`. It is a diagnostic, not yet an indexed obligation.

Thirty planted faults in the fault matrix, those of the responsibility `played`, stop
that run, or the run of another Ceres row from its seed, at a recorded cut. For example, a pad one frame stale fails the runner's pad alignment
check at cut 2, and a camera State held rather than carried differs at cut 1 in
`camera.view.x.fraction`. Every button held at the seed taken as newly pressed differs at
cut 17 in `movement.jump-held`, played from the M0 row's cut 16. A frame on which the doors
restart the frame count counted on instead differs at cut 238 in `frame-count`. The last room's
geometry, doors and camera room kept after a passage builds the next differ, played back to back
from the M0 row's seed, at cut 348 (the next row's cut 3) in `movement.position`. Of the Ceres doors' faults,
their State held rather than carried differs at cut 1 in `ceres.0.wait`, and the doors reading her position after her moves rather
than before differs at cut 7 in `ceres.0.phase`. In room 1, the pad's carry dropped from her
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
the colours the row holds), and the random number not drawn as the frame's work begins at cut 1
in `random`.

The M0 row never shows a door blocking her: a fault that drops the doors' boxes from what her
motion meets holds all 344 cuts, and rows 0004 and 0005 as far as they play; the door of
Ridley's room locks behind her, out of her way. Only the unit
test of a door that opens in the frame still blocking her catches it. Retiring the indexed
case `movement-ceres/carried-position-proximity` lost its gating obligation, which the
diagnostic played run compares but does not gate. That loss is accepted until the played run
is indexed.

## Intra-frame timing and the played-alone run

Rungs compare owners' State and output bytes at NMI cuts (MODEL decision 35). Intra-frame
timing is T1's: absolute master-tick clocks, ordered hardware callbacks, ordered CPU port
access, the Contact latches at post-RTI clocks and Random's ordered device transfers. No
indexed case compares them, and production has no frame caller that schedules them.

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

A witness must exercise a rule, not merely compare unchanged fields. Catalogue entries name
rules; cases are witnesses and many cases may be needed per rule. Unit vectors and adjusted
parameter vectors remain development checks. Meaningful planted faults establish whether
indexed comparisons distinguish incorrect behaviour. Historical null-model and harness figures
are not current production acceptance and must be labelled as historical evidence.

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
(an eligible hit on the first frame, none on the second), the clock and equipment that admit
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

## The ratchets and floors

The replay baseline binds the exact index bytes to each measured result, comparison counters
and observation digest. Behaviour obligations require completed comparisons. Boundary
obligations require the exact named Unsupported result and remain distinct from passing cases.
Missing source data, a zero-case selection, an empty comparison schedule or an earlier boundary
cannot satisfy the corresponding obligation. The transition ledger accounts for retired
lockstep, gate, localiser, per-domain execution and frame case claims; there is no second
acceptance runner.

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
Local case selection is useful diagnosis; required acceptance runs the complete index. Cases
carry production State from the admitted start with no feedback from expected records and no
automatic reseed after failure. Scope declares selected owners, borrowed stimuli, compared
fields/outputs and excluded consumers. A movement case without rendering proves no pixels.

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

## Making the record cache


`CACHE` is made by `oracle corpus ROM scenarios/lock.toml DATA --oracle lsnes --native-seeds
BUNDLE/seeds --from-frame-pads PADS --cache CACHE --only ROW` (ORACLE §6): lsnes plays the row's
frame pads from its native seed and writes the lean tier, `CACHE/ROW/lean.bin` and, last,
`lean.txt` with the lock pins `verify::cache` checks: about 2 KB a frame, each frame's memory
changes with the whole memories every 256th frame. `DATA` is the old lock's decrypted bundles
(corpus still reads each row's seed and movie name there); `--cache-pictures` writes the full
tier instead (`records.bin`, `index.txt`, 330 KB a frame with its picture), which only the
compositor's checks need. `tools cache-lean` turns a full tier into a lean one.

`tools/oracle/record_caches.sh ROWS BUNDLE DATA ROM CACHE [JOBS]` does it for a list of rows,
8 lsnes at a time, skipping rows already cached, one log line per row; with `WATCH_PCS` set it
passes `--watch-pcs` too, for the `hits.txt` the independent causal input providers can read.
`ORACLE` names the oracle command (the Python DLL runner where Defender blocks executables).

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

