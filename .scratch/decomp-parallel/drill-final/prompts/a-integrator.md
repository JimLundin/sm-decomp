Resume actual session A only in the isolated fixture. Use absolute
/tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill. Your original
integrator token drill-integrator generation1 remains yours; dispatcher drill-a
generation2 belongs to B. Never set session identities or model overrides.
After reading exact submitted A/B claims and observations, run next-token
drill-integrator 1 once; require integrate-lane-a. Launch actual independent
integration child with launch integration-worker using absolute prompt
/home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/prompts/integration-worker.md.
Wait integration-worker-alive 50. Run start integrate-lane-a integration-worker
drill-integrator 1 then resume integrate-lane-a drill-integrator 1. Run another
next-token drill-integrator 1 and require no second integration reservation.
Mark integration-held-ready. Remain alive via wait interrupt-integrator 50
retries, at most30; controller interrupts you while child remains alive.
Do not merge source, signal child or mutate production.
