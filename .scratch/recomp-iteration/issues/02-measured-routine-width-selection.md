# 02: Select a measured routine/width increment

**What to build:** Given the current accepted state, produce a reproducible choice of one covered routine/width slice that is likely to move useful interpreted CPU work into generated execution. Explain the choice with measured instruction evidence, or report why no useful supported candidate remains.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Verify the accepted run, retained source/config identities and validated domain before using its evidence. When acceptance is absent or invalid for the proposed change, establish the necessary readiness gates before treating a baseline as usable.
- [ ] Use complete, identity-bound interpreted instruction counts by address/MX and opcode/pattern within the validated prefix. Obtain a matching instruction profile if the accepted evidence lacks one; call-target misses remain discovery input rather than instruction counts.
- [ ] Identify bounded routine/width candidates using current timed support and observed execution. Estimate gain from newly compilable executed keys, accounting for overlapping generated work, and allow a routine to remain partly interpreted.
- [ ] Prefer larger measured compiled-step gains, simpler candidates for similar gains, then stable address/width ordering. Preserve the ranked alternatives, chosen slice, supporting counts and rationale so the choice is reproducible.
- [ ] Preserve the accepted predecessor and current user work. Allocate immutable selection evidence before preflight or profiling and record the baseline and source/input identities; selection does not mutate reviewed configuration or advance acceptance.
- [ ] Any call-target merge proposal remains isolated and subject to the same bounded selection procedure; discovered entries cannot silently enlarge the chosen slice.
- [ ] Distinguish useful supported work, a need for one new timed pattern, and no remaining covered candidate. Identify observed unsupported work for the pattern route; require independent domain validation before considering execution outside the accepted prefix.
- [ ] Verify ranking, overlap accounting, missing-profile handling and invalid baseline rejection through meaningful command/evidence cases, and retain a measured selection report for the local accepted domain.
