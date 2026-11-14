# Uncharged power-beam integration gaps

The [model](weapons-projectiles.md) has an independently checked record harness and domain
free run. [Decode notes](weapons-projectiles.decode.md) establish the ordinary M0 inputs and
per-enemy consumption boundary. The engine still runs the old projectile path; its whole-M0
gain remains zero. Wiring waits for these named neighboring outputs (PROCESS step 9).

| Missing output | Existing producer and required next change |
| --- | --- |
| Caller lifetime and consumption of directional travel | `Outcome::travelled`, available through `Ran::outcome`, supplies all four signed collision-returned receipts with their fractions. The engine persists `TravelledDistances` in `State::movement_travel` through a pinned canonical model; its checkpoint tests preserve complete values and named direction order. Normal and elevator-carried PRE clear after the projectile pass, and normal POST stores fresh ordinary receipts. Seven named movement end cuts verify the receipts. Movement under a skipped PRE still needs a direction-write receipt to distinguish unwritten directions from accepted zero; broader caller lifetime and W1 cannon consumption remain unproved/unwired (samus-movement gap 38). Net centre displacement or final velocity cannot substitute. |
| Retained vertical fractional distance from Samus to the remembered centre | `world_camera` exposes position/seen and placement, but no retained receipt. Add this output or prove an adapter derivation, including borrow, pose placement and the paths that retain it. It is independent of clamped camera travel. |
| Pending aim from the pose change | The movement/pose outcome lacks the transition's named firing heading. Add and persist it; consume the model's pending-aim event and clear it at the caller's documented point. |
| Previous newly pressed shoot | `Controller::previous` is held-button history. Preserve the drawing phase's newly pressed controls for the following PRE cannon, including special handlers that restore controls. |
| Beam-path eligibility and stopped-time/update mode | Model the selection producer and neighboring modes. Item icon selection alone cannot decide beam eligibility. The harness rejects selection-button activity and unproved caller paths. |
| Owned-shot consumption before each enemy's AI | C1's enemy run does not supply this event. Its current selector receipt and the enemy's pre-AI contact must produce a consumed slot after W1 update. Earlier-enemy cross-effects, multibox contact and special responses need their own proven contract. |
| Actual room/elevator reset event | R1 must replace W1 State while preserving slot fractions. DoorLoading alone is not a reset event; the harness masks only Free trails under the documented M0 contract and rejects live passage/reset cases. |
| Complete ordered contact continuation | W2 must resolve limited-row/width aliases, overlapping words, continuation past the level and all reactions in visit order. The harness reads available continuation and accepts only unconditional Air; other reactions or unreadable data are defects. |

AF3's production importer already supplies complete power-beam content, muzzle descriptors and
sprites. Add that content to the runtime `Content`, hold the clean State through
`canonical::model::Model<P>` with a pinned-shape test, and replace the power-beam path once the
outputs above exist. Send fire/protection/pending/trail events to their actual consumers and
place flying/impact picture producers through the sprite encoder in global order, using the
presentation camera origin and preceding-cut flicker phase. Whole OAM bytes, sound/protection
and trail effects remain integration checks; State matches or low-OAM presence do not prove them.

Live C1 consumption is admitted only for the five independently watched preceding M0 records,
identified by their full decoded WRAM hash, frame and NMI-return cut, with enemy slots 0 then 1.
Full input identity preserves the earlier actors' possible AI and species-response dependencies;
room, species or selected fields cannot prove that boundary. A following expected record never
authorizes it. Wider live C1 cases are defects until their actual pre-AI receipts are proved.
The source-proven count-zero or no-Flying case performs no consumption search.

Unresolved wider-row differences remain visible in the committed floors and coverage. The
16 State-field harness has no after-shot outputs in its inputs, and the null run receives the
same masks. Its R1 masks end free segments; aggregate held frames are not an uninterrupted run.
