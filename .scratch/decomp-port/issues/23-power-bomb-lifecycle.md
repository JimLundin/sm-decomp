# Power-bomb projectile lifecycle and radius reaction
Status: open
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
