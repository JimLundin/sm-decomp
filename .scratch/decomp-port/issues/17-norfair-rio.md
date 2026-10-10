# Norfair and Lower Norfair Rio state closure
Status: resolved
Blocked by: 15
Triage: ready-for-agent

Own both six-state enemy AI dispatchers, ordinary and paired child drawing/spawn state, animation and random attack selection, active movement/reversal and audio. Reuse public enemy_ai explicit Host movement dispatch; no shared source edits. Validate canonical overlays, bounds and all supported state dispatch against actual compiled pinned C. Bank-specific animation instruction callbacks remain C-owned.

## Completion evidence
18 actual C routine translations: Norfair and Lower Norfair Rio init, main and six-state handlers plus animation-list helper. Main dispatch rejects unsupported preinstructions before advancing RNG. Movement reuses `enemy_ai::dispatch_host`; its PLM host remains explicit. No shared module edits.

Paired child rendering follows actual packed EnemySpawnData `[31]` (64 bytes per struct) at `0x77c0 + enemy_offset`. Drawing queue accesses likewise use enemy byte offsets and overlap canonical enemy fields; independent copies would be wrong. Base offset is aligned below 0x800; extended variables are `0x7800 + offset`; child accesses are bounded within combined spawn/extended canonical storage 0x7000..0x8800. Frozen parent, missing parent, invisible animation parent, signed Y offset, and both visible child forms are preserved.

Final actual compiled pinned C full-WRAM/return/ordered-host evidence: `/tmp/enemy-rio-current-final/20261010T203935.357572Z/verdict.json`, 18 routines /162 cases, zero CPU fallback. Cases exercise every state including horizontal reversal, upward/downward collision and acceleration thresholds, random attack gating, near/far Samus, animation signals, frozen/deleted child states, negative child offset, and highest admitted enemy slot1984. Real ROM source identity retained; original-ROM parity and live integration not tested.

Four independent Rust tests pass: lower rising transition, packed child signed offset, missing-parent deletion, animation-gated recovery/unknown state rejection. Isolated ignored RustySNES dependency prevents Cargo workspace use, so tested real library rlib plus `rustc --test`; rustfmt --check passes. Root Cargo tests required after public registration.

Earlier actual C160-case pass retained `/tmp/enemy-rio-final/20261010T203839.735077Z/verdict.json`; superseded because public module registration changed memory/dependency identity. No bank-specific animation instruction handlers are claimed by this ticket.

Commit: this ticket's implementation commit in git history.
