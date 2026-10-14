# Prove one complete restricted Translation scope execution

Status: resolved
Blocked by: 03, 04, 05, 06
Triage: ready-for-agent
Owner: /root

Own complete interface proof using a real supported existing Translation scope
against actual C, deterministic rewrite replay and the restricted production
adapter. Run focused tests and a bounded representative complete attempt before
expanding verification. Do not launch a long telemetry experiment matrix.

## Acceptance

All [spec test-surface outcomes](../spec.md#test-surface-and-acceptance) pass, including
faults, quota exhaustion, recovery, exact retries and honest stage publication.
Record complete elapsed time, executed/reused gates, allocated/retained bytes,
measured cleanup and post-cleanup authentication. Existing evidence readers work.

## Completion evidence

Source checkpoint `27b8b0a` contains the complete deterministic Rust supervisor
and adapters. [Native checkpoint](../supervisor-native/actual-c-02-checkpoint.json)
binds its real enemy-jump actual-C proof: 16 Fixture pairs, two routines, four
candidate/merged gates, 34 unique gate producers and 102 reused gate references.
The complete loop took 255.393 seconds; the interface test including repeated
authentication/recovery passed in 484.17 seconds. Resume allocated zero new
evidence. The retained measurement records output allocation, finalization and
publication extent, and 1,926,142 reclaimed checkout bytes.

Both proof cleanup passes succeeded with zero eligible bytes, 96 checked source
snapshots, and post-cleanup retained authentication passed in 43.47 seconds.
The retained reader is exact and must survive disposable target retirement.
Its immutable Installation's original disposable controller path was later
rebuilt; historical read-only authentication uses the retained reader and exact
source/evidence dependencies. Original-ROM parity and Live replay were not tested.

The complete infrastructure and deliberate retry/repair lifecycles also pass,
as do focused failure/accounting/deadline/denial/tamper/recovery suites. Their exact
compact results are in
[checkpoint references](../verification/checkpoint-27b8b0a.references.json).
The complete managed diagnostic actual-C lifecycle with an installed nonrecorded
Rust protocol worker passes on `f845516b975c3bfc413800c6bed9e329696a79e6`.
Its loop takes 103.420 seconds and full recovery test 601.65 seconds; four gates
retain two Fixture pairs each, six unique gate producers and 18 reused references.
Exact pending recovery allocates zero producer evidence and preserves the
original source/claim/worker/attempt and both normal queue transitions.

Both scoped cleanup passes, measured target/helper retirement and exact-reader
post-retirement authentication pass. Total allocated reclamation is 4,834,795,520
bytes; final observed available space is 191,088,582,656 bytes. The original native
proof authenticates in 27.31 seconds and managed 11 in 41.98 seconds after disposal.
No verification process remains. [Final completion](../completion-20261014.md),
[managed checkpoint](../managed-gates/actual-c-managed-11-checkpoint.json) and
[47 retained references](../verification/completion-f845516.references.json)
bind these results. The managed fixture uses synthetic stopped-owner observation
and grants no production liveness or shipped gameplay acceptance. No completed
broad/linker matrix was rerun; production preparation/activation remains ticket 08.
