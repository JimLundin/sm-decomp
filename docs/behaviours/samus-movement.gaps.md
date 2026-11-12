# Samus moving through the world: integration gaps

Where wiring [the spec](samus-movement.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec. A gap that is answered
leaves this file; its text and answer are in git history.

## Which point of her is inside each cell (filed by the blocks spec)

36. **`TouchedCells`' cells she is inside need their point.** Every point is read: a scroll cell
    reacts only to the cell at her centre, and a cell that holds a place when she is inside it
    holds one at each point (`docs/behaviours/world-blocks.md`, `ScrollCells`, `BlocksMarkedNothing`,
    its header's "What samus-movement must add"). The report gives the cells in order, bottom edge's
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

## The crouched turn aimed down-forward (filed by the observer, from Ceres's free run)

37. **Which way the crouched turn aimed down-forward hands her over.** chain-any-0006-E0B5 534 to
    541: crouched facing left aimed down-forward, right pressed, she turns (535: 9Ah) and skids at
    0 (`TurningRound`, CE4), creeps 0.C000 right at 536 and 538, and on 538 → 539, the pad let
    go, her animation's end hands her over the crouch facing left (539: 28h), not the new facing
    `AnimationEnds` gives for a crouched turn (right, aim kept). The pad then aims (540: 74h) and
    turns her again (541), so the run creeps right while she faces left. The original's crouched
    aimed turns carry the jump turn's movement type (17h, decode notes "The pose map"), and the
    hand-over of 9Ah with the pad let go reads 73h in the decode notes, not 28h; the frame's
    hand-over isn't traced yet. It shows in the free run only once it carries her pose (decode
    notes, "Ceres's free run"); settled by tracing `$91:EB88`'s hand-over for 9Ah on that frame.
