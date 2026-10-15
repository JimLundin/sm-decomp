# Alternative B: durable verification action graph

Status: design only. Baseline: main `5c011eb`; implement after the active
`offline-verification` extraction, without changing its worktrees or claims.

## Recommendation and Interface

Make a durable, content-addressed verification graph the batch execution
Implementation behind `offline-verification`. Its Interface describes required
verification, exposes an inspectable plan, and authenticates completed evidence.
It does not accept arbitrary commands or confer Translation scope approval.
`pipeline` retains selection, installed policy, Translation attempt identity,
integration and publication. The existing Translation journal authorizes and
records Native execution.

This Module gives batch callers Leverage over shared preparation, compilation,
Fixture chains and evidence reads. Its Depth comes from hiding dependency
resolution, scheduling and recovery; Locality comes from keeping graph recipes
beside family recipes. The external Seam is the complete verification request,
not a collection of publicly editable task nodes.

```rust
pub struct VerificationPlan { /* sealed description; private constructors */ }
pub struct AuthenticatedVerification { /* bound requirement and evidence */ }
pub enum Evaluation { ReuseAccepted, FreshProducers }
pub enum Progress { Pending(RunReference), Finished(RunReference) }

impl OfflineFamilies {
    pub fn plan(&self, request: PlanRequest) -> Result<VerificationPlan>;
    pub fn run(&self, plan: &VerificationPlan,
               context: &VerificationContext) -> Result<Progress>;
    pub fn authenticate(&self, run: &RunReference,
                        required: &VerificationRequirement)
        -> Result<AuthenticatedVerification>;
}

let plan = offline.plan(PlanRequest {
    required: required_catalog,
    source: frozen_source,
    evaluation: Evaluation::FreshProducers,
})?;
emit(plan.summary())?; // dependencies, unknowns, sharing, estimated costs
let progress = offline.run(&plan, &existing_batch_context)?;
// After writer teardown and the existing cleanup workflow:
let accepted = offline.authenticate(progress.finished()?, &requirement)?;
```

The context comes from existing ordinary or confined execution machinery; its
policy can strengthen freshness, never weaken frozen requirements. Plan summaries
are read-only. Existing `verify` and prepared Native calls wrap this protocol.
Planning establishes no acceptance. Finished includes retained rejection;
Pending preserves outstanding ownership. Changed inputs, invalid declarations
and unavailable enforcement are errors. Authentication launches no producer and
needs no runtime installation beyond the retained admission dependencies.

## Actions, keys and receipt scope

Use a closed typed action vocabulary, extensible by trusted recipe code:

1. Freeze source/catalog and discover dependencies, including generated inputs.
2. Prepare compiler/runtime inputs; compile objects, harnesses and executables.
3. Execute Fixture pairs or bounded groups, with explicit Fixture chain edges.
4. Compare, inventory, pack evidence and construct family admission records.

The graph key hashes canonical, versioned action descriptions: recipe and ABI,
complete logical input paths and bytes, dependency keys, compiler/helper/runtime
identities, target, flags, declared environment, capture/ROM identities, Fixture
declaration and comparison rules. Exclude output destinations only where the
recipe proves relocation invariance; paths affecting `__FILE__` or generated
code remain inputs. Preserve existing broad source requirements until a separate
dependency proof supports narrowing them.

Distinguish three identities:

| Identity | Meaning |
| --- | --- |
| Content key | This exact computation and reusable artifact closure |
| Execution binding | This run, gate, Translation attempt and required producer ordinal |
| Admission receipt | This requirement, coverage, provenance and exact producer/result references |

Matching content cannot manufacture fresh execution. `FreshProducers` schedules
every required Rust and actual-C invocation again; cached C outputs never satisfy
it. Identical Fixture pairs in different required families retain their own
execution obligations. Completed operations may reconcile within the same
attempt without relaunch. Cross-attempt accepted-result reuse requires the
existing policy and full admission. A new run may reuse compiled artifacts only
where its contract permits artifact reuse.

