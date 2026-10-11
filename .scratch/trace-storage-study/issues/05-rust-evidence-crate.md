# Build the combined Rust evidence crate

Status: claimed
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

Pending.
