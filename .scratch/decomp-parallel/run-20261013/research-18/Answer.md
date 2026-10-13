# Audio acknowledgment and UI/menu research

Research is complete for the claimed question. The recovered C supports a nine-routine audio service boundary, separate equipment rendering and interaction boundaries, and separate options-object and options-screen boundaries. Full pause/map and file/file-map orchestration still need successor research. Original ROM acknowledgment waits materially differ from the pinned C; actual-C parity alone cannot establish original audio behavior.

No gameplay source was changed, no new Offline family or Live replay was run, and no gameplay stage was earned. The coordinator must publish the proposed children to consumers 07/08, supplement shared provenance ticket 21, authenticate this package through the existing evidence store, and then admit the exact research result.

## Claim, source and evidence identity

- Claim: `ticket-2966e528f4696d00eed786e3a895d9034fd217c5ad6db59153329ef23da35b6d`, revision `2b5be5ee42e86f922e679effe3968f60fc7d94afffa98e078f88a1c6107fa81d`, attempt 1, coordinator `decomp-run-20261013`.
- Real worker UUID: `01a135ca-3b5d-75a2-b717-23d486d7b6ec`; parent UUID: `01a13592-d02b-7143-8489-a15b03f19da2`. The [successful started retry](../receipts/start-research-18-retry.stdout.toml) records this worker and exact claim as running. No further claim or authority mutation was performed here.
- The returned research Job has empty writable paths/workspace and no `base-head`. Its initially inspected source baseline was `7ed3d383d6d3e28d21d1bb960acc5576ce82f5e9`; the continuation observed `52f44a1a7599f540397dfbd24a31ff8b3ef5f19e`, and the final read-only source observation was `3a8b9ea28b716d90df55f798941bba68e1df6531` at 15:52:12 UTC. Committed baseline-to-final changes are `crates/pipeline/src/progress.html.in` and tracker/dashboard artifacts, leaving the inspected C, Rust and inventory unchanged in that comparison. Tracker files are concurrently mutable and require final coordinator seals.
- Source archive revision: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`. Inherited inventory seal: `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`; source identity: `e74864c229949fc6f1a6bd7528f339bdb3f9364f1e4f9544f08693881d10f111`.
- Inherited `sm_80.c`/`sm_81.c`/`sm_82.c` seals are `ab9abc1042ea876c51de90d00632d1af13e9a765c77cf371950ddcdc783130c7`, `bd05747f344ac18d4f5230e10fd4678b087048c24e8aeb5f685691b36badb1a0`, and `afa8b6fb9e9e5ad378db4779c1c39574856a57d50d25c4536aa2bafeeb79e3ad`. They are declarations read from retained source evidence, pending current evidence-store authentication.
- Original ROM (3 MiB): `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`. Retained patched native image (4 MiB): `40aaebb25b9646bea2dffdc5e1ae680768a39dd122de9353885e1c7456ceae27`. Recorded movie: `cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e`. [Static range observations](rom-observations.json) inherit these whole-input seals from research 15; they are not runtime or fresh authentication receipts.

[Routine observations](routine-observations.json) retain 132 exact inventory definitions, definition hashes, reviewed dynamic selections and the five proposed routine sets. [Input paths](inputs.paths) list 112 dependencies for coordinator retention; [dependency review](dependency-review.json) maps their actual ownership and current readiness. [Inherited seals](inherited-input-seals.toml) distinguish declared historical identity from current authentication. The actual [REA capability observation](rea-capability-observation.json) reports no open target and an unavailable native/Ghidra provider; the original-ROM slices were ordinary static shell observations. No REA native analysis or original CPU/SPC timing was observed.

## Current bounded progress

Resolved decomp-port [02 system](../../../decomp-port/issues/02-system-state.md) and [03 audio queues](../../../decomp-port/issues/03-audio-queues.md) provide reusable bounded leaves in `crates/port/src/system.rs` and `audio.rs`. The retained coverage audit reports historical actual-C system coverage of 17 routines/475 pairs and audio coverage of 21 routines/1,147 pairs in the [retained coverage audit](../../../decomp-strategy/retained-coverage-authentication.json). Audio verdict seal is `d8944be07479cd5bb535108b3ea4a73fdd31a66cc1f5617c045ecfa8b9f46e14`; system verdict seal is `71a515f9853c74c2753246fe7fe96a837ff581da482d44e1e6f15485e32dab96`. Historical module seals are respectively `4308e496267857ca869db45f4127e505bec4ad29c4c9ede8f08d6e9769ccf69d` and `bedc7662f4c1c435c968a6c07c73331a76f6c58621b4ca885f73f6be33ccbf16`.

These historical results do not cover `HandleMusicQueue`, `HandleSoundEffects`, complete UI roots, or playback. No registry stage was found for the newly proposed roots at inspection. Existing [109 room pause hooks](../../../decomp-port/issues/109-room-pause-hooks-and-shaktool.md) is a dependency for full pause restoration, not completion of the pause menu. Controller [78](../../../decomp-port/issues/78-original-movie-controller-boundary.md), current child 12 and the retained Ceres controller handoff own actual polling/VBlank/ScreenOn timing. Research 19 owns SRAM semantics; 21 owns the shared native-root producer; 22/23 own the hardware/NMI contract. Parent completion 11 and frozen all-affected verification remain required before shared source work.

## Complete pinned-C audio boundary

| Routine | Declared original entry | Inherited definition SHA-256 |
| --- | --- | --- |
| [APU_UploadBank](../../../../deps/snesrev-sm/src/sm_80.c:16) | 0x808028 | `32aec38ebfa1bf4b44005479c7527547a5d64adebdaa22b149af89caa7b8f47b` |
| [HandleMusicQueue](../../../../deps/snesrev-sm/src/sm_80.c:646) | 0x808F0C | `f635d911c2c92f687fc62c3eee96617f10f92fadb01d8e9700101980ebeaab2f` |
| [HandleSoundEffects](../../../../deps/snesrev-sm/src/sm_82.c:738) | addressless inventory entry | `455dfd8579639a95fe3e4e08833b94813f64576dfdfd57df11d0cd418783fc6d` |
| [ResetSoundQueues](../../../../deps/snesrev-sm/src/sm_82.c:791) | 0x828A9A | `53a4c43e98176d230cb1a163909992d28ccb5f96d511573b49a9fbe432503f2b` |
| [SfxHandlers_0_SendToApu](../../../../deps/snesrev-sm/src/sm_82.c:753) | 0x828A2C | `4965c0cb4c9cd1ca226aca43f4a88f8f071e8e7e4533a9f4c149d2630027a8bb` |
| [SfxHandlers_1_WaitForAck](../../../../deps/snesrev-sm/src/sm_82.c:764) | 0x828A55 | `4198f7478a20df511fdca15887ba96e3f073b00af5855b8a8b634fe51d680316` |
| [SfxHandlers_2_ClearRequest](../../../../deps/snesrev-sm/src/sm_82.c:771) | 0x828A6C | `e1ae710260a7e16bc486bfb4985a311b01b4a10147225c0e19f335909db90574` |
| [SfxHandlers_3_WaitForAck](../../../../deps/snesrev-sm/src/sm_82.c:779) | 0x828A7C | `6df227d31f1218c16d6790cf04f6cd98d61f38ca437d7798a013e47643fb9195` |
| [SfxHandlers_4_Reset](../../../../deps/snesrev-sm/src/sm_82.c:786) | 0x828A90 | `37a4f8f7d6f1d4debe9e1242d93d79fba378d4a19c7f1b7ac10753d0a49bb45a` |

Exact future gameplay files are `crates/port/src/audio_frame.rs`, `crates/port/tests/audio_frame.rs`, and `tools/port/audio-frame.toml`. The audio/system queue leaves are read-only dependencies. Parent owns public state/ROM/lib wiring, callbacks, fixture ABI, root dispatch, manifests/catalog and shared registration.

`HandleSoundEffects` is addressless in the current inventory. The primary names file and static original bytes identify code at `$8289EF`; the five-entry ten-byte handler table actually starts at `$828A22`. The C table comment labels `$8289EF` incorrectly as its own address. Preserve the existing addressless source/definition identity until shared ticket 21 admits a reviewed provenance extension; do not assign the table or a callee's address to this root.

Scheduling is a separate caller obligation. [HdmaObjectHandler](../../../../deps/snesrev-sm/src/sm_88.c:463) calls music before game-state dispatch. [RunOneFrameOfGame](../../../../deps/snesrev-sm/src/sm_82.c:702) calls SFX after a completed inner frame. The [message coroutine](../../../../deps/snesrev-sm/src/sm_85.c:307) invokes both around multiple waits, and `Play20FramesOfMusic_Async` invokes music before each NMI. An adjacent synthetic audio pair cannot replace these schedules; the ABI must invoke each real root at its original scheduling point and preserve yield/resume state.

The nine routines must preserve:

- Unsigned 16-bit music timer postdecrement. Old value 1 sends/uploads; nonnegative resulting values otherwise return. Underflow loads the next queue entry/timer or zero.
- Bank-entry high bit and low-byte **byte offset** into the 25 three-byte music pointers at `$8FE7E1..$8FE82B` (75 bytes), including full selected pointer and upload stream extent. `cur_music_track` is byte `FF` during service and zero afterward. Bank upload dequeues and sets downtime 8, then returns without loading the next timer. Ordinary entries set the word track index from low seven bits, write APUI00, set the byte current track, dequeue, and load the next timer.
- Music ring cursors are even word offsets 0..14. Three SFX rings have byte cursors 0..15 and channels execute in ascending order. A send writes first, updates current request/cursor and advances state. Delay bytes postdecrement with wrap.
- Downtime tests `(int8)(sound_handler_downtime - 1) >= 0`, updates only the low byte of the WRAM word, and writes zero to APUI01, APUI02, APUI03 in that order while clearing the corresponding current bytes. The separate byte `cur_music_track` at `064C` is not part of that three-byte clear.
- `ResetSoundQueues` clears exactly six cursor bytes and three state bytes; ring bytes, current requests, delay/quota bytes and unrelated state remain intact.
- Canonical WRAM aliases: music entry/rings/cursors/timer `0619..063F`, SFX read `0643`, write `0646`, state `0649`, current `064D`, delay `0650`, quota `0653`, rings `0656..0685`, downtime word `0686`, data/track indices `07F3/07F5`, and uploading word `0617`. Preserve byte/word widths and overlap.

`APU_UploadBank` is a gated no-op when `g_use_my_apu_code` is false; otherwise it delegates to `RtlApuUpload`. [SpcPlayer_Upload](../../../../deps/snesrev-sm/src/spc_player.c:1576) consumes word-length/word-target chunks terminated by zero length, wraps target writes through 64 KiB, changes DSP EVOLL/EVOLR/KOF, resets four input ports to FF, reads music pointer `581E`, and sets counter 2. The service contract must preflight the entire selected stream and sentinel before the first effect. ROM mapping, reset, the chosen backend and actual upload completion belong to the infrastructure owner.

## Original acknowledgment and host gaps

Static read of `$8289EF..$828A99` at LoROM file offset 68079 compared 171 original/patched bytes and found the 12 recorded changes. [Patch declarations](../../../../deps/snesrev-sm/src/sm_cpu_infra.c:879) explain them.

| Handler | Pinned recovered C | Original-ROM static behavior |
| --- | --- | --- |
| State 1 at `828A55` | Advances without reading acknowledgment; sets delay 6 | Compares current request with APUI01+channel; mismatch rewrites request/remains 1; match advances and sets delay 2 |
| State 2 at `828A6C` | On delay expiry clears current byte and advances | Also clears the hardware request port before clearing current byte |
| State 3 at `828A7C` | Resets state and immediately re-enters send without reading acknowledgment | Compares cleared current with acknowledgment; mismatch rewrites/remains 3; match resets and re-enters send |

The proposed gameplay implementation must match the declared pinned C domain for actual-C verification. Original-ROM acknowledgment requires its own reviewed source/backend contract and independent timing evidence. Restoring it must be an explicit reviewed extension, with a separate comparison domain and receipt, never an unexplained oracle change.

[RtlApuWrite](../../../../deps/snesrev-sm/src/sm_rtl.c:570) normally records only the latest value per port in a pending four-byte frame entry and suppresses writes for `runningWhichVersion == 2`. During uploading it catches up APU execution and writes inPorts. `RtlPushApuState`/`RtlPopApuState_Locked` use a 16-entry asynchronous queue and can merge/drop empty entries. `RtlApuUpload` resets that queue and invokes the SPC player under the lock. A hardware log must retain the ordered root writes **and** the resulting pending/backend state, rather than infer one from the other.

Headless hosts initialize the player without driving `RtlRenderAudio`/`SpcPlayer_GenerateSamples`; write parity therefore proves no playback or acknowledgment progress. The native [APU read branch](../../../../deps/snesrev-sm/src/snes/snes.c:181) currently asserts before its 32-cycle catchup/outPorts return. Setting `g_use_my_apu_code = false` alone does not make the original acknowledgment route usable. Required infrastructure includes real clock/sample progression, selected SPC/native backend identity, synchronous read/write/upload semantics, full SPC/DSP/cycle/queue state and reset/load restoration. Original CPU sentinel/upload waits and restored controller timing still need runtime capture.

## Separate UI boundaries

| Boundary | Concrete routine ownership | Exact future gameplay files | Required outside ownership |
| --- | --- | --- | --- |
| Equipment rendering | 8 routines, including real `DrawMenuSpritemap` | `pause_equipment_render.rs`, test of same stem, `pause-equipment-render.toml` | ROM/OAM/VRAM image and asset preflight; NMI/DMA service |
| Equipment interaction and root | 29 routines: 28 category/interaction routines plus `EquipmentScreenMain` | `pause_equipment.rs`, test of same stem, `pause-equipment.toml` | Rendering scope, audio queues, canonical player state, actual controller polling |
| Options objects | 29 routines, including real `DrawSpritemap` | `options_menu_objects.rs`, test of same stem, `options-menu-objects.toml` | ROM/OAM image, explicit supported continuations |
| Options screen/root | 29 routines, including `GameState_2_GameOptionsMenu` | `options_menu.rs`, test of same stem, `options-menu.toml` | Objects, audio, controller, SRAM, NMI/MMIO/DMA/decompression |
| Pause/map restoration | Successor research; reviewed MainPause closure lower bound 79 routines | Provisional `pause_menu.rs`, corresponding test, `pause-menu.toml` only after exact closure | Equipment, map/render, hooks, player restore, full hardware and coroutine services |
| File/file-map flows | Successor research; 34/23 table entries with 29/20 unique handlers | Provisional separate `file_menu.rs`/`file_map_menu.rs` triplets only after closure | SRAM, controller waits, map/render/draw/NMI and new-save player reset |

Every source file above belongs under `crates/port/src/`, every test under `crates/port/tests/`, and each manifest under `tools/port/`. Full routine arrays and pinned definition identities are in the observations and [proposed children](children.toml). Adding the actual drawing routines keeps the two rendering dependencies concrete; no new blanket drawing callback service is proposed.

### Equipment interaction/rendering

[EquipmentScreenMain](../../../../deps/snesrev-sm/src/sm_82.c:2410) selects four categories from the low byte of the canonical word at `0755`, then draws selector/reserves and queues the wireframe. Whole category ownership covers reserve AUTO/manual transfer, timers and empty/full health transitions, movement between category/items, equipped/collected bit masks, plasma/spazer exclusion and palette changes. Keep the category/item word, delay `0757`, player bitsets `09A2..09A8`, health `09C2/09C4` and reserves `09D4/09D6` canonical. These are shared player-state aliases; separate UI copies would diverge.

The render scope owns selector animation, reserve digits/palette and partial tanks, wireframe tilemap generation and real OAM spritemap writes plus ordered VRAM queue updates. Preflight every selected sprite/wireframe/animation/palette range and queue extent before effects. Preserve DrawMenuSpritemap's actual `0x1FF` OAM wrap and all selected records; DrawSpritemap has separate pinned clipping/capacity behavior. The [AUTO HUD routine](../../../../deps/snesrev-sm/src/sm_82.c:2583) explicitly reads “garbage” ROM ranges `$82998B/$829997`; preserve pinned behavior for C comparison and declare original comparison separately.

### Pause/map orchestration

[MainPauseRoutine](../../../../deps/snesrev-sm/src/sm_82.c:1314) has eight handlers. The complete pause sequence traverses game states 12→13→14→15 and 16→17→18→8, including the unpause-hook coroutine. GameState 15 also executes ReleaseButtonsFilter(3), HUD and palette work. Required behavior includes actual VRAM reverse-DMA backups, palette/register restore, map scroll/resource state, cancellation of audio, Samus pose/beam restore, HDMA/animtile enable and Kraid/Draygon/MotherBrain hooks. Existing hook leaves cannot replace this orchestration.

The reviewed MainPause closure is already 79 routines after excluding several existing services; it is a lower bound, not an approved complete 1–32 scope. The successor must finish map/assets/drawing/player/hook closure and partition coherent full roots behind accepted dependencies before proposing any new writes. None of the map or restore routines is implicitly owned by the equipment children.

### Options objects and screens

The complete object boundary has eight even slot ids 0..14, descending allocation/iteration, six entry initializers, six prehandlers and seven bytecode handlers. [OptionsMenuFunc2](../../../../deps/snesrev-sm/src/sm_82.c:969) may continue using a changed global `optionsmenu_index` after a prehandler. Timer postdecrement, byte-pointer continuation, deletion and real drawing preserve that global behavior. [Aliases](../../../../deps/snesrev-sm/src/variables_extra.h:62) overlap the enemy-projectile arrays at `1A8F..1B3D`; they must use the existing canonical WRAM image.

Static scripts/descriptors at `$82F442..$82F4DB` cover 154 bytes; six-byte descriptors start at F4B8/F4C4/F4CA/F4D0/F4D6. Selected sprite/other resource extents remain to be authenticated in the live domain. [OptionsInstr_8C79](../../../../deps/snesrev-sm/src/sm_82.c:1008) installs pre-pointer `$828C81`, which [CallOptionsPreInstr](../../../../deps/snesrev-sm/src/sm_82.c:944) does not dispatch. Static ROM puts an RTS at 8C81 inside the preceding immediate bytes. `sub_82F404` is also not registered in that dispatcher. Record both unsupported continuations and refuse whole requests before effects pending source/ROM review; a guessed no-op must not hide a genuine C/original difference.

The screen root has 13 screen states; item table has five entries/three unique handlers, special settings three/two, controller settings nine/three. The 29-routine screen scope excludes hardware service `EnableNMI` and depends on the object scope. It owns Japanese/special toggles, seven controller-binding slots, uniqueness swaps and defaults, dissolve/scroll states and file/start transitions. Controller binding aliases in `eproj_F` and saved options must be copied through the canonical word layout. DMA/decompression/save/poll/audio effects are accepted services. Pinned zero placeholders/initializer defects need explicit source-domain handling; avoid undefined-behavior fixtures and declare original differences.

### File and file-map menus

The [FileSelectMenu table](../../../../deps/snesrev-sm/src/sm_81.c:609) contains 34 entries and 29 unique handlers; [FileSelectMap](../../../../deps/snesrev-sm/src/sm_81.c:1343) has 23 entries and 20 unique handlers. Both addressless roots require exact symbol/definition provenance; counting unique handlers alone omits draw/persistence/map dependencies and exceeds 32 once closures are included.

FileSelectMenu_4_Main changes selected SRAM slot/inverse, loads a slot or creates a new save and map mirror; preserve the two SFX calls on its B branch. [Copy operation](../../../../deps/snesrev-sm/src/sm_81.c:1073) copies 1,628 bytes and all four checksum locations, then persists after drawing. [Clear operation](../../../../deps/snesrev-sm/src/sm_81.c:1241) zeros the same extent/checksums, invokes new-save/load/maps and persists. SRAM transaction and file-service failure behavior belong to research 19, including selected slot/inverse and new-save controller/player/map reset.

File menu states 0/16 reach original VBlank/ScreenOn polls already owned by held controller work. Preserve real polling count and position within dispatch; a one-poll-per-frame assumption or scheduler rewrite would change original timing. The successor must split file selection/copy-clear and file-map material roots with sealed menu tile/sprite/map resources and the existing controller/persistence dependency receipts.

## Provenance, adapters and controls

Current [translation discovery](../../../../crates/pipeline/src/translation.rs:400) admits bank86 projectile schema-2 diagnostics only. There is no supported direct-root exercise for these audio/menu roots. The inventory also omits `HandleSoundEffects`'s `kSfxHandlers[sfx_state[i]](i)` indirect edge; lexical [scope reachability](../../../../crates/pipeline/src/translation.rs:659) cannot prove that dynamic root closure. Supplemental child 01 explicitly assigns this as reviewed Rust infrastructure work through shared ticket 21. It must authenticate actual root entry/return, selected tables/handlers and material state with full bank or addressless symbol/definition identity, preserve historical bank86 evidence, and never infer a caller from a leaf hit.

Audio and UI have separate proposed infrastructure adapters. Audio includes the APU/SPC/DSP transaction and the declared original-ack comparison domain; menus reuse hardware state with real dispatch/object/category/resource/persistence/poll effects. Both require:
 
- Direct compiled-C root observation, immutable source/definition and selected-asset identity, original/generated C, host/templates, producer/readers, exact build/binary/argv/environment, ROM/native-patch/movie inputs and complete nonzero observations. A receipt is planning provenance until gameplay gates pass.
- Four independent C0, C1, mixed0 and mixed1 runs against the same frozen source and authentic inputs. Compare full canonical WRAM, PPU/DMA and audio/SPC/DSP/pending/cycle state, ordered reads/writes/uploads, raw root/coroutine returns and material ownership counters. Audio playback advances actual clocks/samples.
- Preflight the complete root domain/assets and private-state/public-state boundary before the first effect. Unsupported selector/state/continuation/ROM extent/backend refuses the entire request unchanged, with visible C/refusal/CPU fallback counters.
- Once accepted execution attempts its first external service, any first/late service failure terminates that subject. Never retry the root in native C after partial effects.
- Independent mutations: missing/reordered/coalesced port writes, wrong acknowledgment/delay/clear, ring/timer/reset footprint, wrong root/channel/category/index/handler/source/definition/digest/backend, zero/truncated material execution, changed bytecode/descriptor/palette/VRAM extent, controller/SRAM/new-save state and unchanged-refusal/service-failure behavior. Controls must fail for the intended semantic change.
- Independent original-ROM route comparisons after controller and APU read/upload support. Keep original versus patched-native identity and accepted source semantics separate in every receipt. Render or audio logs alone do not establish a material menu or playback run.

## Concrete gates and publication

All following new targets/manifests/adapters are proposed and unimplemented. All commands are unrun. These are reviewable requirements, not current successful CLI invocations.

Submission gates (60 seconds each) are exact `cargo test -p port --locked --offline --test <stem>` commands with nonzero tests for each owned stem; audio additionally runs `--test audio --test system`, equipment interaction also runs `--test pause_equipment_render --test audio`, and options root also runs `--test options_menu_objects --test audio`.

After implementing the separately reviewed actual-C adapter, the 3,600-second family gate for each scope is:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/audio-frame.toml --adapter audio-frame --output out/port/audio-frame --case-jobs 4
```

