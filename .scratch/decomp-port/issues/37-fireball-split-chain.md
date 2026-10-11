# Directional fireball split and repeat chain
Status: resolved
Blocked by: 35
Triage: ready-for-agent

Translate directional initializers 94A0/94B4/94C8/94DC, area-dependent properties,
split/repeat opcodes 95BA/95ED/9620 and collision preinstructions 950D/9522/9537/
9540/9549. Preserve source-slot aliasing, byte-only timer decrement and allocation
order. Verify actual C through script, spawn and scheduled frame seams; clean runs.

## Completion evidence

Native four-direction spawning now copies source position/E, applies area-specific
properties and records repeat definition F. Three split/repeat instructions and
five collision preinstructions are integrated into public script/frame dispatch.
Repeat preserves the high timer byte; split allocation remains descending and
new lower slots execute in the same frame.

Actual pinned bank86 full-WRAM/raw-return comparison passes 528 cases:
96 allocation, 96 instruction, 120 preinstruction and 216 full-frame cases at
`out/port/fireball-split-chain/20261011T034257.078970Z/verdict.json`.
Two independent Rust tests verify byte-only decrement with a full pool and
velocity-preserving impact transitions. Full port suite, fmt and strict Clippy pass.
This proves offline fixture equivalence; live execution remains open.

The initial frame run rejected an unsupported preinstruction in an auxiliary
occupied fixture slot; retained diagnostic evidence. Corrected those slots to
valid sleeping scripts and reran all cases successfully.

Cleanup: `port_cleanup.py --apply` reclaimed zero bytes in the scoped run root;
`--deduplicate --apply` reclaimed 155,189,248 snapshot bytes. The initial rejected
run was also cleaned (zero bytes). No implementer worktree created. Retained
fixture ROM `out/port/fireball-split-fixtures/split.sfc` and immutable source/run
evidence; final verdict authenticated after cleanup. 36 GiB free.
