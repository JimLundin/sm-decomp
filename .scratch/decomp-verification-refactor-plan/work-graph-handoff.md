# Ticket 07 work graph handoff

Status: substantive source ready at `4ed5f33`, awaiting the remaining scoped
controls, finite latency comparison, final cleanup and root-owned integration.
Ordinary graph and serial Native composition are implemented; no new Native
scheduling contract has been accepted.

- Owner: `/root/dependency_facts_resume`, parent session
  `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
- Authoritative scope: ticket 07 on main, including its 2026-10-15 ownership comment.
- Original claim baseline: `b1f869102ac69897be674acfef682e3d867c6be6`.
- Parent-approved clean source foundation after fast-forward:
  `45b89f844df928478b890c4b4814208dfe44bfb8` (includes optimized Native admission
  and Ceres recipe construction).
- Worktree: `/home/jim/sm-decomp-performance-graph`.
- Branch: `performance/batch-work-graph-20261015`.
- Proof: `/home/jim/sm-decomp-performance-graph-proof`.
- Exclusive target: `/home/jim/sm-decomp-performance-metrics-target`, transferred
  from completed ticket 03 by root. The prior required binaries are retained.
- Environment: `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`,
  `SM_PIPELINE_WORKERS=4`, locked/offline Cargo, one build job, dev/test debug 0,
  proof-local TMPDIR. Dependencies are read-only links to main.

## Approved implementation seam

Keep the closed `OfflineFamilies` batch interface. A private graph owns ready
work, stage dependencies, canonical outcome order and invocation cancellation.
Ordinary families retain their requested family window while submitting owned
Fixture work to one bounded executor, instead of multiplying a helper/thread
pool per family. Each actual pair retains Rust-before-C admission, its existing
shared worker lease, complete comparison and predecessor equality. Existing
`Preparation` owns eligible build-key coalescing and waits without producer
leases. Prepared/build/publication/admission stages remain explicit graph nodes.

On terminal batch family/publication failure or cancellation, stop new admission
and settle every admitted writer. Retain blocked/failure diagnostics and join
helpers before finalizing their family metrics/evidence. Bound resident helpers,
queued metadata and concurrent verification/readback work independently from
the shared cross-process producer count. Test bounds using actual process
records, not only scheduler counters.

Native stays on its current indivisible waves and atomic reservation groups.
Original stays in canonical serial family order. A closed prepared-Native
sequence facet should own that composition without exposing caller-supplied
graph tasks or enabling parallel Native admission. No reservation API change is
planned; root must be consulted before any such change. Root owns Ceres recipe
construction and source registration; ticket 08 owns profiles/tool startup.
Other workers' changes must be preserved.

Progress files are bounded advisory projections of frozen requirements and
existing operation/results. They grant no attempt, lease or producer authority.
Missing/wrong graph metadata must never rerun a Native producer, extend a saved
deadline, accept another attempt or hide an ambiguous owner. Independent
admission continues to reopen required evidence.

## Controls and next steps

First controls exercise actual-C batch admission after a rejected family or
publication failure and aggregate helper/process bounds. Then prove independent
families and chains progress, full semantic equality and canonical reports,
two competing batches sharing four slots, cancellation/descendant cleanup, and
Native/Original recovery with missing projections and publication interruptions.
The existing standalone independent-case diagnostic behavior remains a contract
to preserve while applying stop-admission to the batch graph.

## Baseline feedback loop

`cargo test -p offline-verification --locked --offline --test bulk_inputs graph::
-- --nocapture --test-threads=1` reaches all three intended failures on the
unchanged scheduler. Two controls show family b starting after a terminal
comparison rejection or observer publication failure. The complete 48-pair
actual-C workload observes four active producers but **16 resident Fixture
helpers**, exceeding two per shared worker. Full chain observations agree before
the bound assertion. Baseline batch elapsed time was 8.332222724 seconds; this
single sample is not a speedup claim.

The log is `/home/jim/sm-decomp-performance-graph-proof/red-controls.log`.
The exact reader/test executable is retained at
`/home/jim/sm-decomp-performance-graph-proof/tools/bulk-inputs-baseline`.
Both cleanup passes ran for every finished successful/rejected control, including
the initial rejection assertion correction; per-project cleanup output remains
under `proof/red/`. No build/test process remains active at this checkpoint.

## Resumed checkpoint, 2026-10-15

Substantive source commit: `4ed5f33`. It includes the prior ordinary/Native
graph commits through `00edc00`, bounded helper-locality/result-memory work in
`c08c729`, the generated PLM correction (`2bebe14`, `9dea09f`), source-contract
and complete-diagnostic correction `d27aca4`, and the approved ordinary capture
phase correction `4ed5f33` (upstream `2b0e206`). Root owns final assembly and
shared source registration; its assembled candidate is `6bc0a78`.

- Graph modules are inlined into the already registered `bulk.rs`/`native.rs`
  paths. Strict implementation input authentication is unchanged.
- One shared executor bounds resident helper pairs by worker capacity. A finite
  eight-pair locality quantum does not advance or renew a bypassed fair turn.
- Provisional result reservations shrink to actual retained record/String/Vec
  capacities. Long canonical errors grow their reservation; error formatting is
  counted without a temporary full buffer. Rejection records cross the graph
  seam with their error instead of being loaded as an unaccounted second copy.
- Complete primary diagnostics remain in canonical outcomes. Bounded advisory
  errors are not copied into later blocked canonical outcomes.
- The two-subject BTree leaf allowance is independently checked with the real
  allocator for all four concrete value types, not only the accounting formula.

Passing checks at this checkpoint:

- Long actual-C WRAM rejection, including an 8,192-character distinguishing
  symbol tail and its complete rejection CaseRecord: first red in
  `long-diagnostic-red.log`, then green in `long-diagnostic-green.log` and
  `4ed5f33-controls.log`. Each public Project ran both cleanup passes.
- Real full-catalog capture plus worst-eight-active reservation:
  `4ed5f33-full-catalog-memory.log`; 177 families, 150,204 cases,
  348,313,314 captured/base bytes + 393,630,334 completed result bytes +
  277,268,796 extra active bytes = **1,019,212,444 bytes**, below the unchanged
  **1,073,741,824-byte** resident bound. This uses the complete real public
  capture; it does not substitute diagnostic manifests or increase the limit.
- Two absent-field expired saved Native failure controls:
  `4ed5f33-expired-replays.log`.
- Actual historical Native replay: `4ed5f33-legacy-native.log`; eight unchanged
  producers, original saved deadline/receipt, no new projection or allowance.
- New Native recovery projection/tamper/publication controls and serial family
  failure-prefix retry: `4ed5f33-controls.log`; automatic cleanup and independent
  retained Native prefix reader pass. No repeat of the full Native matrix.

The same public control set exposed a test-setup issue: its synthetic Rust
fork/filesystem/process imports were rejected by the existing strict source-fact
contract before scheduling. A test-only correction now uses a permitted Rust
yield loop, pinned C fork/sleep, an independently completed actual-C case, and
trusted controller `/proc` observations. Production admission remains unchanged.

Remaining checklist:

1. Finish corrected cancellation control, checkpoint its test-only changes.
2. Root runs the five already requested standalone family regressions using its
   combined target; no duplicate pipeline build in this worktree.
3. Run exactly three alternating baseline/candidate maximum-window pairs with
   `SM_PIPELINE_METRICS=enabled`, four shared producer slots, all 192 Fixture
   pairs, fixed profile/workload, and recorded host/cache conditions.
4. Retain exact readers/tools and evidence, run both whole-proof cleanup passes,
   authenticate retained proofs, then hand root the final source/test commits
   for integrated worktree retirement.

Earlier maximum-window diagnostic samples remain in `window-maximum-before.log`
and `window-maximum-affinity.log`: helper starts fell 322 to 56, but Fixture wall
rose 52.761 to 55.935 seconds in that single pair. This is a resource reduction,
not established latency improvement. The final finite comparison remains open.

## Final CLI harness follow-up

Root reproduced the four legacy harness failures against byte-identical retained
production CLI `6bc0a78`; the original Rust-before-C rejection test passes.
Evidence is under
`/home/jim/sm-decomp-performance-final-proof/c0cc7fc/logs/family-red-status.tsv`
and the corresponding exact-test-name `.red.log` files. At root's request, the
four tests in `crates/pipeline/tests/family.rs` now move their rendezvous/fork
instrumentation into pinned actual C or trusted `/proc` observations. Both
fresh/fork scheduling modes, canonical completion/continuation order, independent
failure diagnostics, cancelled-Rust-before-C and active/idle helper/descendant
reaping remain asserted. Cancellation uses the same four-case setup that passed
the graph control. Production code and runtime capability admission are unchanged.

This follow-up edits only that test file and this handoff. Root owns its actual
recompiled tests. The worker ran no Cargo or producer command and made no target
changes for this follow-up; formatting and `git diff --check` pass. Timing remains
queued for root's explicit quiet-window notice.
