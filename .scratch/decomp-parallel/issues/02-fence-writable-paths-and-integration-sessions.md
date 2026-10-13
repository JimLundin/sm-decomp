# Fence writable paths and integration sessions

Status: open
Blocked by: 01
Triage: ready-for-agent
Type: task

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

Append full commits, ownership/role schemas, legacy bridge hashes, rejection and
retry results, focused test receipts and remaining limitations. Complete scoped
[run cleanup](../../../docs/agents/run-cleanup.md) and record measured bytes/free
space and retained evidence before resolving.
