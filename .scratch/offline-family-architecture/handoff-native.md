# Native extraction handoff

Status: production library extraction compiles; caller/proof migration in progress
Owner: `/root/translation_architecture`, delegated by `/root`
Parent session: `01a13c3b-d421-7bb3-80a2-ec91045678e7`
Worktree: `/home/jim/sm-decomp-offline-native`
Branch: `architecture/offline-native-20261015`
Baseline: `446dd6418a9f91d4bd90163955b66c84a55ef9a9`

## Ownership

Own `offline-verification` Native preparation, opaque prepared state,
execution/admission, indexed operations and private Native tests. Own the
pipeline Translation scope adapter, Native proof tests and Native-specific
caller migration in `translation_execution.rs`, `translation_original.rs`,
`translation_original_prepare.rs`, `translation_original_tests.rs`,
`translation_migration.rs`, `translation_execution_tests.rs` and
`translation_managed_tests.rs`.

Root owns Cargo/lib scaffolding, ordinary verification, shared source stamps,
builds and ordinary callers. `/root/gameplay_architecture` owns common
`recipe.rs`/`recipe_commands.rs`; `/root/pipeline_architecture` owns foundation
follow-up fixes. Other worktrees, source and production claims remain intact.
No gameplay stage is claimed by this infrastructure extraction.

## Agreed interface

- `HostInputs` contains trusted compiled host-source stamps and producer hash;
  `OfflineFamilies` owns it. `inputs::with_host` scopes its use without a
  global authority cache; `inputs::required_sources` adds all three lower owners.
- Native declaration/preparation calls use that context. `PreparedNative`
  keeps all fields private, preserves current serialized records and exposes
  immutable contract/source/identity/resource summaries.
- Verification takes a target, confined environment and trusted host.
  Authentication uses retained prepared/receipt data and source paths without
  current host checks or producer launches. `AuthenticatedNative` represents
  both passed and rejected outcomes with report/reference getters.
- `ScopeAdmission` is caller-owned; pipeline retains `validate_scope` and
  ticket/assignment facts. It is called only during current provenance checks,
  never during completed replay or retained admission.
- Common recipe code uses explicit model values, with no Native-owned type or
  operation traversal. Current serde tags, compiler/Fixture limits, coverage,
  indexed attribution and retry/ownership semantics remain unchanged.

## Verification and cleanup

Use `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`,
`SM_PIPELINE_WORKERS=4`, locked/offline Cargo, isolated build output and one
Cargo build job. Root owns the final real Native proofs after integration;
the audited commands and complete recipe matrix are in the main checkout's
`native-verification-plan.md` beside this handoff.

The implementation library check passed with `cargo check --locked --offline
-p offline-verification`, an isolated `out/native-target`, one build job and the
shared four-worker pool. Its log is
`native-verification/check-library.log`. This check was not frozen-source
verification; tests and the new application proof file still need migration.
No Native producers have run. Cleanup and worktree retirement remain pending
implementation/verification. Parent owns integrated worktree retirement after
exact required source is retained.
