# Samus moving through the world: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec.

The previous spec's gaps and answers are in git history (`1dc3a12`).

## Answered: the Tick on the model (integrator, 2026-11-07, lane/wire-tick; observer, lane/gaps-tick)

The engine's Tick now steps Samus with the model (`game::samus_world::step`): the State is
rebuilt as the model's Samus each frame from the game's words and written back after the step
(`samus_world::state`). Every free-run gate pair holds; the lockstep ratchet falls on
item-morph-ball (1372 to 1296 Ticks fully equal). Each item below is a first differing frame
of the lockstep (record N decoded, one Tick, compared with record N+1), with the engine's value
first. None is the adapter's: each step's input matches what the game held at record N.

Items 10-16 are answered in the spec (lane/gaps-tick), each with a recorded vector from
item-morph-ball (`W11`-`W20`; gap 10's is RW1's new `readied`). The pad of a frame N to N+1 is
record N's.

10. **The wall test's result without the kick (`WallJump`).** The engine's animation holds the
    spin a frame short of the kick while a wall is near and the spin hasn't turned far enough
    (the game's armed wall jump). `Outcome` reports only the kick, so the adapter steps a copy
    of her with the spin turned far and jump pressed and reads `Met::WallKick`. Asked: the
    test's answer in `Outcome` (a wall near, kicked or not). No recorded frame of the two rows
    needs it.
    **Answered.** `WallJump`'s new output: each frame reports readied (a wall near, found by the
    block test or the solid enemy test, while the spin hasn't turned far: the animation skips
    ahead), kicked (`WhatHerMovesMet`'s wall jump), or neither (no test, no wall, or a wall near
    once the spin has turned far without jump pressed). A wall near after the spin has turned far
    readies nothing, so a copy stepped with the spin turned far can't stand in for it. Vector: RW1
    (samus-walljump-16:47, the readying frame) gains `readied = true`; no item-morph-ball frame
    readies.
11. **The ground pull with nothing under her takes the whole speed, subpixel and all
    (`FollowingTheGround`, `TurningRound`).** item-morph-ball 682 → 683, turning on the ground
    (pose 25h) over air, speed 0.C000: y 02A9.0000 → engine 02AA.0000, game 02AA.C000. The
    game pulls her by the speed plus 1 px (1.C000); the model by the speed's whole pixels plus
    1 px. On a floor the flush stop hides the difference.
    **Answered.** The pull is the frame's speed (horizontal speed plus dash) with its subpixel
    part, plus 1.0000; "whole speed" meant all of it. `FollowingTheGround` and `TurningRound`
    say so. Vector W11 (682 → 683).
12. **The aim step also comes when the pad asks for a running pose from another pose
    (`AimingAndShooting`).** item-morph-ball 687 → 688 (standing 02h → running 12h, left and aim
    down held): x 057D.D000 → engine 057D.D000, game 057C.D000; 740 → 741 (landing A5h →
    running) and 1327 → 1328 (landing E4h → running 11h) the same 1 px. Into a wall it puts her
    against it: 1334 → 1335 (standing 07h, right and aim down held, a wall ahead): engine pose
    09h (running), game 89h (against the wall, aimed down-forward), and so on every other frame
    to the row's end. The spec has the step only while running and changing aim.
    **Answered.** The step comes whenever the pose chosen from the pad and what her moves met is a
    running pose other than hers, from any pose; forward is her facing at the frame's start. A
    landing or a fall that replaces the pad's choice brings none, and an animation's end neither
    brings nor cancels it (§11's and §7's open item on that is closed). A block or solid enemy in
    the way gives `AgainstWall` with the chosen running pose's aim (`WhatHerMovesMet`). At 1334
    the pad (record 1334's) is run and right, no aim: the chosen pose is running unaimed (09h),
    the wall is 0 px ahead, so she is against it unaimed (89h). Vectors W12 (687), W17 (740),
    W18 (1327), W19 (1334).
13. **A stop short of a solid enemy clears her y subpixel (`SolidEnemies`).** item-morph-ball
    796 → 797, falling onto a solid enemy 1 px below (no block near): y 029A.2BFF → engine
    029B.2BFF, game 029B.0000; 1227 → 1228 the same (029B.9400 against 029B.0000). Standing on
    it after, 1229 → 1230 (running 09h, y 029B.0000), the engine's pull finds no floor (pose 29h,
    falling) and the game keeps her running.
    **Answered.** The spec's gap was off by one: it is the enemy's near side less hers, the sides
    being the boxes' edges at centre ± half-size, with no "less 1" (boxes whose pixels are next
    to each other are touching). At 1227 the enemy is 2 px below (her bottom side 02AEh, its top
    02B0h): the landing stops her 2 px down, subpixel kept (029D.9400); the landing pose's body is
    2 px taller, and pose fit's look down (a test like any move's) finds her touching the enemy,
    which sets her y subpixel to 0, and the push lifts her 2 px: 029B.0000. 796 is the same with
    3 px. At 1229 her bottom side is the enemy's top: the pull touches it and she stays on it. So
    no subpixel rule changes; only the gap's measure. The constructed vectors C-enemy-ahead,
    C-enemy-adjacent, C-enemy-down and C-aim-enemy are redone on the corrected gap (two keep their
    `after` with the enemy or the start moved 1 px; C-enemy-ahead now stops 2 px). §11 item 5 is
    corrected. Vectors W13 (1227), W20 (1229). A note for the adapter: the body size a frame
    uses is the pose's own (§1.3); the game's radius word still holds the old pose's 13h at
    records 740, 797 and 1228 (after a landing that grew her), and the next frame's moves use 15h.
