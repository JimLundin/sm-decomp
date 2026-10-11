# Conditional native host emitters and transport selection

Status: open
Blocked by: 04
Triage: ready-for-agent
Type: task
Phase: conditional: binary selected in 03

Reuse effect_writer.c.in, live_host.c.in, packed.rs, live.rs and batch.rs. Generate explicit typed PPU/APU and exact bounded literal emitters, END only on successful final gameplay close, opt-in typed Rust transport selection, buffered short-write/EINTR/death handling and TOML sidecar metrics. No Python adapter.

## Acceptance

Preserve fresh processes, current WithGfx/host ownership, observer bytes, FIFO readiness/private stores and one pair lease. Failed producer/collector retains diagnostics/dependencies and cannot produce an accepted verdict.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Implementation is pending.
