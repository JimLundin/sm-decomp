# Wrecked Ship robot lasers
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate D30C/D32E/D341 and common D35B initialization. Preserve sign-selected
muzzle offset, variant graphics and the asymmetric native sound visibility test.
Verify full-WRAM actual-C allocation and existing D3BF collision callback.

## Work evidence

Three initializers and native common positioning/sound gate translated.
Full port tests and strict Clippy pass. All 617 actual-C collision/native
instruction cases pass and authenticate after zero ordinary cleanup and
80,871,424 bytes deduplication:
out/port/robot-laser-collision/20261011T104405.931746Z/verdict.json.
First allocation run rejected invalid fixture sound cursors; preserved and
cleaned with zero bytes. Corrected fixture enables sound with valid cursors.
Allocation rerun pending. Retain native ROM, snapshots, binaries and inventories.

## Completion evidence

All 480 corrected allocation cases pass and authenticate:
out/port/robot-laser-allocation/20261011T104510.274487Z/verdict.json.
Ordinary cleanup zero; deduplication 130,023,424 bytes. Free disk
264,138,993,664 bytes. Covers all five native headers, three velocity variants,
signed muzzle offset, full/low/high pools and native visibility sound bounds.
Together with collision evidence, full port tests and strict Clippy establishes
offline equivalence. Retain ROM, binaries, snapshots and full-WRAM cases.
Live ownership separate; whole-decomp translation remains active.
