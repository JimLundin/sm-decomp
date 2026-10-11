# Trace and frame storage study

Status: open

Investigate lossless storage for translated-code validation in the isolated
`research/trace-storage-format` worktree, starting at `8bbef1f`. Measure existing
captures without modifying them, compare codecs and structure-aware encodings,
and recommend a format with bounded seeking and authenticatable evidence.

This is a study, not authorization to migrate accepted evidence. The separate
`worktree-trace-storage-format` draft remains owned by that effort.

## Tickets

- [01 — Format investigation](issues/01-format-investigation.md)
- [02 — Global-store comparison](issues/02-global-store-comparison.md)
- [03 — Disk-backed storage spike](issues/03-storage-spike.md)
- [04 — Matched other-worktree comparison](issues/04-other-spike-comparison.md)
- [05 — Combined Rust evidence crate](issues/05-rust-evidence-crate.md)

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

Working follow-up spike: [spike-results.md](spike-results.md), ticket03. Full two
capture store54.93MB includes all indexes/manifests versus5.73GBoriginal. Native
writer, full conversion/exact decoding, reopened snapshot reads and indexed frame
extraction measured; cleanup complete. Retained working store under
`out/storage-spike/final`, with physical seals in ticket03 and compact benchmark
JSON committed with source. Production integration and end-to-end replay timing
remain outside this spike.
