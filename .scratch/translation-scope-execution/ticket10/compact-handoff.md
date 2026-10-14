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
proof cleanup is underway and final cleanup evidence remains pending. No
original controller files or production ledgers have been changed. The temporary
local main.rs registration is excluded from commits.
