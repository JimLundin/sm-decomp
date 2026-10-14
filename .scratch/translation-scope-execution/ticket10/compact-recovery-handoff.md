# Ticket 10 compact verdict-only recovery

Status: exact regression observed red then green; supplementary refusal controls pending
Owner: `/root/compact_diagnostic`
Worktree: `/tmp/sm-ticket10-compact-recovery-20261014`
Branch: `ticket10-compact-recovery`
Baseline: `30bef717a560f450c1e11cf5f77573f91996a3a3`

## Scope

Own `crates/pipeline/src/translation_compact.rs`, its narrow supplementary
`translation_compact_recovery_tests.rs` and this handoff. Other agents remain
active. Do not edit main Git/index, native/confinement source, original Submitted
controller worktree or any existing compact proof root.

The requested fix recovers a sealed `compact-verdict.toml` left by a crash before
`compact-receipt.toml` publication. Recovery must authenticate the exact stored
identity, source, timing, phase, limit and operation bindings, then publish the
receipt once without setup, a new deadline or another producer/capture.

The agreed supplementary seam is the existing `run`, `authenticate` and
`references` adapter API. A tiny already-prepared fixture records an expired
stopped experiment, removes its receipt to simulate the crash, and expects the
same receipt and verdict bytes on recovery. It cannot grant original-contract,
gameplay, Live or original-ROM parity acceptance. Root separately owns the
original supervisor bounded-failure/resume control.

## Current verification checkpoint

The exact regression is
`translation_compact::recovery_tests::sealed_verdict_recovers_an_expired_stopped_experiment_without_relaunch`.
Its normal-profile test builds use jobs 2 and target
`/tmp/sm-ticket10-compact-recovery-target-20261014`. This is a private copy of the
existing root authoring target, not a write to that shared target. No optimized
full build is requested.

The fresh worktree lacks the ignored RustySNES checkout. The first Cargo attempt
stopped before compilation; its log is retained as
`red-build-missing-local-dependency.log`. The next attempt identified six
compile-time pinned C fixtures required by existing tests; its log is
`red-build-missing-c-fixtures.log`. An ignored `deps/snesrev-sm` symlink in this
fresh worktree points to the existing pinned main-checkout C dependency. No
tracked dependency source changed. The build command overrides the RustySNES
existing path dependency on the Cargo command line, without changing workspace
source or its lockfile:

```sh
CARGO_TARGET_DIR=/tmp/sm-ticket10-compact-recovery-target-20261014 \
CARGO_BUILD_JOBS=2 \
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
cargo --config 'patch."https://github.com/doublegate/RustySNES".rustysnes-core.path="/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core"' \
test -p pipeline --locked --offline --no-run --bin sm-pipeline --message-format=json
```

New verification evidence belongs under
`/tmp/sm-ticket10-compact-recovery-proof-20261014`. The red reader and exact
source were frozen before implementation, then the named test ran with
`SM_COMPACT_RECOVERY_PROOF_ROOT=PROOF/red`. It failed in 0.08 seconds with
`compact immutable output changed`, after the original verdict authenticated.
This reproduces the reviewer's exact failure; no broad instrumentation or
alternate-cause experiment is needed. The reader is
`tools/compact-recovery-red-reader`, SHA-256
`9576f499c34c4098007ecf0b612a7e4a4484ca507e2866eec5d296e9c7178998`.
Source and reader seals are in `red-reader.sha256`; the regression-only source
checkpoint is `0df92dc`.

The fix now recovers an existing verdict before setup or production-deadline
paths. Both normal completion and recovery use the same authenticated receipt
publisher. Existing receipts and verdict-only finalization use the bounded
read-only reconciliation scope, while the original production interval stays
unchanged. All existing identity, timing, source, phase, limit and operation
checks run before receipt publication.

The unchanged regression passed in 0.09 seconds with exact receipt/verdict bytes,
no operation/capture, and stable completed replay. Its first green reader is
`tools/compact-recovery-initial-green-reader`, SHA-256
`fab850b759b6cf405375bae3fb7a8e042e2bcd7f301e4e042cdb19261787e044`.
The initial green source/reader seals are in `green-initial-reader.sha256`, and
the retained fixture is under `green-initial/`. Both red and green runs finished.
Existing proofs 01–05 stay untouched; no large compact experiment will be run.

After the fix and focused controls, freeze a green reader and source, run both
project cleanup passes on this new proof, authenticate retained fixtures, record
bytes reclaimed/remaining space and checkpoint the exact owned files for root.
Root owns final integration and retirement of this worker worktree/target.
