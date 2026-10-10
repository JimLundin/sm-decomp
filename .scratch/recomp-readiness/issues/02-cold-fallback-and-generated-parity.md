# 02: Run cold fallback and generated parity with complete evidence

**What to build:** One command builds and replays the current fallback-only or generated subject against the reference, retaining a complete identity-bound result for the requested scope.

**Blocked by:** 01: Capture immutable runs and verified preflight.

**Status:** done

- [x] Reuse the existing frame comparator and resumable generated-step runner; keep untimed whole-subroutine output separate from cycle evidence.
- [x] Cold readiness parity consumes exactly input frames 0–17999, seed 0 and P1 under the verified recording profile, comparing decoded CPU/clocks, complete WRAM and framebuffer.
- [x] Regenerate/build current generated inputs, preserve exact inputs and source/generated/binary digests, and reject stale or incompatible ROM/table identities.
- [x] Fallback and generated results are separate; generated validation requires a nonempty table and actual generated execution.
- [x] Report expected/completed frames, first frame disagreement, generated/interpreted totals, logs and generation/build/replay durations through the uniform evidence contract.
- [x] Truncation, budget exhaustion, interruption, timeout, setup/build failure and mutation cannot produce a successful validation result.



## Completion evidence

The public cold parity command builds isolated run-local generated inputs and
preserves generator/candidate binaries. It consumes real structured comparator
reports, verifies exact identities/scope, records generation/build/replay times,
and rejects short movies, empty/unexecuted tables, stale outputs and failed or
interrupted stages. The default validated cold scope is exactly 18,000 inputs;
full readiness exercises that final integrated source in ticket 12.

Validation: five public diff command tests and two comparator tests pass;
repository Python tests pass (full-domain local-material checks are explicit);
formatting, diff check and all-target Clippy for the affected crates pass.
A fresh cold fallback 2-frame integration command passed with preserved binary
and inputs. A fresh generated 300-frame integration command verifies actual
timed execution; its completion is recorded in the implementation handoff.
Two initial 18,000-frame attempts were interrupted during build and remain
visibly incomplete evidence, with no verdict or acceptance. These attempts and
short verification runs do not substitute for ticket 12's full current-domain gates.
