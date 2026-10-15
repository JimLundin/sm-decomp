# Schedule shared batch work through one private graph

Status: open
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

The graph is private implementation detail; callers do not submit arbitrary tasks.
