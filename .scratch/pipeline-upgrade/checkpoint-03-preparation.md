# Continuation checkpoint: benchmark preparation

## Recovery on 2026-10-12, 08:20 UTC

Session 21359 ended with compile errors in dynamic TOML test values, a private
blob constant and a consumed Store. Fixed; initial 6 benchmark, 6 process,
5 resource and 1 actual live-refusal controls all passed. Evidence/model suites
passed (11 evidence + 4 model), formatting and strict pipeline Clippy passed.
Runner now records pre-measurement preparation refusal with scoped cleanup;
host observation includes CPU consumed by newly appearing external processes.
No controlled timing or overhead acceptance is established.

A repeat launched Clippy concurrently against the same target and two process
controls failed before publishing producer PIDs. No remaining writers found;
serial verification is running to distinguish build interference from a bug.
Current session 46750: all pipeline unit tests plus benchmark/metrics/resource
integration controls; `/tmp/sm-pipeline-serial-controls.log`, target
`out/ticket-03/target`. Two large existing fixture matrices are still running.
Latest strict Clippy: `/tmp/sm-benchmark-clippy-final.log`. Do not mutate or
build against this target while executable controls are running.

Optional quiet-window question was sent because unrelated Node tests resumed.
Proceed with correctness while awaiting it; telemetry only under continuous
host observation. Main and unrelated trace-storage worktree remain untouched.

Serial verification passed: 61 pipeline unit tests, 6 benchmark refusals,
6 process controls (including both cancellation controls), 5 resource controls.
Existing 44-domain matrix passed. Evidence/model tests remain green. Strict
Clippy and formatting pass after the final preparation-cleanup edit. Preserved
logs in `out/ticket-03/logs/`. Release pipeline/evidence compilation is session
13300, log `/tmp/sm-benchmark-release.log`; no telemetry measurement yet.

Release build passed, 3m34s; retained pipeline/evidence in
`out/ticket-03/retained-tools/`. Scoped native cleanup/dedup succeeded, both
0 bytes. Cargo clean removed 2.1 GiB control target and 126.0 MiB release target.
Cleanup/logs retained under `out/ticket-03/logs/`; free disk
219,748,634,624 bytes. Test-owned temp fixtures are removed by their controls.
No verification writers remain. The integration worktree remains active because
02/03 and final optimization/regression gates are unfinished.

2026-10-12. Scope remains pipeline tooling and optimization only.

Integration `/tmp/sm-pipeline-upgrade-integration`, branch
`pipeline-upgrade-integration`; scope clarification committed as `4abc67f`.
Main checkout remains unchanged. Ticket 03 runner preparation is claimed while
its admitted measurements remain blocked by ticket 02.

## Active control

Telemetry process session `9267`, retained executable and output
`out/ticket-02-final/telemetry-pairs-quiet`. Six-sample order is disabled,
enabled, enabled, disabled, disabled, enabled. First results: disabled
585,069,061,079 ns; enabled 643,242,416,019 ns.
One pair exceeds the 2% gate; do not resolve 02 or admit 03 measurements until
the full median gate passes. Same binaries/stores/observations, no competing
heavy workload found in host snapshots so far. Third sample enabled completed
in about 520.465s; fourth sample disabled is running. Read exact samples.toml
for all accepted values. The six-sample median remains pending.
First enabled capture has three truncated disk scans and explicitly incomplete
resource peaks; do not use it as a complete resource baseline. Capture times
236.855s disabled versus 310.331s enabled; publication varies in both directions.
Second enabled sample first wave took 107.863s versus prior enabled 137.699s,
showing substantial timing variation. Finish the full control before changing
sampling settings; no overhead success is currently supported.
Host-continuous-0186 caught an unrelated short npm/Node test during sample 4's
publication/authentication. It finished by the follow-up ps, but sample 4 is
explicitly incomparable: `sample-4-disabled/comparability.md`. The runner's raw
report cannot establish a passing quiet-host control using this sample. Retain
correctness/cleanup and replace this disabled sample or rerun the complete control.
Sample 4 completed in about 613.323s but remains incomparable. Sample 5 then
overlapped a larger unrelated Playwright/Chrome workload (host snapshots 0198
and follow-up ps). Own capture PID 1755995 was stopped by SIGTERM; root and four
subject receipts report interruption, no accepted fifth sample or full report.
Control session 9267 exited 1; host observer functions cell 102 was terminated.
Only completed samples 1–3 were observed without a heavy competing workload.
The full overhead gate remains open. Native scoped cleanup/dedup receipts:
`out/ticket-02-final/logs/quiet-control-{cleanup,dedup}.toml`.

