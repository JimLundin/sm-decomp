# Ticket 02 restart checkpoint

Updated 2026-10-12 after recovering the crashed session.

Scope confirmed by the user: complete optimization and improvement of the
pipeline itself. A separate session resumes actual decomp work. Gameplay runs
here provide regression evidence for pipeline changes; new porting work is
outside this session. See `spec.md` for the completion boundary.

Active integration: `/tmp/sm-pipeline-upgrade-integration`, branch
`pipeline-upgrade-integration`. The developer checkout remains at `3ad1bf3` on
`decomp-port-integration`, clean and unchanged by this continuation.

## Implemented and integrated

Recovered uncommitted metrics work in `/tmp/sm-pipeline-upgrade-metrics` was
verified and committed as `a3290f5`. Merge `bd82cf2` integrates it; merges
`8d1f071` and `3260224` record the resource/evidence support branches whose final
test files were already identical. Those three clean integrated implementer
worktrees have been removed. See `evidence/metrics-recovery.md`.

The shared-clock root/nested receipts now retain individual scope transport,
configuration and stage/process ownership; wait4 reaps each owned process once.
Root process-tree and allocated-disk sampling, cancellation teardown, readiness
timeouts, C/compiler/cache accounting, collector/import/authentication stages,
and complete live cleanup/authentication are integrated. Text transport and
sm-evidence-v2 remain in use.

## Verified evidence

Stable evidence root: `out/ticket-02-final/` under the integration worktree.
Retained release tools: `retained-tools/sm-pipeline`, `sm-evidence`, and
`telemetry-control`. No disposable Cargo target is needed to read this evidence.

- Decompressor pointer: `decomp/decomp-acceptance.toml`, run
  `decomp/1791787925659241965-2`: accepted 40 synthetic plus 23 captured
  fixtures, three original-ROM checks and eleven failure controls.
- RNG: `rng/1791787985280153590-2/verdict.toml`: accepted exhaustive 65,536
  seeds and four 9,000-frame subjects.
- System: `system/1791788000875167-2-0/verdict.toml`: 475 verified cases.
- Live: `live/1791788150135886861-2/verdict.toml`: four fresh 9,000-frame
  subjects with system/decompressor/RNG installed and exact state/effects.
  Root `pipeline.metrics.toml` reports success through
  `post-cleanup-authentication`, complete resource samples and elapsed
  403,799,239,476 ns. This is correctness evidence under host load, not a
  benchmark result or the full projectile promotion gate.
- All four public authentications passed after final cleanup and worktree
  removal. Live verdict SHA-256:
  `1feaaffa8155a3b75167d483cb70a5c7f60fbbbc870cbf7abdfea38f86214b5b`.

## Remaining work

Ticket 02 remains **claimed**: the <=2% telemetry overhead gate is unproved.
Ticket 03 remains blocked by 02; conditional binary tickets remain unclaimed.

An actual four-subject 45,000-input on/off control was prepared in
`out/ticket-02-final/telemetry-control/`. It uses the same admitted live/reader
binaries, three alternating pairs, independent empty stores, exact observation
comparison, publication/authentication and cleanup/retained authentication.
Prepared binary builds are excluded; this is a telemetry control, separate from
ticket 03's frozen cache/store matrix. Host snapshots accompany each sample.

The first disabled sample overlapped unrelated host processes consuming about
two CPU cores and several GiB of memory. It was explicitly stopped by SIGTERM;
its receipt records `interrupted`, all four subjects report interruption, no
accepted pointer exists and its FIFOs are removed. No timings/median/overhead
success are claimed. Partial staging/store diagnostics remain in
`telemetry-pairs/sample-1-disabled/`, deduplicated by the cleanup utility.

Resume on a quiet host with a **new output directory** (the runner creates
immutable sample directories):

```sh
out/ticket-02-final/retained-tools/telemetry-control \
  /tmp/sm-pipeline-upgrade-integration/out/ticket-02-final/retained-tools/sm-pipeline \
  /tmp/sm-pipeline-upgrade-integration/out/ticket-02-final/live/1791788150135886861-2 \
  /tmp/sm-pipeline-upgrade-integration/out/ticket-02-final/telemetry-pairs-quiet
```

Check host activity throughout before accepting performance results. If
telemetry overhead exceeds 2%, reduce sampling cost and revalidate before
resolving 02. Keep ticket 03's complete controlled baseline/profile/selection
and the final 129,712-input projectile gate separate and unchanged.

## Recovery diagnosis and cleanup

The old full unit-suite failure was a missing `rom/sm.sfc` symlink in the metrics
worktree: the single palette generator failed before restoring the link and
passed afterward; the complete 44-domain fixture comparison passed in 150.10s.
The old decompressor attempt used a relative original-ROM path from that
worktree and remained unaccepted. A fresh current-source run with an absolute
ROM and the original capture manifest passed. Copied `captures.toml` files in
sealed decompressor runs are not relocatable replay manifests: their relative
ROM/WRAM assets still belong to the baseline fixture directory.

Native cleanup of previous failed runs reclaimed 0 bytes; final run cleanup
reclaimed 0 bytes and deduplication reclaimed 332,450,186 bytes across 2,162
snapshots. Retained evidence authenticated afterward. Disposable Cargo targets
were cleaned, the old failed fixture matrix was removed, and integrated support
worktrees were removed. Logs and measured free space are recorded in
`evidence/metrics-recovery.md`. No verification process from this continuation
remains running.
