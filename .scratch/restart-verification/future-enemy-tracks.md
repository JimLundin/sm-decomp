# Enemy and boss source boundaries

Source research: /root/future_enemy_tracks; read-only, no current assignments.
Recovered C578f90b is retained under restart/out/scroll/native/
2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/.
Main Rust implementations are reuse material; historical receipts do not admit
them into restart. Paths below are proposed ownership, not new source claims.

| Scope | Exact source anchors and reuse | Dependency/readiness | Proposed paths |
| --- | --- | --- | --- |
| Common enemy lifecycle | EnemyMain A08FD4, sm_a0.c:1724; SpawnEnemy A09275:1833; EnemyDeathAnimation A0A3AF:2518; InitializeEnemies/DetermineWhichEnemiesToProcess. Main enemy.rs, enemy_instructions.rs:160, enemy projectile/contact/render and sprite_objects.rs provide leaves. | Specify complete active-list iteration, initialization, selected AI/instructions, contacts, death/draw and global finalizers. Respawn callback86EF10 and RespawnEnemy86F264 in sm_86.c:4627/4845 have no direct named Rust implementation found. Whole scheduler needs native state/service preflight. | restart/enemy-core/; sole owner of population/scheduler/contact/instructions/death/pickup files. |
| Ordinary actors | Waver instruction A386E3/initA386ED/mainA3874C/helperA387FE, sm_a3.c:98/103/119/145. Main enemy_ai.rs:451, enemy_rio.rs, enemy_crawler.rs and enemy_jump.rs contain Waver/Ripper/Rio/crawler/jump families. Sidehopper initA3AB09/mainA3ABCF. | Actor logic/ROM callbacks can prepare independently. Complete scheduled initialization/contact/death/draw waits enemy-core. Keep common lifecycle outside actor files; generic main interpreter rejects bank-specific callbacks. | restart/ordinary-enemies/, separate actor modules; dependency on frozen enemy-core interface. |
| Whole Ridley | CeresRidley_Main A6A288 sm_a6.c:1225; Ridley_Main A6B227:1729, plus init/hurt/shot/powerbomb/cutscene/explosion roots. Main ridley_subsystem.rs:876 and movement/cutscene/rendering/instructions/admission modules provide substantial decomposition. | Needs restart adaptation and exact services/copyback. Main service trait includes SpawnEnemy, EnemyDeath, EnemyItemDrop, normal contact reactions and SamusCode; Mode7 hardware ordering remains explicit. Source preparation need not wait route arrival. Whole live lifecycle depends on common enemy/host boundaries. | restart/ridley/, boss-specific lifecycle/movement/cutscene/rendering/instruction/admission files. |
| Mother Brain later | BodyInit A98687 sm_a9.c:271, brainInitA98705:297, bodyHurtA9873E:312, brainHurtA9878B:345; CallMotherBrainFunc:128. | Needs separate closure research for shared body/brain slots, custom scripts/drawing, tube enemies, FX/VRAM/palettes/corpse/cutscene/player continuations. Main mother_brain_bomb.rs implements projectile86C482/86C4C8, not the boss. | Future restart/mother-brain/ after exact scope research. |

The scheduler call site sm_82.c:851..865 orders DetermineWhichEnemiesToProcess,
EnemyMain and EprojRunAll. EnemyMain dispatches each active slot's contact/AI/
instructions/drawing and performs HandleSpriteObjects, RNG and collision-index
finalizers after the loop. Rust AI inside the native scheduler demonstrates
actor ownership only; whole-root ownership requires the complete admitted list
and finalizers. Player damage/grab remains an explicit dependency rather than
giving enemy workers ownership of player movement files.

Research/source preparation may overlap across the three first scopes after
their interfaces and exact files are frozen. Builds/live integration stay with
the integrator. No source edits, builds, tests, captures, runs or claims; cleanup
not applicable,0bytes. Inspect current owner and baseline before dispatch.
