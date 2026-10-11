# Walking lava seahorse fireball lifecycle
Status: resolved
Blocked by: 34
Triage: ready-for-agent

Translate 9EB2 initialization and 9EFF flight through allocation/frame dispatch.
Preserve enemy facing, three trajectory choices, vertical-first collision and
signed horizontal deceleration clamps. Verify pinned C and clean runs.

## Completion evidence

Implemented facing-dependent spawn coordinates/velocity, three Y trajectories
and vertical-first collision/deceleration through public allocation and frames.
241 actual pinned C full-WRAM/raw-return cases pass (48 spawn, 96 preinstruction,
96 scheduler and one sleep control) in
`out/port/seahorse-fireball/20261011T035228.955246Z/verdict.json`.
Independent Rust test proves floor contact skips horizontal movement/deceleration.
Full port suite, fmt and strict Clippy pass. Offline fixture equivalence only;
live seahorse enemy/projectile ownership remains unproven.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 69,337,088 snapshot bytes. No implementer
worktree created. Immutable ROM/source/run dependencies retained, final evidence
authenticated after cleanup; 35 GiB free.
