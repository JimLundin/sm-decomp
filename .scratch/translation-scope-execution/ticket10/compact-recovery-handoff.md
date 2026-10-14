# Ticket 10 compact verdict-only recovery

Status: regression written; red test build running
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

No production fix has been applied yet. The new regression is
`translation_compact::recovery_tests::sealed_verdict_recovers_an_expired_stopped_experiment_without_relaunch`.
Its initial normal-profile test build is tool session 72451, jobs 2, target
`/tmp/sm-ticket10-compact-recovery-target-20261014`. This is a private copy of the
existing root authoring target, not a write to that shared target. No optimized
full build is requested.

The fresh worktree lacks the ignored RustySNES checkout. The first Cargo attempt
stopped before compilation; its log is retained as
`red-build-missing-local-dependency.log`. The active command overrides only that
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
`/tmp/sm-ticket10-compact-recovery-proof-20261014`. Freeze the red reader and exact
source before the first implementation change, then run the named test with
`SM_COMPACT_RECOVERY_PROOF_ROOT=PROOF/red` so the failing evidence survives.
Existing proofs 01–05 stay untouched; no large compact experiment will be run.

After the fix and focused controls, freeze a green reader and source, run both
project cleanup passes on this new proof, authenticate retained fixtures, record
bytes reclaimed/remaining space and checkpoint the exact owned files for root.
Root owns final integration and retirement of this worker worktree/target.
