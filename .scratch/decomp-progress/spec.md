# Shared decompilation progress inventory

Status: resolved

The user requested subsystem-sized parallel translation and a continuously
updated inventory of all recovered code, displayed through interactive HTML.
This resumes session `01a1319d-9382-7a82-9a1e-1ea4fc26f6b1`, whose coordinator
failed on an encrypted tool-result decoding error. Its original gameplay agents
continue to own their worktrees.

Required tickets: [01](issues/01-registry-and-dashboard.md) and
[02](issues/02-report-layout.md).

The maintained source inventory supplies definition identity, bank, original
address, body extent and direct dependencies. A Rust command publishes concurrent,
atomic stage observations in JSON and regenerates a standalone HTML dashboard.
The HTML also reads refreshed JSON when served locally or selected from disk.
Fixture declarations, translation, verification, integration and live exercise
remain distinguishable. Historical evidence and a stage report do not establish
full-game completion or original-ROM parity.

## Completion evidence

Resolved locally on integration branch `affected-family-assignment`; required
ticket [01](issues/01-registry-and-dashboard.md) is resolved. Maintained Rust
refresh/update/status/serve commands and the interactive standalone dashboard
publish the complete pinned 6,053-definition inventory with independent hashed
stage observations. The current registry has 28 observations, four historical,
including current translating220/translated247/verified34/integrated34 native
definitions. Live-exercised remains zero while independent gameplay gates finish.

Fresh real CLI refresh, fourteen focused progress tests, browser desktop/phone
behavior, six real concurrent writers with 421 complete atomic reads, idempotent
retries, source/evidence validation and scoped cleanup pass. The four-megabyte
declaration regression improved from 49.324 seconds to 58.8 milliseconds; release
current status takes 0.16 seconds and six full-registry writers take 2.629 seconds.
Exact source/artifact digests and remaining gameplay frontiers are retained in
[the checkpoint](recovery-refresh-checkpoint-20261013.md),
`recovery-current-publication.sha256`, and the ticket's completion evidence.
The coordinator owns the later shared wiring commit and new live observations.

Final reconciliation supersedes the above historical work-status snapshot.
Viewer/server is committed at `b8c3ffc`, gameplay wiring at `3ce28ba`, and
evidence contexts plus permanent reviewed corrections at `66cde48`.
Tickets 01 and 02 are resolved. The final 256-observation registry reports
translated 1,460 / verified 924 / integrated 38 / live-exercised 121, preserving
16 stale records and one superseded record excluded from counts. All maintained
owned mappings and substantiated retained fixture/live coverage are included;
`../decomp-strategy/coverage-audit.md` records exact domains and remaining gaps.
Nineteen focused Rust tests and final desktop/phone served/offline checks pass.
The three-layout prototype comparison is retained on throwaway branch
`throwaway/progress-layout-comparison-20261013` at `e574fef4`. Scoped cleanup and
disposable target removal are complete, with measurements in ticket 02.
