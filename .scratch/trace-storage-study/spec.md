# Trace and frame storage study

Status: resolved

Investigate lossless storage for translated-code validation in the isolated
`research/trace-storage-format` worktree, starting at `8bbef1f`. Measure existing
captures without modifying them, compare codecs and structure-aware encodings,
and recommend a format with bounded seeking and authenticatable evidence.

This is a study, not authorization to migrate accepted evidence. The separate
`worktree-trace-storage-format` draft remains owned by that effort.

## Tickets

- [01 — Format investigation](issues/01-format-investigation.md)
- [02 — Global-store comparison](issues/02-global-store-comparison.md)

## Integration evidence

Research branch: `research/trace-storage-format`, based on `8bbef1f`. Retained
study worktree: `/tmp/sm-trace-storage-study`; this is the requested deliverable
worktree, not an integrated disposable implementer worktree.

[Recommendation](recommendation.md) records measured results, format contract,
limitations and implementation sequence. Four primary benchmark runs plus a
final three-offset smoke run passed byte-exact codec/representation round trips.
Original sealed traces/WRAM/frames were reauthenticated (1,144 files,
5,734,473,474 bytes). Both retained verdicts remained byte-identical.

Scoped cleanup apply/deduplicate succeeded with zero eligible bytes. Disposable
Python bytecode (13,452 bytes) was removed. No captures or build outputs were
created. The research ticket contains detailed cleanup and evidence dependencies.
