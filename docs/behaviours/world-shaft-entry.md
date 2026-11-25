# Entering the Ceres shaft during the escape

[Room loading](../architecture/domains.md#rooms) prepares the destination passage,
the collision cells at the door, and the shaft's first view. It uses the doorway chosen by
[the passage](world-doors.md) and the destination version selected by progress. [World blocks](world-blocks.md)
owns the short-lived placement that closes the door. [Ceres haze](world-haze.md)
owns the red haze once the room requests it. The platform publishes the display,
and [the Ceres actors](boss-ceres-ridley.md) own the later rumble, rotation and
elevator arrival.

| Quantity | Type and units | Read by |
| --- | --- | --- |
| World view | Affine view enabled, its background mode and a signed 8.8 matrix in A, B, C, D order | The display |
| View split | 31 lines of tiled HUD mode, then the shaft mode for the room | The display |
| Pivot | Whole pixels from the room transform's origin, x then y | The display |
| First rotation sample | The centred sample, with the next sample toward clockwise rotation | The room's rotation |
| First hold | Whole turns before the first rotation table sample, counted down once rotation starts | The room's rotation |
| Door cells | Four neighbouring cells vertically, starting at the placement's anchor | Movement, world blocks |
| Temporary placement | The pending door placement, which lives until the first block-script turn | World blocks |

Cells: the room grid counts cells of 16 px and starts at its top left. The four door
cells keep their looks, flips and marks. The shaft's escape version is 16 cells wide
and 48 high. Its closing placement is at column 15, row 38, with both counted from 0.

### ShaftViewStarts

- kind: mechanic.
- summary: the way into the shaft prepares the flat view before it tilts.
- params: pivot (128, 1008) px, initial rotation hold 60 turns.
- switch: Fixed.
- The passage reaches the entry setup for the shaft → enable the affine room view,
  select Mode 7 for the room, set the identity matrix and the shaft pivot. Seed the
  centred rotation sample with its first direction and hold. Request the HUD and shaft
  mode split [C-shaft-view].
- checked: oracle (R-entry-setup), constructed.

The setup carries the HUD mode the platform uploads and the affine outside and
mirror settings. Scroll comes from the passage as it moves the view. The platform may
publish the matrix and pivot at a cut with the screen still set to its HUD mode. The setup stores the starting hold before rotation is enabled.
The room's rotation owns counting from that hold.

### ShaftDoorSeals

- kind: mechanic.
- summary: the shaft's escape setup makes the four door cells solid.
- params: the placement's top cell is supplied by room content.
- switch: Fixed.
- The escape version starts its room setup → make the four door cells solid
  and place the red haze. Keep a temporary placement for the four cells
  [C-seal-door].
- The temporary placement takes its first block-script turn → retire it with
  the four cells still solid [C-seal-placement-retires].
- checked: oracle (R-entry-setup, R-placement-retired), constructed.

The ordinary shaft version starts haze without the closing placement. The new solid
cells keep Samus's position and the slide's schedule.

## Order that matters

Select and build the destination room before running these effects. The shaft's
view and closed door can already be visible at an interrupted cut while the passage
is fetching the room's contents. Keep both effects through the rest of that work.
The slide continues and the block-script turn can retire the placement before the
passing work finishes. The destination actors may initialize between the setup
and the end of the fetch.
