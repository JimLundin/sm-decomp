# M1a safe-signature review

**Repair update (2026-10-25):** replacement `6d93dd10` restores all 22 production
contracts and the verifier bridge listed below. A bounded source recheck found no
remaining concrete safe-signature violation in its 86 conversions; hosted verification
is pending. The script still omits `use ... as ...` alias edges. The affected aliases
in this patch were reviewed, but future codemod runs must resolve or conservatively
reject aliases rather than claim complete dependency analysis.

Rejected predecessor: `29d74efd482cef934bb3c3ede28ea03dd777efac`.

**Block landing of this predecessor: 22 newly safe production entrypoints and one verifier bridge expose unchecked operations.** Locations below refer to the rejected commit. This is a source-only, nonexhaustive review; the remaining conversions are not certified sound.

## Exact restoration list

All paths are relative to `sm/src/`.

| File | Newly safe entrypoints requiring their unsafe contracts |
|---|---|
| `game/message_boxes.rs` | `358 display` |
| `game/room_mechanisms/mod.rs` | `87 run`; `90 run_door`; `165 checks::command` |
| `game/sm_80.rs` | `1576 StartGameplay_Async`; `2321 Irq_FollowDoorTransition`; `2482 DecompressToVRAM`; `2486 LoadFromLoadStation` |
| `game/sm_81.rs` | `752 FileSelectMenu_26_ClearCompleted` |
| `game/sm_82.rs` | `3895 GameState_22_SamusNoHealth_Dying`; `3943 EnsureSamusDrawnEachFrame`; `3950 CallDoorTransitionFunction_Async`; `3957 GameState_9_HitDoorBlock`; `3964 GameState_10_LoadingNextRoom_Async`; `3971 GameState_11_LoadingNextRoom_Async`; `4184 GameOptionsMenu_4_StartGame` |
| `game/sm_8b.rs` | `745 HandleSamusDuringIntro`; `759 DrawIntroSprites` |
| `game/sm_a6.rs` | `662 CeresRidley_Func_25`; `2725 RidleysExplosion_Func_2`; `4153 Zebetites_Func_8`; `4161 Zebetites_Func_9` |
| `verification/room_mechanism_callback_edges.rs` | `39 display` |

## Concrete witnesses

- **Message and protocol hardware:** safe `Game::new()` leaves `g_snes` null. `message_boxes::display` reaches `InitializePpuForMessageBoxes -> WriteReg(HDMAEN, 0)` before any ROM read; RAM `$0613 = 1` gives mechanism `run/run_door` the same path. The verifier bridge unconditionally calls display. Default initial continuation in `StartGameplay_Async` writes DMA enable. Door selector `$82E3C0` (RAM `door_transition_function = $E3C0` for states 9/11), continuation zero and RAM `$84 = 0` reach `EnableIrqInterruptsNow` and unchecked register writes before ROM loading.
- **Other hardware bridges:** a nonempty compressed stream reaches `DecompressionVram::write -> WriteReg`. Mechanism command `$84870B` with operand `$91D4E4` selects Varia pickup and `spawn_hdma -> WriteRegWord`. Intro preparation with nonzero `cinematic_var15`, selector `$E713`, a live Power Bomb with motion `$B0AE` and fuse 1, unfrozen time and a free HDMA slot reaches the same hardware leaf. Sized loaded ROMs satisfy preceding checked reads without validating `g_snes`.
- **Drawing:** movement type 0, pose `$A4`, frame 16 reach `samus_animation.rs:327`, reading a native word beyond the 16-byte `LANDING` array. This affects death drawing, ensure-drawn, state 10 and intro drawing. Death timer 2 retains frame 16; intro/state 10 can select Frozen drawing. Sized ROM images satisfy preceding checked reads.
- **Load station:** a loaded station-table value of 1 with no native ROM reaches `room_pointer_diagnostic -> RomPtr($800001) -> null.offset(1)` before checked native capture can reject the missing image.
- **Save transfer:** native ROM length `$8132`, independently registered longer loaded ROM, and loaded cursor `$FFFF` at `$8182D6` reach `save_slots::transfer_map -> RomPtr`, forming a pointer beyond the native allocation. Start-game save requires debug enabled, Select clear and `loading_game_state != 5`; clear-completed load needs a new key and valid SRAM checksum.
- **Scrolling:** direction 0; layer position `(300, 2048)`; previous layer blocks `(19, 0)`; room width 229; others zero. Right scrolling produces x-block 19 and row 143. `build_scrolling_tilemap` computes wrapped source offset `$FFFE`, hence RAM `$1FFFE`; its second word reads beyond RAM `$20000`. No ROM or hardware is needed.
- **Actor initialization:** use a sufficiently sized zero-filled loaded image. Set the first population record at `$A6AA2F/$A6C9E7/$A6FCE1/$A6FCF9`, respectively, to definition native word `$8000`. At `$A08000`, set bank byte `+$0C = $A7`, initializer native word `+$12 = $CDF3`, multipart count `+$14 = 1`; leave other fields zero. Default free slots allow `spawn_legacy -> finish_initialization -> Runtime::actor_ai_legacy -> CallEnemyAi -> Phantoon_Init -> spawn_hdma -> WriteRegWord` through null hardware.

## Minimal repair and codemod admission

Restore `unsafe` on the listed signatures and dependent converted callers. Repair unchecked leaves separately. Hiding unsafe calls inside newly safe wrappers does not repair their contracts.

The candidate filter at `tools/codemod/strip_unsafe_fixpoint.py:287` checks local syntax and compilation, which cannot establish transitive safety. Immediately exclude these roots; admit only audited call closures. Conservatively reject closures reaching existing safe runtime/trait bridges with unchecked operations, unresolved dispatch or unaudited macro effects, and propagate exclusions through subsequent rounds. Compiler checks and function-body `forbid(unsafe_code)` are useful local guards, not proofs of callee preconditions.

No additional concrete root emerged from the bounded PLM spawn/draw check. No edits to repository source, builds, Git writes or CI observation were performed for this review.

