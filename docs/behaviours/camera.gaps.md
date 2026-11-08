# The camera following Samus: integration gaps

Where wiring [the spec](camera.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec, or in the spec of the
domain the gap belongs to.

## The camera in the engine and the harness (integrator, 2026-11-08, lane/wire-cam)

The Tick's scroll steps the camera model through `game::camera_world`, and
`verify::harness::camera` checks it against the oracle over the 32 cached rows: 13,310 of 14,947
compared frames match on the camera's position (1,632 masked whole), against 5,438 for the null
model, with no harness defect. The proof against main `cfa5b8144` holds: the free-run gate's
output and its localisation are identical, and the lockstep ratchet is at 1372 of 1427
(item-morph-ball) and 476 of 476 (area-brinstar). Five frames differ, all on y and all from
samus-motion's `camera_from`, not the camera's rules.

### C1. samus-motion must export `rising` (open, to samus-motion's observer)

The spec's header asks for it (`AimingHeight`). Until it is exported the engine derives it at the
edge from her rise state as the engine keeps it, the vertical direction up (`0B36` = 1), which
holds from the rise's start through the frame that spends the rise; the harness reads the same
word at the scroll. Both agree on every compared frame, but the model's own export is what the
spec names, and what a samus-motion that no longer keeps that word would need.

### C2. `camera_from` misses a placement under a block-inside push (open, samus-motion)

terrain-quicksand 62, 75, 151 and 389, `position.y`: the model's camera moves (0000.C000, 0000.BC00,
0000.1400, 0000.2C00) where the oracle's holds (0000.9C00, 0000.9800, 0000.E400, 0000.FC00). Each
is a pose change under the quicksand's push (82h → 6Ch, 81h → 6Bh, A5h → 36h, A6h → F3h): the
original places the y the camera follows her from at her new y (her y at the scroll, 00A7, 00A8,
00B0, 00BB), so she didn't move and the view holds; samus-motion's step reports no placement
(`camera_from` her start y). Samus-motion's harness defers the push (its y is masked on those
frames), so its `camera_from` there is unchecked. Suspected: `PoseFitAfterMoves` (or the push's
own entry, when it comes) under the push.

### C3. `camera_from` where samus-motion's y differs (open, samus-motion)

chain-0059-A293 400, `position.y`: model 0000.2000, oracle 0000.C400. Samus-motion's step puts
her and `camera_from` at 00AB, 2 px above her start y 00AD, where the original leaves her at 00AD
(samus-motion's own harness differs there on `position.y` and `vertical`, its known miss). The
camera follows the wrong placement. It closes when samus-motion's frame does.

### C4. The engine has no knockback and never stops time (open, the engine's other domains)

`backwards` takes the skid and the moonwalk from the engine's State; the knockback's turn of the
lead needs the damage domain, which the engine doesn't have yet, so it is never set. Likewise
`time_stopped` is always false: the engine has none of the states that freeze time (the x-ray
scope, the reserve tanks' refill). The harness reads both from the oracle (`0A52`, `0A78`) and
matches on every compared frame (`TimeStopped` 95 of 95), so the model's rules hold; the engine's
edge gains them when those domains come.
