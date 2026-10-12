# Consolidate local branches into main

Status: resolved
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
Release diagnostics four tests, generated timing three tests and prospective
timing six tests pass. The diagnostic suite took 220.53 s; the standalone release
probe passed in 206.36 s. Cache validation serializes ordered observations for
each restored side through two rollovers; this is a slow existing verification
path, not a new decomp-family timing result. Focused Clippy with warnings denied,
Rust formatting and git diff checks pass.

## Completion evidence

Iteration integration: `2ceb792`. Storage archive integration: `d32d1a6`.
Every one of the 75 recorded branch tips (including the original main) is
reachable from main. Retired all 74 non-main branch names with safe `git branch
-d`; original names/commits remain in [branch-tips.toml](../branch-tips.toml).
Only main and the primary registered worktree remain; prune dry-run is empty.

End-to-end nonaccepting demonstration:
`out/runs/1791828305410752190-2`. Preserves failed publication, three-experiment
history, matching predecessor/disabled controls, valid divergent cache and warm
mismatch at instruction 86. Source frozen at
`0a8bff0666330fc94e487e28b83f19e688372777f7732339c4ab5d693a795f1f`.
Isolated build 74.08 s; probe 181.00 s. Public `iteration-demonstrations verify`
passes after both project cleanup passes and Cargo cleanup. No production
acceptance or live gameplay coverage is claimed by this synthetic run.

Cleanup commands used retained `sm-pipeline` binaries with `cleanup --root ROOT
--apply` and `--deduplicate --apply` on the interrupted diagnostic directory,
standalone probe, private lineage and complete demonstration. Complete run build
cleanup reclaimed 133,116,956 file bytes; other passes reported zero. Interrupted
diagnostics at `/tmp/sm-focused-later-1976142` and standalone successful probe at
`/tmp/sm-branch-integration-probe` remain as small diagnostic evidence. Successful
tests removed their temporary trees. `cargo clean` removed 9,216 files / 3.5 GiB.
Retained verifier/probe binaries are in `/tmp/sm-branch-cleanup-retained`.

Removed stale storage worktree lock after confirming recorded owner PID 73148
does not exist. Removed its registration/build tree after backing up all draft
source and the referenced reader. Original path now holds only 847,872 allocated
bytes of evidence source/manifests/reader, without Git metadata. Initial worktree
allocated 71,815,168 bytes; net reduction 70,967,296 bytes. Reader SHA-256 remains
`953cf9c865363305ce4a424a0db26a3eb1c00712039e2def69920e9a2ab50205`.
After removal, the reader extracted image
`907ddd00b9f07bb4c7ee7e5439de24b6fe24144ce073bf29d9079600f040fd5f`
from `/tmp/evidence-spike/store`; cmp against original frame-001000.bin passed.

Remaining disk space 218,883,952,640 bytes; final demonstration allocated
433,811,456 bytes. Preserve its generated executables, source snapshots, process
receipts, diagnostic states and verdict; historical store/reader dependencies,
accepted/rejected evidence and shared stores are unchanged. Final source diff
check passes; consolidation record is committed on main.
