# Power-bomb projectile lifecycle and radius reaction
Status: resolved
Blocked by: 19
Triage: ready-for-agent

Close the bank90 power-bomb branch omitted by ticket19: projectile preinstruction,
PowerBomb_Func3, BombOrPowerBomb_Func1 power-bomb terrain/radius reactions and
associated lifecycle routines needed for their complete native execution.
Study actual bank88 explosion/HDMA dependencies. Translate gameplay state in
readable Rust; any hardware/explosion host callback must be explicit, counted,
ordered and preserve full WRAM effects. Do not claim complete HDMA ownership
unless implemented and verified. Do not replace HandleProjectile prematurely.
Own power_bomb.rs, tests/contracts and its documentation; avoid modifying
projectile_special.rs unless necessary, and coordinate shared changes with root.

Use TDD at the approved full-WRAM actual-C routine/callback seam. Exercise fuse,
activation, radius limits, tiles, allocation failure and cleanup as applicable.
Reject malformed C accesses before execution. Integrate public crate and catalog,
record exact source/dependency seals and distinguish offline from live evidence.

## Completion evidence

Implemented `power_bomb.rs`: native fuse, preinstruction cleanup, activation
bookkeeping, clipped bank94 radius terrain walk, existing normal-bomb delegation.
Bank88 HDMA execution remains C-owned; explicit counted synchronous hardware
allocation preserves full WRAM. See docs/DECOMP_POWER_BOMB.md for domains and the
sealed one-token allocator-definition seam needed to interpose local C calls.
Public crate registration and all-profile-only batch `power-bomb` admission added.

Actual-C offline evidence: out/port/power-bomb/20261010T214306.622558Z/verdict.json,
88 cases across four exported symbols, 54 ordered PLM/HDMA callbacks, full WRAM
and return parity, zero CPU fallback. Evidence revalidated using dedicated batch
kind; exact upstream/archive, ROM, module and dependency seals are retained.
Four independent Rust tests pass; complete port suite passed during implementation.
Rejected intermediate builds/comparisons are retained in the same output family.
No live exercise or complete bank88 HDMA ownership is claimed.
