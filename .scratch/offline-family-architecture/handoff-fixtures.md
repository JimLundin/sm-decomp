# Shared Fixture command execution

Status: implemented; library compilation passed. Root owns integration and the
combined runtime verification.

- Owner: `/root/gameplay_architecture`
- Worktree: `/home/jim/sm-decomp-offline-recipes`
- Branch: `architecture/offline-fixtures-20261015`
- Baseline: `996ed62`
- Prior branch `architecture/offline-recipes-20261015` remains at `3166662`.

## Changes

`recipe::fixture` now constructs every case command through
`recipe::fixture_command`, using the selected recipe, actual manifest, routine,
Fixture and scheduler index. Its execution closure sends the same description
to either the ordinary worker or `StrictExecution::fixture_recipe` with the
description's `FixtureKind`. Native receipt reconstruction already consumes the
same command builder.

The family, callback, PLM and Ridley adapters now each expose one crate-private
case function. Each owns its state preparation, subject order, comparison and
adapter artifacts. Their execution callback receives only the prepared case
directory and subject label. Duplicate ordinary/confined wrappers and raw family
and Ridley argv construction were removed.

The callback and PLM raw command bodies moved into `recipe_commands.rs`; that
module no longer calls adapter helpers. Callback retains its exact case index
position. PLM retains allocator values 0/1/2 and rejects unknown allocators.

Callback snapshot unpacking stays in the callback adapter and occurs only when
the case uses confined execution. It follows the execution closure's successful
return, after `StrictExecution::fixture_recipe` has authenticated the producer
result. It still occurs before advancing from Rust to C and before comparison.
Preparation, captured/continuation inputs, subject order and retry behavior are
otherwise unchanged.

Removed the unused request field from `FixtureExecution::Ordinary` and updated
its sole ordinary call site in `family::verify`, as authorized by root. Native's
`FixtureExecution::Confined` interface is unchanged.

Scope is the case/command portions of six Offline files: `family.rs`,
`callback.rs`, `plm_spawn.rs`, `ridley_subsystem_adapter.rs`, `recipe.rs` and
`recipe_commands.rs`. No builds, library, input, Native or pipeline file changed.

## Integrator follow-up

`builds::StrictExecution::fixture` now has no caller and can be removed by its
owner. All confined case and ABI execution uses `fixture_recipe` with an
explicit common kind. That cleanup was coordinated with root and is deliberately
outside this commit.

This completes the command-helper migration noted in `handoff-recipes.md`:
callback/PLM commands now live entirely in the recipe owner, so the previous
warning about recursive adapter delegation no longer applies.

## Completion evidence

- `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 cargo
  check --locked --offline -p offline-verification --lib -j 4` passed in 19.98 s.
  Remaining diagnostics are unused imports/dead code while Native integration
  and root cleanup are pending; no type errors occurred.
- `rustfmt --edition 2024 --check` passed on all six changed Rust files.
- `git diff --check` passed.
- Source review confirms unchanged callback/PLM argv fields and ordering and
  unchanged family/Ridley preparation/comparison bodies. Every case now uses
  the same command owner as receipt reconstruction. No actual-C or ignored
  Native proof run was started, and no runtime proof is claimed here.
- After compilation exited, `du -sb target` measured 270,134,723 bytes.
  `cargo clean --locked --offline --target-dir
  /home/jim/sm-decomp-offline-recipes/target` removed 1,109 files (259.7 MiB
  reported). `unlink deps` removed the temporary link to main's dependency tree.
- The compiler exited; this work created no run directories or retained evidence
  dependencies. `df -B1` reported 156,751,196,160 bytes available after cleanup.
  Root removes the implementer worktree after integration.
