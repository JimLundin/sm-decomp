# Reuse bounded evidence reads within independent admission

Status: claimed
Blocked by: 02, 03, 05
Triage: ready-for-agent
Type: task

## Scope and ownership

Own bounded Offline evidence reading in `offline-verification`, its Native
operation-admission composition, and narrowly required generic retained-reader
operations in `translation-execution`. Keep current directory/operation-index
formats and producer output paths. Coordinate shared recipe/input files with 02
and build preparation with 04.

## Required work

Use one bounded owned reader for the payloads a case/producer admission needs.
Authenticate bytes while reading them, and reuse those immutable bytes for the
corresponding inventory, semantic comparison and chain checks. Preserve exact
names/member counts and complete source/producer reference closure; missing,
duplicate, redirected, corrupt or undeclared evidence must still refuse.
Unconsumed members require explicit authentication before success. Do not trust
path metadata or a caller-supplied digest instead of reading the bytes.

Bound live memory by an explicit case/operation window, including branch/chain
continuations, and preserve input/output/deadline accounting. A later admission
or post-cleanup check creates a fresh reader and independently reopens its entire
required closure. Preserve current case inspection, failed diagnostics, cleanup,
Native compiler slots, exact Program/context/request checks, copied output checks
and aggregate reservations. No new output transport or receipt format is selected.

Compose the narrow Native finalization recovery's authenticated-operation result
with the bulk input/identity work. That early slice removes repeated result and
reference loads without changing storage; this ticket also removes redundant
ordinary inventory/comparison reads wherever the bounded reader permits it.

## Acceptance

Production ordinary and affected Native recipes preserve all applicable missing,
corrupt, extra-member, equal-wrong-output, chain, source and producer controls.
Authenticate without producers or invocation memory after cleanup and worktree
retirement. Structural read counts prove removed duplicate payload work inside
one admission; independent later checks perform new reads. Retained file counts
remain those of the existing format. Include complete admission and cleanup in
timings, with all required evidence still readable.

## Completion evidence

Ordinary reader implementation: `8d9d00eb5b3d0b7d31414bae7be6ef3bfa8114d9`,
including structural regression `3621cf6`. The original control read 67 bytes
for a 44-byte closure; six final bounded reader controls pass. The complete
owner gate passed 47 tests with one ignored in 6.25 seconds, including chain,
capture, equal-wrong-output, host/service, Native owner and compiled PLM ABI
checks. Exact logs and retained reader are under
`/home/jim/sm-decomp-performance-admission-proof`.

Both scoped cleanup passes over that proof's `unit-tmp/` passed, reclaiming
zero bytes (`logs/unit-cleanup*.log`). Worktree/target retirement is pending
combined-source acceptance. The reader preserves current formats, authenticates
unconsumed members, and bounds retained windows to 256 MiB with reservations
held until every shared owner releases its buffer. Document and file limits
remain 32 and 64 MiB. Independent admission opens a new reader.

Bulk source `b7f2c0d` includes this reader (integrated `edf69e3`/`b7f2c0d`).
Four preliminary public actual-C controls passed, including complete ordinary
single/bulk observations and a three-pair/eight-producer Native capture/replay
with independent compiler-free authentication after both cleanup passes.
Final combined controls remain pending ticket 03/04 source completion; no
complete-batch performance claim or ticket resolution follows from these probes.

The final02/03/04/06 foundation at `b1f8691` has since passed 48 owner tests
(one explicit real-producer entry ignored), five catalog controls and four
public actual-C/Native controls. Both independent readers passed again after
aggregate cleanup. Complete source/evidence is in
[the bulk handoff](../bulk-context-handoff.md). Root still owns final Native
admission integration, worktree retirement and complete-domain acceptance.

## Comments

2026-10-15 combined checkpoint: `45b89f8` retains optimized Native operation
admission and the complete Ceres recipe. The 48 owner controls and public Native
capture/replay/source-binding check passed; independent admission passed after
both cleanup passes. Root retired the admission implementer worktree and checked
all retained evidence hashes. Complete 1,296-pair Ceres ordinary admission also
passed after that retirement. The target transferred to the test-only Native
negative-control repair. See [integration-foundation.md](../integration-foundation.md).
Final complete Native acceptance is still required before resolution.

2026-10-15 decision from ticket 05: 18 alternating storage-only samples of the
complete 16-pair ordinary proof (333 members, 43,675,132 logical bytes) took
0.246–0.358 seconds for deduplicated directories, 0.880–1.100 seconds for raw
Store payloads, 1.016–2.221 seconds for 256-byte Merkle pages, and 1.366–2.144
seconds for private raw stores plus authenticated import. Packing reduced retained
allocation from 40,206,336 to 9,912,320 bytes, but increased elapsed time.
All 16 reported negative controls passed. Final prototype handoff/cleanup remains
ticket 05's responsibility. Raw measurements:
`/home/jim/sm-decomp-performance-evidence-proofs/20261015/ordinary/matrix.toml`.

Select this narrower admission change for performance. The study measures packing
already completed staging, not direct producer transport; it cannot rule out a
future transport improvement or establish a whole-batch speedup.

2026-10-15: root starts the ordinary admission-reader slice in isolated worktree
`/home/jim/sm-decomp-performance-admission`, branch
`performance/ordinary-admission-20261015`, from the frozen `b333ca5` input seam
plus accepted Native admission source `7729faf`. This only refines the start of
independent reader work; final acceptance remains dependent on complete 02/03/05.
Owner session: `01a13e7f-ab01-7413-a1c2-288b08a4b719`. Own the new private
reader, ordinary authentication in `family.rs`, and recipe comparison functions
in callback/PLM/Ridley modules. Bulk retains preparation/generation functions in
those modules. Shared source registration is reconciled by root at integration.
Private proof: `/home/jim/sm-decomp-performance-admission-proof`; private copied
target: `/home/jim/sm-decomp-performance-admission-target`. No live target files
were copied: ticket 04 waited before its first Cargo write until the independent
copy completed. Frozen `7b27ea1` and its ongoing Native gate are unchanged.
