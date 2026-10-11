# Native readiness and decomp gates

Status: resolved
Blocked by: 01

Replace decomp acceptance/jobs, readiness source snapshots and gates, instruction
pattern evidence, parity/reuse, iteration selection and session protocols with
Rust. Port meaningful adversarial tests without weakening origin, contamination,
fallback or acceptance checks. No Rust command may invoke legacy Python.

## Completion evidence

Native decomp/RNG, feedback9, session/selection and refusal controls passed;
see [completion.md](../evidence/completion.md) for final cleanup and limits.
Full gameplay readiness/new optimization increments are outside the user-narrowed
migration scope. This supersedes pending verification plans below.

Native decompressor acceptance passed 40 synthetic cases, 23 captured calls,
3 bounded original-ROM cases and 11 executed rejection controls. Native reset
epoch live verification passed four fresh 9000-frame subjects, two epochs each,
complete state/effects equality, 23 decompressor calls per epoch and actual
invalid-source exit-4 control. Verdict:
`out/port/rust-migration/native-routine-live/1791733818275298105-2/verdict.json`.
Native job admission controls passed. Native RNG job accepted at
`out/port/rust-migration/native-rng-job/1791734426542005268-2/verdict.json`:
all 65,536 seeds, 16 captured complete-WRAM fixtures, 8,783 ordered gameplay
effects, four subjects/two reset epochs, exclusive Rust RNG in mixed subjects.
Legacy RNG/job scripts and their transferred admission test were removed.
Readiness/iteration ports and final cleanup remain pending.

Native bounded proposal and session orchestration implemented. Session-v2
reconstructs count/request/lineage from ordered TOML events, recovers durable
events before state replacement, authenticates published candidates before
counting, and retains a validation intent before configuration mutation. An
unresolved increment keeps its original predecessor. Pointer refresh is bound
to exact prior bytes and rejects a concurrent replacement under publication lock.
Four native session controls passed; current pipeline unit suite passed 43/43
(`/tmp/sm-native-current-unit.log`). Iteration diagnosis adds five authenticated
history/novelty/three-attempt controls; native model and diff tests pass after
moving instruction profiles and generated key inventories to TOML. Native
pipeline-model has no JSON dependency or reader. Full acceptance/progress
production controls and remaining Python deletion are still required.

Earlier full pipeline regression passed unit, baseline, callback, cleanup,
decomp acceptance, family, live, observations, packed and parity suites, then
stopped at a pattern test contaminated by edits in the active checkout. That
test now owns a private source checkout; rerun is in progress at
`/tmp/sm-native-pattern-private-regression.log`.

Cleanup this stage: disposable Cargo targets reclaimed 2.9 GiB and 648.9 MiB;
six completed run builds reclaimed 736,532,508 bytes. Retained feedback 9/9,
independent sync 390 frames/47 events, and complete pattern evidence all
reauthenticated afterward. Further current-run cleanup remains pending.
