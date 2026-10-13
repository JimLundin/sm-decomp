# Parallel-session preparation readiness

Status: tooling and catalog complete; recovered final drill/readiness validation pending
Blocked by: 04
Triage: ready-for-agent
Updated: 2026-10-13
Coordinator: `01a13500-1ef8-73d3-9a0d-3446b6050eba`

The [spec](../spec.md), [ticket map](../map.md) and
[session runbook](../../../docs/agents/decomp-sessions.md) define the workflow.
Preparation completes tickets 01–05; the overall gameplay spec remains open.
The user's separate gameplay swarm launch instruction is still required.

| Ticket | Status | Retained evidence |
| --- | --- | --- |
| [01 Legacy reconciliation](../issues/01-reconcile-legacy-ownership.md) | resolved | Twelve retained holdings, immutable original/successor bridges and checked maintenance adoption |
| [02 Ownership and integration fencing](../issues/02-fence-writable-paths-and-integration-sessions.md) | resolved | Cross-effort path/ticket/submission fencing, capacity and real-session generations; focused verification/cleanup complete |
| [03 Rust commands and handoffs](../issues/03-maintained-rust-commands-and-handoffs.md) | resolved | Typed infrastructure admission, Start/Checkpoint/Resume/Adopt and exact retries; focused verification/cleanup complete |
| [04 Actual concurrency/recovery drill](../issues/04-concurrency-and-crash-recovery-drill.md) | claimed | Fresh execution at corrected source; final eleven controls, two serial authenticated integrations, maintained validation/readiness and cleanup receipts pending |
| [05 Subsystem catalog](../issues/05-subsystem-scope-catalog.md) | resolved | Whole-subsystem catalog and children 11–20; later implementation readiness remains research |

## Current validation subject

Execution and validation use the same retained source and executable:

| Identity | Value |
| --- | --- |
| Frozen checkout | `/tmp/sm-decomp-preparation-final-source-20261013` |
| Full source commit | `2d59536c059a1ca9c31d9278bff2d9222eafa56c` |
| Source fingerprint | `e5e412053f6dbe2cbadaba077192e885684fbb9f2699cafeaa6acd694879d78d` |
| Engine | `../drill-final/bin/sm-pipeline` |
| Engine SHA-256 | `df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84` |
| Build receipt | [engine-build.toml](../drill-final/engine-build.toml), SHA-256 `659caced91feb9fc00bdbce7e7ced4e464874a9b8db8db84a69051dbff610bb6` |

The [serial fix](serial-fix-handoff.md) reproduced the prior second-integration
scope rejection and passed all 51 maintained assignment/session/authority
controls. It was integrated as `23fde6a`; this result proves the maintained
regression controls, while the fresh actual drill proves independent sessions.
The [fresh drill handoff](../drill-final/handoff.md) names isolated fixture
`/tmp/sm-decomp-concurrency-drill-final-20261013`. Recover surviving owners and
sealed state before issuing any replacement work.

Root must add these actual final receipts before setting preparation ready or
resolving 04:

- **PENDING:** final `drill-final/concurrency-drill.toml`, sealed execution
  manifest, both sequential integration results and passing maintained validation.
- **PENDING:** current-root readiness successor at
  `readiness-recovery-20261013/`, actual private namespace validation with state
  `prepared`, host authority/ledger preservation and unchanged legacy seals.
- **PENDING:** both scoped cleanup passes, measured reclamation/free space,
  required source/readers retained, integrated worktrees retired and passing
  post-cleanup authentication.
- **PENDING:** durable final documentation/source checkpoint owned by root.

The prior `c8799f3` execution and separate `f54fbed` validator, original `drill/`
diagnostics, recovery records and old readiness/cutover packages remain evidence
of their own epochs. Current root is `01a13500-1ef8-73d3-9a0d-3446b6050eba`;
prior preparation roots `01a13460-4aa9-7fe0-865e-a8f54ffca310` and
`01a134cd-dafb-7e50-8284-f285a905e05b` retain their historical observations.
Current main Git/index and process owners require direct observation by root;
these documents do not establish liveness.

## Production and later launch

Production `.scratch/decomp-parallel/authority.toml` and `ledger.toml` remain
absent. Ceres authority remains `.scratch/ceres-continuation/ledger.toml`, token
`ceres-20261013`, capacity `3`, SHA-256
`77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`.
Its original controller/host submissions and host integration retain their
claims, source, readers and acceptance gates. The
[v2 bridge](../reconciliation/legacy-owner-bridge-v2.toml) retains SHA-256
`cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.

After preparation resolution and the user's separate launch instruction, follow
[the launch handoff](launch-handoff.md). Reinspect exact real owners, source,
bridge and ledger before creating current production requests. Private readiness
validation prepares and validates a policy view; production prepare, actual role
registration, validate-activation and activation occur at launch. Canonical queue
creation follows admitted operations after activation.

All production sessions share `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`
and `SM_PIPELINE_WORKERS=4`; assignment capacity `3` is a separate budget.
Ticket 11 continues the existing Ceres integration first; research 13/14 can use
free slots while it integrates. Native promotion accepts bank86 exercise
provenance and coherent 1–32 routine scopes; other lanes need their researched
matching contracts. One integrator owns frozen verification, stage publication,
main Git/index and run cleanup.
