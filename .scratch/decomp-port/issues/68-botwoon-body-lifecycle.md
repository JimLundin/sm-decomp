# Botwoon body lifecycle
Status: resolved
Blocked by: 67
Triage: ready-for-agent

Port EA80 main/flash/death delay/fall/landing scheduler with ROM speed data,
explosion sound and slot10 completion flag. Verify actual C and cleanup.

## Completion evidence

Implemented EA80 main script/flash updates, EAF4/EB04 delayed death, EB1F
quadratic falling/landing and EB93 inert state. Landing queues explosion SFX
and sets slot10 completion flag. Independent test checks same-frame delay setup.
833 actual-C full-WRAM/raw-return cases pass (193 allocation/control regressions,
320 preinstructions and 320 scheduler frames):
`out/port/botwoon-body-lifecycle/20261011T054622.255200Z/verdict.json`.
Use `tools/port_botwoon_lifecycle_fixtures.py`. Full port tests, fmt and strict
Clippy pass. Offline coverage; live gameplay ownership remains open.

Cleanup applied/deduplicated all three finished runs in this scope, including
retained flash-alias rejection: zero ordinary bytes, 455,999,488 snapshot bytes
reclaimed. Final verdict authenticated after cleanup. Immutable ROM/source
snapshots retained; no implementer worktree created. 32 GiB available.
