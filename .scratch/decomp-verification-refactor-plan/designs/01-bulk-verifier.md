# Alternative A: one bulk verifier invocation

Status: design only. Baseline: main `5c011eb`; the approved
[Offline family extraction](/home/jim/sm-decomp/.scratch/offline-family-architecture/proposal.md)
and its [active checkpoint](/home/jim/sm-decomp/.scratch/offline-family-architecture/checkpoint.md)
take precedence over the source locations below. No implementation or production
authority change is proposed for the current extraction session.

Recommend a stateless bulk verification Module in `offline-verification`. Its
Depth comes from owning a bounded, immutable authenticated snapshot for an entire
invocation, including preparation, comparison and admission. It gives callers
Leverage without exposing a task graph or reusable execution session. No daemon,
new production ledger or cross-invocation authentication cache is required.

## Interface and caller

Illustrative Rust; names are not an implementation commitment:

```rust
pub struct BatchRequest {
    pub checkout: PathBuf,
    pub families: RequiredFamilies, // EntireCatalog | application-selected IDs
    pub changes: ChangePolicy,     // Full | AffectedSince(revision)
    pub output: PathBuf,
    pub reuse: ReusePolicy,        // AuthenticateRetained | Fresh
    pub limits: RunLimits,
}

pub fn plan(request: SelectionRequest)
    -> Result<SelectionReport, BatchError>;
pub fn verify_batch(request: BatchRequest, tools: &ToolEnvironment)
    -> Result<BatchRun, BatchError>;
pub fn authenticate_batch(request: AdmissionRequest)
    -> Result<AuthenticatedBatch, BatchError>;

let run = offline_verification::verify_batch(BatchRequest {
    checkout,
    families: RequiredFamilies::EntireCatalog,
    changes: ChangePolicy::Full,
    output,
    reuse: ReusePolicy::AuthenticateRetained,
    limits: policy.run_limits(),
}, &tools)?;
publish_batch_summary(run.report())?;
```

These are three bulk entry points. Existing standalone `verify(request)` remains
ergonomic and delegates a one-family operation. `plan` is optional, read-only and
does not prepare compilers. Its report cannot authorize subsequent execution;
`verify_batch` authenticates its own inputs. Profile policy and mandatory-family
choices remain in `pipeline`; the verifier supplies conservative dependency facts.
The existing catalog/Fixture authoring facets remain available.

Requests cannot supply raw compiler commands, recipe callbacks, dependency
allowlists or omitted cases. Admission checks the policy-required family set;
an incomplete successful subset cannot become a completed batch.

The Interface promises canonical family/case ordering, complete Fixture pairs and
Fixture chains, bounded memory/bytes/deadlines, and the existing shared four-slot
process budget. Reports distinguish selected, newly executed, reused and rejected
families and measure this invocation. Preserve complete state, return values and
ordered hardware effects against actual compiled C. Malformed contracts and
changed inputs are errors; a comparison failure retains a rejected result. Native outstanding
ownership remains `ExecutionPending`. No implicit replacement run is allowed.

`authenticate_batch` needs retained inputs and evidence, not compiler preparation
or execution capacity. Its opaque result exposes checked declarations to Live
replay; a deserialized report is not an authenticated result. Existing Native
prepared handles retain their confined lifecycle and indexed operation contract.
The bulk call never converts a rejected Translation attempt into a retry.

## Implementation behind the Seam

The private snapshot binds catalog membership, declaration bytes, dependency
closures, Recovered C source, assets, recipe identity and verifier requirements.
It owns bounded immutable byte buffers and exact content references, not merely
paths with remembered hashes. Identical reads and parsing within the invocation
can share authenticated bytes. Payloads loaded later must match the snapshot's
bound digests. Mutable checkout paths and namespace membership receive explicit
freshness checks before publication; lock waits, compilation and reconciliation
retain their existing checks. A changed input invalidates the operation rather
than silently refreshing one family.

