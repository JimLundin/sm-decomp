# Ticket 01 instrumentation handoff

Status: implementation active; root owns ticket resolution and integration.

Owner: `/root/batch_metrics`, assigned by session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Worktree: `/home/jim/sm-decomp-performance-metrics`.
Branch: `performance/batch-metrics-20261015`.
Provisional baseline: `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`.

The extraction's remaining frozen-source gates are owned by root. No production
policy, installed controller, managed attempt or shared cache is changed here.

## Scope

- `crates/pipeline/src/batch.rs`, approved benchmark attribution consumer, and public family/metrics controls.
- `crates/tool-execution/src/metrics.rs`, existing build-cache and capacity hooks.
- `crates/offline-verification/src/family.rs` and `ordinary.rs` admission timing.

Current-invocation elapsed time and original-run time are separate. Repeated reuse
must retain the first execution duration. Stage spans use `CLOCK_MONOTONIC`; summed
overlapping producer durations are never described as elapsed batch time. Cheap
stage recording stays enabled when resource sampling is disabled. Missing stages
have explicit unavailable reasons. Metrics remain outside authenticated Native
producer semantics.

## Initial control

The new public batch regression creates one complete actual-C Fixture family,
seeds its advisory historical batch duration to 86,400 seconds, and authenticates
the same retained result twice. Each reported current time must fit inside the
observed invocation, while the original duration stays unchanged across both
reuses. This makes copying the historical value a deterministic failure.

Compilation is bounded at 900 seconds, Cargo jobs 1, locked/offline, with both
dev/test debug information disabled. This is a build budget, not a successful
performance sample. The first execution control is bounded at 60 seconds.

Private target: `/home/jim/sm-decomp-performance-metrics-target`.
Private temporary directory: `/home/jim/sm-decomp-performance-metrics-tmp`.
Build/verification records: `/home/jim/sm-decomp-performance-metrics-proof`.
The worktree's `deps/rustysnes` is a read-only-use link to the existing dependency;
all Cargo outputs use the private target.

New measurement controls isolate their artifact cache but link its workers
directory to `/home/jim/sm-decomp/out/pipeline/workers`, with capacity 4. Existing
isolated capacity controls are unchanged. No shared cache is cleared.

## Verification and cleanup

The real batch reuse control failed on the baseline: 86,400 seconds reported
for a 3.32114051-second invocation. Log:
`/home/jim/sm-decomp-performance-metrics-proof/reuse-red.log`.
The initial fix passed the same control (1 test, 31.75 seconds). Its two reuse
rows reported 0.554539 and 0.554415 seconds; original time remained 86,400 and
new producer counts were zero. Rejected reuse and explicit fresh work also
passed. Both explicit cleanup passes reclaimed zero bytes, then independently
authenticated the passing evidence. The disposable test project was removed.
Log: `/home/jim/sm-decomp-performance-metrics-proof/reuse-green.log`.
These are correctness observations on a contended host, not speedup evidence.

The next source checkpoint is WIP: bounded stage summaries, honest benchmark
attribution and the actual batch interruption control are implemented but their
final controls are still pending. Limits are 128 labels / 128 bytes per label,
512 coarse or 4,096 detailed trace events. Existing complete producer and scope
records remain; complete summed durations/counts/outcomes survive trace omission.
Nested scopes share the label budget; imported scopes are bounded at merge.

Active build (2026-10-15): timeout process 1477273, Cargo 1477274, private target,
jobs1, locked/offline, both dev/test debug disabled; no feature flags. Exact argv:

```sh
timeout --signal=TERM 900s env CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/home/jim/sm-decomp-performance-metrics-target CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 TMPDIR=/home/jim/sm-decomp-performance-metrics-tmp SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 cargo test -p pipeline --locked --offline --bin sm-pipeline --test family --test metrics --no-run
```

Output: `/home/jim/sm-decomp-performance-metrics-proof/bounded-build.log`.
Check actual process identity after recovery; these PIDs are observations.
That build returned 101 because the CLI unit-test target needed six recovered-C
`include_str!` files. `deps/snesrev-sm` now links to the existing main dependency
for read-only build use; no dependency source changed.

The original warmed target was explicitly released to root/native_admission.
An independent `cp -a --reflink=auto` clone completed in 5.88 seconds (exit 0),
with 1,213,911,040 allocated bytes at
`/home/jim/sm-decomp-performance-metrics-continuation-target`.
The clone contains no new cross-target hardlinks. Its temporary disk cost is
separate from any performance result. Exact transfer executables are retained
under `/home/jim/sm-decomp-performance-metrics-proof/tools/target-transfer/`:

