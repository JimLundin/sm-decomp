# Successor scope: scheduled Ceres room transitions

Status: actionable successor packet complete; integrator to select after ticket08
Date: 2026-10-16
Research owners: `/root/transition_boundary` (pinned C), `/root` (reuse, host, scenario and integration recommendation)
Writable ownership for this packet: this file only

Recommend the complete **door-transition scheduler**, from the already scheduled
state-9 entry through state 10 and every state-11 continuation back to state 8.
The coherent minimum is three scheduled roots, their private dispatcher and all
15 private transition states: **19 recovered-C routines**. Keep room-loading,
actor, hardware and IRQ scrolling services as explicitly declared dependencies
unless the integrator selects existing Rust replacements. This owns the complete
scheduling boundary for both crossings; it does not establish that every service
executed during a room transition is Rust-owned.

This is a successor recommendation for DF45 → DF8D → DFD7 after ticket 08.
There is no implementation, new acceptance gate, actor audit or gameplay-stage
publication in this packet. The implementation/integration owner chooses the
final Translation scope.

## Source identity and citation convention

Recovered C revision: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`.
All `sm_*.c:L`, `types.h:L`, `funcs.h:L`, `variables.h:L` and
`ida_types.h:L` citations below refer to this exact retained, instrumented source
directory, not the similarly named main-checkout dependency:

`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src`

- [sm_82.c](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_82.c)
  SHA-256: `20ae568cc21260a577e441bfcd25ac3fdc33e00fce3eb5c82fa85c7f9ccabebe`.
- ROM data was read directly from `/home/jim/sm-decomp/rom/sm.sfc`, SHA-256
  `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
  ROM citations give SNES address and headerless file offset. These are data
  reads, not a replay or original-ROM behavioral comparison.
- Read root `GLOSSARY.md` and `docs/agents/domain.md` before exploration.
  There is no `docs/adr/` directory in the main checkout.

## Entry and exit boundary

`RunOneFrameOfGameInner` chooses its scheduled function from
`kGameStateFuncs[game_state]`, whose relevant entries are state 8, 9, 10 and 11
(`sm_82.c:609–621`). State 8 runs movement through
`HandleSamusMovementAndPause` (`sm_82.c:864`), whose ordinary frame handler runs
Samus movement (`sm_90.c:6012–6028`). The block-collision table selects
`BlockColl_Horiz_Door` for a level-data high nibble of 9
(`sm_94.c:916–962`). That routine publishes
`door_transition_function = E17D`, selects the door-list entry using
`BTS[cur_block_index] & 0x7f`, writes `door_def_ptr`, and changes `game_state` to
9 when the destination pointer has its high bit set (`sm_94.c:858–870`).
`BlockColl_Vert_Door` performs the corresponding vertical entry
(`sm_94.c:873–885`). Keep that collision boundary and the movement subsystem
outside this scheduler scope.

Changing `game_state` during movement does **not** abandon the current state-8
body. Its PLM, drawing, room-main, timer and other tail work still runs
(`sm_82.c:864–887`). The dispatcher latches `game_state + 10` in
`coroutine_state_0`; a yielded invocation resumes that original function even
if `game_state` changed (`sm_82.c:657–686`). Do not invoke the new door scheduler
immediately at the collision write or replace this latch with a fresh
`game_state` lookup on resume.

For a normal non-elevator entry, state 9 calls `HandleElevator`, which increments
state to 10 and directly calls `GameState_10_LoadingNextRoom_Async`; state 10 then
sets the sound-wait function and increments state to 11 before returning
(`sm_82.c:3791–3799`, `3823–3865`). Thus a state-10 sampled frame is unnecessary:
state 9 → state 11 can happen in one scheduled invocation. State 10 is still a
real root in the game-state table and belongs in the same ownership unit.

The exit is `DoorTransition_FadeInScreenAndFinish`: only a completed palette
fade clears the transition actor flags, reenables sounds and writes state 8
(`sm_82.c:4168–4183`). Returning `kCoroutineNone` from an intermediate phase
means that invocation completed; it does **not** mean the door crossing ended.

## Exact roots and private transition states

The three scheduled roots are:

| Symbol | Source / original address | Responsibility |
| --- | --- | --- |
| `GameState_9_HitDoorBlock` | `sm_82.c:3791`, `82:E169` | Dispatch the current door function. |
| `GameState_10_LoadingNextRoom_Async` | `sm_82.c:3823`, `82:E1B7` | Freeze transition actor behavior, prepare fade targets and sound queues, enter state 11. It currently has no coroutine suspension. |
| `GameState_11_LoadingNextRoom_Async` | `sm_82.c:3868`, `82:E288` | Propagate the phase's pending return; draw the timer only after the phase invocation completes. |

