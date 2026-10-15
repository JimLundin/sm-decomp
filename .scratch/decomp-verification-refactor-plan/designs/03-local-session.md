# Alternative C: a local verifier session

Status: design only. Source inspected: main `5c011eb5d02a54d74903715bffb3f8c15ab78c51`.
This follows the [approved extraction](../../offline-family-architecture/proposal.md)
and preserves the [active owners](../../offline-family-architecture/checkpoint.md).

**Recommendation: retain this as a measured, later option.** A persistent Rust
process can make the common caller trivial and retain authenticated immutable
inputs between complete batches. It cannot eliminate required execution, and
much of its proposed benefit also belongs in a simpler one-shot implementation.
Adopt it only after comparing those implementations on complete required catalogs.

## Interface and usage

The application Module is `LocalVerifier`; its Seam accepts complete batch requests, hiding
catalog preparation, scheduling and retention. No public action graph, source
cache handles or compiler commands are required.

```rust
impl LocalVerifier {
    pub fn submit(&self, request: BatchRequest) -> Result<SubmittedBatch>;
}
impl SubmittedBatch {
    pub fn wait(self) -> Result<BatchRun>;
}
pub fn authenticate(request: AuthenticationRequest)
    -> Result<AuthenticatedBatch>;

let run = LocalVerifier::default().submit(required_batch)?.wait()?;
```

`BatchRequest` carries existing family/profile selection, checkout, output,
freshness and bounded execution requirements. A preview CLI flag selects the
session; after acceptance, ordinary `batch verify` can connect or start it
lazily. `authenticate` always works independently, without a socket, compiler
or daemon. The one-shot Adapter remains available for unsupported environments.

A submission token permits waiting on the exact existing submission; it grants
no Translation scope authority. Reconnection cannot authorize another Translation
attempt. Changed inputs require a distinct request. Fixture mismatch returns
retained rejection; invalid contracts and exhausted limits return errors;
ambiguous producer ownership remains `ExecutionPending`. Client disconnection
does not imply cancellation. Existing cancellation commands retain authority.
Successful waiting includes sealing, authentication and publication.

This Interface has Depth because one request exercises preparation through
authenticated publication. Its Leverage is greatest for repeated complete
batches; its Locality concentrates lifecycle complexity in the verifier rather
than every CLI caller.

## Authenticated immutable inputs

The Implementation owns one private namespace per session generation, with a
0700 directory, 0600 Unix socket, checked peer credentials and bootstrap identity.
Protocol and implementation versions, exact verifier identity and installation
identity partition sessions. Candidates cannot reach the socket or namespace
through their confined mounts. Requests cannot select arbitrary host paths or
reuse another installation's authority.

Import reads and authenticates actual catalog, Recovered C source, Rust inputs,
ROM, captures and tool dependencies into bounded immutable representations.
Resolve inputs beneath checked directory descriptors; reject redirects and
unexpected aliases. Cached authority requires owned immutable bytes: sealed
memory files or fs-verity-backed disk objects where available. A read-only bind or mode bit
alone cannot protect a file from a host writer. Ordinary disk cache entries are
rehashable hints until authenticated into that representation. Never use
mtime/size/inode observations as authenticity. Rehash mutable origin closures at
the existing freshness transitions, including final publication checks.

Retain decoded catalog/dependency facts once per immutable generation. Affected
Offline family selection remains conservative for unknown inputs. The existing
operation-local source cache deliberately rechecks pinned files; extending its
lifetime alone is insufficient ([source context](../../../crates/decomp-source/src/context.rs)).

Evidence authentication validates the complete reachable closure: actual bytes,
declared kinds and extents, exact membership, provenance, image/input identities,
coverage, Fixture chain inheritance, full state and ordered callbacks. A declared
hash or cached deserialized verdict cannot produce `AuthenticatedBatch`.
Reuse validated bytes within that closure, but maintain separate validation
results keyed by the full contract and validator identity. Each independent
admission and post-cleanup authentication reconstructs its own proof from retained
bytes; daemon memory is never a required evidence dependency.

## Compiler and helper lifetime

Reuse authenticated compiler descriptions and compiled artifacts, including
identical translation units across families. Keys include actual compiler and
runtime dependencies, flags, environment, target, generated wrappers, source
closure and ABI. Preserve emitted dependency checks and post-build validation
from [build preparation](../../../crates/tool-execution/src/build_cache.rs).
This does not require keeping an arbitrary compiler backend alive.
Freeze each request's environment explicitly; never mutate daemon cwd or ambient
environment between concurrent submissions.

The current helper parent is already single-threaded and never executes gameplay;
every invocation runs in a pristine fork child
([worker](../../../crates/pipeline/src/fixture_worker.rs):141).
Extend lifetime only for an exact linked image, helper protocol, loader/library
closure, preloaded ROM, initialization environment and confinement configuration.
Family-specific wrappers may make cross-family hits rare. ROM or global
initialization changes require another parent; do not promise to reset arbitrary
pooled processes.

The daemon may use threads, but gameplay forks occur only inside separately
exec-started, proven single-threaded helper parents. Reject recipes whose
initialization starts threads or captures unaccounted mutable resources; use
fresh execution. Each request supplies its own declared state and output area.
Rust admission precedes C; continuations await matching predecessor state;
comparison and canonical ordering remain unchanged. Failure or cancellation
drains children and descendants and retires uncertain helpers.

