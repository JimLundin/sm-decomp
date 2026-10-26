---
name: sm-decomp-work
description: Implement, review, compile, replay, or coordinate an assigned sm-decomp engine migration item using the living reviewer plan and role-specific handoffs.
---

# sm-decomp work

Work toward an owned deterministic Session, our renderer/audio, and Map Rando
content, topology and opt-in mechanics. Bank retirement is an intermediate step.

Use the authoritative repository for living instructions and the assigned
checkout for source. A pruned snapshot may have no docs; retain the authoritative
repository path in its assignment. Read [PLAN](../../../docs/PLAN.md)
first, then [REVIEW](../../../docs/REVIEW.md); its current Milestone plan determines
the next unclaimed item. These living files own process, verification, resource
limits and reviewer decisions. Keep changing counts, pins and lane assignments
in the assignment or existing handoff, rather than this skill.

## Select the assigned role

Load only the reference for the work you own; load both if explicitly assigned
two roles. Preserve the team's assigned file, compiler and Git ownership.

| Role | Work | Reference |
| --- | --- | --- |
| Source | Implement one complete responsibility and retire its callers | [source](references/source.md) |
| Review | Assess the exact candidate and its safety/behavior contracts | [review](references/review.md) |
| Build | Compile a fixed candidate and qualify its actual artifacts | [build](references/build.md) |
| Replay | Run named rows and required capture/restore checks | [replay](references/replay.md) |
| Coordinate | Assign work, integrate changes and keep lanes active | [coordinate](references/coordinate.md) |

## Read contracts when the task reaches them

- Changes to ownership, interfaces or backend architecture: read the
  [accepted destination](../../../docs/target-architecture/README.md).
- RAM ownership, aliases, reset/load, oracle adoption or persistence: read
  [state migration](../../../docs/ENGINE_STATE_MIGRATION.md).
- Content/topology/options or media changes: use the accepted destination's
  links to the relevant architecture and technical contract sections.
- Host commands or native dependencies: read the needed section of
  [iteration](../../../docs/ITERATION.md) or [native dependencies](../../../docs/NATIVE_DEPENDENCIES.md).
- Claiming destination completion: apply [validation](../../../docs/target-architecture/VALIDATION.md)
  to the full requested engine. Passing slices alone do not prove it.

## Assignment and handoff

An assignment needs role, item, owner, repository, checkout/base, allowed files,
verification rows and the next recipient. Reuse existing artifacts for pins and logs.
For example:

    Use $sm-decomp-work. Role: source. Item: <responsibility>.
    Checkout/base: <path and revision>. Allowed files: <paths>.
    Rows: <ids>. Deliver the frozen change to <integrator>.

Finish the assigned role with exact artifact identity, any live process/session
handles, what passed, what remains unverified and the next action. Send unrelated
user requests to the coordinator while preserving the active assignment.
A role handoff does not complete the project.
