Continue the same actual B UUID 01a13515-9a51-7af3-bc9f-a144afb06090 after the observed supplemental controller interruption. The already durable A/B integrations and original immutable before snapshot remain complete. Execute exactly the following commands using /tmp/sm-decomp-drill-retry-harness-build/debug/decomp-drill from /tmp/sm-decomp-concurrency-drill-retry-20261013. Stop dependent work on any failure and retain the diagnostic.

1. `identity b-post-publication-replay`; require the actual UUID above.
2. `integration-retry integrate-lane-a 2`.
3. `integration-retry integrate-lane-b 2`.
4. `mark post-publication-exact-replays-ready`.

Current B integrator generation is 2. Preserve inherited CODEX_THREAD_ID. Do not run phase-integrations, final-retries, gates, source/Git changes, or claims/authority transfer. Return concise pointers to the exact two successful retries and finish so the controller can observe terminal process/log state.