Their private dispatch is `CallDoorTransitionFunction_Async`
(`sm_82.c:3770–3788`). It accepts `door_transition_function | 0x820000` and
admits precisely the following states. The function-word constants are also
declared at `funcs.h:4354–4368`; in particular the uncommented loading routine
is `82:E4A9`, not an inferred address.

| Function word | Private symbol | Source | Phase behavior and next phase |
| --- | --- | --- | --- |
| `E17D` | `DoorTransitionFunction_HandleElevator` | `sm_82.c:3795` | Non-elevator/upward-elevator paths call state 10 in the same invocation; downward elevators enter `E19F`. |
| `E19F` | `DoorTransitionFunction_Wait48frames` | `sm_82.c:3810` | Predecrement 16-bit elevator delay; only negative completes. Otherwise process/draw enemies and Samus. |
| `E29E` | `DoorTransitionFunction_WaitForSoundsToFinish` | `sm_82.c:3875` | Continue actor drawing; advance to `E2DB` only when all three queue write/read differences are zero modulo 16. |
| `E2DB` | `DoorTransitionFunction_FadeOutScreen` | `sm_82.c:3888` | Fade; process/draw actors only on an incomplete fade; completion selects `E2F7`. |
| `E2F7` | `DoorTransitionFunction_LoadDoorHeaderEtc` | `sm_82.c:3900` | Load destination door header, clear HDMA channels, select initial door IRQ and `E310`. |
| `E310` | `DoorTransitionFunction_ScrollScreenToAlignment` | `sm_82.c:3909` | Move the perpendicular camera coordinate one pixel per invocation until its low byte is zero, update scrolls, then select `E353`. |
| `E353` | `DoorTransitionFunction_FixDoorsMovingUp` | `sm_82.c:3932` | Apply upward-door correction only for direction 3; select `E36E`. |
| `E36E` | `DoorTransitionFunction_SetupNewRoom` | `sm_82.c:3939` | Map/header/state/effects/level-scroll loading; select `E38E`. |
| `E38E` | `DoorTransitionFunction_SetupScrolling` | `sm_82.c:3950` | Reset scroll/IRQ setup fields and invoke directional scrolling setup; select `E3C0`. |
| `E3C0` | `DoorTransitionFunction_PlaceSamusLoadTiles` | `sm_82.c:3963` | Rebase Samus, clear completion flag, install transition IRQ, decompress/copy tiles and palette; select `E4A9`. |
| `E4A9` | `DoorTransitionFunction_LoadMoreThings_Async` | `sm_82.c:4007` | Load/reset room services once; await IRQ completion and then PLMs; align Samus X and select `E659`. |
| `E659` | `DoorTransition_C_HandleAnimTiles` | `sm_82.c:4125` | Run animtiles once; select `E664`. |
| `E664` | `DoorTransition_WaitForMusicToClear` | `sm_82.c:4131` | Poll music; on empty queue select `E6A2` and queue a changed track. |
| `E6A2` | `DoorTransition_HandleTransition` | `sm_82.c:4139` | Final Samus adjustments, clear HDMA buffers, restore room IRQ, liquid physics and brightness; select `E737`. |
| `E737` | `DoorTransition_FadeInScreenAndFinish` | `sm_82.c:4168` | Animate/process/draw actors and queue BG2 transfers each invocation; completed fade restores state 8. |

Except for `HandleElevator`/`Wait48frames` directly entering state 10, selecting
the next phase does not execute that phase immediately. The next normal
scheduled invocation dispatches it. Preserve this one-phase-per-invocation
behavior, including the no-op upward-correction phase on rightward doors.

## Coroutine and external-progress contract

The coroutine ABI is a `uint8_t`: zero completes the invocation and a nonzero
value propagates suspension (`types.h:83–112`). `COROUTINE_AWAIT` stores its
position only on a nonzero child result; on resume its case label calls the
child again. `COROUTINE_AWAIT_ONLY` forwards a nonzero return without owning a
separate continuation state. These are explicit persisted state machines, not
ordinary blocking function calls.

`LoadMoreThings_Async` owns `coroutine_state_1` (`sm_82.c:4010`):

1. Position 0 performs the loading/reset/initialization sequence and complete
   background command stream (`sm_82.c:4012–4036`). Those effects execute once,
   before its first suspension.
2. Position 1 calls `WaitForNMI_Async` until `door_transition_flag & 0x8000`
   (`sm_82.c:4037–4040`). The wait first sets `waiting_for_nmi`, sets
   `coroutine_completion_flags = 1` and returns 1. Reentry clears that completion
   flag and returns 0 (`sm_80.c:106–115`). Resume the pending wait once even when
   the IRQ has meanwhile set the door-completion bit; entering at the loop's
   top would skip its completion-flag cleanup.
