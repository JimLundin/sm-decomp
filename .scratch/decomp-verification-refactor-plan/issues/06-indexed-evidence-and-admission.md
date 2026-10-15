# Integrate indexed output, independent admission and scoped retention

Status: open
Blocked by: 02, 03, 05
Triage: ready-for-agent
Type: task

## Scope and ownership

Own the measured Offline evidence writer/reader and private output transport in
`offline-verification`, generic required storage operations in `evidence`, Native
output-reference integration and cleanup consumers. Keep build recipe edits with
ticket 04; coordinate any shared recipe file through one owner.

## Required work

Implement the prototype's selected representation using ticket 03's content,
computation and admission identities. Preserve all logical case
members and producer records, even when equal bytes share content. Bound active
staging, pending packs, collector buffers and total retained data. Commit, reopen
and authenticate before publishing pointers or releasing required staging.
Failure diagnostics remain readable. Import independently produced stores through
the supported ordinal-remapping path.

Within each independent admission, reuse validated immutable payload bytes for
inventory/comparison/chain checks. Later admissions and post-cleanup checks reopen
their own closure. Preserve exact member counts and detect missing data even when
claimed roots match. Support single-case inspection and exact extraction through
the current reader.

Update Native producer ownership, output capabilities, reservations and indexed
references together under one current contract. Reuse existing operation chunks.
Do not treat a collector's aggregate success as evidence for each child. Introduce
no old-format readers or automatic rewrites of frozen receipts.

## Acceptance

All storage/protocol controls from 05 pass in production ordinary and affected
Native recipes. Authenticate without producers or invocation memory after cleanup
and worktree retirement. Physical payload file counts and actual reads match the
selected design; import/compression costs are included in performance results.
Cleanup uses explicit finished-run facts and preserves every published shared
object/source/reader. No general shared-store garbage collector is required.

## Completion evidence

Pending. Record exact byte comparisons, full producer attribution, interrupted-run
controls, timing/resource deltas, cleanup receipts and post-retirement admission.

## Comments

If the prototype selects a narrower improvement, revise this ticket before claiming
it rather than implementing an unmeasured storage architecture.