Current Native compiler and Fixture producers remain separately confined and
individually attributed under `translation-execution`. A daemon cache hit cannot
replace a required fresh C execution or fabricate an operation receipt. Preserve
indexed attribution and limits, reservations, inherited deadlines and all
producer identities ([Native index](../../../crates/pipeline/src/translation_native_operations.rs)).
Session rollout begins with ordinary batches. Native adoption requires a newly
reviewed, frozen installation; pinned controller and sandbox remain authoritative.
The session neither reschedules the current
[serial Native family loop](../../../crates/pipeline/src/translation_original.rs):700
nor removes its [Fixture wave barriers](../../../crates/pipeline/src/fixture_execution.rs):87.

## Lifecycle, contention and bounds

Bootstrap authenticates executable inputs before serving. Retain actual producer
identity; a daemon handshake claiming a hash is insufficient. A smaller launcher
can avoid becoming an evidence producer, but existing controller identity checks
still apply. The current 333-MB executable's startup cost cannot simply be
subtracted from every invocation.

Provisional ordinary-session ceilings are 32 queued requests, 512 MiB combined
daemon/helper resident memory, eight idle helpers, 8 GiB reclaimable cache and a
five-minute idle lifetime. Measure and tune these before adoption. Count decoded
inputs, mapped objects and helpers, with streaming/eviction before refusal.
These ceilings do not raise any Native operation, output, read or deadline limit.
Published evidence storage remains separately reserved and accounted.
Each request keeps its inherited read/deadline accounting even when physical
bytes are shared; saved I/O does not forgive prescribed trusted-read charges.

All sessions and one-shot callers use the existing
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`
capacity. Compilers and active Fixture pairs acquire shared leases; idle helpers
hold no slot. Bounded fair queues prevent a large batch monopolizing admission;
never wait holding partial multi-slot reservations. Charge queue, import,
hashing, setup and execution against applicable deadlines. Limit background
hashing and imports so they cannot become an unmetered competing workload.

The evidence Store has one writer lock and a `RefCell` eight-block read cache;
it is not a shared synchronized object
([pack storage](../../../crates/evidence/src/pack.rs):51).
Use bounded per-run shards and worker-local read handles. Publish/import through
one bounded writer only after sealing; do not place all execution behind its
lock. `import_store` re-encodes dependencies and ordinals; copying independent
pack files is invalid. Include import cost in elapsed time and storage budgets.

Before success, promote every referenced input, reader and evidence object into
retained storage. Eviction removes only unreferenced cache objects. Cleanup
cannot touch active writers or published dependencies. Restart drops in-memory
trust and authenticates disk again. The existing production journal remains the
sole authority for selected dispositions, recovery and attempts; session tokens
are disposable projections. Lost contact never launches replacement producers.
Stop/reconcile writers using exact process identities before releasing ownership.
Upgrades start a new generation, drain the old one and preserve exact historical
readers; no in-place pinned binary replacement occurs.

## Ownership, migration and controls

`pipeline` owns transport/CLI composition, selection, installed policy and
cleanup orchestration. `offline-verification` owns session verification state,
catalog/recipes, scheduler and admission. `decomp-source` owns authenticated source
facts. `tool-execution` owns immutable local inputs, compilation, leases and
accounting. Preserve the approved downward crate dependencies. Build the small
daemon entry point in Rust after the active extraction lands; do not redirect
its workers or widen their files now.

Computation is in-process. Filesystem, compiler and process dependencies are
local and tested with real temporary workspaces and installed test tools. The
ordinary and confined execution Adapters remain private. No remote dependency
or generic plugin framework is justified.

First land syntax-aware selection, one authenticated catalog per operation,
phase metrics and shared preparation in the common implementation. Then expose
an opt-in session with unchanged evidence formats. Test through its Interface:
input mutation with preserved file metadata, false digest declarations, missing
closure members, aliasing, quota exhaustion, two simultaneous batches, ROM/image
changes, helper global mutation/crash/timeout, descendant teardown and daemon
death during publication. Require standalone authentication after shutdown,
cleanup and producer-worktree retirement. Replace redundant internal tests once
these controls cover the same behavior.

## Performance decision

The [diagnosis](../../decomp-batch-performance-20261015/findings.md) observed
160/177 selected families, including 158 first refusals from comment tokens,
three catalog loads and repeated evidence reads. Selection is not proof of
160 recompilations. Stripped-controller timings were inconsistent; no end-to-end
speedup is established.

Compare optimized one-shot and session modes on identical frozen full required
catalogs, both with fresh Fixture execution and with legitimate retained-result
reuse. Run at least three paired repetitions on a quiet host: empty caches/cold
daemon, warm disk/cold daemon, and warm disk/live daemon. Include start, import,
catalog parsing, source checks, compiler work, capacity waits, Fixture execution,
comparison, authentication, publication and both cleanup passes. Measure wall
time, CPU, physical reads, peak memory/storage and contention separately.

Proposed adoption gates: at least 15% median improvement for a warm **complete
fresh catalog** against optimized one-shot, at most 5% cold end-to-end regression,
equivalent coverage/verdict/provenance, unchanged Native attribution and passing
tamper/cleanup controls. These are targets, not measurements. Report aggregate
break-even submissions as cold overhead divided by per-submission savings;
reject the option if savings are absent or typical sessions end first.

A cold complete catalog still imports all required bytes, builds every missing
distinct image and runs every required Fixture pair. Existing disk compilation
caching and within-family helpers already capture substantial reuse; serial
Native families and wave barriers remain. If only warm authentication retries
improve, describe the daemon as a convenience for that workload, not the primary
whole-batch performance refactor.

No builds, producers or verification runs were started for this design; there
are no run artifacts to clean up.
