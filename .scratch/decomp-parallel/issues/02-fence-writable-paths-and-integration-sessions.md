# Fence writable paths and integration sessions

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Type: task
Owner: `/root/ownership_engine`, thread `01a13453-b599-79c1-84d5-713e6d1e3f57`
Recovery coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`

The Rust queue takes a global mutation lock, but writable overlap and active
integration checks inspect only the chosen effort. Its persistent integrator
token defaults to the first submitter's coordinator and does not identify or
fence a real session. Close those gaps before independent dispatchers mutate
the future shared queue. Reuse `assignment.rs` and `coordination.rs`.

## Work

- Create a maintained Rust ownership boundary across admitted efforts and
  worktrees that rejects overlapping writable paths and duplicate ticket/claim
  ownership, including reserved, running, submitted and bridged legacy work.
- Register one explicit persistent integrator role and allocated persistent
  dispatcher tokens with real session/thread ids. A replacement requires an
  observed stopped owner and a checked/fenced transition; string possession,
  reused `/root` names and elapsed time cannot release authority.
- Preserve legacy coordinator identities for existing updates and seal the
  bridge from ticket 01. The proposed `.scratch/decomp-parallel/ledger.toml`
  remains unactivated; declare exact cutover conditions for ticket 04.
- Coordinate disjoint files and the public contract with ticket 03. Keep shared
  Rust engine edits under one approved owner; interface work can proceed beside
  it without parallel writes to common files.
- Extend meaningful existing assignment tests for two efforts, two sessions,
  overlap, stale attempts, live-child survival, rejected old integrator writes,
  checked transfer and identical retry behavior. Preserve native admission.

## Acceptance

Two admitted dispatchers cannot own overlapping source or duplicate legacy work.
Only the current real integrator authority can mutate shared integration state.
Stopped-owner transfer is checked and old attempts/tokens cannot complete newer
work. Existing queue tests and focused controls pass with exact legacy lineage
preserved. Passing tests prepare the real drill; they do not activate the queue.

## Completion evidence

Original engine commits through `6d23fdab55213f6101c337d12f163550817364d8`
and reviewed corrections `0b46487`, `c2fbc17`, `2714b57`, `c88c7a1` are integrated
on `decomp-preparation-20261013`, frozen source
`c8799f326ffe198b652301f08866f1b7a7bd9ebc`. See
[engine handoff](../preparation/02-owner-handoff.md),
[authority API](../preparation/02-authority-api.md) and
[validator recovery](../preparation/activation-recovery-handoff.md).

Final combined locked/offline verification passed **23 queue, 15 session and
10 authority controls** at the frozen source, including cross-effort/path alias
and duplicate-ticket fences, shared capacity, real caller generation binding,
explicit integrator transfer, surviving-child retention, stale result/authority
rejection, exact retries and submitted-only historical integrator checks.
Typed activation requires real launch/command/claim/process/budget/integration
receipts; boolean/text-only manifests reject. Read-only `validate-activation`
performs identical acceptance checks without activating canonical authority.
Bounded final logs are in `preparation/integration-evidence/frozen-final.*`;
format and locked/offline build passed. Reporting source commit `263ad5a` is
included in the frozen baseline. Ticket 04 owns actual-session acceptance.

Both cleanup passes succeeded, 0 run bytes; private original engine target
1,260,555,955 bytes and validator target 1,481,441,788 bytes were removed.
Clean implementer worktrees were removed after exact source/receipt retention;
all main evidence hashes reauthenticated. Root combined target 1,806,295,023
bytes was removed after retaining the frozen executable separately from Git.
Final free space observation: 223,381,778,432 bytes. Exact source commits,
small retained source/log snapshots, the frozen reader and immutable legacy
bridge/ledger remain dependencies; original gameplay holdings remain intact.

## Comments

2026-10-13: owner reconciliation 01 resolved through retained legacy ownership.
The checked exact-path maintenance approval preserves this surviving worker in
`/tmp/sm-decomp-preparation-ownership-20261013`, baseline
`deceefdb568990fe273cc3f6cdf849726d8380bb`. Implementation and scoped WIP commits
are authorized only for the approval's engine/test files. The original claims,
native admission, main Git ownership and pending production authority remain
recorded in the legacy bridge. See `../preparation/02-owner-handoff.md` and
`../preparation/02-authority-api.md` for implementation recovery.
