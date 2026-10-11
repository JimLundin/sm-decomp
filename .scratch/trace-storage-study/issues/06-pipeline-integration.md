# Integrate packed evidence into ongoing live validation

Status: resolved
Blocked by: 05
Triage: ready-for-agent
Type: task

Default future live replays to a shared Rust content-addressed store. Stream
hardware events directly into the encoder, pack snapshots/fixtures after each
producer exits, publish durable subject pointers before removing staging bytes,
and compare/authenticate the same exact observation inventories as before.
Preserve legacy readers and explicitly selectable legacy storage. Keep original
historical evidence unchanged. Add extraction/frame tools and meaningful failure
controls, run the actual C/mixed pipeline, and complete scoped run cleanup.

## Completion evidence

Implemented default packed main live/batch replay, direct streamed trace capture,
staged snapshot packing, durable per-subject pointers, exact inventory admission,
packed/legacy authentication and extraction/frame tooling. Details and measured
costs in `../pipeline-results.md`; raw seals/timings in `../pipeline-results.json`.

Real four-process 1000-frame C/mixed runs pass in packed and plain modes with
identical exact observation inventories. All retained packed pointers and source
snapshots authenticate after cleanup. 319MB logical -> 3.20MB packed; replaying
same observations adds zero objects. Ten crate tests, five new pipeline controls,
16 live tests (one existing skip), three cleanup tests, fmt/Clippy/whitespace pass.
Fresh decompressor acceptance rerun required by updated Cargo.lock also passes.

Cleanup utility apply/deduplicate on `out/port/storage-pipeline` and
`out/port/decompressor-offline/20261011T103125.565480Z` completed: 477861689
snapshot bytes reclaimed, zero build bytes eligible. Removed 319347708 identified
unsealed timing/source-symlink bytes. Exact source dependencies retained under
`out/port/storage-pipeline/source` without Git metadata or build trees. No
implementer worktrees created. Shared workspace target remains in use by ongoing
work. After cleanup about 266.49GB free; verdict/pack hashes reauthenticated.

Integrated into `decomp-port-integration`; integration commit includes this ticket.
