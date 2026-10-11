# Conditional text/binary compatibility and failures

Status: open
Blocked by: 05
Triage: ready-for-agent
Type: task
Phase: conditional: binary selected in 03

Reuse native pipeline packed/live/observation tests and historical evidence readers. Compare real generated C emitters, mixed typed/literal streams and four independent subjects. Cover signed/canonical literals, frame resets, fragmentation/long lines and partial literal→typed, giant-line→typed, typed→partial literal→END.

## Acceptance

Exact bytes/roots/counts/frames and negative gates pass. Corrupt/reordered/dropped/duplicated/truncated records, missing END, nonzero producer and collector death retain diagnostics. Separate durable subject pointers from accepted verdicts; authenticate retained replacement before release. Independent text/binary equivalence controls shared encoder/decoder mistakes.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Implementation is pending.
