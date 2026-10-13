# Gameplay swarm launch handoff

Status: ready for separately requested gameplay swarm launch
Blocked by: the user's separate gameplay swarm launch instruction
Preparation coordinator: `01a13559-e517-72d1-88b5-e554fed61da4`

For a fresh or replacement session, read [AGENTS.md](../../../AGENTS.md),
[domain rules](../../../docs/agents/domain.md), [GLOSSARY.md](../../../GLOSSARY.md)
and relevant ADRs, then [the session runbook](../../../docs/agents/decomp-sessions.md),
[spec](../spec.md), [map](../map.md), [preparation status](status.md),
[checkpoint](../checkpoint-20261013.md), [fresh drill recipe](../drill-retry/recipe.md),
[current-root readiness successor](completion-20261013.md),
[catalog](../catalog/subsystem-catalog.md) and [later lanes](../catalog/later-lanes.md).
Use [assignments](../../../docs/agents/assignments.md),
[translation progress](../../../docs/agents/translation-progress.md) and
[run cleanup](../../../docs/agents/run-cleanup.md) for dispatch and integration.

## Retained preparation evidence

Execution and validation use clean frozen checkout
`/tmp/sm-decomp-preparation-retry-source-20261013`, full source commit
`57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`, fingerprint
`bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec`.
Retained engine [sm-pipeline](../drill-retry/bin/sm-pipeline) has SHA-256
`04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573`;
[engine-build.toml](../drill-retry/engine-build.toml) has SHA-256
`cdf1295fdc8ad86b574866636f68966205e5c46bf2fd19649fa9a4dd2f5bdbb9`
and checkpoint `9acff17`. Main integration is `0d042e0`.

[Actual drill](../drill-retry/concurrency-drill.toml), SHA-256 `63912c67e38680fd163d09126ba285f8ce26da8a2445c801f489f8971b74751f`,
passed all eleven controls, both serial integrations and both completed replays
at final merged HEAD. [Readiness completion](completion-20261013.md)
links private `prepared` validation and host-preservation receipts; output
manifest SHA-256 is `241018cd062ef561c5650c90bce7840bec5e0749e9ebc4d2b220203fa1e4269d`. Both cleanup passes and
post-retirement authentication passed; exact measurements/dependencies are in
[ticket04](../issues/04-concurrency-and-crash-recovery-drill.md). Production
authority/ledger remain absent. Old drill/frozen/readiness packages and failures
remain immutable historical evidence; the prior A-after-B replay defect is
fixed and exercised by this retained corrected-source drill.

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
3. Use the policy and exact receipts linked from
   [final completion](completion-20261013.md). Derive fresh typed production
   requests from the reviewed readiness policy, exact legacy bridge and passing
   retained drill. Observed integrator
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
