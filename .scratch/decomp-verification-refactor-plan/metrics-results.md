# Ticket 01 retained measurement and handoff

Status: worker implementation and bounded controls complete; parent owns ticket
acceptance, integration and retirement.

Implementation/source checkpoint:
`eb8811829ef080d35a61b0c859ba1787908b6e18`, following
`e08619089313c187dbaacc279a4a5c36adc19007`. The production changes are the same
in both commits. The later commit adds the retained measurement harness and
passing control records. No production installation or Native evidence contract
was changed.

The earlier red control proved the defect: a 3.32114051-second invocation
reported a seeded 86,400-second historical duration. Current-invocation,
original-run and reuse-authentication durations are now separate. Fresh,
reused, rejected and interrupted work all passed the public actual-C controls.
The phase recorder also retains complete counts/summed durations/outcomes when
bounded individual traces omit events. Benchmark time unions are unavailable
unless every contributing interval is retained.

## Controls

Thirteen tests passed: nine public metrics controls, one benchmark attribution
control, one real batch reuse/rejection/fresh control, one real batch
interruption/recovery control, and one retained measurement control. The ignored
metrics subprocess helper was invoked only by its owning tests.

The exact commands, initial build setup failure, corrected harness compilation,
durations, cleanup and executable hashes are in `metrics-handoff.md` and
`/home/jim/sm-decomp-performance-metrics-proof/`. Each executable control was
bounded by `timeout --signal=TERM 60s`; builds had a separate 900-second bound.
Cargo used locked/offline dependencies, jobs1, no extra features and dev/test
debug information disabled. New actual-C controls used four shared process slots,
one family worker and one Fixture pair worker; existing isolated capacity tests
were unchanged.

## Retained workload

Evidence root:
`/home/jim/sm-decomp-performance-metrics-proof/baseline-20261015`.
The authoritative measurement is `measurement.toml`, with eight complete
per-invocation receipts under `samples/` and stage/resource sidecars under
`batch/`. The measurement test passed in 26.23 seconds. Its measured boundary,
from the first invocation through cleanup and independent retained admission,
was 25.072966732 seconds; initial source/tool preparation is outside that boundary.

The complete declared domain is one routine / one actual-C and Rust Fixture
pair. Both fresh invocations executed that domain. This is an instrumentation
baseline, not complete required-catalog or gameplay acceptance.

| Fresh scenario | CLI seconds | Batch seconds | Owned child records | Eligible cache actions |
| --- | ---: | ---: | ---: | --- |
| Empty private cache | 3.795068397 | 2.902898354 | 17 | 2 misses, 0 hits |
| Existing private cache | 3.727782679 | 2.645244151 | 15 | 0 misses, 2 hits |

These are observed hits from the sidecars and sealed cache result records, not
an inference from the cache directory's existence. Other required compilation,
linking, metadata and execution work remains. The single cold/warm observations
do not establish a repeatable latency improvement.

The six reuse invocations authenticated the same first verdict and launched no
new child processes. They retained its original 2.829105006-second family duration.

| Pair | Sampling off: CLI s | Sampling on: CLI s | On minus off: ms | Off: authentication s | On: authentication s |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 1.594732520 | 1.567685041 | -27.047479 | 0.611534395 | 0.487020477 |
| 2 | 1.539428972 | 1.452756299 | -86.672673 | 0.348521755 | 0.394809973 |
| 3 | 1.602778190 | 1.577451425 | -25.326765 | 0.287830361 | 0.410274513 |

Disabled CLI median: 1.594732520 seconds, range
1.539428972–1.602778190 (63.349218 ms spread). Enabled CLI median:
1.567685041 seconds, range 1.452756299–1.577451425 (124.695126 ms spread).
The median difference is -27.047479 ms, below the predeclared useful-change
threshold of 5% or 20 ms and below observed variation. This does not establish
a speedup or a reliable incremental end-to-end sampling cost.

The internal batch median changed from 0.380886973 to 0.485092985 seconds;
its +104.206012 ms difference is smaller than the disabled samples' 301.246315 ms
spread. The three paired internal differences have mixed signs. Do not use the
internal median alone to declare a sampling regression.

Enabled runs recorded 7, 6 and 6 sampler calls, with summed sampler work of
2.008063, 1.087529 and 1.192095 ms. Disabled runs recorded zero calls and zero
RSS/disk samples. Those probe sums are not additive wall time and do not include
every instrumentation cost. Sampling remains opt-in.

