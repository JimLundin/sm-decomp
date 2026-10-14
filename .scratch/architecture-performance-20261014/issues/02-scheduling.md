# 02 — Runnable Fixture capacity

Status: resolved
Triage: ready-for-agent
Blocked by: None
Type: task
Owner: /root/implement_scheduling
Worktree: /home/jim/sm-decomp-architecture-20261014/02-scheduling
Branch: architecture/perf-02-scheduling-20261014
Baseline: 1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0

## Scope

crates/pipeline/src/fixture_execution.rs; crates/pipeline/tests/family.rs or a dedicated fixture scheduling integration test.

## Acceptance

Admission follows runnable Fixture pairs; dependency waits do not hold execution leases. Preserve pristine helper reuse, Rust-before-C refusal, dependency failure propagation, canonical result ordering, cancellation and teardown. Replace repeated ready-state scans where useful. Public-seam contention regression and bounded scheduler comparison.

## Comments

2026-10-14: Authorized by the user's selection of all five report candidates.
See ../spec.md for shared resource, source and evidence contracts.

2026-10-14 implementation: the Fixture execution module acquires a shared lease
at worker-thread startup, before it knows whether a Fixture pair is ready. A
completed short root can therefore retain a slot while waiting on an unrelated
long root's continuation. Move that lease inside each admitted Fixture pair;
retain the pristine helper in the worker between pairs. Concentrate dependency
readiness in an ordered ready set with direct dependent admission. This adds
locality behind the existing family interface and gives every Offline family
leverage without exposing another adapter or seam.

Validation crosses the family and process-execution interfaces with actual compiled
C: both workers first execute roots, then an unrelated caller must acquire the
idle worker's slot and retain it through the continuation. The bounded
measurement uses identical three-Fixture inputs, two slots, a 600 ms root delay,
and separately warmed, exact-source builds in fresh/fork execution; it reports peer wait rather than a
full-route throughput claim.

The unrelated process controller starts before the family so executable-identity
hashing precedes the admission rendezvous. Its Rust process request is submitted
only after the short Fixture finishes. Baseline and candidate must retain their
own exact scheduler source because family verification rejects mismatched
compiled source; that refusal remains enforced.

## Completion evidence

Implemented in `d593bc8eb19a1f9105694e59e79ee56f46e4c98f` on the assigned
branch/worktree. The worktree is clean and retained for review/integration;
it has not been integrated or retired. Changes are limited to
`crates/pipeline/src/fixture_execution.rs` and `crates/pipeline/tests/family.rs`.
The final commit supersedes the earlier `f890e40` checkpoint.

The module now indexes direct dependents and the lowest-index ready Fixture,
instead of repeatedly scanning all Fixture states. A shared execution lease
covers each ready Fixture pair and ends before another dependency wait.
Pristine helpers survive between pairs. Blocked descendants require no lease,
and sorted outcomes, Rust-before-C admission and helper teardown retain their
existing interface.

### Validation

