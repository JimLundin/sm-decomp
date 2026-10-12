# Implement shared fixture execution

Status: resolved
Blocked by: 03
Triage: ready-for-agent
Type: task

User authorized production implementation. Share dependency scheduling,
Rust-before-C execution, prompt completion, worker ownership and aggregate
metrics across family/callback/PLM. Add sealed preloaded fork execution for
ordinary/projectile fixtures; keep fresh execution selectable for comparison.
Preserve evidence and historical authentication. Callback/PLM stay fresh until
their initialized images have separate proof. Validate real family coverage,
failure/cancellation/isolation controls and live admission, then scope cleanup.

## Completion evidence

[Implementation and validation](../implementation.md). Shared scheduler/prompt
execution across family/callback/PLM; sealed fork subjects default for ordinary
Linux x86_64 families. Fresh mode selectable. No Python/runtime/live changes.
Implementation and validation complete on decomp-port-integration.

Full projectile: 1,840 cases, 71.85 s total / 31.083 s fixture span; 18,400 files
match original. Full system: 475 cases, 16.92 s. Both authenticated. All relevant
family/worker/callback/PLM/metrics/resource/live ABI controls passed.

System live: mixed-C-Rust-integration, 1,000 frames, matching state/effects;
four subjects / 108 artifacts each authenticated after cleanup. Run
/tmp/sm-fixture-production/live-system/1791825803119621155-3. Capture 6.311 s;
total 92.32 s. Short projectile replay correctly rejected empty activity.
Earlier failed diagnostics: relative predecessor path, toolchain mismatch.
Evidence working-directory context and explicit toolchain resolved those.
Full-route projectile live coverage remains outside this short check.

Both cleanup passes succeeded for every finished/failed run. Projectile dedup
554,696,704 bytes / 5,520 snapshots; system 156,368,896 bytes / 1,425; others zero.
Live auto-cleanup removed intermediates; test owners cleanup then remove trees.
No worktrees. cargo clean -p pipeline removed 3,671 files (2.8 GiB); verifier
retained at /tmp/sm-fixture-production/sm-pipeline. Free disk 218,087,964,672 bytes;
evidence allocated 849,006,592 bytes. Family/live authentication repeated after
build cleanup. Preserve original predecessors/snapshots, shared stores/cache,
live-store, subject evidence.toml pointers/reader, family generated files/archive
and source/manifest/pinned-archive paths. git diff --check passed.
