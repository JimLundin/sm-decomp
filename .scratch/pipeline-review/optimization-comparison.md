# Recommendations compared with the optimization branch

Reviewed 2026-10-11 using committed Git objects only. Compared
`decomp-pipeline-performance` at `e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`
with the decomp review at `e557ab7`. Their common ancestor is `16f1883`.
The optimization worktree was not entered, and its branch was not changed.
No merge, build, replay or fresh artifact authentication was performed.

## Conclusion

The optimization branch already implements most of the infrastructure suggested
by the [earlier review](pipeline-review.md). It should become the starting point
for pipeline integration. Reimplementing worker leases, caches, narrow wrappers,
private capture stores or authenticated store import would duplicate completed
work. The earlier report describes the active decomp checkout; those limitations
remain true there until the branch is integrated.

The next work is to reconcile the branch with the newer decomp contracts, verify
that combined source, and measure its full-route costs. The original-ROM accuracy
assessment is unchanged: parallel scheduling and exact storage improve throughput
and evidence handling, rather than extending original timing/gameplay fidelity.

## Recommendation mapping

| Earlier recommendation | At optimization commit `e820a6b` | What remains |
| --- | --- | --- |
| Phase/resource measurements | Batch/command elapsed times and live worker/store wait, production, sealing and publication timings implemented | CPU time, peak RSS, swap/I/O and complete attribution of source-validation/fixture/hash/cleanup costs are not fully reported |
| Authenticated reuse and source freeze | Existing reuse retained; per-batch record locks and atomic pointers allow independent batches; changed-family selection added | Integration owner still freezes coherent source and reconciles shared contracts; selectivity needs controls on current manifests and shared dependencies |
| Compiler artifact cache | Rust `sm-pipeline` implements keyed locks, sealed artifacts, byte checks and source rechecks; evidence/decomp Cargo artifacts, RNG and pinned C objects use it | Generated C hosts/wrappers still rebuild and link; cache eviction is manual, bootstrap serialized; generic caches need explicit complete input closure |
| Narrow dependency invalidation | Canonical Wram extracted unchanged to `memory.rs`; conservative wrapper discovery plus compiler dep-info seals actual sources under `compiler-v2` | Reconcile newer modules and host bridges; broad projectile dependencies may stay broad; historical contracts stay conservative |
| Fewer offline process/file costs | Parallel independent fixture pairs and expanded manifest catalog implemented; Rust admission precedes C and records remain deterministic | Process-per-case execution remains. Persistent batching and offline packed fixtures are future work with separate state-isolation requirements |
| Concurrent live capture | Packed subjects capture in separate private stores; `--live-jobs` controls concurrency under shared leases | Plain/gzip remain sequential; latest native projectile capture/admission needs integration and full-route validation |
| Safe shared publication | Rust imports/remaps canonical objects into the shared store, authenticates source/reopened destination, publishes pointers durably and retains private dependencies on failure | Publication remains serialized/material; measure full-route consolidation and recovery. Do not copy packs or remove locks |
| Faster evidence reader/verification | `verified_inventory` combines reopened member verification and inventory collection; import returns verified member digests, eliminating repeated restoration traversals | Further index/read caching and typed transport remain possible after profiling |
| Typed event transport | Not implemented; packed path still sends C formatted text to the Rust parser | Profile after the completed improvements before replacing event protocol |
| Diagnostic checkpoint windows | Explicitly excluded from this branch's scope | Separate resumability/state-equivalence design; retain final cold replay acceptance |
| Independent original-ROM/timing checks | Existing distinction preserved in scope and verdicts | Causal adapter, APU/hardware timing, original milestones/endings and route coverage remain separate accuracy work |

Branch primary sources (all read at `e820a6b`):
`crates/pipeline/src/{main,inputs,builds}.rs`, `tools/port_pipeline.py`,
`tools/port_inputs.py`, `tools/port_family.py`, `tools/port_batch.py`,
`tools/port_live.py`, `tools/port_evidence.py`,
`crates/evidence/src/{main,run,pack}.rs`, `docs/DECOMP_BATCH.md`, and
`.scratch/decomp-pipeline-performance/implementation.md`.

[optimization-source-index.json](optimization-source-index.json) pins the
source/record blobs for reproduction with `git show <commit>:<path>`.

## How strong are the speed measurements?

The committed implementation record reports exact serial/parallel integration
agreement at 1,000, 9,000 and 45,000 inputs. For 45,000 it records matching full
inventories, observation roots, progress, transformations and measured ownership;
both final long runs authenticate 510 observation members per subject after
cleanup. It records 52 harness tests, Rust tests and strict Clippy, with existing
unrelated formatting debt stated explicitly.

Reported capture totals excluding scheduling waits are approximately:

| Horizon | Serial capture | Parallel capture | Scope |
| --- | ---: | ---: | --- |
| 1,000 inputs | 14.4 s | 5.3 s | Shared publication adds about 5–6 s |
| 9,000 inputs | 144.0 s | 58.6 s | Exact integration comparisons; concurrent load |
| 45,000 inputs | ~900 s | 233.7 s | About 3.9× lower reported capture time; publication/end-to-end time separate |

A cold evidence-reader build is reported at 90.27 s versus 0.228 s warm
restoration. A narrow system compiler experiment records 5.136 s broad versus
0.891 s narrow, with 475 full-WRAM/return cases matching. These support the
architectural priorities; neither is a general whole-pipeline speedup factor.