This deliberately extends, rather than weakens,
[SourceContext](/home/jim/sm-decomp/crates/decomp-source/src/context.rs:1): today's
cache avoids archive expansion but still rehashes every pinned input. Replacing
those reads requires consumers to use owned authenticated bytes or confined
copies, followed by required current-source checks. An open file, mtime, readonly
permission or cached digest alone does not establish immutable content.

A metadata catalog records recipe, routines, declared inputs and Fixture payload
references without decoding every payload. Initially, verified sidecars can save
decoding only: validating a sidecar against the current monolithic `.smf` still
reads that file. Genuine payload IO savings require independently authenticated
metadata/payload representation and consumer checks. Missing or stale metadata
falls back to decoding or leaves the Offline family affected; it never excludes it.

Exact dependency manifests combine syntax-aware Rust discovery with compiler
dependency output, recipe templates, C headers/units, generated inputs, relevant
environment/toolchain settings and ROM/capture inputs. Records bind the discovery
implementation and configuration. A changed closure, unresolved macro/build input,
new namespace entry, deleted input or unsupported generator remains an Affected
Offline family until re-established. Dependencies are not inferred from words in
comments. This addresses
[Discovery::source](/home/jim/sm-decomp/crates/pipeline/src/affected.rs:287)
without treating the first fixed refusal as proof that all later uncertainty
disappeared.

Internally, typed dependencies connect preparation, builds, ready Fixture pairs,
chain predecessors, comparison and sealing. Existing dependency-ready scheduling,
Rust-before-C admission and pristine fork workers stay intact. Shared build
preparations group identical C/Rust inputs and tool descriptions across families;
family wrappers and actual dependency closures still participate in keys. The
existing [build cache](/home/jim/sm-decomp/crates/tool-execution/src/build_cache.rs:198)
continues artifact authentication, lock-wait rechecks and final closure checks.
No extra process pool is introduced, and leases are not held by idle preparation.

## Identity, evidence and admission

Keep four distinct concepts:

| Identity | What it can establish |
| --- | --- |
| Content | Exact canonical bytes, lengths, names and referenced closure; every consumed object is authenticated. |
| Build | Content inputs plus compiler/toolchain, target, flags, relevant environment, recipe/ABI and generated wrapper; permits artifact lookup. |
| Result | Builds, Fixture declarations/order, assets, execution requirements and comparison implementation; permits finding potentially reusable evidence. |
| Authority | Required current/retained source contract, producer provenance, frozen controller/sandbox, approved Translation scope and exact claim/Translation attempt; governs admissibility. |

A matching content, build or result identity never grants authority. Ordinary
admission authenticates retained producer provenance under its source contract;
it does not compare that producer with today's whole CLI executable. Native
admission preserves the installed controller fence and full per-operation producer
attribution. Keep the extraction's broad source requirements until a separately
reviewed dependency proof justifies narrowing them. Unrelated CLI independence is
an acceptance gate, not an assumption behind this design.

Make offline evidence indexed during production. Each completed Fixture pair
contributes named before/C-after/Rust-after objects, return values, ordered effects,
capture expectations and required producer references to a canonical inventory.
The final inventory checks the exact prescribed case set and chain inheritance.
It must retain both subjects' attribution even when their bytes deduplicate.
Do not first write and inventory an entire directory and then repeat that work
to pack it. Bounded staging remains necessary where current producers require
files and for failure diagnostics.

Use existing `evidence` content storage, with private bounded stores or a per-run
writer followed by authenticated publication/import. Its
[Store](/home/jim/sm-decomp/crates/evidence/src/pack.rs:48) has one writer and
`RefCell` caches; sharing one mutable Store across worker threads is not supported.
Preserve Native's existing
[operation index/chunks](/home/jim/sm-decomp/crates/pipeline/src/translation_native_operations.rs:134).
Prototype exact-byte restoration,
complete inventory/producer attribution and retained reader dependencies before
adopting any new offline representation. Existing sealed receipts are not rewritten.

