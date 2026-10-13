# Whole subsystem continuation catalog

Status: researched holdings; no new gameplay assignment or production cutover
Owner: `/root/subsystem_catalog`, preparation coordinator thread `01a13460-4aa9-7fe0-865e-a8f54ffca310`
Baseline inspected: `deceefdb568990fe273cc3f6cdf849726d8380bb`
Read-only preparation: ticket 05; gameplay starts only under later authorization

The next concrete gameplay continuation is the existing shared-host integration,
then the submitted compact controller integration, then diagnosis of the first
extra NMI/controller poll. It continues ticket 78 and the old Ceres claims; it
creates no replacement gameplay ownership. Every later lane first reuses the
translated Rust and retained acceptance below. Child tickets 11–20 expose work
that the composed milestones cannot inherit from routine counts.

## Inspected identity and evidence

The live reporting snapshot inspected at `updated_at = 1791881031` is
`.scratch/decomp-progress/registry.json`, SHA-256
`b00d386b0a1f9d03343703c5e4768f339752bdb95da785434e68ba13a03bb40d`.
Its recovered-C revision is `578f90b3cc49557bb70060ad033bb90b8cf8ac50`,
inventory SHA-256 `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`
and dependency identity SHA-256
`e74864c229949fc6f1a6bd7528f339bdb3f9364f1e4f9544f08693881d10f111`.
The lexical 6,053 definitions/75,229 body lines and stage counts (1,461 translated,
925 verified, 38 integrated, 121 live-exercised) are inventories with overlapping
stages and source contexts, not a full-game fraction. The selected observations
and limitations are retained in [inventory-observations.json](inventory-observations.json).
The current-checkout/frozen distinction is explicit in `evidence_context`.
The reporting owner republished the live registry during this research. Final
observed SHA-256 is `89063f00cecfae39f4a47e54da1ee5f9f6aeaa6c8423bca6435c97c06c9e73a6`,
with the same source identity and stage counts; the selected observation artifact
above intentionally preserves the initial snapshot. Neither registry digest is
a mutable gameplay assignment input without rechecking it at actual dispatch.

The ownership bridge [legacy-owner-bridge.toml](../reconciliation/legacy-owner-bridge.toml)
is SHA-256 `b4046304db39ed5d70501d0b7b29a24abe75b17c03fd0c20ead702857161b4c5`.
The original v1 remains historical evidence. The normalized maintained-preparation
[legacy-owner-bridge-v2.toml](../reconciliation/legacy-owner-bridge-v2.toml) is now
available, SHA-256 `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`;
it preserves the same holdings and must be checked under actual cutover authority.
All its holdings remain fenced, including Ceres 01–03, ticket 78, ticket 141 and
affected-family-assignment 03–07. Main Git belongs to reporting thread
`01a13394-29b8-76a3-aab7-5ab8eff60694`; isolated preparation integration belongs
to thread `01a13460-4aa9-7fe0-865e-a8f54ffca310`. Catalog files convey no Git,
queue, source, shared registry or integration authority.

Primary receipt and exact inspected-source hashes are in
[evidence-sha256.txt](evidence-sha256.txt) and [source-sha256.txt](source-sha256.txt).
Rehashing these pointers checks their identity; it is not a new replay or full
recursive admission authentication. The retained original readers, frozen source,
subject `evidence.toml` pointers, `out/evidence`, ROM/movie and tooling inputs stay
required. This research changed none of them.

## Reused acceptance

