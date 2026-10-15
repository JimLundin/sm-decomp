# Native finalization dependency recovery

Status: implemented; owner controls passed; full combined Native verification active
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

## Implementation and owner evidence

Production source is `7729fafbb4a4eef7606da832d994237c9a84cd73`; final worker
documentation is `60053293bd28e899c37f792815e1b7a858f78722`. Both are assembled
with extraction, metrics and the storage study at frozen
`7b27ea1a29bb2ade3bc6483f99bf59626e298671`. The combined checkout remains
unaccepted and unchanged while `/root/recover_extraction_verification` builds
matched optimized tools and runs the required Native gates in
`/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015`.

`AuthenticatedOperation` owns the freshly authenticated result, exact result
reference and complete dependency references. Native collection and subsequent
whole-receipt admission each create their own owned result, reducing nine
complete result loads per producer to two across finalization. Retention hashes
the same bounded bytes it returns. Program, context, limits, output copies,
reservations, extent accounting and producer slots are unchanged.

The real-producer structural control failed with 39,501 reader bytes for a
10,482-byte closure (0.89 seconds). The candidate read exactly its 10,514-byte
closure once in each of two independent admissions (0.49 seconds). Different
absolute retained paths account for different closure sizes; these are work-count
controls, not controlled throughput measurements. Operation controls passed 14
tests, retention 6, and Offline owner controls 41 with one explicitly ignored
Native gate. Two initial owner failures were missing cleanup-helper configuration;
the unchanged executable passed after configuring the retained helper. All
diagnostics and cleanup outputs remain in the proof parent below.

## Integrated worker retirement

Root removed the integrated worker Git worktree after confirming no writer and
preserved the five exact files listed in `logs/green-source.sha256` at their
original paths, without Git metadata or build output. The dependency symlinks
were unlinked without following their targets. The checkout changed from
208,373,202 to 220,101 logical bytes; this is a 208,153,101-byte logical reduction,
not an allocated-block measurement. A second small source copy is retained in
the proof parent. The released warmed Cargo target remains available for later
owned work and is not counted as reclaimed.

Proof parent:
`/home/jim/sm-decomp-performance-native-admission-proofs/20261015`.
Both post-retirement project cleanup passes succeeded with zero bytes reclaimed
(`logs/post-retirement-cleanup.log`, `logs/post-retirement-deduplicate.log`).
Fresh retained-result admission passed through the copied operation reader
(`logs/post-retirement-admission.log`, one test). Original intent, result,
runner and sandbox file identities/times remained identical in the before
and after stat records; no producer was replaced. All green and diagnostic red
retained hashes, exact readers and source hashes passed again. Available disk
space was 118,004,621,312 bytes at that observation.

Retain the exact operation reader under `tools/green/`, its request/result tree,
and the cleanup reader at
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015/tools/sm-pipeline`.
These successful owner controls and retirement admission do not grant full
Native acceptance; the complete new 12-recipe and application gates remain active.
