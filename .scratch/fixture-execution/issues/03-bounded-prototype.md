# Compare fixture execution with a preloaded isolated helper

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Type: prototype

Implement the accepted ordinary/projectile throwaway Rust experiment. Compare
the existing process/receipt path, prompt fresh-process execution, and loaded
helpers with fork-isolated children using identical real fixtures. Experiments
are capped around 60 seconds each, including startup, execution/evidence and
cleanup. Authenticate source/input identities and exact per-fixture state,
returns and eight host words. Exercise repeat/reorder isolation, refusal,
crash/timeout/cancellation and blocked descendants. Keep production untouched.

## Comments

User accepted prototype-first scope and preserved per-case evidence with
aggregated metrics. Record measured limitations and cleanup before resolving.

## Completion evidence

[Results](../results.md) record comparisons, controls and limits; sources in
../prototype/. Final baseline 9.389–9.601 s; fresh two-worker 1.656 s; fork
two-worker 0.915–0.943 s. Full 1,840-fixture prototype 12.622 s, exact equality.
Isolation/repeat/reorder/refusal/crash/timeout/cancellation/truncated-reply and
blocked-descendant controls passed. Production unchanged.

Both cleanup passes succeeded for every run, reporting zero bytes. Additional
Rust retention shared 40,734 files / 1,483,691,388 logical bytes; cargo clean
removed 1,052 files (284.9 MiB); copied/generated objects and disposable prepare
executable also removed, retaining source and archive. Allocated size
287,424,512 bytes; free disk 217,824,473,088 bytes. Original evidence/source/cache preserved; original
family and 59,052 retained files authenticated after cleanup. No worktrees.

Branch decomp-port-integration, base 5b66ee83fb11d6f13e2aadef2b6d26fd54b8dacb.
Sources/docs remain uncommitted.
