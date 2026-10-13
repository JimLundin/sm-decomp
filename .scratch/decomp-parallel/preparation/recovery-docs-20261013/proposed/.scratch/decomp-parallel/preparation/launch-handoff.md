# Gameplay swarm launch handoff

Status: proposed final handoff; actual final receipts pending
Blocked by: 04 and the user's separate gameplay swarm launch instruction
Preparation coordinator: `01a13500-1ef8-73d3-9a0d-3446b6050eba`

For a fresh or replacement session, read [AGENTS.md](../../../AGENTS.md),
[domain rules](../../../docs/agents/domain.md), [GLOSSARY.md](../../../GLOSSARY.md)
and relevant ADRs, then [the session runbook](../../../docs/agents/decomp-sessions.md),
[spec](../spec.md), [map](../map.md), [preparation status](status.md),
[checkpoint](../checkpoint-20261013.md), [fresh drill handoff](../drill-final/handoff.md),
[current-root readiness successor](readiness-recovery-20261013/README.md),
[catalog](../catalog/subsystem-catalog.md) and [later lanes](../catalog/later-lanes.md).
Use [assignments](../../../docs/agents/assignments.md),
[translation progress](../../../docs/agents/translation-progress.md) and
[run cleanup](../../../docs/agents/run-cleanup.md) for dispatch and integration.

## Retained preparation evidence

Execution and validation use clean frozen checkout
`/tmp/sm-decomp-preparation-final-source-20261013`, full source commit
`2d59536c059a1ca9c31d9278bff2d9222eafa56c`, fingerprint
`e5e412053f6dbe2cbadaba077192e885684fbb9f2699cafeaa6acd694879d78d`.
Retained engine [sm-pipeline](../drill-final/bin/sm-pipeline) has SHA-256
`df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84`;
[engine-build.toml](../drill-final/engine-build.toml) has SHA-256
`659caced91feb9fc00bdbce7e7ced4e464874a9b8db8db84a69051dbff610bb6`.

**PENDING:** root attaches the final actual drill/eleven-control manifest and
passing maintained validation, both sequential integration receipts, current-root
private readiness verdict and host-preservation seals, both cleanup passes and
post-retirement authentication. Preparation remains claimed until these receipts
are present. Old `drill/`, frozen subjects, readiness packages and their failures
remain immutable historical evidence.

## Launch steps

1. Recheck checkout HEAD/worktrees, exact retained source/build/drill seals and
   checkpoint archive. Inspect Ceres status with effort `ceres-continuation`,
   token `ceres-20261013`, capacity `3`; inspect every other ledger/claimed ticket
   for overlap. Share `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
   `SM_PIPELINE_WORKERS=4`. Inspection is complete when every held path and claim
   has one observed owner or a retained ambiguous owner.
2. Inspect actual session/thread ids, surviving children and process start
   identities. Preserve live/ambiguous claims and stopped submitters' submissions.
   Record main Git/index and integration owners. The preparation coordinator UUID
   is a historical observation for later sessions; names and elapsed time cannot
   establish current liveness. Replacement roles use checked transfers/adoption.
3. Derive fresh typed production requests from the reviewed current-root readiness
   policy, exact legacy bridge and passing retained drill. Observed integrator
   calls maintained `decomp-authority prepare`, registers actual dispatcher and
   integrator roles/checked transfers, then `validate-activation` and `activate`
   with current real session id/generation. Preserve inherited `CODEX_THREAD_ID`.
   The private readiness result has state `prepared`; production activation and
   canonical admitted queue creation are separate later operations. This step
   completes when active returned roles/generations match inspected owners.
4. Before any legacy mutation, recheck Ceres ledger SHA-256
   `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`
   and [v2 bridge](../reconciliation/legacy-owner-bridge-v2.toml) SHA-256
   `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.
   Reconcile changed seals. Retain original tuples/attempts/tokens/gates; legacy
   envelopes replay only exact requests sealed as requests in that bridge.
5. Choose the first open, unblocked, unclaimed ticket by number. Reserve one job
   per available subagent slot and give each actual child the returned claim,
   pinned baseline, exact writable files, isolated worktree/run path and gates.
   Persist its returned UUID through typed Start; checkpoint before/after gates.
   Completion requires a durable handoff naming the exact next command and cwd.

Activation binds the retained checkout's exact path, HEAD and fingerprint.
Mutation/admission code changes require a fresh actual drill; validator/docs
changes require explicit source-equivalence review. Cross-machine recovery
restores required absolute source paths, binary/readers, evidence store/pointers
and private ROM/movie inputs before authentication.

## First work and completion

[11](../issues/11-finish-held-ceres-integration.md) continues original Ceres work.
Finish held `(integrate-shared-game-host, 48656a2-host-v1-1, 1, ceres-20261013)`;
host `73547105d870f8b05f8c6bcf55b768c223889fdb` is already merged by
`e8c9c3e10ac71bf0377dc00d7de5764a4263b445`. Then reserve existing ready
`integrate-compact-controller`, linked to submitted
`(compact-controller, 48656a2-compact-v1, 1, ceres-20261013)` at
`223f8a1b0c7382817b1290d13c7a41f4e722d5e1`. Preserve original gates/readers.
[12](../issues/12-ceres-nmi-poll-continuation.md) follows 11: research extra poll
at ordinal 7,741/frame 8,508 and publish its Answer/reviewed correction scope
before mutation. Its observer/pipeline/C-template work uses infrastructure
admission; the custom holding TOML is not native `assign --scope` input.

Read-only whole-subsystem research 13/14 can fill free slots while 11 integrates;
15 and 17–20 follow ticket order. Research creates exact implementation children
and milestone blockers. Native promotion currently admits bank86 exercise
provenance and coherent 1–32 routines; other whole-root domains need matching
supported evidence or reviewed Rust contract extensions. Continue held whole
Ridley ticket 141 via 16 only when route dependencies/user steering permit.
Reuse accepted leaves with their bounded domains. The integrator owns shared
lib/state/ROM, CLI/catalog/ABI/observer/host wiring, inventory and main Git/index.

Initial reviewed experiments are at most 60 seconds; integration gates at most
3,600 seconds. Implement tools/orchestration/tests in Rust. Integrator freezes
merged source, verifies every affected Offline family and required actual-C/live/
original-ROM gates, publishes stages independently, completes both cleanup
passes, retains source/store/readers/private inputs, retires integrated worktrees
and authenticates before durable completion. Ceres escape requires ordinary
reset and observed `32 → 33 → 34`; later route/save/load/credits and zero gameplay
C/CPU fallback milestones retain their separate gates.