| Existing boundary | Stages and directly exercised domain | Reuse and limit |
| --- | --- | --- |
| Shared synchronous host, Ceres 02–03 | Source `73547105d870f8b05f8c6bcf55b768c223889fdb` merged by `e8c9c3e10ac71bf0377dc00d7de5764a4263b445`; 7 projectile/3 room-display actual-C pairs, full WRAM/returns and ordered PLM/HDMA/17 VRAM flushes. Reporting observation `shared-game-host-6053-current-verified-audit-20261013` is verified/current-checkout. | Durable integration still running. `live_exercised=false`; PLM/HDMA/VRAM remain C Host services. This host exposes only SpawnPlm/SpawnHdma/FlushVram, not complete reset/NMI/IRQ/PPU/APU lifecycle. |
| Controller, ticket 78/Ceres 01 | Submitted `223f8a1b0c7382817b1290d13c7a41f4e722d5e1`; 2,240 original frames match 1,736 completed causal updates/1,887 samples; independent RNG control rejects. | Diagnostic only, integration pending. Final 8,600-frame diagnosis stops matching at 7,740 updates/frame 8,317; divergence ordinal 7,741, frame 8,508/event 63,799, 7,960 C polls versus 7,959 native polls. |
| Samus projectile roots | Current source `c711faea56f20d42113eb63393cc4a76d4940557`, 476 actual-C pairs; integrated observations plus composed live HandleProjectile 97,049/21,746 active, DrawPlayerExplosions2 102,632/21,558 and DrawBombAndProjectileExplosions 96,336/20,998 per mixed subject. | Reuse source and host. `sub_9382FD` has offline evidence without this live-root observation. Ceres drawing refusals/uninitialized upstream locals remain explicit; this does not own complete player movement/input/contact frame. |
| Projectile trails, ticket 139 | `2b62134bfbee7c5dcd4c81616edc5d91e781d44f`, 3,316 controlled pairs plus 14 natural captures; integrated; live root 3,824 exclusive frames/167 active/0 refusals in 9,000-input gate. Independent composed gate preserves native137. | Ticket resolved after retained authentication. Three movement opcodes retain offline stages because no natural hits. Creation stays projectile_flight-owned. |
| Camera and room scroll, ticket 140/room20 | Camera `7512b6479afe55da270931a9f3ae26c9d558e5a5`, 467 pairs, 94,425 whole-root calls/35,580 active/0 refusals. Room scrolling 1,166 pairs; UpdateScrollVarsUpdateMap 3,875 active and CalculateLayer2PosAndScrollsWhenScrolling 94,608 active calls. | Integrated/live bounded domain; preserve camera ownership and X-before-Y order. Room display has separate 17-column host fixtures and no live claim. Complete door/loading/PLM lifecycle is larger. |
| Native137/enemy projectile contacts and rendering | Resolved original native137 admission: SpawnEprojWithRoomGfx, SpawnEprojWithGfx, EprojRunAll and synthetic EprojInstruction fixtures; composed 95,325 scheduler calls/866 active, all sixteen 9642 and four/four A387/A395 births. Separate current observations cover both OAM render and both contact passes. | Preserve exact nine-ID admission, allocator/scheduler/contact/render boundary. Synthetic EprojInstruction adds no lexical definition. Other ID/initializer/lifecycle domains remain visible. |
| Bounded Ridley attack, ticket 138 | `e947e968086a15c170f58472764bf9244b403a2a`, 1,804 controlled pairs/90 natural captures; composed ProcessEnemyInstructions owns 2,393 frames/36 timer-due, eight opcode totals sum to 48 and sixteen resulting 9642 allocations. | Resolved exact A6:E73A..E828/238-byte script; 209,673 other interpreter calls stay C/refused. Internal opcode and initializer evidence imports keep historical-frozen context where appropriate. No broad 220-definition inference. |
| Whole Ridley, ticket 141 | Maintained 220-definition source correspondence translated; frozen v2 6,457 controlled pairs/33 roots and 670 natural Ceres pairs/20 roots. Final frozen candidate offline gate and native137 regression retained. r7 four-subject 38,000-input diagnostic has material Ceres Rust init/main/draw/getaway/script ownership. | Broader integration and formal 129,712 gate never started. Only 574/670 natural calls pass public selection, with 92 fixture-only states and four getaway refusals; seven of eighteen supported opcode counters remain naturally zero at r7. Real boss/explosion domains remain unproved. |
| Audio/system/PLM/enemy leaves | Resolved tickets and maintained modules establish the exact bounded leaf families; reporting maps source correspondence separately from actual-C/live observations. | Reuse every admitted leaf; do not regenerate translations to inflate scope or turn historical dispatch fixtures into complete caller acceptance. Later lane source/dependency details are in [later-lanes.md](later-lanes.md). |

The retained composed verdict is
`out/recovery-session/composed-live-release/1791861180321696848-129445/verdict.toml`,
SHA-256 `5e224bda08b3f597b07dd3a03c59bf8cf136b288993459c62e453ef3fac400c2`;
post-cleanup authentication is
`.scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml`,
SHA-256 `9251ee29114c3ac5ea7da19279050ac7e83227cb790e624c5b6a0900372f3845`.
Its raw 129,712-input route ends in Ceres DF8D/state 8, Samus 173/159, RNG 13022.
It establishes mixed C/Rust parity, not original-ROM escape or credits. Source:
[recovery-live-completion-20261013.md](../../decomp-port/recovery-live-completion-20261013.md)
and resolved tickets 137–140; stale earlier audit tables do not reopen them.

## Whole ownership lanes and remaining work