3. After scrolling, write palette entry 196, spawn BG3 scroll HDMA and enable
   HDMA, then position 2 awaits `PlmHandler_Async` (`sm_82.c:4041–4044`). That
   handler owns `coroutine_state_2`; it can itself await a message box
   (`sm_84.c:224–265`, especially `246–251`). Do not model it as an always-void
   foreign call just because this Ceres route may not open a message.
4. Only after the PLM continuation finishes does X alignment occur and
   `door_transition_function` advance to `E659`; `COROUTINE_END(0)` resets the
   loading continuation (`sm_82.c:4046–4053`).

`RunOneFrameOfGameInner` skips its frame prologue and resumes the latched root
while pending. On completed invocations it runs sound effects and its tail,
then clears `coroutine_state_0` (`sm_82.c:657–699`). `RunOneFrameOfGame` runs
`Vector_NMI` after either return kind (`sm_82.c:702–710`); `Vector_NMI` consumes
palettes, OAM, graphics and queues when `waiting_for_nmi` is set
(`sm_80.c:1061–1084`). A host step therefore must not turn a nonzero root return
into an ordinary completed game-dispatch step.

The loading wait is completed by **IRQ scrolling**, not by state 11 itself:

- Setup dispatches `DoorTransitionScrollingSetup` according to direction
  (`sm_80.c:2299–2309`). Right setup invokes `DoorTransition_Right` once
  immediately (`sm_80.c:2312–2323`).
- Horizontal IRQ phases 22/24/26 run the HUD/register schedule; phase 24 calls
  `Irq_FollowDoorTransition` while the completion bit is clear
  (`sm_80.c:1200–1225`). `Vector_IRQ` dispatches the selected IRQ routine
  (`sm_80.c:1245–1263`).
- `DoorTransition_Right` moves Samus by the door speed, moves both layers by
  four pixels, updates map/scroll data, and finishes when the *old* frame
  counter was 63 (`sm_80.c:2397–2408`). Thus its ordinary initialized path has
  64 movement calls, including the initial setup call. Its terminal call makes
  a second scroll-update call.
- `Irq_FollowDoorTransition` then snaps layer 1 to the door destination and
  sets bit `0x8000` (`sm_80.c:2389–2394`). The scheduler must preserve this live
  hardware/scroll service; replacing it with a fixed host-step wait loses the
  causal completion dependency.

The named `WaitUntilEndOfVblankAndEnableIrq` is **synchronous in this recovered
C**: it conditionally calls `EnableIrqInterruptsNow` and never returns a
coroutine suspension (`sm_82.c:3667–3670`; `sm_80.c:1234–1239`). Preserve the
reference's actual behavior rather than inferring a yield from its name.

## Shared state that must remain interoperable

Key scheduler state is ordinary shared WRAM, not wholly private Rust state:

| Field | WRAM offset | Source |
| --- | --- | --- |
| `coroutine_state_0` through `_4` | `0611`–`0615` (bytes) | `variables.h:112–116` |
| `coroutine_completion_flags` | `0782` (byte) | `variables.h:117` |
| `door_def_ptr`, `door_direction`, door properties | `078D`, `0791`, `0793` | `variables.h:274–277` |
| transition enemy/elevator flags | `0795`, `0797` | `variables.h:278–279` |
| `room_ptr`, room/header/CRE/state fields | `079B` onward | `variables.h:281–307` |
| transition counter, destination X/Y, subspeed/speed, elevator delay, completion flag | `0925`, `0927`, `0929`, `092B`, `092D`, `092F`, `0931` | `variables.h:338–344` |
| `game_state`, `door_transition_function` | `0998`, `099C` | `variables.h:394–395` |
| `elevator_flags` | `0E16` | `variables.h:738` |

In particular the original `door_transition_function` words are observable
outside this subsystem. Ceres haze checks `E737` to begin fading in and `E2DB`
to begin fading out (`sm_88.c:2441–2448`, `2475–2481`). A private Rust enum is
fine internally only if it continues publishing the original words at the
same points. Likewise, `LoadDoorHeader` changes `room_ptr` at `sm_82.c:3613`,
before the later `LoadRoomHeader`/`LoadStateHeader` phase. Intermediate state can
legitimately contain the destination room pointer with source-room actor or
header data. Do not combine those separate scheduled phases based solely on
the room-pointer change.

## Direct dependencies and synchronous foreign needs

These are the 19-routine scheduler's service boundary, grouped to make ownership
explicit. They are actual calls in the cited phase bodies. The integrator may
reuse existing Rust implementations; any C remainder must retain its ordering,
return value and shared-state effects.

