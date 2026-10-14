# Door/room/PLM scheduled-boundary research

No retained supported receipt establishes a complete observed door transition or `PlmHandler_Async` invocation. The next coherent candidate is **one complete invocation of `PlmHandler_Async` at `$84:85B4`, including all active slots, over an explicitly admitted original-script domain**. Start with the four closing-door scripts below. The whole door transition remains a successor with substantial room, enemy, player, audio and hardware dependencies. This is researched scope, not gameplay acceptance or an executable `translation-scope-v1`.

The coordinator must publish the proposed children from [children.toml](children.toml) to consumers 07/09 and admit supported provenance before route assignment. [scope-holding.toml](scope-holding.toml) deliberately records `accepted = false` and `assign_scope_supported = false`.

## Exact claim and source

Research ticket: [.scratch/decomp-parallel/issues/15-door-room-plm-scope.md](/home/jim/sm-decomp/.scratch/decomp-parallel/issues/15-door-room-plm-scope.md). Claim `ticket-fd25bf762e6e92ca542277cba01e52aaaaeb05c209485978d2511a2413b31c0e`, revision `8957b66f91dcad619098905199286c7fef94bd8581e60455273128817ba33aa4`, attempt 1, coordinator `decomp-run-20261013`. Actual launched worker UUID `01a135a1-959f-7e80-b593-21a91052e0dc`; parent UUID `01a13592-d02b-7143-8489-a15b03f19da2`. The coordinator supplied the UUID from the launch receipt and recorded `started`.

Checkout and read baseline: `/home/jim/sm-decomp`, `7ed3d383d6d3e28d21d1bb960acc5576ce82f5e9`. No source writes, builds, fixtures, emulator/replay runs, authority mutations or Git/index changes were performed. Only this assigned scratch directory is authored. The returned native research Job omits `base-head`; this manual source observation does not amend or fabricate the Job.

