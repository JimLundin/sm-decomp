# Bulk input context implementation

Status: claimed; provisional source, acceptance remains root-owned.
Owner: `/root/bulk_context`, parent session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Worktree: `/home/jim/sm-decomp-performance-bulk`.
Branch: `performance/bulk-context-20261015`.
Baseline: `c09e47b66a35e6d61dab4065cb7e30d16ff28119`.

## Current checkpoint

The first slice is committed at `94104cc`. The next source checkpoint adds
ordinary captured compiler/source/asset inputs, pre-lock `BatchInputs`, logical
path remapping and compiler dependency binding, bounded generation from the
captured PLM base ROM, and Native declaration/admission cache sharing. Source
bytes remain Arc-owned; streamed assets are copied with bounded digest checks.
Every retained evidence admission still opens and authenticates retained bytes.

The ordinary source wiring compiles. `bulk_catalog` has five passing public
metadata controls: sharing/lazy decode, configuration and namespace mutation,
missing/redirected declarations, memory/I/O/deadline bounds, and incomplete,
duplicate or extra admission membership. Latest commands used jobs 1,
locked/offline cargo, dev/test debug information disabled, and shared pipeline
root `/home/jim/sm-decomp/out/pipeline`, workers 4. Logs are under
`/home/jim/sm-decomp-performance-bulk-proof/metadata-20261015`:

- `ordinary-capture-build.log`: successful compile, 18.91 seconds.
- `ordinary-capture-metadata.log`: all five controls passed.
- Earlier `affected.log` and `batch.log`: focused pipeline unit controls passed
  against the first slice. Pipeline has not yet been rebuilt for this checkpoint.

No actual-C proof has run and no acceptance evidence is claimed. The capture
guard is temporarily conservative; ticket 03 owns its syntax-aware replacement.
The current source view rechecks the union of captured compiler inputs for each
family; narrow that to the selected family and common sources before performance
acceptance. Full catalog freshness checks occur at invocation boundaries, not
inside every worker. Declaration/source counters must remain separate from
freshness reads.

## Remaining implementation and gates

1. Send this compiled source checkpoint to root and ticket 03 for review and
   scanner integration. Root owns final acceptance and Native matrix execution.
2. Finish retained Native capture after completed-receipt reconciliation. Root
   authorized deterministic retained mappings in `builds::compiler_program`,
   original freshness, exact membership, and reservation accounting under the
   existing deadlines and quotas. Reconstruct saved Programs independently;
   receipt replay must not rematerialize captured inputs or launch producers.
3. Narrow per-family source checks; account transient declaration buffers and
   the complete source-preparation budget. Review capture failures against
   existing independent ordinary family outcome semantics.
4. Add and run finite public actual-C single/bulk, complete canonical observation,
   Rust `file!()`/C `__FILE__`, captured mutation and Native replay controls.
   Rebuild exact source before any producer evidence. The warmed private target
   remains `/home/jim/sm-decomp-performance-metrics-continuation-target`;
   TMPDIR is the proof root's `tmp` child. Use shared pipeline workers 4.
5. Run both scoped cleanup passes, retain exact source/tools/logs, and authenticate
   retained evidence after cleanup. Root owns final integration gates, target
   cleanup and implementer-worktree retirement.

Current limitations: retained Native capture, actual-C controls, and cleanup
remain pending; there is no speedup or complete verification claim. All listed
commands have exited. No build/test writer is active at this checkpoint.
Two ignored dependency links were created for cargo: `deps/rustysnes` and
`deps/snesrev-sm`, both pointing into `/home/jim/sm-decomp/deps`. Remove only
these owned links during cleanup; runtime readers use canonical pinned paths.