| Dependency group | Exact direct services / source caller |
| --- | --- |
| Transition actor ticks/drawing | `DetermineWhichEnemiesToProcess`, `EnemyMain`, `DrawSamusEnemiesAndProjectiles`, `EnsureSamusDrawnEachFrame` in `sm_82.c:3815–3818`, `3828–3831`, `3876–3879`, `3892–3895`; fade-in additionally `AnimtilesHandler`, `EprojRunAll`, `QueueEnemyBG2TilemapTransfers`, `PlaySpinJumpSoundIfSpinJumping` at `4169–4178`. |
| Sound, music and fades | `ClearSoundsWhenGoingThroughDoor`, `QueueSfx2_Max15` (`sm_82.c:3860–3861`); `AdvancePaletteFadeForAllPalettes_0xc` (`3889`, `4176`); `HasQueuedMusic`, `LoadNewMusicTrackIfChanged` (`4132–4134`); `DrawTimer` (`3857`, `3871`). The fade helper sets denominator 12 (`3317–3319`); sound queues continue to be serviced by the outer frame tail (`688`, `740–752`). |
| Door and room data | `LoaadDesinationRoomCreBitset` (`sm_82.c:3832`, definition `3602`), `LoadDoorHeader` (`3901`, definition `3608`), `SaveMapExploredifElevator`, `LoadRoomHeader`, `LoadStateHeader`, `LoadMapExploredIfElevator` (`3940–3943`; definitions `3631`, `3651`, `3675`, `3682`). |
| Scroll/effects setup | `sub_8882AC` (`sm_82.c:3902`; body `sm_88.c:340`), `CalculateLayer2PosAndScrollsWhenScrolling` (`sm_82.c:3924`, `3927`), `FixDoorsMovingUp` (`3934`), `InitializeSpecialEffectsForNewRoom`, `LoadLevelScrollAndCre` (`3944–3945`), `DoorTransitionScrollingSetup` (`3958`). |
| Immediate graphics | `WaitUntilEndOfVblankAndEnableIrq`, `DecompressToMem`, `CopyToVramNow` (`sm_82.c:3976–3988`); `ClearFxTilemap` and optional FX copy (`4027–4029`); the background commands below (`4030–4035`). |
| Once-per-load room services, in exact call order | `LoadEnemyGfxToVram`, `LoadRoomMusic`, `ClearEprojs`, `ClearAnimtiles`, `ClearPaletteFXObjects`, `ClearPLMs`, `CreatePlmsExecuteDoorAsmRoomSetup`, `LoadFXHeader`, `SpawnDoorClosingPLM`, `UpdateBeamTilesAndPalette`, `LoadColorsForSpritesBeamsAndEnemies`, `LoadEnemies`, `InitializeEnemies`, `ResetProjectileData`, `Samus_LoadSuitTargetPalette` (`sm_82.c:4012–4026`). |
| Resume/finish | `WaitForNMI_Async`, `SpawnBG3ScrollHdmaObject`, `PlmHandler_Async` (`sm_82.c:4039–4044`), `AnimtilesHandler` (`4126`), `PointlessFunctionStupidToo`, conditional `CallSomeSamusCode`, `SetLiquidPhysicsType` (`4155–4162`). `CallSomeSamusCode` also appears in the elevator entry (`3800`). |

The loading background dispatcher is private to this path:
`kUpdateBackgroundCommands` (`sm_82.c:3996–4005`) calls
`UpdateBackgroundCommand_0_Finish`, `_2_TransferToVram`, `_4_Decompression`,
`_6_ClearFxTilemap`, `_8_TransferToVramAndSetBg3`, `_A_ClearBg2Tilemap`,
`_C_ClearKraidBg2Tilemap`, `_E_DoorDependentTransferToVRAM`
(`sm_82.c:4081–4123`). Preserve its returned next pointer; it is not a list of
void deferred effects. For a general scheduler implementation, own this small
command loop with its states or declare a synchronous service for its complete
existing C behavior. The similarly named `LoadLibraryBackground` at
`sm_82.c:4309` belongs to initial/resume loading and is not this call path.

Three dependency closures deserve explicit treatment:

- **Loaded state before subsequent services.** `LoadRoomHeader` selects the
  room state through `HandleRoomDefStateSelect` (`sm_82.c:3646`;
  `sm_8f.c:695–705`). `LoadLevelScrollAndCre` decompresses level data, copies
  BTS/background and loads scrolls (`sm_82.c:4381–4433`). These writes must be
  visible before directional scrolling and before room/actor initialization.
