# Complete Samus collision and movement closure
Status: resolved
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

Integrated source seal: `out/port/collision-final/20261010T202435.794119Z/verdict.json`.
The four-process 9,000-frame run at
`out/port/collision-plm-live/20261010T203112.471878Z` matched state/effect and
progress comparisons, executing native normal movement 2,108 times and five
other newly installed collision/instruction paths. The run rejects its additional
native PLM admission because every process had zero PLM calls. It must not be
reported as PLM live acceptance. An 18,000-frame replay is investigating that
execution boundary; ticket remains claimed pending the required integration gate.


Final exercised admission: cumulative batch `20261010T213631.753519Z`, live
verdict `replays/20261010T215856.742725Z/evidence/20261010T215856.841221Z/verdict.json`.
Four independent18,000-frame C/mixed processes match periodic hardware/game
checkpoints, full terminal WRAM/SRAM, per-call before/after states, raw returns
and ordered PPU/APU effects. Collision executes12,021 Rust calls across5 symbols,
including10,791 normal movement dispatches,996 right movements,212 pose-change
collision checks and11 each vertical speed and bomb-jump rising movement calls.
This resolves the exercised collision/movement integration gate. Terrain PLM
callbacks remain zero; native PLM admission is not claimed.
