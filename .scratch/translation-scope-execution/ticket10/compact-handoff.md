# Ticket 10 compact-controller adapter

Status: implementing
Owner: `/root/compact_diagnostic`
Worktree: `/tmp/sm-ticket10-compact-20261014`
Branch: `ticket10-compact`
Baseline: `808efe2ee473e633a42bbf330f3872f5a0499df9`

Owned source is `crates/pipeline/src/translation_compact.rs` and this handoff.
The root integrator owns permanent command registration and production changes.
The original Submitted controller worktree is read-only and remains retained.

The agreed test seam is the existing supervisor/installed deterministic command
specified in ticket 10. The adapter exposes typed preparation, run, retained
authentication and reference collection for that seam. A trusted confined Rust
producer translates the original fixed experiment phases: original-ROM compact
capture, exact C host build, causal candidate and independent changed-RNG control.
The parent recomputes ordered polls and completed-state comparisons from raw
evidence; candidate reports cannot grant acceptance.

The original target remains frame 2240 with one whole-experiment 60-second
producer deadline and aggregate hard output quota. Failure or infeasibility is
retained and unchanged replay launches no new process. Diagnostic acceptance
establishes no Live replay or original-ROM parity. Historical captures/readers
and their private inputs are retained and authenticated independently.

The typed API is implemented: `contract_for_original`, `historical_for_original`,
`prepare`, `prepare_until`, `run`, `authenticate`, `references`,
`authenticate_historical` and the internal confined `producer` command. The
root must register `translation-compact-producer` to call `producer(&args[1..])`.
`StrictRuntime` supplies helper, producer, operation_root, pipeline_root, workers,
gate_timeout_seconds, optional deadline_unix_ms and max_output_bytes.
`CompactReceipt` exposes passed, diagnostic, evidence_bytes, context, verdict and
the prepared/candidate identities. It derives PartialEq, not Eq, because Asset
has no Eq implementation.

Preparation now checks the complete original C units/private source against the
pinned archive, exact original ROM/movie/lsnes, any% manifest/import semantics,
aggregate private bytes and exact compiler-header directory bindings. Other
runtime binds are individual regular files. The receipt records an experiment
start/deadline pair and every invocation's start and remaining duration.

Verification: initial `cargo check --bin sm-pipeline` passed. All three current
preparation controls pass. The complete-original-lineage control was observed
failing before its implementation, then passing. The targeted test build uses
`CARGO_TARGET_DIR=/tmp/sm-ticket10-compact-target-20261014`,
`CARGO_BUILD_JOBS=2`, shared `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`
and `SM_PIPELINE_WORKERS=4`.

The initial adapter checkpoint is `b82ce75`. Preparation requires all five
original bridge and submission verdicts plus their exact readers; the old compact
ELF is `2898c6a3f553e74051abbb00f47a429fd653522d0caa4a06be33161eb581e57a`.
Private/runtime reads honor the inherited preparation IO budget, and duplicate
private references are authenticated once within that preparation. The verdict
retains historical dependencies even if a later history fails authentication.

Initial real proof `/tmp/sm-ticket10-compact-proof-01-20261014/test.log` completed
in 644.79 seconds. It authenticated the two original full captures and recomputed
matching prefixes 1097 and 1096. Its test fixture incorrectly supplied a readable
run id instead of the operation engine's required digest. The retained failure
is `gate/compact-receipt.toml`, diagnosis `durable operation requires exact context
identities`, with no producer launched. The old broad proof control passed its
unchanged-receipt checks; this does not establish a fresh compact diagnostic.
The fixture is fixed and now also requires all five authenticated histories and
an attempted native phase. The initial failed proof's source is `b82ce75`.