All commands used `--locked --offline`, Cargo jobs `1`,
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`,
`SM_PIPELINE_WORKERS=4`, and the owned private target/temp directories.

- `cargo test -p pipeline --test family --locked --offline -j 1 idle_fixture_chain_workers_release_shared_execution_capacity_in_both_modes -- --exact --nocapture --test-threads=1`: **1 passed**, covering fresh and fork modes. Final log:
  `/home/jim/sm-decomp-architecture-runs-20261014/02-scheduling/logs/candidate-regression-final.log`.
- `cargo test -p pipeline --test family --locked --offline -j 1 -- independent_chains_overlap_and_failed_returns_block_descendants_in_both_modes rust_admission_rejection_precedes_c_and_other_cases_keep_diagnostics fork_parent_is_pristine_and_execution_seals_cannot_be_stripped cancelling_family_reaps_helper_fixture_and_descendant --nocapture --test-threads=1`: **4 passed**. The cancellation control now also requires the idle helper to exit and the continuation to remain blocked.
  Log: `.../02-scheduling/logs/family-controls.log`.
- `cargo test -p pipeline --bin sm-pipeline --locked --offline -j 1 fixture_execution::tests -- --nocapture --test-threads=1`: **2 passed**, including malformed replies, oversized requests, mutation isolation, crashes, timeouts and descendant reaping.
  Log: `.../02-scheduling/logs/helper-controls.log`.
- The exact baseline binary and source fail the contention control as expected:
  the external process waits **2.999298987 s**, the root refuses with
  `idle worker stranded shared capacity`, and its continuation is blocked.
  Retained under `.../02-scheduling/baseline-regression-frozen`;
  log `.../02-scheduling/logs/baseline-regression-frozen.log`.
- Independent read-only review by
  `/root/implement_scheduling/scheduling_review` found no material scheduler
  issue. Its timer-label finding was fixed: `harness_seconds` explicitly
  includes family execution, peer shutdown and authentication.

### Bounded measurement

`cargo test -p pipeline --test family --locked --offline -j 1 measure_fixture_chain_shared_capacity -- --exact --ignored --nocapture --test-threads=1`
passed, including post-cleanup authentication of all **6** warmup/measured
actual-C verdicts. Retained report:
`/home/jim/sm-decomp-architecture-runs-20261014/02-scheduling/measurement-corrected/measurement.toml`.
The report seals both controller binaries, baseline scheduler source, fixed
Fixture source and peer execution receipts. Compiler versions and process
observations are in `.../02-scheduling/logs/`.

| Execution | Baseline peer wait | Candidate peer wait |
| --- | ---: | ---: |
| Fresh | 622.458 ms | 0.203 ms |
| Fork | 632.958 ms | 10.364 ms |

This is one sample per binary/mode with three actual-C Fixture pairs, two shared
slots, separately warmed caches, exact source snapshots and a fixed 600 ms root
delay. Before the measurement, no Cargo/rustc/sm-pipeline compiler or runner
activity was observed. The result demonstrates available shared capacity;
it does not establish full-route throughput or a reduction in aggregate RSS.
Idle helpers can outnumber active execution slots. Per-pair lease acquisition
retains the existing filesystem locking and 10 ms contention polling costs.

The initial harness attempts exposed executable-startup hashing, exact-source
freshness rejection and a peer marker written after family inventory sealing.
The final harness starts the controller before the rendezvous, retains separate
exact source snapshots, and writes peer handshakes only in the peer run.
All diagnostic logs/runs remain. A late unsealed marker and two duplicate peer
receipt files were moved outside sealed family directories; original sealed
artifacts and their hashes were unchanged. The broad interval in the completed
measurement was relabeled from `family_seconds` to `harness_seconds` without
altering any timings; original metadata and the Rust correction tool remain
in `logs/measurement-before-label-correction.toml` and `logs/finish-retention.rs`.

### Cleanup and retention

Every retained project applied both project cleanup passes through the existing
test retention adapter:
`sm-pipeline cleanup --root PROJECT/out --apply` and
`sm-pipeline cleanup --root PROJECT/out --deduplicate --apply`.
The ordinary temporary family/helper controls also perform both passes before
removing their owned temporary directories.

- Utility reclamation across retained runs: **10,354,688 bytes**.
- Identified disposable compiler-cache files removed: **42,084 bytes**.
- Owned Cargo target/temp outputs removed: **3,495,399,755 logical file bytes**
  (hard-linked paths may count more than once; this is not a physical disk delta).
- Final available disk space: **188,009,644,032 bytes** (about 175.1 GiB).
- Final authentication after removing the private target succeeded for **all
  11 retained successful actual-C verdicts**, including the restored diagnostic
  warmup. Per-verdict receipts and `logs/final-authentication.toml` are retained.
- Cleanup aggregation/correction source and results:
  `logs/finish-retention.rs`, `logs/retention-summary.toml`; final reader source:
  `logs/authenticate-retained.rs`. Their disposable executables were removed.
- The clean assigned worktree, each exact per-run source snapshot, baseline and
  candidate controller binaries in `bin/`, peer executables/source, verdicts,
  actual-C artifacts, receipt logs and compiler metadata remain. These are the
  retained evidence dependencies; no shared production state was changed.

See `logs/disk-final.txt` for the final available disk bytes. The coordinator
was observed in an errored state (`429 Too Many Requests`) during handoff;
this ticket and branch retain the complete result for recovery. The shared
plan/main Git index remain the coordinator's responsibility.
