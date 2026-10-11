# Run independent live subjects with private stores

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Type: task

Extract subject execution into one callable operation returning immutable inventory,
progress and ownership results. Run up to four independent subjects concurrently,
each in its own output directory and private packed store. Preserve four fresh
producer processes and the existing full exact comparisons. Apply the shared
resource budget across worktrees. Keep sequential operation selectable.

Retain private store dependencies with their pointers initially. Existing pointers
already support a relative store per subject. Shared deduplication is ticket 07;
do not block concurrency on that larger format/publication change.

## Acceptance

- Serial and concurrent 1,000-frame and representative 9,000/45,000-frame runs
  match exact inventories, progress, counters, nonzero/exclusive execution gates.
- The four provenance/subject seals remain distinct and authenticated.
- Failure in one worker reaps its processes, preserves staging and publishes no
  successful final verdict; successful sibling evidence remains retained.
- Measure total elapsed time, CPU, peak disk usage and private-store bytes.
- Authentication passes after scoped cleanup.

## Completion evidence

Four fresh packed producers capture concurrently into independently owned private stores. Serial/parallel 1000, 9000 and 45000 complete inventories and roots agree, with byte-identical long progress/counters. Representative capture timings are recorded in implementation.md. Final optimized 1000 run passes integration and reauthentication.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
