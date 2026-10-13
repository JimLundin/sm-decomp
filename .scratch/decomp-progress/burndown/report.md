# Full native coverage burndown

The maintained report now answers how much native source remains, how quickly
committed coverage is arriving, and whether throughput has changed over equal
calendar-time windows. The existing route is `http://localhost:4185/index.html`,
opened successfully in the Windows browser after final verification.

The default measure is native C body lines. Definitions and explicit source-bank
groups are selectable. Both graphs and all four headline readings use the same
selected scope and weighting. Warnings, report-import activity, evidence contexts
and the exact routine explorer remain collapsed and available on demand.

| Latest audited snapshot | Value |
| --- | ---: |
| Fixed native denominator | 6,053 definitions / 75,229 body lines |
| Source mapped to Rust | 1,461 definitions / 16,527 body lines |
| Remaining native coverage | 4,592 definitions / 58,702 body lines |
| Retained source checkpoints | 108 over about 2.6 days |
| Net additions in last 24 hours | 253 definitions / 3,871 body lines |
| Net additions in prior 24 hours | 286 definitions / 2,111 body lines |
| Body-line throughput change | +83.4% |
| Straight-line body-line scenario | About 15.2 days, if that rate continues |

The equal-window definition figure changes as older source arrivals leave the
window. The source curves now advance
with successful served refreshes, including zero-output time. Offline and opened
JSON snapshots retain their recorded end time. The projection is conditional,
not a delivery date, and the visible short-history qualifier stays with it.

`sm-pipeline progress history --root /home/jim/sm-decomp` reconstructs committed
body correspondence over retained first-parent Git snapshots. It uses reviewed
selectors/helper mappings and the fixed pinned C inventory. The current source
union exactly equals the reviewed current-checkout translated reports before
publication. Multiple selectors and forwarding adapters add no duplicate native
coverage. Removals count as negative output and raise remaining coverage.
Uncommitted/unmerged work gets no velocity event. Historical helper-family
arrival dates can be approximate where a dispatcher grew.

The graph measures source arriving on the retained branch, including pauses,
verification and tooling time. Active effort and source complexity are not
recorded. Report imports and source re-audits add no implementation-velocity
events. Verification, integration and bounded live exercise remain independent
and include their exact original evidence contexts.

The verification gap is a deduplicated set difference: translated source without
any valid verified observation, alongside missing checkout verification reports.
Historical verification does not authenticate later source edits. Source-group
rows show remaining work, net output and that evidence gap; clicking a row filters
the burndown and velocity to its explicit source banks.

Three retained successful pipeline samples are read from their original metrics
headers and sealed into the report with SHA-256 evidence references. The report
displays their domains and elapsed time, suppresses changed/unavailable samples,
and discloses the missing detailed stage timings. It recommends comparing build,
wait, comparison and publication time across similar changes before deciding
which bottleneck needs improvement. It does not infer causality from coverage.

The three-layout prototype is retained at
`throwaway/progress-burndown-layouts-20261013`, commit
`6912d24a234097ea62ac7ca955b3f222707c5605`. The selected paired-trend layout keeps
burndown and changing pace visible together; the work-ledger layout demoted
velocity, and the large decision-led layout demoted burndown. This is an inspected
design choice based on the user's clarified objective, not a claimed user vote.
The losing variants and floating selector are absent from maintained source.

Reporting source commit: `263ad5acb096e66ae5d272c6f94534ae50c4b963`.
[Verification](verification.json), [source freeze](source-freeze.sha256),
[served assertions](served-final/checks.json), [synthetic edge cases](history-final/checks.json)
and [cleanup](cleanup/result.md) retain the final checks and measured removals.
All 22 focused Rust reporting tests passed, including an actual temporary Git
history, and final browser checks passed for desktop, phone, themes, keyboard,
hover, live refresh, JSON loading and preserved notices/corrections/hashes.
No decomp implementation, new gameplay fixture/replay, assignment publication or
integration acceptance was performed.
