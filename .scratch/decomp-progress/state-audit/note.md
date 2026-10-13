# Source implementation coverage audit

Status: resolved
Triage: ready-for-agent
Blocked by: None

The audit enumerates owned executable source correspondences across all 45 files in `crates/port/src/`, rather than treating fixture roots as the full implementation. It does not establish differential equivalence, live exercise, acceptance, or complete native-engine replacement. Those are separate evidence stages and require authenticated records with matching source identities.

## Scope and counts

- 45 Rust files, 19629 source lines, with the SHA256 of every current source file recorded.
- 177 maintained `tools/port/*.toml` manifests, including canonical array tables and older quoted inline routine tables; manifest hashes and root lists are recorded. The source inventory is handled separately.
- 1453 distinct mapped native definitions out of 6053 (24.00%), representing 16454 native body lines out of 75,229 (21.87%). These are body-line size proxies, not proven branch coverage or equivalent translated line counts.
- 835 exact native manifest root names; all have an owned source mapping. 618 mapped native definitions have no explicit maintained manifest root, including private helpers and numeric/dynamic dispatch implementations. Manifest declarations alone establish no stage evidence.
- 1479 per-module mapping entries collapse to 1453 unique definitions; repeated implementations/specializations are deduplicated by native symbol.
- 329 unresolved reference records, which include known forwarding and support references as well as genuine uncertainty. They are not 329 distinct unimplemented routines. Remaining unaliased Rust helpers are retained for review; many are accessors, validation, host adapters, or bodies already covered by named selectors.

The pinned native inventory has 6,053 lexical `sm_*.c` definitions. Its revision is `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, archive SHA256 is `69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`, source identity SHA256 is `e74864c229949fc6f1a6bd7528f339bdb3f9364f1e4f9544f08693881d10f111`, and inventory SHA256 is `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.

## Method and confidence

A std-only Rust extractor tokenizes executable functions, skipping comments, and records native symbol selectors only in dispatch/execute/render/select/instruction/callback functions. Reviewed numeric scopes restrict bank/address extraction to executable selector arms, so script tables, admission predicates, support lists and unrelated constants do not count as bodies. Each numeric address must match an exact lexical native definition. Explicit reviewed aliases connect renamed/inlined helpers to native definitions after comparing the implementations; alias records include Rust function/line and native definition address/hash. Dynamic selectors cover only implemented audio limits, Rio Func 1 through 7 and validated projectile direction suffixes. The map records guards, specialization and delegated effects alongside these correspondences.

Known forwarding wrappers are removed from duplicate owner counts. `lib`, `memory`, `game_host`, `observed_projectile_ownership`, and `ridley_subsystem_admission` contribute zero owned native bodies. The latter two select ownership and forward or dry-run calls; admission is not another gameplay translation. Other modules can own real bounded adapters: `ProcessEnemyInstructions` mappings specialize the admitted generic or Ridley script sets, and `PlmHandler_Async` is a bounded synchronous frame adapter, not proof of full coroutine equivalence. Positive-count A6 palette copies, A6/B4 map specializations, script budgets, slot bounds and terrain-chain rejection remain binding.

The alias review corrected `ceres_palette` to `Ridley_Func_61`, `update_tangible` to `Ridley_Func_39`, and `SpawnEnemyDrops` to the actual `spawn_enemy_drops` wrapper. A dubious `Ridley_Func_1` alias was removed. Bank94 shot helpers use reversed native Horiz/Vert names; the map follows their actual axis behavior. The byte-product SBA sine helper is not counted as the hardware Mode7 `ComputeSinMult`/`ComputeCosMult` routines. Macro-only no-op addresses, including bank86:950C (`fnnullsub_85`), A6:AA4F/C600/BF19 and several bank86 preinstructions, have no inventory body and remain explicit unresolved/noninventory records. Synthetic seams such as `EprojInitializer`, `EprojInstruction`, `EprojPreinstruction`, `PortDrawProjectileSpritemap` and display adapters are never added to the native denominator; their actual executable branches/roots are mapped independently.

## All modules

The counts below are per module and can overlap. “Manifest roots” means mapped symbols explicitly named in any current maintained manifest, not evidence of a passed fixture. The JSON contains every symbol, definition hash, proof pointer, scope, limitation, unresolved literal, synthetic selector and unaliased helper.

