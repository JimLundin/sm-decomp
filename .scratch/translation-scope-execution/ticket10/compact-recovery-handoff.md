# Ticket 10 compact verdict-only recovery

Status: implemented; red-green and refusal controls passed; retained fixtures authenticated after cleanup; ready for root
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

The final focused reader passed seven controls in 0.25 seconds: the exact crash
regression, three supplementary identity/evidence/operation-declaration refusal
controls, and the three existing original preparation guards. The two large
original-evidence tests and one read-only retained-fixture audit were ignored in
that run. No original experiment or producer was launched. Caller/source and
negative-control fixture changes are restored before returning, so all four
final fixture receipts remain authenticatable.

The final reader is `tools/compact-recovery-green-reader`, SHA-256
`4d8f470ae59a021d59eef883a765ee68a7e9d54083528411836e74f6b57056a4`.
Exact source/reader seals are in `green-reader.sha256`. Its production module
matches the initial green source hash
`e85a0e5a8f9919f4bc8d4540be1bd6216ec280ed046f0d1203771f711be77edb`.
The final test source hash is
`9bcfa4afc9753211a5823100543191c3bd936d4a28b91dedf16ec059763e0b65`.
The code change is commit `7693752`, after the regression-only `0df92dc`.

The control command was:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
SM_COMPACT_RECOVERY_PROOF_ROOT=/tmp/sm-ticket10-compact-recovery-proof-20261014/green \
/tmp/sm-ticket10-compact-recovery-proof-20261014/tools/compact-recovery-green-reader \
translation_compact:: --nocapture --test-threads=1
```

## Completion evidence and cleanup

The scoped source checkpoints, in order, are `0df92dc` (regression before fix),
`7693752` (authenticated verdict recovery) and `27416a1` (supplementary refusal
controls). The final module and tests passed rustfmt and `git diff --check`.
All authoring, test and cleanup sessions have finished. There is no compact
producer to stop and no unfinished operation in the supplementary fixtures.

Both project cleanup passes succeeded, each reclaiming 0 bytes:

```sh
/tmp/sm-ticket10-compact-proof-05-20261014/tools/sm-pipeline-producer \
cleanup --root /tmp/sm-ticket10-compact-recovery-proof-20261014 --apply
/tmp/sm-ticket10-compact-proof-05-20261014/tools/sm-pipeline-producer \
cleanup --root /tmp/sm-ticket10-compact-recovery-proof-20261014 --deduplicate --apply
```

The existing utility was only read/executed; its proof 05 root was not modified.
Logs are `cleanup-pass1.log` and `cleanup-pass2.log` in the new recovery proof.
All red/green fixture bytes, frozen reader/source seals and context logs remained
unchanged; see `after-cleanup-hash-check.log` against the `before-cleanup-*` and
three reader manifests. The red fixture retains its deliberately missing receipt
and original failure, without a recovery mutation after the red run.

After cleanup the final reader authenticated four final green fixtures in
0.04 seconds and the initial green fixture in 0.01 seconds. Each audit passed,
checked unchanged receipt/reference bytes and confirmed no operation/capture.
Logs are `after-cleanup-authenticate-green.log` and
`after-cleanup-authenticate-initial-green.log`. The exact read-only command is
recorded in the proof's `reader-context.md`; it can be repeated after retirement.
Available space after cleanup was 162,899,931,136 bytes, recorded in
`free-space-after-cleanup.txt`.

Retain `/tmp/sm-ticket10-compact-recovery-proof-20261014` with all fixture source,
prepared/runtime/context records, verdicts/receipts, readers, source snapshots
and manifests. `reader-runtime.txt` confirms the final reader links only system C
libraries and does not depend on the disposable target or worktree. The exact
build compiler is Rust 1.101.0-nightly (`c1070d693`), preserved in
`compiler-version.txt`. Existing proofs 01–05 and original Submitted source stay
untouched and retained.

Root owns integration and retirement of only this new recovery worker worktree
and `/tmp/sm-ticket10-compact-recovery-target-20261014`, then the quick retained
authentication check. Its ignored pinned-C dependency symlink is disposable;
the canonical dependency in the main checkout is retained. Root separately owns
the original supervisor control, receipt index, final review and ticket status.
