# Native power bomb lifecycle

`crates/port/src/power_bomb.rs` translates bank90 `ProjPreInstr_PowerBomb`
and `PowerBomb_Func3`, bank88 `SpawnPowerBombExplosion` bookkeeping and bank94
power-bomb radius traversal reached through `BombOrPowerBomb_Func1`. The existing
normal-bomb path is reused. Radius helpers keep the original repeated corners
and top/left/bottom/right order, signed clipping, negative activation sentinel
and timer wrap. `ClearProjectile` remains the existing native implementation.

HDMA allocation is an explicit synchronous host call. Both original objects are
requested even on allocation failure. Frozen activation requests no objects.
The host receives a full owned WRAM transaction, and the returned state is copied
back before the next allocation. Bank88 HDMA preinstructions, color/shape tables,
channel updates and final explosion completion remain C-owned. This module does
not replace `HandleProjectile`, nor establish complete HDMA ownership or live
movie parity.

`sm-pipeline family-verify --adapter power-bomb` compares actual pinned C and Rust with full WRAM,
returns, fallback counts and ordered PLM/HDMA callback before/after snapshots.
Because sm88 calls its allocator locally, linker wrapping alone does not measure
those calls. The harness seals and retains a temporary source copy with only the
allocator definition renamed to `PortActualSpawnHdmaObject`; its body is unchanged,
and callers use the counted external wrapper. Original upstream files remain
untouched. ROM, upstream archive, source files, all Rust dependencies, harness,
binaries and retained callback data are sealed in each verdict.

Declared domains: canonical even projectile slots 0..18; original bounded projectile type-table indexes for shared contact; room dimensions 1..255; radius
perimeter within tile backing extent 0x3200; existing bounded extension and PLM
setup guards. Invalid inputs are rejected by Rust before actual-C fixture execution;
only deliberately valid fixtures invoke C. Offline fixtures include fuse 0/1/2/16/
65535, pointer wrap, activation frozen/unfrozen, all channels allocated, radius
0/1/32/255, rejected projectile direction, offscreen coordinates, all 16 tile types
and cleanup. Independent Rust tests check allocation failure order, literal final
perimeter block index and malformed-input rejection.

Offline verification: `out/port/power-bomb/20261010T214306.622558Z/verdict.json`
contains 88 matching cases across four exported symbols and 54 matching ordered
callbacks, with zero CPU fallback. Four independent Rust tests pass. Dedicated
batch kind `power-bomb` is available in the `all` profile only.

Review hardening rejects signed or wide dispatch slot arguments before converting
them to the original 16-bit domain. `65536` and `-65536` cannot alias slot zero;
all three argument-taking entries reject without WRAM changes or host callbacks.

The dedicated harness deliberately retains the same full-WRAM transaction,
artifact and callback comparison protocol as `sm-pipeline callback-verify`. Its HDMA seam
requires a second host interface, an allocator-definition rename in bank88,
allocation-specific arguments and a retained source-copy seal. Moving build,
wrapper generation and evidence validation into a shared configurable harness
would change both existing PLM/crawler and power-bomb evidence consumers and their
source seals. That is a separate harness-maintenance change requiring regression
controls for both callback kinds; this review keeps those boundaries stable.
User-visible errors now identify power-bomb evidence rather than collision evidence.
