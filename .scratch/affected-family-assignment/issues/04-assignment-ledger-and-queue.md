# Implement exclusive assignment state and pull queue

Status: open
Blocked by: 02
Triage: ready-for-agent
Type: task

## Required work

Implement a Rust pull queue backed by one authoritative main-checkout
`.scratch/` ledger. Publish local ticket scopes and launch briefs, claim before
returning ready work, distinguish stable work/revision/attempt identities and
reserve capacity/writable paths atomically. Preserve existing tracker readiness,
blockers and triage conventions; do not reclaim existing claimed jobs implicitly.

Record started, launch failure, result and checked owner-stop updates. Interrupted
or ambiguous launches require reconciliation; claims never expire automatically.
Enforce disjoint implementation writes, separate worktrees/batches and one
integration owner. Research can overlap. Test concurrency, repeated/stale updates,
restart, publication interruption and stopped-owner recovery through the interface.

## Comments

May proceed alongside ticket03; avoid modifying existing batch execution here.
