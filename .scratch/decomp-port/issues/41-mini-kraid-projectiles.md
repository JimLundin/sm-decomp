# Mini-Kraid spit and spikes
Status: resolved
Blocked by: 40
Triage: ready-for-agent

Translate spit/spike initializers 9DEC/9E46/9E4B and preinstructions 9E1E/9E83.
Preserve extra enemy RAM aliases, bounded spike height lookup, signed velocity
clamp and collision short circuit. Verify allocation and scheduled frames against
actual pinned C, then clean runs.

## Completion evidence

Implemented spit and left/right spike initialization with canonical extra enemy
RAM 7800/7802/7804/780C aliases and validated spike-height index. Spit preserves
collision short circuit and signed acceleration/clamp; spikes delete on horizontal
collision. Public allocation/preinstruction/frame dispatch owns these paths.

Actual pinned C full-WRAM/raw-return evidence passes 401 cases (144 allocation,
128 preinstruction, 128 full-frame and one sleep control) at
`out/port/mini-kraid-projectiles/20261011T034954.947189Z/verdict.json`.
Independent Rust test verifies positive speed clamp versus negative signed wrap.
Full port suite, fmt and strict Clippy pass. Offline fixture equivalence only;
live execution and full Mini-Kraid enemy ownership remain unproven.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 125,435,904 snapshot bytes. No implementer
worktree created. Immutable ROM/source/run evidence retained, final verdict
authenticated after cleanup; 35 GiB free.
