# Later gameplay lanes: primary-source catalog

Status: preparation research complete; implementation scopes remain blocked
Owner: `/root/subsystem_catalog/later_lanes`
Observed at: 2026-10-13 08:59 UTC
Main checkout HEAD: `deceefdb568990fe273cc3f6cdf849726d8380bb`

This note covers audio, UI/menus, persistence/credits and remaining complete
Samus/room call boundaries for [ticket 05](../issues/05-subsystem-scope-catalog.md).
It records candidate files and evidence gaps; it creates no gameplay assignment,
claim release, stage publication or acceptance. Gameplay remains held during
preparation, including existing controller/shared-host and Ridley holdings.
[Owner reconciliation](../reconciliation/owner-inventory.md) owns those fences.

## Source identity and evidence interpretation

The owning strategy is [DECOMP_PIVOT.md](../../../docs/DECOMP_PIVOT.md): vanilla
readable Rust gameplay first, any% as the first live target, visible temporary
C/CPU fallback, then full gameplay ownership before Map Rando. The
[revised strategy](../../decomp-strategy/revised-strategy.md) orders ordinary-reset
Ceres escape before normal play, save/load and credits and retains the held
broad Ridley gate. These are requirements, not completion evidence.

The current [registry](../../decomp-progress/registry.json) records native revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, source identity SHA-256
`e74864c229949fc6f1a6bd7528f339bdb3f9364f1e4f9544f08693881d10f111`, and inventory
SHA-256 `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`
(6,053 definitions, 75,229 body lines). Direct hashing of
[source-inventory.toml](../../../tools/port/source-inventory.toml) matches that
inventory digest. The source archive identity in that file is
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.

At this read the registry has 1,461 translated, 925 verified, 38 integrated and
121 live-exercised definitions, across separately retained evidence contexts.
The revised strategy's 1,460/924 snapshot is older; neither count measures game
completion. Registry SHA-256 at final inspection:
`89063f00cecfae39f4a47e54da1ee5f9f6aeaa6c8423bca6435c97c06c9e73a6`.
The registry was republished during research; source identity and stage counts
remained unchanged at this final read.
[source-coverage-map.json](../../decomp-strategy/source-coverage-map.json), SHA-256
`a6c1f07e6519ce53b9c38045585b9c4f332b5945391541a60c9938da335544b5`, maps executable
Rust correspondences, with its stated branch/delegation limits. It grants no
verification or live stage. The checked-in module bytes below match that map.

## What can be reused

Counts in the following table are deduplicated native symbols, filtering stale
and superseded registry observations. `current` means `current-checkout`, and
`historical` means `historical-frozen`; overlapping context counts are not added.

