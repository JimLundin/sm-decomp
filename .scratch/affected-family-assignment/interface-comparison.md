# Selection and assignment interface comparison

Design exploration. User selected C in Q9 and focused verification in Q10.
Final shared-understanding confirmation remains pending.

All designs preserve Q1–Q8 in [spec](spec.md): conservative affected-family
selection, automatic bounded translation research/promotion, local jobs/existing
subagent tools, isolated worktrees, one integration owner, exclusive shared claims
with checked recovery, and existing worker capacity.

## A: Immutable plan and advancement

Two operations: read-only `plan(request)` and mutating
`advance(plan_id, reports, policy)`. Research/source changes create successor
plans. Claims, ticket publication and conflict checks live behind advancement.

Strong depth for reviewing a fixed source snapshot; locality concentrates
selection and advancement. Ongoing research creates more plan lineage for
the caller to manage.

## B: Persistent work graph

Open/refresh/inspect/claim/record/recover operations maintain stable logical
work identities, input revisions and attempt identities. Queries expose readiness,
dependencies, source lanes and writable conflicts; findings enrich the graph.

Highest leverage for graph exploration and many evolving source lanes, but the
interface exposes more lifecycle concepts. Stable identities and revision-aware
completion are useful implementation ideas for any design.

## C: Pull queue

Open a target, request ready assignments, and record started/result/recovery
updates. The result distinguishes ready work, reconciliation, waiting and no
remaining ready work. Requests atomically reserve claims and publish local jobs
before returning launch briefs.

High depth for the ongoing coordinator: one pull yields useful owned work.
Selection, candidate ranking, research promotion, conflict handling, ticket
projection and recovery gain locality behind one seam. Existing batch verification
and subagent dispatch remain real execution adapters rather than a new launcher.

## Recommendation

Choose C with a read-only affected-family report for existing changed-since
callers. Keep B's stable work identity, input revision and attempt identity inside
the implementation; use immutable source identities for explanations and
revision checks. User selected this approach in Q9.

## Test surface

The interface is the test surface. Exercise observable selected/skipped work,
local jobs, claims, promotion, retries, conflicts and checked recovery through
the same seam as batch and the coordinator. Small actual-C/Rust families prove
preserved verification and reuse. Timing experiments remain bounded and must
report actual work avoided, not assume a prospective speedup. User accepted this
verification scope in Q10; full Live replay is required if execution/admission
contracts change.

## Completion evidence

Three independent design explorations; no implementation, build, verification
run, worktree or evidence-store write. Scoped run cleanup has no new run scope;
zero bytes reclaimed. Historical evidence remains untouched.
