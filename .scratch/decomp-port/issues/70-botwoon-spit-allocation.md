# Botwoon spit allocation
Status: resolved
Blocked by: 69
Triage: ready-for-agent

Port EBC6 enemy-relative allocation with canonical byte angle and quarter-circle
ROM velocity conversion. Verify actual C and cleanup. Live ownership open.

## Completion evidence

Implemented EBC6 enemy-relative allocation with byte angle 060B and unsigned
A0:B7EE quarter-circle products split into high/fraction fields. Independent
read-only subagent confirmed table and aliases; Rust test checks poisoned angle
high byte and magnitude split. Parallel Yapping Maw allocation tracked in71.
257 actual-C full-WRAM/raw-return cases pass (256 angle/speed allocations and
sleep control): `out/port/botwoon-spit-allocation/20261011T055009.845014Z/verdict.json`.
Use `tools/port_botwoon_spit_allocation_fixtures.py`. Final combined full port
tests, fmt and strict Clippy pass. Live ownership remains open.

Cleanup applied/deduplicated `out/port/botwoon-spit-allocation`: zero ordinary
bytes, 34,603,008 snapshot bytes reclaimed. Accepted evidence authenticated after
cleanup; immutable source/ROM snapshots retained. No worktree created.31GiB free.