| Existing owner | Current translated mapping | Historical verified / live roots | Current integrated / live roots | Evidence and retained limitation |
| --- | ---: | ---: | ---: | --- |
| `audio.rs` | 21 | 21 / 9 | 0 / 0 | [Ticket 03](../../decomp-port/issues/03-audio-queues.md): 1,147 actual-C cases and rejecting quota mutation; queue bookkeeping only, explicitly excludes `HandleMusicQueue`. Registry `historical-audio-6053-verified-accuracy-20261013` preserves exact historical source, not current integration or SPC playback. |
| `system.rs` | 17 | 17 / 8 | 0 / 0 | [Ticket 02](../../decomp-port/issues/02-system-state.md): 475 actual-C cases; flags, bit access, OAM/map fills, fades and input release filter operate on WRAM only. |
| `samus.rs` | 27 | 27 / 10 | 0 / 0 | [Ticket 05](../../decomp-port/issues/05-samus-physics.md): 5,184 full-WRAM/return cases; no full pose/movement scheduler ownership. |
| `samus_control.rs` | 61 | 46 / 16 | 0 / 0 | [Ticket 08](../../decomp-port/issues/08-samus-animation-input.md): 4,528 full-state cases for selected animation/input roots; 15 further mapped definitions lack explicit historical manifest roots. No complete Samus engine claim. |
| `collision.rs` | 98 | 63 / 5 | 0 / 0 | [Ticket 12](../../decomp-port/issues/12-samus-collision-movement.md): 1,752 cases, 28 normal movement handlers and bounded live movement; PLM callbacks remain explicit host dependencies, no grapple engine or full pose/environment scheduler. |
| `samus_projectile_frame.rs` | 8 | 4 / 0 | 4 / 3 | [Current integrated stage evidence](../../decomp-port/progress-current-integrated-stage-evidence-20261013.md): 476 actual-C pairs; `HandleProjectile`, `DrawPlayerExplosions2`, `DrawBombAndProjectileExplosions` have current measured live roots. `sub_9382FD` remains Offline/current integrated. |
| `room.rs` | 130 | 130 / 20 | 2 / 0 | [Ticket 20](../../decomp-port/issues/20-room-loading-scroll.md) owns nonhardware room/door data, music and scroll leaves; later callback tickets expand it. Registry historical room verification retains 2,665 cases across runs, with possible duplicate domains. The two current integrated roots are `UpdateBgScrollOffsets` and `UpdatePreviousLayerBlocks`, from the room-scroll domain. Hardware transition scheduling remains outside the module. |
| `room_scroll.rs` | 12 | 2 / 2 | 11 / 2 | Current 1,166 actual-C pairs for 11 normal scrolling roots; live `UpdateScrollVarsUpdateMap` and `CalculateLayer2PosAndScrollsWhenScrolling`. A [later guarded-display audit](../../decomp-progress/state-audit/room-display-6053-current-translated-audit-20261013.toml) adds translated `DisplayViewablePartOfRoom`, missed by the older lexical map; its three shared-host fixtures are separately current-verified, with integration/live pending. Normal supported viewport/layer/queue domains; DMA/VRAM execution remains host-owned. |
| `camera_scroll.rs` | 4 | 0 / 0 | 4 / 1 | [Ticket 140](../../decomp-port/issues/140-camera-scroll-frame-ownership.md): 467 pairs and eight controls; live `MainScrollingRoutine`. Room dimensions 1..255, at most 256 scroll cells and only zero/SporeSpawn finished hook. Camera remains in the room lane. |

The three scheduled projectile live roots, room-scroll roots and camera root
come from [bounded live completion](../../decomp-port/recovery-live-completion-20261013.md),
integration `3ce28ba58aba319b60bbc949153baafd4433132e`. Four independent C/mixed
subjects matched state/ordered effects with zero CPU fallback in their declared
installed domain, but 129,712 raw inputs ended in **Ceres room DF8D/game state 8**.
This is no later progression, credits or original-ROM movie proof. Projectile
PLM/HDMA services still delegate to actual C. Selected Ceres player/explosion
draws refuse because the pinned C reads uninitialized coordinates; the
[original projectile handoff](../../decomp-port/parallel-independent-scope-samus-projectile-frame.md)
documents those oracle limits. Successful root hits confer no live coverage on
unmeasured helpers or later branches.

`game_host.rs` supports synchronous `SpawnPlm`, `SpawnHdma` and `FlushVram`
requests, and complete projectile/display/scroll entries
([source](../../../crates/port/src/game_host.rs)). It has no APU or
SRAM request contract. Registry `shared-game-host-6053-current-verified-audit-20261013`
retains seven projectile and three room-display actual-C fixtures; the durable
`integrate-shared-game-host` claim still runs, so this is no integrated host
admission. Preserve its existing source/registration ownership. The canonical
[memory.rs](../../../crates/port/src/memory.rs) exposes WRAM, not a SRAM view.

## Remaining complete boundaries and proposed disjoint ownership

