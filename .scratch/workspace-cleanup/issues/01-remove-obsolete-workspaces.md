# Remove obsolete workspaces and caches

Status: resolved
Blocked by: None
Triage: ready-for-agent

## Scope

Audit linked worktrees and ignored generated storage. Remove integrated clean
worktrees while preserving evidence at required paths. Apply the project run
cleanup utility and deduplicate authenticated immutable observations. Preserve
dirty or unmerged worktrees and evidence dependencies.

## Completion evidence

Initial available disk space: 263625355264 bytes.

- Pruned 22 registrations for missing `/tmp` worktrees with
  `git worktree prune --verbose`. Git metadata required sandbox escalation.
- Removed 10 clean integrated worktrees using the audited `plan.json` and
  `/tmp/sm-workspace-cleanup.py`. Reclaimed 784154624 allocated bytes. Preserved
  every real `out/` tree at its original path and seven cited research source
  files. Removed six empty archive remnants afterward (77824 allocated bytes).
- Removed main `target/`, `deps/rustysnes/target`, `.ruff_cache` and Python cache
  using the exact audited paths: 1517416448 allocated bytes. No evidence manifest
  referenced the two Cargo cache roots.
- `/tmp/sm-run-cache-cleanup.py` removed only unreferenced compiler intermediates
  inside finished run build directories: 4087648256 allocated bytes. Recorded
  executables stay at their original paths. Three runs without verdicts were
  skipped, listed in `retained-unfinished-runs.json`.
- Applied `python3 tools/port_cleanup.py --root <run> --apply` to all 15 roots in
  `port-runs.json`. Initial accepted-run cleanup removed 189637892 file bytes;
  the final pass reported zero eligible bytes for every root. Removed 24
  unsealed link archives from rejected runs after checking nested verdict and
  acceptance inventories: 569699338 file bytes. Diagnostic failures stay intact.
- `python3 tools/port_cleanup.py --root out --deduplicate --apply` reclaimed
  288738536 snapshot bytes before being interrupted during large-log reads.
  Continuing the same eligibility, SHA-256, byte equality and atomic hardlink
  operations with the bounded-memory `/tmp/sm-stream-dedup.py`.
- After worktree/cache removal, original-path AD proof and scoped demonstration
  both pass. All 13 sealed root owned-evidence inventories pass. The retained
  live batch and both packed runs authenticate, including shared stores and
  readers. Both legacy decompression runs pass binary and all eight subject/epoch
  checkpoint inventories (456 and 472 files). See `authentication-final.json`
  and `live-authentication.json`. The newer live reader does not support the old
  `decomp-live-v1` layout, so its exact recorded inventory was checked directly.

Retained worktrees:

- `.claude/worktrees/trace-storage-format`: modified Cargo files and untracked
  storage experiment source. Its referenced experimental reader remains.
- `/home/jim/sm-recomp-iteration-review-fixes`: unmerged commit `cd91328`.

Retained evidence-only archives at old paths: iteration 02, 04 and 07 plus
`/tmp/sm-trace-storage-study`. Context proof copies, all accepted/rejected runs,
acceptance pointers, shared store `out/port/storage-pipeline/store`, pinned
dependencies/corpora, downloads, ROM and movie inputs remain. Source and branch
history were preserved. Historical pointers already naming the pre-rename
`/home/jim/sm-recomp` path were not rewritten.

Streaming deduplication completed: 901233 existing snapshots checked,
16387302578 additional bytes reclaimed, 1204.7 seconds. Every replacement used
SHA-256, streaming byte equality and atomic hardlinks; no recorded evidence
contents or paths changed. The batch and both packed-store runs authenticate
again after the complete pass (`live-authentication-post-dedup.json`). Final
selected-run cleanup reports zero candidates, worktree prune dry-run is empty,
and `git diff --check` passes. Removed regenerated Python caches and all four
cleanup-only files under `/tmp`; `final-sweep.json` records them.

Measured reclamation: approximately 23.8 GB. Exact byte totals and final free
space are in `reclamation.json`. Measurements combine allocated cache/worktree
sizes and utility-reported file bytes; filesystem-wide free-space changes also
reflect concurrent work elsewhere. Only `.scratch/workspace-cleanup/` is added
to the main checkout; no maintained source was changed.
