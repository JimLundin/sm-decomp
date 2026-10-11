# Publish private observations to shared storage without serializing capture

Status: resolved
Blocked by: 03, 06
Triage: ready-for-agent
Type: task

Choose a content-addressed import or versioned portable-pack format after measuring
private-store retention costs. Existing packs have store-global ordinal references
and consecutive index ranges, so copying independent packs into the shared store
is invalid. Import must reconstruct/remap physical references while preserving
canonical object identities, complete observation roots and producer provenance.

Concurrent capture stays private. Serialize only atomic shared publication/import,
authenticate the imported closure, publish the updated pointer durably, and release
private packs only when every dependent pointer is safe. Interrupted imports keep
private dependencies and never steal crash-recovery locks.

## Acceptance

- Two concurrent candidate runs import shared observations without writer collisions.
- Private and shared restored inventories/roots are identical.
- Crash at each publication step cannot lose referenced evidence.
- Missing packs, ordinal inconsistencies and corrupt imports reject.
- Measure import cost, lock occupancy, physical deduplication and throughput.
- Reauthenticate after private-store cleanup/relocation.

## Completion evidence

Rust import reconstructs canonical objects in dependency order and remaps physical ordinals. Source and reopened destination inventories/provenance must agree; returned inventory is checked against the pointer before durable replacement. Private stores release only after final authentication. Failed imports retain private evidence; real runs share a store safely. Full import cost remains material for multi-GB logical observations.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
