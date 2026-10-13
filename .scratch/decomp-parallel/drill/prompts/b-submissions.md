Resume actual B isolated same roles drill-a2,drill-b1,drill-integrator2. A original
worker stopped checked, replacement lane-a attempt2 RESERVED returned workspace;
use durable queue. Run absolute decomp-drill dispatch lane-a worker-a-replacement
drill-a2 (spaced args), which recovers retained scoped WIP to new exact worktree
via cherry-pick before new actual worker/subagent edits. Run stale-attempt lane-a
worker-a drill-a2 must reject prior claim. Do not re-reserve lane-a. Wait own
edited marker120secs then checkpoint lane-a drill-a2; if timeout preserve claim
and wait same worker again. Finish lane-a drill-a2 and retry exact result once.
Finish lane-b drill-b1 and retry exact result once. Mark submissions-ready.
Five native shared lease clients need actual caller: launch from your same
session command five decomp-drill lease lease-1 ... lease-5 processes concurrently
in isolated cwd, each stdout separate .scratch/drill-control/lease-N.log. Wait
until four acquired receipts exist, then budget-snapshot, mark leases-held-ready.
Wait release-leases marker, collect all five finish, budget-finish. Stay available
for next fenced integrator instructions. Never set session IDs/production paths.
Use absolute /tmp/sm-decomp-drill-harness-build/debug/decomp-drill everywhere.