- **Ordered hardware work.** `CopyToVramNow` performs immediate blanking,
  DMA/register writes and display restoration (`sm_80.c:1087–1097`). In
  `PlaceSamusLoadTiles`, decompression into `7E:2000` and palette RAM happens
  before the VRAM copies (`sm_82.c:3980–3988`). `LoadEnemyGfxToVram` similarly
  loops through immediate copies (`sm_82.c:3692–3716`), and `ClearFxTilemap`
  writes its RAM buffer before copying it (`4056–4062`). A foreign transaction
  must expose each preceding Rust RAM write and apply returned RAM/hardware
  effects synchronously; merely recording future calls changes the reference.
- **Room callbacks and existing Rust hooks.**
  `CreatePlmsExecuteDoorAsmRoomSetup` invokes room PLM setup, then door setup,
  then room setup (`sm_82.c:4435–4445`; `sm_8f.c:785–791`, `881–885`). Enemy
  initialization/ticks and the scrolling services can enter already owned
  gameplay. Preserve those existing hooks when a C dependency calls back into
  Rust. No further actor closure is audited here.

The service ABI needs scalar result propagation for the palette fade and music
queries, and the full nonzero/zero coroutine return for both waits and PLMs.
The background commands also return `uint16` next pointers. The surrounding
shared-state service calls must finish before their callers inspect or mutate
state again. These requirements follow from the phase bodies above; they do
not require a new scheduling framework.

## What the two selected crossings actually require

The ROM uses these two outgoing doors (layout: `ida_types.h:260–270`):

| Crossing | Door definition and bytes | Consequence |
| --- | --- | --- |
| DF45 → DF8D | `83:AB4C`, file `0x1AB4C`: `8d df 00 00 00 00 00 00 00 80 13 e5` | Rightward, destination screen `(0,0)`, negative/default Samus distance, door callback `8F:E513`. |
| DF8D → DFD7 | `83:AB64`, file `0x1AB64`: `d7 df 00 00 00 00 00 00 00 80 00 00` | Rightward, destination screen `(0,0)`, negative/default Samus distance, no door callback. |

DF45's door list at `8F:DF8B` contains `AB4C`; DF8D's at `8F:DFD3` contains
`AB58, AB64` (ROM file offsets `0x7DF8B`, `0x7DFD3`). `LoadDoorHeader` derives
rightward default distance 200 and speed `200 << 8` in the combined speed/
subspeed representation (`sm_82.c:3608–3628`). It sets elevator flags from the
loaded header. The earlier `HandleElevator` checks the *current* flag, before
this load, so record its actual entry value instead of inferring that branch
only from the destination header.

Pre-escape state selection uses `RoomDefStateSelect_IsBossDead` with boss bit 1;
when clear, the finish selector chooses DF8D state `DF9F` and DFD7 state
`DFE9` (ROM `0x7DF98`, `0x7DFE2`; selector bodies `sm_8f.c:695–705`,
`723–728`; room-state layout `ida_types.h:983–998`). The data gives:

| Loaded field | DF8D state DF9F | DFD7 state DFE9 |
| --- | --- | --- |
| Room dimensions | 2 × 1 screens | 1 × 2 screens |
| CRE bitset | 0 | 0 |
| Compressed level source | `CD:BBFE` | `CD:BD78` |
| Graphics set | `0F` | `0F` |
| Requested music data/track | `0 / 0` | `0 / 0` |
| FX header | `83:A156` | `83:A158` |
| Layer-2 policy word | `01C1` | `4101` |
| Room PLM list | `8F:C8BD` | `8F:C8BF` |
| Background stream | `8F:E4A5` | `8F:E4A5` |
| Room setup callback | `8F:C976` | `8F:C976` |

These are direct ROM reads at `0x7DF8D`/`0x7DF9F` and
`0x7DFD7`/`0x7DFE9`. The room headers' widths/heights and CRE fields use
`ida_types.h:889–900`. DF45 has CRE bitset 5, so the first crossing's
`previous_cre_bitset | cre_bitset` differs from the second's 0. The state-10
palette-preservation condition must remain intact (`sm_82.c:3602–3605`,
`3838–3859`); these two crossings are useful distinct inputs to that branch.

Both selected room PLM lists start with zero (ROM `0x7C8BD`, `0x7C8BF`), and
both FX headers start with `FFFF` (ROM `0x1A156`, `0x1A158`), which makes
`LoadFXHeader` return (`sm_89.c:62–74`). Direction 0's closing-PLM table entry
at `8F:E68A` is zero (ROM `0x7E68A`), so no generic closing PLM is spawned from
that table (`sm_82.c:4259–4269`). These facts do not allow removal of PLM or FX
services from the general scheduler contract.

