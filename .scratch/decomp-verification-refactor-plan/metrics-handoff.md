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
The warmed target is exclusively held until emitted tools are retained and
both root and native_admission receive an explicit release. Next run the new
stage controls, benchmark attribution control, disabled/timeout/interruption
metrics controls, real batch reuse regression and real batch interruption.
Each executable control has a 60-second limit, separate from the build budget.

Still pending: sampling-disabled/enabled retained measurement pairs, exact
reader/tool retention, final scoped source commit, final cleanup record and
independent retained admission. At the current checkpoint, the filesystem has
123,365,834,752 bytes available; private target/TMPDIR/proof paths are retained.
Root owns final acceptance and worktree retirement. Do not resolve ticket 01.