Native references below resolve to the pinned files
[sm_80.c](../../../deps/snesrev-sm/src/sm_80.c),
[sm_81.c](../../../deps/snesrev-sm/src/sm_81.c),
[sm_82.c](../../../deps/snesrev-sm/src/sm_82.c),
[sm_90.c](../../../deps/snesrev-sm/src/sm_90.c),
[sm_91.c](../../../deps/snesrev-sm/src/sm_91.c) and
[sm_8b.c](../../../deps/snesrev-sm/src/sm_8b.c); table entries give exact
definition lines. The composed draw scheduler is
[sm_a0.c](../../../deps/snesrev-sm/src/sm_a0.c).

Every row is **research-blocked**, not a ready `translation-scope-v1`. Proposed
new paths are absent from the present [public module registry](../../../crates/port/src/lib.rs)
and dedicated source/test/manifest listing. The workers would own only their
three named files; shared `lib.rs`, CLI/catalog/ABI, canonical RAM/SRAM/ROM and
host adapters belong to the integrator. Existing verified modules are read-only
dependencies, preventing player/room/audio workers from editing the same state
owner. Whole envelopes may need several successive complete boundaries.

| Candidate complete boundary | Exact candidate writable files | Owning native source and dependencies | Receipts/contracts still required |
| --- | --- | --- | --- |
| Scheduled music dequeue + three-channel SFX lifecycle | `crates/port/src/audio_frame.rs`; `crates/port/tests/audio_frame.rs`; `tools/port/audio-frame.toml` | `sm_80.c:646` `HandleMusicQueue`, `sm_82.c:738–795` `HandleSoundEffects`, five `SfxHandlers_*`, `ResetSoundQueues`. Reuse Rust queues. Depend on `APU_UploadBank`/ROM music-bank extent, ordered `RtlApuWrite`, reset and APU lifecycle. | Fresh direct execution with full WRAM and ordered APU upload/write trace, timer/queue wrap, downtime/channel order and invalid-state refusal. SFX handlers 1/2/3 explicitly say “Modified from original”: C omits original acknowledgement polling/clear behavior, so original-ROM/APU comparison must separately settle that difference. |
| Samus gameplay input `alfa` frame | `crates/port/src/samus_input_frame.rs`; `crates/port/tests/samus_input_frame.rs`; `tools/port/samus-input-frame.toml` | `sm_90.c:5930–5978`: `HandleControllerInputForGamePhysics`, `CallFrameHandlerAlfa` and its three handlers. Normal handler resets pose state, sets radius, calls input, suit palette, gravity, block-inside detection, HUD/projectiles and `Samus_Func10`. Reuse `samus`, `samus_control`, `collision` and projectile modules. | Direct alfa/root counter and state before/after frame, complete numeric selector/ROM closure, HUD/grapple/inside-block dependencies and ordered PLM/HDMA effects. Ticket 78 still owns causal controller/poll/NMI work; this candidate consumes its contract and does not edit scheduler/input observer files. |
| Samus movement/contact/transition `beta` frame | `crates/port/src/samus_movement_frame.rs`; `crates/port/tests/samus_movement_frame.rs`; `tools/port/samus-movement-frame.toml` | `sm_90.c:5984–6046`: `HandleSamusMovementAndPause`, `CallFrameHandlerBeta`, normal `Samus_FrameHandlerBeta_Func17`; latter calls movement, minimap, gamma, animation, hit interruption, block collision transitions, pose transitions, palette, periodic damage, pause and low-health. Reuse existing movement leaves. | Complete selected beta boundary and every external/dynamic dependency review. Normal movement hit counts do not prove beta-frame ownership. Need health/frozen/knockback/gamma/pose gates and a reviewed pause handoff. Alfa and beta must serialize if either scope expands into the other's mutable transition owner. |
| Door transition coroutine and room initialization | `crates/port/src/room_transition.rs`; `crates/port/tests/room_transition.rs`; `tools/port/room-transition.toml` | `sm_82.c:3768–4177` `CallDoorTransitionFunction_Async` and 15 selected transition functions; `sm_82.c:74` `InitAndLoadGameData_Async`, `sm_80.c:1634` `Play20FramesOfMusic_Async`. Reuse room/header/scroll/PLM/camera and player modules. Load-more calls real enemy, PLM, HDMA, tiles/palette/music and NMI services. | Direct natural door/coroutine counters and continuation state, source/table/asset extents, full room/player restoration, ordered DMA/VRAM/APU/NMI/IRQ and actual-C callback state. `room.rs` explicitly rejects unowned transition routines; leaf header/scroll evidence and synchronous PLM handling do not prove the coroutine. Split initialization from transition only at reviewed complete boundaries with pinned shared state ownership. |
| Paused map/equipment frame | `crates/port/src/pause_menu.rs`; `crates/port/tests/pause_menu.rs`; `tools/port/pause-menu.toml` | `sm_82.c:1285–1419` paused states, `MainPauseRoutine`'s eight-way table, `PauseMenu_1_EquipmentScreen`; `sm_82.c:2410` `EquipmentScreenMain`'s four categories. Depends on player inventory/reserve state, system filter/fades, audio queues, map/graphics/HDMA/VRAM and Samus wireframe tables. | Exercise captured paused frame and each admitted selector, inventory toggles/reserve transfer, map transitions, restoration on unpause and material queue/effect order. [Ticket 109](../../decomp-port/issues/109-room-pause-hooks-and-shaktool.md) owns four Draygon pause hooks/PLM setup callbacks, not pause UI; it is Offline-only. Closure may exceed 32 routines. |
| File-select/options/new-game UI | `crates/port/src/file_menu.rs`; `crates/port/tests/file_menu.rs`; `tools/port/file-menu.toml` | `sm_81.c:646` `FileSelectMenu` (indirect table), file copy/clear/select handlers, `sm_81.c:1368` `FileSelectMap`, `sm_81.c:2435` `NewSaveFile`; `sm_82.c:4462` `GameState_2_GameOptionsMenu`. Depends on save codec, canonical controller bindings, room/load-station state, sprites/assets, palettes, HDMA/audio/VRAM. | Review table dispatch and split complete menus into successive scopes; copy/clear SRAM receipt and controller/menu poll contract. No file/UI/credits routines appear among current source-map owners or stage observations searched. Existing ticket 78 specifically diagnoses native file-menu poll order; preserve that claim. |
| Save codec + fresh-process continuation | `crates/port/src/save.rs`; `crates/port/tests/save.rs`; `tools/port/save.toml` | `sm_81.c:40–145`: `SaveToSram`, `LoadFromSram`, `PackMapToSave`, `UnpackMapFromSave`. Native save copies 96 bytes of player fields, area map and saved station/area, writes slot data and lower/upper checksum/inverse copies, then `RtlWriteSram`. Load restores fields/maps or clears invalid slot state. Depends on canonical SRAM/WRAM ownership, slot/area/packed-table bounds and filesystem persistence host. | Shared SRAM interface and maintained actual-C fixture/evidence support, checksum wrap/one-copy-valid/corrupt-slot and unchanged-byte controls. Fresh process must read retained SRAM and then exercise the real load-station/room/player restoration plus reset/audio/hardware lifecycle. [Ticket 76](../../decomp-port/issues/76-reference-soft-reset-crash.md) preserves SRAM through a fixed reset sentinel; it proves the crash fix, not save codec or fresh-process save/load. |
| Credits object scheduler, then ending frame | `crates/port/src/credits.rs`; `crates/port/tests/credits.rs`; `tools/port/credits.toml` | `sm_8b.c:1516–1565` `CreditsObject_Init/Process/ProcessOne/Func1`, `sm_8b.c:2991` `CallCreditsObjectFunc`, IntroObject script dispatch at bank8C data; full `GameState_39_EndingAndCredits_` `sm_8b.c:4287` adds cinematic sprites/BG, palette, draw and Mode7. | No declared later-domain exercise or Rust credit/cinematic closure receipt located. Need checked 8B/8C scripts, finite instruction closure, 32-row ring/WRAM aliases and ordered graphics effects; full ending depends on cinematic/Mode7/hardware/audio scopes. Scheduler alone does not prove credits route. Final adapted any% route requires ticket 08's independent subjects, original-ROM boundaries, exact inputs/adaptations and fallback report. |

