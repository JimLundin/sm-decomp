# Research one complete host/hardware lifecycle boundary

Status: open
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumers:06–08
Read-only work can proceed beside11; held host source mutation waits11

## Question / Work

Reuse `game_host.rs`'s typed SpawnPlm/SpawnHdma/FlushVram transaction and ten
material actual-C pairs, described in [catalog](../catalog/subsystem-catalog.md).
Select a complete accepted-NMI/VRAM or reset lifecycle scheduled boundary from
sm_80/82 with original reference evidence. Review WRAM/hardware pending state,
DMA/HDMA, NMI enabled/disabled, IRQ, PPU and APU domains and complete external
services. Keep unchanged Samus projectile/room consumers and current refusal
versus fatal-after-host/no native retry contract.

Exact held files: `crates/port/src/game_host.rs`,
`crates/port/tests/game_host.rs`, `tools/port/game-host-projectile.toml`,
`tools/port/game-host-room-scroll.toml`. Proposed dedicated files:
`crates/port/src/hardware_lifecycle.rs`, `crates/port/tests/hardware_lifecycle.rs`,
`tools/port/hardware-lifecycle.toml`. Shared lib/state/ROM and all pipeline
registration/templates/adapters are solely integrator-owned. Select only the
needed exact files; do not take held host files before11's completion.

## Acceptance / Answer

Publish an exercised coherent1–32-routine boundary with source/exercise hashes,
every direct/indirect/MMIO dependency review and exact source/test/manifest
ownership. Existing native exercise reader accepts bank86 projectile receipts
only: locate supported provenance or create an actionable reviewed Rust
exercise-contract extension task instead of substituting unrelated counters.
Require actual-C full state/returns and ordered material effects, refusal before
mutation, fatal-after-host/no retry and unchanged-consumer regression. Pre/merged
argv,60s initial/3600s integration limits and original-ROM gate must be concrete.
Create exact implementation/extension children and add required consumers as
blockers before gameplay dispatch. No global host redesign or fabricated readiness.

## Completion evidence

Pending Answer, generated child/scope receipts and hashes. No-run or scoped-run
cleanup with measured bytes/free space, retained dependencies and authentication.
