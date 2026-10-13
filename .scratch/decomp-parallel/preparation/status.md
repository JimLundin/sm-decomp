# Parallel-session preparation readiness

Status: tooling and catalog complete; corrected actual drill active, final readiness pending
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
| Frozen checkout | `/tmp/sm-decomp-preparation-retry-source-20261013` |
| Full source commit | `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd` |
| Source fingerprint | `bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec` |
| Engine | `../drill-retry/bin/sm-pipeline` |
| Engine SHA-256 | `04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573` |
| Build receipt | [engine-build.toml](../drill-retry/engine-build.toml), SHA-256 `cdf1295fdc8ad86b574866636f68966205e5c46bf2fd19649fa9a4dd2f5bdbb9`, checkpoint `9acff17` |

The prior `drill-final` completed both serial integrations but A's completed
exact retry failed after B advanced shared HEAD. The
[completed-retry correction](../../decomp-integration-retry/handoff.toml) retains
that diagnosis, red/green controls and the Rust fix; all 23 assignment, 17 session
and 11 authority controls pass. Main integration is `0d042e0`; root checked
Cargo/crates/tools source equivalence against the retained frozen source.
The new [drill recipe](../drill-retry/recipe.md) uses isolated fixture
`/tmp/sm-decomp-concurrency-drill-retry-20261013`, effort
`concurrency-drill-retry`. Actual independent sessions A
`01a13515-9a51-7c73-9913-1deb499ba15c` and B
`01a13515-9a51-7af3-bc9f-a144afb06090` are active. Recover exact surviving owners
and sealed state before issuing replacement work.

Root must add these actual final receipts before setting preparation ready or
resolving 04:

- **PENDING:** final `drill-retry/concurrency-drill.toml`, sealed execution
  manifest, both sequential integrations and both completed exact retries at
  final merged HEAD, plus passing maintained validation.
- **PENDING:** current-root readiness successor at
  [readiness-retry-20261013](readiness-retry-20261013/README.md), actual
  `production-validation-1` private verdict with state `prepared`, host
  authority/ledger preservation and unchanged legacy seals.
- **PENDING:** both scoped cleanup passes, measured reclamation/free space,
  required source/readers retained, integrated worktrees retired and passing
  post-cleanup authentication.
- **PENDING:** durable final documentation/source checkpoint owned by root.

Original `drill/`, `drill-final/` A replay failure, prior source/validator epochs,
recovery records and old readiness/cutover packages remain historical evidence.
Current root is `01a13500-1ef8-73d3-9a0d-3446b6050eba`;
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
