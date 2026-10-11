# Optimization branch: measurements and remaining uncertainty

Reviewed 2026-10-11 using read-only Git objects from optimization revision
`e820a6b77338eb4e84a2d91baeed2dc9f33c72f1` (abbreviated `e820a6b`), based on
`16f18837023d1498f46aa555a1f0f6cc0c938092`. No physical optimization worktree,
run directory, build or replay was accessed. This note compares that revision
against [the decomp pipeline recommendations](pipeline-review.md).

Sources below use explicit `revision:path` Git object identifiers. Optimization
files are absent from this checkout, so they are not presented as local links.
They can be inspected with `git show 'e820a6b77338eb4e84a2d91baeed2dc9f33c72f1:<path>'`.

## Assessment

The optimization branch implements most of the recommended first wave: shared
worker leases, cached binaries/C objects, actual family compiler dependencies,
parallel fresh packed subjects, verified inventory traversal and supported shared
import. It also extends batch selection and parallel isolated fixture pairs.
The main remaining performance task is a controlled measurement campaign with
complete end-to-end and resource accounting. The branch's resolved tickets do
not establish every measurement originally requested: its implementation report
explicitly says CPU and peak RSS per stage are still absent. Full 129,712-input
packed throughput is also outside its reported 1,000/9,000/45,000-input comparison.
Sources: [I], [T2], [T3], [T8], [L], [LL].

## What the checked-in numerical results actually cover

| Evidence | Scope and timer | Recorded result | Practical limit |
| --- | --- | --- | --- |
| Initial live differential study | Frozen live/reader binaries; four fresh 1,000-frame subjects; `wall_seconds` spans capture, sealing and exact comparisons | Serial/shared 36.61 and 59.69 s; parallel/shared 68.25 and 17.90 s; parallel/private 17.35 and 8.05 s | Builds and subsequent full authentication excluded; shared store still serializes producers |
| Instrumented live study | Same 1,000-frame experiment, one further sample per configuration | Serial/shared 35.18 s; parallel/shared 37.30 s; parallel/private 19.16 s | Host load averages around 9–13 on an eight-logical-CPU host; offline work overlaps this sample |
| Original offline family study | Actual system 475 cases and audio 1,147 cases, including builds and retained authentication | One worker 63.58 / 96.67 s; two workers 42.10 / 55.39 s | Two small families, fixed order, concurrent unrelated load; no production-wide speedup estimate |
| Study-only compiler closure | One broad/narrow system compile and replay of 475 actual-C cases | Broad 5.136 s / 26 input paths; narrow 0.891 s / three input paths; all WRAM/returns match | Narrow-system feasibility, not a measured speedup for every production family |
| Later implementation integration | Fresh combined-source 1,000, 9,000 and 45,000 frame serial/parallel integrations | Narrative capture totals excluding scheduling wait: about 14.4 / 5.3 s, 144.0 / 58.6 s, 900 / 233.7 s | Final raw run timings/verdicts are referenced under `out/` but are not checked-in branch objects inspected here; totals exclude publication and other end-to-end costs |
| Compiler cache implementation | Evidence-reader cold build vs later warm restoration | 90.27 s cold / 0.228 s warm | Narrow build/restore measure under host contention, not full verification runtime |

The first four rows follow the JSON and study harness directly ([B], [L], [LL],
[O], [C]). The last two are reported implementation evidence ([I], [T4]); they
should be labeled as reported measurements, rather than independently reread raw
final-run results. The final run paths and verdict hash in [I] provide follow-up
targets without requiring this review to touch the protected worktree.

The initial live results measure authentication separately: 5.09–17.38 s across
the six samples in [L], and 9.19–13.17 s across the instrumented samples in [LL].
Those costs must be added when discussing capture-plus-authentication, while
still accounting for excluded builds, final publication and cleanup. These are
not end-to-end pipeline figures. All nine samples record exact inventories,
roots, progress and ownership; [A] records post-cleanup authentication of those
same nine historical binary/observation verdicts plus eight offline family runs.

## Timer semantics and narrative qualifications

The branch correctly identified a misleading historical field. Initial
`producer_seconds` starts before store acquisition, and the instrumented shared
sample measures waits of approximately 0.001, 17.533, 10.617 and 31.178 s. Its
parallel/shared 17.90 s sample therefore does not prove overlapping production.
Private-store waits in the instrumented sample are about 0.001–0.003 s. [LL], [F]
and the before/after `tools/port_evidence.py` diff establish the lock explanation.

At `e820a6b`, production `packed_replay` records store wait, worker wait,
producer runtime and sealing separately. `pipeline_seconds` still includes both
waits and collector startup; producer timing begins just before producer launch
and ends after its polling loop. Sealing includes collector completion and staged
file hashing/comparison. Pointer publication and staging deletion occur after
the pointer's pipeline timestamp; the separate final `pipeline-timing.json`
covers a longer interval. This granularity is useful, but it is not a complete
profile of encoding, snapshot I/O and closure checking individually. Source: [P].

