# Prove concurrent sessions and crash recovery

Status: open
Blocked by: 02, 03
Triage: ready-for-agent
Type: task

Existing queue tests and the manual runbook do not prove that independent live
sessions can cooperate and recover. Exercise the implemented contracts from
02 and 03 with two actual sessions before activating production authority.

## Work

- Prepare a disposable isolated checkout and effort with no writable overlap
  with production claims, private inputs or retained evidence. Retain the drill
  source and receipts outside disposable build trees. Production
  `.scratch/decomp-parallel/ledger.toml` remains unactivated during this drill.
- Launch two actual independent sessions with distinct real identities and
  admitted dispatcher roles. Both read the same queue, request work concurrently,
  and dispatch subagents with disjoint worktrees, handoffs and run directories.
  Prove capacity and the shared `SM_PIPELINE_WORKERS=4` budget hold across them.
- Crash or interrupt a parent after its worker starts; the other session must
  observe and retain that live child's claim. Then stop a worker, preserve its
  scoped WIP commit/evidence, check its exact identity and claim, and launch a
  replacement only after the supported checked-stop/fenced transition.
- Interrupt after durable result publication and retry the exact update.
  Reject a stale attempt and old integrator authority. Transfer an observed
  stopped integrator while retaining submission ownership and verify one
  integration at a time. Record the resumed exact command and worktree.
- Authenticate retained receipts after scoped cleanup/removal. Use the passing
  drill and ticket 01's ownership inventory to seal production cutover with
  admitted roles, current source, capacity and legacy claim bridge. Validate
  that unresolved legacy paths remain held before any canonical claim launches.

## Acceptance

Retained evidence identifies both actual sessions and their workers and proves
exclusive disjoint work, global capacity, surviving-child ownership, stopped
worker recovery, exact retries, stale rejection and fenced integration transfer.
A replacement resumes from source/handoff files without reconstructing a chat.
Only then may a validated cutover activate production canonical authority;
new gameplay scopes still require ticket 05. An unrun drill leaves the workflow
unproven and the production queue unactivated.

## Completion evidence

Append session/thread ids, full source commits, role/claim tuples, commands,
process identities, handoff hashes, run/queue/rejection receipts, evidence
authentication and cutover/legacy bridge hashes. Complete scoped
[run cleanup](../../../docs/agents/run-cleanup.md), remove disposable worktrees
after preserving dependencies, and record measured bytes/free space before
resolution. Keep production source, claims and retained evidence intact.
