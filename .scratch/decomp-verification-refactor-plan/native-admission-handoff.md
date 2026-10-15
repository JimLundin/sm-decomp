# Native operation admission recovery

Status: claimed; structural regression prepared, awaiting shared build target
Owner: `/root/native_admission`, delegated by performance session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`
Blocked by: build target release from `/root/batch_metrics`

Worktree: `/home/jim/sm-decomp-performance-native-admission`
Branch: `performance/native-admission-20261015`
Baseline: `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`
Proof parent: `/home/jim/sm-decomp-performance-native-admission-proofs/20261015`

Ownership is limited to `translation-execution/src/operation.rs`, required
retention reader changes and operation/retention tests, the test-support feature,
`offline-verification/src/native.rs`, and this handoff. The integration owner
owns main, the recovered extraction, installation, and the fresh full Native
proof. No receipt schemas, producer membership, evidence bytes, or deadlines
are being changed.

## Design and current source

The retained PLM failure and source-derived redundant-read diagnosis are in the
main worktree's `native-finalization-recovery.md` and the extraction worktree's
`native-deadline-diagnostic.md`. This worker does not retry or mutate that proof.

The planned owned `AuthenticatedOperation` keeps output bytes, their exact
result reference, and transitive references together. Its private constructor
performs a fresh request admission. Native collection consumes one such owned
admission per producer; a later whole-receipt admission opens all required
retained evidence again. Buffers are bounded to one operation, with no trusted
metadata cache surviving an admission.

The prepared regression uses the real complete confined operation interface
and asserts that complete admission reads exactly the summed retained-reference
bytes. `/proc/self/io` and `/proc/thread-self/io` are absent on this host; the
control instead uses narrowly scoped thread-local numeric reader accounting
under the explicit `translation-execution/test-support` feature. Production
builds have no counter state or instrumentation. No test has run yet.

## Execution and cleanup

No build or producer is currently active for this worker. The metrics worker
retains exclusive ownership of
`/home/jim/sm-decomp-performance-metrics-target` until its explicit release.
Planned flags: `CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`; Cargo commands use `--locked --offline -j1`.
The regression controls additionally enable the `test-support` feature.

All runs use a private proof TMPDIR and
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4`.
An ignored `deps/rustysnes` symlink points read-only to main's existing dependency
for offline Cargo resolution. It must be removed with the worker checkout after
integration; the target dependency is not owned by this task.

After finished controls, run both project cleanup passes on this proof parent,
record reclaimed bytes and remaining free space, and retain exact logs/readers
and source. Worktree retirement remains with the integration owner after merge.
