# Complete capture and publication measurements

Status: open
Blocked by: None
Triage: ready-for-agent
Type: task

Reuse lease/cache/capture/import boundaries; add pipeline-metrics-v1 process/stage receipts, full outer wall time, CPU/RSS and bounded transport/disk sampling. Failure receipts and unsupported-field semantics are mandatory. One lease covers each producer/collector pair; child accounting helpers must not reacquire it. Measure both processes. Verify telemetry overhead on text is at most 2% before benchmarking transport.

## Acceptance

Follow the exact contract and verification/rollout gates in `../plan.md`.
Record source revision, controls, measured results and scoped cleanup under
Completion evidence before resolving. Implementation is pending.
