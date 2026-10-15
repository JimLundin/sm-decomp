# Ticket 07 work graph handoff

Status: claimed implementation, first three public actual-C controls are red on
the unchanged scheduler. No new Native scheduling contract has been accepted.

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

Next: implement the shared ordinary executor and bounded stage projection,
retaining existing Native waves/Original ordering. Final independent readback,
both whole-proof cleanup passes and root-owned integrated worktree retirement
remain required before ticket resolution.