The broader `GameState_8_MainGameplay` at `sm_82.c:847` owns order between
alfa, enemy processing, message-box coroutine, beta, projectiles, PLM coroutine,
animation, contacts, camera, drawing, HUD, room main code and timers. Its caller
integration is shared scheduler ownership, not a ninth lane-owned module.
`DrawSamusEnemiesAndProjectiles` at `sm_a0.c:286` interleaves sprites/explosions,
low/high-priority enemy projectiles, Samus and eight enemy layers; it crosses
player/enemy/render ownership and belongs to a separately reviewed composed gate.

## Native promotion and exercisability blockers

[assignments.md](../../../docs/agents/assignments.md) requires pinned inventory
and exercise receipts, resolved dependency tickets, complete reviewed dependency
evidence, current input hashes, exact paths, and concrete bounded pre-integration
and integration argv. [translation.rs](../../../crates/pipeline/src/translation.rs)
at lines 596–768 requires 1..32 routines, directly observed execution, exercised
reachable closure and a ready/evidenced review for every direct, indexed and
external/macro dependency. Exact `.rs`/`.toml` writable files must lie under
`crates/port/src`, `crates/port/tests` or `tools/port`. Existing paths and all
dependency evidence must match their input seals.

**The current exercise reader cannot promote these non-bank86 boundaries.**
`translation.rs:400–488` accepts only schema-2 `diagnostic-complete`, nonaccepting
projectile-inventory receipts; binds ROM/movie, `sm_86.c`, `headless.c`, the
instrumented build and `projectiles.tsv`; and produces addresses by
`0x860000 | low`. `validate_scope:659` requires a directly observed address.
Historical counters and current composed root observations therefore remain
planning inputs for these lanes, not sufficient native promotion receipts.
A maintained Rust extension must admit appropriate typed exercise provenance,
including addressless table-dispatch roots, without weakening source seals or
using callee counts as caller exercise. Ticket 05 should expose this as a child
research/tooling dependency before claiming later scopes ready.

