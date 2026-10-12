# Consolidate local branches into main

Status: claimed
Blocked by: None
Triage: ready-for-agent
Type: task

User authorized merging all local branches and cleaning the workspace before
further work. No remotes are configured. Initial main: `d7a1c66`.

Two committed branches had work outside main: `decomp-port-ticket-03` (`c95fe69`)
and `recomp-iteration-review-fixes` (`cd91328`). The registered storage worktree
also contained an uncommitted, superseded prototype based on `8bbef1f`.

## Integration decisions

- Samus branch merged as `ae8966b`. Current synthetic ROM tables already preserve
  its intended isolation and pose assertions; retain the current table values.
- Iteration merge keeps retired Python entry points deleted. Current Rust APIs
  require sealed preparation, reject unproved correctness fixes, enforce exact
  selected generated keys, preserve interrupted children and authenticate/count
  publication once. Carry the valid cache/late warm mismatch control into current
  typed TOML APIs and share passive hardware observations between timing tests.
  Historical v2 probe receipts retain their original refusal contract; new v3
  receipts require actual cache validation and matching warm mismatch evidence.
- Storage draft preserved in commit `9a0a750` under
  `storage-draft/`, outside the active workspace. Retain its exact source,
  manifests and original notes; the maintained packed evidence crate stays active.
  Retain original-path reader and `/tmp/evidence-spike/store` dependencies.

## Verification and cleanup

Samus six tests, session four recovery/count controls and public policy test pass.
An initial Samus compilation encountered unresolved markers after a sandboxed
checkout failed; explicit resolution and rerun passed. Debug cache validation was
interrupted after exceeding 60 seconds; retained diagnostics and release rerun
will receive scoped cleanup. Release hardware/cache controls and probe pending.
Final branch ancestry, removal, reader authentication, build cleanup and disk
measurements will be recorded before resolution.
