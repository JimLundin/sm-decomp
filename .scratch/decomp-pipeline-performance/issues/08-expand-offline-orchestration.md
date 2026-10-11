# Cover the current manifest frontier and parallel fixture pairs

Status: open
Blocked by: 02, 05
Triage: ready-for-agent
Type: task

Extend the batch catalog to explicit manifest paths, fixture preparation, validator
kind and live adapter/eligibility. Support dispatcher evidence and changed-family
selection. Keep offline coverage separate from live-admitted profiles; adding a
manifest never silently installs it into gameplay. Record execution first-hit
frames where available to choose useful diagnostic horizons and cumulative runs.

For large fixture sets, benchmark independent case-pair concurrency with distinct
case directories. Preserve Rust-before-C admission within each pair and full WRAM,
return and host-state comparison. Use the shared budget rather than multiplying
family worker count by case worker count. Persistent processes require a separate
proven complete-global-state reset contract and are outside this ticket.

## Acceptance

- Recent family and dispatcher manifests run via resumable checked-in orchestration.
- Missing/stale evidence or unexercised live leaves fail the existing gates.
- Serial/parallel fixture inventories and returns agree; rejecting Rust prevents C.
- Failed pairs retain diagnostic evidence and safe retry behavior.
- Measure throughput and resource usage, then complete scoped cleanup.
