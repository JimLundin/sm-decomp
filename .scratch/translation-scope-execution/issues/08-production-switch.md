# Switch reviewed production scopes to deterministic execution

Status: open
Blocked by: 07
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

Pending validated switch preparation, user switch decision, integration and cleanup.
