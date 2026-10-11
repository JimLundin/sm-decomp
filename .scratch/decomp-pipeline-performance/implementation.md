# Parallel local pipeline implementation

Branch: `decomp-pipeline-performance`
Worktree: `/tmp/sm-decomp-pipeline`

The new `sm-pipeline` Rust CLI owns OS-backed shared process leases, atomic
per-key compiler artifact publication and byte authentication, compiler dependency
inventories, conservative gameplay module closure generation, and C object reuse.
Python remains the adapter for existing family-specific generation and admission.
Four fresh packed subjects use private stores; shared import runs only after
state/effects/progress/ownership checks. `sm-evidence` reconstructs/remaps store
ordinals while preserving canonical identities and authenticates inventory in
one traversal. Canonical WRAM lives independently of gameplay declarations.

## Verification recorded so far

- Rust evidence: 11 integration tests passed, including ordinal-remapping import,
  idempotent reuse and corrupt/truncated object rejection.
- Rust pipeline: dependency parser and four process integration tests passed,
  including killed lease holder, concurrent equivalent checkout cache, mutation,
  abandonment, corruption and real C header/define behavior.
- Complete Rust port tests passed after canonical memory extraction.
- Clippy for pipeline/evidence/port with all targets and denied warnings passed.
- Python harnesses: family 10 (serial and four workers), collision 2 (serial and
  four workers), PLM 1, PLM instructions 3, live 16 (one preexisting skip), evidence
  7, batch 7 and pipeline 3 passed.
- Frozen decompressor acceptance regenerated for the new Cargo lock: 40 synthetic
  cases, 23 captured calls, 3 original-ROM calls and 11 rejecting controls passed.
- Initial final-batch system/audio/dispatcher/animated-tiles: 475/1147/342/43 passing cases.
  Repeated command authenticated and reused all four; pinned C object hits were
  1/1, 1/1, 47/47 and 2/2. Later stale-bootstrap hardening requires refreshing
  this source-dependent evidence; final refresh is underway.
- 1000-frame serial and parallel runs passed full integration. All 108 observation
  members, per-subject roots, progress, counters and transformations match exactly.
  About 14.4 seconds serial capture excluding queue vs 5.3 seconds parallel capture.
  Shared publication adds about 5–6 seconds; uncontrolled concurrent host load
  prevents claiming a stable end-to-end speedup.
- Cold evidence reader build: 90.27 seconds. Concurrent waiter reused the same
  artifact; later warm restoration: 0.228 seconds.

## Limits

The shared capacity is per Git common checkout (or explicit shared root); all
workers must request the same configured capacity. Cargo bootstrap itself uses
one serialized build job. Cache entries are retained independently of evidence
cleanup and have no automatic eviction. Generated C hosts always rebuild/relink.
Historical admission seals appropriately reject changed harness/tool sources.
Full live observations authenticate independently of current-source admission.
Plain/gzip live capture remains sequential. Original-ROM parity and persistent
process reset contracts are outside this pipeline change.

## Remaining verification

Finish serial/parallel 9000 and 45000 frame runs, final source-sealed offline
refresh, cleanup and reauthentication. Record exact timings and retained paths.

## Long capture measurements

Parallel 9000 capture finished in 58.6 seconds at the slowest subject, with
32–33 second producers. Parallel 45000 capture finished in 233.7 seconds at
the slowest subject; producers took 155–158 seconds. Private store retention
was about 26.7 MB per 45000 subject for 1.9–2.6 GB logical observations.
Shared import/authentication costs remain material and queue time is recorded.
A follow-up during verification changed store/worker lock order: shared-store
waiters now acquire store ownership before taking a CPU lease. This avoids
occupying the machine budget while blocked behind another importer. Retained
pre-fix long runs remain valid byte/counter tests, with their older scheduling
behavior identified in timings.

Serial/parallel 9000 final integration verdicts now both passed: identical full
member inventories, per-subject observation roots, progress, family/predecessor
counters, callback ownership and transformations. Capture excluding scheduling
wait: 144.0 seconds serial vs 58.6 seconds parallel under concurrent host load.

## Final implementation checks

Final optimized 1000-frame live run:
`out/port/pipeline-implementation/final-live-1000/20261011T141718.386372Z/verdict.json`.
It passed full integration and shared publication using the new import inventory
return and store-before-worker lock ordering. All observations, progress, counters
and transformations match the prior serial/parallel 1000 evidence; retained
reader and all four subjects reauthenticated. Final-source batch is
`out/port/pipeline-implementation/current-batch/batch.json`: system/audio/dispatcher/
animated-tiles passed and all four reused after authentication. Complete harness
suite: 52 tests passed (one preexisting skip), 250.3 seconds under concurrent load.
Final Rust evidence/pipeline tests and strict Clippy passed; focused rustfmt and
diff whitespace checks passed.

