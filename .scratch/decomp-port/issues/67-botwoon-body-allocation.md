# Botwoon body allocation
Status: resolved
Blocked by: 66
Triage: ready-for-agent

Port EA31 body allocation, ROM script selection, linked enemy record and flags.
Verify actual C and cleanup. Botwoon lifecycle scheduling remains open.

## Completion evidence

Implemented EA31 enemy-zero position copying, direction-selected ROM script,
E/F script state, EA98 movement state, linked 7800 body slot/health marker and
projectile flag2. Enemy linkage validated before allocation writes.
193 actual pinned C full-WRAM/raw-return cases pass (192 allocations including
full pool and varied source/linkage records, sleep control):
`out/port/botwoon-body-allocation/20261011T043444.688664Z/verdict.json`.
Rebuild with `tools/port_botwoon_fixtures.py`. Full port tests, fmt and strict
Clippy pass. Botwoon lifecycle scheduling and live ownership remain open.

Cleanup applied/deduplicated both finished runs in `out/port/botwoon-body-allocation`:
zero ordinary bytes, 57,278,464 snapshot bytes reclaimed. Final accepted evidence
authenticated after cleanup; immutable ROM/source snapshots retained. No
implementer worktree created. 32 GiB available disk space.