`publish_subject` starts its `publication_seconds` timer before shared-store and
worker leases, so that field includes both queueing and the import operation.
It stops before durable pointer replacement, the subsequent independent
`authenticate_subject`, and private-store removal. [PL] copies this value into
verdict storage metrics. Thus the reported 5–6 s short-run shared publication
cost is not a full publication-through-cleanup stage, and final long shared
import/wait costs need explicit presentation. Source: [P], [PL], [I].

Offline study stage timings overlap. `build-total` contains compiler execution;
`hashing-total-overlapping` is deliberately aggregated inside other calls. They
must not be added into a total. The first audio sample spends about 24.92 s in
Rust/C fixture subprocess execution, about 6.14 s authenticating evidence and
0.16 s authenticating the source archive. This supports prioritizing process,
build and evidence work before source-archive shortcuts. Sources: [B], [O].

[I] contains chronological provisional statements such as “remaining verification”
and “serial final publication/admission remains underway”; its later final section
reports both long verdicts completed and authenticated. Those earlier statements
are superseded by the final completion section and resolved tickets. Conversely,
[X] reports the historical broad system invalidation by an unrelated audio change;
it does not test the newly narrowed production contract. The new behavior is
reported separately in [T5]. Reading the historical JSON as a current failure
would be incorrect.

The pre-final long captures also used older store/worker lock ordering. The
implementation later acquires shared-store ownership before a CPU lease, avoiding
CPU tokens held by import waiters. A final optimized 1,000-frame run checks the
new order. Retained long-run byte equality remains useful, but long timings must
retain their pre-fix scheduling label rather than being advertised as final-code
throughput. Source: [I], [P].

## Recommendations that remain useful

1. Collect machine-readable final-source timings for quiet-host jobs 1/2/3 and
   live workers 1/2/4, separating cold/warm tools, empty/warm shared stores,
   scheduling, import, post-import authentication and cleanup. Repeat short and
   representative 45,000-input cases; add a full-route packed run after useful
   exact candidates stabilize. The branch's fixed-order, contended samples prove
   feasibility but do not establish a stable numerical speedup. [F], [I], [B]
2. Add CPU time, peak RSS, peak private/staging bytes, bytes written and I/O/swap
   observations before selecting hardware capacity. Retained private bytes are
   approximately 26.7 MB per 45,000-frame subject for 1.9–2.6 GB logical data;
   that is compression/retention evidence, not peak memory or physical disk
   occupancy throughout capture/import. [I], [T2], [T3]
3. Use the implemented local shared budget with consistent
   `SM_PIPELINE_WORKERS` and shared root across worktrees. The default is four
   leases, rather than one lease per logical CPU. One lease can cover a
   producer/collector pair; it is an activity budget rather than an OS process
   limit. Bootstrap Cargo uses one serialized job. A dedicated runner can isolate
   accuracy work from throughput measurement, but this branch deliberately scopes
   scheduling to one machine and does not implement distributed workers. [W], [S]
4. Keep persistent C workers, typed event transport, offline packed fixtures and
   checkpoint windows as separate future improvements. The branch parallelizes
   independent fixture pairs; it does not eliminate their fresh processes or
   establish a persistent-process global reset contract. Cache eviction remains
   manual. [T8], [I], [F]

These throughput results strengthen confidence that the optimized pipeline
preserves the measured C/Rust observations. They do not establish original-ROM
timing, hardware fidelity, intended game ending or broader route accuracy. The
historical authenticator explicitly labels that distinction, and the
implementation excludes original-ROM parity. Sources: [A], [S], [I]. Keep the
separate accuracy acceptance track described in the decomp review.

## Pinned primary sources

All entries below have revision `e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`.

| ID | Git object path |
| --- | --- |
| I | `.scratch/decomp-pipeline-performance/implementation.md` |
| F | `.scratch/decomp-pipeline-performance/findings.md` |
| B | `.scratch/decomp-pipeline-performance/benchmark.py` |
| L | `.scratch/decomp-pipeline-performance/live-results.json` |
| LL | `.scratch/decomp-pipeline-performance/live-lock-results.json` |
| O | `.scratch/decomp-pipeline-performance/offline-results.json` |
| C | `.scratch/decomp-pipeline-performance/closure-results.json` |
| X | `.scratch/decomp-pipeline-performance/invalidation-results.json` |
| A | `.scratch/decomp-pipeline-performance/post-cleanup-authentication.json` |
| S | `.scratch/decomp-pipeline-performance/spec.md` |
| T2 | `.scratch/decomp-pipeline-performance/issues/02-measure-and-schedule.md` |
| T3 | `.scratch/decomp-pipeline-performance/issues/03-parallel-live-capture.md` |
| T4 | `.scratch/decomp-pipeline-performance/issues/04-keyed-build-cache.md` |
| T5 | `.scratch/decomp-pipeline-performance/issues/05-isolate-family-inputs.md` |
| T8 | `.scratch/decomp-pipeline-performance/issues/08-expand-offline-orchestration.md` |
| P | `tools/port_evidence.py` |
| PL | `tools/port_live.py` |
| W | `tools/port_pipeline.py` |

No verification run or disposable output was created by this read-only review;
run cleanup has no new run scope. This note owns only its Markdown file.
