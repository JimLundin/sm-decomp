Continue actual A in disposable fixture only. Gen2→3 transfer succeeded but
adoption accurately rejected because the integration worker was B controller
itself, now stopped. Preserve that rejection; never spoof a surviving child.
Before B resumes, run full /tmp/sm-decomp-drill-harness-build/debug/decomp-drill
stop-owner integrate-lane-a b drill-integrator 3. This seals observed stopped
actual B identity and routes checked owner-stopped. Release only unlaunched
lane-c reservation via release-reserved lane-c drill-integrator 3. Observe
worker-a-replacement and worker-b; if still running, do not stop arbitrarily,
report and wait. Do not merge yet. Mark integrator-checked-stop-ready and wait
integrator-finish-go in bounded 50sec loops max5. No CODEX_THREAD_ID override.