Per-sample host observations include process names/CPU/RSS, load and available
memory. No concurrent compiler appeared in the reuse boundary snapshots; load
was low (about 0.25–0.36 during those observations). These snapshots do not
establish complete host isolation, and measurable variation remains. Earlier
build/correctness observations were taken on a contended host and are not
comparable throughput samples.

## Cleanup and retention

Automatic batch deduplication reported 131,072 bytes on the first fresh run and
393,216 on the final fresh run (524,288 total). Both explicit cleanup passes on
the finished batch and selection directories reported zero additional bytes.
The control removed its private cache and temporary `.git`: 22,328,985 logical
file bytes. The private cache alone had 6 files, 22,036,528 logical bytes and
22,044,672 unique allocated file bytes before removal. The workers symlink was
removed without following it; shared worker capacity was preserved.

The final batch has 280 files, 64,860,745 logical file bytes and 64,966,656 unique
allocated file bytes. These include evidence, source copies and metrics.
Logical size, allocated blocks and the cleanup utility's reported bytes are
different accounting measures. Physical OS disk I/O was not measured.

Cleanup took 3.509835647 seconds. Independent admission of both retained fresh
verdicts, using the copied reader after private cache/.git removal, took
4.484185267 seconds. Free filesystem space after measurement was
119,772,553,216 bytes. Its stdout/stderr are in `post-cleanup/`.
The metrics private TMPDIR is empty. Required retained
dependencies are:

- The evidence root's `crates/`, `deps/` and `tools/` source/input snapshot at its
  existing absolute path. The verdict's manifest and source-root references use
  this root, not the implementer worktree.
- Both `batch/families/fixture/evidence/*` runs, their verdicts, full Fixture
  results, source copies and binaries, plus the batch/metrics/sample receipts.
- `retained-tools/sm-pipeline`, SHA-256
  `0820872e413d28b60a23990bf8db23982ef3cac6b0c3bf70b8a235cebe201352`.
- `retained-tools/family-control`, SHA-256
  `578743d6f1b6b6b523b50c6b1393696c5241e8bd11511b778a4d0a6f54d58c3a`,
  and `measurement-source.rs`, SHA-256
  `242cd34b3244c6e24d6ffe63ad4af60aaa53da3b5b179bdc135cb90edcdc5f12`.

`/home/jim/sm-decomp-performance-metrics-proof/baseline-retained.sha256` hashes
the measurement, all eight sample/metrics files, both verdicts, retained tools,
control source, manifest, and post-cleanup requests/stdout/stderr. Its SHA-256 is
`e3581965c20be7906be7c85bf09d2d66ba52c55877ab249ee158a62f560f054c`.

The control source and measurement record retain compiler release information;
the ordinary evidence seals cache identity keys and the exact compiled Fixture
binaries. The deleted cache's expanded compiler/environment seals are not a
retained evidence dependency. Final baseline/candidate comparisons should retain
those expanded toolchain records as well as the required evidence; this small
control is not a frozen full-catalog speedup comparison.

After worktree/target retirement, root should repeat these read-only admissions:

```sh
/home/jim/sm-decomp-performance-metrics-proof/baseline-20261015/retained-tools/sm-pipeline family-authenticate < /home/jim/sm-decomp-performance-metrics-proof/baseline-20261015/post-cleanup/0.request.toml
/home/jim/sm-decomp-performance-metrics-proof/baseline-20261015/retained-tools/sm-pipeline family-authenticate < /home/jim/sm-decomp-performance-metrics-proof/baseline-20261015/post-cleanup/1.request.toml
```

## Parent actions

No worker-owned writer/build remains. Root retains ticket 01 resolution,
acceptance against the extraction's recovered final baseline, source integration
and implementer retirement. Both warmed targets were explicitly released:

- `/home/jim/sm-decomp-performance-metrics-target` to `native_admission`.
- `/home/jim/sm-decomp-performance-metrics-continuation-target` to `bulk_context`.

Do not delete those targets while their new owners are active. Their final
cleanup belongs to root. The metrics worktree can be removed after integration;
remove its `deps/rustysnes` and `deps/snesrev-sm` links without following them.
Keep the independent evidence/source/reader root above, and reauthenticate after
retirement. The archived integration test binary additionally has compile-time
worktree/controller paths; reproducing the whole control after their retirement
requires rebuilding the recorded source or restoring those paths. Standalone
admission uses only the copied reader and retained evidence/source root.
