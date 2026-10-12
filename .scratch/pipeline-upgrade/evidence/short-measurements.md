# Bounded measurements and selected improvement

The user rejected hour-long telemetry tuning on 2026-10-12. The original
12-cell matrix and repeated 45,000-input telemetry controls are deferred.
There is no passing large-workload 2% telemetry-overhead claim. Detailed
telemetry is now opt-in for ordinary verification; benchmarks explicitly enable
it. Exit status, one-owner wait4 accounting, timeout/cancellation and elapsed
receipts remain available in normal verification. Sampled resource fields remain
unavailable when sampling is disabled, never fabricated as zero.

## Attribution

The actual resource sampler over approximately 700 snapshot files takes median
0.024 ms for RSS and 0.814 ms for RSS plus allocated-disk sampling. This alone
does not explain the stopped long control's apparent overhead. That control's
five completed samples cannot establish a passing gate; sample six was stopped
at the user's request. Retain its failure diagnostics and accepted observations.

Repeated legacy restoration/authentication is a measured cost. The selected
change renders each PPU/APU event into an 88-byte stack buffer with itoa decimal
conversion and a single write, preserving exact legacy bytes and canonical
objects. No binary transport change is selected.

Five authenticated restorations of the same retained 23,891,991-byte stream:
baseline median 442,694,984 ns; candidate 64,429,448 ns (85.4% reduction).
Receipts: `out/ticket-03/restore-{baseline,candidate}.toml`.
Both stream IDs are
`548782cb106543c0e52262e7ac07d3848092dc86b039886536387290003e61a3`.
The retained 1,739,672,236-byte stream restores with candidate median
5,444,594,756 ns, five repetitions; no paired large baseline is asserted.

## Short complete-boundary comparison

`out/ticket-03/reader-comparison-short/report.toml` records 1,000 inputs,
four fresh subjects per sample, capacity two, independent empty stores,
prepared identical producer/runner binaries and fixed enabled telemetry.
Order: baseline/candidate/candidate/baseline. Measured boundary includes capture,
canonical comparison, publication, authentication, cleanup/deduplication and
retained subject authentication. Builds and five-second host preflight are
excluded. Continuous host receipts enforce the existing external CPU guard.

| Reader | Samples (seconds) | Median (seconds) |
| --- | --- | --- |
| Baseline | 15.680693, 15.253377 | 15.467035 |
| Candidate | 12.393699, 12.542504 | 12.468102 |

Short complete-wall reduction: **19.389%**. This is two repetitions per variant,
with OS page-cache/host warmth uncertainty; it is neither the original matrix
nor a large-workload speedup claim. All exact observation IDs match across
variants. Binaries and hashes are sealed in the report. The Rust experiment
source is retained in `out/ticket-03/telemetry-control/src/reader_compare.rs`.
The first launch rejected relative executable paths, admitted no timing and
completed diagnostic cleanup; corrected absolute paths supplied all samples.

The short enabled/disabled control is separate:
`out/ticket-03/telemetry-short-decimal/report.toml`. Four samples take
14.888/10.768/10.623/12.569 seconds. The arithmetic -22.095% result is noisy
and cannot establish precise telemetry cost or the large-workload ceiling.
Enabled capture receipts report complete resource samples, zero missing
processes/paths and zero truncated scans.

## Cleanup

Both short-control and reader-comparison global cleanup/dedup receipts succeeded
with zero bytes reclaimed; per-sample cleanup and retained authentication passed.
Receipts are `logs/short-control-{cleanup,dedup}.toml` and
`logs/reader-comparison-{cleanup,dedup}.toml`. No sealed objects were relocated.
The comparison harness build target was cleaned: 481 files / 70.6 MiB.
Final integration regression, source retention and cleanup are recorded
separately before resolution.
