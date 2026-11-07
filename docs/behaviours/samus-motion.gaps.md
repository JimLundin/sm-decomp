# Samus moving through the world: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec.

The previous spec's gaps and answers are in git history (`1dc3a12`).

## Open: the Tick on the model (integrator, 2026-11-07, lane/wire-tick)

The engine's Tick now steps Samus with the model (`game::samus_world::step`): the State is
rebuilt as the model's Samus each frame from the game's words and written back after the step
(`samus_world::state`). Every free-run gate pair holds; the lockstep ratchet falls on
item-morph-ball (1372 to 1296 Ticks fully equal). Each item below is a first differing frame
of the lockstep (record N decoded, one Tick, compared with record N+1), with the engine's value
first. None is the adapter's: each step's input matches what the game held at record N.

10. **The wall test's result without the kick (`WallJump`).** The engine's animation holds the
    spin a frame short of the kick while a wall is near and the spin hasn't turned far enough
    (the game's armed wall jump). `Outcome` reports only the kick, so the adapter steps a copy
    of her with the spin turned far and jump pressed and reads `Met::WallKick`. Asked: the
    test's answer in `Outcome` (a wall near, kicked or not). No recorded frame of the two rows
    needs it.
11. **The ground pull with nothing under her takes the whole speed, subpixel and all
    (`FollowingTheGround`, `TurningRound`).** item-morph-ball 682 → 683, turning on the ground
    (pose 25h) over air, speed 0.C000: y 02A9.0000 → engine 02AA.0000, game 02AA.C000. The
    game pulls her by the speed plus 1 px (1.C000); the model by the speed's whole pixels plus
    1 px. On a floor the flush stop hides the difference.
12. **The aim step also comes when the pad asks for a running pose from another pose
    (`AimingAndShooting`).** item-morph-ball 687 → 688 (standing 02h → running 12h, left and aim
    down held): x 057D.D000 → engine 057D.D000, game 057C.D000; 740 → 741 (landing A5h →
    running) and 1327 → 1328 (landing E4h → running 11h) the same 1 px. Into a wall it puts her
    against it: 1334 → 1335 (standing 07h, right and aim down held, a wall ahead): engine pose
    09h (running), game 89h (against the wall, aimed down-forward), and so on every other frame
    to the row's end. The spec has the step only while running and changing aim.
13. **A stop short of a solid enemy clears her y subpixel (`SolidEnemies`).** item-morph-ball
    796 → 797, falling onto a solid enemy 1 px below (no block near): y 029A.2BFF → engine
    029B.2BFF, game 029B.0000; 1227 → 1228 the same (029B.9400 against 029B.0000). Standing on
    it after, 1229 → 1230 (running 09h, y 029B.0000), the engine's pull finds no floor (pose 29h,
    falling) and the game keeps her running.
14. **Reversing a spin while skidding keeps the skid (`TurningRound`).** item-morph-ball
    767 → 768: spinning right (19h) while skidding left (`0B4A` = 1, speed 2.3000), left pressed:
    she reverses to 1Ah, moving 1.B000 left that frame. Engine `0B4A` 2 (her way left, along her
    new facing), game 1: by the decode notes her way is now right, and 768 → 769 she moves right
    (04AB.DFFF → 04AB.FFFF).
15. **The wall test's cells are reached too (`TouchedCells`, `WallJump`).** item-morph-ball
    816 → 817: spinning carried on (19h, `0A27` = 3) with right held, the test looking left
    reaches the item's block (45h, 29h), 8 px past her left edge; the game picks the item up
    (record 817 is in the message box). The model's `Outcome::reached` lists only her moves'
    cells, so the engine never touches the item, and the row diverges from there (the free run's
    first room field is the item's PLM).
16. **Taking off from a crouch with run held gains no dash (`Jumping`, `Dash`).** item-morph-ball
    1311 → 1312 (take-off 4Bh, run and right held): engine x 0523.4FFF, dash 0.1000, `0B4A` 2;
    game x 0523.3FFF, dash 0, `0B4A` 0. The model's take-off gains dash and moves by it, and the
    jump then carries on along her facing.

Item 9 above now has a frame: item-morph-ball 1310 → 1311, the crouch's take-off (27h → 4Bh,
radius 10h → 13h): y 02C0.FFFF → engine 02B6.FFFF, game 02B3.FFFF. The game pushes her up the
3 px the taller body needs, then lifts her 10 px; the engine's pose fit runs after the model's
lift and finds no floor to push from. Round 7's push (`PoseFitAfterMoves`) answers it.

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