Writable files below are exact proposed envelopes, not concurrent claims. A
native scope must seal existing-file hashes and reviewed dependencies before
dispatch. Shared `crates/port/src/lib.rs`, canonical `memory.rs`/ROM access,
pipeline CLI/catalog/ABI/callback/observer wiring and inventory belong to the
sole integrator. The current WRAM interface has no general SRAM fixture/view.
Lane-owned new tests/manifests must be registered by that owner.
No implementer can replace shared registration wholesale from frozen source.

| Lane / child | Exact implementer envelope after reviewed promotion | Dependencies and admission |
| --- | --- | --- |
| Scheduler/reset/controller, 11→12 / milestone 06 | Current controller five-file hold unchanged: `crates/pipeline/src/decomp_prefix.rs`, `prefix_observer.lua.in`, `prefix_compact_observer.lua.in`, `crates/pipeline/tests/prefix.rs`, `docs/DECOMP_PREFIX.md`. Extra-poll continuation additionally proposes `tools/decomp/prefix_host.c`, exclusively reviewed under the existing ticket78/infrastructure owner. | 11 completes held host then controller integration; 12 proves the NMI enable/disable/setup continuation behavior at the exact mismatch. This pipeline/observer/C-template work is infrastructure, not native gameplay scope. Detailed next commands/controls are [ceres-continuation-holding.toml](ceres-continuation-holding.toml). |
| Shared host/hardware lifecycle, 13 / 06–08 | Existing held `crates/port/src/game_host.rs`, `crates/port/tests/game_host.rs`, `tools/port/game-host-projectile.toml`, `tools/port/game-host-room-scroll.toml`; prospective new `crates/port/src/hardware_lifecycle.rs`, `crates/port/tests/hardware_lifecycle.rs`, `tools/port/hardware-lifecycle.toml`. No ownership of lib.rs or shared adapters. | Host integration remains prerequisite to mutation of held files. Research a complete accepted-NMI/VRAM or reset lifecycle boundary and shared transaction contract, preserving unchanged Samus/room consumers. Actual-C ordered reset/DMA/NMI/IRQ/PPU/APU effects, pre-host refusal/no writes and fatal-after-host/no retry controls. No observed complete NMI-root receipt exists in the current bank86 promoter. |
| Samus movement/collisions/player projectiles, 14 / 07 | `crates/port/src/samus.rs`, `samus_control.rs`, `samus_projectile_frame.rs`, `projectiles.rs`, `projectile_flight.rs`, `projectile_special.rs`, `projectile_trails.rs`, `power_bomb.rs`, `collision.rs`; exact matching tests and `tools/port/samus-frame.toml` proposed by 14. Camera files excluded from this lane. | Whole player beta/frame root and input/contact calls reuse existing helpers and host. Identify live observed caller and complete dependency closure first; full WRAM/returns, integer wrap/alias/transient/order controls and material route exercise. Preserve accepted projectile/trail domains. Exact future scope selects only needed files from this envelope, with no competing worker. |
| Rooms/doors/camera/PLMs/assets, 15 / 07 | `crates/port/src/room.rs`, `room_scroll.rs`, `camera_scroll.rs`, `plm.rs`, `plm_instructions.rs`, `plm_render.rs`, matching exact tests; proposed `crates/port/src/door_transition.rs`, `crates/port/tests/door_transition.rs`, `tools/port/door-transition.toml`. | Research full door/load transition or whole PLM scheduled boundary; shared hardware/state/ROM dependencies stay explicit. Retain resolved camera domain. Native observed calls, validated ROM data extents/digests, complete coroutine continuation/graphics effect order, actual-C and four-subject live transition plus original-ROM comparison. |
| Whole Ridley continuation, 16 / 07–09 | Existing ticket141/bridge exact port envelope: `crates/port/src/ridley_subsystem.rs`, `ridley_subsystem_admission.rs`, `ridley_subsystem_instructions.rs`, `ridley_movement.rs`, `ridley_rendering.rs`, `ridley_cutscene.rs`, `crates/port/tests/ridley_subsystem.rs`. All pipeline/source/template/CLI/family integration paths remain held by sole integrator as enumerated in the bridge/handoff. | Continue original held candidate; 16 tracks its original work, never duplicates it. Review per-root header/area/window/timer/frozen/material/PPU-domain admission before integration; preserve native137 nine-ID/allocation admission, original/native image distinction and exact authorized operand patch. Freeze focused merged source, refresh affected families, run original formal four-subject 129,712/material/fallback gates with bounded 3,600s integration limit. Missing contexts remain refused/unproved. Broad real-boss/formal141 integration is not a default06blocker. |
| Other enemies/bosses/projectiles, 17 / 07–09 | One owner per successive family. Existing `enemy.rs`, `enemy_ai.rs`, `enemy_instructions.rs`, `enemy_rio.rs`, `enemy_jump.rs`, `enemy_crawler.rs`, `enemy_projectiles.rs`, `enemy_projectile_instructions.rs`, `enemy_projectile_collision.rs`, `enemy_projectile_contacts.rs`, `enemy_projectile_rendering.rs`, `enemy_projectile_sparks.rs`, `observed_projectile_ownership.rs` and matching tests/manifests. Ridley envelope excluded. | Select a complete observed enemy lifecycle/script/contact/render boundary outside retained Ridley scope. Allocation/room/player-contact/host dependency evidence is mandatory. Dynamic function/script dispatch, EnemyItemDrop/SpawnEnemy footprints, ROM lists, all relevant state/effects and exclusive Rust caller ownership must be reviewed; no leaf-count caller inference. |
| Audio/UI/menus, 18 / 07–08 | `crates/port/src/audio.rs`, `system.rs`, `crates/port/tests/audio.rs`, `system.rs`; proposed separate `crates/port/src/audio_frame.rs`, `crates/port/tests/audio_frame.rs`, `tools/port/audio-frame.toml`, and UI files only after 18's exact contract. | Existing queue/system leaves stay reused. Research HandleMusicQueue + complete sound-effect acknowledgment boundary separately from pause/file/options menus; record source's modified native acknowledgment semantics. Host APU/reset and canonical player state required. Ordered port reads/writes/queues, real acknowledgment, menu inventory/state and original-ROM controls; leaf queue tests cannot admit full APU/menu state machines. |
| Persistence, 19 / 08 | Proposed `crates/port/src/persistence.rs`, `crates/port/tests/persistence.rs`, `tools/port/persistence.toml`. SRAM/canonical accessors remain integrator-owned. | Research SaveToSram/LoadFromSram transaction + checksum/slot/packed-map dependency closure; do not invent modern save format. Complete actual-C WRAM/SRAM/return/error controls, ordinary reset plus fresh-process save/load/continue and original-ROM resource/room state. Include Ceres GameState33 save dependency; no credits prerequisite for research. |
| Credits, 20 / 08–09 | Proposed `crates/port/src/credits.rs`, `crates/port/tests/credits.rs`, `tools/port/credits.toml`, exact ancillary files only after 20's root/closure research. | Research 8B credits cinematic root, indirect handlers, decompression/DMA/VRAM/audio/SRAM dependencies and selected adapted any% completion route. Observed whole-root provenance and rendering/data extents, independent subjects and controls, material state/effects, original-ROM comparison, input adaptations and remaining fallback. No existing credits replay or complete Rust credits module was found. |

