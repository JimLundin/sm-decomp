# The intro's playable rooms

## Sources and mapping

For room loading, the importer and the replay decoder. The spec is
[The intro's playable rooms](world-intro-rooms.md).

| Entry | Original source | Record mapping |
| --- | --- | --- |
| ShowIntroRoom | The fight inherits the 16 × 16 shape set by `$8B:A395`. `$8B:AF21..AF2E` copies 224 words from `$8C:BEC3`. `$8B:AF30..AF3F` clears 512 marks. | Grid width and height at `$7E:07A5` and `$7E:07A7`. The first cell word is at `$7F:0002` and the first mark at `$7F:6402`. |
| ShowIntroRoom | `$8B:AF83..AF8C` sets the discovery shape to 32 × 16. `$8B:AFDF..AFEF` copies 384 words from `$8C:C083`. It leaves marks as they were after the fight. | Same grid and base addresses. The first 512 cell words and their marks suffice to check the retained tail and both loads. |

[Bank 8B](https://patrickjohnston.org/bank/8B) · [Bank 8C](https://patrickjohnston.org/bank/8C)

Both source ranges give cell words without a length header or a mark table. Import them
through the existing block-word decoder with neutral marks. Use
`data::level::Block::new` and the existing typed collision conversion to make
`production::world::MovementWorld`. The populated rectangles have no stand-ins,
reacting cells or doorways. There are no solid actor boxes supplied by room loading.
The fight and discovery encounter owners supply their own contacts.

The marks and the cells beyond each load belong to the platform's memory image. Borrow
the completed text cut `any-intro-0568:1650` only as the initial memory image for an
independent scoped room test. Compare the inherited tail against the same carried image.
The test must account for the borrow and must exclude the inherited tail from a claim
about imported content. The movie body bounds while visible are x 143–209, y 46–136 px
in the fight, and x 35–381, y 126–168 px in discovery. The fight's active missiles stay
above the bottom of its populated rectangle. A scoped typed collision provider can use
the populated rectangles and expose a query leaving them as a boundary. The raw tail
contains cells left from before the intro and needs the opening's memory image producer
if the integrated replay declares it. Borrow no record State in integrated play.

## First room turns

`any-intro-0568:1650` is movie cut 2,218, the finished text turn that latched the movie
press. `any-intro-2219:0` is movie cut 2,219, an interrupt after the fight load and before
Samus's room turn. Cut 1 is movie cut 2,220, its completion. The discovery's equivalent
sequence is row cuts 1297, 1298 and 1299, movie cuts 3,516, 3,517 and 3,518. Use the change
in `$7E:05B6` to distinguish completed turns. Continue a pending turn through a second NMI
as one turn. The intro emits the room start before the demo and movement, then the encounter
reads the result. A cut with pending setup can already show new geometry.

| Quantity | Fight setup cut 0 | Discovery setup cut 1298 | At the next finished cut |
| --- | --- | --- | --- |
| Centre x and y | (155.0000, 115.0000) px | (376.2000, 147.FFFF) px | Fight y becomes 115.FFFF. Discovery keeps both components. |
| Previous centre x and y | (155.0000, 115.0000) px | (376.0000, 147.0000) px | Unchanged. |
| Pose and prior pose | Standing left, older pose still the text pose | Standing left, older pose also standing left | Older pose becomes standing left in fight. |
| Body radius | (0, 0) at setup | (5, 21) px | (5, 21) px in both. |
| Standing clip | Step 0, hold 10 | Step 0, carry hold 3 | Hold 9 in fight and 2 in discovery. |
| Selected item and supplies | Missiles, carry 900 and energy 99 | None selected, carry 896 missiles and energy 99 | Unchanged. |
| Demo pad | Neutral held and new | Neutral held and new | Neutral held and new. |

The position fractions in the table are hexadecimal fractions of a pixel. Both starts
set only the whole coordinates for current and previous centre. The fight starts with
empty shot slots, zero shot cooldown, no invulnerability and no knockback. Both first
turns have no shot or hit. The setup loads pose and clip through `$91:F433` and `$91:FB08`.
Discovery enters the standing pose it already had and carries its clip. Do not recreate
the movement, appearance or weapons state for discovery. Keep each owner's carried state
and write the room's centre and selection. At `$8B:A373`, the occupied room takes its demo
and movement turn. The original keeps the whole previous centre written at setup. The
typed movement State does not read that mirror.

The positive downward probe from standing reaches cell (9, 8) in the fight and cell
(23, 10) in discovery, measured in cells from the top left. Both cells are square slope
shape 0. [FollowingTheGround](samus-movement.md#followingtheground) already supplies
the probe and flush stop. Use the ordinary 1 px ground pull, including when the room is
on the cinematic path. The blocking half starts at y = 136 px in the fight and y = 168 px in discovery.
The scene's background offset (0, 8) is a draw transform. It
does not move Samus or the collision grid. Keep the intro palette, picture and sprite
ordering comparisons in their respective owners.

| Projection | Addresses |
| --- | --- |
| Current and previous centre | `$7E:0AF6..0AFC`, `$7E:0B10..0B16` |
| Pose history and radius | `$7E:0A1C..0A26`, `$7E:0AFE..0B00` |
| Appearance hold and step | `$7E:0A94`, `$7E:0A96` |
| Energy, missiles, selection and shot activity | `$7E:09C2`, `$7E:09C6`, `$7E:09D2`, `$7E:0CCC`, `$7E:0CCE` |
| Demo held and new presses | `$7E:0A84`, `$7E:0A86` |

## Replay evidence

The reference loads collision content from the ROM and carries the geometry from the
finished text cut. It compares both dimensions, 512 cell words and 512 marks, with no
mask, at every cut of `any-intro-2219:0..2103` and `any-intro-4323:0..128`. It matches
1,298 of 1,298 fight and page 2 cuts, including five interrupted cuts, and 935 of 935
discovery cuts, including three interrupted cuts. Holding the finished text cut's
geometry matches zero cuts. Three constructed cases exercise both loads and the carry.
The movie reaches discovery with neutral marks after the fight cleared them.
`C-discovery-room` adds a nonneutral mark inside the overwritten cells. A discovery
fault that clears only the overwritten cells' marks differs at that mark.

Delaying the fight start until its first completed turn differs at `any-intro-2219:0`
in the loaded cells. Omitting its last loaded cell and clearing the inherited tail also
differ there, in distinct cells. Preserving the fight's incoming marks differs there
in the marks. Keeping the fight's width on discovery differs at `any-intro-2219:1298`
in the width.

The first-turn reference takes the prior completed cut's actor State at each join. It
compares 15 actor fields at each of the four setup and completed cuts. The setup and
first-turn model matches all four. A frozen prior actor matches zero. Moving early fails
on the fight's centre at cut 0 and discovery's clip hold at cut 1298. Skipping the first
turn fails on the same fields at cuts 1 and 1299, respectively. Zeroing discovery's
fractions or restarting its standing clip fails at cut 1298. The actor check witnesses
the join only. It takes the completed cut before discovery as input and does not
reconstruct the intervening room play.

The indexed fight and discovery cases each compare the interrupted installation and the
first completed turn through the production room adapter. Delaying installation first
changes the fight's cell 0 at movie frame 2,219 and discovery's width at 3,517. Each case
also restores the pending installation and completes the first turn from that checkpoint.
