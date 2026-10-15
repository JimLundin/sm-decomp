# Verification and measurement gates

Status: implementation gates active; no whole-batch improvement or gameplay
acceptance has been established. Storage-only packing was rejected for this
iteration after ticket 05's measured comparison.

## Establish a usable baseline

Use the final integrated Offline family extraction as the implementation baseline.
Record its complete commit, exact controller/sandbox/readers/compiler identities,
flags/profile, required catalog and Fixture counts, ROM/capture hashes, cache
state, worker capacity, budgets, source ownership and host activity. Do not compare
an old installed controller against a newly optimized candidate and attribute the
whole difference to one source change.

Enable `SM_PIPELINE_METRICS=enabled`. Add low-overhead monotonic stage and reuse
timing; keep expensive resource sampling explicit and measure its overhead.
Record end-to-end wall time through post-cleanup authentication separately from
overlapping compiler/Fixture durations. Distinguish logical bytes processed,
bytes consumed by readers, allocated storage, and OS disk I/O where available;
page-cache reads are not necessarily physical disk traffic.

The previous 160/177 selection is a diagnostic observation, not a baseline for
current full verification throughput. The stripped-plan timeout and historical
disabled metrics cannot be promoted into a speedup claim. Reuse the completed
correctness evidence from the extraction where its exact source applies; do not
rerun its completed preparation just to populate this plan.

## Required scenarios

| Scenario | Required behavior | Main measurement |
| --- | --- | --- |
| Metadata/planning | Documentation-only and narrow transitive edits; genuine unknowns stay affected | Catalog reads/decodes, selection/reasons and wall time |
| Unchanged retained batch | Entire required set authenticated; reuse only where permitted | Current authentication duration, content reads and zero new producers when allowed |
| Fresh execution, warm builds | Entire required Fixture domain executes, including actual C | Scheduling, process span, evidence production/authentication and cleanup |
| Fresh execution, cold builds | Same domain with no eligible prebuilt artifacts | Unique builds versus family count, compilation and complete elapsed time |
| Large or chained family | Complete declared cases, with real continuations/callbacks/captures | Long-chain stalls, memory bounds, repeated state reads and packing cost |
| Interrupted/recovered run | Existing attempt reconciles without duplicate producers | Recovery cost, retained ownership, exact original result and cleanup |

Ordinary and Native execution are separate rows in the recorded results whenever
their contracts or producer requirements differ. Full Live replay is a separate
gate when source/admission/installation changes affect it; keep the entire
prescribed frame horizon and fresh subjects. Original-ROM parity is never inferred
from Offline or mixed Live agreement.

Start each exploratory experiment with a declared scope that fits approximately
60 seconds. Use at least three alternating baseline/candidate samples for bounded
representative workloads, with the same profile, cache seed and four-worker budget.
Record variance and concurrent activity; a timeout remains an incomplete sample.
Increase an experiment budget deliberately before launch when a larger complete
family needs it; never enlarge a production gate's frozen limit opportunistically.

Run a complete required-catalog comparison only after targeted controls and
measurements pass. Initially perform one frozen baseline and one frozen candidate
run, labeling single-run results as observations. Additional long repetitions need
a demonstrated unresolved question and a finite experiment budget; do not launch
a full repeated matrix for every intermediate change. A repeatable full-batch
speedup requires enough comparable evidence to exceed the observed variation.

Use private, reproducible cache seeds/namespaces without clearing an active shared
cache. Controlled producers continue using the shared capacity. If the harness
cannot separate cache state from capacity safely, document that limitation before
calling a measurement cold. Measure import/finalization, not only the timed middle
of the execution loop.

## Structural performance assertions

These verify removed work without depending on noisy wall-clock limits:

- One parsed catalog snapshot and at most one full declaration decode per unique
  manifest within the batch; later freshness reads are recorded separately.
- Comment tokens cannot produce dependency edges or unsupported-input refusals.
  Test known complete closures selecting zero families for unrelated documents,
  and true unknown closures staying affected.
- One compiler preparation per compatible tool/input context. One compilation
  per eligible unique build action; required distinct Native producer slots remain
  distinct until a new contract explicitly supports shared build references.
