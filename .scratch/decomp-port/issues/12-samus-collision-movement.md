# Complete Samus collision and movement closure
Status: claimed
Blocked by: 05, 08
Triage: ready-for-agent

Translate bank94 complete collision traversal/reactions/slopes, bankA0 solid-enemy
collision, then bank90 movement controllers and dispatcher. Preserve real PLM
callback semantics through ticket11, room/BTS/door data, fixed-point amount and
directional effects. Scope context: /tmp/sm-port-collision-closure.md. Own
collision.rs and tests/contracts. Unknown tiles/callbacks cannot return defaults.
Differential valid synthetic/captured cases plus exercised live behavior required.

## Completion evidence

Implementation: `0d753e2` (collision.rs), integration tip merged before handoff.
63 public routines include all 28 normal movement handlers and dispatcher,
terrain traversal/reactions for all 16 tile types, square/nonsquare slopes,
directional solid-enemy collision, wall/spin/bomb/grapple movement and footsteps.
PLM setup remains explicit counted C callback through ticket 11, with independent
full WRAM transactions and displacement transferred both ways.

Four independent synthetic Rust tests pass with `rustc -D warnings`, covering
solid wall clamp/fraction, synchronous PLM displacement mutation, active enemy
contact, and original `0xffff` enemy collision flag. Actual upstream full-WRAM,
return and callback equivalence: 63 routines / 1,752 cases at
`/tmp/sm-collision-final/20261010T201630.417763Z/verdict.json`.
Focused acceptance: all 16 tile categories in both axes (32 cases), all 28
movement handlers (448 cases), four enemy directions/frozen gates (8 cases).
Deliberate wall clamp +1 rejected at
`/tmp/sm-collision-reject/20261010T201702.912994Z/verdict.json`.
Failures retained: initial shifted enemy base and incorrect boolean collision
flag at `/tmp/sm-collision-focused/20261010T201356.809185Z` and
`/tmp/sm-collision-enemy-focused/20261010T201509.789890Z`; actual source layout
and signed collision flag corrected before acceptance.

Exact committed manifest and latest shared dependencies passed all 1,752 cases at
`/tmp/sm-collision-committed/20261010T201757.945722Z`. Live integration is root's
remaining ticket gate; offline evidence alone does not resolve this ticket.