## Native scope readiness and dependency graph

No new `translation-scope-v1` document is published by this preparation. Existing
accepted boundaries are reused, so creating a new open task for their unchanged
implementation would duplicate work. For missing caller boundaries the evidence
facts are incomplete. `crates/pipeline/src/translation.rs` requires 1–32 unique
coherent routines reachable from measured roots; all dependency tickets resolved,
review of every lexical/indirect/external dependency, current source/ROM/movie
and exact writable-file hashes, and executable pre-integration/integration gates.
Its `Exercise` reader currently authenticates **schema 2 bank86 projectile
inventory** (`projectiles.tsv`, ID/init/pre/instruction counters, pinned host/
instrumented-bank/binary/build/replay artifacts). It cannot substitute Ceres
controller provenance, Samus beta, door/NMI/UI/SRAM or 8B credits receipts. A
callee hit is insufficient for its caller. Each research child must either locate
a genuinely supported matching receipt or request an explicit reviewed Rust
exercise-contract extension under the sole integrator before promotion.

Scope input/gate constraints come directly from `translation.rs` Scope,
inventory(), exercise(), validate_scope() and verification_command(); SHA-256
`7d581d67f3bda1526ad0815c7f6c149ddf7874e9749d75ac13fc428b3e6f85cd`.
Promoter paths are exact `.rs`/`.toml` under port source/tests/tools-port, and
allowed gates are Cargo tests or existing Rust verification operations.
Pipeline/observer/docs/C-template writes and `decomp-prefix` are unsupported.
The custom holding format cannot be passed to `assign --scope`.

