# Continuation checkpoint: benchmark preparation

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

Ticket-02 pre-runner exact Git source `4abc67f` was preserved in
`out/ticket-02-final/source-freeze/` (no Git/build trees) plus a source-freeze.tar.
ROM/dependency symlinks point to main checkout. Preserve it for historical
current-source admission as main.rs/metrics/resource source evolve. Refresh
source-dependent decomp/RNG/system before final-source seed preparation.

No new verification run or Cargo target was created by runner preparation.
Telemetry runner performs scoped cleanup/authentication per completed sample.
Final cleanup, overhead evidence, new controls and benchmark/profile remain open.