Current Native admission expects per-family `c-build`, `rust-build`, optional ABI
controls and every Rust/C case operation
([translation_native.rs:1242](/home/jim/sm-decomp/crates/pipeline/src/translation_native.rs:1242)).
Initially retain those producers even when their content keys match. Replacing
them with shared build references is a dedicated current-format contract change,
including reservation and complete indexed attribution—not an optimization flag.
Family receipts remain complete, while batch receipts reference their exact
coverage and execution bindings. Live replay consumes authenticated declarations
and retains its separate acceptance gate.

## Immutability and authentication

A digest is an identity, not an authority or a filesystem write barrier. Freeze
committed source from Git objects; use existing candidate ownership fences for
mutable source. Open regular inputs without following aliases, copy and hash
their actual bytes into private staging, validate directory membership and all
dependencies, then publish a sealed snapshot atomically. Producers receive those
captured bytes through read-only confined mounts. If coherent mutable capture
cannot be established, refuse rather than treating timestamps as proof.

Recheck source and namespace bindings after lock waits and before acceptance.
Dependency discovery uses Rust syntax and compiler dependency inventories;
comments do not create imports. Includes, environment, generated files, macros,
captures and unresolved discovery must have complete declared inputs or leave the
Offline family affected with a conservative closure. Compiler discovery that
adds an undeclared input refuses admission and requires a new checked plan; it
cannot patch a launched plan or automatically retry a Native attempt. Never
equate an empty discovered set with independence.

Reuse the authenticated artifact cache's input/compiler/environment rechecks and
destination validation
([build_cache.rs:244](/home/jim/sm-decomp/crates/tool-execution/src/build_cache.rs:244)).
Quarantine an invalid cache entry under its key lock and rebuild only when the
current execution policy allows it. Conflicting outputs for one computation key
block reuse pending diagnosis. Corrupt accepted evidence causes authentication
failure; rebuilding equal bytes cannot silently repair its provenance or restart
a stopped attempt.

Inside one authentication call, retain bounded validated bytes or immutable
descriptor-backed snapshots and share their inventory/comparison results. Scope
that capability to the exact evidence root, requirement and read budget.
Independent admission, restart and post-cleanup authentication reacquire and
authenticate their own closure. There is no persistent “trusted hash” database.
The installed controller remains pinned to its exact executable even if reusable
content keys eventually omit unrelated CLI source.

Reuse `evidence::Store`, canonical images/trees and typed receipts. Workers write
isolated packs; a bounded coordinator serializes authenticated imports and pointer
publication. `Store::import_store` exists, but its single writer lock and mutable
read cache preclude assuming a freely shared concurrent store
([pack.rs:71](/home/jim/sm-decomp/crates/evidence/src/pack.rs:71)).
Publish pointers only after committed objects authenticate. Retain required store
packs, source snapshots and exact readers after cleanup; garbage collection must
not guess liveness from one batch directory.

## Scheduling and durable reconciliation

Schedule ready actions across all required families under the existing four-slot
`tool-execution` lease pool. Cache-lock waits hold no producer slot. Preserve
existing helper accounting and avoid multiplying family concurrency by case
concurrency. Admit all-or-nothing multi-slot reservations, inherited deadlines,
bounded temporary/output extent, retained bytes and trusted-read accounting.

Each Fixture pair runs fresh Rust first, admits its output, then invokes actual C
and compares complete WRAM, return values and ordered callbacks. Only a matching
predecessor releases a Fixture chain continuation. Reuse pristine fork helpers
while retaining isolated process state, statics, host state and failure teardown.
Record canonical declaration order regardless of completion order.

The first implementation schedules existing bounded Native groups as indivisible
actions. Removing their all-wave barriers requires a separately verified
reservation/failure protocol
([fixture_execution.rs:41](/home/jim/sm-decomp/crates/pipeline/src/fixture_execution.rs:41)).
Cross-family readiness can already improve utilization over Original acceptance's
sequential family loop
([translation_original.rs:700](/home/jim/sm-decomp/crates/pipeline/src/translation_original.rs:700)),
subject to its frozen gate ordering and an explicit scheduler integration.

