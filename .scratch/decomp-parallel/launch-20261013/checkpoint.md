# Activated swarm setup checkpoint

Status: handoff ready
Blocked by: observed setup turn completion before integrator transfer
Updated: 2026-10-13

The user explicitly authorized activating and starting the decomp swarm.
Production is active and the dedicated coordinator task `Run decomp swarm`
(`01a13592-d02b-7143-8489-a15b03f19da2`) is running. This checkpoint completes
setup; the gameplay queue remains open. [Launch status](status.md) records the
roles, completed work and outstanding evidence gates.

The setup turn retains main Git/index and integrator ownership until completion.
Its real thread is `01a1356e-a8dd-7a80-88b0-1239a6a9b876`, integrator token
`ceres-20261013`, generation 1. The background dispatcher is registered as
`decomp-run-20261013`, generation 1. After checking the actual completed setup
turn and this committed handoff, follow [coordinator-start.md](coordinator-start.md)
for the checked generation-2 transfer. All original claims/submissions remain
held. Resume uses the current real caller identity and retained lineage.

## Durable source and ownership

Setup began at `40a8489856f36d21925188823e33b520f74d4cb4` on branch
`affected-family-assignment`, main checkout `/home/jim/sm-decomp`.
The launch commit preserves project agent config/roles, active authority and
canonical queue, exact requests/receipts/observations, completed research 13/14,
children 21–24, updated milestone dependencies, cleanup and host handoff.
Find the containing commit with:

```sh
git log -1 --format=%H -- .scratch/decomp-parallel/launch-20261013/checkpoint.md
```

Cargo/crates/tools remain identical to the validated frozen source at
`57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`, retained in
`/tmp/sm-decomp-preparation-retry-source-20261013`. The original Ceres baseline
remains `48656a284329d432291eca23a2ae0d1d7c0a3e48`. Preserve
[sealed ownership](ownership.md), its observations and the legacy bridge.

| Retained artifact | SHA-256 |
| --- | --- |
| `../drill-retry/bin/sm-pipeline` | `04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573` |
| `../drill-retry/concurrency-drill.toml` | `63912c67e38680fd163d09126ba285f8ce26da8a2445c801f489f8971b74751f` |
| `../reconciliation/legacy-owner-bridge-v2.toml` | `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9` |
| `../../ceres-continuation/ledger.toml` at handoff | `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89` |

## Completed verification and cleanup

Configuration load, TOML lint, scoped Git checks, 31 focused Rust tests and
seven projectile plus three room actual-C fixture comparisons passed.
Both matrix cleanup passes and post-cleanup verdict authentication passed.
Research input/artifact manifests and all three checkpoint archive hashes were
rechecked. [Cleanup](cleanup.md) records measured reclamation, free space and
every retained evidence dependency. No implementation worktree was created;
retained submitted/frozen worktrees continue to be required. Only the existing
progress server and the background coordinator's read-only planning remained
active at checkpoint; no verification/build writer remained.

## Next action

After checked transfer, the coordinator owns main Git/index. Complete callback
and family controls, then the conservative all-affected Offline batch from the
original Ceres baseline using [held-host.toml](handoffs/held-host.toml). Perform
its own cleanup/authentication and record the original integrated result
through the authorized Rust Ceres wrapper. Host source is already merged;
controller submission `223f8a1b0c7382817b1290d13c7a41f4e722d5e1` follows host
admission and must be merged once. Delegate dependency-ready whole-subsystem
research/implementation with exact claims, isolated writable ownership and
durable worker handoffs. Publish achieved gameplay stages with their required
evidence before each later handoff.
