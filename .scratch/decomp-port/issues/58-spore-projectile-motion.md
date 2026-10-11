# Spore Spawn stalk/spawners and spore movement
Status: resolved
Blocked by: 57
Triage: ready-for-agent

Port stalk/spawner allocation, countdown and spore ROM movement. Actual spore
creation depends on ticket35 host-side spawn point and remains open. Verify
actual C full WRAM, clean and authenticate.

## Completion evidence

Implemented DCA3 stalk and DCD4 spawner allocation, DCEE ROM-phase movement,
DD46 boss-gated random countdown, and DD44 inert stalk preinstruction. Movement
sign extends ROM bytes, mirrors X using F bit7, doubles Y delta, clears the
phase high byte and preserves signed removal comparison at Y=768.

1,185 actual pinned C full-WRAM/raw-return cases pass: 160 allocations,
512 preinstructions, 512 scheduler frames and sleep control. Evidence:
`out/port/spore-projectile-motion/20261011T042056.762294Z/verdict.json`.
Rebuild through `tools/port_spore_fixtures.py`. Independent Rust tests check
signed/mirrored motion, phase wrapping/removal and frozen countdown/RNG.
Full port tests, fmt and strict Clippy pass. DC77/DC8D actual spore creation
still needs ticket35 host spawn-point support; live ownership remains open.

Cleanup: `port_cleanup.py --root out/port/spore-projectile-motion --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 182,583,296 snapshot
bytes. Accepted evidence authenticated after cleanup; immutable source/ROM
snapshots retained. No implementer worktree created. 33 GiB available.
