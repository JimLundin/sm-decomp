# Serial integration recovery fix

Status: source complete; root owns frozen engine build, actual drill and final cleanup
Blocked by: None

Preparation recovery assignment:
[recovery-serial-20261013.md](recovery-serial-20261013.md).
Worker display identity: `/root/serial_integration_fix`; actual thread identity
is recorded in [serial-fix-evidence/worker-identity.toml](serial-fix-evidence/worker-identity.toml).
Root retains the main Git/index and preparation integration ownership. This
worker changed only its isolated worktree and its assigned preparation evidence.

Source branch: `decomp-serial-integration-20261013`.
Worktree: `/tmp/sm-decomp-serial-integration-20261013`.
Baseline: `f54fbed0f30f2c84e2c6aba171dcc54e96099c9a`.
Committed source: `2d59536c059a1ca9c31d9278bff2d9222eafa56c`.
Exact changed files: `crates/pipeline/src/session.rs` and
`crates/pipeline/tests/decomp_sessions.rs`; isolated Git status is clean.

## Diagnosis and change

The real InfrastructureRecord → generated integration Start regression ran red
twice. Two disjoint reviewed docs scopes share a baseline; lane A is submitted,
merged and durably integrated, then lane B is submitted from that same baseline.
Lane B Start refused `change exceeds approved files: docs/drill-a.md` in 1.98 s
and 1.76 s. Evidence is retained in `red.log` and `red-repeat.log`.

Ranked falsifiable hypotheses shared before inspection were: (1) scoping
baseline..shared HEAD as the current integration's ownership; (2) stale generated
submission lineage; (3) runtime or dirty-source contamination. The first was
confirmed: `validate_handoff` applied the implementation ownership rule to shared
source. The rejected path was committed lane A docs, and the shared source was
clean. Generated lane B id/revision/attempt remained exact.

Integration handoffs now scope baseline..the exact held submitted commit. They
require the shared checkout, current exact submitted id/revision/attempt,
matching baseline/writable files, a matching `submitted_commit`, and frozen clean
shared source at the separately checked `source_commit`. Implementation handoff
scoping remains strict against baseline..HEAD.

Controls validate prior disjoint merge plus an unrelated clean shared commit,
exact lane B before merge, repeated Start, Resume before merge, stale Resume
after merge, republished merged source Resume, wrong and missing submitted
commit, staged/unstaged/untracked foreign shared source, and committed foreign
source in an implementation worktree. Both lanes finish through native
infrastructure integration receipts; all four jobs become complete.

The initial regression compile had a fixture String/PathBuf mismatch, retained
as `initial-build-type-error.log`. The first fixed-engine attempt exposed a
dirty-control restore issue; its diagnostic logs are retained separately. The
control now explicitly restores `HEAD` and asserts source is clean after each
probe. There is no temporary engine instrumentation.

## Completion evidence

All commands ran in the isolated worktree with
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`,
`CARGO_TARGET_DIR=/tmp/sm-serial-fix-target-20261013` and the reviewed dependency
path override:

```sh
cargo --config 'patch."https://github.com/doublegate/RustySNES".rustysnes-core.path="/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core"' test -p pipeline --locked --offline --test decomp_sessions serial_integration_start_accepts_prior_disjoint_merge_and_keeps_source_frozen -- --exact --nocapture
cargo --config 'patch."https://github.com/doublegate/RustySNES".rustysnes-core.path="/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core"' test -p pipeline --locked --offline --test decomp_sessions
cargo --config 'patch."https://github.com/doublegate/RustySNES".rustysnes-core.path="/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core"' test -p pipeline --locked --offline --test assignment_queue --test session_authority
rustfmt --edition 2024 --check crates/pipeline/src/session.rs crates/pipeline/tests/decomp_sessions.rs
git diff --check
```

The focused green regression passed in 2.04 s (`green.log`, also retained as
`control-diagnosis-2.log`). The
complete session suite passed all 17 tests in 4.38 s (`session-suite.log`).
Adjacent suites passed 23 assignment tests and 11 authority tests with one
existing ignored process helper (`adjacent-suite.log`). Owned file format and
Git diff checks passed. Source SHA-256:

- `session.rs`: `79cf28914567750d885b68698056b9dca25582d563ebc8ace49faef00e50a61d`
- `decomp_sessions.rs`: `a7c6b8cdf574c2c2aec703324f75ac9969a43fac34e685b2267868fcf356e078`

All cargo/test writer processes finished before cleanup. The disposable
integration fixtures remove their roots/worktrees after tests; no
`/tmp/sm-session-test-*` roots remain. Root requested retaining the private target
and implementation worktree for the final engine build/freeze, so these remain
root-owned cleanup actions after integration. The isolated target's debug
`sm-pipeline` exists, but final build identity belongs to root's frozen build.

Both finished evidence cleanup passes succeeded and reclaimed zero bytes:

```sh
/tmp/sm-serial-fix-target-20261013/debug/sm-pipeline cleanup --root /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/serial-fix-evidence --apply
/tmp/sm-serial-fix-target-20261013/debug/sm-pipeline cleanup --root /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/serial-fix-evidence --deduplicate --apply
```

Cleanup receipts are `cleanup-pass1.toml` and `cleanup-pass2.toml`; remaining
filesystem space observed after cleanup was `220749799424` bytes. Retained
logs/receipts are sealed by `artifacts.sha256`; source is preserved in the exact
commit. No gameplay stage or production authority was changed.

## Next action

Root builds and freezes the engine at the returned source commit, then uses
fresh actual independent sessions for the new serial drill and readiness
validation. Do not attribute old drill evidence to the corrected source. After
root retains the final build/source identity and integrates source, root removes
this implementation worktree and disposable private target under run cleanup.