- Zero new compiler/Fixture producers on authenticated unchanged reuse when that
  policy allows it. Fresh execution still contains every prescribed producer.
- Within one independent authentication stage, each distinct immutable payload
  is read/authenticated at most once where the new representation permits it.
  Every later admission stage starts a new validation scope.
- The selected admission reader preserves physical file counts while reducing
  repeat payload reads. Report retained and temporary file counts without claiming
  a packing benefit. If direct packing is reconsidered later, its physical payload
  count must scale with bounded packs/chunks while preserving every logical case
  and producer record.
- Scheduler activity never exceeds shared capacity or aggregate reservations.
  Waiting dependencies/cache keys consume no producer slot. Existing barriers
  remain until the approved scheduling contract permits their removal.
- Standalone authentication succeeds after workers, invocation memory, temporary
  targets and integrated worktrees are gone, with exact required source retained.

For each candidate, define the smallest useful latency/resource improvement after
the baseline but before comparison. Promote only when correctness passes and the
target workload improves beyond measurement noise without a material regression
elsewhere. Reject or revise packing/cohort/daemon candidates that merely move cost
into compression, store import, cold startup or cleanup.

## Behavioral gates through the Interface

Run the same complete Fixture domains through the baseline and candidate. Compare
actual input/output bytes, returns, host state, callback order, expected captures,
case ordering and continuation inheritance. Different physical storage or producer
ids may be legitimate; compare their canonical meaning and full attribution.

Exercise these actual failure patterns:

- Changed source while waiting on a cache/build lock, changed namespace, newly
  included/generated/env input, moved/deleted input and redirected path.
- Corrupt artifact bytes, false declared digest, missing pack/page/chunk, extra
  or omitted member, duplicate case and both subjects rebound to equal wrong bytes.
- Wrong/missing Native producer, wrong attempt or candidate, missing compiler
  slot, under-reserved retained data and collector writes outside its scope.
- Rust rejection prevents C for that pair; mismatch blocks descendants; independent
  work follows the contract's failure ordering and retains its own diagnostics.
- Crash/timeout/cancellation with child descendants, helper mutation, changed ROM,
  partial framed output and interruption at commit/pointer publication transitions.
- Two competing batches, multi-slot admission, slow chain plus ready independent
  work, and cancellation while shared build or pack publication is pending.
- Reader restart, expired same-attempt recovery, missing runtime tools during
  read-only admission, and authentication after cleanup/worktree retirement.

Use the maintained family/callback/PLM/Ridley/Fixture, cache/process, evidence,
Native-attribution and Translation recovery tests after their extraction. The
three explicit Native proofs listed in the extraction's
[verification plan](../offline-family-architecture/native-verification-plan.md)
must be invoked explicitly where affected; a default suite with ignored tests
does not prove them. Replace duplicate internal tests only after their behavior
is covered through the owning Interface. Do not add tests that only assert file
layout or repeat the new implementation.

## Scope-preserving rollout

Ordinary bulk execution first preserves current recipes and evidence. Packed
outputs and Native sharing/scheduling have separate contracts and tests. Select
new behavior through reviewed implementation/configuration, not an arbitrary
caller flag that can reduce required cases or producers. At cutover, the current
producer and reader agree on one current format; add no historical conversion
programs or permissive old-format admission path.

Unimplemented adapters, broken baseline families and absent private prerequisites
are explicit external blockers to the affected final gate. Resolve them or leave
that rollout incomplete; do not choose a smaller passing catalog. Existing
Original production prerequisites retain their separate ticket/ownership.

## Cleanup is timed completion work

After all owned writers have finished or stopped, run both scoped cleanup passes
from `docs/agents/run-cleanup.md`, authenticate retained ordinary/Native results,
and use `live-authenticate` for any Live verdict. Record reclaimed bytes, remaining
space, exact source/reader/store dependencies and all failed-run diagnostics.
Preserve those dependencies before retiring implementer worktrees and build
outputs, then reauthenticate. Do not delete shared objects based on one run's
directory inventory. Include these steps in final elapsed measurements.