14. **Reversing a spin while skidding keeps the skid (`TurningRound`).** item-morph-ball
    767 → 768: spinning right (19h) while skidding left (`0B4A` = 1, speed 2.3000), left pressed:
    she reverses to 1Ah, moving 1.B000 left that frame. Engine `0B4A` 2 (her way left, along her
    new facing), game 1: by the decode notes her way is now right, and 768 → 769 she moves right
    (04AB.DFFF → 04AB.FFFF).
    **Answered.** Every turn and every spin reversed leaves her skidding: her speed then points
    against her new facing, whatever she was doing. Reversed while already skidding, her travel
    turns round: that frame she moves the old way (1.B000 left), and from the next her speed points
    right. `TurningRound` and §1.2's skid say so. Vector W14 (767 → 768, speed_x +1.B000 after).
15. **The wall test's cells are reached too (`TouchedCells`, `WallJump`).** item-morph-ball
    816 → 817: spinning carried on (19h, `0A27` = 3) with right held, the test looking left
    reaches the item's block (45h, 29h), 8 px past her left edge; the game picks the item up
    (record 817 is in the message box). The model's `Outcome::reached` lists only her moves'
    cells, so the engine never touches the item, and the row diverges from there (the free run's
    first room field is the item's PLM).
    **Answered.** The near-wall test reaches cells as a horizontal move of 8 px would, after the
    horizontal move's cells and before the vertical move's, with the way `Look`: a reaction that
    depends on the way she moves (a crumbling floor, a station's side) doesn't answer a look, the
    others (an item, a scrolling cell) do. A solid enemy that decides the test leaves no cells.
    `TouchedCells` and `WallJump` say so; the vector format adds the way "Look". Vector W15
    (814 → 817 with two lead-in frames: her left edge's column 46h, then the test's column 45h,
    then the fall's row).
16. **Taking off from a crouch with run held gains no dash (`Jumping`, `Dash`).** item-morph-ball
    1311 → 1312 (take-off 4Bh, run and right held): engine x 0523.4FFF, dash 0.1000, `0B4A` 2;
    game x 0523.3FFF, dash 0, `0B4A` 0. The model's take-off gains dash and moves by it, and the
    jump then carries on along her facing.
    **Answered.** Only running gains dash; a take-off gains none, and a dash she is to lose is gone
    before its move. `Jumping`'s "still gains dash" was wrong and is replaced; `Dash` says a
    take-off gains none. With no dash the jump that follows follows the pad. Vector W16
    (1311 → 1312).

Item 9 above now has a frame: item-morph-ball 1310 → 1311, the crouch's take-off (27h → 4Bh,
radius 10h → 13h): y 02C0.FFFF → engine 02B6.FFFF, game 02B3.FFFF. The game pushes her up the
3 px the taller body needs, then lifts her 10 px; the engine's pose fit runs after the model's
lift and finds no floor to push from. Round 7's push (`PoseFitAfterMoves`) answers it: the
frame is equal on lane/wire-r7.

## Answered: the domain harness on round 7 (integrator, 2026-11-07, lane/wire-r7; observer, lane/gaps-harness)

With round 7 the Tick steps her through the model with pose fit and the shrink inside the step
(`game::samus_world::step`), and the proof holds: the free-run gate's eight green pairs, and the
lockstep ratchet at the spine's numbers (item-morph-ball 1372 of 1427 Ticks fully equal, the same
Ticks and fields differing as on the spine). The old model is gone, and the domain harness
(`verify::harness::samus_motion`) steps the model over each record's own level and solid enemies,
with nothing copied from record N+1: 11,559 of 12,782 compared frames match over the 32 cached
rows, against 5,660 for the null model. The rows below the null model or the old model's floor
come from these frames. The pad of a frame N to N+1 is record N's.

Items 17-19 are answered in the spec (lane/gaps-harness). All three are this domain's: none is the
speed booster's, and no grapple runs in gap 19's frames. Vectors HV1-HV4 and AS1-AS3.

17. **Running on the speed booster's run with a downward vertical speed (`MovesAsShePosed`,
    `FollowingTheGround`).** samus-shinespark-up 0 → 1: running (09h), base speed 2.C000, dash
    2.7000, `0B3C` set, `0B36` = 2 with `0B2E`.`0B2C` 0.E000, on the floor. Model: base speed
    0.0000, vertical none (she lands, and the landing halts her); game: base speed 2.C000,
    vertical down 0.E000, both unchanged every frame of the run (frames 1-151; the speed booster's
    mask takes x and the dash, not these). The same on samus-shinespark-left, -right and -up-left
    from frame 1, and samus-mockball frames 1-42. The decode notes call a running pose with
    `0B36` = 2 and a magnitude not plain. Asked: what this State is in the spec's terms, and its
    rule.
    **Answered** (`FollowingTheGround`, the speed she keeps on the ground; `Landing`): on the
    ground her vertical speed is kept as it is. She doesn't move by it, the pull doesn't grow it,
    and the floor under her doesn't land her, so nothing halts her. Only her next change of shape
    shows it. The speed booster plays no part: samus-space-jump-chain and the three wall-jump rows
    run, brake, stand and turn with the same +0.E000, and their dash is 0 from frame 1. The decode
    notes now call such a start plain.
