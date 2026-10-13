# Typed integration handoff regression

Status: WIP committed; awaiting exact assignment dependency fix for gate.
Owner thread: `01a13474-5fcc-7861-84f8-5c353938bdd2`.
Baseline: `11f13b4` integration source.
Branch: `decomp-preparation-integration-handoff-20261013`.
Worktree: `/tmp/sm-decomp-preparation-integration-handoff-20261013`.
Writable file: `crates/pipeline/tests/decomp_sessions.rs` only.
WIP: `05a165a326fa7e58ba84e7e0b3cec633d9804c9c`.

The existing full infrastructure lifecycle now starts generated integration
through typed session Start, retries the identical Start, resumes its durable
handoff and checks returned exact claim/source/next command before source merge.
It asserts the auto-generated job retains full base-head. The root's validator
worker owns that narrow assignment source seam; any dependency copied here for
testing remains unstaged and is restored before handoff. Shared workers4/private
build root apply. No gate, run outputs or cleanup obligations exist yet.

Baseline focused gate launched (exec25973) from assigned worktree, private target
`/tmp/sm-decomp-preparation-integration-handoff-target-20261013`. Command:
`cargo test -p pipeline --test decomp_sessions infrastructure_cycle_integrates_frozen_source_after_retiring_worker --locked --offline` with only the retained
RustySNES declared patch override, shared pipeline root/workers4. Expected
baseline missing-base-head failure will be retained as diagnostic evidence;
then repeat against the validator worker's exact narrow dependency source.
Logs `out/integration-handoff-verification/baseline-test.stdout/stderr`.
No gameplay or production mutation. Scoped WIP05a165a precedes gate.

Baseline control completed with the expected concrete failure: generated
integration has no `base-head`, at test line611 before typed Start. No queue
claim was released; all fixture writes are disposable and their Drop removed
fixture worktree/repository. Retained baseline log will authenticate the fix's
trigger. Waiting exact source dependency from `/root/subsystem_catalog`.

Validator worker provided stable narrow uncommitted assignment source; exact
copied SHA-256 `922aae9fc20ab8467e3267bbdbe03f049467a72cda580946e4e53dc9bdd49e4e`.
It copies original full baseline to generated integration input and falls back
to exact submitted commit for older native jobs without baseline. Foreign file
is unstaged in this test worktree. Full15 decomp_sessions gate startedexec13004,
logs `fixed-test.stdout/stderr` in own retained root. Next: passing gate ->
scoped final test commit -> both cleanup passes -> remove private target and
restore copied dependency -> root integrates test/dependency together.

Completed: all15 focused decomp_sessions tests passed using exact narrow foreign
dependency SHA above. Full cycle passes typed integration Start, identical Start
retry, durable Resume, source merge/worktree retirement, Integrated result and
retired submission retry. Final owned test commit
`a3384094c1e5259b9f789e3afb3334c918d44877` (comment-only final step after gate);
code-tested WIP `05a165a326fa7e58ba84e7e0b3cec633d9804c9c`. Own Git clean;
foreign assignment restored to own baseline. Both project cleanup passes on
selected receipts succeeded0 bytes. Private target removed1,218,133,244 bytes;
free disk220,778,647,552 bytes. Post-cleanup manifest authentication passes.
Logs/evidence retained at worktree `out/integration-handoff-verification`;
aggregate `artifacts.sha256` SHA
`a3c474c2f3b08817dfd96a4bb8b30e5246145ba20554293209c459eba2a587aa`.
Root preserves these small logs and exact source dependency before integrated
worktree removal, then integrates owning validator source/test commit together.