The long-run numbers are recorded in `implementation.md`, while checked-in
`live-results.json`, `live-lock-results.json` and `offline-results.json` describe
an earlier frozen-binary investigation. They are different phases, not one
replicated final benchmark dataset. Initial live `producer_seconds` included lock
wait; the final implementation moves its timer after admission and separates
worker/store wait. Concurrent host load was uncontrolled. Do not report the
~3.9× capture comparison as a stable end-to-end speedup, and do not substitute
45,000-input evidence for the latest 129,712-input projectile acceptance.
The long capture timings also predate the final store-before-worker lock-order
fix, which was checked in a final short run. `publication_seconds` includes
store/lease queueing and import, but excludes the later durable pointer write,
independent authentication and private-store deletion. Complete stage accounting
must include those costs.
See [the supporting measurement review](optimization-measurements.md).

## Integration with the newer decomp work

The branch starts at `16f1883`, before the subsequent projectile ownership,
rendering/contact and WithGfx changes. Both branches edit shared harnesses,
`crates/port/src/lib.rs`, the batch catalog and workflow docs. Its passing
benchmarks therefore certify its frozen pipeline/gameplay combination, rather
than the current decomp tree.

Three concrete integration requirements are visible without attempting a merge:

1. **Preserve dependent fixture chains.** Current decomp `port_family.py` has
   `execution_routines`, `case_order`, `continue_from`, inherited full WRAM and
   `projectile_host_words`, with continuation verification during authentication.
   Optimization `parallel_cases` submits a flattened case list and reads explicit
   `host_words`; these newer continuation facilities are absent there. Parallelize
   independent cases/chains, retain ordered steps within each chain, and keep
   inherited all-eight-word host/state authentication. Parallelizing a dependent
   chain step as an independent case would break the actual oracle experiment.
2. **Preserve the new native projectile contract.** Current decomp adds
   `--projectile-verdict`, a three-word WithGfx bridge, transactional eight-word
   host publication, all-nine scheduler and 16/4/4 allocation gates, and material
   captures. Optimization live code predates that bridge. Integrate its private
   producer scheduler around the current wrappers and counters; compare all
   native ownership/capture requirements before publishing shared pointers.
3. **Retain public modules and source identities.** Apply memory extraction while
   keeping current module declarations and tested Wram behavior. Update wrapper
   discovery/dep-info and manifests for new shared modules. Cargo.lock/workspace,
   harness and ABI changes invalidate affected seals. Regenerate source-dependent
   predecessor/family evidence; retain old runs under their exact historical
   identities rather than rewriting acceptance.

Newer frame, initializer, chain, contacts and rendering families total 13,624
cases, plus 1,674 supplemental Ceres cases in the decomp checkpoint. These are
materially different from the optimization final system/audio/dispatcher/
animated-tile batch (475/1,147/342/43 cases). Its expanded offline catalog is a
useful starting point, but does not by itself verify those newer families or
make discovered manifests live eligible.

## Updated parallel setup

Use the branch's actual controls after integration:
`SM_PIPELINE_ROOT` selects the shared scheduler/cache directory;
`SM_PIPELINE_WORKERS` fixes its process capacity; `--jobs` schedules families,
`--case-jobs` schedules isolated fixture pairs and `--live-jobs` schedules the
four fresh packed subjects. Separate batch records permit separate owners.
All users of one shared root must use the same capacity; changing capacity
requires a new root. OS leases release after worker death, and publication
acquires store ownership before a CPU lease so blocked importers do not consume
the worker budget.

On this four-core/roughly-11-GiB host, begin with shared capacity two while other
heavy work continues, then compare two versus four on a quiet machine. The branch
defaults to at most four. This is a total leased-task budget, not a guarantee of
one CPU per token: a packed subject runs a producer and collector, and CPU/RAM
usage varies across tasks. No measured memory-weighted scheduler or distributed
worker implementation is demonstrated. Avoid multiplying family/case/live
parallelism, and monitor peak RSS, swap and I/O before increasing capacity.

Keep the earlier team structure: one integration owner, two implementation
workers on separate coherent boundaries and one independent reference/review
worker. The branch provides process isolation, budget and publication mechanisms;
file ownership, interface coordination, source freeze and independent correctness
review remain team responsibilities.

## Revised next steps

1. Integrate the completed pipeline changes while preserving current dependent
   chains, native projectile host ABI/counters/captures and module declarations.
   Integration is proposed future implementation work.
2. Verify combined current-source Rust/harness tests and actual-C boundary,
   continuation and negative controls; check unrelated/relevant dependency
   invalidation and cache corruption/interruption against current manifests.
3. Compare serial/parallel exact evidence through first combat, then perform the
   129,712-input acceptance with all nine IDs, 16/4/4 allocations and full
   WRAM/host captures. Record source freeze, final cleanup and authentication.
4. Record machine-readable final total wall time, builds, queue, producer,
   sealing, import and post-publication authentication; add CPU/RSS/I/O and
   cold/warm cache/store conditions. Use this to choose worker limits.
5. Prioritize remaining typed-event, process/file and reader costs from that
   profile. Continue original-ROM adapter and hardware accuracy independently.

No new run artifacts, caches, extraction or worktrees were created for this
comparison. Cleanup scope is documentation only; existing evidence and the
optimization worktree/branch remain intact.
