# Shared synchronous game host

Status: claimed
Blocked by: None
Triage: ready-for-agent
Type: task

Implement one typed synchronous host interface for unchanged whole Samus
projectile and room-scroll execution. Execute once on private canonical WRAM;
refusal before any host call leaves caller state untouched, while any failure
after a host request begins forbids native retry. Native host effects must
remain ordered and synchronous. Tests use a recording adapter with explicit
returned values/WRAM mutations and failures.

Add focused actual-C matrices requiring ordered power-bomb HDMA allocation,
terrain PLM allocation and all 17 room display VRAM flushes. Coordinator owns
shared adapter/catalog wiring. Preserve existing live admission and manifests.

## Completion evidence

Pending implementation, focused actual-C comparison and cleanup.

<!-- decomp-assignment:shared-game-host -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `shared-game-host`
Input revision: `48656a2-host-v1`
Lifecycle: `submitted`

Implement typed shared Host Request Response and execute unchanged Samus projectile and room-scroll roots once, preserving refusal versus owned fatal errors. Add recording adapter tests and actual-C fixture matrices with PLM HDMA and17VRAM effects. Root owns callback/catalog integration. Shared pipeline root /home/jim/sm-decomp/out/pipeline workers4.

Writable ownership:

- `crates/port/src/game_host.rs`
- `crates/port/src/lib.rs`
- `crates/port/tests/game_host.rs`
- `tools/port/game-host-projectile.toml`
- `tools/port/game-host-room-scroll.toml`

Claim owner: `ceres-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "submitted"
commit = "73547105d870f8b05f8c6bcf55b768c223889fdb"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/ceres-continuation/host-implementation.toml"
sha256 = "7b4f118b21b969bd02abc16192cbc3caccb1c118d5cef6a5d39e3751afeaffeb"

```

<!-- decomp-assignment-record:end -->

## Checkpoint handoff, 2026-10-13

Submitted source `73547105d870f8b05f8c6bcf55b768c223889fdb` is merged by
`e8c9c3e`. The two actual-C matrices passed ten pairs with material PLM, HDMA
and ordered VRAM flushes; post-cleanup authentication is retained in the main
effort's receipts. The durable host integration remains running because final
merged-source completion was interrupted. Reconcile that existing claim before
resolving this ticket. It establishes a reusable host experiment; broader live
acceptance remains separate.
