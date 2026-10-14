# Performance-first Fixture format

Status: claimed

## User constraints

- Improve execution performance, not only repository appearance or storage size.
- Backwards compatibility is not required.
- Preserve the meaning and integrity of Fixture pairs, Fixture chains and Offline families.
- Binary fixtures are the source of truth (explicit user decision). Inspection may render text on demand; no parallel authoritative text fixture representation.

## Decision

Use uncompressed indexed, merged spans with typed metadata and shared payloads.
The completed study compared TOML with indexed writes, merged spans, full root
images and compressed spans. The production migration makes binary fixtures
authoritative and retains inspection/import tooling for authoring. Existing
retained evidence and installed production policies are preserved; new
verification uses the new format.

## Evidence scope

Use Rust, the current typed Manifest contract, real representative manifests, exact state comparisons and retained measurements.
Separate loading/authentication from state materialization. Include warm reads and per-file cache-eviction hints on private copied inputs.
Record competing machine load; no global cache eviction and no concurrent benchmark variants.

## Deliverables

- Reproducible isolated Rust benchmark.
- Measured format recommendation and concrete integration scope.
- Cleanup of disposable generated corpora and build outputs; retain source and results.
- A binary Fixture module, inspection/import tools, migrated source fixtures and callers.
- State-equivalence and actual-C integration checks, with existing authentication invariants.

See issues/01-format-study.md, issues/02-binary-fixtures.md and
issues/03-evidence-cost-audit.md for work status. The follow-up audit addresses
the user's request to leave no avoidable evidence cost in active execution and
to commit coherent changes directly on `main`.

## Completion evidence

The study and migration tickets are resolved; the evidence-cost audit is in
progress. The implementation on `main` contains 177 migrated
families with 149,120 equivalent cases. Final source fixture size is 40,045,625
bytes versus 272,739,363 bytes previously. Final authenticated-loading medians
for three representative families are 2.52, 3.20 and 2.50 ms; these are input
measurements, not complete verification or gameplay throughput.

See `findings.md`, `results-production-migration.json`, `results-final-corpus.json`
and `results-final-loading.json`. Actual-C verification, negative controls,
authoring, discovery and lifecycle checks passed. Retained fresh/fork C verdicts
still authenticate after scoped cleanup removed the private build directory.
All 423 files in `retained-final.sha256` passed their checksum check. Exact
source, readers, measured binary inputs and retained C runs are under
`out/fixture-format-performance/20261014T202443Z/`; ticket 02 records cleanup
commands, bytes reclaimed and remaining storage. The core migration is committed
as `5105f5c`; standalone producers are committed as `f7dcbd0`. The isolated
inventory test-path fix is `604b17b`. No PR is required for local integration.
