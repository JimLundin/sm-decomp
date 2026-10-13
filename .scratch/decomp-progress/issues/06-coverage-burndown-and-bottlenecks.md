# Coverage burndown, changing velocity and pipeline bottlenecks

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: reporting session /root; main Git/index owner

## Required work

The user rejects the routine-counter representation and clarifies the question:
how fast are we reaching full native decomp coverage, what is the burndown, and
is changing velocity a reason to pause and improve a pipeline bottleneck?

Reconstruct a coverage history from retained first-parent source snapshots,
using the current pinned native denominator and reviewed source mappings.
Keep audit/report-import dates out of implementation velocity. Show remaining
coverage, net calendar-time velocity, equal-period comparison and a conditional
projection. Retain exact definition/line weighting, known evidence gaps, source
bank breakdown and the existing evidence explorer. Do not infer active effort,
gameplay completion, current verification or causal bottlenecks from counts.

Inspect retained pipeline timing only. Do not implement gameplay or run new
gameplay fixtures, replays, integration gates or acceptance checks.

Build three distinct layouts with the prototype skill on the existing route,
retain the throwaway variants on a throwaway branch, and implement the design
that answers the clarified burndown question. Open the maintained result in
the user's Windows browser. Verify reporting calculations and browser behavior,
preserve original observations/evidence, complete scoped cleanup and checkpoint
only reporting-owned files.

## Comments

2026-10-13: user explicitly prioritizes coverage burndown and the change in
velocity over gameplay-milestone framing. History spans about three days;
report imports span about one day and are not engineering output timestamps.
Use committed source snapshots with disclosed reconstruction limits.

The parallel preparation session now owns its isolated assignment/session
tools worktree and `.scratch/decomp-parallel/{preparation,reconciliation,issues}`
documents. Reporting owns `progress.rs`, `progress/*`, `progress.html.in`,
new reviewed reporting inputs, and `.scratch/decomp-progress/`. Do not stage
the preparation session's files. Gameplay remains stopped.

## Completion evidence

The maintained report is now a compact paired burndown/velocity view with a fixed
native denominator, body-line/definition weighting, complete equal 6/12/24-hour
windows, previous-window comparison, conditional straight-line coverage scenario,
independent verification gaps, explicit source-bank groups and retained hashed
pipeline timings. Supporting warnings, original report activity and routine
evidence remain collapsed. Source-import dates contribute no velocity events.
Successful served checks advance calendar windows even while source is paused;
opened/offline snapshots keep recorded time.

Rust `progress history` reconstructs 108 retained first-parent checkpoints and
exactly validates the current reviewed mapped set: 1,461 native definitions /
16,527 body lines, zero undated uncommitted coverage. Remaining: 4,592 definitions
/ 58,702 body lines. The snapshot's last 24 hours add 253 definitions / 3,871
lines; prior 24 hours add 286 definitions / 2,111 lines. Forecasts are visibly
conditional and limited by roughly 2.6 days of history. Source complexity,
active effort, gameplay completeness and causal bottlenecks are not inferred.

Scoped source commit: `263ad5acb096e66ae5d272c6f94534ae50c4b963`.
Three structurally different read-only layouts are captured on
`throwaway/progress-burndown-layouts-20261013`, commit
`6912d24a234097ea62ac7ca955b3f222707c5605`, including their switcher, standalone
real-data HTML and desktop/phone images. Layout A was selected through visual
inspection against the user's explicit objective because both trends are visible
together. This is not a claimed user vote. Losing variants and selector are
absent from main. No additional user approval was required for the reversible
authorized reporting work.

All 22 focused Rust reporting tests pass. A temporary Git-history test covers
real reconstruction, deduplication, removals, unmerged work, backwards commit
time and stale nested-source/mapping/inventory detection. Final browser checks
pass for real weighted windows, every bank, source groups, keyboard/hover,
independent stage/context counts, 165 original notices, correction and hashes,
live refresh, advancing pause windows, older JSON clearing, themes and phone
layout with no horizontal overflow. Six synthetic edge cases pass. The stored
registry, served JSON and served embedded snapshot agree. Windows browser
launch succeeded for `http://localhost:4185/index.html` after final checks.
Evidence: `burndown/report.md`, `verification.json`, `source-freeze.sha256`,
`served-final/checks.json`, `history-final/checks.json` and
`file-final/verification.json`.

Cleanup succeeded before resolution: new profile/upload removals reclaimed
86,364,265 apparent bytes; new disposable outputs reclaimed 166,203,256 bytes.
Both maintained ordinary and dedup passes reported zero remaining supported
targets. The prototype worktree was removed after branch capture. Free disk:
221,569,400,832 bytes. Exact paths/measurements are in
`burndown/cleanup/result.md`, `browser-profiles.json` and `disposable-outputs.md`.
All 49 raw review/audit receipt files retain exact bytes in the hash-verified
archive; original inputs/archives, sealed evidence, readers and shared stores
remain unchanged. The intended report server is retained. No gameplay source,
new fixture/replay or acceptance gate was executed or changed.
