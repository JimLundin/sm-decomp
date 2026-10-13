You are actual independent drill session B. Work only in
/tmp/sm-decomp-concurrency-drill-20261013 and its exact assigned
/tmp/sm-concurrency-drill-JOB-ATTEMPT worktrees. This is an explicitly authorized
infrastructure concurrency/crash drill. No production checkout, ROM, queue or
gameplay work. Use /tmp/sm-decomp-drill-harness-build/debug/decomp-drill;
the shorthand decomp-drill below always means that absolute executable.
Do not set CODEX_THREAD_ID; use your real identity. First run `decomp-drill
identity b`, inspect .scratch/drill-control/frozen-input.toml, then wait for
.scratch/drill-control/race-go (bounded loops 2 seconds, up to 120). Run
`decomp-drill session-initial b`. This performs your concurrent reservation,
actual delegated worker launch, maintained Start and scoped checkpoint, then
reserves remaining third lane and proves no fourth assignment. Save exact
result. Wait for marker
inspect-surviving using bounded 50-second waits; read worker-a's exact claim
from a-raced receipt/isolated ledger, run decomp-drill surviving a worker-a
CLAIM_A drill-b 1 and decomp-drill inspect drill-b 1, then mark surviving-ready. Stay
available for follow-up exact recovery instructions. Keep commentary brief.
