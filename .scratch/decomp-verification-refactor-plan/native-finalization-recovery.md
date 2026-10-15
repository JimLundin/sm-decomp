# Native finalization dependency recovery

Status: diagnosis complete enough to assign a bounded implementation control
Owner: performance session `01a13e7f-ab01-7413-a1c2-288b08a4b719`

The frozen extraction's PLM proof completed its entire 1,969-pair domain and
five rejected ABI controls, then exceeded the unchanged 60-second trusted
reconciliation deadline. It published the ordinary verdict and operation index,
but no Native receipt. All 3,941 operation results and diagnostics remain under
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015/recipes/`.
The exact failure log is the proof parent's `logs/recipes.log` (510.41 seconds).
This result is an incomplete baseline, not acceptance or a throughput comparison.

## Source evidence

At extraction source `4a736587af557de9e44b3126ff08f6d6d11beee8`:

- `offline-verification/src/native.rs:1305` enters one 60-second reconciliation
  scope for inventory, operation collection, indexing, extent accounting and
  complete pre-publication Native authentication.
- `collect_operations` authenticates each prescribed request, calls
  `operation::reference` and `operation::references`, and authenticates the
  operation again. Whole-receipt admission then repeats operation validation.
- `translation-execution/src/operation.rs:999` implements `reference` by calling
  full `authenticate` / `load_result`. `references` also calls `load_result`,
  then rereads result metadata. Each output in `load_result` is hashed by
  `authenticate_reference`, then read again into its owned result buffer.
- `authenticate_operation` repeats reference collection and per-reference
  authentication after its own request admission. These calls compose complete
  payload admission repeatedly, although each caller needs projections of the
  same already authenticated operation.

The precise final child operation that exhausted the deadline is not instrumented.
Repeated payload admission is the leading hypothesis; tree traversal and metadata
reconstruction remain secondary costs to distinguish in the bounded control.

The verifier's source count is nine `load_result` calls per producer during a
successful finalization: six in collection, then three in whole-receipt admission.
For the completed 3,941-entry PLM index, collection alone made 23,646 such calls;
a complete finalization would make 35,469 before later independent proof/retry
admissions. Each call reads each material payload twice. These are counts derived
from the executed loop structure, not measured syscalls or completed finalization.

## Recovery implementation contract

Prepare a private worktree from the clean extraction commit. Introduce a small
owned authenticated-operation result in `translation-execution`, exposing its
result, exact result reference and complete transitive references from one
coherent bounded read scope. Private constructors establish admission; caller
supplied hashes cannot construct it. Use those projections in Native operation
collection and verification, preserving each expected Program, context, request,
producer slot, copied output, log, reservation and retained-extent check.

Within a single admission, authenticate bytes as they are read and reuse only
those owned immutable bytes. Keep memory bounded to one operation or an explicit
small bounded scope. A later independent Native admission opens and authenticates
the entire required evidence again. Do not introduce a global trusted-file cache,
alter a receipt format, remove a producer, extend any deadline or silently retry
the expired historical attempt.

First add a public-seam control using a complete small actual producer. Count
logical reader bytes or equivalent structural work to reproduce the repeated
read cost, compare all returned bytes/references with the existing interface,
and exercise missing/corrupt output, wrong request/context, rejected output,
retired runtime and repeat admission. Then run the full PLM proof from new frozen
source in a fresh proof root under the original limits. Retain the failed root.

This is a dependency-recovery slice of the planned independent-admission work;
it can precede the broader ticket 06 storage decision because it changes neither
storage nor computation identity. Record its exact implementation/gates before
using it as the recovered extraction baseline. The final required matrix and
all existing attribution/refusal controls still apply to combined source.

## Cleanup

Failed baseline cleanup passed both project passes. Deduplication reclaimed
430,571,520 bytes across 5,907 snapshots; retained intent, verdict, index, chunk
and result hashes and mtimes are unchanged. Final receipt authentication is
unavailable because finalization never published one. The passing ordinary
verdict and exact producer evidence remain, with no claim of Native acceptance.
