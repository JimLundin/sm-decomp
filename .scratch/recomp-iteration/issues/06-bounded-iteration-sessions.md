# 06: Run bounded iteration sessions with progress briefs

**What to build:** An explicit iteration request carries selection, candidate changes, validation and diagnosis through a requested count or target, defaulting to one accepted increment. Each acceptance becomes the verified baseline for the next increment, and completion or a required decision produces a short linked progress brief.

**Blocked by:** 03: Prepare and accept one supported routine/width increment; 04: Validate and accept one new timed instruction pattern; 05: Carry diagnosis through successive candidate retries.

**Status:** implemented; full real session demonstration pending ticket 07

- [x] Surface a repository-root iteration entry point that honors an explicit increment count or target and defaults to one accepted increment when unspecified. Readiness requests finish readiness without starting an unbounded iteration session.
- [x] Verify the accepted baseline before each selection, establish necessary readiness gates when absent or invalid, and dispatch the measured supported-slice or new-pattern route. Handle no remaining covered candidate with a specific result; independently validate any proposed domain extension first.
- [x] Own routine selection, candidate changes, checks, controls, repairs and retries within the agreed scope. Preserve user work, accepted predecessors, rejected candidates and immutable evidence; no per-change human checkpoint is required for local acceptance.
- [x] Count an increment only after mandatory validation and successful atomic acceptance. Failed, uncovered, incomplete, errored, unchanged or zero-gain expansion attempts cannot satisfy the requested count; an explicitly identified correctness fix may have zero gain with demonstrated repair and complete acceptance evidence.
- [ ] Continue from each newly verified accepted state only while requested scope remains. Honor user stop, missing required inputs, decisions beyond the design and the three-experiment escalation rule, reporting accepted/requested progress and preserved candidate state.
- [x] Before/after metrics use identical validated inputs/domain and report compiled/interpreted totals and fraction, per-key counts, validated distinct keys, declared pattern coverage, deferred/unsupported inventory and comparable deltas. Record evidence reuse and refreshed gates explicitly.
- [x] Report warm focused replay and per-cold-replay targets of 60 seconds and 20 minutes, with build/cache/corpus/test/total-suite durations separate. Investigate/report completed overruns; actual timeouts remain incomplete and cannot advance acceptance.
- [x] Write a concise session brief with outcome, accepted/requested count or target progress, selected changes/rationale, domain, metrics, reused evidence, timings and debt, linking source diffs, verdicts, coverage and diagnostics. Failures include earliest disagreement, control conclusions, recent experiments, next choices and candidate/predecessor IDs.
- [x] Verify default-one behavior, count/target stopping, failed acceptance handling and continuation from the current accepted lineage through meaningful session fixtures. Commit, external publication and merge remain separately authorized actions.


## Completion evidence

Implemented `just recomp-iteration` with default one accepted increment, explicit
positive count or measured generated-instruction target, supported-first measured
selection and conditional independently proved pattern promotion. Production
operations use real readiness/selection/preparation/validation APIs; fixture runner
is available only at the in-process session operation seam. Baseline refresh and
invalid-pointer recovery preserve exact rejected bytes and enforce acceptance-lock
race checks with no reuse or lineage from corruption. Own publication/current
pointer/origins are reverified before incrementing count.

Resume retains accepted/requested counts and stable unresolved increment history.
Semantic repairs get new exact-source validation; only safe new-key pruning is
automatic. Stop preserves current process logs and prevents later gates. Child
validation/acceptance require sealed bounded preparation, exact config/table and
positive comparable progress. Briefs include dynamic pattern scope, per-key and
legacy/deferred/unsupported inventories, reuse/timing, diagnosis and actual unified
source diffs against the preserved predecessor.

Six session fixtures passed for default/count/target stopping, own lineage,
failed publication, same-increment resume, no-count stop, invalid CLI and exact
corrupt-pointer/race preservation. Nine readiness tests including real SIGINT
process interruption, five orchestration tests and five increment tests including
prepared-evidence tamper rejection passed. Real multi-gate 18,000-frame session
acceptance remains the final integration demonstration in ticket 07; none was
claimed or published from this worktree.
