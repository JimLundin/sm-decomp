# Implement exclusive assignment state and pull queue

Status: claimed
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
Claimed by queue implementer on `assignment-queue` at
`/tmp/sm-assignment-queue`; integration owner retains the authoritative status.

Claimed by assignment-queue implementer in `/tmp/sm-assignment-queue` on
`assignment-queue`. The integration owner combines and verifies the merged source.

## Implementation evidence

The native `assignment` module provides typed plan/next/record/status operations,
stable work/revision/attempt identities and an authoritative main-checkout ledger.
It reserves before launch, projects local tickets idempotently, keeps old attempts,
rejects stale completion and preserves submission ownership until integration.
Started claims never expire; recovery requires sealed matching stopped-owner
observations. Existing agent tools and native evidence admission remain caller-owned.
Implementation work includes concrete isolated worktree/branch/batch requirements.

`CARGO_TARGET_DIR=/tmp/sm-assignment-target cargo test -p pipeline --locked --offline
--test assignment_queue` passes ten controls: read-only planning, restart, concurrent
cross-worktree claims, real ticket blockers/claim protection, stale scope updates,
checked recovery, repeated launch/results, evidence retention, path conflicts and
automatic integration lifecycle. Targeted strict Clippy, Rust formatting and
`git diff --check` pass. Temporary test repositories are removed by their Rust
fixtures. Root owns final target cleanup, merged-source checks and ticket resolution.
