# Reconcile legacy ownership and integration

Status: open
Blocked by: None
Triage: ready-for-agent
Type: task

The source checkpoint is committed, but older queues still own unfinished
integration and broad gameplay gates. Reconcile those owners before allocating
new authority. Follow the [runbook](../../../docs/agents/decomp-sessions.md),
[checkpoint](../checkpoint-20261013.md), [completed checkpoint review](../review.md)
and each original ticket.

## Work

- Observe real session/thread ids, registered agents, surviving children and
  process command/start metadata for all held claims. Retain live and ambiguous
  owners; recover stopped owners through exact sealed observations under their
  original coordinator tokens.
- Reconcile `.scratch/ceres-continuation/ledger.toml` under
  `ceres-20261013`, capacity `3`. `integrate-shared-game-host` remains running;
  source `73547105d870f8b05f8c6bcf55b768c223889fdb` was merged by `e8c9c3e`
  and ten actual-C pairs pass, but durable completion is still pending.
- Preserve submitted controller commit
  `223f8a1b0c7382817b1290d13c7a41f4e722d5e1`, clean retained worktree
  `/tmp/sm-ceres-continuation-compact-controller-1`, and ready
  `integrate-compact-controller`. Complete its existing integration gates after
  host-owner reconciliation; keep legacy readers and capture compatibility.
- Inspect affected-family-assignment tickets 03-07 and their ledger: they remain
  claimed pending final integration/verification/cleanup. Preserve decomp-port
  78 and 141 and their original broad gates; record held dependencies rather
  than granting acceptance from newer bounded evidence.
- Write the legacy-owner inventory and bridge requirements for tickets 02-04.
  Keep original ids, revisions, attempts, token, source/evidence and ticket
  lineage. No fresh canonical claims are created in this ticket.

## Acceptance

Every held legacy claim has an observed real owner, preserved source/evidence,
and either completed original admission or an explicit retained ownership entry
with its exact next action. Required host/controller integration is durably
completed or its remaining action is owned and bridged without overlapping
paths. Existing claimed tickets are resolved only through their original gates.
The future canonical ledger remains unactivated.

## Completion evidence

Append owner observations and hashes, original claim tuples, integration
commits and admission receipts, retained bridge records and next commands.
Complete scoped [run cleanup](../../../docs/agents/run-cleanup.md), including
post-cleanup authentication, measured reclaimed bytes/free space, integrated
worktree removal and retained reader/source dependencies before resolution.
