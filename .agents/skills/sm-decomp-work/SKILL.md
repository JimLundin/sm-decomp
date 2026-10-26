---
name: sm-decomp-work
description: Implement, review, compile, replay, or coordinate an assigned sm-decomp migration using the living reviewer plan and role-specific handoffs.
---

# sm-decomp work

Target: owned Session, our renderer/audio, Map Rando. Bank retirement is intermediate.

Read living [PLAN](../../../docs/PLAN.md) first, then [REVIEW](../../../docs/REVIEW.md)'s
Milestone plan. They own priorities, verification, Git and resources. Use the
authoritative repository for instructions and the assigned checkout for source.

## Load your assigned role

| Role | Guide |
| --- | --- |
| Source | [Implement](references/source.md) |
| Review | [Assess](references/review.md) |
| Build | [Compile](references/build.md) |
| Replay | [Verify](references/replay.md) |
| Coordinate | [Integrate](references/coordinate.md) |

## Read contracts when reached

- Ownership/interfaces/backend/content/topology/options/media: [destination](../../../docs/target-architecture/README.md), then its relevant contracts.
- RAM/aliases/reset/load/oracle/persistence: [state migration](../../../docs/ENGINE_STATE_MIGRATION.md).
- Host/native commands: [iteration](../../../docs/ITERATION.md), [dependencies](../../../docs/NATIVE_DEPENDENCIES.md).
- Project completion: [validation](../../../docs/target-architecture/VALIDATION.md).

## Assignment and handoff

Assign role, item, owner, repository, checkout/base, allowed files, rows and recipient.
Pruned snapshots still need the authoritative repository path.

    Use $sm-decomp-work. Role: source. Repo: <path>. Checkout/base: <path@revision>.
    Item/files: <scope>. Rows: <ids>. Owner/recipient: <agents>.

Handoff: exact pins/artifacts, live handles, passed checks, unverified gaps, next action.
Route additional user requests to the coordinator. Role completion is not project completion.
