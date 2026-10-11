# Draygon turret and gunk scheduling
Status: resolved
Blocked by: 64
Triage: ready-for-agent

Port power-bomb collision and 8DFF/8E0F scheduling using paired movement.
Preserve movement after collision deletion and gunk proximity script transition.
Verify actual C and clean evidence. Allocation/attached gunk remain open.

## Completion evidence

Implemented 8D5C power-bomb rectangle collision and 8DFF turret/8E0F gunk
scheduling using paired movement. Preserves movement after ID deletion,
Samus speed-divisor clearing, signed proximity tests and 8C38 script transition.
Independent Rust test checks post-deletion movement.
897 actual pinned C full-WRAM/raw-return cases pass (321 paired helper/control
regressions, 192 preinstructions, 192 scheduler frames, 192 bomb collisions):
`out/port/draygon-projectile-scheduling/20261011T043123.801594Z/verdict.json`.
Rebuild with `tools/port_draygon_fixtures.py`. Full port tests, fmt and strict
Clippy pass. Allocation, attached gunk and live ownership remain open.

Cleanup applied/deduplicated `out/port/draygon-projectile-scheduling`; measured
reclamation and final authentication recorded below. Immutable source/ROM
snapshots retained; no implementer worktree created.
Zero ordinary bytes and 170,262,528 snapshot bytes reclaimed. Accepted verdict
authenticated after cleanup; 32 GiB available disk space.
