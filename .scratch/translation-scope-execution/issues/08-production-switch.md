# Switch reviewed production scopes to deterministic execution

Status: claimed
Blocked by: None
Triage: ready-for-agent

Prepare the concrete production-switch result after the complete restricted-path
proof. Reconcile active real owners, existing claims/submissions and evidence.
Integrate under current checked authority and obtain the user's explicit switch
decision before activating the new production path, as agreed in [the spec](../spec.md).

## Acceptance

Migrated work uses the trusted supervisor and restricted rewriting. Unrestricted
dispatch cannot mutate migrated attempts or invoke their evidence producers.
Original lineage and historical inspection/authentication remain valid. Activation,
rollback/recovery steps and completed cleanup have durable evidence.

## Completion evidence

The user explicitly authorized production migration on 2026-10-14 and paused
other work for the switch. The separate switch decision is satisfied. Root owns
the migration implementation, checked authority recovery and production activation.
Existing submissions and held integrations retain their original lineage.

[Read-only original Ceres preparation](../production-preparation/preparation-checkpoint-27b8b0a.md)
was built and separately revalidated against the complete retained native proof
and unchanged main authority/source/ledgers/progress. Original Submitted source
commits, Running linked integration source, claim tuples, owners, results,
observations and abbreviated `base-head` bytes are preserved. Identical builder
retries return the original receipt and run no new validator. Historical initial
read-budget refusal remains immutable; the same package's later bounded read-only
report passes as `validated-with-blockers` and `activated = false`.

Actual blockers remain: exact per-original reviewed installations and policies;
unsupported Submitted/linked Integrate admission and overlapping held sequencing;
specialized native catalog recipes and families outside the proved scope; canonical
queue work outside the Ceres package; fresh real-owner/process reconciliation;
and the user's separate explicit switch decision. Main authority, queue, source/
index and progress were not changed. Do not ask for activation approval until the
exact supported production preparation is concrete and reviewable.

Implementation and complete managed proof cleanup are now complete; ticket 07
is resolved. [Final completion](../completion-20261014.md) records passing
post-retirement authentication and an unchanged final production inspection.
This ticket remains open until its real per-original preparation, decision,
integration/activation and cleanup contracts are fulfilled. The retained package
is `validated-with-blockers`; it is not a ready activation request.
