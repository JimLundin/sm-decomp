---
name: sm-decomp-work
description: Implement, review, compile, replay, or coordinate an assigned sm-decomp migration using the living reviewer plan and role-specific handoffs.
---

# sm-decomp work

Target: a portable boot-to-credits engine with one deterministic owned Session,
our renderer/audio and Map Rando, within PLAN/REVIEW's size goals. Bank retirement
is intermediate.

Read living [PLAN](../../../docs/PLAN.md) first, then [REVIEW](../../../docs/REVIEW.md)'s
Milestone plan; re-read it at each item's start. Direct user instructions win.
REVIEW sets current priorities; PLAN owns the Fast loop, verification, resources,
Git and completion. Use the authoritative repository for instructions and the
assigned checkout for source. sm/src is authoritative; preserve behavior and
SNES RAM compatibility.

Transfer to one primary root only after explicit release and current event/turn
checks establish the prior writer is terminal; carry live jobs, pins and reservations.
The primary owns implementation, coordination, Git, builds and one resource ledger.
Work on main by default; use branches/private copies as useful isolation.
Keep one actual writer and PLAN's required checks.

## Load your assigned role

Load only the assigned guide and the technical contracts it reaches.

| Role | Guide |
| --- | --- |
| Source | [Implement](references/source.md) |
| Review | [Assess](references/review.md) |
| Build | [Compile](references/build.md) |
| Replay | [Verify](references/replay.md) |
| Coordinate | [Integrate](references/coordinate.md) |

## Read contracts when reached

- Ownership/interfaces/backend/content/topology/options/media: [destination](../../../docs/target-architecture/README.md), including its Windows migration policy, then relevant contracts.
- RAM/aliases/reset/load/oracle/persistence: [state migration](../../../docs/ENGINE_STATE_MIGRATION.md). Resolve unclear behavior with the oracle, ram_map.rs and [bank logs](https://patrickjohnston.org/bank/).
- Host/native commands: [iteration](../../../docs/ITERATION.md), [dependencies](../../../docs/NATIVE_DEPENDENCIES.md).
- Project completion: [validation](../../../docs/target-architecture/VALIDATION.md).

## Assignment and handoff

Assign role, item, owner, authoritative repository, checkout/base/source pin,
allowed files, rows, resource budget and next recipient. Pruned snapshots still
need the authoritative repository path.

    Use $sm-decomp-work. Role: source. Repo: <path>. Checkout/base/pin: <path@revision/hash>.
    Item/files: <scope>. Rows/budget: <ids/allowance>. Owner/recipient: <agents>.

Before finishing, name owner, allowed files, exact source/artifact hashes,
actual live or terminal handles, results, unverified gaps, next action and recipient.
Transfer accepted checks only for unchanged pinned source and relevant dependencies.
Route additional user requests to the coordinator. Role completion is not project completion.
