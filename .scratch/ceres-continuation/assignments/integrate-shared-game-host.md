# Integrate Shared synchronous game host and material callback fixtures

Status: claimed
Blocked by: None
Triage: ready-for-agent
Type: task

<!-- decomp-assignment:integrate-shared-game-host -->

Stable work: `integrate-shared-game-host`
Input revision: `48656a2-host-v1-1`

## Assignment

Integrate submitted Translation scope Shared synchronous game host and material callback fixtures from its isolated worktree. Freeze merged source, run required verification, authenticate evidence and finish scoped cleanup before recording integrated completion.

## Completion evidence

Lifecycle and sealed result pointers are authoritative in `../ledger.toml`.

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `integrate-shared-game-host`
Input revision: `48656a2-host-v1-1`
Lifecycle: `running`

Integrate submitted Translation scope Shared synchronous game host and material callback fixtures from its isolated worktree. Freeze merged source, run required verification, authenticate evidence and finish scoped cleanup before recording integrated completion.

Writable ownership:

- `crates/port/src/game_host.rs`
- `crates/port/src/lib.rs`
- `crates/port/tests/game_host.rs`
- `tools/port/game-host-projectile.toml`
- `tools/port/game-host-room-scroll.toml`

Claim owner: `ceres-20261013`; attempt: `1`.

<!-- decomp-assignment-record:end -->

## Checkpoint handoff, 2026-10-13

Source is already merged at `e8c9c3e`; final durable integration completion is
pending after the coordinator crashed. Preserve the existing claim and passing
actual-C evidence. Reconcile it before taking the ready controller integration.
