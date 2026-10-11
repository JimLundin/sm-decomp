# Maridia floater spikes
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate D23A initializer and D263 eight-direction acceleration/collision
callback. Preserve horizontal short circuit before vertical acceleration.
Require actual-C full-WRAM evidence and finished-run cleanup.

## Work evidence

D23A/D263 translated. Port suite and strict Clippy pass. All 216 actual-C
allocation cases pass and authenticate after zero ordinary cleanup and
53,084,160 bytes deduplication:
out/port/floater-spike-allocation/20261011T104123.307872Z/verdict.json.
Motion oracle pending. Retain ROM, binaries, source snapshots and full-WRAM
inventories. Available disk 264,704,892,928 bytes.

## Completion evidence

All 4,929 motion/native instruction full-WRAM/return cases pass:
out/port/floater-spike-motion/20261011T104124.492344Z/verdict.json.
Ordinary cleanup zero; deduplication 646,053,888 bytes. Evidence authenticates
after cleanup. Combined with allocation cases, full port tests and strict
Clippy, verifies both callbacks offline. Available disk 264,490,860,544 bytes.
Retain ROM, binaries, snapshots and case inventories. Live ownership separate;
whole-decomp translation remains active.
