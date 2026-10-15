# Ticket 04 shared compiler preparation

Status: implementer gates and proof cleanup complete; combined integration
admission and coordinated worktree/target retirement remain. No rollout claimed.

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

## Historical next step at first red

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

## Semantic key and volatile eligibility checkpoint

`logs/eligibility-controls.log`: **6 tests passed**, 24.25 s execution / 4.49 s
Cargo build. Actual controls cover one preparation/common compilation for two
families, independent failed-family cancellation, shared Rust selection, raw
physical `__FILE__` separation, explicit logical `__FILE__`/`__BASE_FILE__`/header
mapping, changed logical targets and optimization flags, and fresh volatile C
through headers plus direct and macro-expanded diagnostic pragmas.

`logs/system-angle-eligibility-controls.log`: the expanded volatile test passed in
17.29 s, including real angle-bracket includes resolved through `-isystem`, with
and without a diagnostic pragma. Seven distinct input cases each require two
actual C compilations, empty cached-object records and the explicit fresh reason.
The preceding `system-eligibility-controls.log` used quoted includes and is not
the system-header proof; it is retained as diagnostic test development evidence.

Cache identities are now version 5. Literal paths and a bound working directory
are the default. The private C path contract normalizes only typed source/include
operands under explicit `-ffile-prefix-map`; targets, macro values and other flags
remain literal. Debug/switch-recording contexts keep physical identity. Eligible
pinned objects use the stable checkout cwd with absolute source/include operands;
generated units keep their original run cwd and remain fresh. Supported standalone
helpers also compile in their selected checkout; unreviewed helper flags retain
their run cwd in the key. Removing generic repository substring normalization also
removes its Git identity subprocess. Existing version-4 cache entries are not
removed and cannot satisfy the corrected version-5 contract.

Eligible C performs full preprocessing and dependency emission. A bounded stream
discards code payloads while preserving a <=4 KiB line prefix and <=1 MiB stderr,
with <=64 MiB total stdout and the caller's deadline/byte budget. Active diagnostic
pragmas (including `_Pragma`) make the unit fresh because they can suppress
`-Werror=date-time`; unclassifiable long lines also remain fresh. Unsupported or
volatile preprocessing falls back to actual fresh C compilation with emitted
dependency equality, while IO, compiler identity, deadline and byte-budget errors
remain hard failures. Contended artifact waits repeat the actual dependency proof
before accepting a hit. Every unit records why it was eligible or fresh.

The following final checkpoint completes the focused controls, measurements and
both scoped proof cleanup passes. Ticket acceptance still waits for the
integration owner's combined-source proof and retirement/admission.

## Final source and actual controls

Final gate source: `330e8c0a2d03385a868e76754ec28a6b1324af78`.
Production source: `cb00f32da6a0965755f04d7b9dea8cafcf9aa8ec`.
The later commit contains only the environment control and opt-in measurements.
The earlier ordered changes are `af36c4a`, `7a0fb97`, `88a0c30` and `f425730`.
The bulk-context owner has been sent both final commits; root owns their
integration registration and final source freeze.

`logs/line-red.log` captures a genuine incorrect cache hit under the earlier
logical-path contract: identical C text with a synthetic absolute `#line`
filename produced different values under the two prefix selectors, but the
second checkout restored the first result. The corrected C path contract is
`c-file-prefix-map-v2`. Compiler-emitted filenames must belong to the actual
dependency closure before logical equivalence is allowed. Synthetic/unbound
filenames, incomplete filename parsing and unused or partial selectors preserve
literal physical identity. `logs/line-green.log` has both the synthetic-path
control and ordinary mapped-path sharing control passing (5.35 s).

`logs/wait-controls.log` records the first 11-control green (48.19 s).
`logs/final-controls.log` records **13 controls passing**, with the opt-in
measurement excluded, in **35.02 s** (0.04 s Cargo check/build). Besides the
previous controls, it includes header mutation and newly available transitive
includes during a real cache-lock wait, compiler-byte replacement during that
wait, retained-context refusal after mutation, fresh-context recovery, corrupt
artifact rebuilding and subsequent authenticated restoration, and preprocessing
byte-budget failure with no object compiler/publication. The external validator
and complete dependency proof still execute after contended waits.

The environment control runs the same retained test executable as separate
processes and changes only `CFLAGS`. It confirms equal source/command/closure
identities, an equal-environment hit and a changed-environment miss. Every other
cache input is explicitly equal after removing `environment_sha256` from the
comparison. There is no unsafe mutation of the multithreaded test environment.
`logs/environment-green.log` records this control passing in 2.45 s. The earlier
`logs/environment-control.log` is a test compilation error (borrowed `PathBuf`
instead of owned `PathBuf`), preserved as diagnostic development evidence.

## Matched preparation measurements

`logs/measurements.log` and
`measurements/matched-measurements-206211-0/measurements.toml` retain three
alternating private/shared pairs, each with cold and warm observations. Source
and header bytes, compiler, optimization, profile and worker capacity are equal.
Separate physical checkouts provide empty cache seeds without clearing the
shared cache; only the typed absolute source/include operands vary. These C
fixtures contain no file/cwd macros. Adapter compilation and fixture setup are
outside the timer; compiler preparation, discovery, compilation/restoration,
input validation, linking, metrics/cache sealing and actual program checks are
inside it. Every warm observation starts a new invocation context.

