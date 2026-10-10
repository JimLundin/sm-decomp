# 07: Restore paired checkpoints validated against cold execution

**What to build:** Focused replay can start from a trusted paired checkpoint, with incompatible or corrupt state rejected and rebuilt cold.

**Blocked by:** 06: Compare focused instruction and bus execution.

**Status:** done

- [x] Capture complete reference CPU/hardware and subject pending/dispatch state plus input position and relevant counters; do not interchange emulator-core state formats.
- [x] Bind each checkpoint to ROM/movie/source/profile/serializer identities, settings and its own digest.
- [x] Validate restoration against at least the next complete cold-executed frame, comparing ordered traces and ending state, and preserve the validation report.
- [x] Reject incompatible/corrupt checkpoints and rebuild a candidate cache cold; never relabel an old state to imply compatibility.
- [x] Cross-candidate reuse must precede affected execution and verify equivalent replay history; otherwise require a new cold candidate cache.
- [x] Report cache-creation and warm focused-replay times separately, targeting 60 seconds without weakening comparison or using cache execution as a substitute for required cold gates.


## Completion evidence

Implemented `4da1c0c` on readiness-ticket-07 based on integration3437a85.
Four public paired-checkpoint tests pass: distinct complete envelopes, faithful
restore through partial plus next complete cold frame, corrupt/incompatible cache
preservation and cold rebuild, and wrong generated-store cache that faithfully
restores divergent execution then reports its warm first mismatch. Tests91.65s;
5 diagnostic/coverage regressions pass;26Python tests pass (2optional long parity
skips);workspace Clippy and diff check pass. Maintained hardware restoration patch
preparation is idempotent and identity-bound preflight passes.

Cache creation, validation and warm replay times are separately exposed.
Cross-candidate reuse conservatively rebuilds cold, preventing old state relabeling.
Real fallback cache evidence attemptf474e4f445d9 timed out180sec and remains
incomplete. Runb7fa4f286e60 continues with1200sec budget; final real-domain
acceptance remains ticket12 and no scoped cache pass certifies readiness.
