# Shaktool attack circles
Status: resolved
Blocked by: 54
Triage: ready-for-agent

Port BDA2/BD9C allocation and BE03/BE12 collision scheduling, preserving
angle offsets, linked slot semantics and front-circle horizontal short circuit.
Verify actual C full WRAM, scheduler and allocation; clean before resolution.

## Completion evidence

Implemented BDA2/BD9C angle/sine/octant allocation, BE03 front-circle collision
short circuit and BE12 linked-circle movement/lifetime. Odd references preserve
C's E >> 1 alias; invalid out-of-pool references return an error. Added the
84FB no-op preinstruction used by middle/back circles before real script setup.

1,159 actual pinned C full-WRAM/raw-return cases pass: 768 allocations covering
all 256 angles for all three definitions, 192 collision/lifetime preinstructions,
198 scheduler cases including six original-ROM script transitions, and one sleep
control. Verdict:
`out/port/shaktool-attack-circles/20261011T041528.470999Z/verdict.json`.
Rebuild fixtures through `tools/port_shaktool_fixtures.py`. Independent tests
check front horizontal-hit short circuit, both linked checks, odd alias and
reference-death expiration. Full port tests, fmt and strict Clippy pass.
Offline verification; live gameplay ownership remains open.

Cleanup: `port_cleanup.py --root out/port/shaktool-attack-circles --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 672,792,576 snapshot
bytes across both finished runs. Final evidence authenticated after cleanup.
No implementer worktree created; immutable source/ROM snapshots retained.
34 GiB available disk space.
