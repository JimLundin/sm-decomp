# Torizo drool impacts and explosion placement
Status: resolved
Blocked by: 44
Triage: ready-for-agent

Translate A81B/A871 explosion initializers, A3BE position restore and A887 drool
collision response. Preserve shared spawn-parameter mutation, facing/table bounds,
velocity near-zero quirks and impact scripts. Verify actual C and clean runs.

## Completion evidence

Implemented low-health/death explosion placement, E/F position restore opcode
and drool collision response. Preserve facing-dependent shared parameter mutation,
wrapping table offsets, horizontal collision short circuit, velocity-three quirk,
vertical impact scripts and speed-range deletion.
292 pinned actual C full-WRAM/raw-return cases pass (96 allocation, 96
preinstruction, 96 scheduled frame, four restore) in
`out/port/torizo-impact/20261011T035659.313784Z/verdict.json`.
Independent Rust test verifies the velocity-three behavior. Full port suite, fmt
and strict Clippy pass. Offline fixture equivalence only; live ownership open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 88,604,672 snapshot bytes. No implementer
worktree created. Retain script ROM `out/port/torizo-impact-fixtures/impact.sfc`
and immutable source/run evidence; final verdict authenticated after cleanup.
35 GiB free.
