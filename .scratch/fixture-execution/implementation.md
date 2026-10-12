# Production fixture execution

Implemented the shared fixture execution module. It schedules independent work
within the shared worker capacity, waits for complete predecessor success,
blocks descendants with causal records and returns canonical report order.
Family/callback/PLM adapters use its prompt execution and aggregate metrics.
Ordinary families on Linux x86_64 default to preloaded Rust/C subject workers
with fresh fork children; callback/PLM retain fresh processes. `--execution fresh`
keeps the comparison path. No Python source/runtime added.

Worker parents never invoke gameplay. ROM loads once; requests carry explicit
inputs/outputs, directory, identity and deadline. A child leads its own process
group; helper waitid observes it without consuming its group identity, kills
remaining group members, wait4 reaps the leader, then reaps adopted descendants.
Orchestration uses nonblocking bounded framing and signal-aware deadlines.
Cancellation gives helpers time to stop descendants before forced teardown.
Incomplete replies are failures; no implicit fresh-process retry occurs.

Per-case state/return/host/capture/stdout/stderr and command/timing evidence stays
sealed. Fork receipts add per-child CPU/RSS/reap detail. Aggregate process records
retain fixture identities and helper-inclusive CPU without duplicating child CPU.
New execution/worker/metrics/sampler sources and generated worker archive join
explicit source/binary seals. Missing execution contracts cannot authenticate new
evidence; historical evidence remains under its original contract.

## Verification

- Production projectile family: 1,840 fixtures, 4 routines, fork, two workers;
  offline-verified and authenticated after cleanup. Run:
  `/tmp/sm-fixture-production/projectile/1791825003690054-3-0`.
- Exact equality with retained original: 18,400 input/output/stdout/stderr files.
- Full command 71.85 s (builds/sealing included); metrics interval 69.646 s;
  3,680 fixture-child execution span 31.083 s. This run overlapped callback
  validation and briefly its compilation, so it is not a controlled benchmark.
  Retained original full command approximately 315 s on a different session.
- Production system family: 475 fixtures, 17 routines, no ROM, fork, two workers;
  16.92 s total, authenticated after cleanup. Run:
  `/tmp/sm-fixture-production/system/1791825728666503-3-0`.
- Final family suite: 12 controls passed in 55.31 s; native projectile check
  separately passed in 39.91 s. Covers chain overlap/order/blocking, Rust-before-C
  refusal, parent isolation, tampering/source freshness, batch reuse, cancellation
  including a descendant and stale live admission.
- Real worker controls: 2 passed, mutation/crash/timeout/recovery/descendant
  reap and incomplete/oversized framing.
- Callback ordered captures/amount/tampering passed; PLM allocation amount,
  ABI controls/tampering passed. Live ABI controls: 3 passed. Process metrics:
  6 passed; resource metrics: 5 passed. Generic process behavior preserved.
- `git diff --check` passed.

One initial native suite check rejected a stale executable after an edit during
its run. Rebuilt with frozen source; the native check and family controls passed.
Test-run cleanup used the project utility before temporary test trees were removed.

## Live validation

Evidence root `/tmp/sm-fixture-production` retains all diagnostics. Initial live
attempt stopped at an older RNG predecessor's relative missing dependency.
Second stopped at linking because the original evidence working directory
selected a different Rust toolchain. Third used the project toolchain and ran
1,000 frames; projectile admission rejected the empty activity window. Its
coverage gate is preserved. The short system integration replay uses the new
system admission, original evidence working directory and explicit
`RUSTUP_TOOLCHAIN=nightly-2026-09-29`. Result and cleanup recorded in ticket04.

Existing decomp and final RNG predecessors authenticated and were reused
without changing them. Runtime/live replay code was not modified. Native
projectile full-route live coverage was not established by this short window.

## Cleanup and retained dependencies

Every finished production run, including live failures, receives both project
cleanup passes. Projectile dedup reclaimed 554,696,704 bytes (5,520 snapshots);
system dedup reclaimed 156,368,896 bytes (1,425 snapshots). Both family verdicts
authenticated afterward. Failure runs reported zero bytes. Final live cleanup,
build cleanup/free-space evidence and remaining dependencies are in ticket04.

Retain the family verdicts, generated sources/binaries, original source checkout
and manifest paths. Packed live evidence also requires its store, per-subject
evidence pointers and retained reader. Existing shared stores/cache and historical
source snapshots remain intact. Implementation and validation are complete on
`decomp-port-integration`; ticket 04 records the implementation commit.