The shared background stream `8F:E4A5` (file `0x7E4A5`) is materially nonempty:
decompress `BA:C22A` to RAM `4000`; transfer `7E:4000` to VRAM `4800`, size
`0800`; transfer the same source to VRAM `4C00`, size `0800`; finish. Its
opcodes are 4, 2, 2, 0, interpreted by `sm_82.c:4030–4035` and
`4081–4102`. This is immediate background loading, distinct from gameplay
scrolling background uploads.

`InitializeSpecialEffectsForNewRoom` clears Mode7 before scrolling setup
(`sm_82.c:3944`; `sm_88.c:346–423`, specifically `377`). DF8D's door callback
`DoorCode_CeresElevatorShaft_2` restores fake background mode 9 and clears
Mode7 again (`sm_8f.c:619–622`). Both room callbacks are
`RoomCode_CeresColorMathHdma`, which spawns Ceres haze
(`sm_8f.c:444–446`; `sm_88.c:2424–2431`). This explains why ordinary
rightward IRQ scrolling can feed the already existing scrolling hooks in both
destination rooms; `UpdateScrollVarsUpdateMap` is visibly hooked in this
retained C tree (`sm_80.c:1783–1791`).

## Recommendation and explicit limits

Assign one worker the full 19-routine scheduling boundary and its private
background command handling, with the above dependencies fixed as service
contracts or selected Rust reuse. The integrator owns root registration,
synchronous/coroutine service plumbing and the combined scenario. Keep
directional IRQ scrolling as an identified dependency for this first scheduler
scope; a later coherent scope can own `DoorTransitionScrollingSetup`, its four
directional setup/movement pairs, `FixDoorsMovingUp`,
`Irq_FollowDoorTransition` and their map/scroll dependencies together.

The count of 19 is the scheduler core. Translating all eight private background
command routines alongside it produces a coherent 27-routine source set; the
clear/copy/decompress callees remain explicit service dependencies. Alternatively
the complete background stream can remain one synchronous C dependency. Do not
claim those eight helpers from the 19-routine core alone.

The first scenario should exercise the two complete real scheduled crossings,
including at least: the 9→11 same-invocation handoff, source-versus-destination
CRE palette branch, nonempty background command stream, pending and resumed
loading phase, unchanged input polling while the latched root is pending,
IRQ completion, haze-observable phase words, and completed fade back to state
8. The integration owner will supply exact existing replay/evidence pointers;
none was generated by this research packet.

Left/up/down movement, elevators, timer drawing, music changes, active message
boxes, alternative boss-selected room states and the other background opcodes
remain outside this **live** scenario. If translated generically, retain that
distinction between translated behavior and live-exercised behavior. A Ceres
room pointer or a completed crossing alone is insufficient evidence that every
private phase was owned or that a coroutine resumed correctly.

Initial `StartGameplay_Async` is a separate scheduled loading path with music
waits, `LoadLevelDataAndOtherThings`, `LoadLibraryBackground` and
`DisplayViewablePartOfRoom` (`sm_80.c:1599–1641`). It happens to write the same
fade-in function word at its end. Do not expand or route initial room loading
through this successor merely because that word matches.

No original-ROM parity follows from the recovered-C coroutine behavior. Even
sound acknowledgements in this reference are explicitly modified
(`sm_82.c:766–785`). Preserve the pinned reference contract for this scope and
keep any original-ROM comparison as separate evidence.

## Existing Rust reuse: bounded algorithms, no inherited acceptance

Main source was inspected at `193cad941713b3539d23e08238a79e7fb09dde61`.
Its [room.rs](/home/jim/sm-decomp/crates/port/src/room.rs:1) explicitly leaves
hardware transition scheduling and tilemap upload outside its ownership.
Search of `crates/port` found no implementation of the three scheduled roots or
`DoorTransitionFunction_*`. Reuse its algorithms selectively; do not import
the old crate or claim that these helpers already establish this live lifecycle.

| Reusable source | Useful material | Limit |
| --- | --- | --- |
| [room.rs:16](/home/jim/sm-decomp/crates/port/src/room.rs:16), [room.rs:32](/home/jim/sm-decomp/crates/port/src/room.rs:32) | Bank-bounded ROM reads and room-state selection, including the boss/event selectors. | Uses old `Wram`/`system` interfaces; port only the required algorithm or retain the complete loader as an explicit service for the first scheduler scope. |
| [room.rs:80](/home/jim/sm-decomp/crates/port/src/room.rs:80), [room.rs:97](/home/jim/sm-decomp/crates/port/src/room.rs:97) | Room/state header decoding and tileset pointers. | Header decoding does not own decompression, room initialization or suspension. |
| [room.rs:568](/home/jim/sm-decomp/crates/port/src/room.rs:568) | Header dispatch, destination CRE transfer, door fields and negative-distance defaults. | Keep destination-pointer publication at its original phase; do not merge header phases. |
| [room.rs:612](/home/jim/sm-decomp/crates/port/src/room.rs:612) | Music-change decisions. | Old audio dispatch remains a dependency; copying this does not translate playback. |
| [tests/room.rs:227](/home/jim/sm-decomp/crates/port/tests/room.rs:227), [tests/room.rs:242](/home/jim/sm-decomp/crates/port/tests/room.rs:242) | Negative-distance/direction case; selector success, refusal and ROM-range cases. | Focused regression ideas only. No scheduler, coroutine or live transition test is present here. |

