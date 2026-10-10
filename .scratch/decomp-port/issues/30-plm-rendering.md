# Native PLM rendering and drawn frame instructions
Status: resolved
Blocked by: 29
Triage: ready-for-agent

Translate DrawPlm and its VRAM queue helpers, coordinate calculation and frame
integration. Preserve horizontal/vertical clipping, tile flips, screen-boundary
queue splitting, camera offsets, chained draw segments and scratch aliases.
Verify through actual pinned C full-WRAM cases and independent Rust examples.
Rendering writes queued VRAM state; hardware execution remains the host's job.
Complete finished-run cleanup and remove integrated implementer worktree.

## Completion evidence

Renderer integrated from decomp-plm-render commits c71edf4/8d5350d. Native
horizontal/vertical clipping, all tile flips, VRAM screen splits, chained offsets
and capacity stops preserve actual C behavior. Coordinate divisor uses low byte,
including zero-divisor result. Public process_frame now executes level-data draw,
coordinate calculation and renderer; control-only seam remains available.

Actual-C evidence out/port/plm-rendering/20261010T225759.092054Z/verdict.json
passes 1,100 full-WRAM/raw-return cases across six seams: 256 renderer cases,
48 coordinate cases and 40 drawn frames (including eight multi-slot frames),
plus prior instruction/control/data cases. Seven independent renderer tests and
one full drawn-frame expected-state test pass; full Rust suite/fmt/Clippy pass.
No live rendering or original-ROM parity claim. Message coroutine and unsupported
preinstructions still require host ownership.

Cleanup --apply reclaimed 0 bytes; --deduplicate --apply reclaimed 181,272,576
snapshot bytes. Implementer cargo clean removed 84.8 MiB, then its integrated
worktree /tmp/sm-plm-render-worktree was removed. Evidence authenticated after
removal and cleanup; sources depend only on retained integration files and pinned
C tree. Small synthetic ROM/manifest retained in out/port/plm-render-fixtures;
39GB remains free.