- `sm_pipeline`: `0820872e413d28b60a23990bf8db23982ef3cac6b0c3bf70b8a235cebe201352`
- `family-e6290bf228387814`: `8e872e73b442f06a758d1ce010c5f6e4a104349acafab7478c5df88cf3a12453`
- `metrics-2dc9ee0393fa426b`: `bf62cb6f3108dec02b8c665760b68bb14535bd37764dc8a600721873c3cc9571`

Transfer test binaries predate the bounded controls; this is not an accepted
frozen tool set. The same no-run argv is active on the continuation target,
with output in `bounded-continuation-build.log`. The source now also contains
an ignored retained sampling measurement harness; its compilation/control is
pending. Next run the new
stage controls, benchmark attribution control, disabled/timeout/interruption
metrics controls, real batch reuse regression and real batch interruption.
Each executable control has a 60-second limit, separate from the build budget.

## Passing bounded-source controls

The final no-run build succeeded after the pinned-source link and five integer
conversions in the new measurement harness were corrected. The last incremental
build took 1.50 seconds. Those setup/compiler failures are retained in
`bounded-build.log` and `bounded-continuation-build.log`; passing output is
`bounded-final-build.log`. No producer verification ran in the failed builds.

All controls used `timeout --signal=TERM 60s`, the private TMPDIR, shared
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, and four workers, with
`--exact --nocapture --test-threads=1` for individual tests. The full metrics
suite used `--nocapture --test-threads=1` and deliberately did not execute its
ignored subprocess helper as a standalone workload.

- `metrics-bounds-controls.log`: 9 passed, one subprocess helper ignored,
  19.46 seconds. Includes repeated/high-cardinality/nested/imported accounting,
  disabled sampling, complete owned waits, process timeout, SIGTERM and capacity
  recovery.
- `benchmark-attribution-control.log`: one passed, 0.19 seconds; complete sums
  remain available when a time union cannot be computed from the capped trace.
- `reuse-bounded-green.log`: one passed, 22.53 seconds. Two authenticated reuses
  reported 0.361097 / 0.369189 seconds (external 1.418482 / 1.446085 seconds),
  retained the seeded 86,400-second original, and launched zero producers.
  Failed reuse plus fresh rejection and explicit fresh success also passed.
  Both cleanup passes reclaimed zero bytes; passing evidence authenticated
  independently after cleanup.
- `batch-interruption-control.log`: one passed, 13.16 seconds. Actual sleeping
  Rust Fixture work was interrupted through the batch controller; current time
  was 2.650761 seconds inside a 3.510133-second invocation. Interrupted outcome,
  stopped writer identities, unreached comparison accounting, fresh capacity
  recovery and independent admission passed. Both cleanup passes reclaimed zero.

These disposable actual-C test projects were removed after their scoped cleanup;
the retained logs and exact source describe the controls. No owned control writer
or build remains active. At this checkpoint, free space is 119,953,764,352 bytes.

Exact executables are copied (not cross-target hardlinked) under
`/home/jim/sm-decomp-performance-metrics-proof/tools/bounded/`; the adjacent
`bounded.sha256` records:

- Controller: `0820872e413d28b60a23990bf8db23982ef3cac6b0c3bf70b8a235cebe201352`
- CLI unit tests: `4560e78a465b9ad219468e166f122eb94dea8a9e00916687f29399c5f0df2d55`
- Family tests: `578743d6f1b6b6b523b50c6b1393696c5241e8bd11511b778a4d0a6f54d58c3a`
- Metrics tests: `f7bea41f51100e12562db7a178e4a87863db38fbc5af3e2dcb56083adf219680`

Still pending: sampling-disabled/enabled retained measurement pairs, final
measurement cleanup and independent retained admission. The ignored Rust
`measure_batch_metrics_with_retained_evidence` control requires a new absolute
`SM_BATCH_METRICS_MEASUREMENT_OUTPUT`; it preserves source and exact tools,
uses an empty private cache plus the shared workers link, alternates three reuse
off/on pairs, performs fresh work in the existing cache namespace, runs both
cleanup passes, removes its private cache/.git and independently admits retained
verdicts using its copied reader. Cold/existing namespaces do not imply cache
hits, and contention is recorded per sample. Root owns final acceptance and
worktree retirement. Do not resolve ticket 01.
