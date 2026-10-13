Resume your actual independent drill session B. Work only in the isolated
fixture repository and its returned worktrees. Use the durable ledger and
.scratch/drill-control launch/assignment/handoff files, never chat guesses.
Use /tmp/sm-decomp-drill-harness-build/debug/decomp-drill; never set session id
environment variables. A's actual parent process is observed stopped and its
dedicated worker survived. Inspect surviving-child.toml and actual process
observations before recovery. Read the exact A claim id from the running
ledger entry whose stored agent matches worker-a.toml.

1. Run `decomp-drill transfer drill-a a 1 b`. It seals observed stopped parent
   and every actual held child and transfers only role generation to 2. Run
   `decomp-drill resume CLAIM_A drill-a 2 reject` to prove an unadopted handoff
   rejects. Run `decomp-drill adopt CLAIM_A drill-a 2
   .scratch/drill-control/drill-a-generation-1-transfer.toml`; then run
   `decomp-drill resume CLAIM_A drill-a 2`. Confirm reviewed-resume includes
   the exact existing worktree/WIP commit and command.
2. Write recovery-adopted-ready. Wait for worker-a-stopped marker in bounded
   50-second waits. Inspect actual worker-a process. Run `decomp-drill stop-owner
   CLAIM_A worker-a drill-a 2` only after its exact process is stopped. Preserve
   old scoped WIP/source evidence first. Release the third lane reservation
   you made but never launched through `release-reserved CLAIM_C drill-b 1`;
   its real launch is known absent. Run `decomp-drill next-token drill-a 2`
   once. Read newly returned assignment, require same lane with attempt 2,
   and preserve both attempt receipts.
3. Run `decomp-drill dispatch CLAIM_A worker-a-replacement drill-a 2`. The
   new worktree/attempt is returned by the queue; recover prior scoped WIP
   commit by cherry-pick before replacement edits only if needed. Wait for
   replacement's edit marker, run maintained checkpoint. The old attempt's
   stopped/result update must reject after attempt2 admission. Write
   replacement-ready. Run `decomp-drill finish CLAIM_A drill-a 2`, then retry
   the exact published finish update; require same claim/update and no rerun.
   Retain full source and run/receipt paths.
4. Finish your original B worker's exact clean scoped commit through
   `decomp-drill finish CLAIM_B drill-b 1`. Retry exact update. Write
   submissions-ready. Remain available; do not integrate or touch shared Git
   until the integrator generation transfer and exact recovery handoff are
   reviewed by the controller.

If a step rejects unexpectedly, preserve its durable diagnostic and stop its
dependent mutation. Report the exact command and error briefly. These are
infrastructure receipts only and grant no gameplay launch or production cutover.
