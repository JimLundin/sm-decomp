# Prove text/binary compatibility and failure behavior

Status: open
Blocked by: 03
Triage: ready-for-agent
Type: task

Compare real C generated emitters and mixed typed/literal streams with legacy text ingestion and unchanged evidence readers. Require exact bytes/roots/counts; cover signed values, canonical literals, fragmentation/long lines, reset frames, dropped/reordered/corrupt/truncated records, missing END, nonzero producer, collector death and retained dependencies. Explicitly cover partial literal→typed, giant-line→typed and typed→partial literal→END. Distinguish authenticated durable subject pointers from accepted live verdicts; retain sealed divergent observations on failed admission. Collector SHA authenticates stored bytes; independent text/binary equivalence controls shared encoder/decoder mistakes.

## Acceptance

Follow the exact contract and verification/rollout gates in `../plan.md`.
Record source revision, controls, measured results and scoped cleanup under
Completion evidence before resolving. Implementation is pending.