Host observer functions cell `102` records escalated host `ps` every ten seconds
to `host-continuous-NNNN.txt` inside the control output; it runs up to one hour.
Stop or extend it when the control finishes/needs more observation. Host PID
visibility from ordinary sandbox execution is insufficient.

## Uncommitted preparation

Rust `benchmark.rs`, `benchmark/host.rs`, public CLI `benchmark
prepare|freeze|validate|run CONFIG.toml`, `tests/benchmark.rs`. Freeze exact
tracked build sources/tools, pipeline/compiler/input/predecessor identities,
native v4 cache/store inventories. Prepare authenticates current native live
1000/45000 seeds separately from the fixed twelve-sample warm matrix. Each
sample clones the original complete cache/store seeds and preserves its final
cache for audit. No runner lease wraps live; one-owner process accounting and
common-clock child receipts. Outer boundary includes public live, cleanup/dedup
and retained authentication. Sample failures withhold medians and retain
diagnostics/cleanup. Host process/load/memory/disk/state/wchan sampling is broad
attribution only; perf remains unavailable on this host.

Metrics exposes snapshot/add_live_output/stdin/poll_exit/terminate. Inherited
top-level native commands now install cancellation handlers to reap separately
grouped producers on a benchmark budget/interference stop. Resource sampler
rediscovers ephemeral live staging/private paths with a bounded scan; existing
static-path unavailable semantics remain strict.

**Not compiled or tested yet**. Formatting and diff whitespace pass. Do not
compile during the active telemetry control. Review the ~1000 lines of new
runner/control code and fix compile/check failures after the control completes.
Add inherited-clock cancellation and actual live refusal controls. Existing
benchmark tests target real public cache/sealed-store changes, frozen source,
missing cache, wrong capacity, rebound corrupt-cache seal.
Inherited-clock SIGTERM descendant teardown was added to public `tests/metrics.rs`.
A real missing-ROM live refusal/diagnostic/cleanup control was added under
`benchmark.rs` unit controls. README documents the new CLI and measurement limits.
Runner stage attribution reports role/stage summed and union intervals with
explicit cross-group overlap limitations. All remain unexecuted until the
telemetry measurement is finished.
The interrupted control is finished; compilation has now started under the busy
host: tool session 21359, `cargo test -p pipeline --locked --offline --test
benchmark --test metrics --test resource_metrics --no-run --target-dir
out/ticket-03/target`, log `/tmp/sm-benchmark-build.log`. Inspect/fix compile
errors, run controls and strict Clippy before any performance claims. WIP code
was checkpointed as 981f037; small preparation/freeze/audit fixes remain
uncommitted afterward.

Ticket-02 pre-runner exact Git source `4abc67f` was preserved in
`out/ticket-02-final/source-freeze/` (no Git/build trees) plus a source-freeze.tar.
ROM/dependency symlinks point to main checkout. Preserve it for historical
current-source admission as main.rs/metrics/resource source evolve. Refresh
source-dependent decomp/RNG/system before final-source seed preparation.

No new verification run or Cargo target was created by runner preparation.
Telemetry runner performs scoped cleanup/authentication per completed sample.
Final cleanup, overhead evidence, new controls and benchmark/profile remain open.
