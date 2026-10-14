# Own complete equipment categories, reserve transfers and EquipmentScreenMain root

Type: task
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `translation-scope-v1-after-supported-exercise-and-adapter`. No ownership or gameplay acceptance is granted.

EquipmentScreenMain is included alongside the 28 category/interaction routines, making 29 owned routines. It selects four categories by low byte of pausemenu_equipment_category_item and then draws selector/reserves/wireframe in source order through the rendering dependency. This is complete category/root ownership for admitted equipment input, not a stand-in for the whole pause state machine.

Use canonical WRAM category/item word0755, delay0757, equipped/collected item/beam bitsets09A2..09A8, health09C2/09C4 and reserves09D4/09D6. Preserve AUTO/manual reserve switch/transfer, held/new keys, empty/full/transient health, all category/navigation boundaries, plasma/spazer exclusion and repeated palette/audio effects.

EquipmentScreenHudReserveAutoTilemap_On_BUGGY reads source-described garbage ROM at82998B/829997. Seal these actual inputs and preserve pinned output in actual-C tests; original behavior is a separate comparison. Reject unsupported selector/state/assets before effects. Do not create UI copies of player state or change controller polling.

## Exact proposed files and ownership

- `crates/port/src/pause_equipment.rs`
- `crates/port/tests/pause_equipment.rs`
- `tools/port/pause-equipment.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Exact routine boundaries

| Routine | Pinned source | Entry | Inherited definition SHA-256 |
| --- | --- | --- | --- |
| `ChangePaletteValues` | [src/sm_82.c:1863](../../../../../deps/snesrev-sm/src/sm_82.c:1863) | 0x82A29D | `744b9adeeee9594df1646a8a2702f1aa292498a68899517d54c51514646279c7` |
| `EquipmentScreenCategory_Boots` | [src/sm_82.c:2707](../../../../../deps/snesrev-sm/src/sm_82.c:2707) | 0x82B150 | `93ef38ad4ea25d1eaa8662fc243e6675538cf920346505a5793fc61bb9b6dfc3` |
| `EquipmentScreenCategory_Boots_MoveResponse` | [src/sm_82.c:2714](../../../../../deps/snesrev-sm/src/sm_82.c:2714) | 0x82B160 | `2e3288606e0be0d6ce7d65a9c5cdcd19b5e956f79463f1afd23bdf3f3d37583e` |
| `EquipmentScreenCategory_Boots_MoveUpInBoots` | [src/sm_82.c:2934](../../../../../deps/snesrev-sm/src/sm_82.c:2934) | 0x82B53F | `e5b66ab6289f25b0354409f5637708580d98dc0df49a30ceb63247afbf501489` |
| `EquipmentScreenCategory_ButtonResponse` | [src/sm_82.c:2946](../../../../../deps/snesrev-sm/src/sm_82.c:2946) | 0x82B568 | `1aff67797b988cc0f5c84f0226bea16a92f1de0f7fd96c991e709f9178b454f9` |
| `EquipmentScreenCategory_Suit` | [src/sm_82.c:2677](../../../../../deps/snesrev-sm/src/sm_82.c:2677) | 0x82B0C2 | `f9da861517064a1767656feff663145161a5154d72ec28c5fed20157632f9acb` |
| `EquipmentScreenCategory_Suit_MoveResponse` | [src/sm_82.c:2684](../../../../../deps/snesrev-sm/src/sm_82.c:2684) | 0x82B0D2 | `a080aa83e3a1a03166d3f7c4de205247a91bb8c7cd3a7bafa73a22f4c28336eb` |
| `EquipmentScreenCategory_Tanks` | [src/sm_82.c:2420](../../../../../deps/snesrev-sm/src/sm_82.c:2420) | addressless | `a160a3b3b85552519d7769634c89560cecbe9c9135f33438bfb39b4029598b52` |
| `EquipmentScreenCategory_Tanks_0` | [src/sm_82.c:2553](../../../../../deps/snesrev-sm/src/sm_82.c:2553) | addressless | `7978f90e86634549d3b1827a9a0e894576cf5589c32955f47a3fe2bdb081bcff` |
| `EquipmentScreenCategory_Tanks_1` | [src/sm_82.c:2606](../../../../../deps/snesrev-sm/src/sm_82.c:2606) | 0x82AF4F | `dab2a84e73db8d153bf41ad9c08952658ecc02b168653792b7073dfe07a8ecda` |
| `EquipmentScreenCategory_Weapons` | [src/sm_82.c:2629](../../../../../deps/snesrev-sm/src/sm_82.c:2629) | 0x82AFBE | `7169c2c0c897b7e3b54c0de7b80b3421e3184347077fff2500d1012202969bdb` |
| `EquipmentScreenCategory_Weapons_MoveButtons` | [src/sm_82.c:2641](../../../../../deps/snesrev-sm/src/sm_82.c:2641) | 0x82AFDB | `5ebee69d0a3bbedb011d5c9f6324c7604dcf4779d484ec71f038546e76b21749` |
| `EquipmentScreenCategory_Weapons_PlazmaSpazerCheck` | [src/sm_82.c:2664](../../../../../deps/snesrev-sm/src/sm_82.c:2664) | 0x82B068 | `e11dd9bd9181175062f877388d576e7b361ba52028dcbcf3ffe0d2bbb917dae3` |
| `EquipmentScreenEnergyArrowGlow_Off` | [src/sm_82.c:2493](../../../../../deps/snesrev-sm/src/sm_82.c:2493) | 0x82AE46 | `4f5ee4dce66d7c15ecda4af8cf78460e8793a918fcbdf35ae735d823fbeff601` |
| `EquipmentScreenEnergyArrowGlow_On` | [src/sm_82.c:2486](../../../../../deps/snesrev-sm/src/sm_82.c:2486) | 0x82AE01 | `62525d082b259d325411ffe6113104636d04b594c3301e8fe287bafef82a6caf` |
| `EquipmentScreenGlowingArrow` | [src/sm_82.c:2451](../../../../../deps/snesrev-sm/src/sm_82.c:2451) | addressless | `5647003c2ad5275b34f29c01d6f093cb995d4e14f9386d039a6fcd805441bb98` |
| `EquipmentScreenGlowingArrowAnimated` | [src/sm_82.c:2461](../../../../../deps/snesrev-sm/src/sm_82.c:2461) | 0x82AD29 | `abcd11884278f99b19f459b12b118922b02724b7ad1858f16862ecb0ddf54bbb` |
| `EquipmentScreenGlowingArrowSolidOff` | [src/sm_82.c:2480](../../../../../deps/snesrev-sm/src/sm_82.c:2480) | 0x82ADEF | `b332ecdffdfc263e303bde297c5e17c46369f5dc6eb8b303b8bc32845db427ee` |
| `EquipmentScreenGlowingArrowSolidOn` | [src/sm_82.c:2474](../../../../../deps/snesrev-sm/src/sm_82.c:2474) | 0x82ADDD | `b573b75d37c7f37cd7b016f2c988c439212700fd8c6241c81bfb1d114f37729c` |
| `EquipmentScreenHandleDpad` | [src/sm_82.c:2426](../../../../../deps/snesrev-sm/src/sm_82.c:2426) | 0x82AC8B | `c8e0bf589610fe7ac5a0b515b9655cb8cb7f3f9bb84a41dc168cb091e55b8799` |
| `EquipmentScreenHudReserveAutoTilemap_Off` | [src/sm_82.c:2597](../../../../../deps/snesrev-sm/src/sm_82.c:2597) | 0x82AF33 | `ace801e86db443dd290e1558e28b88e3b4df47287d985dbd0b913a7cd9ef0160` |
| `EquipmentScreenHudReserveAutoTilemap_On_BUGGY` | [src/sm_82.c:2583](../../../../../deps/snesrev-sm/src/sm_82.c:2583) | 0x82AEFD | `302b1aea0e2990d4669ad17633243e16418b39d1c0793beec14c829e063a4e3a` |
| `EquipmentScreenMain` | [src/sm_82.c:2410](../../../../../deps/snesrev-sm/src/sm_82.c:2410) | addressless | `10ae5793f50b6d1d99dbae019549a9ab085582908f1b25f0668ce617b796dece` |
| `EquipmentScreenMoveLowerOnSuitsMisc` | [src/sm_82.c:2895](../../../../../deps/snesrev-sm/src/sm_82.c:2895) | 0x82B4B7 | `48d455efb8f1b60ce06134d1d07e8f2b1d5e85f9d0d172c6a165e320af816e2a` |
| `EquipmentScreenMoveToBeams` | [src/sm_82.c:2862](../../../../../deps/snesrev-sm/src/sm_82.c:2862) | 0x82B456 | `9e9f13f115f092e0cb11d1e687de2b949b177fad07c2c1e18db0532c6326072b` |
| `EquipmentScreenMoveToBottomOfBeams` | [src/sm_82.c:2881](../../../../../deps/snesrev-sm/src/sm_82.c:2881) | 0x82B489 | `ade0e6e0ab8f31885682aa170023c39ad3b7eec9f0c6248f405589e3d67c1d0c` |
| `EquipmentScreenMoveToHighJumpOrLowerInBoots` | [src/sm_82.c:2922](../../../../../deps/snesrev-sm/src/sm_82.c:2922) | 0x82B511 | `8dcd1a4d7851bf7d60a08185f2297bd23512adbcd921b257171da408a82d6904` |
| `EquipmentScreenMoveToReserveTanks` | [src/sm_82.c:2852](../../../../../deps/snesrev-sm/src/sm_82.c:2852) | 0x82B43F | `59188ec56c635a5de2293fb9733b82ca9a85d8e92fbd4adbc593d9e4318fd99d` |
| `EquipmentScreenMoveToScrewOrHigherOnSuits` | [src/sm_82.c:2910](../../../../../deps/snesrev-sm/src/sm_82.c:2910) | 0x82B4E6 | `8db19cc05b56d7425e3e64113c297eb08e64f1f079f7fba2f4bc992e4a7e570b` |

## Dependencies

- 18-exercise-contract-supplement
- 18-ui-transaction-adapter
- 18-pause-equipment-render
- current-source-verified-audio03
- 14-canonical-player-state
- 12-controller-polls

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test pause_equipment --test pause_equipment_render --test audio
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/pause-equipment.toml --adapter pause-equipment --output out/port/pause-equipment --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- 29 routines own whole four-category EquipmentScreenMain invocation plus reserve mode/transfer/navigation/timer/palette interaction.
- Canonical category/item word0755, delay0757, equipped/collected bits09A2..09A8 and health/reserves09C2/4/09D4/6; preserve plasma/spazer exclusion.
- Pinned AUTO HUD garbage ROM reads82998B/829997 remain source-bound; original comparison is declared separately.
- Root executes the rendering dependency in exact selector/reserve/wireframe order and synchronous audio leaf effects.
- Exact current-source actual-C full WRAM/raw return/ordered effect parity; independent semantic negative controls.
- Supported directly observed complete root/selected-handler/assets provenance with source/build/binary/ROM/movie/readers sealed.
- Whole-request preflight and canonical-state commit; unsupported domain refuses unchanged and fallback is visible; first/late service failure never retries native root.
- Four independent material C0/C1/mixed0/mixed1 subjects, actual playback/menu resources and separate original-ROM route evidence.
- Integrator alone wires shared lib/state/ROM/callback/ABI/catalog, freezes merged source, verifies all affected families and publishes independent stages.
- Both scoped cleanup passes, retained source/readers/store/private inputs, retired implementer worktree and post-cleanup authentication before completion.

## Limitations and cleanup

- Pause map switching/backups/unpause/hooks are successor research; no implicit MainPause ownership.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

