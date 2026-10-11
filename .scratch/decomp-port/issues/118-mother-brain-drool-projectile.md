# Mother Brain drool projectile
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate C843 initializer, C84D head-position tracking and C886 falling
callback. Preserve six offsets, wrapping velocity/subpixel motion and unsigned
floor threshold. Require actual-C full-WRAM evidence and run cleanup.

## Work evidence

C843, C84D and C886 translated. Full port suite and strict Clippy pass.
108 actual-C allocation cases pass and authenticate after zero ordinary cleanup
and 29,491,200 deduplicated bytes:
out/port/mother-brain-drool-allocation/20261011T102109.512232Z/verdict.json.
Retain source snapshots, binaries, ROM and full-WRAM cases. Available disk
268,219,531,264 bytes. First tracking run had incorrect native script fixture
address; preserved failure, cleaned with zero bytes reclaimed, restarted with
native C8C8 Add12 operand. Tracking/falling verification remains pending.

## Completion evidence

Corrected tracking run passes all 475 actual-C full-WRAM/return cases (324
tracking, 150 falling, one native Add12 instruction):
out/port/mother-brain-drool-tracking/20261011T102158.514192Z/verdict.json.
Ordinary cleanup zero bytes; deduplication 63,832,064 bytes. Retained evidence
authenticates after cleanup. Full port tests, strict Clippy and three harness
tests pass; harness tests now generate their own temporary ROM fixture.
Retained binaries, snapshots, ROM and case inventory. Offline equivalence;
live ownership and original-ROM parity separate. Whole-decomp remains active.
Available disk after completion: 268,080,820,224 bytes.
