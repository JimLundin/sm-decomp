# Enemy-projectile terrain collision closure
Status: resolved
Blocked by: 33
Triage: ready-for-agent

Translate horizontal/vertical bank86 terrain collision, extension chains,
square slopes and vertical slope alignment with actual WRAM aliases. Integrate
eye-door projectile/sweat and other supported collision preinstructions. Preserve
raw carries, subpositions, global index and block scratch writes. Verify valid
inputs against actual C and independent tests, then cleanup runs/worktree.

## Completion evidence

Translation integrated in a079726. Both bank86 terrain movement entry points
preserve fixed-point carries, square slopes, vertical slope alignment, extension
chains and global-slot aliases. Five collision preinstructions cover eye-door
projectiles/sweat, robot lasers and room effects through the existing scheduler.

Actual pinned C comparisons pass 1,232 full-WRAM/raw-return cases in
`out/port/enemy-projectile-collision/20261011T033446.054879Z/verdict.json`.
Added explicit positive/negative extension chains into air/solid targets, alongside
the block-type/slope fixture matrix. Six instruction/movement/preinstruction/frame
seams pass 940 cases in
`out/port/enemy-projectile-collision-frames/20261011T033400.949490Z/verdict.json`,
including the collision preinstructions and scheduler integration. Four independent
Rust collision tests and the complete port suite, formatting and strict Clippy pass.
This establishes offline equivalence within the fixtures; live installation and
original-ROM parity remain unproven.

Retained diagnostic failures: using the generic harness for internal bank86
dispatch symbols rejected at C link; the existing dispatcher harness fixes that.
An expanded collision run rejected duplicate case directory names; unique names
fixed the manifest and the complete rerun passed.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes in both run roots;
`--deduplicate --apply` reclaimed 1,202,454,528 collision snapshot bytes and
594,542,592 frame snapshot bytes. Removed the integrated clean
`/tmp/sm-eproj-collision` worktree including its 306 MiB disposable target.
Retained immutable run artifacts and source snapshots; 36 GiB disk space remains.
Both accepted verdicts authenticated against current source after cleanup.
