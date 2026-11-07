# Samus moving through the world: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec.

The previous spec's gaps and answers are in git history (`1dc3a12`).

## Open: the engine's pose at the edge (integrator, 2026-11-07, after `c44e48c`)

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
