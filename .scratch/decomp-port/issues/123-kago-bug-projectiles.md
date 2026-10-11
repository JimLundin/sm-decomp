# Kago bug projectiles
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate D088 initializer, D0B3/D1E4 shared updates, D0CA/D0EC/D128
preinstructions and D15C launch. Preserve RNG reads, signed proximity and
collision branching. Require actual-C full-WRAM/return evidence and cleanup.

## Completion evidence

D088, D0B3, D1E4, D0CA, D0EC, D128 and D15C translated, plus native D0EB
no-op. All actual-C cases pass: 81 allocation, 1,737 callbacks, 90 native-ROM
launch instruction cases. Full port tests and strict Clippy pass.
out/port/kago-bug-allocation/20261011T103929.013863Z/verdict.json
out/port/kago-bug-callbacks/20261011T103930.237180Z/verdict.json
out/port/kago-bug-launch/20261011T103931.429610Z/verdict.json
Ordinary cleanup zero for each; deduplication respectively 10,616,832,
380,764,160 and 11,796,480 bytes. All evidence authenticates after cleanup.
Retain native ROM, snapshots, binaries and full-WRAM/return inventories. Free
disk 266,049,982,464 bytes. Offline equivalence; live ownership separate.
Whole-decomp translation remains active.
