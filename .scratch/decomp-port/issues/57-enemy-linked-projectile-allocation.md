# Enemy-linked decoration and Nuclear Waffle body allocation
Status: resolved
Blocked by: 56
Triage: ready-for-agent

Port BB30 and BB92 initializers, preserving extended RAM coordinates, ROM script
selection, all position fractions, linked enemy extended-RAM slot write and
projectile flags. Verify pinned C full WRAM and clean retained runs.

## Completion evidence

Implemented BB30 extended-8000 coordinates/three ROM script selections and BB92
Nuclear Waffle body position/fraction copying, linked 7800 enemy slot write and
F380 projectile flags. Enemy and linked record validation bounds unchecked C
aliases to supported records. Independent Rust test checks slot34 linkage and
both coordinate fractions.

385 actual pinned C full-WRAM/raw-return cases pass (384 allocations and sleep
control), covering source records, linked offsets, free-slot selection and full
pool no-op behavior. Evidence:
`out/port/enemy-linked-projectile-allocation/20261011T041936.806333Z/verdict.json`.
Rebuild fixtures through `tools/port_enemy_linked_fixtures.py`.
Full port tests, fmt and strict Clippy pass. Offline allocation verification;
live gameplay ownership remains open.

Cleanup: `port_cleanup.py --root out/port/enemy-linked-projectile-allocation --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 82,313,216 snapshot
bytes. Accepted evidence authenticated after cleanup; immutable source/ROM
snapshots retained. No implementer worktree created. 34 GiB free disk space.
