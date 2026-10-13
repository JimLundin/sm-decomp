# Admit bounded actual-C offline translation tasks

Type: task
Status: deferred
Blocked by: None
Triage: ready-for-agent

The current translation promoter requires an observed live projectile receipt
before assigning every translation. Add an explicit offline scope mode that
checks pinned source, coherent roots/dependencies, exact writable ownership and
real actual-C verification gates, without granting live acceptance. Preserve the
existing observed-live scope contract and all authority fencing.

Proposed ownership: crates/pipeline/src/translation.rs only. The coordinator owns shared
catalog/library wiring and the main Git/index. No storage or harness refactor.

Submission and merged checks: cargo test -p pipeline --locked --offline --bin
sm-pipeline translation::tests. Require nonzero tests, refusal controls for
missing actual-C gates, unrelated routines, malformed roots and forged live
exercise, and a regression for producer/consumer lexical call agreement.

## Comments

2026-10-13: essential assignment unblocker for user-authorized coverage work.
Integration tests are implementation/verification, not research.

2026-10-13T16:34Z: the maintained queue refused this write because retained
legacy affected-family-assignment-05 owns translation.rs. Preserve that hold and
defer the shared change. A narrow Rust offline coverage command can validate
source/dependencies and call the existing queue engine without changing shared
source or live admission.
