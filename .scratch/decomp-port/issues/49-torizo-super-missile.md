# Golden Torizo super missile tracking and aim
Status: resolved
Blocked by: 48
Triage: ready-for-agent

Translate B1CE spawn, B20D tracking, B269/B272 aim and B237 flight.
Preserve enemy-slot alias, ROM facing script, angular half-plane and upward
collision behavior. Compare actual pinned C and clean runs.

## Completion evidence

Implemented enemy-relative spawn/script selection, saved-slot tracking, both
angular aiming opcodes and collision/gravity flight. 368 actual pinned C
full-WRAM/raw-return cases pass (48 spawn, 64 aim, 128 preinstruction, 128 frame)
in `out/port/torizo-super-missile/20261011T040402.516130Z/verdict.json`.
Independent Rust test verifies saved enemy slot versus current slot. Full port
suite, fmt and strict Clippy pass. Offline fixture equivalence only; live open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 104,333,312 snapshot bytes. No implementer
worktree created. Retain ROM `out/port/torizo-missile-fixtures/missile.sfc` and
immutable source/run evidence; final verdict authenticated after cleanup.
35 GiB free.