The root approved a narrow compatibility bind for the exact original compact
reader: source equal to Program.executable, destination `/proc/self/exe`, realized
as a synthetic symlink to `/program`, without mounting procfs. The native worker
owns confinement support and helper verification. The compact recipe requests
that bind only for direct legacy-reader execution. New producers enter before
the ordinary CLI's current_exe initialization. A new confined proof awaits that
reviewed runtime change; no diagnostic producer is currently running. Initial
proof cleanup completed with 0 bytes reclaimed on both standard and deduplicate
passes; logs are `cleanup-pass1.log` and `cleanup-pass2.log`. This provisional
format has no disposable files recognized by the cleanup utility. Its exact
adapter reader source is retained as `reader-source-b82ce75.rs`; its receipt,
verdict and reader-source hashes are in `after-cleanup.sha256`. The receipt SHA
is `5be5e3f9118f868f0d8482d4c216061304dd5796712d7c777604cc122f0c8326` and
verdict SHA is `da9ab25ce012b3dc3eaec06049254e1765d32e999226337a9bf1c4eaaa3bc21d`.
Remaining space after these passes was 174504767488 bytes. Final proof and
post-cleanup adapter authentication remain pending. No
original controller files or production ledgers have been changed. The temporary
local main.rs registration is excluded from commits.

Proof 02 is running at `/tmp/sm-ticket10-compact-proof-02-20261014` with a
frozen test reader (`9b6c911fde7fb408a1d713b5bf8de0f44c4108d996308dd4264d99062804ccaa`)
and the native worker's verified helper from `05785a6`
(`d3bee79b2a711126d6e0de4405e578c1ab51d27b420fadce86515c0eef8f3338`).
The compact focused suite passed 3 ordinary controls after rebuilding the host
client with that confinement revision. Historical-reader operations may use the
persisted outer gate deadline (at most 3600 seconds); the fresh experiment still
shares exactly 60 seconds. Compiler headers are mounted only for the compile
phase. The frozen producer has debug sections removed, with its complete
executable seal retained in `producer-stripped.sha256`; its producer behavior is
unchanged. The proof reader and exact adapter source are retained independently
of the disposable Cargo target.

Proof 02 authenticated all five original histories, covering 6,629,732,570 bytes
of historical artifacts. It recomputed matching prefixes 1097, 1096 and 1736;
both original 8600-frame error statuses remain errors. The exact original compact
reader completed inside confinement with status 0 and stdout `Prefix diagnostic
authenticated; acceptance remains false` (stdout SHA
`2bcde0bb1a6f2e3a0e4a31510e11f29d15a58a4ba8b7d6f30f4b88d5261b8cd1`).
The fresh native phase exhausted the original budget before compile/candidate/
control. Its experiment began at 1791974982178 ms, had deadline 1791975042178 ms,
and finished termination at 1791975042217 ms. The exact retained diagnosis is
`confined compact native failed: sandbox timeout`; no shorter target was used.
Receipt SHA is `c88f4651afafc5e9561cfe523c1314dca4bcca698d55de93d94d87dd030e8cc6`,
verdict SHA is `9e42de4c9b0461b5e55e82c8bad42b739a4d5d0319cd9c73c8a901b23d963fb0`,
and prepared.toml SHA is `365e26f230ad0fc4ccc92db60c05ad141b532aba70da4cc9e72a58e0df19d21d`.
Unchanged-resume validation and cleanup are still running/pending for this proof.

The native worker identified the v1 helper as unoptimized and is validating an
optimized v2 helper. Root approved a separate proof 03 with that new exact helper
identity and the unchanged frame/deadline contract. Its frozen host test reader
is `a5e46472ac191805ed8e264368df09772d71bd084c86eaefb1010d37400cd54b` under
`/tmp/sm-ticket10-compact-proof-03-20261014/tools/compact-proof-reader`.
The current compact suite passes 3 ordinary tests and has 2 original-evidence
tests ignored by default. The new read-only cleanup test authenticates one
retained receipt and verifies that its operation inventory and references remain
unchanged. The fresh proof harness still authenticates both initial run and
unchanged resume, without a third redundant historical pass between them.
