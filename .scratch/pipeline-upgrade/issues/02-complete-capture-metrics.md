# Extend native capture and publication measurements

Status: claimed
Blocked by: 01
Triage: ready-for-agent
Type: task
Phase: first phase

Reuse builds::execute_timeout, packed::capture/publish, live::verify and batch::operate elapsed/wait records. Add only missing pipeline-metrics-v1 fields with typed Rust models and TOML sidecars: complete outer wall, process CPU/RSS, bounded concurrent RSS/disk sampling, stage intervals, failure outcome and unavailable-field semantics. Preserve old timing readers. Instrument existing text capture/import in evidence/src/main.rs in this phase. Use a common CLOCK_MONOTONIC/run origin or parent-normalized offsets for cross-process intervals.

## Acceptance

One owner reaps each child once and integrates resource collection with polling/timeout/kill. One lease covers the producer/collector pair, without helper reacquisition. Metrics remain outside canonical comparison inventories.

User-directed completion revision (2026-10-12): routine verification defaults to
no detailed telemetry; benchmarks and explicit `SM_PIPELINE_METRICS=enabled`
retain all detailed metrics. Preserve exit/CPU accounting and elapsed receipts.
The enabled-versus-disabled 2% target remains a requirement for future automatic
telemetry promotion, not a claimed result or a reason to repeat hour-long tuning
runs. Record the short control's limits and the failed/stopped long control.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Claimed on `pipeline-upgrade-metrics` from integration
`09d3ed16585211d697d91e04bd60e0ab062f8605`; public test seams were confirmed.
Design/context: `/tmp/sm-pipeline-upgrade-metrics-design.md`.

## Comments

Claimed on `pipeline-upgrade-metrics`, based on ticket 01 completion. Producer
and resource sampling support use separate worktrees with exclusive files.
Process accounting (`797f3a2`) and capture/import stage instrumentation
(`f081f21`) are partial checkpoints awaiting the complete metrics contract,
controls, overhead measurement, integration and cleanup. They do not resolve
this ticket.

2026-10-12 crash recovery integrated the completed scope/process/resource
implementation as `a3290f5` / merge `bd82cf2`; public controls and fresh
system/decompressor/RNG/live admission passed. The telemetry overhead control
was interrupted because unrelated host CPU/memory activity made it incomparable.
The <=2% gate remains unproved, so this ticket stays claimed and 03 stays blocked.
Restart: [checkpoint-02.md](../checkpoint-02.md).

## Completion evidence

Latest bounded measurements and default-off delivery policy:
[short-measurements.md](../evidence/short-measurements.md). The public default-mode
execution control went red on the old default (enabled=true), then exercises no
RSS/disk samples, one owned process record and successful execution. Final
verification/integration cleanup remains pending; no large-case <=2% claim.

Partial implementation, controls, fresh admission, cleanup, retained dependencies
and the remaining overhead gate are recorded in
[metrics-recovery.md](../evidence/metrics-recovery.md). Native cleanup/dedup and
post-cleanup authentication succeeded; all three integrated support worktrees
were removed. This evidence does not resolve the ticket.
