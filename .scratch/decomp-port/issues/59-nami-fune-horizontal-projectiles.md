# Nami/Fune fireball and DBF2 horizontal projectiles
Status: resolved
Blocked by: 58
Triage: ready-for-agent

Port DED6/DB18 allocation and DF39/DB5B scheduling with directional velocity
selection and inclusive camera culling. Verify actual C and clean evidence.
Drop-spawn opcodes remain dependent on ticket35 host spawn-point support.

## Completion evidence

Implemented DB18/DED6 allocation: enemy position/fraction copying, direction
state/script selection, Y spawn offsets and Nami/Fune eight-row velocity table.
DB5B/DF39 scheduling selects the correct velocity field, moves X in 16.16 and
uses inclusive two-axis camera bounds. DB62/DB8C direct preinstructions are
supported too. Invalid state/table records return errors.

769 actual pinned C full-WRAM/raw-return cases pass (256 allocations, 256
preinstructions, 256 scheduler frames and sleep control):
`out/port/nami-fune-horizontal-projectiles/20261011T042237.836397Z/verdict.json`.
Rebuild through `tools/port_horizontal_projectile_fixtures.py`. Independent Rust
test checks left-state Y-velocity selection, fraction carry, stationary Y and
inclusive boundary. Full port tests, fmt and strict Clippy pass. Offline only;
drop-spawn host state and live ownership remain open.

Cleanup: `port_cleanup.py --root out/port/nami-fune-horizontal-projectiles --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 118,620,160 snapshot
bytes. Final verdict authenticated after cleanup; immutable ROM/source snapshots
retained. No implementer worktree created. 33 GiB free disk space.