Within an invocation, inventory, comparison and authentication can consume the
same verified bytes. This removes duplicate reads, not evidence checks.
Independent admission, cleanup reauthentication and later retries create fresh
snapshots and verify referenced closures again. Source mutation, missing objects,
redirected references, truncated packs and incomplete producer receipts must fail.
Logical Merkle equality cannot substitute for authentication of the full closure.

## Ownership and dependency strategy

`offline-verification` owns the bulk Module, metadata/dependency facts, recipes,
private scheduling, Fixture comparison and admission. `decomp-source` owns
Recovered C source authentication and source facts. `tool-execution` owns bounded
reads, compiler preparation/cache, leases, process ownership and telemetry.
`evidence` owns generic storage/restore mechanics, not acceptance policy.
`translation-execution` keeps existing confinement. `pipeline` keeps profile/claim
policy, batch publication, Live composition and cleanup orchestration. No lower
crate imports `pipeline`.

Parsing, dependency analysis and comparison are in-process dependencies and need
no Adapter. Filesystem/storage and compiler processes are real local dependencies:
exercise the Interface using temporary workspaces, real compilers and established
confinement controls. The existing ordinary/confined execution Adapters justify
their private Seam. Do not add a mock filesystem/process port solely for testing.
Locality comes from concentrating recipe and authentication changes under the
same owner, rather than exposing preparation or task nodes to callers.

## Migration and measurement gates

1. Finish the approved extraction first. Establish current, frozen full-catalog
   baselines with metrics enabled. Fix reuse duration reporting; historical
   `FamilyRecord.seconds` is not current authentication time. Record controller
   and sandbox build identities, host load, cache state and all retained inputs.
2. Introduce the bulk call as composition over existing behavior. Share catalog
   parsing and source preparation inside one invocation, retain current evidence
   contracts, and prove standalone/bulk equivalence and tamper rejection.
3. Add metadata and dependency manifests. Test documentation-only changes,
   transitive edits, include/env/generated inputs, captures, missing inputs,
   renames and mutation during selection. Never trade conservative coverage for
   a lower selected count.
4. Share build preparation across families, retaining authenticated cache hits
   and four-slot accounting. Compare full mandatory cold and warm batches, not
   just selected-family reductions. Keep existing scheduling unless measurements
   identify a remaining scheduling cost.
5. Prototype direct indexed offline production through the existing store.
   Gate adoption on byte-exact restoration, full attribution, independent
   admission, post-cleanup authentication and interruption recovery. Compare
   physical bytes, file counts, CPU, peak memory and elapsed time; compression
   and import costs can outweigh saved IO.
6. Choose optimized controller/sandbox artifacts using matched quiet-window
   measurements. Defer unnecessary executable identity work only for commands
   whose contracts permit it. Installed adoption requires correctly frozen
   installation/policy, never replacement of pinned binaries in place.

For each stage measure exclusive planning/authentication/publication/cleanup time,
compiler and Fixture spans, shared-capacity waits, bytes hashed/read/written,
decoded manifests, artifact hits and fresh reuse latency. Report total elapsed
time separately from overlapping producer duration. Test fresh processes for
independent admission; interface tests replace obsolete shallow tests once their
behavioral coverage is demonstrated.

Adopt each optimization only after matched, successful baseline/candidate runs
show a repeatable improvement in its targeted phase without material full-batch
or resource regression. Set thresholds from observed baseline variance before
the comparison. Failed or incomplete measurements remain explicitly incomplete.

The [diagnosis](/home/jim/sm-decomp/.scratch/decomp-batch-performance-20261015/findings.md)
observed 160/177 selected families, including 158 first refusals from documentation
words; selection is not proof of reruns. Full mandatory catalogs can still benefit
from shared parsing/authentication/build preparation and indexed storage. Cold
batches still execute every required Fixture; fresh admission still reads enough
evidence to authenticate it. Snapshot memory, longer operation failure domains,
storage writer contention and stricter dependency maintenance are the main costs.
There are no measured speedups for this alternative. The stripped whole-plan
probe was inconclusive and historical batches had incomplete metrics.

Plan-only cleanup: no producers, benchmarks, builds, worktrees or claims were
created or changed; no verification-run cleanup is required.
