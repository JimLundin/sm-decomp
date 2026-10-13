You are actual independent drill session B. Work only in the isolated fixture
/tmp/sm-decomp-concurrency-drill-final-20261013 and its exact returned
/tmp/sm-concurrency-drill-final-JOB-ATTEMPT worktrees. This is an authorized
preparation concurrency/crash validation. Always use
/tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill; shorthand below
means that absolute executable. Never set CODEX_THREAD_ID or override models.

Run identity b and read frozen-input.toml. Wait race-go in bounded waits 50,
retry marker timeouts. Run session-initial b to race reservation, launch an
actual child that dispatches a real subagent, maintain Start and checkpoint,
reserve third lane C and prove capacity3 defers a fourth. Mark b-ready.
Wait inspect-surviving 50, retry until marker. Read .scratch/drill-control/a-claim-id
as CLAIM_A and b-claim-id as CLAIM_B. Read their assignment and worker-a launch
files; run surviving a worker-a CLAIM_A drill-b 1, then inspect
drill-b 1 (use separated args). Mark surviving-ready.

Wait recover-go 50, retry until marker. Run transfer drill-a a 1 b. Run resume
CLAIM_A drill-a 2 reject (separate args) to require unadopted rejection. Adopt
CLAIM_A drill-a 2 using absolute
/tmp/sm-decomp-concurrency-drill-final-20261013/.scratch/drill-control/drill-a-generation-1-transfer.toml.
Run resume CLAIM_A drill-a 2 successfully. This must retain the actual still-live
worker and exact WIP claim. Mark recovery-adopted-ready. Wait worker-a-stopped 50.
Inspect original worker-a stopped; run stop-owner CLAIM_A worker-a drill-a 2,
release-reserved lane-c drill-b 1, next-token drill-a 2 once. Require CLAIM_A
attempt2 with new returned worktree; dispatch CLAIM_A worker-a-replacement
drill-a 2. Run stale-attempt CLAIM_A worker-a drill-a 2, require rejection. Wait
worker-a-replacement-edited 50 then checkpoint CLAIM_A drill-a 2. Mark replacement-ready.

Finish CLAIM_A drill-a 2 then retry exact finish once. Finish CLAIM_B drill-b 1
then retry its exact update once. Mark submissions-ready. Run control-scopes
then absolute /tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill-evidence
pure-next drill-b 1 overlap to observe
writable ownership deferral with zero slots and no new claim. Run absolute
/tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill-evidence prepare-ticket
then the same helper pure-next drill-b 1 to require a pure disjoint duplicate
ticket deferral. No new assignment may be dispatched from those controls.

Launch five actual native subprocesses concurrently from your real session
using decomp-drill lease lease-1 through lease-5. Redirect each stdout to its
own .scratch/drill-control/lease-N.log. Wait until exactly four acquired
receipts and five started receipts exist, then budget-snapshot and mark
leases-held-ready. Wait release-leases 50, collect all five processes, run
budget-finish and mark controls-ready. Stay available through bounded waits
for b-next 50. Do not integrate or modify shared Git before explicit next prompt.
Unexpected failures: preserve diagnostics and stop dependent mutation.
