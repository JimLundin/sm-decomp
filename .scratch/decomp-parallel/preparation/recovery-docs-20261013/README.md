# Reviewable recovery documentation

Status: proposed; root finalizes actual receipts before adoption
Owner: root `01a13500-1ef8-73d3-9a0d-3446b6050eba`

`proposed/` mirrors six destination docs: preparation status, new launch handoff,
spec, map, ticket04 and session runbook. `proposed-docs.patch` is a reviewable
patch against the observed originals; root may copy/adapt it after adding actual
final drill/readiness/cleanup receipts. [stale-claims.md](stale-claims.md) lists
the exact operational corrections and final status transitions.

All final verdict/cleanup slots remain **PENDING**. Preparation readiness follows
completed 04; overall spec/map/gameplay remain open. Historical source/evidence
documents and original proposal remain unchanged. Root owns current-source
equivalence and helper-v2 review receipts, actual validation and final adoption.

Read-only checks confirmed frozen source HEAD `2d59536c059a1ca9c31d9278bff2d9222eafa56c`
and a clean frozen worktree; retained engine SHA-256
`df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84`, build
SHA-256 `659caced91feb9fc00bdbce7e7ced4e464874a9b8db8db84a69051dbff610bb6`, Ceres
ledger `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89` and v2
bridge `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.
These document checks are not actual drill/readiness validation.
`git apply --check proposed-docs.patch` passed against the observed originals;
the patch covers six docs with 284 added and 116 removed lines. Ticket04's
historical completion-evidence body remains byte-for-byte intact.

No builds, fixtures, replays, worktrees, production mutations or Git/index
changes ran for this task. No-run cleanup reclaimed 0 bytes; only this retained
proposal directory was created. Source/docs links in `proposed/` are expressed
relative to their eventual destination paths.
