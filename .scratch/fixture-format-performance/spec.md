# Performance-first Fixture format

Status: resolved

## User constraints

- Improve execution performance, not only repository appearance or storage size.
- Backwards compatibility is not required.
- Preserve the meaning and integrity of Fixture pairs, Fixture chains and Offline families.
- Binary fixtures are the source of truth (explicit user decision). Inspection may render text on demand; no parallel authoritative text fixture representation.
- Historical results are not a compatibility requirement. Remove obsolete
  readers, migration scaffolding and legacy verification branches; prioritize
  current verification throughput (latest explicit user direction).

## Decision

Use uncompressed indexed, merged spans with typed metadata and shared payloads.
The completed study compared TOML with indexed writes, merged spans, full root
images and compressed spans. The production migration makes binary fixtures
authoritative and retains inspection/import tooling for authoring. Verification
uses one current format. Historical replay and old receipt compatibility are
not requirements of the final implementation.

## Evidence scope

Use Rust, the current typed Manifest contract, real representative manifests, exact state comparisons and retained measurements.
Separate loading/authentication from state materialization. Include warm reads and per-file cache-eviction hints on private copied inputs.
Record competing machine load; no global cache eviction and no concurrent benchmark variants.

## Deliverables

- Measured format comparison using isolated Rust experiments.
- Compact measurements and the integrated format decision; no one-off study
  programs, obsolete readers or archived verification bundles in the final tree.
- Cleanup of this session's generated corpora, historical bundles and build outputs.
- A binary Fixture module, inspection/import tools, migrated source fixtures and callers.
- State-equivalence and actual-C integration checks, with existing authentication invariants.

See issues/01-format-study.md, issues/02-binary-fixtures.md and
issues/03-evidence-cost-audit.md and issues/04-current-verification-only.md for
work status. The follow-up audit addresses
the user's request to leave no avoidable evidence cost in active execution and
to commit coherent changes directly on `main`.

## Completion evidence

All four tickets are resolved. The implementation on `main` contains 177 migrated
families with 149,120 equivalent cases. Final source fixture size is 40,045,625
bytes versus 272,739,363 bytes previously. Final authenticated-loading medians
for three representative families are 2.52, 3.20 and 2.50 ms; these are input
measurements, not complete verification or gameplay throughput.

See `findings.md`, `results-production-migration.json`, `results-final-corpus.json`
and `results-final-loading.json`. Actual-C verification, negative controls,
authoring, discovery and lifecycle checks passed. Fresh/fork C verdicts
authenticated after scoped cleanup. At the user's subsequent direction, obsolete
readers, migration/study code and this session's archived results are retired
under ticket 04. Ticket 02 records the initial validation and cleanup.
The core migration is committed
as `5105f5c`; standalone producers are committed as `f7dcbd0`. The isolated
inventory test-path fix is `604b17b`. No PR is required for local integration.

`1777785` shares authenticated bounded reads with comparisons and input decoding;
248 selected unit/integration checks passed. `1c7e592` requires one current native
receipt/ownership/reservation contract, removes historical compatibility code,
and avoids duplicate attribution and per-producer intent reads. Its 18 selected
native/producer tests passed, including real C/Rust and current retry controls.
Formatting and diff checks passed. One-off source, obsolete readers and this
session's archived build/run bundles were retired; current commands and Rust
tests remain. Ticket 04 records scoped cleanup and disk measurements.
