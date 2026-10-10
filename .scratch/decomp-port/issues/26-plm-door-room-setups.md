# Native door, open gate and escape-room PLM setups
Status: resolved
Blocked by: 25
Triage: ready-for-agent

Expand actual bank84 setup closure: grey/colored/blue doors, open gates and
Mother Brain/critter escape terrain, Shaktools scroll initialization. Preserve
canonical room-argument and projectile overlays, BTS neighbors, signed sentinel
and original tile masks. Closed gates spawning enemy projectiles remain explicit
unowned dependency. Use TDD at approved public setup/full-WRAM actual-C seams.
Register supported headers and public dispatcher, verify independent worked
examples and actual-C branch cases. Record offline/live status and cleanup;
no whole frame-engine or original-ROM parity claim.

## Completion evidence

Ten additional actual bank84 setup routines translated: grey/colored/blue doors,
up/down open gates, Mother Brain escape door, critter escape terrain, four-cell
solid door, Shaktools scroll initialization and B974 no-op. Public setup/header
admission registration preserves WRAM aliases, byte BTS writes, room argument
condition masks and projectile sentinel/deletion. Invalid projectile indices
and gate/door column extents reject before mutation. Closed gates remain C-owned
because their enemy-projectile spawn dependency is not translated here.

Actual compiled pinned C full-WRAM+raw-return comparisons:54routines/432cases
(80new cases) at `out/port/plm-door-setups/20261010T224620.249147Z/verdict.json`.
Twelve independent PLM public tests, complete port Rust tests, cargo fmt and
Clippy pass. Source review caught/corrected canonical scroll0xcd20 after actual
C rejected prior0x7cd offset; rejection retained in preceding run. No live or
original-ROM parity claim; newly expanded module needs fresh consumer seals.

Cleanup completed with `tools/port_cleanup.py --root out/port/plm-door-setups
--apply` (0bytes) and `--deduplicate --apply` (124,780,544bytes reclaimed).
Sealed fixtures, actual-C outputs and rejection retained; no worktree created.
Remaining disk approximately40GB. Final evidence reauthenticated after cleanup.