Pinned recovered C revision: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`; archive SHA-256 `69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`. `tools/dependencies.toml` SHA-256 `e74864c229949fc6f1a6bd7528f339bdb3f9364f1e4f9544f08693881d10f111`; lexical inventory SHA-256 `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`. Exact source/routine observations are in [source-observations.json](source-observations.json), all read input hashes in [inputs.sha256](inputs.sha256). The full lexical record is evidence of source identity; data-selected calls still require explicit review.

## Complete PLM invocation contract

Recovered root: [sm_84.c:217](/home/jim/sm-decomp/deps/snesrev-sm/src/sm_84.c:217), definition SHA-256 `09bef2ad6e96bcb3b92f08b33b6376353051317e77fd2384151f94bbe5abd666`.

1. At coroutine position 0, a clear `plm_flag & 0x8000` returns zero without resetting draw index or PLM id. Otherwise reset draw index and scan global `plm_id` from 78 to 0 by wrapping subtraction of 2. Empty headers skip the slot.
2. Call the preinstruction before decrementing its instruction timer, even if the preinstruction deletes the header. A timer decremented from 0 wraps to `0xffff`; any nonzero result skips bytecode.
3. Bank84 instructions are byte-addressed. High-bit words dispatch repeatedly, consuming one-byte operands where declared. A null continuation ends only that slot. Normalize returned native ROM pointers and low16 continuation sentinels back into the bank84 address space.
4. If an instruction queues a message, store its continuation in the slot, await **the actual** `DisplayMessageBox_Async` at `coroutine_state_2 = 1`, and preserve its `uint8 CoroutineRet`. A resume jumps directly to the await: it must not rerun the preceding preinstruction, timer decrement or instruction. After completion, clear the queue, reconstruct the ROM pointer from WRAM, and continue the same slot. This resume bypasses the start-position flag test.
5. A duration/draw tuple writes the instruction timer, draw pointer and list pointer plus four, then executes `ProcessPlmDrawInstruction → CalculatePlmBlockCoords → DrawPlm` in that order. Finish all slots, set id 0 and coroutine state 0, and return 0.

The scheduler must honor mutation of **global** `plm_id` by selected callbacks. A fixed local loop index is only equivalent for a reviewed callback domain that cannot change it. Use full-WRAM comparison and a deliberate callback-mutation control when admitting a wider domain.

Canonical WRAM includes `0613` (PLM coroutine), `0dc8` (queued message), `1c23/1c25/1c27` (flag/draw index/id), `1c29/1c2b` (coordinates), and forty two-byte slot arrays at `1c37` headers, `1c87` block indices, `1cd7` preinstructions, `1d27` lists, `1d77` timers, `1dc7` room arguments, `1e17` variables, `de1c` instruction timers, `de6c` draw pointers, `debc` links and `df0c` variables/hits. Level data starts at `10002`, BTS at `16402`; renderer inputs include room width, scroll/layer coordinates, tiles at `a000`, draw buffer at `c6c8`, and VRAM queue `00d0`/tail `0330`. Preserve the complete image, including aliases, rather than comparing only these named fields.

## Reusable leaves and ownership

| Responsibility | Existing Rust / recovered C dependency | Proposed owner |
| --- | --- | --- |
| Whole scheduled invocation and publication boundary | New `crates/port/src/plm_frame.rs`, `crates/port/tests/plm_frame.rs`, `tools/port/plm-frame.toml` | One isolated PLM gameplay worker after provenance/adapter admission |
| Allocation, setup, preinstructions | `plm.rs`; `SpawnRoomPLM` `$84846A`, `SpawnHardcodedPlm` `$8483D7`, `PlmSetup_DeactivatePlm` `$84B3C1`; admitted empty preinstruction `$8484E6` | Reuse as explicit dependencies; no competing leaf rewrite |
| Bytecode and level-data drawing | `plm_instructions.rs`; `CallPlmInstr`, selected opcode bodies, `ProcessPlmDrawInstruction` `$84861E` | Reuse existing leaves; any changed leaf requires expanded exact ownership and affected-family verification |
| Tilemap queue and geometry | `plm_render.rs`; `DrawPlm` `$848DAA`, `CalculatePlmBlockCoords` `$848290`, `CalculatePlmDrawTilemapVramDst` `$8491DC`, `PartiallyQueueVramForSingleScreenPlm` `$849220`, `RomPtr_84orRAM` | Reuse with original asset/full-state/order controls |
| SFX queue for closing opcode | `audio.rs`; `QueueSfx3_Max6` `$80914D` and `QueueSfx3_Internal` `$809155` | Existing pure WRAM queue dependency; music/APU service stays audio18 |
| Native root exercise, adapters, ABI, source/catalog registrations | Reviewed extensions to research13 files plus proposed PLM adapter files in children | Integrator-owned infrastructure after held Ceres gates; no gameplay worker edits `lib.rs`, canonical state/ROM, host or registry |
| NMI/IRQ, DMA, PPU, message service, controller and audio scheduling | hardware13, controller12 and audio18; separate message child | Explicit external dependencies; an entire PLM/door root cannot be delegated to a host callback |
| Whole room transition | Proposed `door_transition.rs` and its tests/manifest, after room/enemy/player/FX/asset closure | One successor lane owner; player and enemy modules remain dependencies |

Current [plm_instructions.rs:332](/home/jim/sm-decomp/crates/port/src/plm_instructions.rs:332) already executes a bounded synchronous frame, including drawing. It rejects nonzero coroutine state and preexisting queued messages, loops over a local `k`, and can return an error after writing WRAM. It therefore needs a new transactional public caller and whole-invocation admission before live ownership. Do not publish partial writes or skip unsupported slots. For a no-service domain, run all work against a private full-WRAM image and commit only success. For any external service domain, preflight before the first request; once a request is attempted, any failure or later unsupported condition is an owned fatal failure with no C retry. Preserve the existing execution-once host contract.

Leaf aliases require attention when broadening bytecode: `SetTimer` writes only the low byte; door completion reads an overlapping word from a byte bit array; draw-data decoding uses a low8 block count while the renderer uses a 15-bit count; coordinate division uses low8 room width and SNES zero-divisor results; relative offsets are signed. Clipping, flips, screen splitting, tilemap/queue capacity and low/high write order belong in the full-state/order gate. Existing instruction budget 4096 and unsupported rejection remain explicit bounds.

## Exact original closing-door asset domain

[rom-ranges.json](rom-ranges.json) records 27 exact ranges, file offsets, lengths and SHA-256 digests. Read method was `dd` range streaming into `sha256sum`; no ROM assets were copied or executed. LoROM offset is `(bank & 0x7f) * 0x8000 + (address & 0x7fff)`.

Original `rom/sm.sfc`: 3,145,728 bytes, SHA-256 `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`. Retained native `room-plm-current-live/.../c-0/rom-view.bin`: 4,194,304 bytes, SHA-256 `40aaebb25b9646bea2dffdc5e1ae680768a39dd122de9353885e1c7456ceae27`. **All 27 selected ranges match between these files; whole-image identity and original-ROM execution parity remain different claims.**

The selector `$8F:E68A..E699` is 16 bytes: `0000,0000,0000,0000,C8BE,C8BA,C8C6,C8C2`. SHA-256 `43c2f9b610c35c6cdee42ce94be11c079d56357cc1291df663166bd7a44d8ee4`. Closing selection uses full `door_direction` 4–7, although motion dispatch masks with 3. Restricting direction to 0–3 would exclude all four closers.

The four 4-byte PLM headers together occupy `$84:C8BA..C8C9`, SHA-256 `e017b8c435689eb14ea4568a5a02ba46ae2b613d0b55b57adc978c9da40f79e6`. All use setup `$B3C1`.

| Full door direction | Header / script start | Exact script range and SHA-256 | BTS / ordered draw streams |
| --- | --- | --- | --- |
| 5 | `C8BA / C49E` | `C49E..C4B9`, 28 bytes; `3b6975cbc96bf06d091b2e1f7063711e92361840da1d437505fd4b692bcb1c46` | `40`; `A677,A9D7,A9CB,A9BF,A9B3` |
| 4 | `C8BE / C4CF` | `C4CF..C4EA`, 28 bytes; `e61a7d46c4abd4b73a857aa1055ec0dd62753b45c7e07c6b0ddc29be85723667` | `41`; `A683,AA13,AA07,A9FB,A9EF` |
| 7 | `C8C2 / C500` | `C500..C51B`, 28 bytes; `ac601eac112bade8201846a21bd8d73e29c526d943172578d389a4e780a9b95f` | `42`; `A68F,AA4F,AA43,AA37,AA2B` |
| 6 | `C8C6 / C531` | `C531..C54C`, 28 bytes; `bf7ab13b5da0d1203c4c37fc8923730474c38b75d5f1855fe1a9199e4c1c7ffa` | `43`; `A69B,AA8B,AA7F,AA73,AA67` |

Each stream is 12 bytes including its terminator; all twenty stream hashes are recorded separately. Each closing script has duration/draw tuples and only three high-bit opcodes: `$8C19 QueueSfx3_Max6` with one-byte sound `$08`, `$8AF1 SetBTS` with a one-byte direction value, and `$86BC Delete`. **The 49-byte spacing between script starts is not script length.** Each 21-byte gap after the first three closes includes a separate opening script; bytes after the fourth close enter unrelated code. Do not seal those gaps as closing scripts or accept their `$07` opening SFX as observed closing behavior.

Admission must bind every active slot's actual header, selected preinstruction, current list boundary, reachable operands and draw stream. Renderer tile-table reads and WRAM geometry remain dependencies even for sealed bank84 scripts. A whole-root fallback is required if any active slot falls outside the admitted invocation domain. The boot header `$B7EB` and `$B7E9 Sleep` are a separate reusable domain: the 6-byte script/header range hash is `f32b6acc1aec9e14a60b455b06428234a2070438ff9b54f81dad6d9f8b68612d`; `$B7DD` preinstruction decrements a timer and deletes/enables sound at old value 1. Boot allocation evidence does not prove the boot scheduler or closing scripts were Rust-owned.

## Message coroutine successor

[sm_85.c:373](/home/jim/sm-decomp/deps/snesrev-sm/src/sm_85.c:373) implements `DisplayMessageBox_Async` `$858080`, definition SHA-256 `dffb1142236e461fbb02efe5671b40714a9bb82caed5e109642edb0b331ac8d3`. Its complete bank85 dependency closure has 22 functions: the async root; PPU initialize/restore; clear/write large/small/general tilemaps and draw dispatch; HDMA construction/setup; active/small/large PPU setup; run/shoot/special button drawers and modify dispatch; message initialize; save-selection toggle; open, interaction and close coroutines. Exact symbols/hashes are in source observations and the proposed child.

`coroutine_state_3 = 0614` owns root positions 2/3/4 and special save-confirmation positions 5/6/7/8. `coroutine_state_4 = 0615` is shared by open/interaction/close. These await `WaitForNMI_NoUpdate_Async`, service music/SFX and read joypad input. Include nested state, counters, selection, PPU shadows/latches/DMA/VRAM and ordered MMIO. `message_box_das0l_value` is a C global `uint16` outside WRAM and must be represented explicitly in any native image. The C port's message delay/controller behavior and original ROM behavior require separate comparison. A canned message return cannot verify the coroutine. Initial closing-domain admission refuses message positions and queued messages before publication; wider PLM admission depends on the real message adapter and its owned native/C dependency.

## Full door transition successor

Recovered roots are `GameState_9_HitDoorBlock` `$82E169`, `GameState_10_LoadingNextRoom_Async` `$82E1B7`, and `GameState_11_LoadingNextRoom_Async` `$82E288`. Together with `CallDoorTransitionFunction_Async` and its 15 selectors they form 19 orchestration symbols, within the 32-routine cap. The selector sequence is:

`HandleElevator E17D → Wait48frames E19F → WaitForSoundsToFinish E29E → FadeOutScreen E2DB → LoadDoorHeaderEtc E2F7 → ScrollScreenToAlignment E310 → FixDoorsMovingUp E353 → SetupNewRoom E36E → SetupScrolling E38E → PlaceSamusLoadTiles E3C0 → LoadMoreThings E4A9 → HandleAnimTiles E659 → WaitForMusicToClear E664 → HandleTransition E6A2 → FadeInScreenAndFinish E737`.

`E4A9` is a `funcs.h` dispatch constant, not a lexical opening-line address annotation. `CallDoorTransitionFunction_Async`, `CopyToVramNow`, `DoorTransitionScrollingSetup` and `Irq_FollowDoorTransition` are also addressless C definitions. Keep their source-definition identities; do not fabricate lexical addresses to satisfy the current reader.

The dependency closure is larger than the 19 orchestrators:

- Elevator path starts a 48-frame counter and waits until decrement sets its sign bit, while old enemies and Samus drawing still run. State10 loads destination CRE information, fades/palettes and sound queues, and installs state11's first selector.
- New-room setup loads room/state/map/FX/level/scroll/CRE data and decompresses assets. Scrolling setup dispatches four directions. Placement issues immediate graphics DMA and enables IRQ after vblank.
- [LoadMoreThings:4005](/home/jim/sm-decomp/deps/snesrev-sm/src/sm_82.c:4005) uses state1 `0612`, with positions 1 waiting for NMI and 2 awaiting PLM. Its initialization order includes enemy graphics, music, clearing eprojs/animated tiles/palette FX/PLMs, room PLM/door/room setup, FX, closing allocation, beam/palette/enemy load and initialization, projectile reset, suit palette and FX/background transfers. The background table has eight command kinds `0,2,4,6,8,A,C,E`; selected operands, decompression consumption and MMIO effects need authenticated extents.
- `SpawnDoorClosingPLM` `$82E8EB` first calls `CheckIfColoredDoorCapSpawned` `$82E91C`. Existing colored doors can redirect to a 6-byte header's second script instead of the four closers above. `CreatePlmsExecuteDoorAsmRoomSetup` `$82EB6C` scans 6-byte room entries to a zero header and invokes dynamic door/room setup code. Those paths are not admitted by the four closing headers.
- NMI waits cannot finish until IRQ `Irq_FollowDoorTransition`/four `DoorTransition_*` handlers update `door_transition_flag & 0x8000`. Horizontal motion uses counter 64 and vertical motion 57, with distinct frame-zero behavior. Then BG3 HDMA, PLM invocation and Samus alignment run.
- `CopyToVramNow` writes INIDISP80, VMADDR low/high, DMA1 mode/address/bank/size, VMAIN80, MDMAEN2 and INIDISP0F in that exact byte order. Full hardware state belongs to hardware13; queue display's 17 ordered flushes do not verify immediate DMA.
- Final animation/music/IRQ/liquid-physics/palette handling leads to fading while enemy/eproj/Samus draw logic still runs. Only successful final fade clears transition/sound flags and sets `game_state = 8`.

`room.rs` explicitly rejects unowned room/hardware transition routines. Complete enemy initialization/main, selected room setup, player callbacks, animated tiles, palette FX/HDMA/audio, decompressors and hardware scheduling must be independently owned and evidenced. Split coherent dependencies as successive scopes when closure exceeds 32; do not enlarge the cap or hide gameplay behind a whole-root host request. Preserve resolved140 camera X-before-Y semantics and accepted room scrolling's 11 routines. Compressed room/CRE/background/enemy/FX asset source extents remain **unresolved** until a direct observed route and decoder consumption receipt exists; output transfer sizes are not compressed input lengths.

## Retained evidence and limits

The paths and hashes below are retained inputs, read and hashed here. No recursive family/live authentication was rerun during this research.

The rendering receipt binds `plm_instructions.rs` hash `aeec081fad8fb1032d0a4fa65f0e9fb4ab599ef441d2122403d0caae60ae27f1` and `plm.rs` hash `11fcf907066b1402bdfd40f497451acf4e31decc2daa09783e6abc0afd5d2df1`, which differ from this checkout; its `plm_render.rs` hash matches current source. Its synthetic ROM hash is `3d8c2f3b0b5114e87ac8bb6e3ce919430df39dadde28bc7800e7aa1cb105f1db`. These identities make historical leaf evidence reusable context, while current-source admission still needs the affected gate.

| Evidence | SHA-256 | What it establishes / limit |
| --- | --- | --- |
| `out/port/room-plm-final/20261010T223633.137790Z/verdict.json` | `3b8e49aa750a4ee629bb6adb43d61b9dfcadf7762783232b5352e66f2556e916` | 1,969 allocator fixture pairs; no scheduled PLM or live root claim |
| `out/port/plm-rendering/20261010T225759.092054Z/verdict.json` | `4b168e02d73fc424649b010c2b4091503dae4f6320634ec968214732c48bf1fc` | 1,100 pairs across six instruction/render seams on historical frozen source and synthetic ROM; no original live closing-domain claim |
| `out/port/room-plm-current-live/20261010T224032.887967Z/verdict.json` | `a3c23353ba8fa110e50cc4ba5d45957f86046d4019885527bd43b553f507eb8a` | Four subjects, 9,000 inputs; one hardcoded allocation per mixed subject, zero room/terrain calls; no whole PLM loop observation |
| `.scratch/decomp-port/recovery-integrated-camera-scroll-post-cleanup-authentication.toml` | `dda85bda679c84dc2fea9bef73a1d553901fc6efbcc2c39db2d4b70db466f3bf` | 467 camera fixture pairs; accepted X-before-Y dependency |
| `out/recovery-session/composed-live-release/1791861180321696848-129445/verdict.toml` | `5e224bda08b3f597b07dd3a03c59bf8cf136b288993459c62e453ef3fac400c2` | Four subjects, 129,712 inputs; 94,425 camera calls, 35,580 active, no camera refusals; `room_plm_installed = false`; no door or original-ROM claim |
| `.scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml` | `9251ee29114c3ac5ea7da19279050ac7e83227cb790e624c5b6a0900372f3845` | Historical frozen composed receipt remains authenticated after cleanup; shared evidence store/readers remain required |
| `out/port/ceres-continuation/game-host-projectile/1791873394532541-258459-0/verdict.toml` | `18f355479a34ef10be6196e2c1507f6ded34d522693f068feb1313078492abad` | Accepted shared projectile host dependency; no door caller inference |
| `out/port/ceres-continuation/game-host-room-scroll/1791873418836128-259326-0/verdict.toml` | `ddb49cbd79a7249b45745044f9f6c95a7e98671fa74e0b313a7f81bf8c869d1d` | Three actual-C pairs and 17 ordered flushes; room display contract, not complete door/live acceptance |

The current [translation reader:400](/home/jim/sm-decomp/crates/pipeline/src/translation.rs:400) accepts only schema2 `diagnostic-complete`, `accepted = false`, bank86 projectile receipts and their `projectiles.tsv`/source/producer/build/replay/ROM/movie seals. Routine addresses are constructed as `0x860000 | low16`. PLM/door roots, camera counters, hardcoded allocator hits and final Ceres room state cannot be relabeled into that domain. The returned research receipt also records `routine lexical dependency inventory differs from pinned source`; a fresh inventory alone must not be presumed sufficient. Research13's additive Rust native-root producer/reader extension should absorb bank84/bank82 requirements and reconcile lexical producer/reader equivalence while retaining historical identities.

`dispatcher_plm.c`'s `PlmFrame`/`PlmControlFrame` are synthetic wrappers around the real C root; they grant no separate original-routine provenance. Existing `live_plm.rs.in` owns allocation. Current bounded live families admit trails/camera/room-scroll/player-projectile/Ridley instruction paths, not the complete PLM/message/door boundaries. Merely adding a whitelist entry does not implement exclusive ownership, transactional admission, service image or coroutine evidence.

## Concrete next gates

All commands below are proposed gates after their reviewed child creates the named target. They were not executed here. Use `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, workers 4; submission/initial experiments at most 60 seconds and integration at most 3,600 seconds.

