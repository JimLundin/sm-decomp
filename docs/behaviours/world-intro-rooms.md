# The intro's playable rooms

Room loading supplies the collision geometry for the Zebes fight and the baby's discovery.
The intro selects the room. The [block owner](world-blocks.md) supplies cell reactions, and
[Samus's movement](samus-movement.md) decides what happens when she reaches a cell.
[The intro](screens-intro.md) owns the background picture and gives the order to open the room.
This covers the collision grid handoff within [room loading](../architecture/domains.md#rooms).

| Quantity | Type and units | Read by |
| --- | --- | --- |
| Room size | Grid width and height in 16 px cells | ShowIntroRoom, movement |
| Collision cells | Row-major sequence of room blocks, with their kinds and tile looks | ShowIntroRoom, movement |
| Cell marks | One mark per cell, which says how a cell meets Samus or a shot | ShowIntroRoom, movement |
| Intro room content | Grid size and a sequence of collision cells from the room's top left | ShowIntroRoom |

Cells: width and height count columns and rows. Each block represents one cell. A neutral
mark has the room's default collision detail and no response to a touch or shot.

### ShowIntroRoom

- kind: mechanic.
- summary: each flashback opens its own room for the demo to play in.
- params: the room size and its collision cells are content.
- switch: Fixed.
- The Zebes fight starts → place its collision cells in a grid of its dimensions from the top left,
  and clear the cell marks. Preserve the collision cells beyond the loaded area [C-fight-room].
- The discovery starts → place its collision cells in a grid of its dimensions from the top left.
  Preserve the marks and the collision cells beyond the loaded area [C-discovery-room].
- A room turn or a cut between turns without a start keeps the room loading state
  [C-room-kept].
- checked: oracle (the room starts at `any-intro-2219:0` and `any-intro-2219:1298`), constructed.

The fight's loaded area is 16 columns by 14 rows within a grid of 16 by 16 cells. The
discovery's loaded area is 32 columns by 12 rows within a grid of 32 by 16 cells.
Both first room turns meet the square floor supplied by those rooms.

## Order that matters

The selected room is available for the demo's first turn. If the cut interrupts the start
before that turn, the room is already visible to the collision reader and the first turn
has yet to run. Keep the same room across that cut.