The existing restart room-scroll module is already the selected implementation
for its three scheduled entries. The successor must continue reaching those
installed entries through normal service calls; duplicating their algorithms
inside the transition crate would split ownership unnecessarily.

## Current host constraint and first ABI decision

Host inspection below is at restart commit
`3667f974ae0154c598959861c80ad12e793e6221`, during ticket08 integration.
Rebase the eventual assignment onto the completed ticket08 checkpoint.

The public [Game::host_step](/home/jim/sm-decomp-restart/restart/native-host/src/lib.rs:168)
calls the native step synchronously. The [shim](/home/jim/sm-decomp-restart/restart/native-host/src/shim.c:56)
runs `RtlRunFrame`, drains audio and captures observations once per host step.
The [prepared observation hooks](/home/jim/sm-decomp-restart/restart/native-host/src/prepare.rs:138)
count a completed scheduler update only for `kCoroutineNone`, separately from
NMI and explicit waits. Keep this outer execution and comparison contract.

Existing [Transaction](/home/jim/sm-decomp-restart/restart/native-host/src/transaction.rs:18)
copies canonical WRAM before a callback and commits a handled invocation;
refusal after changed RAM or a service request is fatal. Existing
[projectile outcomes](/home/jim/sm-decomp-restart/restart/native-host/src/hooks.rs:16)
express ownership only. They cannot encode whether an owned coroutine completed.

**Resolve this ABI first:** return ownership and the original `u8` coroutine
result separately. A handled pending call must commit its continuation WRAM,
return its actual pending byte to the existing C scheduler, and end all borrowed
state for that invocation. Resume on the next real host invocation from canonical
continuation bytes. Preserve the outer state-0 latch and the exact state-1
resume positions; never turn Pending into Refused or retain a Rust borrow across
an NMI/host-step boundary.

The second concrete constraint is child gameplay. Current
[scroll dispatch](/home/jim/sm-decomp-restart/restart/native-host/src/scroll.c:31),
[projectile dispatch](/home/jim/sm-decomp-restart/restart/native-host/src/pilot.c:91)
and [door dispatch](/home/jim/sm-decomp-restart/restart/native-host/src/ceres_door.c:45)
reject nested sibling entries. New transition services such as actor processing,
room setup and directional scrolling can legitimately reach these existing
consumers. Define one allowed transition-parent/service/child path, synchronize
the parent's private WRAM before each service and refresh it afterward, and keep
the existing sibling/recursive rejection rules. Do not silently bypass the
installed Rust children, remove all nesting checks, or batch ordered services
for later execution. This is a specific composition requirement of the selected
scheduler, not a request for a general nested-callback framework.

## Smallest existing scenario and actual observations

Reuse the current combined 9,380-input scenario. Its fixed inputs are
[reference-02/inputs.txt](/home/jim/sm-decomp-restart/restart/out/scroll/runs/reference-02/inputs.txt);
the source is the retained `ceres-06` prefix with the known firing edge at8380.
The [reference lifecycle](/home/jim/sm-decomp-restart/restart/out/scroll/runs/reference-02/lifecycle.csv)
and [original native route observations](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/observations.csv)
agree on these sampled milestones (host steps, not original-ROM frames):

| Crossing | State9 | State11 | Destination `room_ptr` visible | State8 resumed |
| --- | --- | --- | --- | --- |
| DF45→DF8D |8726|8727|8744|8831|
| DF8D→DFD7 |9030|9031|9053|9140|

Each crossing adds63 explicit waits in the native observation ledger
(217→280 and280→343). These are observed route facts, not hard-coded completion
rules. State10 has no sampled row, consistent with its same-invocation execution.
The remaining steps to9380 preserve existing stairwell row/DMA and projectile
postconditions. A first-crossing-only8831-step diagnostic is shorter, but changing
the accepted scenario solely to save about550 steps adds no useful evidence;
prefer the already shared scenario for the first implementation acceptance.

The old trace lacks detailed transition-phase/service ownership counts. Add
those narrow counters/events to the existing runner as part of the new host
recipe, then capture exactly one new C reference. Do not regenerate original-ROM
captures or reinterpret the old reference as containing these new observations.
Afterward candidate edits reuse that new reference through `sm-pilot dev`.

