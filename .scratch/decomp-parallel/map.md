# Restartable parallel decomp work

Status: open
Blocked by: 06, 07, 08, 09, 10

Production activated 2026-10-13. Read [current launch status](launch-20261013/status.md)
before historical preparation. Research 13/14 is resolved; children 21–24 keep
actual exercise/hardware/Samus dependencies explicit. Original Ceres integration 11
is claimed under retained authority and remains the first merged-source task.
Plan deliverable: complete; implementation remains open

Preparation: tickets 01–05 resolved; production launch activated
Preparation coordinator: `01a13559-e517-72d1-88b5-e554fed61da4`

The [spec](spec.md) defines the outcome and authority transition. The
[session runbook](../../docs/agents/decomp-sessions.md) contains supported
commands, subagent dispatch, WIP/handoff rules and fresh/resume prompts.
The [committed checkpoint](checkpoint-20261013.md) preserves recovered source
and evidence. The [preparation status](preparation/status.md) records the current
tooling, actual-session drill and launch boundary.
The [launch handoff](preparation/launch-handoff.md) records the current retained
subject, passing final receipts and later launch procedure.

## Decisions so far

- Source/receipts are checkpointed at
  `d8a90451695408bbf72017ea04e7f4e915b06dd9`. Passing evidence and a Git
  merge do not resolve pending durable integration.
- Production authority is active at `.scratch/decomp-parallel/authority.toml`
  with canonical queue `.scratch/decomp-parallel/ledger.toml`, capacity `3`.
  Held Ceres work retains `.scratch/ceres-continuation/ledger.toml`, coordinator
  `ceres-20261013` and its original claim tuples/submissions. Current real role
  owners and the background handoff are recorded in the launch status.
