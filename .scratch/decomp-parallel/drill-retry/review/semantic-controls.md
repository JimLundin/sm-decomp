# Independent semantic control review

Observer: `/root/drill_review/eleven_controls`
Status: all eleven typed controls supported; stronger post-publication interruption requirement identified

The secondary reviewer independently read command/request/stdout/process
lineage and exact original final execution/concurrency seals. No artifacts,
fixture, Git/index or production state were mutated.

`R` below means fixture `.scratch/drill-control/receipts/`;
`C` means fixture `.scratch/drill-control/`.

| Control | Actual supporting evidence |
| --- | --- |
| independent-sessions | A/B actual launch JSONL `thread.started` UUIDs, distinct PID/start, initial-next-a-503578-1791892020279261709 and initial-next-b-503081-1791892012333937702 return distinct disjoint lane/worktree claims |
| writable-overlap | `C/scope-overlap.toml` targets held A document; final `R/ticket-disjoint-next-508541-1791892503632669284.stdout` rejects writable ownership without assignment |
| duplicate-ticket | `C/scope-ticket-disjoint.toml` candidate path is disjoint from held B, exact same ticket; holder observation plus final zero-slot stdout rejects `ticket is claimed` independently |
| shared-capacity | `R/next-drill-b-503081-1791892160240552581` retains A/B running plus C reserved and assigns no fourth job; budget snapshot seals four distinct live holders plus fifth waiter and five successful finishes |
| surviving-child | `C/surviving-child.toml` retains stopped A controller and original live A UUID/PID503608/start2835204/attempt1 |
| stopped-transfer | Both `C/drill-a-generation-1-transfer.toml` and `drill-integrator-generation-1-transfer.toml` account for stopped parent, exact live child/claim; integrator also accounts for unlaunched C attempt2 |
| crash-restart | `R/resume-505887-1791892306933452562` reviews adopted A1 WIP and exact cwd/argv; integration adoption/resume retains same UUID/attempt1/source |
| stale-authority | Old dispatcher and integrator next/resume stderr rejects stale owner/generation; stopped-worker record at1791892340 precedes replacement attempt2 launch1791892346 and stale attempt rejection |
| same-attempt-retry | A/B finish/exact-retry pairs have identical request/stdout seals; both final completed integration retries repeat original exact requests under current B generation2 |
| integration-success | A admitted at source0812fbc then B at merged2b1400e; each exact locked/offline gate runs one passing test and authentication retains submission lineage |
| cleanup | Both integration apply/deduplicate command seals and zero-byte receipts retain scoped source; only fixture main Git worktree remains |

The final evidence covers all eleven typed names. Actual independent sessions
were concurrently live; reservation calls were several seconds apart, so this
proves distinct concurrent sessions and serialized authority operations rather
than overlapping reservation-call execution. Fresh submission finish/retry
pairs do not themselves establish an intervening controller interruption after
publication. Ticket 04 Work requires that stronger step; a fresh explicit
post-publication interrupt and exact replay must supplement this evidence.
The 18-file post-B map covers maintained publications/authentication/gates;
cleanup and retained source seals are separately authenticated. Isolated
infrastructure acceptance supplies no gameplay or production acceptance.
