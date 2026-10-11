# Tourian statue eye glow, souls and particle allocation
Status: resolved
Blocked by: 51
Triage: ready-for-agent

Translate B88E/B8B5/B8E8/B8F8 initialization. Preserve palette writes, source
projectile aliases, RNG angle and sine velocities. Verify actual pinned C
allocation with full WRAM and complete cleanup. Effect scheduling remains open.

## Completion evidence

Implemented eye-glow coordinates/four palette colors, soul velocity and particle/
tail source-slot allocation with RNG/sine motion initialization. 193 actual pinned
C full-WRAM/raw-return cases pass (192 allocation, one sleep control) in
`out/port/tourian-statue-effects/20261011T040815.545068Z/verdict.json`.
Full port suite, fmt and strict Clippy pass. Allocation fixture equivalence only;
effect scheduling and live ownership remain open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 41,025,536 snapshot bytes. No implementer
worktree created. Immutable ROM/source/run evidence retained; final verdict
authenticated after cleanup. 35 GiB free.