The ordinary [family CLI contract](../../../docs/DECOMP_PORT.md) compares all
WRAM and raw returns and explicitly excludes store order/hardware effects.
Current `family_model.rs` has WRAM patches/callback contracts, no general SRAM
fixture model. Current shared Host only exposes PLM/HDMA/VRAM. Audio and save
need maintained ordered-APU/SRAM contracts and actual compiled C adapters;
inventing per-routine stubs or accepting WRAM-only receipts is insufficient.

Dependency order for later work: reconciled controller + shared host → reviewed
alfa/beta and door/initialization boundaries → normal progression; independently
reviewed audio hardware contract → audio frame; player/room + graphics/audio →
pause/file UI; SRAM host + codec → real reset/load/fresh process; progression +
cinematic/render hardware + credits object closure → ending/credits route.
Milestone route gates stay serialized even when disjoint lane research proceeds.

## Executable gate argv found in primary sources

These commands were **read, not run**. They are concrete existing gates for
reused domains, not missing future APU/SRAM/credits gates. Execute from the exact
assigned/frozen checkout only after authorization and scope review.

```sh
# docs/DECOMP_PORT.md:16-17 — existing bounded WRAM/return domains
cargo run -p pipeline --locked --offline -- family-verify --manifest tools/port/system.toml --source deps/snesrev-sm --output /tmp/system-evidence
cargo run -p pipeline --locked --offline -- family-verify --manifest tools/port/audio.toml --source deps/snesrev-sm --output /tmp/audio-evidence

# .scratch/ceres-continuation/host-implementation.toml:20 — unchanged consumers
cargo test -p port --locked --offline --test game_host --test samus_projectile_frame --test room_scroll

# docs/DECOMP_BATCH.md:65,68,77-83 — exact existing family/affected selection
cargo run -p pipeline --locked --offline -- batch verify --profile all
cargo run -p pipeline --locked --offline -- batch verify --families projectile_flight
cargo run -p pipeline --locked --offline -- batch verify --profile offline-all --case-jobs 4
cargo run -p pipeline --locked --offline -- batch verify --profile offline-all --changed-since HEAD~1
cargo run -p pipeline --locked --offline -- batch plan --profile offline-all --changed-since HEAD~1

# docs/DECOMP_BATCH.md:6-10 — project checks and established gameplay profile
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo run -p pipeline --locked --offline -- batch run
```

