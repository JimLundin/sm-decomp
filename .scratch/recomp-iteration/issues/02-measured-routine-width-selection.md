# 02: Select a measured routine/width increment

**What to build:** Given the current accepted state, produce a reproducible choice of one covered routine/width slice that is likely to move useful interpreted CPU work into generated execution. Explain the choice with measured instruction evidence, or report why no useful supported candidate remains.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Verify the accepted run, retained source/config identities and validated domain before using its evidence. When acceptance is absent or invalid for the proposed change, establish the necessary readiness gates before treating a baseline as usable.
- [x] Use complete, identity-bound interpreted instruction counts by address/MX and opcode/pattern within the validated prefix. Obtain a matching instruction profile if the accepted evidence lacks one; call-target misses remain discovery input rather than instruction counts.
- [x] Identify bounded routine/width candidates using current timed support and observed execution. Estimate gain from newly compilable executed keys, accounting for overlapping generated work, and allow a routine to remain partly interpreted.
- [x] Prefer larger measured compiled-step gains, simpler candidates for similar gains, then stable address/width ordering. Preserve the ranked alternatives, chosen slice, supporting counts and rationale so the choice is reproducible.
- [x] Preserve the accepted predecessor and current user work. Allocate immutable selection evidence before preflight or profiling and record the baseline and source/input identities; selection does not mutate reviewed configuration or advance acceptance.
- [x] Any call-target merge proposal remains isolated and subject to the same bounded selection procedure; discovered entries cannot silently enlarge the chosen slice.
- [x] Distinguish useful supported work, a need for one new timed pattern, and no remaining covered candidate. Identify observed unsupported work for the pattern route; require independent domain validation before considering execution outside the accepted prefix.
- [x] Verify ranking, overlap accounting, missing-profile handling and invalid baseline rejection through meaningful command/evidence cases, and retain a measured selection report for the local accepted domain.


## Completion evidence

Implemented `just select-iteration` and a production CFG-backed selection command.
Selection binds a complete interpreted instruction profile to verified accepted
source/input/executable identities and preserves configuration and acceptance.
Exact newly compilable keys, overlap, deferred inventory, stable ranking and
unsupported pattern alternatives are retained in immutable reports. Invalid
baseline/setup results explicitly require readiness refresh; missing-count
recovery uses the preserved accepted executable with exact replay identity.

Public command/evidence tests passed for overlap, equal-gain simplicity, uncovered
and unsupported routes, invalid baseline preservation, missing-profile refresh,
noninstruction dispatch accounting and invalid counts/identities/result rejection.
`cargo test -p recomp --locked --offline`, targeted Python tests, recomp Clippy,
format and diff checks passed.

Measured local domain report:
`/home/jim/sm-recomp-iteration-02/out/runs/20261010T122316.916691Z-471cf0dd8bbe/selection.json`.
Against accepted run `20261010T112635.727097Z-6bf369628146`, it chooses `$80B119 m0x0`
with 908,233 estimated newly compiled instructions across 12 observed keys;
763 ranked alternatives, 228 rejected/uncovered roots, zero chosen overlaps or
deferred keys. Accepted evidence and pointer remain unchanged. Final strengthened
result verifier independently passed this preserved measured report.
