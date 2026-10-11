# Projectile paired 16.16 movement
Status: resolved
Blocked by: 63
Triage: ready-for-agent

Port E73E paired high/fraction velocities with angle sign selection and E722
fixed 512-square extent predicate used by Draygon. Verify actual C and cleanup.

## Completion evidence

Implemented E73E paired high/fraction velocity movement with angle-driven
complete 32-bit negation and wrapping position addition. Added E722 fixed
512-square predicate. Independent Rust test checks negative fractional velocity
borrow on both axes. These helpers support Draygon; full Draygon preinstruction
collision/ownership is not claimed.

321 actual pinned C full-WRAM/raw-return cases pass (256 angle movement,
64 extent combinations and sleep control):
`out/port/projectile-paired-motion/20261011T043003.873949Z/verdict.json`.
Rebuild with `tools/port_paired_motion_fixtures.py`. Full port tests, fmt and
strict Clippy pass. Offline coverage; live gameplay ownership remains open.

Cleanup applied/deduplicated `out/port/projectile-paired-motion`: zero ordinary
bytes, 50,724,864 snapshot bytes reclaimed. Accepted evidence authenticated
after cleanup; immutable source/ROM snapshots retained. No implementer worktree
created. 32 GiB available disk space.
