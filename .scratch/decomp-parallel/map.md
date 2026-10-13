# Restartable parallel decomp work

Status: open
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10
Plan deliverable: complete; implementation remains open

The [spec](spec.md) defines the outcome and authority transition. The
[session runbook](../../docs/agents/decomp-sessions.md) contains supported
commands, subagent dispatch, WIP/handoff rules and fresh/resume prompts.
The [committed checkpoint](checkpoint-20261013.md) preserves current source
and evidence; it is the completed preparation for this plan.

## Decisions so far

- Source/receipts are checkpointed at
  `d8a90451695408bbf72017ea04e7f4e915b06dd9`. Passing evidence and a Git
  merge do not resolve pending durable integration.
- Current Ceres authority remains `.scratch/ceres-continuation/ledger.toml`,
  coordinator `ceres-20261013`, capacity `3`. Existing claims in other efforts
  retain their original authority. The future `decomp-parallel` queue is
  unactivated until reconciliation, fencing and the two-session drill establish
  a preserved claim bridge and explicit cutover.
- All workers use `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
  `SM_PIPELINE_WORKERS=4`; these four process workers are shared across sessions.
  Delegate actual work to subagents. Keep branches, writable files, run paths
  and handoffs independent; one identified integrator owns shared mutations.
- Research and independent implementation run concurrently. Final registration,
  merged-source verification, stage publication and Git/index mutations serialize
  under one durable integration owner.
- Preserve the actual-C and original-ROM admission contracts. Scope counts,
  WIP commits and diagnostic prefixes describe bounded progress.

## Tickets and parallel order

| Child | Type | Status | Blocked by | Lane |
| --- | --- | --- | --- | --- |
| [01 Reconcile legacy ownership](issues/01-reconcile-legacy-ownership.md) | task | open | None | Recovery/integration; ready for owner inspection |
| [02 Fence writable paths and integration sessions](issues/02-fence-writable-paths-and-integration-sessions.md) | task | open | 01 | Assignment engine |
| [03 Maintain Rust commands and durable handoffs](issues/03-maintained-rust-commands-and-handoffs.md) | task | open | 01 | Session interface; can develop beside 02 with disjoint files |
| [04 Prove concurrent sessions and crash recovery](issues/04-concurrency-and-crash-recovery-drill.md) | task | open | 02, 03 | Actual two-session launch gate |
| [05 Catalog dependency-ready subsystem scopes](issues/05-subsystem-scope-catalog.md) | research | open | 01 | Read-only research beside 02-04 |
| [06 Ordinary-reset Ceres escape](issues/06-ordinary-reset-ceres-escape.md) | task | open | 04, 05 | First gameplay milestone; continue existing ticket 78 |
| [07 Playable vanilla progression](issues/07-playable-vanilla-progression.md) | task | open | 06 | Parallel subsystem children; composed route gate |
| [08 Save/load and credits](issues/08-save-load-and-credits.md) | task | open | 07 | Persistence and completion children |
| [09 Complete Rust gameplay ownership](issues/09-full-rust-gameplay-ownership.md) | task | open | 08 | Remaining coherent ownership gaps |
| [10 Map Rando compatibility](issues/10-map-rando-compatibility.md) | research | open | 09 | Later version/seed/custom-behavior contract |

Choose the first open, unblocked, unclaimed ticket by number. Readiness never
reopens an existing claim. Tickets 02, 03 and 05 can proceed together after 01
using disjoint approved ownership; 05 performs research while the tooling drill
is prepared. Ticket 04 runs first in an isolated disposable checkout/effort;
production canonical authority activates only with passing retained drill
receipts and a validated legacy bridge/cutover. Gameplay mutation through the
future workflow starts after 04 and 05. Ticket 05's lane table produces concrete
child tickets/scopes; each child lists dependencies and is added to its
milestone. Parent milestones do not make subsystem implementation serial.

Existing affected-family-assignment 03-07, Ceres 01-03 and decomp-port 78/141
remain authoritative for their original work and acceptance. Follow their
preserved claims and gates; these plan tickets coordinate continuation.

## Milestone evidence

| Milestone | Evidence needed before resolution |
| --- | --- |
| Restartable sessions (04) | Two actual sessions, exclusive claims, stopped-parent/live-child recovery, checked-stop replacement, fenced integrator transfer, retained source/evidence and exact resume commands |
| Subsystem catalog (05) | Exercised coherent scopes, reused Rust/evidence, complete reviewed dependencies, exact disjoint writable files, passing criteria and honest readiness |
| Ceres escape (06) | Ordinary reset reaches escape; causal controller order, material gameplay state/effects, exercised Rust ownership and explicit remaining C ownership |
| Vanilla progression (07) | Normal-play checkpoints along the declared route with authenticated actual-C and original-ROM comparisons; first unresolved mismatch and remaining fallback visible |
| Save/load and credits (08) | SRAM and fresh-process continuation plus a declared adapted completion route; timing adaptations and supported gameplay domains evidenced |
| Full Rust gameplay (09) | Supported boot/play/save/load/credits behavior exercised with zero gameplay C/CPU fallback; remaining untested domains and data dependencies explicit |
| Map Rando (10) | Researched version, seed and custom-code contract after vanilla ownership; implementation children and selected-seed acceptance still required |

Resolve each ticket only with required implementation/evidence pointers and
[run cleanup](../../docs/agents/run-cleanup.md). Research records its `## Answer`
and appends a pointer here; the overall spec remains open until required work
and integration evidence are complete.
