# Implement durable Translation attempts and supervisor interface

Status: resolved
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

Implementation is checkpointed at
`27b8b0ac78080d9ccc953956e1829bf0571efbe2`. The Rust supervisor journal owns
preparation, rewriting, source application, candidate/merged gates, integration,
finalization and publication. Each side effect retains its intent and sealed
result. Ordinary completed/stopped resume starts no candidate evaluation;
operator `retry` and changed-source `repair` require separate recorded reasons
and remaining frozen attempt slots. Real process identities preserve ambiguous
writers across controller crashes.

`verification/supervisor-core-final-02.stdout.log` passes 18 attempt-interface
tests plus checkout/confinement/Git/operation/retention/rewrite suites.
`supervisor-infrastructure-final-01.stdout.log` passes the complete infrastructure
recovery lifecycle (316.04 seconds), and `supervisor-retry-final-01.stdout.log`
passes deliberate failed-candidate retry and changed-source repair (328.99
seconds). Native completed resume allocates zero new evidence, bound by
[native checkpoint](../supervisor-native/actual-c-02-checkpoint.json).

The complete managed lifecycle and exact pending recovery pass on source
`f845516b975c3bfc413800c6bed9e329696a79e6`. Both normal queue entries complete,
the original attempt remains 1, and ordinary/pending resume allocates zero new
producer evidence. Both scoped cleanup passes, measured disposal, helper removal
and retained authentication are complete. See
[final completion](../completion-20261014.md) and
[47 exact references](../verification/completion-f845516.references.json).
The earlier [checkpoint references](../verification/checkpoint-27b8b0a.references.json)
remain retained; production activation is separate.