| Rust module | Owned native mappings | Manifest roots | Role | Implemented scope |
|---|---:|---:|---|---|
| `animated_tiles` | 66 | 63 | scheduled gameplay | Bank87 animated tiles and bank8D palette allocation, supported scripts and queues |
| `audio` | 21 | 21 | gameplay helper | Music and SFX queue state, delays and per-channel limits |
| `camera_scroll` | 4 | 4 | scheduled gameplay | Bank90 camera closure with supported finished hook |
| `collision` | 98 | 63 | gameplay subsystem | Samus terrain, solid enemies, movement-type handlers and bounded PLM collision closure |
| `enemy` | 32 | 32 | gameplay helper | A0 geometry, angles, distance, coordinate and enemy variable helpers |
| `enemy_ai` | 29 | 17 | gameplay subsystem | Ripper, Waver, Rio paths and A0 terrain movement |
| `enemy_crawler` | 17 | 17 | gameplay subsystem | A3 crawler allocation/state, slope alignment and terrain movement composition |
| `enemy_instructions` | 15 | 15 | bounded script adapter | A0 generic bytecode operations and a budgeted generic script pass |
| `enemy_jump` | 25 | 25 | gameplay subsystem | A3 Sidehopper and Desgeega floor/ceiling state machines |
| `enemy_projectile_collision` | 14 | 2 | gameplay subsystem | 86 enemy projectile terrain movement and square/non-square slope tests |
| `enemy_projectile_contacts` | 6 | 4 | gameplay subsystem | A0 enemy projectiles touching Samus and player projectiles |
| `enemy_projectile_instructions` | 211 | 54 | scheduled gameplay | 86 projectile timer/script scheduler, implemented instructions, preinstructions and motion helpers |
| `enemy_projectile_rendering` | 6 | 2 | scheduled rendering | 86 projectile priority/OAM passes and81 spritemap emission |
| `enemy_projectile_sparks` | 2 | 1 | gameplay subsystem | 86 F391/F3F0 sparks initialization, gravity, bounce and trails |
| `enemy_projectiles` | 105 | 11 | gameplay subsystem | 86 allocation, graphics wrappers, implemented initializer branches and host dispatch |
| `enemy_rio` | 18 | 18 | gameplay subsystem | A2 Norfair and LowerNorfair Rio Init/Main/Func 1 through 7 selectors |
| `game_host` | 0 | 0 | shared host forwarding | Typed synchronous host bridge for seven existing scheduled Samus projectile and room-scroll roots |
| `glass_shattering` | 5 | 3 | gameplay subsystem | 84 tank break spawn calls and86 shard/sparkle lifecycle |
| `lib` | 0 | 0 | infrastructure | Public module registry and canonical WRAM exports |
| `memory` | 0 | 0 | infrastructure | Canonical128KiB WRAM view and byte/word access |
| `mother_brain_bomb` | 5 | 3 | gameplay subsystem | 86 C482/C4C8/C564/C5C2 bomb spawn, motion, overlap and effects |
| `n00b_tube` | 13 | 0 | gameplay subsystem | 86 D5E1 throughD8DF tube cracks, fragments and bubbles |
| `observed_projectile_ownership` | 0 | 0 | admission and forwarding | Immutable admission for nine replay-observed86 definitions and maintained dispatch delegation |
| `plm` | 103 | 96 | gameplay subsystem | 84 PLM allocation, supported setup callbacks and preinstructions |
| `plm_instructions` | 55 | 1 | bounded script adapter | 84 supported bytecode, draw-data writes and synchronous frame handling |
| `plm_render` | 2 | 0 | gameplay rendering | 84 PLM block coordinates, level draw data and tilemap/VRAM queue preparation |
| `power_bomb` | 5 | 4 | gameplay subsystem | Power-bomb gameplay stage/lifetime and PLM/bomb effects |
| `projectile_flight` | 55 | 42 | gameplay subsystem | 90/94 normal beam/wave/missile flight, firing, terrain and trail allocation |
| `projectile_special` | 33 | 30 | gameplay subsystem | 90/94 bombs, spreading attacks, SBA phases and bomb terrain |
| `projectile_trails` | 4 | 4 | scheduled rendering | 90 eighteen-slot trail script update and OAM emission |
| `projectiles` | 34 | 34 | gameplay helper | 90/93 Samus projectile data, animation, lifetime and inherited velocity |
| `ridley_cutscene` | 67 | 5 | gameplay subsystem | A6 Ceres initialization/appearance/getaway, escape text/transfers and Ridley death states |
| `ridley_enemy_instructions` | 9 | 9 | bounded script adapter | A6 alternate Ridley attack script frame and aiming helper |
| `ridley_movement` | 46 | 1 | gameplay subsystem | A6 fixed-point seek/arc/integration, wings and seven-segment tail states |
| `ridley_rendering` | 29 | 1 | gameplay rendering | A6 Ridley/baby maps, palettes, script frames and VRAM/Mode7 queues |
| `ridley_subsystem` | 62 | 9 | gameplay subsystem | A6 scheduled Ridley AI, state selection, contact and carry orchestration |
| `ridley_subsystem_admission` | 0 | 0 | admission | Immutable Ridley native entry/state/ROM validation and private dry-run host |
| `ridley_subsystem_instructions` | 20 | 19 | bounded script adapter | A6 Ridley script pass, numeric callbacks and supported generic instructions |
| `room` | 130 | 130 | gameplay subsystem | 80/82/8F room/door ROM helpers, selection, camera scroll leaves and door callback writes |
| `room_scroll` | 12 | 12 | scheduled gameplay | 80 visible room tilemap preparation and DMA queues |
| `samus` | 27 | 27 | gameplay helper | 90 run/base speed, jump, fixed-point displacement and distance helpers |
| `samus_control` | 61 | 46 | gameplay subsystem | 90/91 input transitions, momentum, animation command states and environmental effects |
| `samus_projectile_frame` | 8 | 4 | scheduled gameplay | 90 Samus projectile update/render passes and dispatch composition |
| `sprite_objects` | 8 | 7 | scheduled gameplay | B4 sprite allocation, script scheduling and81 base-tile OAM specialization |
| `system` | 17 | 17 | gameplay helper | 80 canonical system flags, event/boss bits, map/OAM resets and palette fades |

