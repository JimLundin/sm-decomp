# Verify save/load and credits

Status: open
Blocked by: 07, 13, 18, 19, 20, 21, 22, 23
Triage: ready-for-agent
Type: task

Complete the declared vanilla route through credits and verify supported
save/load behavior independently. Persistence and completion child scopes can
develop concurrently once their actual dependencies are ready; final route
integration remains serialized.

## Work

- Assign coherent save/UI/audio or route gaps from ticket 05 and add their child
  tickets as blockers. Preserve canonical SRAM/player/room ownership and actual
  C evidence; extend existing bounded domains only with explicit scopes.
- Verify save, reset/load and fresh-process continuation from retained SRAM.
  Compare declared persistent state, resources, room/player restoration and
  hardware/audio lifecycle effects with actual C and original-ROM boundaries.
- Complete the agreed adapted any% route through credits using declared inputs.
  Record every timing adjustment and its reason, first mismatch, exercised Rust
  behavior and remaining gameplay ownership. Preserve independent subjects,
  source identity and existing negative controls.
- Address encountered persistence, rendering, audio and lifecycle failures
  through dependency-ready children; checkpoint source and handoffs before
  long route gates.

## Acceptance

Supported save/load survives a fresh process and preserves declared game state.
The integrated declared route reaches credits with required original-ROM and
actual-C/live evidence. Input adaptation remains reviewable; state injection and
hidden fallback cannot turn failures into completion. Remaining C/CPU ownership
is reported for ticket 09 rather than concealed by a credits result.

## Completion evidence

Append full commits, child pointers, SRAM/source/input identities, fresh-process
and reset/load commands, material state/effect hashes, independent route verdicts,
adaptations, rejecting controls, ownership metrics and stage observations.
Complete scoped [run cleanup](../../../docs/agents/run-cleanup.md), authenticate
retained evidence after worktree removal, and record measured bytes/free space.
