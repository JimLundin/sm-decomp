# Reconcile legacy ownership and integration

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: `/root/legacy_audit`, thread `01a13462-702f-74f0-8a9a-a2a08dd1cb7f`
Recovery coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`

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

2026-10-13 recovery completion: [owner inventory](../reconciliation/owner-inventory.md)
and [machine-readable bridge](../reconciliation/legacy-owner-bridge.toml) retain
twelve legacy holdings with exact Ceres tuples, ready unclaimed continuation,
original Markdown ticket lineage, real thread/process observations, source,
evidence/readers/private inputs, writable fences and owned next actions. Bridge
SHA-256: `b4046304db39ed5d70501d0b7b29a24abe75b17c03fd0c20ead702857161b4c5`.
Its [hash sidecar](../reconciliation/legacy-owner-bridge.sha256) passes.

Original Ceres integrator has failed terminal turns; submitted controller and
host workers have completed terminal turns. The supported exact original-token
[stopped observation](../reconciliation/host-integrator-stopped.toml) is sealed
and held for later coordinator review; no owner-stop update was admitted.
Host/controller admission, affected-family03–07 completion and decomp-port78/141
broad gameplay gates remain held and owned by the recovery coordinator. This
satisfies the ticket's retained-ownership acceptance branch; these original
tickets and submissions are not resolved or released.

The coordinator reviewed exact-path tooling maintenance through
[the immutable approval](../preparation/approved-maintenance.toml), SHA-256
`c223a2b40b399d9ca9b1047e3f54bd59ab215c07b80528cbf8f1ce118385cce3`.
It preserves the active original engine worker and binds replacement session
worker `01a13464-cb24-77c2-aa7b-ac2aaacd5359` on the clean isolated source.
Main Git/index remains with reporting; future production authority stays pending.

Validation: TOML parser accepts bridge/recommendation/stopped observation; all
bridge `path`/`sha256` artifacts recursively authenticate, including eleven
legacy infrastructure request/receipt artifacts. Checkpoint archives/original
manifests and Ridley retained source/reader/hardware manifests pass. Ceres ledger
hash remains `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`.
No fixtures, replay, build, gameplay merge, index or queue mutation occurred.

Cleanup: only small authored reconciliation evidence was created; there is no
run/build output or integrated implementer worktree to clean in this scope.
Reclaimed **0 bytes**; final available space **223,256,883,200 bytes**. Submitted
controller worktree and all frozen source/readers/private inputs remain retained
for their original gates. Existing lane cleanup is recorded in original handoffs.
The observed Git owner checkpoints these authored files separately.

## Comments

2026-10-13: the user authorized completing preparation tickets 01–05 before a
later gameplay swarm. This claim performs actual owner/process inspection and
retains held legacy source, evidence and next actions in a machine-readable
bridge. Gameplay remains stopped. No controller merge, fixture, replay or
legacy ledger mutation is part of this claim; any proposed checked owner-stop
update requires the primary coordinator's review first.