Cleanup so far: automatic batch deduplication reclaimed 606339072 bytes for
each fresh batch; scoped implementation deduplication reclaimed 1768292352 bytes.
An overlapping cleanup attempt rejected a target disappearing as a live run
finished; rerun after completion is required. Shared packs, source snapshots,
linked readers/live binaries and pointers remain retained.

All four serial/parallel 45000 subject inventories and observation roots now
match exactly. Replay stdout (full progress and ownership counters) matches
byte-for-byte per subject. Parallel final integration passed; serial final
publication/admission remains underway. The broad port format check fails on
preexisting gameplay formatting in enemy_projectile_instructions/enemy_projectiles/
room; changed Rust files pass focused formatting and unrelated gameplay files
remain untouched.

Serial 45000 capture totals (excluding scheduling wait): about 900 seconds
vs parallel 233.7 seconds under concurrent load. Exact observations and progress
are equal; end-to-end publication remains measured separately. All stores have
complete private pointers while shared import is in progress.

## Verification paths

- 1000 serial: `live-serial-1000/20261011T135955.924605Z/verdict.json`
- 1000 parallel: `live-parallel-1000/20261011T140027.949716Z/verdict.json`
- 9000 serial: `live-serial-9000/20261011T140314.717556Z/verdict.json`
- 9000 parallel: `live-parallel-9000/20261011T140215.135370Z/verdict.json`
- 45000 serial: `live-serial-45000/20261011T140347.533111Z/verdict.json`
- 45000 parallel: `live-parallel-45000/20261011T140216.374375Z/verdict.json`

All paths above are under `out/port/pipeline-implementation/`. Failed/stale
verification attempts remain retained as diagnostic evidence.

Post-cleanup checks: all four current offline families and fresh decompressor
acceptance authenticated. Both 1000 serial/parallel retained live runs and serial
9000 authenticated. The final optimized 1000 run already reauthenticated and
parallel 45000 reauthenticated with 510 members per subject.
Disposable build cleanup removed 843648190 bytes and retained exact test CLIs
under `retained-test-tools/`. Latest scoped cleanup reclaimed zero unsealed bytes;
latest deduplication reclaimed zero additional bytes. Final free space about
243 GiB. Integration worktree remains requested and unmerged; it is not a
disposable integrated implementer worktree.

The final import returns source/destination-verified artifact digests to its
adapter, eliminating two redundant restoration traversals before pointer
publication. A post-publication authenticated traversal still precedes release
of private storage. Historical retained readers continue to work unchanged.
CPU/peak RSS are not yet reported per stage; recorded measurements cover elapsed
time, scheduling/store waits, logical/private bytes, cache hits and cleanup.
Cache eviction remains manual and single-machine scheduling is the first scope.

## Final long verification

Serial 45000 completed with `mixed-C-Rust-integration`; parallel/serial final
statuses, full artifact inventories, all family/predecessor/callback/positioned
counters, progress and transformations agree exactly. Both final stores have
shared pointers, and no owned private store remains. Parallel retained long
evidence reauthenticated after cleanup; final serial reauthentication is running.

Finished-run cleanup after serial publication: `port_cleanup.py --root
out/port/pipeline-implementation --apply` reclaimed zero bytes. The shared
store is about 26 MB for retained runs; logical inventories remain exact.
Final deduplication and serial reauthentication are completing.

Final finished-run deduplication reclaimed zero snapshot bytes. Retained source
files, sealed artifacts, readers, shared packs and verdict pointers were preserved.
Previous measured cleanup reclaimed 1768292352 snapshot bytes plus 843648190
bytes of identified disposable build outputs (and automatic per-batch savings).
The dedicated worktree remains for user review and is not integrated into the
main checkout. No original gameplay working changes were modified.

Final serial 45000 post-cleanup authentication passed: 510 observation members
per subject, all four subjects, exact reader/live binary and complete shared
closure. Verdict SHA256: `77e4fa4a04c6b947ee7065a51ba37762957bb8379ce19955436f80cb3e1796f3`.
Both long verdicts and all short/medium verdicts reauthenticated. Remaining
free bytes at final authentication: 256302972928. Implementation complete on
`decomp-pipeline-performance`; no merge into the active main checkout.
