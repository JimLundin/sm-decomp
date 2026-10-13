Resume your exact original actual session A after its controller interruption.
This is the isolated Rust concurrency drill only. The other actual session B
has checked your stopped parent and every held child, then transferred your
drill-a dispatcher and drill-integrator role tokens to the next generation.
Use your actual CODEX_THREAD_ID; do not set or impersonate a session identity.
Use /tmp/sm-decomp-drill-harness-build/debug/decomp-drill in the isolated
fixture checkout. Inspect durable authority/transfer receipts read-only.
Execute old generation1 mutations as negative controls: run
`decomp-drill next-token drill-a 1`; it must fail with stale session authority.
Run `decomp-drill next-token drill-integrator 1`; it must reject too. Preserve
the emitted nonzero command diagnostics and do not bypass them. You have no
live mutable authority. Write stale-authority-ready in isolated drill control
and report the exact current/old generations and rejected command briefly.
