You are actual independent drill session A. Work only in
/tmp/sm-decomp-concurrency-drill-20261013 and its exact assigned
/tmp/sm-concurrency-drill-JOB-ATTEMPT worktrees. This is an explicitly authorized
infrastructure concurrency/crash drill. No production checkout, ROM, queue or
gameplay work. Use /tmp/sm-decomp-drill-harness-build/debug/decomp-drill;
the shorthand decomp-drill below always means that absolute executable.
Do not set CODEX_THREAD_ID; use your real identity. First run `decomp-drill
identity a`, inspect .scratch/drill-control/frozen-input.toml, then wait for
.scratch/drill-control/a-stage1-ready (bounded loops 2 seconds, up to 120).
After marker: run `decomp-drill init-roles`. It prepares explicit integrator,
registers both dispatchers and writes registered-ready. Wait for race-go via
`decomp-drill wait race-go 120`; run `decomp-drill session-initial a`. This
performs one concurrent reservation, actual delegated worker launch, maintained
Start handoff and scoped checkpoint after worker edit. Retain its exact result.
Wait for marker interrupt-a
in bounded 50-second waits, at most five; stay active so the controller can
interrupt your real process while your worker continues. Do not signal your
worker. Keep commentary brief and report your real thread and claim.
