# Show the latest ten progress submissions

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: /root, real session 01a13592-d02b-7143-8489-a15b03f19da2

## Work

User requested an additive table at the top of the existing progress report.
Show the ten latest progress submissions, newest first, with publication time,
scope, stage and what each added. Preserve the current burndown, velocity,
counting, evidence explorer, source history and existing registry observations.
Read the existing registry updates and refresh with the normal live/file picker
workflow. Publication time is a report time, not a fabricated implementation date.

Maintained source ownership is only crates/pipeline/src/progress.html.in.
Build and inspect in an isolated reporting checkout while the original host
verification batch runs against frozen main source. Parent owns integration,
viewer publication, local tracker and main Git/index. Reuse the Rust pipeline
and evidence crate; do not add a second hash or storage implementation.

## Acceptance

The default report immediately shows up to ten submissions in descending
publication order, including updates that add evidence without adding unique
coverage. Display precise time and concrete scope/routine/evidence descriptions.
Fewer or zero submissions render correctly; equal timestamps preserve newest
append order. Live refresh and selected JSON replace the table. Stale and
superseded observations remain explicitly labeled. Render data as text and
preserve existing report behavior and mobile layout.

## Completion evidence

Integrated at `52f44a1a7599f540397dfbd24a31ff8b3ef5f19e` from isolated source
commit `dd4d37d7b76d66bf107fa5d76bfce65457d48c0b`. The only maintained source
change is additive markup/style/rendering in `crates/pipeline/src/progress.html.in`.
The existing charts, counting, schema and registry history remain intact.

The table is published in `.scratch/decomp-progress/index.html` through the
Rust pipeline's normal progress update command and is served at
http://127.0.0.1:4185/. It reads the ten latest observations, orders equal times
by newest append, displays exact UTC seconds and shows first stage additions,
routine/evidence counts, descriptions, owner, commit and evidence references.
Evidence-only, stale and superseded submissions remain visible with clear labels.

All 20 existing focused Rust progress tests passed (`progress-tests.stdout.txt`).
The successful final served desktop/phone browser checks are
`latest-submissions/browser-refresh-polled/verification.json`: sorting, UTC,
content, zero additions, top placement, retained report, mobile layout, live
refresh, empty/single JSON, unordered inputs, tied timestamps, literal HTML
and stale labels. `live-refresh.json` confirms automatic prepend and restoration
of the real registry; no synthetic observation was published. Earlier isolated
dependency/preview-start and short-wait browser failures are retained as diagnostics.

Both scoped Rust pipeline cleanup commands succeeded, reclaiming0 eligible
bytes; deduplication checked 0 snapshots. The identified unsealed reporting targets
and browser profiles were removed: 2,098,678,222 logical file bytes. Observed
remaining space after removal was 172,619,423,744 bytes. See
`latest-submissions/disposable-cleanup.md` and `cleanup{1,2}.stdout.toml`.
The clean integrated reporting worktree was removed with `git worktree remove`;
its exact template remains at `latest-submissions/source/progress.html.in` and
matches main. The intended viewer server PID803313 and its binary are retained.

The existing evidence crate ingested the 99-file, 31,048,909-byte frozen reporting
package after cleanup into shared `out/evidence`, trusted run
`62659567eae8a095738b84324898e629012c0e6871875bb2bab7b257231b7ccd`.
Its authenticated inventory and post-cleanup verify both passed; verify compared
all retained package bytes and membership with the original directory. Receipts:
`latest-submissions/evidence-{ingest,inventory,verify}.stdout.toml`.
Retain shared store packs, `latest-submissions/artifacts`, source and exact reader
`latest-submissions/bin/sm-evidence`. No manual hash/storage path was added and
no gameplay stage was earned by this reporting-only change.

## Comments

2026-10-13: completed host verification was published through Rust progress
update as shared-game-host-current-verified-launch-20261013. The served registry
has 262 observations and authentic current-source evidence for the five already
counted host routines. Research15 and research17 are research results and do
not earn gameplay stages. Subsequent achieved stages must be published before
handoff through the maintained progress command.

2026-10-13T15:41Z: latest-ten table is visible in the running tracker and all
verification, publication and mandatory cleanup are complete. Each achieved
gameplay stage is published through Rust progress update before handoff; research
answers and this infrastructure result are recorded in their local tickets.
