# Build the combined Rust evidence crate

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task

User requests a workspace Rust crate combining the spikes. Reuse exact burst
encoding and Merkle image diffs, compressed bounded object packs with compact
references/indexes, frame access, and independent provenance/observation seals.
Provide native library/CLI ingestion, extraction, verification and recording.
Keep original captures and production pipeline unchanged until integration.

Implement canonical versioned object identities, bounded decoding, exact legacy
restoration including unknown records/resets, writer lock and durable immutable
pack publication. Exercise public interfaces against real captures and corruption,
missing data, changed/reordered events, malformed input and restart controls.
Document format/lifecycle limitations and measure complete-capture storage/timing.
Clean build intermediates after verification; preserve source and evidence seals.

## Completion evidence

Implemented workspace `crates/evidence` (library and CLI), with exact burst pairs,
selective Merkle diffs, zstd packs and compact node references, blocked frame
indexes, unknown/reset/tail preservation, bounded safe parsing, completed run
closures and independent canonical provenance. Format/API and lifecycle limits
are in `crates/evidence/README.md`; measurements in `rust-crate-results.md` and
`rust-benchmark.json`. No production pipeline migration or acceptance changes.

Ten public-interface tests, cargo fmt, focused Clippy with warnings denied and
`git diff --check` pass. Both source captures compare exactly: 1144 files,
5,734,473,474 bytes. Retained packs total 27,457,588 bytes (~209x reduction);
final binary reauthenticates both run closures. Roots/physical hashes in report.
Retained runnable evidence: `out/storage-rust-final/store` and
`out/storage-rust-final/bin/sm-evidence`.

Cleanup: `tools/port_cleanup.py --root <scope> --apply` and
`--deduplicate --apply` succeeded for storage-rust-final, storage-rust-build,
storage-rust-release, storage-rust, storage-rust-retry, storage-native-micro and
storage-native-final, reclaiming zero eligible bytes. Unsupported disposable
builds/failed empty stores/microbenchmarks were explicitly removed (313873225
file bytes). Temporary source hardlink scopes were removed by the benchmark
finally block on both failures and success. About 268.35 GB disk remained.
The retained small packs and final executable hashes were reauthenticated after
cleanup. Source worktree is the requested deliverable, not an integrated
implementer worktree; the other agent's worktree was untouched.

Commit: combined Rust crate commit on `research/trace-storage-format` (this ticket
is included in that commit).
