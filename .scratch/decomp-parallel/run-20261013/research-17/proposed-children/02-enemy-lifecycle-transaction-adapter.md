# Add complete synchronous enemy lifecycle transactions

Status: open
Blocked by: 11
Triage: needs-triage
Type: task

Proposal only. Parent assigns its final ticket number and reviewed infrastructure ownership under milestones07/09. Dependencies include `17-exercise-contract`, reviewed player services14 and room/PLM services15. Shared hooks remain integrator owned.

## Work

Proposed new files are `crates/pipeline/src/enemy_lifecycle_adapter.rs`, `crates/pipeline/src/enemy_lifecycle_model.rs`, `crates/pipeline/src/enemy_lifecycle_host.c.in`, `crates/pipeline/src/enemy_lifecycle_abi.rs.in` and `crates/pipeline/tests/enemy_lifecycle_adapter.rs`. Integrator hook files are `crates/pipeline/src/family_model.rs`, `crates/pipeline/src/family.rs`, `crates/pipeline/src/inputs.rs`, `crates/pipeline/src/live.rs`, `crates/pipeline/src/live_model.rs`, `crates/pipeline/src/main.rs` and `tools/port/batch.toml`. Review exact hunks after the frozen host batch; the paths are a proposal rather than a grant.

Provide an `enemy-lifecycle` fixture/live adapter that carries full128KiB canonical WRAM, actual eight host words, raw/normalized pointer/word returns and ordered effects. Preserve enemy/SpawnData/extended overlay, byte-slot/list-tail/current-index aliases and native partial resets. Actual `SpawnEnemy`, normal death-projectile, `Samus_DealDamage`, `CallSomeSamusCode(4)`, PLM, audio and sprite services execute synchronously on the latest state and return complete copyback. Record native allocation/recursive initializer/RNG/queue/result footprints before accepting the host continuation.

Immutable whole-call preflight refuses missing or foreign continuation domains before effects. Accepted Rust or host errors are fatal and retain evidence; C retry after prior writes is forbidden. Ridley's existing foreign SpawnEnemy/item-drop admission deliberately refuses nested continuations. Waver uses `EnemyDeathAnimation`/F345 and has no selected boss-specific `Enemy_ItemDrop_*` service; preserve that zero/outside-domain boundary and held141. Reuse reviewed player/PLM and native137 services with their genuine state contracts.

## Acceptance

Source-bound compiled actual-C controls compare complete pre/post state, all host words, raw/normalized returns and ordered services, including nested allocation/initialization, SamusCode4-before-damage, actual PLM writes and queue effects. Independent unsupported-domain/first-service/mid-service/failure controls prove unchanged refusal and fatal accepted failure without C retry. Existing consumers and native137 allocator/scheduler/contact/render admission retain their source/evidence lineage.

Submission command is `cargo test -p pipeline --locked --offline --test enemy_lifecycle_adapter` at the exact returned isolated worktree, bounded60s. Frozen integration command is `cargo test -p pipeline --locked --offline --test enemy_lifecycle_adapter --test callback --test family --test live` at `/home/jim/sm-decomp`, bounded3600s. These targets and the adapter are proposed and unimplemented; research17 ran neither command. Use shared process workers4 and the shared pipeline root.

## Completion evidence

Pending reviewed infrastructure scope and service-domain admission, semantic/refusal/failure controls, frozen source/build/reader/evidence seals, all affected consumer regression, both scoped cleanup passes, retired implementer retention and authentication. [Answer](../Answer.md) records source/evidence hashes and exact ownership; [children](../children.toml) records proposed dependencies. Infrastructure acceptance earns no gameplay stage.