| Seed | Private-context median | Shared-context median | Private range | Shared range |
| --- | ---: | ---: | ---: | ---: |
| Cold | 1.127 s | 1.148 s | 1.098–1.180 s | 1.094–1.187 s |
| Warm | 0.965 s | 0.940 s | 0.860–0.972 s | 0.898–1.008 s |

The process reduction is consistent: compiler identity queries fall from **2 to
1** per two-family invocation. Both modes compile the shared pinned object once
cold and zero times warm. Each retains two generated C compilations, two links
and five actual dependency scans (including the contended wait rescan). Both
modes have one cold cache hit and two warm hits. The cold/warm means are slightly
slower with sharing; these samples establish **no reliable wall-time improvement**
and grant no full-batch speedup claim. Sharing immutable preparation remains a
bounded interface improvement and removes measured duplicate processes; the
combined batch gates must establish its practical workload effect.

No broad reference image, new cache or larger incremental invalidation group was
selected. Generated C, candidate Rust and all Native producer slots remain fresh.

## Completion evidence

All producers and test processes finished before cleanup. The retained cleanup
reader is
`/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015/tools/sm-pipeline`,
SHA-256 `20c8a35fab96ce9e34e6c8817d26eb4abb1b30f73c162116f31d093d91fbe108`.
The following commands both succeeded, scoped to this implementer's proof root:

```sh
/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015/tools/sm-pipeline cleanup --root /home/jim/sm-decomp-performance-builds-proof --apply
/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015/tools/sm-pipeline cleanup --root /home/jim/sm-decomp-performance-builds-proof --deduplicate --apply
sha256sum -c /home/jim/sm-decomp-performance-builds-proof/retained-files.sha256
sha256sum -c /home/jim/sm-decomp-performance-builds-proof/retained/cache-artifacts.sha256
```

Both cleanup receipts report **0 bytes** and no eligible disposable targets;
deduplication reports zero recognized snapshots. These are build-control runs,
not full Offline/live verdict directories. Their small compiler adapters,
executables, source fixtures, diagnostics and cache snapshots are retained
evidence. No shared cache was cleaned or cleared.

The post-cleanup checks authenticated **3,454 retained files** and **66 copied
cache artifacts** against their seals. All **63 retained programs** from the
final controls and measurements were then independently executed and matched
their expected outcomes. `retained/program-expectations.json` lists the checked
exit/stdout requirements; `logs/retained-programs-after-cleanup.json` records the
results. Header/include/compiler mutation seeds remain historical, deliberately
stale source-cache records; their original executables return the original
value, while the positive fresh-context programs return the changed values.
Historical red evidence is retained as diagnosis and grants no acceptance.

Required retained dependencies are the entire proof root, its exact fixture
source paths and programs, `retained/cache/<key>/{artifact,seal.toml}`, the shared
cache at `/home/jim/sm-decomp/out/pipeline/build-cache`, and the copied reader
`retained/tools/shared-preparation-tests` with SHA-256
`bf0040fc4929dd4055310443bdf4c7c189d1f997f7b6db1534abe878624f075c`.
`retained/source/` holds exact owned Rust source snapshots and
`retained/source-freeze.toml` binds the gate commit, producer/cleanup readers and
system C compiler. Running the test reader also requires the installed Rust/C
toolchains and the host `libgcc_s`, `libpthread`, `libdl`, `libc` and ELF loader;
they are not disposable proof intermediates. Re-executing the existing small
programs uses the retained executables and host loader/runtime, not Cargo.

Proof size after cleanup/authentication: **317,395,506 bytes**. Available disk at
the final observation: **104,507,097,088 bytes**. The writer-free exclusive Cargo
target `/home/jim/sm-decomp-offline-verifier/target` occupies **3,137,264,927 bytes**
and is released back to root for coordinated reuse/removal. Evidence does not
depend on this target. Root must still retire the integrated implementer worktree
and dispose of unused target output as part of combined completion; this worker
does not remove another owner's Git worktree or begin ticket 08.

| Evidence | SHA-256 |
| --- | --- |
| `logs/final-controls.log` | `f700ce88524e67f249984f26e758f0ce72d90730d2ac6c95fa8b99e96dc1d776` |
| `logs/line-red.log` | `9b07f25a5f4f6be592f118dbeb38c65cc39742ff1bb8deb7a34a1912331e9387` |
| `logs/line-green.log` | `1408238ce84074b90e657f52b2825c231316b9fafcc097ca3bc3a7ab9e3c0e2e` |
| `logs/measurements.log` | `b483b4814e62fcfbc089b8189a985762c3982195e92dfe339d52731b4b337a46` |
| `measurements/matched-measurements-206211-0/measurements.toml` | `7ecc8d32203f64c63bf5770dafc9ebc968ffa075781dfd44221a0047b0feaae6` |
| `logs/cleanup-apply.toml` | `db5bac8a65aebadf59f0fdf35212019050be91606a9165b298d3f3d192ecacdc` |
| `logs/cleanup-deduplicate.toml` | `2624524f0fb2f448e1351f8dad525947deffaa761537e3f3206162e1f18829b7` |
| `logs/retained-programs-after-cleanup.json` | `076b94b409d21ae0a7596401a69577633a1d39d1f0a2b7519968f3551021ccba` |
| `retained-files.sha256` | `6fef147dbff859386b8969943b38d5645f4f778675ae2ce1085b18a801af1c51` |
| `retained/cache-artifacts.sha256` | `707735a273b813f2f62a8287dc7d5379aac4d343e66e74b1906874a14baf470f` |
