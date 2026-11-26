# Receiving the Landing Site load

Vectors: [landing-site-prefix.toml](../../scenarios/vectors/landing-site-prefix.toml).

## Header

This prefix begins with the pending destination from the [flight to Zebes](screens-zebes.md).
It ends during the first sprite graphics upload. The scheduler opens the next work interval
and supplies the part of the graphics transfer that can run before the cut. The admitted work
opportunity reaches the end of the standard BG3 graphics upload. The platform
installs display settings and writes its own live video storage. R2 selects the destination's
area and entrance, and restores that area's explored cells and map availability. The current
room and player placement continue to describe Ceres until the load reaches the station
table. Room contents, palettes for play, the visible landing and subsequent loading remain
outside this prefix.

The scheduler carries the finished-work count, pending destination, shared random sequence
and whether the pending work has opened. Health and the flight supply the carried energy,
display blanking and cleared affine plane. The owner of explored maps supplies the explored cells
and map availability by area. The renderer consumes the normal committed display view.

### Carried quantities

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| load destination | typed area and entrance | LandingWorkOpens |
| load phase | Pending, TransferringSprites, or ready for the next load phase | LandingWorkOpens, StandardGraphics |
| selected area, entrance and current room | identities | map projection, later room load, output |
| explored cells and map availability | set of map cells and area status | map display, output |
| graphics pages and prepared tilemaps | byte spans and tile words with named destinations | GameplayDisplaySetup, StandardGraphics |
| transfer state | source and destination content identities, byte length, accepted and committed byte counts, optional pending write and selected read | StandardGraphics, output, restoration |
| work state and video | existing scheduler counts, palette, object table, settings, live graphics memory and published display | GameplayDisplaySetup, StandardGraphics, output |

Serialize the carried phase and transfer state along with the already-written live memory.
The pending write retains its byte and its location, or an immutable asset identity and
an offset that reproduces that exact byte. A selected read retains its source location.
The scheduler owns finer timing if the selected read is partway through a bus operation.

### LandingWorkOpens
- kind: Mechanic
- summary: The blank flight hands off to the selected entrance as the next work begins.
- switch: Fixed
- pending Landing Site destination and an opening work interval → start loading once, draw from the shared random sequence for the new work, select Crateria and its Landing Site cutscene entrance, and replace the active map with Crateria's stored explored cells and availability [L-open, L-map].
- an open load enters a cut before the work finishes → carry the same finished count, energy, room header, placement and affine pose. Keep forced blank [L-open, L-no-finish].
- A pending load waits for a work opening. Another destination goes to its own loader [L-no-work, L-other-target].
- checked: oracle (L-open, L-map, L-no-finish, L-no-work, L-other-target)

### GameplayDisplaySetup
- kind: Mechanic
- summary: Gameplay display settings and tilemap buffers are prepared under forced blank.
- switch: Fixed
- the Landing Site work starts → keep forced blank at zero brightness. Select Mode 1 with BG3 priority and small background tiles. Set the gameplay object page, tile bases and map dimensions. Enable BG1, BG2, BG3 and objects on the primary image. Disable mosaic, the secondary image, windowing and colour blending. Use standard noninterlaced display settings. Reset the first object's x and y and the object write cursor, retaining every other object byte and the current palette. Prepare gameplay tilemap buffers for their later upload [L-open, L-display].
- Until the requested display commit completes, retain the last display publication. The loader's direct video writes still take effect at their scheduled opportunities [L-no-finish, L-split].
- checked: oracle (L-open, L-display, L-no-finish, L-split)

### StandardGraphics
- kind: Mechanic
- summary: The standard BG3 graphics load before the sprite graphics, which can stop partway.
- switch: Fixed
- display setup completed → transfer the standard BG3 graphics to their gameplay page and drain the transfer before beginning the sprite graphics upload [L-open, L-split].
- the sprite transfer accepts a byte → commit the earlier pending byte, if any, and retain the newly accepted byte for the next write. Alternate the two bytes of a graphics word in low-first order. Advance the word position after its high byte is written [L-split, L-resume].
- transfer has accepted its last byte → write the final pending byte at the drain opportunity and make the next load phase ready [L-drain].
- resuming an interrupted upload → keep the prepared area, map and settings, use the retained transfer position and commit each remaining write once [L-resume, L-reopen].
- A cut in the upload preserves the bytes beyond the written prefix and all bytes outside the two graphics destinations. The tilemap uploads wait for the next load phase [L-split, L-no-finish, L-drain].
- checked: oracle (L-open, L-split, L-resume, L-reopen, L-drain, L-no-finish)
