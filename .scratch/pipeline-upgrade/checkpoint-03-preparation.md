# Continuation checkpoint: benchmark preparation

## Latest recovery, 2026-10-12 11:42 UTC

Update 12:29 UTC: session 32260 also ended in host-interference refusal at
offset407940ms, rolling0.27256. Snapshot deltas show Codex daemon/agents and
a short zsh process, no browser/compiler; guard remains unchanged. Failure
cleanup/dedup zero bytes and four retained subject authentications passed.
Only samples1/2 remain completed. Resume again after verification, preserving
this attempt separately. Use `login:false` for lightweight measurement reads
to avoid interactive shell startup work. No active measurement now.

Found/fixed a runner preparation bug: the publication store's root-level
`pipeline.lock` is runtime coordination, excluded from immutable seed inventory
and cloning. Public validation control exercises a real sealed pack plus this
runtime file; all physical pack seals remain required. Verification session
59466, target `out/ticket-03/seed-lock-target`, log `seed-lock-controls.log`.

Seven public benchmark controls passed (including runtime lock exclusion),
strict all-target pipeline Clippy/fmt/whitespace passed. Release refresh passed;
new baseline runner retained as `retained-tools/sm-pipeline-baseline`. Preserve
the old `sm-pipeline` unchanged for the ongoing on/off control. Cargo cleanup
removed1.3GiB tests,126.0MiB release,70.4MiB resume harness. External scoped
cleanup/dedup zero bytes. Resume harness now uses distinct `attempt-N` output
paths and records them per completed sample, preserving absolute published
store pointers on interrupted attempts. `sample-3-enabled` is the second
interrupted attempt at its original path; do not move it again. The first
interruption remains `interrupted-sample-3-enabled` and has retained private
stores for unpublished mixed subjects. Both attempts remain excluded.

Session 85476 finished with host-interference refusal during sample 3 publication.
The triggering external process was this session's recursive `rg --files` over
retained ticket-01 evidence: host samples at offset 403683/404304ms identify
PID 1863081 consuming 154/221 ticks. This was a measurement-window mistake;
do not weaken the guard or admit the interrupted sample. Narrow future reads
to specific files and never walk large evidence trees during measurements.

Samples 1/2 remain valid (529113032159ns disabled, 565550236893ns enabled).
Failure cleanup/dedup both reclaimed zero bytes and all four retained subject
authentications passed. The interrupted directory is preserved as
`telemetry-quiet-rolling/interrupted-sample-3-enabled`. No writers remain.
The local Rust control now resumes only the completed prefix, verifies its
fixed order/observations/final host receipt and starts a fresh sample 3. This
does not change the workload, producer/reader, telemetry, gate or boundary.
Rebuild session 33542, log `logs/telemetry-harness-resume-build.log`; finish and
clean that disposable target before restarting the control. The retained
pipeline/producer/reader binaries remain unchanged. Ticket 02 stays claimed.

Resume harness build succeeded; Cargo clean removed 70.4MiB. External scoped
cleanup/dedup succeeded with zero bytes, logs `rolling-interruption-*`.
**Active session 32260** now resumes sample 3 with host process visibility;
logs `telemetry-quiet-resumed.{stdout,stderr}`. Do not compile or recursively
scan evidence while active. Resume executable is
`out/ticket-03/retained-tools/telemetry-control-resume`.

Main and unrelated trace-storage worktrees are untouched. Active integration
HEAD 88207a6; tracked WIP is host rolling-window change and checkpoint/docs.

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

Verified runner committed as `88207a6`. Fresh system predecessor passed all 475
cases at `out/ticket-03/current-system/1791794394473867-2-0/verdict.toml`.
System cleanup 0 bytes, dedup 156,368,896 bytes/1,425 snapshots, post-cleanup
authentication passed. That invocation used the existing main capacity-four
root without changing its capacity; subsequent runs explicitly use the private
capacity-two root. Current-source decomp and RNG authentications passed: their
exact implementation closures are unchanged by runner work, so retain those
original predecessors rather than implying regenerated evidence.

