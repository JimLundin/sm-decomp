# Yapping Maw body allocation
Status: resolved
Blocked by: None
Triage: ready-for-agent

Port EC62 body allocation with orientation-dependent script and the exact
wrapped extra-enemy-RAM body-slot linkage. Compare actual C across allocation
slots, enemy slots, both orientations and linkage wrap boundaries.

## Completion evidence

EC62 copies enemy coordinates, selects EC56/EC5C from parameter2 and writes
the allocated projectile slot through the wrapped 16-bit extra-enemy-RAM
index. Source integrated by the parent alongside Botwoon allocation.
385 actual-C full-WRAM/raw-return cases pass:
`out/port/yapping-maw-body/20261011T055011.426077Z/verdict.json`.
Fixtures cover all 32 enemy slots, orientation variants, pool exhaustion and
wrapped linkage indices. Rebuild with `tools/port_yapping_maw_fixtures.py`.
Independent Rust allocation example verifies both scripts and linkage wrap;
`cargo test -p port --test yapping_maw --quiet` and formatting pass.

Cleanup applied and deduplicated `out/port/yapping-maw-body`: zero ordinary
bytes and 50,462,720 snapshot bytes reclaimed. Evidence authenticated after
cleanup; immutable evidence dependencies retained. No implementer worktree
created. 31 GiB available. Offline coverage; live gameplay remains open.

Full port tests, formatting and strict Clippy passed in the parent integration
verification. Commit contains this ticket, reproducible fixtures and independent
test; combined Rust implementation is committed by the parent.
