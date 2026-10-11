# Rust offline family verification

Status: resolved
Blocked by: 01

Port full canonical WRAM/return verification, exact pinned source checking,
dependency and artifact authentication, retained diagnostics and parallel case
ownership. Preserve specialized adapter behavior when migrating those callers.

## Completion evidence

Final typed family/batch verification and completed cleanup are in
[completion.md](../evidence/completion.md). This supersedes pending notes below.

Native full-WRAM verification, compiler inventories, current-source authentication,
parallel case ownership, atomic batch publication and authenticated reuse are
implemented. The `rust-family-v1` seal rejects source changes during a run.
System passed 475 cases/17 routines; audio passed 1,147 cases/21 routines.
The final batch also passed PLM (1,444 cases), pirate host callers (577 cases),
and projectile script leaves (342 cases); all five verdicts reused after cleanup.
See `../evidence/sm-final-native-batch.json` and its reuse counterpart. Source
changes make this admission stale by design; frozen artifact evidence remains.
Final build-output cleanup and commit pending.

## Typed migration follow-up

2026-10-11 family section migrated maintained verification/authentication to typed
`family_model` records and TOML manifests/verdicts with version-2 contracts. Full
WRAM/return comparisons, Rust-before-C admission, pinned archive source validation,
current implementation checks and exact artifact inventory remain admission gates.
New native tests compile through the orchestrator and include rebinding a tampered
inventory, invalid pointer-bank rejection and missing/stale live predecessors.
The config audit verified the exact recursive conversion of 141 original configs;
the separately versioned acceptance contract is an intentional exception. All 142
maintained `tools/port` and `tools/decomp` JSON originals were removed. Historical
sealed snapshots and their readers were preserved. Integration verification and
scoped run cleanup remain with the orchestrator; this ticket is still claimed.
