Resume your actual session B in the isolated infrastructure drill. Use only
durable exact source, receipts and typed authority/handoffs. Never set
CODEX_THREAD_ID. Use the Rust decomp-drill binary. A's integrator owner process
is stopped; every held submission/integration worker remains accounted for.

Run `decomp-drill transfer drill-integrator a 1 b`, observe generation2 for
your own real thread. Retry this exact transfer if its response was lost.
Read held submitted claims and require their writable ownership preserved.
Run `decomp-drill next-token drill-integrator 2` once, save exact integration
claim. Publish `decomp-drill start INTEGRATION_CLAIM b drill-integrator 2`;
this stores your actual identity on the integration worker and durable typed
handoff. Run exact `resume` and verify reviewed command/worktree/full source.
Run a second next-token while first integration remains active; require no
second concurrent integration. Write integration-held-ready.

Wait for integrate-go marker in bounded waits. Run `decomp-drill integrate
INTEGRATION_CLAIM 2`. The helper merges only exact scoped source if absent,
retains source snapshots, performs both actual cleanup passes, removes
integrated worktree, and invokes maintained admission gates and authentication
before durable integrated completion. Save its exact result/authentication
and cleanup receipts. Reserve and complete the other held submission similarly,
only one integration at a time, with each retained source preserved. Write
integration-complete-ready. Preserve all failed or passing receipts and stop
dependent mutation on unexpected failure. No production gameplay/cutover.
