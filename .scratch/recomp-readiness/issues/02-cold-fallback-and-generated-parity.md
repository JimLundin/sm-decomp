# 02: Run cold fallback and generated parity with complete evidence

**What to build:** One command builds and replays the current fallback-only or generated subject against the reference, retaining a complete identity-bound result for the requested scope.

**Blocked by:** 01: Capture immutable runs and verified preflight.

**Status:** ready-for-agent

- [ ] Reuse the existing frame comparator and resumable generated-step runner; keep untimed whole-subroutine output separate from cycle evidence.
- [ ] Cold readiness parity consumes exactly input frames 0–17999, seed 0 and P1 under the verified recording profile, comparing decoded CPU/clocks, complete WRAM and framebuffer.
- [ ] Regenerate/build current generated inputs, preserve exact inputs and source/generated/binary digests, and reject stale or incompatible ROM/table identities.
- [ ] Fallback and generated results are separate; generated validation requires a nonempty table and actual generated execution.
- [ ] Report expected/completed frames, first frame disagreement, generated/interpreted totals, logs and generation/build/replay durations through the uniform evidence contract.
- [ ] Truncation, budget exhaustion, interruption, timeout, setup/build failure and mutation cannot produce a successful validation result.