Persist the sealed plan and an execution binding before launch. Native actions
use existing immutable operation intents, results, process identities and
`ExecutionPending`; the graph's ready/completed index is a reconstructible
projection, never another authority
([operation.rs:805](/home/jim/sm-decomp/crates/translation-execution/src/operation.rs:805)).
Ordinary actions retain bounded intent/result facts under existing batch
ownership; these records carry no Translation scope authority. Recovery derives
progress from authenticated operation records. It neither steals a live lease
nor extends the saved deadline.
Terminal failure, timeout and cancellation remain durable, bounded outcomes;
ordinary retries follow explicit run policy and Native retries remain deliberate
supervisor actions. Cancellation stops admission, tears down owned writers and
preserves an already selected finalization disposition.

Packing, retention and cleanup also reconcile exact published references. Native
indexes must still prove every required producer, reject duplicates and omissions,
and authenticate reservation accounting
([translation_native.rs:1181](/home/jim/sm-decomp/crates/pipeline/src/translation_native.rs:1181)).
The graph does not replace that index with an unbound aggregate success.

## Ownership, migration and performance gates

`offline-verification` owns graph recipes, dependency facts, Fixture semantics,
comparison and admission. `decomp-source` owns Recovered C source authentication
and source inventory. `tool-execution` owns artifact preparation, bounded process
execution and shared capacity. `translation-execution` owns confined durable
operations and attempt reconciliation. `evidence` owns content storage; `pipeline`
owns approval, affected selection, batch composition and cleanup orchestration.

Most graph logic is in-process. Filesystem, compilers and confinement are real
local dependencies exercised through temporary workspaces. The ordinary and
confined execution Adapters justify a private Seam; do not expose a generic
plugin framework. No network Adapter or daemon is required. Tests cross the
verification Interface; replace duplicated shallow scheduler tests once their
behavior is covered there.

Implement in stages after the extraction:

1. Add phase metrics and one authenticated source/catalog snapshot, then precise
   affected discovery. Preserve genuine unknown fallback and stable source checks.
2. Introduce graph planning and execution for ordinary batches, preserving current
   receipts. Share authenticated builds and preparation across the full catalog.
3. Add durable Native scheduling with unchanged producer obligations. Separately
   integrate build sharing, reservation changes, reduced barriers or packed
   Offline receipts under reviewed current-format contracts where justified.
4. Freeze an optimized controller and matching sandbox installation through the
   existing policy workflow. Do not modify active pinned binaries in place.

Measure matched frozen revisions under controlled load for documentation-only
planning, a real transitive edit, warm full-required-catalog authentication, cold
full-required-catalog fresh execution, and interrupted/resumed execution. Report
wall time, critical path, CPU, peak memory, bytes read/written, action/cache counts,
lease waits, producer count, authentication and cleanup separately. Use at least
three comparable observations and declare variance; acceptance requires preserved
coverage plus repeatable full-catalog gains exceeding that variance.

Correctness gates cover input mutation/rename/alias races, unknown dependencies,
cache poisoning, missing/tampered evidence, wrong producer identity, duplicate or
omitted indexed operations, chain mismatch, callback ordering, crashes at every
publication transition, timeout, cancellation, shared capacity and authentication
after cleanup and checkout retirement. No historical readers or receipt migration
layer are part of this work.

## Benefits and limits

Cold full-catalog gains come from one discovery pass, one compilation per allowed
unique artifact key, cross-family scheduling and fewer duplicate evidence reads;
they do not depend on excluding families. Native contracts requiring distinct
compiler producers cap that benefit until explicitly revised. Warm gains add
authenticated artifact/result reuse where allowed.

The current assessment measured 160/177 families selected on documentation-only
tracked edits, with 158 first refusals in comments; three catalog loads and
duplicate authentication are observed costs. It established no controlled new
end-to-end speedup
([findings.md](/home/jim/sm-decomp/.scratch/decomp-batch-performance-20261015/findings.md)).

Compared with bulk-only execution, this design offers more reuse, introspection
and durable recovery at the cost of key correctness and retention complexity.
A daemon could reduce startup hashing and parsing further, but adds resident
state, invalidation and lifetime authority. Prefer the durable graph without a
daemon when flexible cross-family reuse justifies that larger implementation;
retain the bulk Interface as its ordinary convenience path.

No build, verification producer or cleanup run was executed for this design.
