# Measure pipeline costs and parallelism boundaries

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

Determine which changes enable parallel offline verification and live capture
without weakening the existing evidence checks. Record representative timings,
source citations, limitations, recommended sequencing and follow-up tickets.

## Acceptance

- Run an actual frozen replay differential benchmark with deterministic exact
  inventory/root checks and worker configurations differing in store ownership.
- Measure representative offline verification and dependency invalidation.
- Document safe build/evidence reuse and publication boundaries.
- Complete scoped run cleanup and reauthenticate retained packed evidence.

## Comments

User clarified that both automated offline and live work are in scope, with
heavy parallel execution as the goal.

## Answer

Use a shared local resource budget, private live stores for concurrent capture,
and actual compiler dependency closures for independent offline evidence. Follow
with authenticated compiler caching, single-pass reopened inventories, shared
evidence import and broader fixture orchestration. See [findings](../findings.md)
and [source review](../source-review.md) for evidence and code references.

## Completion evidence

- Branch `decomp-pipeline-performance`, worktree `/tmp/sm-decomp-pipeline`, base
  `16f1883`. Original active gameplay edits were not modified.
- Differential command: `python3 .scratch/decomp-pipeline-performance/benchmark.py
  live --baseline /home/jim/sm-decomp/out/port/storage-pipeline/packed-timing/verdict.json
  --rom /home/jim/sm-decomp/rom/sm.sfc --inputs
  /home/jim/sm-decomp/movies/super-metroid-any-3653.inputs --output
  out/port/pipeline-performance/live` (two repeats); one additional instrumented
  repeat used `live-lock-timing` output. All nine four-subject runs match frozen
  exact inventories, roots, progress and routine ownership.
- Offline command: `python3 .scratch/decomp-pipeline-performance/benchmark.py
  offline --source /home/jim/sm-decomp/deps/snesrev-sm --output
  out/port/pipeline-performance/offline`: system 475 and audio 1,147 cases, eight
  passing actual-C family runs and retained evidence authentication.
- Narrow system compilation: 5.14 s broad vs 0.89 s narrow; compiler dep-info
  26 vs three paths; all 475 full WRAM/return results match. Unrelated audio
  comment rejects system evidence; exact source restoration restores authentication.
  Measurement summaries: [live](../live-results.json),
  [lock timing](../live-lock-results.json), [offline](../offline-results.json),
  [compiler closure](../closure-results.json), [invalidation](../invalidation-results.json).
- Python syntax and local report links checked; `git diff --check` passes.
- Cleanup: `python3 tools/port_cleanup.py --root out/port/pipeline-performance
  --apply` reclaimed zero eligible build bytes. The same command with
  `--deduplicate --apply` reclaimed **2,392,719,360 snapshot bytes**. Explicit
  removal of the two identified unsealed compiler experiment binaries reclaimed
  **9,037,376 bytes**. [cleanup.json](../cleanup.json) inventories retained dependencies.
- After cleanup, nine live verdicts and eight offline verdicts reauthenticated:
  [post-cleanup authentication](../post-cleanup-authentication.json). The original
  frozen baseline reauthenticated with its unchanged verdict hash:
  [baseline authentication](../final-baseline-authentication.json). About 264.67 GB
  free at the final baseline check.
- Retain each benchmark run's private/shared stores, subject pointers, exact
  reader/live binaries and offline source snapshots. Keep this investigation
  worktree for follow-up implementation; no integrated implementer worktree was
  created. Source review used one background research agent and created no runs.
