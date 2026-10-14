# Measured performance and concurrent equipment progress

The performance changes are integrated on `affected-family-assignment` through
commit `3c2e9e3`.
Merged `cargo check -p pipeline --bin sm-pipeline --locked --offline -j 1`
passed in 58.51 seconds. Both independent source-review axes found no issues;
see [the review](performance-review-20261014.md).

## Complete Native execution

These measurements execute the complete original `enemy-jump-instructions`
family: 16 Fixture pairs, two routines and 34 producers, with unchanged replay
and original refusal controls. Each reader and all its dependencies used normal
release optimization level 3; both sandbox helpers used optimization level 1.
Inputs, coverage and original command limits were preserved.

| Elapsed seconds | Baseline, V7 helper | Input reuse + four workers, V7 | Both changes, V6 helper |
| --- | ---: | ---: | ---: |
| Positive proof, authentication and replay | 17.662 | 13.317 | 10.682 |
| Complete original control test | 43.11 | 33.54 | 25.56 |
| Fixture wall span | 5.480 | 1.120 | 0.843 |
| Compiler producer totals | 10.748 | 10.834 | 8.820 |
| Finalization after last producer | 0.512 | 0.499 | 0.376 |

The combined control test uses about 41% less elapsed time. Its fixture phase
is 6.50 times faster. These are complete small-family observations on a shared
host, not a whole-catalog throughput claim. Owned competing builds were held;
unrelated host activity was not stopped. Compiler variation is shown explicitly.
The first two columns were adjacent runs; the combined check followed the
final sandbox guard and measurement window.

The Native changes reuse protected input identities after full metadata checks
and run independent fixtures in bounded waves. Successful predecessors still
precede continuations. Every admitted worker joins on failure, later waves stop,
and aggregate output/read/path/deadline budgets remain shared. Historical intents
retain serial execution through their defaulted stored job count.

## Parallel execution

Only fixture job count changes in this fixed-V7 comparison; the shared pool
always has four slots:

| Fixture jobs | Fixture wall seconds | Throughput relative to one job |
| ---: | ---: | ---: |
| 1 | 3.103 | 1.00x |
| 2 | 1.805 | 1.72x |
| 4 | 1.212 | 2.56x |

Compiler variation made total times non-monotonic in this particular comparison;
the fixture span is the scheduling measurement. A future family-level scheduler
must use this same shared pool. Compact capture/reference compilation and the
two independent replay comparisons are further opportunities under their
unchanged whole-experiment deadline; those changes are not implemented here.

Independent subsystem implementation can continue alongside these runs, as the
equipment result below demonstrates. Builds and fixture workers share the same
four-process capacity across runs. Each continuation chain remains ordered;
independent roots provide the parallel work.

A separate architecture effort retains candidates for scheduling, build reuse,
evidence handling and Live execution, alongside the adopted sandbox changes.
Its [composition handoff](/home/jim/sm-decomp-architecture-20261014/03-builds/.scratch/architecture-performance-20261014/composition.md)
records 32 passing controls. That composition predates these Native changes;
combining them must preserve shared reservations, fixture waves and build-cache
source registration. Its five worktrees remain separate, and the measurements
above cover the changes integrated in this checkpoint.

## Sandbox accounting

The sandbox now tracks affected inodes and in-flight targets instead of scanning
every retained file and descriptor around every write. It preserves exact quota
checks, follows actual shared FD tables and retries incomplete directory scans
without treating missing entries as zero usage.

Final same-reader medians of three runs, 256 operations and 256 retained files:

| Workload | V7 seconds | V6 seconds | Speedup |
| --- | ---: | ---: | ---: |
| Regular writes | 0.174422 | 0.024090 | 7.24x |
| Pipe write/read pairs | 0.195146 | 0.044410 | 4.39x |

Eight-job V6 batches at one/two/four workers took 1.654305/0.893009/0.607344
seconds for regular writes and 1.826053/1.037842/0.699265 for pipes: 2.72x and
2.61x four-worker throughput. The final helper passed 16 runtime controls,
eight original guards and 32 stress repetitions. The Native changes passed
31 focused pipeline and 12 operation controls plus all complete real-family
controls. The combined Native/V6 control test also passed.

The explicit retained V6 helper is
`/home/jim/sm-decomp-performance-20261014/confinement/tools/sm-translation-sandbox-fd-table-v6-opt1`,
SHA-256 `8f825e97e00ce534878131d6838411aee99576f7e0e2a088e1501b0c67cda0b0`.
The Native reader is
`/home/jim/sm-decomp-performance-20261014/native/tools/native-tests-waves-v2`,
SHA-256 `78804b95e8a815f3b074910e85ec5b54f7e5d054ec9979b74192f15fe7b30bdb`.
Rebuilding pipeline alone does not select or install this helper. Existing
production Installation, registry, fence, claims and original stopped attempt
are unchanged.

## Concurrent decomp

Private equipment source commit `847d76f` implements all 29 routines. All 13
independent Rust controls and all 427 declared actual-C Fixture pairs pass,
including 181 complete-root cases and both continuation chains. The complete
Native proof, replay and post-cleanup authentication took 97.78 seconds with
two fixture workers while other work continued. This elapsed time is not a
quiet-window benchmark.

The shared inventory has `translated` and `verified` observations. After review,
the source and manifest remain unchanged. Main integration, Live replay,
hardware effect order and original-ROM parity remain separate. See
[the equipment handoff](../../equipment-interaction-concurrent-20261014/verification-handoff.md).

## Retention and remaining work

Both performance worktrees were retired after integration. Their selected
source archives occupy 4,673,536 and 770,048 bytes under
`native/source-final-dfc06db` and `confinement/source-final-ccc04bc` in
`/home/jim/sm-decomp-performance-20261014`; existing exact proof-source snapshots,
readers, helpers, private inputs and diagnostics remain retained. Seven Native
proof sets and the 856-producer equipment proof authenticate after retirement.
All retained confinement and historical manifests also pass their integrity
checks after retirement.

Identified build targets/worktrees removed totaled 2,621,067,264 allocated bytes,
including the earlier 54,501,376-byte private equipment target. Scoped duplicate
cleanup reclaimed another 146,014,208 bytes. Normal cleanup passes reported
zero additional disposable bytes. The main checkout's pre-existing shared Cargo
cache remains available for subsequent checks. Last observed available space:
187,063,205,888 bytes; concurrent sessions also change disk usage.

The combined proof is `native/combined-v6-controls-v1`; its original test,
phase analysis, two cleanup passes and post-retirement authentication are
`native/logs/combined-v6-controls-v1*`. Equipment source/run pointers and its
91,488,256-byte deduplication are recorded in its handoff. Complete worker
details are in [Native](native-performance-handoff.md) and
[confinement](confinement-performance-handoff.md) handoffs.

Ticket 10 remains claimed. No complete 177-family/150,204-pair catalog run was
started. Remaining work includes the largest-family reference/finalization
limits, finite whole-catalog budgets, the parked journal-bound Original reuse
implementation, installed Original acceptance and ordered continuation/recovery.
The original 600-second Cargo dependency proof has not been repeated unchanged.
JSON format maintenance remains deferred.