## Actionable assignment recommendation

Recommend **one whole scheduler delivery with the eight private background
commands included**:19 core routines plus8 background routines,27 source routines
in total. This is a scope inventory, not a progress metric. Both reached rooms
execute nonempty background commands, so keeping that small interpreter in Rust
removes a real part of the transition's control flow. Decompression, immediate
VRAM copies and unrelated room/gameplay services remain explicit synchronous
dependencies for this first delivery. A documented19-core variant with the
entire background stream retained as one service is valid if the integrator
needs that smaller boundary; do not imply the eight commands are then owned.

Suggested disjoint ownership, to record in the successor ticket before edits:

| Owner | Exclusive implementation files | Deliverable |
| --- | --- | --- |
| Gameplay worker | new `restart/room-transition/**` (`lib.rs`, scheduler and private background code) | Three roots, all private phases/commands, exact WRAM continuation and phase words, typed service interface and focused branch/resume cases. |
| Native-host worker | `restart/native-host/**` | Exact root hooks, coroutine-result propagation, named service adapter with typed scalar returns, counted allowed child calls, transaction/terminal-failure handling and native recipe. Coordinate public types before independent edits. |
| Integrator | Cargo manifests/lock, `restart/src/gameplay.rs`, `runtime_cli.rs`, `bin/sm-pilot.rs`, new `bin/sm-pilot/room_transition.rs`, ticket/progress/docs | Register both workers, retain all current Rust consumers, own scenario/reference/candidate execution, exact claims and cleanup. |
| Read-only reviewer | one assigned review Markdown file | Check continuation positions, phase-word visibility and the two observed crossing predicates against the fixed sources; no separate runner or duplicate capture. |

The first implementation slice is this complete admitted ordinary-transition
coordinator, not a header-only function collection. Keep collision entry and
directional IRQ movement outside it; retain other services by explicit name and
count. Translate generic branches where practical but claim live coverage only
for the two rightward pre-escape Ceres crossings. If a service prevents the
admitted path, report that exact call and adjust the scope rather than wrapping
the whole old transition root as a foreign service.

Once the completed ticket08 baseline and ABI above are frozen, the first real
experiment should be the existing command family with a new scenario module:

```sh
./target/debug/sm-pilot reference --scenario room-transition \
  --library NEW_PREPARED_LIBRARY --rom /home/jim/sm-decomp/rom/sm.sfc \
  --prefix out/runs/ceres-06/inputs.txt --out out/room-transition/runs/reference-01
./target/debug/sm-pilot dev --scenario room-transition \
  --library NEW_PREPARED_LIBRARY --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/room-transition/runs/reference-01 --out out/room-transition/runs/dev-01
```

These are proposed commands after registration; `room-transition` does not exist
yet. The material result must include both completed crossings, owned roots and
phases, pending/resumed loading with no repeated one-time initialization, actual
background transfers and preserved projectile/scroll/door results. Measure the
existing warm loop and keep foreign services and unexercised branches explicit.
No new framework, speculative whole-ROM synchronization or broader replay is
required to begin this scope.

## ABI follow-up and assignment readiness

The integrator selected the27-routine variant and committed the agreed ABI as
`b80a083`, [room-transition-abi.md](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/room-transition-abi.md).
It consumes this packet and freezes three scheduled entries,53 named services,
separate ownership/coroutine returns, a controlled transition/service/child
grammar and passive observations for the27 routines. The independent follow-up
found no research blocker to dispatching that complete scope.

The contract preserves the specific risks identified here: state0 latching,
pending-byte propagation and committed state1/state2, wait reentry before the
IRQ-bit loop check, original shared phase words, immediate state exchange around
services, and existing child ownership without disabling sibling guards. Passive
routine observations establish a no-refusal-after-effects barrier as well as
service requests; implementations must carry that barrier into `Transaction`
completion even when no foreign service ran.

One high-value focused implementation case is state1 resumed after IRQ has
already set the completion bit: the outstanding wait must still be invoked and
clear its completion flag, without rerunning room initialization. Another is a
named transition service reaching a current Rust child and receiving its
committed WRAM before the parent continues. These check the actual composition
seams; they do not require a second harness or speculative service masks.
No code, tests or runs were added by this follow-up; implementation evidence is
still required from its assigned owners and the shared runner.

## Research cleanup

Read-only source/ROM inspection and this Markdown draft were the only work.
No builds, tests, gameplay runs, worktrees or generated intermediates were
created. No run-cleanup utility invocation is applicable; reclaimed bytes: 0.
Available disk at the handoff check: 105,504,501,760 bytes (`df -B1`). Other
agents' source, active runs, tickets and progress inventory were left unchanged.