| Child | Type / tracker blocker | Operational dependency (preserved beyond local numbers) | Composed consumer |
| --- | --- | --- | --- |
| 11 | task / 04,05 | Old integrate-shared-game-host → old integrate-compact-controller, under original `ceres-20261013` tokens and checked authority | 06,13 |
| 12 | research / 11 | Ticket78 causal seam, source freeze from 11; observed NMI/setup failure and new checked scope before any mutation | 06 |
| 13 | research / 05 | Reuse held host; inspect hardware contract without taking its files; implementation waits 11 and reviewed exercise extension | 06,07,08 |
| 14 | research / 05 | Reuse resolved player/projectile/collision dependencies; eventual source promotion waits required 13 contract | 07,09 |
| 15 | research / 05 | Reuse resolved room/camera/PLM evidence; eventual source promotion waits required 13 contract | 07,09 |
| 16 | task / 04,05 | Preserved ticket141,137,138 gates/ownership; formal gate and focused source integration only after checked gameplay handoff | 07,09; 06 only for independently exercised narrow Ceres dependencies |
| 17 | research / 05 | Allocation/contact/room/host contracts, disjoint non-Ridley family; eventual promotion waits relevant 13–15 facts | 07,09 |
| 18 | research / 05 | Existing audio/system leaves, APU/reset contracts; eventual implementation depends on required13 and player/UI inputs | 07,08 |
| 19 | research / 05 | Current canonical WRAM/SRAM, Save/Load dependencies; implementation depends on 13 and accepted player/room continuation | 08 |
| 20 | research / 05 | 8B scripts/data/host/audio/persistence and researched completion route; implementation depends on needed13,18,19 facts and07 | 08,09 |

Research children 13–15 and 17–20 may run concurrently after this catalog under
read-only authorization, even while composed milestones are blocked. Their
completion must generate exact task/scope children and add those children to
milestones before route completion is claimed. Tasks 11 and 16 are tracking
continuations of held work, never alternate launch lanes. Available subagents,
canonical capacity 3 and shared process workers 4 bound future dispatches.

The integration-owner patch proposal is
[milestone-map-proposal.md](milestone-map-proposal.md). It adds these blockers
to 06–09 and the map; this worker has intentionally not edited shared plan files.
Ticket05 research can resolve after that proposal is applied because actionable
holdings are its acceptance. This catalog establishes no gameplay milestone.
For successors, [later-lanes.md](later-lanes.md)'s eight precise new-module
triplets refine the broad existing-module envelopes: `samus_input_frame.rs`,
`samus_movement_frame.rs`, `room_transition.rs`, `pause_menu.rs`, `file_menu.rs`,
`audio_frame.rs`, `save.rs` and `credits.rs`, each with matching test/manifest.
Research children choose one resulting exact triplet or a reviewed alternative;
this preparation does not authorize both competing file layouts. Reusing existing
modules read-only is the preferred disjoint seam until a material change requires
their owner to revise the source envelope.

## Gates and completion policy

In each future assigned worktree, first execute the minimal semantically relevant
Cargo test(s) (at least one executed) and actual-C family gates for that boundary.
Use `cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify
--manifest tools/port/REVIEWED.toml --output ASSIGNED-RUN` only after the manifest
is registered/reviewed. Initial experiments are at most 60s including shared
worker waits; limits never waive incomplete results. New manifests in this
catalog are proposed, not executable acceptance commands today.

After one serialized source merge/freeze, the integrator runs the existing
conservative selector: `cargo run -p pipeline --locked --offline --bin sm-pipeline
-- batch verify --profile offline-all --changed-since APPROVED-BASE --case-jobs 4`.
Unknown dependency inputs remain affected. Run required independent four-subject
live gates from the reviewed scope (existing `live-verify`/`routine-live` receipt
contracts), including material exclusive Rust ownership, periodic/terminal
state, full declared WRAM/SRAM/host and ordered PPU/APU effects, negative controls,
remaining C and CPU fallback. Add original-ROM comparisons at the researched
matching gameplay boundaries; no raw TAS parity assumption or state correction.
The existing composed acceptance is retained while new contexts remain unknown.

After all writers stop, apply both scoped cleanup passes, retain frozen
source/store/pointers/readers/private inputs, remove integrated worktrees,
authenticate family/live receipts again and publish each achieved stage
independently. The old records remain historical; new source epochs receive new
observations. Do not introduce undeclared exclusions or reuse based on names.

## Investigation cleanup

No gameplay, fixture, replay, build or test run was created by this catalog.
Only source/receipt reads, SHA-256 identity inspection and research artifacts
were produced. Cleanup utility invocation is unnecessary for this no-run step;
0 bytes reclaimed, no disposable worktree/build output created or removed.
Filesystem available space measured during inspection: 223,134,560,256 bytes.
All held source, original readers, shared evidence objects and private inputs
were preserved. Final artifact hash inspection is recorded in ticket05.
