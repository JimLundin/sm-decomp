# Publish private observations to shared storage without serializing capture

Status: open
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
