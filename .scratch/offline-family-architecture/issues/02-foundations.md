# Extract execution and Recovered C source owners

Status: open
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

## Completion evidence

Pending implementation, checks and scoped cleanup.
