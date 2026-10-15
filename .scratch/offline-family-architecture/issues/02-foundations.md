# Extract execution and Recovered C source owners

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Type: task

## Scope

Implement stage 1 of [the approved plan](../implementation.md). Own the new
`tool-execution` and `decomp-source` crates, their manifests, root runtime helpers,
generic build/cache/bounded-input/metrics modules, source authentication and
inventory extraction, and required existing consumer imports. Preserve typed
source-cache lifetime and shared read accounting. Keep source-specific data out
of generic execution and lower dependencies out of pipeline.

Temporary explicit imports/reexports in pipeline may ease this checkpoint; the
complete migration must remove old family helper access. Do not redesign
schedulers, compiler caching, evidence formats, production policies or gameplay.

## Validation

Compile the new libraries and pipeline with locked/offline Cargo. Run existing
bounded-input, source and compiler preparation tests relevant to the extraction.
Record exact tests and any remaining full-migration dependency before handoff.
Shared process budget: `/home/jim/sm-decomp/out/pipeline`, workers 4.

## Comments

- Authorized by the user's 2026-10-15 approval. Worker/worktree ownership will
  be recorded in the checkpoint before implementation starts.
- Claimed by `/root/pipeline_architecture` under owner session
  `01a13c3b-d421-7bb3-80a2-ec91045678e7`. Writable checkout:
  `/home/jim/sm-decomp-offline-foundations`, branch
  `architecture/offline-foundations-20261015`, baseline `a6d53c5`.

## Completion evidence

Implemented in `446dd64`, verified in `aeb6632`, integrated on main as
`e430361` and `d2cf88c`. [The handoff](../handoff-foundations.md) records the
independent-library/dependency checks and all 35 distinct passing source,
bounded-input, compiler-cache and process tests. The source-context follow-up
also checks fresh reconciliation and restoration of its enclosing scope.

Both scoped project cleanup passes succeeded (zero remaining run bytes and
zero snapshots). Cargo removed 3,094,654,976 measured allocated build bytes;
temporary extraction tools reclaimed 9,256,960 bytes and empty proof directories
8,192 bytes. Logs/receipts are retained in `../foundations-verification/`.

The integration owner confirmed the worktree was clean and had no surviving
writer, then ran `git worktree remove /home/jim/sm-decomp-offline-foundations`.
The removed worktree/private prerequisites occupied 273,383,424 allocated bytes.
No retained verdict depended on those files. Available disk afterward was
157,111,398,400 bytes. No production policy, claim or unrelated worktree changed.
