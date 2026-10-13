# Ceres continuation and reusable game host

Status: open
Blocked by: None
Owner: /root

Continue `.scratch/decomp-strategy/revised-strategy.md` with two independent,
bounded infrastructure changes. Preserve the existing gameplay acceptance and
evidence contracts. All new tools, orchestration and tests are Rust.

1. Add an explicitly nonaccepting compact original-ROM controller diagnostic;
   extend the causal comparison beyond frame 1600 toward the first later input
   edge at frame 2218. Retain exact poll order, completed state comparisons,
   independent changed-gameplay control, explicit capture limitations and a
   whole initial experiment budget of 60 seconds. The ordinary-reset Ceres
   escape milestone is the observed game-state sequence 32 -> 33 -> 34; later
   captures follow measured cost and first divergence.
2. Execute unchanged complete Samus projectile and room-scroll modules through
   a shared synchronous typed host interface. Verify actual-C WRAM and ordered
   PLM/HDMA/VRAM effects, recording adapter behavior, pre-host refusal and fatal
   errors after host ownership. Add material callback fixtures, not no-op-only
   coverage. Do not broaden live admission during this experiment.

## Ownership and assignment

Persistent coordinator: `ceres-20261013`. Capacity: 3. Shared pipeline root:
`/home/jim/sm-decomp/out/pipeline`; workers: 4.

These are infrastructure tasks reusing existing translations. The gameplay-only
`translation-scope-v1` promotion rejects pipeline/observer/document writes and
the nonaccepting prefix command. Use the existing typed Rust assignment engine
for durable infrastructure claims, isolated returned worktrees, submissions and
serialized integration. The coordinator records focused verification and
cleanup explicitly; no gameplay acceptance is inferred from infrastructure
queue completion. `assignment_driver.rs` includes the maintained engine instead
of implementing another queue.

Root owns shared callback adapter registration and catalog entries, verification,
integration, source snapshots, cleanup and inventory observations where a
gameplay stage is actually established. Existing historical evidence remains
separate from current-source checks.
