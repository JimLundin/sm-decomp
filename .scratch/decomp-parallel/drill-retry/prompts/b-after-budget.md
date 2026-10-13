Resume the same actual B session in the isolated retry fixture only.
The controller sealed a valid worker-budget-snapshot while five actual processes
were alive (four leases and one waiter), then marked release-leases. All five
released, controller budget-finish passed and worker-budget.toml authenticates
snapshot/finished receipts. Your later repeated budget-snapshot rejected because
the leases had already stopped; retain that diagnostic as a rejected repeat.
Do not rerun leases or pure controls. Use unchanged inherited CODEX_THREAD_ID/model.
Use /tmp/sm-decomp-drill-retry-harness-build/debug/decomp-drill.
Read worker-budget.toml, mark controls-ready, then read immutable
/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/prompts/b-integrator-recovery.md
and execute that integration recovery phase in this same real session.
A controller is observed stopped, same integration-worker PID509273 remains live;
phase-integration-adopt accounts held claims and checks exact start identity.
