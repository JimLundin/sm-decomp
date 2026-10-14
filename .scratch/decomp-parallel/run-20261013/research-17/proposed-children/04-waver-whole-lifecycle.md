# Implement the admitted whole Waver lifecycle

Status: open
Blocked by: 11
Triage: needs-triage
Type: task

Proposal only; the maximum32-routine boundary is blocked for gameplay promotion. Parent allocates the final ticket/scope under milestones07/09 after `17-exercise-contract`, `17-enemy-transaction-adapter`, `17-death-pickup-admission`, reviewed13–15 contracts and natural Waver root observation. One isolated family owner receives exact new-file ownership.

## Work

Select native Waver header A0:D63F from room population/runtime allocation through active scheduling, terrain AI, timed native scripts, ordinary Samus/shot/bomb/power-bomb contact, freeze/death/grapple, normal OAM rendering and the death-projectile handoff. The exact32 definitions are:

`InitializeEnemies`, `LoadEnemyGfxIndexes`, `RecordEnemySpawnData`, `DetermineWhichEnemiesToProcess`, `SpawnEnemy`, `EnemyMain`, `EnemyCollisionHandler`, `EprojCollHandler`, `EnemyBombCollHandler`, `EnemySamusCollHandler`, `ProcessEnemyPowerBombInteraction`, `Enemy_NormalTouchAI_A3`, `NormalEnemyTouchAi`, `NormalEnemyTouchAiSkipDeathAnim`, `Enemy_NormalShotAI_A3`, `NormalEnemyShotAi`, `NormalEnemyShotAiSkipDeathAnim`, `Enemy_NormalPowerBombAI_A3`, `NormalEnemyPowerBombAi`, `NormalEnemyPowerBombAiSkipDeathAnim`, `Enemy_NormalFrozenAI_A3`, `Waver_Init`, `Waver_Main`, `Waver_Func_1`, `Waver_Instr_1`, `ProcessEnemyInstructions`, `DrawOneEnemy`, `WriteEnemyOams`, `DrawSpritemapWithBaseTile`, `EnemyDeathAnimation`, `Enemy_GrappleReact_KillEnemy_A3`, `EnemyGrappleDeath`.

Exact proposed new files: `crates/port/src/enemy_waver_lifecycle.rs`, `crates/port/src/enemy_waver_instructions.rs`, `crates/port/src/enemy_waver_admission.rs`, `crates/port/tests/enemy_waver_lifecycle.rs`, `tools/port/enemy-waver-lifecycle.toml`. Read-only dependencies are `enemy.rs`, `enemy_ai.rs`, `enemy_instructions.rs`, `enemy_pickup_ownership.rs`, `sprite_objects.rs`, `audio.rs` and reviewed player/room/PLM/game/hardware contracts. Integrator owns lib/memory/ROM/catalog/ABI registration. Held141/Ridley, Rio/jump/crawler and existing contact/render files remain with their owners.

Compose current Rust Init/Main/Func1 and movement leaves. Add native A386E3 signal through `cur_enemy_index` and the exact bounded bankA3 instruction-PC/timer domain. Preserve unusual `SignExtend8(0x8000)`, unaligned A/B negation, D byte/word phase and the retained native table/ROM behavior. Generic active-list initialization/scheduling and drawing callers receive whole-call attribution only when their entire selected domain and finalizers are admitted before any effect. Foreign/extended/multibox/tilemap domains refuse unchanged.

Retain direct roots and reviewed dynamic AI/opcode/host edges; bank86 callee hits cannot supply Waver provenance. An unobserved `SpawnEnemy` root remains a separately reviewed allocator dependency until direct observation or authenticated observed-root reachability supports inclusion. The32 list is a proposed maximum, not an admitted scope. Static original ROM/header/script/map identities are in [ROM observations](../rom-observations.toml); measured room population/tileset/terrain and native input extents must be added before promotion.

## Acceptance

Actual-C comparisons cover all admitted routine states, full128KiB WRAM/eight host words/raw/normalized returns/transient and ordered services. Independent controls include terminated/empty/mixed population, contiguous pool failure/partial resets, slot/list-tail/current aliases, D wrap/phase56, frozen/timer0/1/FFFF/Sleep-PC operands, first-contact shot0..4/bomb5..9 order, suit/SamusCode4/vulnerability/charged/ice/health semantics, EF89-before-clear/death/drop/respawn continuations, native palette/shake/queue/OAM cursor/extension-bit wrap and direct native Y cast.

Four fresh C0/C1/mixed0/mixed1 processes must independently traverse a genuine Waver window and show material initialization, AI, due opcode, contact, death/drop/pickup and OAM writes. Require whole-root support/ownership, explicit C/refused boundaries, complete periodic/terminal parity, ordered-effect equality and zero CPU fallback. Unobserved contact/lethal/drop/respawn branches stay unexercised. Preserve native137's exact independent full129712 material regression and held141 lineage; original-ROM controller/room/hardware progression credit still needs13–15.

Submission: `cargo test -p port --locked --offline --test enemy_waver_lifecycle --test enemy_ai --test enemy_instructions`, exact returned implementer worktree,60s. Actual-C integration: `cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/enemy-waver-lifecycle.toml --adapter enemy-lifecycle --output out/port/enemy-waver-lifecycle --case-jobs 4`, frozen `/home/jim/sm-decomp`,3600s. These proposed targets/manifests/adapter are not implemented and were not executed. [Answer](../Answer.md) supplies all-affected batch and future sealed live gate requirements; parent binds real baseline/verdict/frame/input paths before executing. Shared pipeline root/workers4 apply.

## Completion evidence

Pending supported native scope, isolated claim/commit, full source/ROM/input/host/build/reader/evidence seals, actual-C and independently observed live ownership, all affected families, original-ROM boundaries and explicit remaining C ownership. Both scoped cleanup passes, retained source/store/pointers/readers/private inputs, integrated worktree removal and post-cleanup authentication precede resolution. [Routine observations](../routine-observations.json) preserve exact definition hashes/addresses/dependencies; [children](../children.toml) records proposed ownership and gates.