Host preflight repeatedly found large unrelated Chrome/TypeScript runs.
No telemetry run was started. A fresh 1000-input current-source live correctness
run is executing through `out/ticket-03/current-live-request.toml` (explicit
capacity-two root), logs `current-live.{toml,stderr}`. It is under shared load,
outside all timing medians. Record its process session after starting it.

Current-source live passed at `current-live/1791794489409287949-2/verdict.toml`:
four 1000-input subjects, exact state/effects, completed through post-cleanup
authentication. Shared heavy host load caused four bounded proc-scan truncations;
resource samples incomplete, so no benchmark admission. External cleanup/dedup
both zero bytes; retained authentication passed four subjects/108 artifacts each.
Verdict SHA `6e93f2d32f2fef662661c5cf24f4959ae49fe4c810595cdc09ecd37d664eb981`.
Free disk 219,507,273,728 bytes. System/decomp/RNG post-cleanup/current-source
authentication logs are under `out/ticket-03/logs/`.

New telemetry control source copied under `out/ticket-03/telemetry-control/`
preserves the original historical control. It uses the runner's host monitor
for every operation, stops native writers on interference and cleans/authenticates
retained subjects before refusing; build session 60839, log
`logs/telemetry-harness-build.log`. No admitted overhead run started yet.
`out/ticket-03/benchmark-preparation.toml` defines final-source workload/budgets
and fresh private capacity-two/four roots. Do not start admitted measurements
until ticket 02 passes. Original main capacity-four root is untouched.

User agreed to a quiet measurement window. Guarded preflight attempt
`telemetry-preflight-control` stopped early at external CPU fraction 0.277 >
fixed 0.25; native child/producer groups stopped, failure cleanup/dedup and
external scoped cleanup/dedup all succeeded, zero bytes. No accepted timing.
Fresh guarded six-sample control is session **49753**, output
`out/ticket-03/telemetry-quiet-final`, stdout/stderr in `logs/` with that prefix.
It uses ticket-03 retained current pipeline CLI and ticket-02 retained identical
producer/reader binaries. Order and full boundary unchanged. Continuous host
snapshots are written per sample. Do not compile during this control.
First sample started with no host compiler/browser workloads; initial external
CPU fractions 0.04–0.08. Gate still pending. No target build trees remain.

Session49753 stopped after38s: raw external CPU0.277 from brief Codex daemon/
agent bursts (six ticks each in500ms), no competing compiler/browser. Native
failure cleanup/dedup and external cleanup/dedup succeeded, zero bytes. No
timing admitted. Updated host monitor to retain raw fractions and enforce the
unchanged0.25 limit over five seconds, with five-second preflight, documented
in benchmark-execution.md before admitted samples. New source host.rs change
needs final pipeline release/fmt/Clippy after control; telemetry harness built
successfully against it and its disposable target was cleaned (70.3MiB).

**Active control session85476**, root `out/ticket-03/telemetry-quiet-rolling`.
Uses retained ticket03 pipeline and ticket02 producer/reader. Six-sample order
disabled/enabled/enabled/disabled/disabled/enabled. Host fractions initial
rolling0.05–0.07; first disabled capture underway. No compilation while active.
Do not confuse previous stopped session49753 with this active writer.

First comparable rolling-window control sample completed disabled:
529,113,032,159ns. Four exact original observations, capture C~124/125s and
mixed~109s, cleanup/dedup and retained authentication completed. Host rolling
CPU~0.04–0.10 throughout. `samples.toml` is authoritative. Sample2 enabled
is running in session85476. No overhead conclusion from one sample.

Sample2 enabled completed565,550,236,893ns (+6.9% versus sample1), exact
observations, complete capture resources/no truncations, cleanup/dedup and
retained auth passed. Full six-sample median gate remains pending; do not mark
02 resolved. Sample3 enabled underway in session85476. If median overhead
exceeds2%, reduce bounded telemetry sampling cost and rerun full equivalent
control; preserve these original diagnostics and unchanged producer/reader.

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
