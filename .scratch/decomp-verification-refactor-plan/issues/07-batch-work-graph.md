# Schedule shared batch work through one private graph

Status: claimed
Blocked by: 04, 06
Triage: ready-for-agent
Type: task

## Scope and ownership

Own private batch action dependencies and scheduling in `offline-verification`,
the Original/Native batch integration in `pipeline`, and required reservation
integration with `tool-execution`/`translation-execution`. The latter keeps durable
operation and attempt authority; this ticket creates no replacement ledger.

## Required work

Use recipe-owned nodes for preparation, eligible shared builds, Fixture
pairs/groups, comparison and publication. Reuse current ready/dependent queues and
the four-slot lease pool. Respect each Fixture chain, Rust-before-C admission and
canonical output order. Bound concurrent hashing, packing, helper memory and
output reservations as well as producer count.

Represent current Native wave groups and Original family ordering first. Removing
barriers or admitting families concurrently requires an explicit new scheduling
contract, with atomic group reservations and preserved complete producer coverage.
On failure/cancellation, stop new admission, settle owned in-flight work and retain
the exact diagnostics; never launch a blocked continuation. Avoid partial-reserve
deadlock and holding slots while waiting on a cache or predecessor.

Persist graph progress as a projection of sealed plan and existing operation
records. Recovery reauthenticates results and reconciles live/ambiguous owners;
it does not rerun a producer because a cached graph node is missing or extend the
original deadline. Read-only graph summaries grant no authority.

## Acceptance

Actual-C tests show independent families making progress around a long chain,
capacity never exceeded, identical full results and deterministic reporting.
Exercise group failure, cancellation, missing projection, wrong attempt, crash
before/after publication and two competing batches. Native producer cardinality,
ownership and aggregate budgets remain complete under the selected contract.
Compare elapsed/lease-idle time on the same mandatory family set.

## Completion evidence

Pending. Record graph/producers, complete semantic controls, scheduler/capacity
measurements, unchanged recovery lineage and scoped cleanup/authentication.

## Comments

2026-10-15 checkpoint `00edc00` compiles the canonical serial Native sequence
composition and bounded advisory progress. Root approved only a fixed 64-KiB
advisory allowance inside the existing parent/max output limit for new intents.
It covers a 32-KiB file and atomic replacement. Absent/zero retained intent fields
preserve their original reservation/receipt semantics and write no new sidecar.
Existing operations, waves, saved deadlines and source/attempt bindings remain
authoritative. No producer slot, limit, reservation API or ownership change occurs.

The preceding ordinary checkpoint `fc2cf3a` passes five public actual-C controls:
terminal rejection, publication error, publication panic/reaping, complete
48-pair overlapping chains and two competing isolated batches. Peak resident
helpers fell 16 to eight at unchanged producer peak four; the single elapsed
sample changed 8.332 to 8.562 seconds and establishes no latency improvement.
Root review requires shrinking conservative completed-result reservations before
final acceptance: retaining 16 KiB per all 150,204 cases would itself exceed the
standard 1-GiB context. Maximum-window helper reuse/Fixture time is also being
checked so bounded residency does not hide repeated helper churn. Native
projection/tamper/recovery and cancellation controls remain with the worker.

The graph is private implementation detail; callers do not submit arbitrary tasks.

2026-10-15: root assigns the implementation to `/root/dependency_facts_resume`
after that worker completes ticket 03's corpus cleanup and releases its target.
The tested foundation is `b1f869102ac69897be674acfef682e3d867c6be6`, with
02/03/04/06 owner, catalog and actual-C/Native controls passing. Final ticket
acceptance still requires integrated retirement and the complete final gates.

Worktree: `/home/jim/sm-decomp-performance-graph`, branch
`performance/batch-work-graph-20261015`. Exclusive target after the explicit
03 handoff: `/home/jim/sm-decomp-performance-metrics-target`. Private proof:
`/home/jim/sm-decomp-performance-graph-proof`. Parent session:
`01a13e7f-ab01-7413-a1c2-288b08a4b719`.

Owned implementation: private graph module and bulk/Fixture scheduling in
`crates/offline-verification`, Original composition in
`crates/pipeline/src/translation_original.rs` and narrowly necessary batch
composition, plus related tests. Any reservation API change in
`tool-execution`/`translation-execution` must be coordinated with root first.
Root owns Ceres recipe construction and final source registration; ticket 08
owns Cargo profiles and tool startup. Preserve the separate optimized Native
operation admission during final integration. No installed policy or production
claim changes belong to this ticket.
