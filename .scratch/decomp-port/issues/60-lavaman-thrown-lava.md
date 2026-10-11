# Lavaman thrown lava
Status: resolved
Blocked by: 59
Triage: ready-for-agent

Port E000 allocation and E049/E050/E07A horizontal scheduling with inclusive
camera bounds. Verify pinned C full WRAM, clean and authenticate evidence.

## Completion evidence

Implemented E000 enemy position/fraction allocation, left/right scripts and
E050/E07A state selection with -768/+768 horizontal velocity fields. E049
scheduling shares exact 16.16 X movement and inclusive camera bounds with
DBF2/Nami-Fune; direct E050/E07A preinstructions supported.

385 actual-C full-WRAM/raw-return Lavaman cases pass:
`out/port/lavaman-thrown-lava/20261011T042411.986884Z/verdict.json`.
Shared allocation regression passes all 769 Nami/Fune/DBF2 cases:
`out/port/lavaman-horizontal-regression/20261011T042439.060620Z/verdict.json`.
Use `tools/port_horizontal_projectile_fixtures.py --lavaman` to rebuild new
fixtures. Full port tests, fmt and strict Clippy pass. Offline only; live
ownership and drop-spawn host seam remain open.

Cleanup applied and deduplicated both scoped run roots: zero ordinary bytes;
50,987,008 Lavaman and 118,620,160 regression snapshot bytes reclaimed.
Both accepted verdicts authenticated after cleanup; immutable source/ROM
snapshots retained. No implementer worktree created. 33 GiB available.
