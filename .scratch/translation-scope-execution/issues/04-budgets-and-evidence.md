# Enforce budgets and finalize attempt evidence

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Owner: /root (recovered bounded operation and retention integration)

Own hard aggregate time/output budgets, admitted observation extent, process
teardown, bounded failed evidence, exact-closure publication and run finalization.
Keep existing evidence formats/readers and accepted historical dependencies.
Use real output enforcement, not only sampler metrics or format maxima.

## Acceptance

Interface tests exercise excess logs/temp files/snapshots, undeclared output,
timeout, interruption and recovery. They prove termination, no acceptance and a
bounded authenticatable diagnosis. Repeated resume allocates no equivalent output.
Missing enforcement refuses execution. Cleanup protects referenced source/store
objects and records reclaimed bytes. See [the spec](../spec.md).

## Completion evidence

Implementation is checkpointed at `27b8b0a`. The Rust runner bounds aggregate
stdout/stderr, filesystem and unlinked open-file output while writers run,
tears down the process tree on overflow/deadline, and refuses missing descriptor
inspection. Original stage deadlines include input hashing, runtime discovery,
reservation and setup. Trusted post-teardown authentication/finalization has its
own finite IO bounds; retained failure/evidence/journal extent is accounted
separately and never silently truncated.

`verification/operation-input-deadline-01.stdout.log` passes all eleven operation
tests, including crash/surviving-writer recovery, lost responses, ignored quotas,
original-deadline input reservation and tamper refusal. The complete core log
passes five exact retention tests. `supervisor-failure-accounting-01.stdout.log`
passes bounded preparation/rewrite failure finalization in 94.19 seconds;
`supervisor-exact-accounting-01.stdout.log` passes exact cancelled terminal
accounting and tamper refusal in 73.23 seconds. Final cancellation does not change
an already selected finalization/publication disposition.

The native measurement records 5,975,348,667 reserved bytes, 60,198,330 allocated
logical bytes, 60,107,351 finalization bytes, 90,979 publication bytes and zero
new resume evidence. Both native cleanup passes and post-cleanup authentication
passed. The actual optional 176-family catalog timing probe exceeded its existing
10-second assertion at 18.97 seconds; the failed timing result remains retained
without a speedup claim or repeated matrix.

The complete managed proof passes on `f845516b975c3bfc413800c6bed9e329696a79e6`.
Its finalization retains 27,249,752 bytes, publication retains 1,203,738 bytes,
and exact pending recovery allocates zero producer evidence. Both scoped cleanup
passes and post-retirement authentication pass. Measured disposable/helper
retirement reclaims 4,834,795,520 allocated bytes while preserving sealed readers
and private input graphs. [Final completion](../completion-20261014.md) records
the separate logical/allocated measurements, remaining space and all receipts.