The last `batch run` defaults to `gameplay`, four independent C/mixed processes,
18,000 frames and three jobs; it does not establish a credits domain.
[tools/port/batch.toml](../../../tools/port/batch.toml) registers the existing
audio/system, Samus/room and translated-subsystem families; unregistered future
manifests are Offline only until explicit live wiring exists.

**Native scope argv differs from direct CLI examples.**
`translation.rs:809–866` permits real `cargo test`, locked/offline pipeline
`family-verify`, `live-verify`/`routine-live` and `batch verify/run`. It rejects
`--source`, `--checkout`, `--root`, `--changed-since`, `--manifest-path` and
`--config` in the embedded scope argv: the coordinator supplies the checkout and
source context. Thus do not copy the direct examples verbatim into a scope.
Initial gates are capped at 60 seconds and integration gates at 3,600 seconds;
both are mandatory, native gates must yield receipts and Cargo must execute a
test. No current primary source supplies an admitted APU-frame, SRAM-codec,
fresh-process continuation or credits-route argv. Their exact commands remain
research acceptance work, not unsupported new flags.

## Inspection digests

Hashes are observations of the current files, not a refreshed source seal. New
scopes must recheck after concurrent maintenance/reporting edits.

| File | SHA-256 |
| --- | --- |
| `docs/DECOMP_PIVOT.md` | `18fd213618aba17ac71c7ca9dad62451d77b5205ea0907039c5a0a471a02d02e` |
| `.scratch/decomp-strategy/revised-strategy.md` | `dc13bc7e470fd00a22eb81e666c40c19ce6fa00842ed6f6672cf1000ddf0624b` |
| `crates/pipeline/src/translation.rs` | `7d581d67f3bda1526ad0815c7f6c149ddf7874e9749d75ac13fc428b3e6f85cd` |
| `crates/port/src/audio.rs` | `4308e496267857ca869db45f4127e505bec4ad29c4c9ede8f08d6e9769ccf69d` |
| `crates/port/src/system.rs` | `bedc7662f4c1c435c968a6c07c73331a76f6c58621b4ca885f73f6be33ccbf16` |
| `crates/port/src/samus.rs` | `4f7e14ecbdf724305094d5924ec931b04dca422e8bcc3108fd5ff13531271234` |
| `crates/port/src/samus_control.rs` | `d0d592bdbfad2b0bb1b845ec3fc2523cb4a6ff15c0dd07015c9736d3efb4ed60` |
| `crates/port/src/collision.rs` | `881431d92845006509d2956eae0b3736459961762f5b4a021577f130c530d4f1` |
| `crates/port/src/samus_projectile_frame.rs` | `d6f6a8e93448f882fc3d49b35e687a856473c172389d900ab984745ce517b712` |
| `crates/port/src/room.rs` | `e7b13c17ac668f8aba50a21aedfcd4d4dcf93f5e1cd2d4c610557cb56774ba72` |
| `crates/port/src/room_scroll.rs` | `cc2c7ee9138a1c781321045e28246c083fca6a53ce284ac626d9abf5bdb5bdf0` |
| `crates/port/src/camera_scroll.rs` | `5700c005e1306ee61771b6c7e603621f0ec15eb9568c5aa3c215dae17bd789f3` |
| `crates/port/src/game_host.rs` | `baaae6c0dbc60b9c384c37088806648e96669c3714d7864bb58b5b5d93972527` |
| `crates/port/src/memory.rs` | `7c0df2e77ed2d05dda05bd6d9e98fef3f7ec5f1c76426da504229af698bfecc5` |
| `deps/snesrev-sm/src/sm_80.c` | `ab9abc1042ea876c51de90d00632d1af13e9a765c77cf371950ddcdc783130c7` |
| `deps/snesrev-sm/src/sm_81.c` | `bd05747f344ac18d4f5230e10fd4678b087048c24e8aeb5f685691b36badb1a0` |
| `deps/snesrev-sm/src/sm_82.c` | `afa8b6fb9e9e5ad378db4779c1c39574856a57d50d25c4536aa2bafeeb79e3ad` |
| `deps/snesrev-sm/src/sm_90.c` | `ad9f103509160abd8a9e0f85f3c3c96187c89f31dd770cf533fa32a97925247a` |
| `deps/snesrev-sm/src/sm_91.c` | `8a087a687e9611d1272797522f1b5fd20f9e344d851e0a6701af8b9052b5070f` |
| `deps/snesrev-sm/src/sm_8b.c` | `b77fe217eb0e74b82d92acd4d32f44141a095da8264c15fb5f703067b0496ba3` |