18. **After that run, a vertical speed of 1.3400 on the ground.** samus-shinespark-left 157 → 158
    (to 183) and samus-shinespark-right the same frames: model vertical speed 0.0000, game 1.3400,
    held. Asked: the same.
    **Answered** (the same entry): the crouch from the run (151-155) is a change of shape. It
    moves her by her kept +0.E000 under the pull, which takes it to +1.3400, and it ends before it
    meets the floor. Crouched, she keeps that speed through the turn on the ground to the
    take-off at 196 (HV3).
19. **A dash merged on the ground moves her a pixel further (`Dash`, `SpeedBuildsAndBrakes`).**
    samus-grapple-walljump 0 → 1: running (11h), x 002C.BFFF, base speed 2.C000, dash 1.2000,
    `0B3C` set. Both end with base speed 3.E000, no dash, `0B4A` = 2; x: model 0030.9FFF (by
    3.E000), game 0031.9FFF (by 4.E000), and 1 px behind to frame 8. Asked: the frame's move.
    **Answered** (`AimingAndShooting`, `PadChart`'s running let-go row): the merge doesn't cause it.
    With no row fitting (only select is held), running with a horizontal speed at the frame's start
    that isn't 0, the let-go row keeps her running pose. That kept pose is chosen as a running pose,
    so the aim change's step moves her 1 px on top of the braking move every frame she brakes so:
    frames 0-7 here, and samus-space-jump-chain 0-11, where the frame braking takes 0.2000 to 0
    moves her only the step. When the speed at the frame's start is 0 (space-jump-chain 12) she
    stands, and there is no step. A row naming her own pose brings no step (forward held,
    running). Grapple doesn't hand this frame over: the grapple beam is inactive, her movement
    handler is the normal one and the grapple wall-jump timer is 0 in records 0-9 (AS1-AS3).

Two things the edge now keeps that the model doesn't hold, neither the model's: the camera's
previous y pixel (`0B14`) becomes hers as pose fit, the crouch taken instead or the shrink left
her (before the crouch take-off's lift), so the camera doesn't follow those moves
(item-morph-ball 796 → 797 and 1227 → 1228, which moved the camera's subpixel otherwise); and a
moonfall's vertical speed word is her speed upward (`0B36` = 0: the decode's "− magnitude"), which
the round-6 edge had the wrong way (chain-0014-96BA 11 → 12 onward).

## Open: frames the narrowed speed-booster mask shows (observer, 2026-11-07, lane/gaps-harness)

The harness's speed-booster mask took y and the vertical speed on every frame she had any dash,
and x and the dash on every frame the dash was 2.0 or more. It now takes only what the booster
writes: x and the dash while she runs with run held and a dash of 2.0 or more, and the vertical
speed of a frame on which she starts a rise with a dash. The frames this uncovers are this domain's.
Most match (mockball's x, 19 of 19 compared before, is 87 of 87 now). These don't:

20. **The aim change's step every frame down a slope (`AimingAndShooting`, `SlopesCarry`).**
    chain-0220-D017 39 → 40 to 42 → 43: running down a slope with dash 0.C000 to 0.F000. Her aim
    changes every frame (the step every frame). y: model 1 px lower than the game (40: 006E.FFFF
    against 006D.FFFF). At 43 → 44 she leaves the slope: model 0078.FFFF, game 007D.CFFF, falling.
21. **A fall from the ground's edge that the model doesn't move (`FollowingTheGround`).**
    chain-0106-AD5E 13 → 14: running with run held (base speed 2.C000, dash 0.C000), into a fall.
    y: model 028B.FFFF (no move), game 0290.8FFF (the whole pull, 4.9000).
22. **The frame a hit starts knockback, falling (`SolidEnemies`?).** chain-0106-AD5E 256 → 257 and
    293 → 294: falling in a straight jump (at 0.3800 and 0.7000), the frame knockback starts. y:
    model 010C.53FF and 010D.17FF, game 010B.FFFF both times, above where she started. The
    knockback mask takes her speeds, not y. Asked: whether the hit's y is this domain's.

23. **The run with a kept vertical speed, on slopes (`FollowingTheGround`, gap 17's answer, not
    yet in the model).** y, newly compared: samus-mockball 10-14 and 16-23; samus-shinespark-left,
    -right, -up and -up-left 10-14, 16-23, 79-81, 95-96, 99-100, 104-105, 110-120, 141-142,
    146-147 and 151-152 (39 frames each; with the two each row failed before, the shinespark
    start and one later frame, 41). She runs (09h) with +0.E000 kept, over slopes. Model: it moves
    her down by the kept speed as a fall (growing it to +0.FC00) instead of the ground's pull,
    and once that move meets the floor it lands her and halts her (base speed 0, vertical none).
    Down a slope she is left above the floor (mockball 11: model 048E.DFFF, game 0493.FFFF), up
    one she rises less than the floor lifts her (shinespark-left 96: model 04B5.FFFF, game
    04B0.FFFF). Game: the ground's pull, `|speed| + dash + 1` with the slope carrying her (§8),
    puts her on the floor, and the +0.E000 is kept unchanged. **Answered** by gap 17 (the spec
    already says it, HV1-HV3, AS3): for the implementer, no spec change.
24. **The spring ball's roll 1 px past the floor (`FollowingTheGround`, `SlopesCarry`?).**
    samus-mockball 91 → 92 and 92 → 93: the spring ball on the ground (7Bh), base speed 3.4000,
    dash 5.0000, no vertical speed. y: model 04BE.FFFF and 04C6.FFFF, game 04BD.FFFF and
    04C5.FFFF (1 px higher, from 04B9.FFFF and 04BD.FFFF); the next frame, on the level floor,
    both are at 04C9.FFFF. Like item 20, a ground pull 1 px lower than the game where the floor
    falls away. Asked: the pull's stop on that floor.

area-maridia 190 → 191 (x 1 px short) is gap 19's step: answered.

## Open: the engine's pose at the edge (integrator, 2026-11-07, after `c44e48c`)

Items 6 and 7 are answered by the observer (lane/pose-map): §2's new parts, the map both ways
in the decode notes, and `SlopesCarry`'s empty column. Both are wired (lane/wire-r6): the
bridge reads the importer's map (`data::assets::samus::import::map`) with `clean_types::Pose`
as its key, and `surface_of` answers an empty column `JustPast` and one far past `FarPast`,
for a floor and a ceiling alike.

Items 1-3 below are answered by `c44e48c` (the pose chart, `Outcome::reached`, `World`'s kinds).
`game::samus_world` (`78dc21a`) answers `World`. What stops the next step, the State swap:

6. **The engine's pose from the model's (§2).** Animation, drawing, the body's size and pose fit
   read the engine's pose, one of 253 sprites; the model chooses a §2 pose, which §2 says has
   fewer parts than the sprites (the gun, the landing sprites, the spin drawn as the space jump or
   the screw attack). Asked of the observer: the decode both ways at the edge, as data:
   - each sprite's §2 pose (body, action, facing, aim, gun), above all the crouch and morph
     transitions' four actions and the take-off's poses;
   - for a chosen §2 pose, which sprite the game shows: by the sprite she was in, her equipment,
     and whether she landed, as the records say.
   Without it the edge map guesses, and every guess is a free-run gate pair at risk.
7. **A ceiling's surface where a column has no solid pixel.** The ROM's surface offsets reach 16
   (a column with nothing solid in the block). A floor's `surface` row 16 is past the block and
   never stops her, as in the game. A ceiling's lowest solid row is then -1, which `Slope::surface`
   (`u8`) can't hold; the adapter answers 0, so a rise whose top edge is on the block's row 1
   stops where the game lets it on. Asked: the spec's surface for such a column.
8. **A door to a room is air (`OtherBlockKinds`), but the level alone can't tell it from a door to
   nowhere.** The adapter answers every door solid (`DoorToNowhere`), as the engine's collision
   does today; the door's destination is in the room's door list, not the block.
9. **Pose fit's push is an input, but it follows the pose the step chooses (`PoseFitAfterMoves`,
   round 6's `Frame::push`).** Round 6 answers item 1 below inside the step (`WhatHerMovesMet`
   and the chart), so the engine no longer knows the frame's pose before the step, yet the push
   that pose's body needs is given before it. The engine can only pass 0 and run pose fit after
   the step, which puts the push after the crouch take-off's 10 px lift. Asked of the model: the
   push as an answer the step asks for once it has chosen the pose (for example a `World` method
   given the new body's radius), so the order §`PoseFitAfterMoves` states holds in the engine.
   No recorded frame tells the two orders apart yet (spec gap 6: a block within 10 px above her).

## Open: what wiring the model into the engine needs (integrator, 2026-11-07)

These come from wiring, not from a differing frame: the engine can't drive the model's public API
as it stands without a stand-in. None has a frame yet; each names the entry it touches.

1. **The pose a frame changes to depends on that frame's moves (§7, `Landing`,
   `WallsAndCeilings`, `FollowingTheGround`).** `Frame::next_pose` is an input given before the
   step, but in the game the pose a frame takes is chosen after its moves: a landing, a fall from
   the ground's edge, a ceiling, a wall stop and a wall jump's kick each choose it. The harness
   can read it from the next record (a stand-in that answers from the oracle, MODEL §4); the
   engine can't. Asked of the model: one step that, after the moves and before the pose change,
   asks the caller for the pose with what the moves met (landed, left the ground, hit a ceiling,
   stopped at a wall, kicked off a wall), for example through a trait the caller implements.
   The alternative, two passes (the moves on a copy to learn what they met, then the real step),
   duplicates the frame's work and infers the contacts from the State's change.
2. **Blocks that react when she reaches them (§8, §10).** In the recorded rows her moves reach
   scroll triggers (area-brinstar, the elevator ride's step) and an item's block (item-morph-ball,
   the pickup), which let her through and start something. `World::block` answers only air,
   solid, stepped or sloped, and nothing tells the caller which cells a move reached. The adapter
   can record every cell asked, but the model may ask cells the game's moves don't, or in another
   order. Asked: the set and order of cells a move reaches, as the spec defines it, so the world
   can report which reactive cells she reached.
3. **The other block kinds (§10).** Doors, spikes, crumble and speed blocks, shootable, grapple
   and bombable blocks, and the blocks that stand for another block (a run of one block's cells).
   The adapter answers solid for the solid-to-touch kinds and resolves a standing-for cell to the
   cell it names, as the engine's collision does today; the spec should say so or list them.
4. **A walk along a sloped floor changes the move's length (§10).** Today's engine scales a
   horizontal move on a non-square sloped floor, with no vertical speed, by the shape's factor;
   the model doesn't. Every recorded frame that walks a slope will differ in x until that entry
   exists.
5. **Pose fit moves her between the pose choice and the pose's speed changes.** When her body
   grows, the game pushes her clear of the blocks (out of scope, the header) after choosing the
   pose and before the crouch lift and the pose change's writes. The model does the crouch lift
   and the writes inside the step, so the push lands after them in the engine. Whether any
   recorded frame tells the two orders apart is not yet measured.
