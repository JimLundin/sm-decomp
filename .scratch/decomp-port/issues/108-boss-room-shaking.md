# Boss room shaking
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate Crocomire and Ridley room shaking with enemy structure offsets,
branch priority, signed reflection and wrapping camera/register arithmetic.
Cover all Ridley table entries and Crocomire gate/underflow boundaries against
actual C full WRAM. Complete tests, cleanup and authentication.

## Completion evidence

Both native room callbacks are Rust, preserving branch priority, signed offset
reflection, countdown wrap and four-register Ridley table updates. All 148
actual-C full-WRAM cases pass:
out/port/boss-room-shaking/20261011T095259.076605Z/verdict.json.
Includes all nine valid Ridley timer states and Crocomire properties/state/
underflow boundaries. Independent test verifies reflection and complete eight-
step zero-net shake cycle. Full port suite, strict Clippy and diff check pass.
Ordinary cleanup zero; dedup reclaimed 28573696 bytes; all cases authenticate
afterward. Free disk 269725655040 bytes. Retained binary/source/pinned ROM
dependencies; no implementer worktree created. Offline only; live ownership
remains separate. Integration branch decomp-port-integration.
