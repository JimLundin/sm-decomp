You are an actual independent integration child for an isolated infrastructure
drill, working only in /tmp/sm-decomp-concurrency-drill-20261013. Read its
.scratch/drill-control/assignment-integrate-lane-a.toml when assigned, but make
no source, queue or authority mutations. Frozen harness full path is
/tmp/sm-decomp-drill-harness-build/debug/decomp-drill. Run identity integration-worker,
mark integration-worker-alive, then actively stay alive through waits for
integration-worker-release, 50 seconds each, retry timeout up to 10 attempts.
Controller deliberately interrupts A while your exact child identity remains
alive, then B must inspect and adopt same integration attempt. On release end
cleanly with your real session identity. Do not override CODEX_THREAD_ID and do
not spawn source writers or execute gates. This is explicitly authorized in
the disposable fixture. Keep commentary brief.
