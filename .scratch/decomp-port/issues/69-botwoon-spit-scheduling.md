# Botwoon spit scheduling
Status: resolved
Blocked by: 68
Triage: ready-for-agent

Port EC05 paired movement and inclusive camera culling. Verify actual C full
WRAM and complete run cleanup. Spit allocation/angle conversion remain open.

## Completion evidence

Implemented EC05 paired movement then inclusive camera culling. Independent
Rust test checks survival at X=256 and removal beyond it.
833 actual pinned C full-WRAM/raw-return cases pass (321 helper/control
regressions, 256 preinstructions, 256 scheduler frames):
`out/port/botwoon-spit-scheduling/20261011T054736.141194Z/verdict.json`.
Rebuild with `tools/port_botwoon_spit_fixtures.py`. Full port tests, fmt and
strict Clippy pass. Spit allocation/angle conversion and live ownership remain open.

Cleanup applied/deduplicated `out/port/botwoon-spit-scheduling`: zero ordinary
bytes, 117,833,728 snapshot bytes reclaimed. Accepted evidence authenticated
after cleanup; immutable source/ROM snapshots retained. No implementer worktree
created. 31 GiB available disk space.
