# Extend native capture and publication measurements

Status: open
Blocked by: 01
Triage: ready-for-agent
Type: task
Phase: first phase

Reuse builds::execute_timeout, packed::capture/publish, live::verify and batch::operate elapsed/wait records. Add only missing pipeline-metrics-v1 fields with typed Rust models and TOML sidecars: complete outer wall, process CPU/RSS, bounded concurrent RSS/disk sampling, stage intervals, failure outcome and unavailable-field semantics. Preserve old timing readers. Instrument existing text capture/import in evidence/src/main.rs in this phase. Use a common CLOCK_MONOTONIC/run origin or parent-normalized offsets for cross-process intervals.

## Acceptance

One owner reaps each child once and integrates resource collection with polling/timeout/kill. One lease covers the producer/collector pair, without helper reacquisition. Metrics remain outside canonical comparison inventories. Enabled versus disabled text telemetry overhead is at most 2%.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Implementation is pending.
