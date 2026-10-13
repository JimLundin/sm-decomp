Resume your actual A session in the disposable fixture only. Frozen engine and
harness remain unchanged. Actual B controller PID435917 was interrupted while
integrate-lane-a attempt1 and unlaunched lane-c attempt2 are held by integrator
token generation2. Preserve source fixture HEAD at baseline until adoption.
Use full /tmp/sm-decomp-drill-harness-build/debug/decomp-drill commands.
Run transfer drill-integrator b 2 a, which must account both held children.
Run resume integrate-lane-a drill-integrator 3 reject, requiring adoption.
Then adopt integrate-lane-a drill-integrator 3 with absolute transfer artifact
/tmp/sm-decomp-concurrency-drill-20261013/.scratch/drill-control/drill-integrator-generation-2-transfer.toml.
Run resume integrate-lane-a drill-integrator 3 successfully. Inspect both owned
submitted worker processes via observe worker-a-replacement and observe worker-b.
Do not merge or release lane-c yet. Mark integrator-adopted-ready, then wait for
integrator-finish-go in 50 second bounded loops up to 5 attempts. Caller needs
one pure deferred duplicate-ticket observation and old B authority rejection
before cleanup and maintained gates. Do not set CODEX_THREAD_ID.
