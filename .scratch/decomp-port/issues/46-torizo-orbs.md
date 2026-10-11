# Torizo Chozo orb spawning and bounce lifecycle
Status: resolved
Blocked by: 45
Triage: ready-for-agent

Translate ABEB/AC7C initializers, ACAD/ACFA collision and AD92 motion/branch.
Preserve table/RNG ordering, wall reflection, floor bounce and script pointers.
Verify actual pinned C allocation/instruction/frame seams and clean runs.

## Completion evidence

Native Bomb/Golden Torizo orb allocation uses facing-dependent ROM tables and
two ordered RNG calls. Collision paths preserve wall reflection, floor damping,
settled script transitions and gravity. AD92 preserves full fixed-point X motion
and signed branch pointer. 384 actual pinned C full-WRAM/raw-return cases pass
(96 allocation, 32 instruction, 128 preinstruction, 128 scheduler) in
`out/port/torizo-orbs/20261011T035830.629421Z/verdict.json`.
Independent Rust test covers wall reflection then settled floor bounce. Full port
suite, fmt and strict Clippy pass. Offline fixture equivalence only; live open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 116,523,008 snapshot bytes. No implementer
worktree created. Retain script ROM `out/port/torizo-orb-fixtures/orb.sfc` and
immutable source/run evidence; final verdict authenticated after cleanup.
35 GiB free.
