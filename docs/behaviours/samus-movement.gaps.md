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

## Directional travel outside the normal interval

38. **A skipped PRE needs independent receipt-write information.** Normal PRE runs the projectile
    pass, then clears the interval; normal POST stores the complete fresh `Outcome::travelled`.
    The engine retains caller-held `TravelledDistances` as a pinned model State. The fresh map
    gives zero both for an unwritten direction and an ordinary move that accepted zero. A caller
    that runs movement without PRE therefore cannot merge it into a retained interval honestly.
    Such a caller needs the set of ordinary directions written, including zero writes, or a
    movement entry point taking the retained interval. No recorded skipped-PRE sequence is
    claimed here. Weapon inheritance and the caller variants outside the engine's normal and
    elevator-carried PRE paths remain unwired; the directional harness checks named end cuts.

## External pose selection and interrupted POST

Pending publication is proven only where the ordinary pose initializer runs. The harness
names priority hit selection, sand-forced spin landing and independently validated
pre-POST cuts as external boundaries in the [decode notes](samus-movement.decode.md#pending-publication-at-external-pose-boundaries).
The engine still needs damage and sand-contact inputs to replace those writers; its
general locked modes and special control-restoration paths remain unwired.

## Accepted contact and performed work consumers

The public movement output publishes accepted floor responses, ordered touchdown cue
requests and actual performed leading-edge collision tests. The checked ordinary Ceres
caller forwards them before subsequent appearance opportunities. Its sealed capability
proves cinematic activity inactive. Wider legacy callers lack the actual cinematic input;
they retain movement while discarding the new producer outputs. Audio queue/sending and
CPU timing costs remain separate consumer obligations.

Prior selected Sprite is imported from the existing selected figure, independently of
current equipment. Rejected pose fit suppresses deferred landing/wall-kick requests;
a changing-body floor stop retains the speed before its response. Native query contexts
remain independent expectations, never production input.
No reachable mixed-condition horizontal frame or movement ordering
correction is established by this prefix.

Samus-appearance part 2 owns admission of landing dust/splashes from the published graphical
opportunity. Ceres deletes its atmospheric landing slots; general room/liquid graphical
admission remains outside the movement output delta. CPU workload prediction outside the
native prefix's ordinary moves, aim moves and fit looks remains the timing owner's gap.

## Movement decisions and their timing consumers

The movement contract publishes actual requests, eligible candidates, ordered floor/ceiling
support and actual selected-body fit considerations. Their production implementation and
CPU-timing consumption must preserve complete geometry and performed order; no elapsed
costs are supplied by movement. Fifteen constructed full-entry delta witnesses are proven.
Native complete attempt/candidate/support receipts remain absent, and the existing query
list cannot prove no-attempt or no-effect admission. Mixed/stepped complete fit receipts
remain unproved; existing movement behaviour and leading-edge vectors remain required.

**Missing caller roster writer and timing consumer.** The actor caller owns beginning-of-frame
interactive membership and current frozen/solid eligibility. Movement receives ordered eligible
boxes only. Selection and tests on excluded actors consume original work outside this bounded
receipt; the caller must publish it and CPU timing must consume it before full enemy-work
prediction is claimed. Empty supplied candidates do not prove absent roster work.

The caller preserves the raw admitted interactive order, current frozen/solid properties and
an eligible-ordinal-to-member mapping. Original filtering occurs at each actual request.
Its receipt includes excluded members in the reached prefix; exhaustion includes the terminator,
and a first accepted stop excludes its following suffix and terminator. Eager filtering once
when constructing the world's eligible boxes is a distinct operation. CandidateCount does not
recover repeated scans or excluded members. Neither raw roster nor eligibility becomes a new
clean movement input; native complete roster coverage remains0.

**Seating cleared after an enemy-blocked ground pull.** The current ordinary vertical movement
clears seating solely from nonzero requested travel. An enemy stop skips terrain and must retain
the seat. The complete constructed slope witness starts unseated, is seated by horizontal
support, then touches an enemy during the ground pull and ends seated. The unconditional-clear
fault loses that field. The next model must clear only when ordinary vertical terrain is reached.
