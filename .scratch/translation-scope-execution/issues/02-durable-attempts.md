# Implement durable Translation attempts and supervisor interface

Status: claimed
Blocked by: 01
Triage: ready-for-agent
Owner: /root (recovered supervisor integration)

Own the Translation scope execution module's durable state, immutable identity,
intent/result publication and small trusted supervisor surface. Resolve policy
and claim inputs internally. Worker messages contain no execution recipe or
authority. Use Rust and the [spec](../spec.md).

## Acceptance

Interface tests prove equivalent starts/submissions/resumes recover one attempt;
lost responses cannot create another execution; changed source and explicit retry
produce recorded new evaluation identity; stopped attempts resume without work.
Cancellation and recovery keep ambiguous writers owned. Preserve existing lineage.

## Completion evidence

Pending implementation, scoped commit, tests and run cleanup.
