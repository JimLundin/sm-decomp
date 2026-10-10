# Automated feedback demonstrations

`just readiness-feedback` executes nine synthetic controls in an immutable run.
A scoped pass verifies the feedback machinery and cannot publish acceptance.
`just readiness` includes the same suite as a mandatory gate; acceptance also
rechecks its actual executable, source/hardware and fixture identities, commands,
exit codes, reports and derived assertions.

The synthetic cartridge's correct table comes from the real decoder and timed
emitter. The isolated probe changes one generated store at `$00:8004/MX=3` to
write `$99` instead of `$12`. Focused comparison names the first differing write,
instruction index 3, and retains the preceding matching boundary and snapshots.
The following correct store restores memory: complete frame-end CPU/clocks,
WRAM and framebuffer still match. This proves why instruction comparison is
mandatory even after frame parity.

Actual replay profiles are evaluated against exact generated bodies. Removing
the unreachable `$00:8100/MX=3` entry defines a preserved synthetic predecessor;
adding it back yields uncovered despite older entries executing. Empty tables
also yield uncovered after complete fallback frame parity. These synthetic
predecessors never impersonate accepted Super Metroid evidence.

Private repositories exercise setup/build failure, signals, timeouts, truncated
capture metadata, unfinished runs, incompatible ROM identity, stale verdicts and
source/input contamination. Real acceptance commands must refuse each candidate
and leave the private predecessor pointer unchanged. Fault controls never target
the workspace accepted pointer or modify maintained configuration/generated code.

A repeated-DMA cartridge exercises pending hardware checkpoint state and
validates both restored sides through the remaining frame and a complete next
frame. Warm and cold focused histories/outcomes must match. Valid reuse,
corruption rejection and incompatible identity rejection each retain their
validation or rejected cache evidence. Cache creation is separate from warm
focus timing; no frame scope is shortened for performance.

Reports live under `diagnostics/feedback/`, execution logs under `logs/`, and the
mandatory summary in `checks/feedback-demonstrations.json`. An expected fault is
a successful demonstration only after its actual failure evidence is checked;
unexpected outcomes, interruption or absent artifacts block the gate.
