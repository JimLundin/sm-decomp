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

Verification so far: initial `cargo check --bin sm-pipeline` passed before the
last closure/timing tightening. The two public preparation controls passed in
the preceding phase. The current targeted test build uses
`CARGO_TARGET_DIR=/tmp/sm-ticket10-compact-target-20261014`,
`CARGO_BUILD_JOBS=2`, shared `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`
and `SM_PIPELINE_WORKERS=4`.

Next: complete targeted tests, build a normal producer binary and run ignored
`translation_compact::tests::original_experiment_retains_exact_target_and_unchanged_resume`
with the sealed original source/assets/helper closure. Historical compatibility
and the fresh frame-2240 diagnostic have not yet been exercised. Any failure
remains retained and requires unchanged-resume authentication. No original
controller files or production ledgers have been changed. The local temporary
module/command registration in main.rs is excluded from commits. No diagnostic
producer is currently running; verification cleanup remains pending.
