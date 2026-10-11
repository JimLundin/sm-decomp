# Spike-shooting plant projectiles
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate D992 initialization, all ten directional movement paths and native
camera bounds deletion. Preserve full subpixels and signed wrapping bounds.
Require actual-C full-WRAM evidence and cleanup.

## Completion evidence

D992 initialization and complete D9DB ten-direction movement/bounds deletion
translated. All 120 allocation and 1,441 motion/native instruction actual-C
full-WRAM/return cases pass. Covers odd parameter aliases, complete subpixels,
velocity wrapping and inclusive camera edges. Full port tests/strict Clippy pass.
out/port/plant-spike-allocation/20261011T104703.433202Z/verdict.json
out/port/plant-spike-motion/20261011T104704.598616Z/verdict.json
Ordinary cleanup zero; deduplication respectively 30,670,848 and 188,874,752
bytes. Both authenticate after cleanup. Earlier missing-manifest runs cleaned
with zero bytes; diagnostic verdicts preserved. Retain ROM, snapshots, binaries
and full-WRAM inventories. Free disk 263,696,236,544 bytes. Offline equivalence;
live ownership separate. Whole-decomp translation remains active.
