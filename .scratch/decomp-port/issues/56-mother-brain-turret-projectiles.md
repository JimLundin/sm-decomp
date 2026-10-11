# Mother Brain room turret allocation and bullets
Status: resolved
Blocked by: 55
Triage: ready-for-agent

Port BE4F/BF59 allocation, C050 rotation, random timer setters and C0E0 bullet
terrain movement, plus C173/C1B4 script operations. Turret BFDF destruction
requires the unresolved host-side explosion spawn point; remains open under 35.
Verify pinned actual C full WRAM, cleanup and authenticate evidence.

## Completion evidence

Implemented BE4F turret layout/ROM direction and rotation masks/random countdown
initialization; BF59 eight-direction bullet offsets and velocities; C050 bytewise
rotation/reversal; C08E/C0A1 timer setters; C0E0 property toggling and non-air
terrain collision with global active-slot deletion; C173 switch jump and C1B4
palette index clearing. Odd bullet source parameters preserve C array aliasing.
BFDF turret destruction remains unsupported because host-side explosion spawn
state is unresolved in ticket35. This ticket does not claim full turret ownership.

1,281 actual C full-WRAM/raw-return cases pass (272 allocation, 192 bullet
preinstruction, 192 scheduler, 96 rotation, 512 timer and 17 opcode/control):
`out/port/mother-brain-turret-projectiles/20261011T041757.660581Z/verdict.json`.
Rebuild through `tools/port_turret_fixtures.py`. Independent tests cover non-air
hit/global-slot deletion and direction reversal. Full port tests, fmt and strict
Clippy pass. Offline coverage; live execution remains open.

Cleanup: `port_cleanup.py --root out/port/mother-brain-turret-projectiles --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 222,691,328 snapshot
bytes across both finished runs. Initial duplicate-case-name failure retained.
Final evidence authenticated after cleanup; immutable ROM/source dependencies
retained. No implementer worktree created. 34 GiB free disk space.
