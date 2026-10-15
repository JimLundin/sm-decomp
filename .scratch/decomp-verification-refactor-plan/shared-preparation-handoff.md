# Ticket 04 shared compiler preparation

Status: implementing; no acceptance or rollout claimed.

Owner `/root/shared_preparation`, integration session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Worktree `/home/jim/sm-decomp-performance-builds`, branch
`performance/shared-preparation-20261015`, baseline
`b333ca5c488e9f923c5672c321284f870f8fbede`.
Own `crates/tool-execution/src/builds.rs`, `build_cache.rs` and scoped public
build tests. The bulk-context owner attaches the shared context in
`offline-verification`; this worktree does not modify its files.

Exclusive Cargo target `/home/jim/sm-decomp-offline-verifier/target` was released
after root completed its independent copy. Cargo uses locked/offline, jobs 1.
All test producers use shared root `/home/jim/sm-decomp/out/pipeline`, capacity 4.
Private Cargo dependency link `deps/rustysnes` points to the existing pinned
checkout; its source is unchanged.

## Interface and eligibility

An invocation owns `tool_execution::builds::BuildContext::default()`. The
proposed `c_build_in(&context, request, validator)` and
`standalone_builds_in(&context, request)` share compiler/toolchain identity and
immutable initial input facts. Each family keeps its own observed closure and
abort state. Existing free functions create a private context for one call.
Compiler, environment and actual dependency validation remain independent after
waits and before publication; warm artifacts still authenticate.

Generated C, candidate Rust and Native compiler slots remain fresh unless exact
path/runtime/dependency controls justify another eligibility decision. Existing
pinned C and supported standalone artifacts retain one build per eligible key.
No all-game image or additional artifact cache is selected.

## Red evidence

Command (2026-10-15):

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 SM_SHARED_BUILD_PROOF=/home/jim/sm-decomp-performance-builds-proof/red CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-verifier/target CARGO_BUILD_JOBS=1 cargo test -p tool-execution --test shared_preparation --locked --offline -- --exact compatible_families_prepare_compiler_once_and_coalesce_the_actual_object --nocapture
```

`/home/jim/sm-decomp-performance-builds-proof/logs/red-shared-preparation-counter-fixed.log`
records one failing assertion in 1.19 s: compiler identity queries are **2**, the
required shared count **1**. The actual common pinned C object compiled once,
one family missed and one hit the same cache key, and both actual programs
returned 7. Compile time was 0.38 s. The initial log
`red-shared-preparation.log` is an invalid count observation because two writes
interleaved; the counter was repaired to append each complete record once.

No process remains from these controls. Their private sources, logs, cache
identities and output artifacts are retained below the proof root. Both scoped
cleanup passes and independent output/cache authentication remain required after
the coherent implementation gates. Shared cache contents must not be cleared.

## Next

Implement the small context; rerun the same actual compiler control with the
shared interface, add bounded mutation/cancellation/path controls, and measure
compatible cold/warm family builds. Final acceptance waits for tickets 02/03 and
root's frozen integration gates. Record every source freeze and cleanup here.

## First C green

`red` source checkpoint: `af36c4a`.
`logs/green-shared-preparation.log` records the matching control passing in
1.76 s (6.06 s Cargo compilation). Compiler identity queries fell from 2 to 1;
the shared pinned object still has one actual compilation, one miss and one
authenticated hit. Both programs return 7. These timings are observations under
host activity, not a throughput comparison.

The initial implementation shares compiler selection/version/environment and
immutable expected source hashes behind `BuildContext`; each `Preparation`
retains separate abort state and its own closure. Compiler/environment validation
now also occurs immediately after cache-key waits and restoration. Shared context
and input maps have explicit extent limits and cancellation/deadline-aware waits.
Standalone calls accept the same context interface; coalescing their selected
Rust toolchain discovery is the next implementation slice.

## Rust selection and path control

C green source checkpoint: `7a0fb97`.
`logs/toolchain-path-controls.log` records two passing controls and a new path
failure (10.41 s for the three tests). Rust has one selected sysroot, one compiler
version query, one detailed Rust version query and one actual helper compilation;
both calls still emit fresh dependency scans. The helper's actual output agrees,
run-local overrides cannot redirect it, and adding a checkout override invalidates
the shared selection. The context binds launcher identity, environment, all
possible ancestor override paths (including absence), rustup settings and selected
compiler bytes; it rechecks before producers and after waits.

The new actual-C `__FILE__` control reproduced an existing incorrect cross-checkout
hit: the second checkout restored an object reporting the first checkout's source
path. Generic substring normalization in `CacheIdentity` is the cause. Preserve
semantic physical paths by default; only typed path operands with explicit
compiler prefix maps may become logically equivalent. Keep prefix-map targets
literal. This is required before extending eligibility.

Root also requires volatile `__DATE__`, `__TIME__` and `__TIMESTAMP__` coverage.
An actual GCC probe showed `-M -Werror=date-time` does not expand expression macros.
Eligibility will use full preprocessing with dependency emission; unsupported or
volatile preprocessing must fall back to fresh compilation, never a cache hit.