## Remaining ownership and uncertainty

Ridley’s synchronous Host explicitly delegates `SpawnEnemy`, `EnemyDeathAnimation`, `Enemy_ItemDrop_Ridley`, normal touch/shot/powerbomb services and `CallSomeSamusCode`. Wrapper request order and arguments can be Rust-owned while the requested native service remains C-owned. Collision and projectile callers likewise use PLM/HDMA host services. Hardware PPU presentation, DMA/VRAM execution, SPC playback, HDMA interpretation and broader native game scheduling are not established by WRAM/queue implementations. Refusal can select C before mutation; an accepted Rust error must terminate its invocation.

The historical Ridley lexical closure lists 220 reachable definitions and 3,129 body lines. Reachability alone does not prove their implementation. The following closure candidates have no owned mapping in this audit; some are the foreign services above, primitive arithmetic represented inline, or genuinely unproved helper specializations:

- `CallSomeSamusCode`
- `ComputeCosMult`
- `ComputeSinMult`
- `DisableMinimapAndMarkBossRoomAsExplored`
- `EnemyDeathAnimation`
- `Enemy_ItemDrop_Ridley`
- `Math_MultByCos`
- `Math_MultBySin`
- `MoveEnemyWithVelocity`
- `Mult8x8`
- `NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy`
- `NormalEnemyShotAiSkipDeathAnim_CurEnemy`
- `NormalEnemyTouchAiSkipDeathAnim_CurEnemy`
- `Ridley_Func_127`
- `Ridley_Func_17`
- `Ridley_Func_18`
- `Ridley_Func_24`
- `Ridley_Func_25`
- `Ridley_Func_27`
- `Ridley_Func_41`
- `Ridley_Func_48`
- `Samus_SetPushedOutOfCeresRidley`
- `SnesDivide`
- `SpawnEnemy`
- `Unreachable`
- `mov24`

The new shared `game_host` forwards seven scheduled roots to the existing Samus projectile and room-scroll owners, contributing no second gameplay bodies. Eight separately reviewed inlined Ridley helpers remain additional translation correspondences in `../ridley-inline-translation-audit.md`, bringing the qualified union to 1,461.

Use `mapped_without_explicit_manifest_symbol` to add focused fixture roots without losing existing source implementations. Use `manifest_native_symbols_without_owned_source_mapping` to detect declaration/source disagreement; it is empty for this snapshot. The source map is a reviewed inventory, not a formal exhaustiveness or semantic-equivalence proof: unfamiliar inlined helpers can remain unaliased, and review notes preserve that uncertainty.

## Completion evidence

The audit compiled and ran only `.scratch/decomp-progress/state-audit/source_audit.rs` with `rustc --edition=2021`; no gameplay verification, replay, staging, source edits, or shared inventory publication was performed by this task. JSON structure, all 45 module/role presence, count consistency, declared-root coverage and native inventory hash were checked.

Retained artifacts: `source-coverage-map.json`, `source-audit-review.txt`, `source_audit.rs`, `source-aliases.tsv`, `source-numeric-scopes.tsv`, `source-forwarders.tsv`, `source-module-roles.tsv`, and this note. The coordinator owns evidence-stage integration and publication. Cleanup of the disposable audit executable is recorded below after the final run.
