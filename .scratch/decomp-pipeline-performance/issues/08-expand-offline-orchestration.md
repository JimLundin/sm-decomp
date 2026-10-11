# Cover the current manifest frontier and parallel fixture pairs

Status: resolved
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

## Completion evidence

Catalog accepts explicit manifests/kind/prepare/live adapters and detects the offline frontier, with explicit dispatcher opt-in and changed-source selection. Family/dispatcher/callback case pairs run independently under shared leases, preserve Rust-before-C and deterministic records, and retain mismatch diagnostics. Current batch verifies 475 system, 1147 audio, 342 dispatcher and 43 animated-tile cases and reuses all four after authentication.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
