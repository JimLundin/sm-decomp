# Broad readable Rust gameplay port

Status: open

Implement the user's goal: translate large parts of the existing decomps into
maintained readable Rust. Follow docs/DECOMP_PIVOT.md. Primary source is the
pinned snesrev/sm; consult reviewed fork corrections and original assembly for
disagreements. Foundation acceptance remains immutable predecessor evidence.

Integration branch: `decomp-port-integration`.

## Contracts and verification seams

Continue the established full canonical WRAM routine boundary: explicit little
endian values, wrapping widths, alias-preserving offsets, full before/after state
and return-value comparisons against compiled actual upstream C. This is the
existing approved foundation test seam. Each family declares valid input bounds;
malformed C accesses are never used as a correctness oracle. Preserve failure
evidence and distinguish translated, offline-verified and live-exercised routines.
Live verification uses independent C/mixed processes with no state repair,
observable hardware effects, Rust hits and fallback measurements.

## Task graph

- 01: reusable family harness and source inventory, no blockers.
- 02: system state routines, no blockers; verification uses 01.
- 03: music and SFX queue routines, no blockers; verification uses 01.
- 04: integrate and exercise family replacements, blocked by 01, 02, 03.
- 05: Samus movement physics and ROM tables, in progress.
- 06: generic enemy geometry and solid-block mechanics, in progress.
- 07: harness assets, captures and 32-bit values, in progress.
- 08: complete Samus animation/input family, in progress.
- 09: generic enemy instruction handlers, in progress.
- 10: live failure controls and dependency seals, in progress.
- 11: actual PLM collision callback harness, in progress.
- 12: complete Samus collision/movement closure, in progress.
- 13: native PLM allocation and terrain setup, offline verified; live integration underway.
- 14: projectile initialization/damage/instructions and missile behavior, in progress.
- 15: common flying enemy AI and shared movement collision, in progress.

This is the first frontier, not the completion boundary of the user's goal.
Expand tickets into substantive gameplay subsystems after this frontier, using
observed dependencies and fixtures. Full ownership remains the direction; do not
equate a count of short helpers with large parts of the game being translated.

## Verified progress

Five-family live run out/port/gameplay-control-live/20261010T201048.927361Z/verdict.json
matches four independent 9,000-frame C/mixed processes, full periodic game/hardware
checkpoints, terminal WRAM/SRAM, per-call captured states, ordered PPU/APU effects
and raw-bit returns. 143 translated symbols have declared offline fixtures;
48 are live-exercised with 48,033 Rust calls plus 23 accepted Rust decompressions
and 8,783 Rust RNG calls per process. Measured CPU fallback is zero in this
prefix; untranslated C still owns scheduling and many game systems. The user's
goal remains active: collision/movement, bank-specific enemy behavior, projectiles,
PLMs, rooms/doors and further gameplay must continue before broad ownership can
be claimed. Counts are inventory, not a percentage of complete gameplay.