The other exact manifest/adapter/output stems are `pause-equipment-render`, `pause-equipment`, `options-menu-objects` and `options-menu`. Current parsing accepts an adapter string, but these adapter implementations do not exist. The infrastructure children must register/validate them and produce real verification receipts before any scope promotion. Infrastructure initial/merged cargo-test argv and required control suites are in the child manifest. Exact material Live replay and independent original-ROM argv remain an explicit adapter/research deliverable; they are unavailable today and cannot be invented.

A native scope gate cannot contain `--source`, `--checkout`, `--root` or `--changed-since`. The integrator independently selects/verifies all affected families from the exact externally recorded baseline with the shared four-worker budget. Freeze source for C0/C1/mixed subjects. Broaden/repeat gates only for new source changes or unresolved failures. Both scoped cleanup passes, immutable source/reader/store/private-input retention, implementer worktree retirement and reauthentication precede implementation completion.

[Scope holdings](scope-holding.toml) name the exact five 1–32 routine sets and unready dependencies. They deliberately do not fabricate `translation-scope-v1` exercise fields or admission. Parent must publish child task origins, resolve required dependencies, obtain real supported direct-root receipts, seal current inputs/dependency evidence, and produce native Scope fields only when those facts exist.

## Completion evidence and next action

The authored Answer, child proposals, scope holdings, routine/ROM/provider observations, dependency review and manual handoff form the read-only result package. [Document validation](document-validation.json) records matching routine identities/lists, local links and regular-file inputs; [artifact paths](artifacts.paths) enumerate the final 25 files. No experiment, run directory, build output or implementer worktree was created by this worker. [Cleanup](cleanup.toml) records final liveness, bytes and retained dependencies. Parent handles artifact retention/hashing with `sm-evidence ingest` followed by authenticated `inventory` and `verify`, records their run/reader/store pointers, publishes tickets and inventory, then admits the exact `researched` claim result. See [seal provenance and commands](input-seal-provenance.toml) and [final handoff](manual-handoff.toml).

No fake artifact seal, gameplay stage, original-ROM verdict or current-source leaf authentication is asserted.

