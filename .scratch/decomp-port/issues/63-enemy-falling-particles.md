# Enemy falling particles at 8AAF
Status: resolved
Blocked by: 62
Triage: ready-for-agent

Port 8A39 random enemy-relative allocation and 8A7D movement with unsigned
Y threshold deletion, ROM script selection and canonical RNG. Verify C/cleanup.

## Completion evidence

Implemented 8A39 RNG/enemy-relative allocation, velocity-selected ROM script
and unsigned Y endpoint; 8A7D movement deletes after reaching that endpoint.
Independent Rust test covers unsigned high-bit Y and exact threshold equality.
305 actual pinned C full-WRAM/raw-return cases pass (48 allocations, 128
preinstructions, 128 scheduler frames, sleep control):
`out/port/enemy-falling-particles/20261011T042857.376152Z/verdict.json`.
Rebuild with `tools/port_skree_fixtures.py --falling`. Full port tests, fmt and
strict Clippy pass. Offline coverage; live ownership remains open.

Cleanup applied/deduplicated `out/port/enemy-falling-particles`: zero ordinary
bytes, 41,680,896 snapshot bytes reclaimed. Accepted evidence authenticated after
cleanup; immutable source/ROM snapshots retained. No implementer worktree
created. 32 GiB free disk space.