- All workers use `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
  `SM_PIPELINE_WORKERS=4`; these four process workers are shared across sessions.
  Delegate actual work to subagents. Keep branches, writable files, run paths
  and handoffs independent; one identified integrator owns shared mutations.
- Research and independent implementation run concurrently. Final registration,
  merged-source verification, stage publication and Git/index mutations serialize
  under one durable integration owner.
- Preserve the actual-C and original-ROM admission contracts. Scope counts,
  WIP commits and diagnostic prefixes describe bounded progress.
- Ticket 01's retained owner bridge and exact tooling adoption are checkpointed
  at `c46b730` on `decomp-preparation-20261013`; normalized successor bridge
  retains its immutable predecessor. Tickets 02/03 are integrated and resolved;
  production authority and all original gameplay submissions remain held.
- The corrected completed-integration replay source is
  `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`, retained at
  `/tmp/sm-decomp-preparation-retry-source-20261013`. Execution and validation
  use the same engine; [preparation status](preparation/status.md) owns its
  exact fingerprint/build/seals. `drill-final` A replay failure after B's merge
  remains historical; the Rust fix/test is integrated as `0d042e0`.
- [Ticket04](issues/04-concurrency-and-crash-recovery-drill.md) resolved with eleven
  actual controls, two sequential authenticated integrations and both completed
  A/B replays at final HEAD, private readiness/host preservation, measured cleanup
  and reauthentication. Gameplay launch and production activation remain the
  user's separate request. The explicit post-publication interruption and
  same-UUID resume passed with both exact replays and zero gate reruns.
- Ticket 05's [catalog](catalog/subsystem-catalog.md) identifies reused acceptance,
  exact held Ceres continuation and child research 11–20. Native promotion still
  accepts bank86 projectile exercise provenance; other observed callers need a
  reviewed evidence-contract extension. Research completion generates exact
  implementation blockers before any gameplay milestone can be claimed.

## Tickets and parallel order

| Child | Type | Status | Blocked by | Lane |
| --- | --- | --- | --- | --- |
| [01 Reconcile legacy ownership](issues/01-reconcile-legacy-ownership.md) | task | resolved | None | Retained legacy bridge and checked isolated tooling adoption |
| [02 Fence writable paths and integration sessions](issues/02-fence-writable-paths-and-integration-sessions.md) | task | resolved | 01 | Integrated Rust ownership engine; focused controls and cleanup complete |
| [03 Maintain Rust commands and durable handoffs](issues/03-maintained-rust-commands-and-handoffs.md) | task | resolved | 01 | Integrated typed Rust session commands; focused controls and cleanup complete |
| [04 Prove concurrent sessions and crash recovery](issues/04-concurrency-and-crash-recovery-drill.md) | task | resolved | 02, 03 | Corrected actual drill/readiness/cleanup passed; production launch remains separate |
| [05 Catalog dependency-ready subsystem scopes](issues/05-subsystem-scope-catalog.md) | research | resolved | 01 | Published catalog and children 11–20; honest research/held readiness |
| [06 Ordinary-reset Ceres escape](issues/06-ordinary-reset-ceres-escape.md) | task | open | 04, 05, 11, 12, 13, 21, 22, 23 | Existing controller/host continuation and required scheduler/hardware contract |
| [07 Playable vanilla progression](issues/07-playable-vanilla-progression.md) | task | open | 06, 13, 14, 15, 16, 17, 18, 21, 22, 23, 24 | Parallel subsystem children; composed route gate |
| [08 Save/load and credits](issues/08-save-load-and-credits.md) | task | open | 07, 13, 18, 19, 20, 21, 22, 23 | Persistence and completion children |
| [09 Complete Rust gameplay ownership](issues/09-full-rust-gameplay-ownership.md) | task | open | 08, 14, 15, 16, 17, 18, 19, 20, 21, 24 | Remaining coherent ownership gaps |
| [10 Map Rando compatibility](issues/10-map-rando-compatibility.md) | research | open | 09 | Later version/seed/custom-behavior contract |
| [11 Finish held Ceres integration](issues/11-finish-held-ceres-integration.md) | task | claimed | 04, 05 | Tracking continuation of original host/controller claims |
| [12 Ceres NMI/poll continuation](issues/12-ceres-nmi-poll-continuation.md) | research | open | 11 | Diagnose exact extra-poll boundary under ticket 78 |
| [13 Host/hardware lifecycle scope](issues/13-host-hardware-lifecycle-scope.md) | research | resolved | 05 | Complete NMI body researched; children 21–23 retain implementation gates |
| [14 Samus frame scope](issues/14-samus-frame-scope.md) | research | resolved | 05 | Samus beta frame researched; children 21/24 retain implementation gates |
| [15 Door/room/PLM scope](issues/15-door-room-plm-scope.md) | research | open | 05 | Preserve camera and establish whole transition |
| [16 Continue held whole Ridley](issues/16-continue-held-whole-ridley.md) | task | open | 04, 05 | Tracking continuation of original ticket 141 |
| [17 Other enemy lifecycle scope](issues/17-other-enemy-lifecycle-scope.md) | research | open | 05 | Disjoint non-Ridley complete family |
| [18 Audio/UI/menu scope](issues/18-audio-ui-menu-scope.md) | research | open | 05 | Ordered audio acknowledgments and menus |
| [19 SRAM/save/load scope](issues/19-sram-save-load-scope.md) | research | open | 05 | Native save transaction and fresh-process continuation |
| [20 Credits cinematic scope](issues/20-credits-cinematic-scope.md) | research | open | 05 | Whole credits root, data and route dependencies |
| [21 Native-root exercise provenance](issues/21-native-root-exercise-provenance.md) | research | open | None | Reviewed source-authenticated NMI/Samus root evidence extension |
| [22 NMI hardware service adapter](issues/22-nmi-hardware-service-adapter.md) | research | open | 11 | Full-state synchronous MMIO/DMA adapter contract |
| [23 Complete NMI body scope](issues/23-complete-nmi-body-scope.md) | research | open | 11, 21, 22 | Complete eleven-routine scope after exercise and adapter admission |
| [24 Samus beta-frame closure](issues/24-samus-beta-frame-closure.md) | research | open | 21 | Whole movement/contact boundary and dynamic dependency closure |

Continue the activated queue by selecting the first open, unblocked, unclaimed
ticket by number. Readiness never reopens an existing claim. Tickets 01–05 are
complete. The retained isolated drill, exact readiness receipts and freshly
validated roles/legacy bridge authorized production activation at the user's
launch instruction. Ticket 05's lane table produces concrete
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