For the shared exercise child, reuse research13's `cargo test -p pipeline --locked --offline --test native_exercise`; extend its tests with actual compiled-C bank84/bank82 root and data-dispatch controls plus source/binary/ROM/movie/counter/inventory rejection controls. For the PLM adapter child use `cargo test -p pipeline --locked --offline --test plm_frame_adapter`, then the same test plus existing `family`/`live` semantic tests at integration. For gameplay:

```sh
cargo test -p port --locked --offline --test plm_frame --test plm_instructions --test plm_render --test plm
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/plm-frame.toml --output out/port/plm-frame --case-jobs 4
```

Acceptance must include source-authenticated actual-C full WRAM/returns and ordered effects, all forty-slot root traversal, timer/deletion/continuation/geometry/queue controls, independent unsupported-before-publication and execution-once failures, exact asset ranges, and a fixture chain through every tuple to Delete. Later message admission adds real nested coroutine state/return and hardware-global controls.

Four fresh live subjects must exercise a material admitted root invocation using naturally obtained state, match hardware/game/terminal WRAM/SRAM and ordered PPU/APU/VRAM effects, and report whole-root Rust/C fallback, supported/unsupported slot/script counters and first/last frame. Original-ROM comparison needs a separate independent emulator route, validated ROM/movie and controller/NMI timing; existing native runtime replay cannot supply it. The current CLI has no complete PLM/door live adapter or original-route contract, so no invented `live-verify` flags are offered. The adapter child must implement and seal that exact argv before scope promotion. Coordinator alone selects and runs every Affected Offline family and publishes source/catalog/state/inventory updates.

Only after exact new input/dependency/writable-file hashes and passing reviewed gates exist should the coordinator generate a native 1–32-routine scope, using the published task as origin. Cleanup and post-cleanup authentication belong to those runs. This research's explicit no-run cleanup is [cleanup.toml](cleanup.toml); final identity/results/limitations/next command are persisted in [manual-handoff.toml](manual-handoff.toml).