Selected exact native definition hashes from the registry make the next roots
reviewable independent of the bank file digests:

| Symbol / source | Definition SHA-256 |
| --- | --- |
| `HandleMusicQueue`, `sm_80.c:646` | `f635d911c2c92f687fc62c3eee96617f10f92fadb01d8e9700101980ebeaab2f` |
| `SaveToSram`, `sm_81.c:40` | `35e57fa261b8a1c8b91cd6635be693bb144444ac44ecbe18fb32971923b6d285` |
| `LoadFromSram`, `sm_81.c:78` | `d0fc4010592bdb2b8656246bf1a7835acdbe248a70cc34c4b5568d8db9e92c78` |
| `Samus_FrameHandlerBeta_Func17`, `sm_90.c:6006` | `5a461686f97d25739ec2b4aede6953c58e386269b42a7639694e89d34e74e9f1` |
| `DoorTransitionFunction_LoadMoreThings_Async`, `sm_82.c:4005` | `ccb4bcf071da38792ace03d55033f335800fa92b131039148922e42e4fe8c36c` |
| `CreditsObject_Process`, `sm_8b.c:1525` | `326b4962d699c15ea6c3ed4778e77e2d3b49139bbba9dbb53ae109589f1be7fd` |
| `GameState_39_EndingAndCredits_`, `sm_8b.c:4287` | `50b2207cf8fb734181d8128240cf8ddfae11cdb5990bda5da4da9af677b0a10b` |

## Verification and cleanup of this research

Only repository reads, `jq` queries, hashes and disk-space inspection were used;
no fixtures, replays, builds, tests, gameplay commands, inventory refreshes or
claim mutations ran. This file is the only authored artifact. No run directories,
temporary binaries or worktrees were created, so scoped run cleanup is not
applicable and reclaimed bytes are **0**. Existing frozen sources, submitted
worktrees, readers, shared evidence store and private ROM/movie/SRAM dependencies
remain retained. Available filesystem space at the last inspection was
**223,134,969,856 bytes**. Coordinator owns catalog integration/checkpointing.
Final note checks found all linked primary-source files present and all 20
listed file digests matching. After concurrent workspace activity, final
available space was **220,356,272,128 bytes**; this research created no bulk output
and removed no other session's files.
