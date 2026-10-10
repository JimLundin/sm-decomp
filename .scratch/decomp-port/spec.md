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

This is the first frontier, not the completion boundary of the user's goal.
Expand tickets into substantive gameplay subsystems after this frontier, using
observed dependencies and fixtures. Full ownership remains the direction; do not
equate a count of short helpers with large parts of the game being translated.
