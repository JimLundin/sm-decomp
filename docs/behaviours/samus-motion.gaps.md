# Samus moving through the world: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec. A gap that is answered
leaves this file; its text and answer are in git history.

## Which point of her is inside each cell (filed by the blocks spec)

36. **`TouchedCells`' cells she is inside need their point.** Every point is read: a scroll cell
    reacts only to the cell at her centre, and a cell that holds a place when she is inside it
    holds one at each point (`docs/behaviours/blocks.md`, `ScrollCells`, `BlocksMarkedNothing`,
    its header's "What samus-motion must add"). The report gives the cells in order, bottom edge's
    row first, then her centre's if its row differs, then her top edge's under the `xor` rule, but
    not which point each is, and the order alone can't say: with her centre in her bottom edge's
    row the one cell is her bottom edge's, not her centre's, and the cell after the first may be
    her centre's or her top edge's. On record: item-charge-beam 139, 140, 141, 815 and 816, where
    the oracle spawns the scroll touch from her centre's point (`$94:995E`) for the scroll cell
    (61, 87). Asked: each inside cell comes with its point, `Top`, `Centre` or `Bottom` (listed top
    to bottom), in the report's order, a cell reported twice once per point, and a stand-in already
    resolved to the cell it stands for, as for her touches (the inside handling follows stand-ins,
    `$94:9B40`). The blocks spec's vectors give `{ inside = [x, y], point }` requests in that
    shape.

## Heading down against the records (filed by round 18's integration)

The harness compares `heading down` with `0B36` = 2 at N+1 where the pose change writes no
direction (the decode notes' "For the harness"): 17,921 of 17,922 frames match. One kind of frame
disagrees, the model heading down where the original's direction after her moves is none. The
harness masks it, by name, until it is answered.

38. **A change of shape's landing clears her direction in her moves.** Movement type 0Fh
    (crouching, morphing, standing up, unmorphing) with a direction at N and none at N+1 (on
    each, `0B36` 2 at N and command 0 at N+1): the change of shape's handler moves her by her
    vertical speed while she has a direction and clears it as she lands (`$90:A61C`, `$90:A640`),
    in the movement handler, before the animation reads it. The model reports her heading down
    there (she keeps the downward speed on the ground). 15 frames (records N+1):
    boss-kraid-defeated 692, boss-phantoon 745, chain-0043-9AD9 393, chain-0057-9FBA 332,
    chain-0097-ACF0 17, chain-0314-DAE1 24, item-charge-beam 90, 754, 866, item-morph-ball 1265,
    item-reserve-tank 33, 564, 1116, 1148, samus-mockball 71. Asked: `RisingAndFalling`'s heading
    down for a change of shape that lands in its moves.

One more frame disagrees and is not masked: samus-grapple-walljump 115, a wall jump's rise
with jump let go (the pad at 114 is 0000). Wall jumping runs the rise check that zeroes the speed
and sets direction down when jump isn't held (`$90:A734` → `$90:8FB3`, `$90:8FDC`..`$90:8FF9`);
`RisingAndFalling`'s arm covers a normal or spin jump and the spring ball's hop, not the wall
jump, so the model is still rising. The frame misses on y and her vertical motion too.
