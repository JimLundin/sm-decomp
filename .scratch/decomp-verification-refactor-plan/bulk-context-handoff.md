# Bulk input context implementation

Status: claimed; provisional source, acceptance remains root-owned.
Owner: `/root/bulk_context`, parent session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Worktree: `/home/jim/sm-decomp-performance-bulk`.
Branch: `performance/bulk-context-20261015`.
Baseline: `c09e47b66a35e6d61dab4065cb7e30d16ff28119`.

## Current checkpoint

The first source slice adds the invocation-owned catalog/declaration cache,
opaque read-only facts and work counters, ordinary bulk verification with
incremental completion publication, complete batch admission, and pipeline
selection using the same catalog. Exact configuration bytes, discovery membership
and initial manifest digests remain bound. Explicit declarations decode lazily;
matching contents share an Arc-owned payload and decoded manifest. Independent
admission opens retained bytes before consulting its invocation-local decode cache.

Public controls in `crates/offline-verification/tests/bulk_catalog.rs` cover
sharing, lazy decode, configuration/namespace mutation, missing/redirected inputs,
memory/I/O/deadline bounds, and incomplete/duplicate/extra batch membership.
They have not run yet. Rustfmt passed for the owned files. No verification
producer has been launched and no evidence has been granted.

## Remaining implementation and gates

1. Compile the first slice using the released warmed private target
   `/home/jim/sm-decomp-performance-metrics-continuation-target`, locked/offline,
   jobs 1, dev/test debug information disabled.
2. Complete captured compiler/source/ROM/capture input plumbing and deliberate
   path remapping/refusal. The catalog cache alone does not complete ticket 02.
3. Extend lazy Native catalog/declaration preparation and consume admitted
   original/effective declarations. `/root/native_admission` owns the separate
   operation collection/authentication block; avoid those internals.
4. Run finite public metadata, actual-C single/bulk, mutation and affected Native
   controls in a private proof/TMPDIR. Use shared pipeline root and workers 4.
5. Run both scoped cleanup passes, retain exact source/tools/logs, and authenticate
   retained evidence after cleanup. Root owns final integration gates, target
   cleanup and implementer-worktree retirement.

Current limitations: source capture and Native cache integration remain pending;
there is no speedup or complete verification claim. No active build/test process
or output directory exists at this checkpoint.
