# Golden Torizo eye beam
Status: resolved
Blocked by: 49
Triage: ready-for-agent

Translate B328 allocation and B38A collision response. Preserve three RNG calls,
facing ROM table, final sine velocity and horizontal-first impact scripts.
Verify actual C spawn/preinstruction/frame seams and clean runs.

## Completion evidence

Implemented eye-beam facing table spawn with three RNG calls and final sine
velocities, plus horizontal-first collision/script response and floor alignment.
193 actual pinned C full-WRAM/raw-return cases pass (64 allocation, 64
preinstruction, 64 frame, one sleep control) in
`out/port/torizo-eye-beam/20261011T040521.710256Z/verdict.json`.
Full port suite, fmt and strict Clippy pass. Offline fixture equivalence only;
live execution/full Torizo ownership remain open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 56,754,176 snapshot bytes. No implementer
worktree created. Immutable ROM/source/run evidence retained; final verdict
authenticated after cleanup. 35 GiB free.
