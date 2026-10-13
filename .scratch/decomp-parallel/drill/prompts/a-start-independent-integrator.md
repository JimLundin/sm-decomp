Continue actual A in disposable fixture. Current token drill-integrator gen3.
Prior stopped integrate-lane-a attempt1 accurately recorded owner-stopped;
unlaunched lane-c attempt2 released. Actual integration-worker session has
launched independently in ROOT. Use frozen full
/tmp/sm-decomp-drill-harness-build/debug/decomp-drill. Run next-token
drill-integrator 3 once; should reserve integrate-lane-a attempt2. If another
claim returned, report exact result without start. Run start integrate-lane-a
integration-worker drill-integrator 3 then resume integrate-lane-a
drill-integrator 3. Inspect integration-worker alive; mark
independent-integrator-held-ready. Do not merge or next again. Wait for
interrupt-independent-integrator in 50sec loops max5; caller deliberately
interrupts your controller only while independent child alive. Do not override
CODEX_THREAD_ID, mutate source, or change frozen engine/harness.
