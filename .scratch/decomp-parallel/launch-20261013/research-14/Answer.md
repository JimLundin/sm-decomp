# Samus frame research Answer

Status: research complete; implementation remains blocked by the dependencies below
Blocked by: None
Type: research
Owner: `/root/launch_policy`, real worker `01a13571-e61e-7e31-8e29-d8f82488f33f`
Assigned source baseline: `40a8489856f36d21925188823e33b520f74d4cb4`

This answers [ticket14](../../issues/14-samus-frame-scope.md) using retained source and evidence only. The exact reserved production claim is in [manual-handoff.toml](manual-handoff.toml) and [next-2.stdout.toml](../receipts/next-2.stdout.toml). No new gameplay stage is established.

## Answer

The next complete movement/contact boundary is `HandleSamusMovementAndPause` ($90E722) → `CallFrameHandlerBeta` → normal `Samus_FrameHandlerBeta_Func17` ($90E725). Its full sequence is: reset contact damage; movement; minimap; gamma; animation; hit interruption; block-collision transitions; pose transitions; palette; periodic damage; pause request; low-health sound. Reusing `Samus_MovementHandler_Normal` implements one dependency, not this complete scheduled boundary. Source: [sm_90.c:5984](../../../../deps/snesrev-sm/src/sm_90.c#L5984), exact beta definition SHA-256 `5a461686f97d25739ec2b4aede6953c58e386269b42a7639694e89d34e74e9f1`.

The input producer is a distinct complete `alfa` boundary: `HandleControllerInputForGamePhysics` ($90E692) → `CallFrameHandlerAlfa` → normal `Samus_FrameHandlerAlfa_Func11`. It clears all three pending poses and their transition indices, sets radius, calls the input handler, updates suit/gravity, detects the inside block, processes HUD/projectiles, then runs `Samus_Func10`. The demo variant changes ordering and saves filtered input; the projectile-only variant is separate. `Samus_CallInputHandler` dispatches normal/demo/frozen/autojump handlers; normal `Samus_InputHandler` indexes 28 movement types. Source: [sm_90.c:5930](../../../../deps/snesrev-sm/src/sm_90.c#L5930), [sm_90.c:6120](../../../../deps/snesrev-sm/src/sm_90.c#L6120), [sm_91.c:46](../../../../deps/snesrev-sm/src/sm_91.c#L46). Ticket78/child12 owns causal poll/NMI work; this proposed module consumes it.

`GameState_8_MainGameplay` fixes the composition: alfa → player/projectile interaction → EnemyMain → optional message-box coroutine → beta → enemy projectiles → PLM coroutine → enemy projectile contacts/power-bomb contacts → camera/drawing → HUD/room/timers. Therefore beta is not every enemy contact or a complete gameplay frame, and alfa/beta mutations of pose/input state must serialize. Source: [sm_82.c:847](../../../../deps/snesrev-sm/src/sm_82.c#L847). Shared scheduler wiring belongs to the integrator.

## Dependencies and exact dynamic domains

| Dependency | Native owner and required review | Existing Rust / unresolved boundary |
| --- | --- | --- |
| Movement | `CallSamusMovementHandler`, sm90:6151, has 28 named/no-op cases including normal, grapple release, drained, shinespark, crystal flash, knockback, bomb jump and Xray. Normal sm90:1849 indexes `kSamusMovementHandlers[28]` and skips movement while frozen. | Reuse [collision.rs:1109](../../../../crates/port/src/collision.rs#L1109), `samus.rs` physics, and the real collision Host. No Rust owner for the complete outer movement pointer dispatcher. |
| Gamma | `CallFrameHandlerGamma`, sm90:5621: 11 cases including two no-ops, timer/time-up, Ceres Ridley pushout, Draygon, drained and other state continuations. | No complete mapped Rust owner. Ceres timer/pushout needs reviewed boss/host contracts, not a substituted no-op. |
| Animation | `Samus_Animate`, sm90:45; pose/delay ROM tables and finite opcodes. | Reuse [samus_control.rs](../../../../crates/port/src/samus_control.rs); only selected admitted animation branches. |
| Hit interruption | sm90:5349/5380: 28-entry `kSamusHitInterrupts`, frozen/debug invincibility, knockback and bomb-jump branches; `SetupBombJump` sm90:5544 and SFX queues. | Root and SetupBombJump unmapped. Preserve branch order and new/interrupted/transitional pose effects. |
| Collision-to-pose | sm91:2293/2302: six-way low-byte selector; landing subdispatch uses high byte and six entries; `word_91E921`, `word_91EB74` and pose params. | Root unmapped; use terrain/solid-enemy leaves without replacing full pose transitions with those leaves. |
| Pose transitions | sm91:2500/2536: precedence transitional → interrupted → walked-into/new pose, A[9]/B[11]/C[9] dispatch. Nested B1[28], A5[7], A5_1[6]; F404/F433, changed-pose collision, jump, palette and animation reset. | Complete transition/F404/F433 roots unmapped. Previous facing+movement word alias and final `input_to_pose_calc=0` must match. |
| Minimap | sm90:2230/2319: room/map coordinates, wrap masks, explored-map byte/unaligned-word loads, area data at $829717 and tilemap pointers at $82964A. | No mapped root. Room/camera leaves do not own minimap/HUD tilemap construction. Pin every area/table extent; review native locals before exposing malformed direct-C fixture domains. |
| Palette | sm91:1392: `off_91D72D[11]` includes null entries 3–5, indirect misc code, charge/screw/boost/shinespark/crystal/Xray/visor palettes. | Reuse CopyToSamusSuitPalette; full root/charge/misc handlers unmapped. Null selectors require refusal before C/Rust effects. |
| Damage/pause/health | sm90:6218/6238/6251: signed 16.16 periodic damage, suit scaling/health clamp, frozen reset; pause gates for power-bomb/frozen/door/area6/state8/Start; health threshold31 and SFX3 queue. | Roots unmapped. Pause is a request to state12, not pause-menu ownership. Audio queue leaves can be reused; hardware playback is separate. |

The dynamic table inventories and absence of complete owners are extracted from primary registry/source correspondence in [source-observations.json](source-observations.json), with pinned bytes in [inputs.sha256](inputs.sha256). No beta/alfa/input/transition root has a progress observation in that snapshot. Lexical inventory omissions of indexed calls do not remove these reviewed dynamic dependencies.

Canonical byte aliases are particularly material: facing at $0A1E and movement type at adjacent $0A1F are sometimes copied as one word; pending poses/indices occupy $0A28–$0A32; alfa/beta pointers $0A42/$0A44, periodic subdamage/damage $0A4E/$0A50, movement/gamma pointers $0A58/$0A5A, input pointer $0A60, and packed collision-to-pose input $0DC6. Source: [variables.h:446](../../../../deps/snesrev-sm/src/variables.h#L446). Pin $91B629 pose params, $91B010 animation pointers, $919EE2 input transition pointers and the palette/map extents; word-width fixed-point addition and partial-byte writes cannot be replaced with wider state fields.

### Collision host seam

`collision::Host::spawn_plm` takes `&mut i32` displacement as well as latest full WRAM and returns a byte. Current `game_host::Request::SpawnPlm { header }` carries no displacement. A beta worker must reuse the original collision adapter or obtain an explicit reviewed extension under child13. Passing this service through the existing header-only request would lose admitted synchronous displacement behavior. Source: [collision.rs:88](../../../../crates/port/src/collision.rs#L88), [game_host.rs:12](../../../../crates/port/src/game_host.rs#L12), [collision tests](../../../../crates/port/tests/collision.rs#L35). Preserve full private WRAM, pre-host refusal and fatal-after-host/no native retry. No redesign or host-file ownership is authorized by this research.

## Existing domains to preserve

| Existing module | Translated correspondence | Retained verified / live domain | Current integration boundary |
| --- | ---: | --- | --- |
| samus.rs | 27 symbols | Ticket05: 5,184 full-WRAM/return pairs; historical verified27/live10 roots. | No current integrated/live module observation. Physics is not full pose/frame ownership. |
| samus_control.rs | 61 | Ticket08: 4,528 pairs for 46 historical fixture roots/live16 roots. Further mapped15 are correspondence only. | No current integrated/live module observation. Frozen input can explicitly refuse a missing pose dependency. |
| collision.rs | 98 | Ticket12: 1,752 pairs for63 roots, all28 normal movement handlers. Historical live5 roots: 12,021 calls, 10,791 normal dispatches, 996 right movements, 212 changed-pose collision checks, 11 vertical-speed and11 bomb-jump calls. Terrain PLM calls were zero. | No current integrated/live module observation. Never claim PLM live acceptance from that replay. |
| samus_projectile_frame.rs | 8 | Current integrated4 roots/476 pairs. Composed live3 roots: HandleProjectile 97,049/21,746 total/active; DrawPlayerExplosions2 102,632/21,558; DrawBombAndProjectileExplosions 96,336/20,998 per mixed subject. | Four independent subjects/129,712 raw inputs ended Ceres DF8D/state8. `sub_9382FD` remains Offline/integrated without this live root. |
| projectile_trails.rs | 4 | Current integrated4/3,316 pairs plus14 separately frozen natural captures. Live HandleProjectileTrails: 3,824 exclusive frames/167 active per mixed subject, no refusals. | Three movement opcodes had zero direct natural hits and retain Offline stages. Creation belongs to projectile_flight. |

Primary receipts: [tickets05](../../../decomp-port/issues/05-samus-physics.md), [08](../../../decomp-port/issues/08-samus-animation-input.md), [12](../../../decomp-port/issues/12-samus-collision-movement.md), [139](../../../decomp-port/issues/139-projectile-trail-frame-ownership.md), [current integration audit](../../../decomp-port/progress-current-integrated-stage-evidence-20261013.md) and [live completion](../../../decomp-port/recovery-live-completion-20261013.md). Source-observations retains the registry update identities and sealed evidence pointers; historical observations are not current-source reauthentication.

Reuse accepted projectile flight/special/power-bomb/trail bodies unchanged. Selected Ceres player/explosion drawing branches retain explicit refusal because the pinned C reads uninitialized locals; do not copy undefined behavior or turn helper/root counters into later-route exercise. Existing live gates do not establish ordinary-reset Ceres escape, original-ROM movie parity or later progression.

## Exact next files, gates and children

Choose the existing catalog's dedicated beta triplet: `crates/port/src/samus_movement_frame.rs`, `crates/port/tests/samus_movement_frame.rs`, `tools/port/samus-movement-frame.toml`. Existing player/room/camera/host modules remain read-only dependencies. The proposed orchestrator list in [scope-holding.toml](scope-holding.toml) has ten symbols, with missing services explicitly unready. The complete dynamic closure may exceed32; schedule successive whole boundaries after direct observations and dependency review. This file is not `translation-scope-v1` and cannot authorize `assign --scope`.

The input producer's later separate triplet is `samus_input_frame.rs`, `tests/samus_input_frame.rs`, `tools/port/samus-input-frame.toml`. Shared lib/state/ROM/callback/catalog/live wiring and GameState8 stay integrator-owned. Do not take both broad old-module and new-module envelopes concurrently.

Concrete proposed beta checks, only after a manifest/module is reviewed and registered:

```sh
cargo test -p port --locked --offline --test samus_movement_frame
cargo test -p port --locked --offline --test samus --test samus_control --test collision
cargo test -p port --locked --offline --test samus_projectile_frame --test projectile_trails --test game_host
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/samus-movement-frame.toml --output ASSIGNED_RUN
```

Initial checks/experiments retain60s; integration gates3600s. Require at least one executed semantic test. Fixtures must compare full WRAM/raw returns, transient alias effects and ordered synchronous host effects through actual compiled C, with rejection controls for wrong sequence, missing movement, ignored displacement, byte alias overwrite, transition priority, frozen movement/damage, invalid/null selectors, health31 crossing and pause request gating. Ordinary family WRAM/return equivalence alone does not establish ordered hardware effects; reviewed callback/live evidence must supply them.

Integrator freezes merged source and runs conservative `batch verify --profile offline-all --changed-since APPROVED_BASE --case-jobs 4`, then required four-subject material live/original-ROM comparisons. Explicit counters must identify beta/alfa/direct dispatch roots, admitted selectors, Rust ownership/refusal, remaining gameplay C/CPU fallback and the first divergence. Existing Ceres input replay cannot be relabeled normal progression. Original-ROM and new beta live argv remain blocked on a matching exercise/capture adapter; no unsupported flags are proposed.

Publish the [exercise-contract child](proposed-children/01-samus-frame-exercise-contract.md) and [beta-closure child](proposed-children/02-samus-beta-frame-closure.md), renumbering them in the shared tracker through root. Add those dependencies and needed13 host/input/pose boundaries to milestones07/09 before declaring implementation or route completion.

## Promotion blockers and cleanup

The native reader [translation.rs:400](../../../../crates/pipeline/src/translation.rs#L400) admits schema2 bank86 projectile provenance only: projectiles.tsv, sealed instrumented sm86/headless build, original source, binary, ROM/movie, naturally positive ID/init/pre/instruction counts; it constructs addresses with $860000. A measured movement leaf or existing composed projectile root cannot authenticate beta. Addressless dispatchers also need exact symbol/source definition identity, not invented addresses. Native planning additionally reports stale source inventory (`routine lexical dependency inventory differs from pinned source`) in the assigned receipt. Review/regenerate inventory through the integrator after the maintained reader matches it; do not rewrite pinned historical receipts.

No new run, build, fixture, replay, worktree or verification output was created. Only source/receipt reads, JSON extraction, SHA-256 checks and assigned scratch notes were produced. Run cleanup is not applicable for this no-run work; reclaimed0 bytes; all retained source, store, subject pointers/readers and private ROM/movie inputs remain intact. Final available space and inspected manifests are recorded in [cleanup.toml](cleanup.toml). Root owns result admission, tracker/inventory publication and Git checkpoint.
