# Observed frozen integration limitations

Executed engine commit `c8799f326ffe198b652301f08866f1b7a7bd9ebc` and binary
SHA-256 `89e3c46596ea556011cfcbb679cace5b96277bbf56abb8a7a2a61db7048268d9`
were unchanged throughout the actual drill.

The first interrupted integration attempt used its controller as its worker.
After B stopped and A acquired generation 3, unadopted resume correctly required
adoption, and adoption rejected `adopt requires an observed surviving child`.
The transfer truthfully recorded that worker completed. A recorded the exact
checked worker stop, retained both submitted claims, and reserved attempt 2.

Attempt 2 used an independently launched actual integration child
`01a134bf-913f-7270-ba31-e1b7c3280eea`, PID 441189, while A owned generation 3.
Only A stopped. B transferred generation 3 to 4, observed that exact child live,
adopted and resumed attempt 2 at the original fixture source and worktree. The
child then completed after release. A's old generation 3 resume rejected stale
session authority. B integrated lane A under generation 4, with a real passing
Rust test gate, source snapshots, utility cleanup, removed A worktrees, and
maintained infrastructure authentication. This is the successful recovery path.

The next sequential integration was reserved for lane B. Its actual `start`
rejected `change exceeds approved files: docs/drill-a.md`: session handoff
validation compares the original baseline with the current integration HEAD,
so lane A's already merged path exceeds lane B's writable scope. No scope was
expanded, result admitted, or source altered to bypass that check. Lane B's
submitted source and claim remain retained. This is an observed engine limitation
requiring a separate behavioral fix and a fresh actual drill before claiming
general sequential integration readiness.

Exact diagnostics are retained under the isolated fixture's
`.scratch/drill-control/a-adopt-integrate-lane-a-generation-3.exact.log` and
`b-start-integrate-lane-b-gen4.exact.log`, with sealed frozen command receipts
under `.scratch/drill-control/receipts/` and actual A/B JSONL logs under this
directory's `sessions/`.
