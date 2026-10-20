use crate::c_library::{__assert_fail, memcpy, printf};
use crate::cpu_infra::DebugGameOverMenu;
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::enemy_instruction;
use crate::game::sm_80::{
    memset7E, CalculateLayer2PosAndScrollsWhenScrolling, CheckEventHappened, ClearBG2Tilemap,
    ClearFXTilemap, ClearOamExt, ClearTimerRam, ClearUnusedOam, CopyToVramNow,
    DebugScrollPosSaveLoad, DecompressToMem, DecompressToVRAM, DisableIrqInterrupts,
    DoorTransitionScrollingSetup, DrawTimer, EnableIrqInterruptsNow, EnableNMI, FixDoorsMovingUp,
    HandleFadeIn, HandleFadeOut, HandleHudTilemap, HasQueuedMusic, InitializeHud,
    LoadFromLoadStation, LoadMirrorOfExploredMapTiles, NextRandom, PrepareBitAccess,
    QueueClearingOfFxTilemap, QueueMusic_Delayed8, QueueMusic_DelayedY, QueueSfx1_Max15,
    QueueSfx1_Max6, QueueSfx2_Max15, QueueSfx2_Max6, QueueSfx3_Max15, QueueSfx3_Max3,
    QueueSfx3_Max6, ReadJoypadInputs, ReleaseButtonsFilter, ResumeGameplay,
    SaveExploredMapTilesToSaved, SetElevatorsAsUsed, SetupDmaTransfer, StartGameplay_Async,
    Vector_NMI, Vector_RESET_Async, WaitForNMI_Async,
};
use crate::game::sm_81::{
    DrawMenuSpritemap, DrawSpritemap, FileSelectMap, FileSelectMenu, GameOverMenu, SaveToSram,
};
use crate::game::sm_84::{
    ClearPLMs, ClearSoundsWhenGoingThroughDoor, EnablePLMs, PlaySpinJumpSoundIfSpinJumping,
    PlmHandler_Async, SpawnRoomPLM,
};
use crate::game::sm_85::DisplayMessageBox_Async;
use crate::game::sm_86::{ClearEprojs, EnableEprojs, EprojRunAll};
use crate::game::sm_87::{AnimtilesHandler, ClearAnimtiles, DisableAnimtiles, EnableAnimtiles};
use crate::game::sm_88::{
    sub_8882AC, DisableHdmaObjects, EnableHdmaObjects, HdmaObjectHandler,
    InitializeSpecialEffectsForNewRoom, SpawnBG3ScrollHdmaObject, WaitUntilEndOfVblankAndClearHdma,
};
use crate::game::sm_89::LoadFXHeader;
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_, LoadTitleSequenceGraphics,
};
use crate::game::sm_8d::{
    ClearPaletteFXObjects, DisablePaletteFx, EnablePaletteFx, PaletteFxHandler,
};
use crate::game::sm_8f::{
    HandleRoomDefStateSelect, PauseHook_DraygonRoom, RunDoorSetupCode, RunRoomMainCode,
    RunRoomSetupCode, UnpauseHook_DraygonRoom,
};
use crate::game::sm_90::{
    CallSomeSamusCode, HandleControllerInputForGamePhysics, HandleSamusMovementAndPause,
    MainScrollingRoutine, ResetProjectileData, Samus_DrawDuringDeathAnim,
    Samus_DrawWhenNotAnimatingOrDying, Samus_LowHealthCheck_0, SetLiquidPhysicsType,
    UpdateBeamTilesAndPalette,
};
use crate::game::sm_91::{
    GameState_28_Unused_, SamusFunc_F433, Samus_Initialize, Samus_LoadSuitTargetPalette,
    Samus_SetAnimationFrameIfPoseChanged,
};
use crate::game::sm_9b::{
    DrawSamusStartingDeathAnim_, GameState_24_SamusNoHealth_Explosion_Helper,
    HandleSamusDeathSequence, StartSamusDeathAnimation,
};
use crate::game::sm_a0::{
    DecrementSamusTimers, DetermineWhichEnemiesToProcess, DrawSamusEnemiesAndProjectiles,
    EnemyMain, EprojProjCollDet, EprojSamusCollDetect, HandleRoomShaking, InitializeEnemies,
    LoadEnemies, ProcessEnemyPowerBombInteraction, QueueEnemyBG2TilemapTransfers,
    SamusProjectileInteractionHandler, TransferEnemyTilesToVramAndInit,
};
use crate::game::sm_a7::{
    Kraid_UnpauseHook_IsSinking, PauseHook_Kraid, UnpauseHook_Kraid_IsAlive,
    UnpauseHook_Kraid_IsDead,
};
use crate::game::sm_a9::MotherBrainsBody_UnpauseHook;
use crate::game_state::Game;
use crate::host::{Die, Warning};
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{
    Load24, Mult8x8, RomPtr, RtlApuWrite, SnesDivide, Unreachable, WriteReg, WriteRegWord,
};
pub use crate::snes::registers::*;
pub use crate::types::RoomPlmEntry;
pub use crate::types::{
    size_t, CoroutineRet, Func_V, Func_V_Coroutine, Func_Y_V, Func_Y_Y, LongPtr, SnesRegs, VoidP,
    VramWriteEntry,
};
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::transmute;
use ::core::ptr::{null, null_mut};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PlmHeader_Size6 {
    pub func_ptr: VoidP,
    pub instr_list_1_ptr: VoidP,
    pub instr_list_2_ptr: VoidP,
}
pub use crate::types::EnemyData;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LoadBg_E {
    pub field_0: u16,
    pub field_2: u16,
}
pub use crate::types::Buttons;
pub use crate::types::LoadingGameState;
pub use crate::types::{DoorDef, SamusPose};
pub use crate::types::{EnemyTileset, GameState, RoomDefHeader};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TileSet {
    pub tile_table_ptr: LongPtr,
    pub tiles_ptr: LongPtr,
    pub palette_ptr: LongPtr,
}
pub use crate::types::{
    ExpandingSquareTransitionHdma, MapIconDataPointers, MapScrollArrowData, RoomDefRoomstate,
    StartDmaCopy,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DemoRoomData {
    pub room_ptr_: VoidP,
    pub door_ptr: u16,
    pub door_slot: u16,
    pub screen_x_pos: u16,
    pub screen_y_pos: u16,
    pub samus_x_offs: u16,
    pub samus_y_offs: u16,
    pub demo_length: u16,
    pub demo_code_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union PauseScreenSpriteAnimationData {
    pub c2rust_unnamed: C2RustUnnamed,
    pub arr: [VoidP; 9],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct C2RustUnnamed {
    pub unused: VoidP,
    pub lr_highlight: VoidP,
    pub item_selector: VoidP,
    pub unused2: VoidP,
    pub unused3: VoidP,
    pub map_scroll_arrow_up: VoidP,
    pub map_scroll_arrow_down: VoidP,
    pub map_scroll_arrow_right: VoidP,
    pub map_scroll_arrow_left: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FileCopyArrowStuff {
    pub spritemap: u16,
    pub xpos: u16,
    pub ypos: u16,
}
pub use crate::types::Consts_82;
pub use crate::types::Consts_83;
pub use crate::types::{
    Consts_60, EnemyDef, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram3800, Ram4000,
    Ram4000_Backups,
};
crate::game::constants::typed_constants! { Consts_83;
    addr_kDoorDef_947a = 38010,
}
pub use crate::types::Consts_8F;
pub const __ASSERT_FUNCTION: [c_char; 28] =
    unsafe { transmute::<[u8; 28], [c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0") };
crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
}
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    '_c2rust_label: {
        if addr < 0x2000u16 {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const c_char,
                51u32,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    ram_ptr!(game; addr as i32 as isize)
}
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
#[inline]
unsafe fn get_DoorDef(a: u16, game: &mut Game) -> *mut DoorDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut DoorDef
}
#[inline]
unsafe fn get_RoomDefHeader(a: u16, game: &mut Game) -> *mut RoomDefHeader {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader
}
#[inline]
unsafe fn get_RoomDefRoomstate(a: u16, game: &mut Game) -> *mut RoomDefRoomstate {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefRoomstate
}
#[inline]
unsafe fn get_TileSet(a: u16, game: &mut Game) -> *mut TileSet {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut TileSet
}
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_EnemyTileset(a: u16, game: &mut Game) -> *mut EnemyTileset {
    RomPtr((0xb40000u32) | ((a) as u32), game) as *mut EnemyTileset
}
#[inline]
unsafe fn get_RoomPlmEntry(a: u16, game: &mut Game) -> *mut RoomPlmEntry {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomPlmEntry
}
#[inline]
unsafe fn get_DemoRoomData(a: u16, game: &mut Game) -> *mut DemoRoomData {
    RomPtr((0x820000u32) | ((a) as u32), game) as *mut DemoRoomData
}
#[inline]
unsafe fn get_LoadBg_E(a: u16, game: &mut Game) -> *mut LoadBg_E {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut LoadBg_E
}
#[inline]
unsafe fn get_PlmHeader_Size6(a: u16, game: &mut Game) -> *mut PlmHeader_Size6 {
    RomPtr((0x840000u32) | ((a) as u32), game) as *mut PlmHeader_Size6
}
crate::game::constants::typed_constants! { u32;
    fnDemoRoom_ChargeBeamRoomScroll21 = 8554778, fnnullsub_291 = 8554788,
    fnDemoRoom_SetBG2TilemapBase = 8554789, fnDemoRoom_SetKraidFunctionTimer = 8554795,
    fnDemoRoom_SetBrinstarBossBits = 8554802, fnOptionsPreInstr_nullsub_57 = 8555536,
    fnOptionsInstr_Destroy = 8555610, fnOptionsInstr_8C64 = 8555620,
    fnOptionsInstr_SetPreInstr = 8555630, fnOptionsInstr_8C79 = 8555641,
    fnOptionsInstr_Goto = 8555650, fnOptionsInstr_8C89 = 8555657,
    fnOptionsInstr_8C93 = 8555667, fnBabyMetroidPlaySfx0x23 = 8567820,
    fnBabyMetroidPlaySfx0x26 = 8567829, fnBabyMetroidPlaySfx0x27 = 8567838,
    fnDoorTransitionFunction_HandleElevator = 8577405,
}
crate::game::constants::typed_constants! { i32;
    fnDoorTransitionFunction_Wait48frames = 0x82e19f, fnDoorTransitionFunction_WaitForSoundsToFinish = 0x82e29e,
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db, fnDoorTransitionFunction_LoadDoorHeaderEtc = 0x82e2f7,
    fnDoorTransitionFunction_ScrollScreenToAlignment = 0x82e310, fnDoorTransitionFunction_FixDoorsMovingUp = 0x82e353,
    fnDoorTransitionFunction_SetupNewRoom = 0x82e36e, fnDoorTransitionFunction_SetupScrolling = 0x82e38e,
    fnDoorTransitionFunction_PlaceSamusLoadTiles = 0x82e3c0, fnDoorTransitionFunction_LoadMoreThings_Async = 0x82e4a9,
    fnDoorTransition_C_HandleAnimTiles = 0x82e659, fnDoorTransition_WaitForMusicToClear = 0x82e664,
    fnDoorTransition_HandleTransition = 0x82e6a2, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
}
crate::game::constants::typed_constants! { u32;
    fnsub_82F296 = 8581782, fnOptionsPreInstr_F2A9 = 8581801,
    fnsub_82F34B = 8581963, fnsub_82F353 = 8581971,
    fnsub_82F35B = 8581979, fnsub_82F363 = 8581987,
    fnOptionsPreInstr_F376 = 8582006, fnOptionsPreInstr_F3A0 = 8582048,
    fnOptionsPreInstr_F3E2 = 8582114, fnsub_82F419 = 8582169,
    fnOptionsPreInstr_F42C = 8582188, fnPauseHook_Empty = 8946657,
}
crate::game::constants::typed_constants! { i32;
    fnCinematicFunctionNone = 0x8b9a47, fnCinematicFunctionOpening = 0x8b9b68,
    fnCinematicFunction_Intro_Initial = 0x8ba395, fnCinematicFunctionBlackoutFromCeres = 0x8bc11b,
    fnCinematicFunctionEscapeFromCebes = 0x8bd480,
}
crate::game::constants::typed_constants! { u32;
    fnPauseHook_DraygonRoom = 9423094, fnUnpauseHook_DraygonRoom = 9423100,
    fnUnpauseHook_Kraid_IsDead = 10994171, fnUnpauseHook_Kraid_IsAlive = 10994254,
    fnKraid_UnpauseHook_IsSinking = 10994336, fnPauseHook_Kraid = 10994469,
    fnMotherBrainsBody_UnpauseHook = 11110243,
}
crate::game::constants::typed_constants! { i32;
    fnlocret_828C81 = 0x828c81,
}
crate::game::address_dispatch! {
    pub unsafe fn CallDemoRoomDataFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        8554778 => DemoRoom_ChargeBeamRoomScroll21; 8554788 => [()];
        8554789 => DemoRoom_SetBG2TilemapBase; 8554795 => DemoRoom_SetKraidFunctionTimer;
        8554802 => DemoRoom_SetBrinstarBossBits;
    }
}
pub unsafe fn InitAndLoadGameData_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x612);
    's_445: {
        let mut current_block_103: u64;
        match *_state_variable_ as i32 {
            0 => {
                if ram_word!(game, game_state) as i32 == kGameState_40_TransitionToDemo as i32 {
                    InitIoForGameplay(game);
                    LoadStdBG3andSpriteTilesClearTilemaps(game);
                    LoadInitialPalette(game);
                    Samus_Initialize(game);
                    LoadDemoRoomData(game);
                } else {
                    if ram_word!(game, loading_game_state) as i32 != kLoadingGameState_5_Main as i32
                    {
                        if ram_word!(game, loading_game_state) as i32
                            == kLoadingGameState_1F_StartingAtCeres as i32
                        {
                            set_ram_word!(game.area_index = 6u16);
                            set_ram_word!(game.load_station_index = 0u16);
                            ClearTimerRam(game);
                        } else if ram_word!(game, loading_game_state) as i32
                            == kLoadingGameState_22_EscapingCeres as i32
                        {
                            set_ram_word!(game.area_index = 0u16);
                            set_ram_word!(game.load_station_index = 18u16);
                            LoadMirrorOfExploredMapTiles(game);
                        }
                    }
                    InitIoForGameplay(game);
                    LoadStdBG3andSpriteTilesClearTilemaps(game);
                    LoadInitialPalette(game);
                    Samus_Initialize(game);
                    LoadFromLoadStation(game);
                }
                current_block_103 = 9959648584402098530;
            }
            1 => {
                current_block_103 = 9959648584402098530;
            }
            2 => {
                current_block_103 = 6249748917338837630;
            }
            3 => {
                current_block_103 = 4453751417072702604;
            }
            4 => {
                current_block_103 = 12557724779959852293;
            }
            _ => {
                current_block_103 = 12705158477165241210;
            }
        }
        if current_block_103 == 9959648584402098530 {
            let mut _coret_: u8 = StartGameplay_Async(game);
            if _coret_ != 0 {
                *_state_variable_ = 1u8;
                return _coret_;
            }
            InitializeHud(game);
            let mut v1: u16 = 0u16;
            for _ in (0i16..32i16).step_by(2) {
                *(ram_ptr!(game, u16; 0xc200)).offset(((v1 as i32 >> 1i32) + 192i32) as isize) =
                    *(ram_ptr!(game, u16; 0xc000)).offset(((v1 as i32 >> 1i32) + 192i32) as isize);
                v1 = (v1).wrapping_add(2u16);
            }
            set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
            EnableNMI(game);
            EnableEprojs(game);
            EnablePLMs(game);
            EnablePaletteFx(game);
            EnableHdmaObjects(game);
            EnableAnimtiles(game);
            SetLiquidPhysicsType(game);
            if ram_word!(game, game_state) as i32 == kGameState_40_TransitionToDemo as i32 {
                set_ram_word!(game.loop_counter_transfer_enemies_to_vram = 6u16);
                current_block_103 = 980989089337379490;
            } else if ram_word!(game, loading_game_state) as i32
                == kLoadingGameState_22_EscapingCeres as i32
            {
                QueueMusic_Delayed8(5u16, game);
                set_ram_word!(game.loop_counter_transfer_enemies_to_vram = 15u16);
                current_block_103 = 10753070352654377903;
            } else {
                set_ram_word!(game.loop_counter_transfer_enemies_to_vram = 6u16);
                current_block_103 = 129780949503461575;
            }
        }
        loop {
            match current_block_103 {
                129780949503461575 => {
                    TransferEnemyTilesToVramAndInit(game);
                    current_block_103 = 12557724779959852293;
                }
                4453751417072702604 => {
                    let mut _coret__1: u8 = WaitForNMI_Async(game);
                    if _coret__1 != 0 {
                        *_state_variable_ = 3u8;
                        return _coret__1;
                    }
                    update_ram_word!(game.loop_counter_transfer_enemies_to_vram, wrapping_sub(1));
                    if (ram_word!(game, loop_counter_transfer_enemies_to_vram) as i32 & 0x8000i32)
                        == 0i32
                    {
                        current_block_103 = 10753070352654377903;
                        continue;
                    }
                    update_ram_word!(game.game_state, wrapping_add(1));
                    let mut v3: u16 = 0u16;
                    for _ in (0i16..512i16).step_by(2) {
                        *(ram_ptr!(game, u16; 0xc000)).offset((v3 as i32 >> 1i32) as isize) =
                            ram_word!(game, target_palettes[(v3 as i32 >> 1i32)]);
                        v3 = (v3).wrapping_add(2u16);
                    }
                    current_block_103 = 12705158477165241210;
                }
                6249748917338837630 => {
                    let mut _coret__0: u8 = WaitForNMI_Async(game);
                    if _coret__0 != 0 {
                        *_state_variable_ = 2u8;
                        return _coret__0;
                    }
                    update_ram_word!(game.loop_counter_transfer_enemies_to_vram, wrapping_sub(1));
                    if (ram_word!(game, loop_counter_transfer_enemies_to_vram) as i32 & 0x8000i32)
                        == 0i32
                    {
                        current_block_103 = 980989089337379490;
                        continue;
                    }
                    let r18: u16 = (*get_DemoRoomData(
                        (crate::rtl::rom_table_word!(
                            game,
                            0x82876c,
                            ram_word!(game, demo_set) as isize
                        ) as i32
                            + 18i32 * (ram_word!(game, demo_scene) as i32 - 1i32))
                            as u16,
                        game,
                    ))
                    .demo_code_ptr;
                    CallDemoRoomDataFunc(((r18) as u32) | (0x820000u32), game);
                    update_ram_word!(game.game_state, wrapping_add(1));
                    let mut v8: u16 = 0u16;
                    for _ in (0i16..512i16).step_by(2) {
                        *(ram_ptr!(game, u16; 0xc000)).offset((v8 as i32 >> 1i32) as isize) =
                            ram_word!(game, target_palettes[(v8 as i32 >> 1i32)]);
                        v8 = (v8).wrapping_add(2u16);
                    }
                    current_block_103 = 12705158477165241210;
                }
                12705158477165241210 => {
                    break 's_445;
                }
                980989089337379490 => {
                    TransferEnemyTilesToVramAndInit(game);
                    current_block_103 = 6249748917338837630;
                }
                10753070352654377903 => {
                    TransferEnemyTilesToVramAndInit(game);
                    current_block_103 = 4453751417072702604;
                }
                _ => {
                    let mut _coret__2: u8 = WaitForNMI_Async(game);
                    if _coret__2 != 0 {
                        *_state_variable_ = 4u8;
                        return _coret__2;
                    }
                    update_ram_word!(game.loop_counter_transfer_enemies_to_vram, wrapping_sub(1));
                    if (ram_word!(game, loop_counter_transfer_enemies_to_vram) as i32 & 0x8000i32)
                        == 0i32
                    {
                        current_block_103 = 129780949503461575;
                        continue;
                    }
                    set_ram_word!(game.game_state = kGameState_7_MainGameplayFadeIn as u16);
                    let mut v5: u16 = 0u16;
                    for _ in (0i16..512i16).step_by(2) {
                        *(ram_ptr!(game, u16; 0xc000)).offset((v5 as i32 >> 1i32) as isize) =
                            ram_word!(game, target_palettes[(v5 as i32 >> 1i32)]);
                        v5 = (v5).wrapping_add(2u16);
                    }
                    if ram_word!(game, loading_game_state) as i32
                        == kLoadingGameState_1F_StartingAtCeres as i32
                    {
                        set_ram_word!(game.palette_buffer[223] = 0u16);
                        CallSomeSamusCode(8u16, game);
                    } else {
                        CallSomeSamusCode(9u16, game);
                    }
                    current_block_103 = 12705158477165241210;
                }
            }
        }
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
pub unsafe fn InitIoForGameplay(game: &mut Game) {
    InitCpuForGameplay(game);
    InitPpuForGameplay(game);
}
pub unsafe fn InitCpuForGameplay(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(NMITIMEN as u16, 1, 0x84, false); byte(WRIO as u16, 0u8);
        byte(WRMPYA as u16, 0u8); byte(WRMPYB as u16, 0u8);
        byte(WRDIVL as u16, 0u8); byte(WRDIVH as u16, 0u8);
        byte(WRDIVB as u16, 0u8); byte(HTIMEL as u16, 0u8);
        byte(HTIMEH as u16, 0u8); byte(VTIMEL as u16, 0u8);
        byte(VTIMEH as u16, 0u8); byte(MDMAEN as u16, 0u8);
        mirror(HDMAEN as u16, 0, 0x85, false); mirror(MEMSEL as u16, 1, 0x86, false);
    }
}
pub unsafe fn InitPpuForGameplay(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
        byte(OAMADDL as u16, 0u8);
    }
    *(ram_ptr!(game, u16; 0x53) as *mut u8).offset(0) = 0u8;
    WriteReg(OAMADDH as u16, 0x80u8, game);
    *(ram_ptr!(game, u16; 0x53) as *mut u8).offset(1isize) = 0x80u8;
    crate::rtl::register_writes! { game;
        byte(OAMDATA as u16, 0u8); byte(OAMDATA as u16, 0u8);
        mirror(BGMODE as u16, 9, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, 0x51, 0x58, true); mirror(BG2SC as u16, 0x49, 0x59, true);
        mirror(BG3SC as u16, 0x5a, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        byte(VMAIN as u16, 0u8); mirror(W12SEL as u16, 0, 0x60, false);
        mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
        mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
        mirror(WH1 as u16, 0, 0x64, false); mirror(WH2 as u16, 0, 0x65, false);
        mirror(WH3 as u16, 0, 0x66, false); mirror(WBGLOG as u16, 0, 0x67, false);
        mirror(WOBJLOG as u16, 0, 0x68, false); mirror(TM as u16, 0x17, 0x69, false);
        mirror(TMW as u16, 0, 0x6c, false); mirror(TS as u16, 0, 0x6b, false);
        mirror(TSW as u16, 0, 0x6d, false); mirror(CGWSEL as u16, 0, 0x6e, false);
        mirror(CGADSUB as u16, 0, 0x71, false); byte(COLDATA as u16, 0xe0u8);
        mirror(SETINI as u16, 0, 0x77, false);
    }
    set_ram_word!(game.oam_next_ptr = 0u16);
    memset7E(ram_ptr!(game, Ram3000; 0x3000) as *mut u16, 0u16, 0x7feu16);
    memset7E(
        ram_ptr!(game, Ram4000; 0x4000) as *mut u16,
        0x6fu16,
        0x7feu16,
    );
    memset7E(
        &raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_tilemap as *mut u16,
        0x2c0fu16,
        0xfeu16,
    );
}
pub unsafe fn LoadInitialPalette(game: &mut Game) {
    let mut v1: u16 = 0u16;
    for _ in (0i16..512i16).step_by(2) {
        set_ram_word!(
            game.palette_buffer[(v1 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x9a8000, (v1 as i32 >> 1i32) as isize)
        );
        v1 = (v1).wrapping_add(2u16);
    }
}
pub unsafe fn LoadStdBG3andSpriteTilesClearTilemaps(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0x40u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_8282F8: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ab200, 0x2000);
    SetupDmaTransfer(&raw const unk_8282F8 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x60u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_828318: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ad200, 0x2e00);
    SetupDmaTransfer(&raw const unk_828318 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x50u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_828338: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x1000);
    SetupDmaTransfer(&raw const unk_828338 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x58u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_828358: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x800);
    SetupDmaTransfer(&raw const unk_828358 as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn GameState_32_MadeItToCeresElevator(game: &mut Game) -> CoroutineRet {
    if ram_word!(game, timer_status) != 0 {
        DrawTimer(game);
    }
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    let v0: bool = (update_ram_word!(
        game.reached_ceres_elevator_fade_timer,
        wrapping_sub(1),
        reread
    ) as i32
        & 0x8000i32)
        != 0i32;
    if ram_word!(game, reached_ceres_elevator_fade_timer) == 0 || v0 {
        update_ram_word!(game.game_state, wrapping_add(1));
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_33_BlackoutFromCeres(game: &mut Game) -> CoroutineRet {
    if ram_word!(game, timer_status) != 0 {
        DrawTimer(game);
    }
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        set_ram_words! { game, 0u16; fx_layer_blending_config_a, cur_irq_handler }
        *ram_ptr!(game; 0x6e) = 0u8;
        *ram_ptr!(game; 0x71) = 0u8;
        *ram_ptr!(game; 0x69) = 16u8;
        *ram_ptr!(game; 0x6b) = 0u8;
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
        *ram_ptr!(game; 0x55) = 9u8;
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.loading_game_state = kLoadingGameState_22_EscapingCeres as u16);
        set_ram_word!(game.game_state = 34u16);
        SaveToSram(ram_word!(game, selected_save_slot), game);
        set_ram_word!(game.cinematic_function = ((0x8bc11bi32) as u16));
        set_ram_words! { game, 0u16; ceres_status, timer_status }
        QueueMusic_Delayed8(0u16, game);
        QueueSfx1_Max15(2u16, game);
        QueueSfx2_Max15(0x71u16, game);
        QueueSfx3_Max15(1u16, game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_35_TimeUp(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    set_ram_word!(game.palette_change_denom = 8u16);
    if AdvancePaletteFadeForAllPalettes(game) != 0 {
        set_ram_word!(game.game_state = kGameState_36_WhitingOutFromTimeUp as u16);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_36_WhitingOutFromTimeUp(game: &mut Game) -> CoroutineRet {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        set_ram_words! { game, 0u16; fx_layer_blending_config_a, cur_irq_handler }
        *ram_ptr!(game; 0x6e) = 0u8;
        *ram_ptr!(game; 0x71) = 0u8;
        *ram_ptr!(game; 0x69) = 16u8;
        *ram_ptr!(game; 0x6b) = 0u8;
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
        *ram_ptr!(game; 0x55) = 9u8;
        set_ram_words! { game, 0u16;
            screen_fade_delay, screen_fade_counter, ceres_status, timer_status,
        }
        QueueSfx1_Max15(2u16, game);
        QueueSfx2_Max15(0x71u16, game);
        QueueSfx3_Max15(1u16, game);
        if CheckEventHappened(0xeu16, game) != 0 {
            set_ram_words! { game, 0u16; game_options_screen_index, menu_index }
            let mut i: i32 = 254i32;
            while i >= 0i32 {
                set_ram_word!(game.eproj_y_subpos[((i >> 1i32) + 15i32)] = 0u16);
                i -= 2i32;
            }
            set_ram_word!(game.game_state = kGameState_25_SamusNoHealth as u16);
        } else {
            set_ram_word!(game.game_state = kGameState_37_CeresGoesBoomWithSamus as u16);
            set_ram_word!(game.cinematic_function = ((0x8bc11bi32) as u16));
        }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_38_SamusEscapesFromZebes(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        set_ram_word!(game.fx_layer_blending_config_a = 0u16);
        *ram_ptr!(game; 0x6e) = 0u8;
        *ram_ptr!(game; 0x71) = 0u8;
        *ram_ptr!(game; 0x69) = 16u8;
        *ram_ptr!(game; 0x6b) = 0u8;
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
        *ram_ptr!(game; 0x55) = 9u8;
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.game_state = kGameState_39_EndingAndCredits as u16);
        set_ram_word!(game.cinematic_function = ((0x8bd480i32) as u16));
        set_ram_word!(game.timer_status = 0u16);
        QueueMusic_Delayed8(0u16, game);
        QueueSfx1_Max15(2u16, game);
        QueueSfx2_Max15(0x71u16, game);
        QueueSfx3_Max15(1u16, game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_41_TransitionToDemo(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HdmaObjectHandler(game);
    update_ram_word!(game.game_state, wrapping_add(1));
    *ram_ptr!(game; 0x51) = 15u8;
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_42_PlayingDemo_Async(game: &mut Game) -> CoroutineRet {
    let mut current_block: u64;
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x614);
    match *_state_variable_ as i32 {
        0 | 2 => {
            let mut _coret_: u8 = GameState_8_MainGameplay(game);
            if _coret_ != 0 {
                *_state_variable_ = 2u8;
                return _coret_;
            }
            if ram_word!(game, joypad1_newkeys) != 0 {
                set_ram_word!(game.substate = 1u16);
                current_block = 4040822987312515878;
            } else {
                update_ram_word!(game.demo_timer, wrapping_sub(1));
                if ram_word!(game, demo_timer) == 0
                    || (ram_word!(game, demo_timer) as i32 & 0x8000i32) != 0i32
                {
                    set_ram_word!(game.substate = 0u16);
                    set_ram_word!(game.my_counter = 90u16);
                    current_block = 12296459429166835708;
                } else {
                    current_block = 13472856163611868459;
                }
            }
        }
        1 => {
            current_block = 12296459429166835708;
        }
        _ => {
            current_block = 13472856163611868459;
        }
    }
    loop {
        match current_block {
            13472856163611868459 => {
                *_state_variable_ = 0u8;
                break;
            }
            12296459429166835708 => {
                let mut _coret__0: u8 = WaitForNMI_Async(game);
                if _coret__0 != 0 {
                    *_state_variable_ = 1u8;
                    return _coret__0;
                }
                if ram_word!(game, joypad1_newkeys) != 0 {
                    set_ram_word!(game.substate = 1u16);
                    current_block = 4040822987312515878;
                } else {
                    if update_ram_word!(game.my_counter, wrapping_sub(1), reread) == 0 {
                        current_block = 4040822987312515878;
                    } else {
                        current_block = 12296459429166835708;
                    }
                }
            }
            _ => {
                update_ram_word!(game.game_state, wrapping_add(1));
                set_ram_word!(game.debug_disable_sounds = 0u16);
                *ram_ptr!(game; 0x51) = 0x80u8;
                set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
                current_block = 13472856163611868459;
            }
        }
    }
    0 as CoroutineRet
}
pub unsafe fn GameState_43_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    if ram_word!(game, substate) != 1u16 {
        CheckForNextDemo(game);
    }
    EnableNMI(game);
    update_ram_word!(game.game_state, wrapping_add(1));
    set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
    WaitUntilEndOfVblankAndClearHdma(game);
    DisableIrqInterrupts(game);
    set_ram_words! { game, 0u16;
        fx_layer_blending_config_a, cur_irq_handler, irqhandler_next_handler,
    }
    DisablePaletteFx(game);
    ClearPaletteFXObjects(game);
    let mut i: i32 = 656i32;
    while i >= 0i32 {
        *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(i as isize) as *mut u16) = 0u16;
        i -= 2i32;
    }
    let mut j: i32 = 538i32;
    while j >= 0i32 {
        *((ram_ptr!(game, u16; 0x77c) as *mut u8).offset(j as isize) as *mut u16) = 0u16;
        j -= 2i32;
    }
    *ram_ptr!(game; 0x6e) = 0u8;
    *ram_ptr!(game; 0x71) = 0u8;
    *ram_ptr!(game; 0x69) = 16u8;
    *ram_ptr!(game; 0x6b) = 0u8;
    *ram_ptr!(game; 0x6c) = 0u8;
    *ram_ptr!(game; 0x6d) = 0u8;
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_44_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    set_ram_word!(game.game_state = kGameState_1_OpeningCinematic as u16);
    if (ram_word!(game, substate) as i32 & 0x8000i32) != 0i32 {
        set_ram_word!(game.game_state = kGameState_40_TransitionToDemo as u16);
    } else if ram_word!(game, substate) != 0 {
        LoadTitleSequenceGraphics(game);
        set_ram_word!(game.eproj_x_pos[4] = 2u16);
        set_ram_word!(game.cinematic_function = ((0x8b9a47i32) as u16));
    } else {
        QueueMusic_Delayed8(0u16, game);
        set_ram_word!(game.debug_disable_sounds = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8b9b68i32) as u16));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn CheckForNextDemo(game: &mut Game) {
    if *(RomPtr_82(
        (crate::rtl::rom_table_word!(game, 0x82876c, ram_word!(game, demo_set) as isize) as i32
            + 18i32 * ram_word!(game, demo_scene) as i32) as u16,
        game,
    ) as *mut u16)
        == 0xffffu16
    {
        set_ram_word!(game.substate = 0u16);
        let mut v0: u16 = (ram_word!(game, demo_set)).wrapping_add(1u16);
        if v0 >= ram_word!(game, num_demo_sets) {
            v0 = 0u16;
        }
        set_ram_word!(game.demo_set = v0);
        set_ram_word!(game.demo_scene = 0u16);
    } else {
        set_ram_word!(game.substate = 0x8000u16);
    };
}
pub unsafe fn LoadDemoRoomData(game: &mut Game) {
    set_ram_word!(game.door_def_ptr = 0u16);
    let mut drd: *mut DemoRoomData = get_DemoRoomData(
        (18i32 * ram_word!(game, demo_scene) as i32
            + crate::rtl::rom_table_word!(game, 0x82876c, ram_word!(game, demo_set) as isize)
                as i32) as u16,
        game,
    );
    set_ram_word!(game.room_ptr = (*drd).room_ptr_);
    set_ram_word!(game.door_def_ptr = (*drd).door_ptr);
    set_ram_word!(game.layer1_x_pos = (*drd).screen_x_pos);
    set_ram_word!(game.bg1_x_offset = ram_word!(game, layer1_x_pos));
    set_ram_word!(game.layer1_y_pos = (*drd).screen_y_pos);
    set_ram_word!(game.bg1_y_offset = ram_word!(game, layer1_y_pos));
    set_ram_word!(
        game.samus_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add((*drd).samus_x_offs)
    );
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    set_ram_word!(
        game.samus_x_pos =
            ((*drd).samus_y_offs as i32 + ram_word!(game, layer1_x_pos) as i32 + 128i32) as u16
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    set_ram_word!(game.demo_timer = (*drd).demo_length);
    *(ram_ptr!(game, u16; 0x79f) as *mut u8).offset(0) =
        (*get_RoomDefHeader(ram_word!(game, room_ptr), game)).area_index_;
    set_ram_words! { game, 0u16; reg_BG1HOFS, reg_BG1VOFS }
    update_ram_word!(game.demo_scene, wrapping_add(1));
    let mut v1: u16 = 0u16;
    loop {
        let v2: i32 = v1 as i32 >> 1i32;
        *(ram_ptr!(game, u16; 0xd830, v1 as isize)) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0xd870, v1 as isize)) = -1i32 as u16;
        set_ram_word!(game.UNUSED_word_7ED8F0[v2] = -1i32 as u16);
        *(ram_ptr!(game, u16; 0xd908, v1 as isize)) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0xd8f8, v1 as isize)) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0xd8f8, (v1 as i32 + 8i32) as isize)) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0xd8b0, v1 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd820, v1 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd828, v1 as isize)) = 0u16;
        v1 = (v1).wrapping_add(2u16);
        if (((v1) as i16).wrapping_sub(8i16) >= 0i16) {
            break;
        }
    }
    loop {
        let v3: i32 = v1 as i32 >> 1i32;
        *(ram_ptr!(game, u16; 0xd830, v1 as isize)) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0xd870, v1 as isize)) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0xd8b0, v1 as isize)) = 0u16;
        v1 = (v1).wrapping_add(2u16);
        if (((v1) as i16).wrapping_sub(64i16) >= 0i16) {
            break;
        }
    }
    let mut v4: u16 = 0u16;
    loop {
        set_ram_word!(game.explored_map_tiles_saved[(v4 as i32 >> 1i32)] = 0u16);
        v4 = (v4).wrapping_add(2u16);
        if (((v4) as i16).wrapping_sub(1536i16) >= 0i16) {
            break;
        }
    }
    set_ram_words! { game, 0u16;
        samus_max_reserve_health, samus_reserve_health, reserve_health_mode,
    }
    set_ram_word!(game.loading_game_state = kLoadingGameState_0_Intro as u16);
    set_ram_word!(game.debug_disable_minimap = 0u16);
}
pub unsafe fn DemoRoom_ChargeBeamRoomScroll21(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 33) = 0u8;
}
pub unsafe fn DemoRoom_SetBG2TilemapBase(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x59)) = 74u16;
}
pub unsafe fn DemoRoom_SetKraidFunctionTimer(game: &mut Game) {
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_preinstr = 60u16;
}
pub unsafe fn DemoRoom_SetBrinstarBossBits(game: &mut Game) {
    *ram_ptr!(game; 0xd828, 1) = 1u8;
}
static kGameStateFuncs: [Option<Func_V_Coroutine>; 45] = callback_table! { unsafe Func_V_Coroutine;
    GameState_0_Reset_Async, GameState_1_OpeningCinematic, GameState_2_GameOptionsMenu,
    GameState_3_Null, GameState_4_FileSelectMenus, GameState_5_FileSelectMap,
    InitAndLoadGameData_Async, GameState_7_MainGameplayFadeIn, GameState_8_MainGameplay,
    GameState_9_HitDoorBlock, GameState_10_LoadingNextRoom_Async,
    GameState_11_LoadingNextRoom_Async, GameState_12_Pausing_Darkening_Async,
    GameState_13_Pausing_Async, GameState_14_Paused_Async, GameState_15_Paused_Async,
    GameState_16_Unpausing_Async, GameState_17_Unpausing_Async, GameState_18_Unpausing,
    GameState_19_SamusNoHealth, GameState_20_SamusNoHealth_BlackOut, GameState_21_SamusNoHealth,
    GameState_22_SamusNoHealth_Dying, GameState_23_SamusNoHealth_Flashing,
    GameState_24_SamusNoHealth_Explosion, GameState_25_SamusNoHealth_BlackOut,
    GameState_26_GameOverMenu, GameState_27_ReserveTanksAuto, GameState_28_Unused,
    GameState_29_DebugGameOverMenu, GameState_37_CeresGoesBoomWithSamus,
    InitAndLoadGameData_Async, GameState_32_MadeItToCeresElevator,
    GameState_33_BlackoutFromCeres, GameState_37_CeresGoesBoomWithSamus, GameState_35_TimeUp,
    GameState_36_WhitingOutFromTimeUp, GameState_37_CeresGoesBoomWithSamus,
    GameState_38_SamusEscapesFromZebes, GameState_39_EndingAndCredits,
    InitAndLoadGameData_Async, GameState_41_TransitionToDemo, GameState_42_PlayingDemo_Async,
    GameState_43_TransitionFromDemo, GameState_44_TransitionFromDemo,
};
pub unsafe fn RunOneFrameOfGameInner(game: &mut Game) -> CoroutineRet {
    let mut st: i32 = *ram_ptr!(game; 0x611) as i32;
    let begin_frame: bool = if st >= 1i32 && st <= 3i32 {
        let mut _coret_: u8 = Vector_RESET_Async(game);
        if _coret_ != 0 {
            return _coret_;
        }
        true
    } else {
        !(st >= 10i32)
            && ({
                if st != 0i32 {
                    Die(b"Incorrect coroutine_state_0\0" as *const u8 as *const c_char);
                }
                true
            })
    };
    if begin_frame {
        let ref mut fresh149 = *ram_ptr!(game; 0x615);
        *fresh149 = 0u8;
        let ref mut fresh150 = *ram_ptr!(game; 0x614);
        *fresh150 = *fresh149;
        let ref mut fresh151 = *ram_ptr!(game; 0x613);
        *fresh151 = *fresh150;
        *ram_ptr!(game; 0x612) = *fresh151;
        ReadJoypadInputs(game);
        HdmaObjectHandler(game);
        NextRandom(game);
        ClearOamExt(game);
        set_ram_words! { game, 0u16;
            oam_next_ptr, nmi_copy_samus_halves, nmi_copy_samus_top_half_src,
            nmi_copy_samus_bottom_half_src,
        }
        st = ram_word!(game, game_state) as i32 + 10i32;
        *ram_ptr!(game; 0x611) = st as u8;
    }
    let mut _coret__0: u8 =
        kGameStateFuncs[(st - 10i32) as usize].expect("non-null function pointer")(game);
    if _coret__0 != 0 {
        return _coret__0;
    }
    HandleSoundEffects(game);
    ClearUnusedOam(game);
    ShowSpareCpu(game);
    if *ram_ptr!(game; 0x612) as i32
        | *ram_ptr!(game; 0x613) as i32
        | *ram_ptr!(game; 0x614) as i32
        | *ram_ptr!(game; 0x615) as i32
        != 0
    {
        printf(
            b"Coroutine state: %d, %d, %d, %d\n\0" as *const u8 as *const c_char,
            *ram_ptr!(game; 0x612) as i32,
            *ram_ptr!(game; 0x613) as i32,
            *ram_ptr!(game; 0x614) as i32,
            *ram_ptr!(game; 0x615) as i32,
        );
        Warning(b"Coroutine State is broken!\0" as *const u8 as *const c_char);
    }
    *ram_ptr!(game; 0x611) = 0u8;
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RunOneFrameOfGame(game: &mut Game) {
    let ret: CoroutineRet = RunOneFrameOfGameInner(game);
    if ret == 0u8 {
        *ram_ptr!(game; 0x5b4) = 1u8;
    }
    Vector_NMI(game);
}
pub unsafe fn GameState_29_DebugGameOverMenu(game: &mut Game) -> CoroutineRet {
    DebugGameOverMenu();
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_26_GameOverMenu(game: &mut Game) -> CoroutineRet {
    GameOverMenu(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_4_FileSelectMenus(game: &mut Game) -> CoroutineRet {
    FileSelectMenu(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_5_FileSelectMap(game: &mut Game) -> CoroutineRet {
    FileSelectMap(game);
    kCoroutineNone as CoroutineRet
}
static kSfxHandlers: [Option<Func_Y_V>; 5] = callback_table! { unsafe Func_Y_V;
    SfxHandlers_0_SendToApu, SfxHandlers_1_WaitForAck, SfxHandlers_2_ClearRequest,
    SfxHandlers_3_WaitForAck, SfxHandlers_4_Reset,
};
pub unsafe fn HandleSoundEffects(game: &mut Game) {
    if ((ram_word!(game, sound_handler_downtime)) as i8).wrapping_sub(1i8) >= 0i8 {
        *(ram_ptr!(game, u16; 0x686) as *mut u8).offset(0) =
            ((ram_word!(game, sound_handler_downtime)) as u8).wrapping_sub(1u8);
        RtlApuWrite(APUI01, 0u8, game);
        *ram_ptr!(game; 0x64d, 0) = 0u8;
        RtlApuWrite(APUI02, 0u8, game);
        *ram_ptr!(game; 0x64d, 1) = 0u8;
        RtlApuWrite(APUI03, 0u8, game);
        *ram_ptr!(game; 0x64d, 2) = 0u8;
    } else {
        let mut i: i32 = 0i32;
        while i < 3i32 {
            kSfxHandlers[*ram_ptr!(game; 0x649, i as isize) as usize]
                .expect("non-null function pointer")(i as u16, game);
            i += 1;
        }
    };
}
pub unsafe fn SfxHandlers_0_SendToApu(j: u16, game: &mut Game) {
    if *ram_ptr!(game; 0x643, j as isize) != *ram_ptr!(game; 0x646, j as isize) {
        let v2: u8 = (*ram_ptr!(game; 0x643, j as isize) as i32 + j as i32 * 16i32) as u8;
        let v3: u8 = *ram_ptr!(game; 0x656, v2 as isize);
        RtlApuWrite((j as i32 + APUI01 as i32) as SnesRegs, v3, game);
        *ram_ptr!(game; 0x64d, j as isize) = v3;
        *ram_ptr!(game; 0x643, j as isize) = (v2 as i32 + 1i32 & 0xfi32) as u8;
        let ref mut fresh134 = *ram_ptr!(game; 0x649, j as isize);
        *fresh134 = (*fresh134).wrapping_add(1);
    }
}
pub unsafe fn SfxHandlers_1_WaitForAck(j: u16, game: &mut Game) {
    let v1: u8 = *ram_ptr!(game; 0x64d, j as isize);
    let ref mut fresh138 = *ram_ptr!(game; 0x649, j as isize);
    *fresh138 = (*fresh138).wrapping_add(1);
    *ram_ptr!(game; 0x650, j as isize) = 6u8;
}
pub unsafe fn SfxHandlers_2_ClearRequest(j: u16, game: &mut Game) {
    let ref mut fresh135 = *ram_ptr!(game; 0x650, j as isize);
    let fresh136 = *fresh135;
    *fresh135 = (*fresh135).wrapping_sub(1);
    if fresh136 == 1u8 {
        *ram_ptr!(game; 0x64d, j as isize) = 0u8;
        let ref mut fresh137 = *ram_ptr!(game; 0x649, j as isize);
        *fresh137 = (*fresh137).wrapping_add(1);
    }
}
pub unsafe fn SfxHandlers_3_WaitForAck(j: u16, game: &mut Game) {
    let v1: u8 = *ram_ptr!(game; 0x64d, j as isize);
    *ram_ptr!(game; 0x649, j as isize) = 0u8;
    SfxHandlers_0_SendToApu(j, game);
}
pub unsafe fn SfxHandlers_4_Reset(j: u16, game: &mut Game) {
    let ref mut fresh132 = *ram_ptr!(game; 0x650, j as isize);
    let fresh133 = *fresh132;
    *fresh132 = (*fresh132).wrapping_sub(1);
    if fresh133 == 1u8 {
        *ram_ptr!(game; 0x649, j as isize) = 0u8;
    }
}
pub unsafe fn ResetSoundQueues(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x643)) = 0u16;
    *(ram_ptr!(game, u16; 0x643, 2)) = 0u16;
    *(ram_ptr!(game, u16; 0x646, 1)) = 0u16;
    *(ram_ptr!(game, u16; 0x649)) = 0u16;
    *ram_ptr!(game; 0x649, 2) = 0u8;
}
pub unsafe fn ShowSpareCpu(game: &mut Game) {
    set_ram_word!(game.joypad1_input_samusfilter = ram_word!(game, joypad1_lastkeys));
}
pub unsafe fn GameState_0_Reset_Async(game: &mut Game) -> CoroutineRet {
    set_ram_words! { game, 0u16; UNUSED_word_7E0DF8, UNUSED_word_7E0DFA, UNUSED_word_7E0DFC }
    set_ram_word!(game.cinematic_function = ((0x8b9b68i32) as u16));
    set_ram_word!(game.demo_set = 0u16);
    if ram_word!(game, num_demo_sets) == 4u16 {
        set_ram_word!(game.demo_set = 3u16);
    }
    update_ram_word!(game.game_state, wrapping_add(1));
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_1_OpeningCinematic(game: &mut Game) -> CoroutineRet {
    GameState_1_OpeningCinematic_(game)
}
pub unsafe fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_37_CeresGoesBoomWithSamus(game: &mut Game) -> CoroutineRet {
    GameState_37_CeresGoesBoomWithSamus_(game)
}
pub unsafe fn GameState_39_EndingAndCredits(game: &mut Game) -> CoroutineRet {
    GameState_39_EndingAndCredits_(game)
}
pub unsafe fn GameState_7_MainGameplayFadeIn(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15u8 {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    GameState_28_Unused_(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_8_MainGameplay(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x612);
    let mut current_block_38: u64;
    match *_state_variable_ as i32 {
        0 => {
            DetermineWhichEnemiesToProcess(game);
            PaletteFxHandler(game);
            HandleControllerInputForGamePhysics(game);
            if ram_word!(game, debug_disable_sprite_interact) == 0 {
                SamusProjectileInteractionHandler(game);
            }
            EnemyMain(game);
            if ram_word!(game, queued_message_box_index) != 0 {
                current_block_38 = 6718109325120641364;
            } else {
                current_block_38 = 13586036798005543211;
            }
        }
        1 => {
            current_block_38 = 6718109325120641364;
        }
        2 => {
            current_block_38 = 8001970166466707590;
        }
        _ => {
            current_block_38 = 652864300344834934;
        }
    }
    if current_block_38 == 6718109325120641364 {
        let mut _coret_: u8 =
            DisplayMessageBox_Async(ram_word!(game, queued_message_box_index), game);
        if _coret_ != 0 {
            *_state_variable_ = 1u8;
            return _coret_;
        }
        set_ram_word!(game.queued_message_box_index = 0u16);
        current_block_38 = 13586036798005543211;
    }
    if current_block_38 == 13586036798005543211 {
        HandleSamusMovementAndPause(game);
        EprojRunAll(game);
        current_block_38 = 8001970166466707590;
    }
    if current_block_38 == 8001970166466707590 {
        let mut _coret__0: u8 = PlmHandler_Async(game);
        if _coret__0 != 0 {
            *_state_variable_ = 2u8;
            return _coret__0;
        }
        AnimtilesHandler(game);
        if ram_word!(game, debug_disable_sprite_interact) == 0 {
            EprojSamusCollDetect(game);
            EprojProjCollDet(game);
            ProcessEnemyPowerBombInteraction(game);
        }
        MainScrollingRoutine(game);
        let debug_scrolling_enabled: i32 = 0i32;
        if debug_scrolling_enabled != 0 {
            DebugScrollPosSaveLoad(game);
        }
        DrawSamusEnemiesAndProjectiles(game);
        QueueEnemyBG2TilemapTransfers(game);
        HandleHudTilemap(game);
        CalculateLayer2PosAndScrollsWhenScrolling(game);
        RunRoomMainCode(game);
        HandleSamusOutOfHealthAndGameTile(game);
        HandleRoomShaking(game);
        DecrementSamusTimers(game);
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
pub unsafe fn DeleteAllOptionsMenuObjects_(game: &mut Game) {
    let mut i: i32 = 14i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_words! { game, 0u16; eproj_y_pos[(v1 + 5i32)], eproj_y_vel[(v1 + 17i32)] }
        i -= 2i32;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallOptionsEntryFunc(ea: u32, j: u16, game: &mut Game) => ea;
        call (j, game); default { Unreachable(game); };
    {
        8581782 => sub_82F296; 8581963 => sub_82F34B; 8581971 => sub_82F353; 8581979 => sub_82F35B;
        8581987 => sub_82F363; 8582169 => sub_82F419;
    }
}
pub unsafe fn CreateOptionsMenuObject_(a: u16, j: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.options_menu_init_param = a);
    let mut v3: u16 = 14u16;
    while ram_word!(game, cinematicspr_arr7[(v3 as i32 >> 1i32)]) != 0 {
        v3 = (v3).wrapping_sub(2u16);
        if (v3 as i32 & 0x8000i32) != 0i32 {
            return 1u8;
        }
    }
    let v5: *const u8 = RomPtr_82(j, game);
    let v6: i32 = v3 as i32 >> 1i32;
    set_ram_word!(game.optionsmenu_arr1[v6] = *(v5.offset(2) as *mut u16));
    set_ram_word!(game.cinematicspr_arr7[v6] = *(v5.offset(4) as *mut u16));
    set_ram_word!(game.cinematicspr_instr_ptr[v6] = 1u16);
    set_ram_words! { game, 0u16;
        cinematicbg_arr8[v6], optionsmenu_arr5[v6], cinematicspr_arr6[v6], optionsmenu_arr7[v6],
    }
    CallOptionsEntryFunc(((*(v5 as *mut u16)) as u32) | (0x820000u32), v3, game);
    0u8
}
pub fn OptionsPreInstr_nullsub_57(k: u16) {}
pub unsafe fn OptionsMenuFunc1(game: &mut Game) {
    let mut i: i32 = 14i32;
    while i >= 0i32 {
        set_ram_word!(game.optionsmenu_index = i as u16);
        if ram_word!(game, cinematicspr_arr7[(i >> 1i32)]) != 0 {
            OptionsMenuFunc2(i as u16, game);
            i = ram_word!(game, optionsmenu_index) as i32;
        }
        i -= 2i32;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallOptionsPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8555536 => [OptionsPreInstr_nullsub_57(k)]; 8581801 => OptionsPreInstr_F2A9;
        8582006 => OptionsPreInstr_F376; 8582048 => OptionsPreInstr_F3A0;
        8582114 => OptionsPreInstr_F3E2; 8582188 => OptionsPreInstr_F42C;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallOptionsInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 => ea;
        call (k, j, game); default Unreachable(game) as u16;
    {
        8555610 => OptionsInstr_Destroy; 8555620 => OptionsInstr_8C64;
        8555630 => OptionsInstr_SetPreInstr; 8555641 => OptionsInstr_8C79;
        8555650 => OptionsInstr_Goto; 8555657 => OptionsInstr_8C89; 8555667 => OptionsInstr_8C93;
    }
}
pub unsafe fn OptionsMenuFunc2(k: u16, game: &mut Game) {
    CallOptionsPreInstr(
        ((ram_word!(game, optionsmenu_arr1[(k as i32 >> 1i32)])) as u32) | (0x820000u32),
        k,
        game,
    );
    let v1: u16 = ram_word!(game, optionsmenu_index);
    let v2: i32 = ram_word!(game, optionsmenu_index) as i32 >> 1i32;
    let ref mut fresh119 = *(ram_ptr!(game, u16; 0x1b1d)).offset(v2 as isize);
    let fresh120 = *fresh119;
    *fresh119 = (*fresh119).wrapping_sub(1);
    if fresh120 == 1u16 {
        let mut v4: u16 = ram_word!(game, cinematicspr_arr7[v2]);
        let mut v5: *mut u16 = null_mut::<u16>();
        loop {
            v5 = RomPtr_82(v4, game) as *mut u16;
            if (*v5.offset(0) as i32 & 0x8000i32) == 0i32 {
                break;
            }
            v4 = CallOptionsInstr(
                ((*v5.offset(0)) as u32) | (0x820000u32),
                v1,
                (v4).wrapping_add(2u16),
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let v7: i32 = v1 as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_ptr[v7] = *v5.offset(0));
        set_ram_word!(game.cinematicbg_arr8[v7] = *v5.offset(1));
        set_ram_word!(game.cinematicspr_arr7[v7] = (v4).wrapping_add(4u16));
    }
}
pub unsafe fn OptionsInstr_Destroy(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_words! { game, 0u16; cinematicbg_arr8[v2], cinematicspr_arr7[v2] }
    0u16
}
pub unsafe fn OptionsInstr_8C64(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicspr_arr7[(k as i32 >> 1i32)] = (j).wrapping_sub(2u16));
    0u16
}
pub unsafe fn OptionsInstr_SetPreInstr(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.optionsmenu_arr1[(k as i32 >> 1i32)] = *(RomPtr_82(j, game) as *mut u16));
    (j).wrapping_add(2u16)
}
pub unsafe fn OptionsInstr_8C79(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.optionsmenu_arr1[(k as i32 >> 1i32)] = ((0x828c81i32) as u16));
    j
}
pub unsafe fn OptionsInstr_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_82(j, game) as *mut u16)
}
pub unsafe fn OptionsInstr_8C89(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh0 = *(ram_ptr!(game, u16; 0x1b2d)).offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 == 1u16 {
        (j).wrapping_add(2u16)
    } else {
        OptionsInstr_Goto(k, j, game)
    }
}
pub unsafe fn OptionsInstr_8C93(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.optionsmenu_arr5[(k as i32 >> 1i32)] = *(RomPtr_82(j, game) as *mut u16));
    (j).wrapping_add(2u16)
}
pub unsafe fn DrawOptionsMenuSpritemaps(game: &mut Game) {
    let mut i: i32 = 14i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, cinematicbg_arr8[v1]) != 0 {
            let v2: u16 = ram_word!(game, cinematicbg_arr8[v1]);
            DrawSpritemap(
                0x82u8,
                v2,
                ram_word!(game, optionsmenu_arr8[v1]),
                ram_word!(game, cinematicbg_arr9[v1]),
                ram_word!(game, optionsmenu_arr10[v1]),
                game,
            );
        }
        i -= 2i32;
    }
}
pub unsafe fn GameState_12_Pausing_Darkening_Async(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) as i32 & 0xfi32) == 0i32 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
crate::game::address_dispatch! {
    pub unsafe fn CallPauseHook(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        8946657 => [()]; 9423094 => PauseHook_DraygonRoom; 10994469 => PauseHook_Kraid;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallUnpauseHook_Async(ea: u32, game: &mut Game) -> CoroutineRet => ea;
        call (game); default {
            Unreachable(game);
            kCoroutineNone as CoroutineRet
        };
    {
        8946657 => [kCoroutineNone as CoroutineRet]; 9423100 => UnpauseHook_DraygonRoom;
        10994171 => UnpauseHook_Kraid_IsDead; 10994254 => UnpauseHook_Kraid_IsAlive;
        10994336 => Kraid_UnpauseHook_IsSinking; 11110243 => MotherBrainsBody_UnpauseHook;
    }
}
pub unsafe fn GameState_13_Pausing_Async(game: &mut Game) -> CoroutineRet {
    DisableHdmaObjects(game);
    crate::rtl::write_register_and_mirror(HDMAEN as u16, 0, 0x85, true, game);
    DisableAnimtiles(game);
    BackupBG2TilemapForPauseMenu(game);
    CallPauseHook(Load24(ram_ptr!(game, LongPtr; 0x601)), game);
    CancelSoundEffects(game);
    BackupSomeGfxStateForPause(game);
    LoadPauseMenuTilesAndClearBG2(game);
    LoadPauseScreenBaseTilemaps(game);
    LoadPauseMenuMapTilemapAndAreaLabel(game);
    BackupGameplayPalettesAndLoadForPause(game);
    ContinueInitPauseMenu(game);
    set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
    set_ram_word!(game.pausemenu_leftarrow_animation_frame = 0u16);
    set_ram_word!(game.pausemenu_palette_animation_timer = 1u16);
    set_ram_words! { game, 0u16; map_scrolling_direction, map_scrolling_speed_index }
    QueueClearingOfFxTilemap(game);
    update_ram_word!(game.game_state, wrapping_add(1));
    kCoroutineNone as CoroutineRet
}
pub unsafe fn BackupBG2TilemapForPauseMenu(game: &mut Game) {
    crate::rtl::register_writes! { game;
        word(VMADDL as u16,
        ((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32 | 1i32) as u16,); byte(VMAIN as u16, 0x80u8);
        byte(DMAP1 as u16, 0x81u8); byte(BBAD1 as u16, 0x39u8);
        word(A1T1L as u16, 0xdf5cu16); byte(A1B1 as u16, 0x7eu8);
        word(DAS1L as u16, 0x1000u16); byte(DAS10 as u16, 0u8);
        word(A2A1L as u16, 0u16); byte(NTRL1 as u16, 0u8);
        byte(MDMAEN as u16, 2u8);
    }
}
pub unsafe fn RestoreBG2TilemapFromPauseScreen(game: &mut Game) {
    crate::rtl::register_writes! { game;
        word(VMADDL as u16,
        ((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32) as u16,); byte(VMAIN as u16, 0x80u8);
    }
    static unk_828DAE: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7edf5c, 0x1000);
    SetupDmaTransfer(&raw const unk_828DAE as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn BackupSomeGfxStateForPause(game: &mut Game) {
    *ram_ptr!(game; 0x765) = *ram_ptr!(game; 0x58);
    *ram_ptr!(game; 0x766) = *ram_ptr!(game; 0x59);
    *ram_ptr!(game; 0x767) = *ram_ptr!(game; 0x5a);
    *ram_ptr!(game; 0x768) = *ram_ptr!(game; 0x5d);
    *ram_ptr!(game; 0x769) = *ram_ptr!(game; 0x5e);
    *ram_ptr!(game; 0x76a) = *ram_ptr!(game; 0x52);
    *ram_ptr!(game; 0x76b) = ram_word!(game, reg_BG1HOFS) as u8;
    *ram_ptr!(game; 0x76c) = ram_word!(game, reg_BG2HOFS) as u8;
    *ram_ptr!(game; 0x76d) = ram_word!(game, reg_BG3HOFS) as u8;
    *ram_ptr!(game; 0x76e) = ram_word!(game, reg_BG1VOFS) as u8;
    *ram_ptr!(game; 0x76f) = ram_word!(game, reg_BG2VOFS) as u8;
    *ram_ptr!(game; 0x770) = ram_word!(game, reg_BG3VOFS) as u8;
    *ram_ptr!(game; 0x771) = *ram_ptr!(game; 0x55);
    *ram_ptr!(game; 0x772) = *ram_ptr!(game; 0x91b);
    *ram_ptr!(game; 0x773) = *ram_ptr!(game; 0x91c);
    *ram_ptr!(game; 0x774) = *ram_ptr!(game; 0x57);
    *ram_ptr!(game; 0x775) = *ram_ptr!(game; 0x71);
}
pub unsafe fn RestoreSomeGfxStateForPause(game: &mut Game) {
    *ram_ptr!(game; 0x91b) = *ram_ptr!(game; 0x772);
    *ram_ptr!(game; 0x91c) = *ram_ptr!(game; 0x773);
    *ram_ptr!(game; 0x55) = *ram_ptr!(game; 0x771);
    *(ram_ptr!(game, u16; 0xbb) as *mut u8).offset(0) = *ram_ptr!(game; 0x770);
    *(ram_ptr!(game, u16; 0xb7) as *mut u8).offset(0) = *ram_ptr!(game; 0x76f);
    *(ram_ptr!(game, u16; 0xb3) as *mut u8).offset(0) = *ram_ptr!(game; 0x76e);
    *(ram_ptr!(game, u16; 0xb9) as *mut u8).offset(0) = *ram_ptr!(game; 0x76d);
    *(ram_ptr!(game, u16; 0xb5) as *mut u8).offset(0) = *ram_ptr!(game; 0x76c);
    *(ram_ptr!(game, u16; 0xb1) as *mut u8).offset(0) = *ram_ptr!(game; 0x76b);
    *ram_ptr!(game; 0x52) = *ram_ptr!(game; 0x76a);
    *ram_ptr!(game; 0x5e) = *ram_ptr!(game; 0x769);
    *ram_ptr!(game; 0x5d) = *ram_ptr!(game; 0x768);
    *ram_ptr!(game; 0x5a) = *ram_ptr!(game; 0x767);
    *ram_ptr!(game; 0x59) = *ram_ptr!(game; 0x766);
    *ram_ptr!(game; 0x58) = *ram_ptr!(game; 0x765);
    *ram_ptr!(game; 0x57) = *ram_ptr!(game; 0x774);
    *ram_ptr!(game; 0x71) = *ram_ptr!(game; 0x775);
}
pub unsafe fn LoadPauseMenuTilesAndClearBG2(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_828E8B: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb68000, 0x4000);
    SetupDmaTransfer(&raw const unk_828E8B as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x20u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_828EAB: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb6c000, 0x2000);
    SetupDmaTransfer(&raw const unk_828EAB as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x40u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_828ECB: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ab200, 0x2000);
    SetupDmaTransfer(&raw const unk_828ECB as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn LoadPauseScreenBaseTilemaps(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0x38u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_828EF0: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb6e000, 0x800);
    SetupDmaTransfer(&raw const unk_828EF0 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(WMADDL as u16, 0u8);
        byte(WMADDM as u16, 0x34u8); byte(WMADDH as u16, 0x7eu8);
    }
    static unk_828F10: StartDmaCopy = StartDmaCopy::wram_bytes(1, 0xb6e400, 0x400);
    SetupDmaTransfer(&raw const unk_828F10 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(WMADDL as u16, 0u8);
        byte(WMADDM as u16, 0x38u8); byte(WMADDH as u16, 0x7eu8);
    }
    static unk_828F30: StartDmaCopy = StartDmaCopy::wram_bytes(1, 0xb6e800, 0x800);
    SetupDmaTransfer(&raw const unk_828F30 as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    let mut v0: u16 = addr_kDummySamusWireframeTilemap as u16;
    let mut v1: u16 = 236u16;
    for _ in 0i32..17i32 {
        for _ in 0i32..8i32 {
            let fresh92 = v1;
            v1 = v1.wrapping_add(1);
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[fresh92 as usize] =
                *(RomPtr_82(v0, game) as *mut u16);
            v0 = (v0).wrapping_add(2u16);
        }
        v1 = (v1).wrapping_add(8u16);
    }
    WriteSamusWireframeTilemap(game);
    LoadEqupmentScreenReserveHealthTilemap(game);
}
pub unsafe fn LoadEqupmentScreenReserveHealthTilemap(game: &mut Game) {
    if ram_word!(game, samus_max_reserve_health) != 0 {
        let r42: i32 = ram_word!(game, samus_reserve_health) as i32 / 100i32;
        let rest: i32 = ram_word!(game, samus_reserve_health) as i32 % 100i32;
        let div10: i32 = rest / 10i32;
        let mod10: i32 = rest % 10i32;
        (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[394i32 as usize] =
            (mod10 + 2052i32) as u16;
        (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[393i32 as usize] =
            (div10 + 2052i32) as u16;
        (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[392i32 as usize] =
            (r42 + 2052i32) as u16;
    }
}
pub unsafe fn BackupGameplayPalettesAndLoadForPause(game: &mut Game) {
    let mut v1: u16 = 0u16;
    for _ in (0i16..512i16).step_by(2) {
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
            [(v1 as i32 + 384i32) as usize] = ram_word!(game, palette_buffer[v1]);
        v1 = v1.wrapping_add(1);
    }
    let mut v3: u16 = 0u16;
    for _ in (0i16..512i16).step_by(2) {
        set_ram_word!(
            game.palette_buffer[(v3 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0xb6f000, (v3 as i32 >> 1i32) as isize)
        );
        v3 = (v3).wrapping_add(2u16);
    }
}
pub unsafe fn ContinueInitPauseMenu(game: &mut Game) {
    SetupPpuForPauseMenu(game);
    ResetPauseMenuAnimations(game);
    LoadEquipmentScreenEquipmentTilemaps(game);
    set_pause_label_palettes(0, game);
    UpdatePauseMenuLRStartVramTilemap(game);
    DetermineMapScrollLimits(game);
    SetupMapScrollingForPauseMenu(0x80u16, game);
}
pub unsafe fn SetupMapScrollingForFileSelectMap(game: &mut Game) {
    ResetPauseMenuAnimations(game);
    DetermineMapScrollLimits(game);
    set_ram_word!(
        game.reg_BG1HOFS = (ram_word!(game, map_min_x_scroll) as i32
            + (((ram_word!(game, map_max_x_scroll)).wrapping_sub(ram_word!(game, map_min_x_scroll)))
                as i32
                >> 1i32)
            - 128i32) as u16
    );
    let mut r18: u16 = (8i32
        * (ram_word!(game, room_x_coordinate_on_map) as i32
            + ((ram_word!(game, samus_x_pos) as i32 & 0xff00i32) >> 8i32))
        - ram_word!(game, reg_BG1HOFS) as i32) as u16;
    let v0: i16 = (224i16).wrapping_sub((r18) as i16);
    if (224i16).wrapping_sub((r18) as i16) >= 0i16 {
        r18 = (32u16).wrapping_sub(r18);
        if (r18 as i32 & 0x8000i32) == 0i32 {
            update_ram_word!(game.reg_BG1HOFS, wrapping_sub(r18));
        }
    } else {
        r18 = (224u16).wrapping_sub(r18);
        update_ram_word!(game.reg_BG1HOFS, wrapping_sub((v0) as u16));
    }
    r18 = (ram_word!(game, map_min_y_scroll) as i32
        + (((ram_word!(game, map_max_y_scroll)).wrapping_sub(ram_word!(game, map_min_y_scroll)))
            as i32
            >> 1i32)
        + 16i32) as u16;
    set_ram_word!(game.reg_BG1VOFS = -(112i32 - r18 as i32 & 0xfff8i32) as u16);
    r18 = (8i32
        * (ram_word!(game, room_y_coordinate_on_map) as i32
            + *(ram_ptr!(game, u16; 0xafa) as *mut u8).offset(1isize) as i32
            + 1i32)
        + (112i32 - r18 as i32 & 0xfff8i32)) as u16;
    let v1: i16 = (64i16).wrapping_sub((r18) as i16);
    if (64i16).wrapping_sub((r18) as i16) >= 0i16 {
        r18 = (64u16).wrapping_sub(r18);
        update_ram_word!(game.reg_BG1VOFS, wrapping_sub((v1) as u16));
        if ram_word!(game, reg_BG1VOFS) as i32 + 40i32 & 0x8000i32 != 0 {
            set_ram_word!(game.reg_BG1VOFS = -40i32 as u16);
        }
    }
}
pub unsafe fn GameState_14_Paused_Async(game: &mut Game) -> CoroutineRet {
    DrawPauseMenuDuringFadeIn(game);
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15u8 {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_15_Paused_Async(game: &mut Game) -> CoroutineRet {
    ReleaseButtonsFilter(3u16, game);
    MainPauseRoutine(game);
    HandleHudTilemap(game);
    HandlePauseScreenPaletteAnimation(game);
    kCoroutineNone as CoroutineRet
}
static kPauseMenuFuncs: [Option<Func_V>; 8] = callback_table! { unsafe Func_V;
    PauseMenu_0_MapScreen, PauseMenu_1_EquipmentScreen, PauseMenu_2,
    PauseMenu_3_MapToEquipment_Load, PauseMenu_4_MapToEquipment_FadeIn, PauseMenu_5,
    PauseMenu_6_EquipmentToMap_Load, PauseMenu_7_EquipmentToMap_FadeIn,
};
pub unsafe fn MainPauseRoutine(game: &mut Game) {
    kPauseMenuFuncs[ram_word!(game, menu_index) as usize].expect("non-null function pointer")(game);
}
pub unsafe fn PauseMenu_0_MapScreen(game: &mut Game) {
    HandlePauseScreenLR(game);
    HandlePauseScreenStart(game);
    HandleMapScrollArrows(game);
    MapScrolling(game);
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    DisplayMapElevatorDestinations(game);
    set_ram_word!(game.pause_screen_mode = 0u16);
}
pub unsafe fn PauseMenu_1_EquipmentScreen(game: &mut Game) {
    set_ram_words! { game, 0u16; reg_BG1HOFS, reg_BG1VOFS }
    EquipmentScreenMain(game);
    HandlePauseScreenLR(game);
    HandlePauseScreenStart(game);
    set_ram_word!(game.pause_screen_mode = 1u16);
}
pub unsafe fn PauseMenu_2(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    HandlePauseMenuLRPressHighlight(game);
    set_ram_word!(game.pause_screen_mode = 0u16);
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn PauseMenu_5(game: &mut Game) {
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    HandlePauseMenuLRPressHighlight(game);
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn PauseMenu_3_MapToEquipment_Load(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    EquipmentScreenSetupReserveMode(game);
    EquipmentScreenTransferBG1Tilemap(game);
    set_ram_word!(game.pause_screen_mode = 1u16);
    SetPauseScreenButtonLabelPalettes(game);
    set_ram_word!(game.pausemenu_lr_animation_frame = 0u16);
    set_ram_word!(
        game.pausemenu_lr_animation_timer =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *mut u8 as *mut u16)
    );
    set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn PauseMenu_6_EquipmentToMap_Load(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    LoadPauseMenuMapTilemapAndAreaLabel(game);
    SetPauseScreenButtonLabelPalettes(game);
    set_ram_word!(game.pausemenu_lr_animation_frame = 0u16);
    set_ram_word!(
        game.pausemenu_lr_animation_timer =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *mut u8 as *mut u16)
    );
    set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
    set_ram_word!(game.pause_screen_mode = 0u16);
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn PauseMenu_7_EquipmentToMap_FadeIn(game: &mut Game) {
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    DisplayMapElevatorDestinations(game);
    set_ram_word!(game.pause_screen_mode = 0u16);
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15u8 {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        let mut v0: u16 = ram_word!(game, pausemenu_button_label_mode);
        if ram_word!(game, pausemenu_button_label_mode) != 0 {
            v0 = 1u16;
        }
        set_ram_word!(game.menu_index = v0);
    }
}
pub unsafe fn PauseMenu_4_MapToEquipment_FadeIn(game: &mut Game) {
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    set_ram_word!(game.pause_screen_mode = 1u16);
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15u8 {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        let mut v0: u16 = ram_word!(game, pausemenu_button_label_mode);
        if ram_word!(game, pausemenu_button_label_mode) != 0 {
            v0 = 1u16;
        }
        set_ram_word!(game.menu_index = v0);
    }
}
static kMapScrollingFuncs: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    MapScrolling_0_None, MapScrolling_1_Left, MapScrolling_2_Right, MapScrolling_3_Up,
    MapScrolling_4_Down,
};
pub unsafe fn MapScrolling(game: &mut Game) {
    kMapScrollingFuncs[ram_word!(game, map_scrolling_direction) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn MapScrolling_0_None(game: &mut Game) {
    set_ram_word!(game.map_scrolling_gear_switch_timer = 4u16);
}
pub unsafe fn MapScrolling_GetSpeedIndex(game: &mut Game) -> u16 {
    let result: u16 = ram_word!(game, map_scrolling_speed_index);
    if ram_word!(game, map_scrolling_gear_switch_timer) == 0 {
        return (ram_word!(game, map_scrolling_speed_index)).wrapping_add(32u16);
    }
    result
}
static kMapScrollingSpeedTable: [u16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub unsafe fn MapScrolling_1_Left(game: &mut Game) {
    update_ram_word!(
        game.reg_BG1HOFS,
        wrapping_sub(
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as i32 >> 1i32) as usize]
        )
    );
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_Common(game: &mut Game) {
    update_ram_word!(game.map_scrolling_speed_index, wrapping_add(1));
    if (update_ram_word!(game.map_scrolling_speed_index, wrapping_add(1), reread) as i32 & 0xfi32)
        == 0i32
    {
        QueueSfx1_Max6(0x36u16, game);
        set_ram_words! { game, 0u16; map_scrolling_direction, map_scrolling_speed_index }
        if ram_word!(game, map_scrolling_gear_switch_timer) != 0 {
            update_ram_word!(game.map_scrolling_gear_switch_timer, wrapping_sub(1));
        }
    }
}
pub unsafe fn MapScrolling_2_Right(game: &mut Game) {
    update_ram_word!(
        game.reg_BG1HOFS,
        wrapping_add(
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as i32 >> 1i32) as usize]
        )
    );
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_3_Up(game: &mut Game) {
    update_ram_word!(
        game.reg_BG1VOFS,
        wrapping_sub(
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as i32 >> 1i32) as usize]
        )
    );
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_4_Down(game: &mut Game) {
    update_ram_word!(
        game.reg_BG1VOFS,
        wrapping_add(
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as i32 >> 1i32) as usize]
        )
    );
    MapScrolling_Common(game);
}
pub unsafe fn GameState_16_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    HighlightPauseScreenButton(game);
    DrawPauseMenuDuringFadeout(game);
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DrawPauseMenuDuringFadeout(game: &mut Game) {
    if ram_word!(game, pause_screen_mode) == 1u16 {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
        HandlePauseMenuLRPressHighlight(game);
    } else {
        DisplayMapElevatorDestinations(game);
        DrawMapIcons(game);
        MapScreenDrawSamusPositionIndicator(game);
    };
}
pub unsafe fn GameState_17_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x612);
    let run_unpause_hook: bool;
    match *_state_variable_ as i32 {
        0 => {
            ClearSamusBeamTiles(game);
            ContinueInitGameplayResume(game);
            ResumeGameplay(game);
            RestoreSomeGfxStateForPause(game);
            RestoreBG2TilemapFromPauseScreen(game);
            set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
            run_unpause_hook = true;
        }
        1 => {
            run_unpause_hook = true;
        }
        _ => {
            run_unpause_hook = false;
        }
    }
    if run_unpause_hook {
        let mut _coret_: u8 = CallUnpauseHook_Async(Load24(ram_ptr!(game, LongPtr; 0x604)), game);
        if _coret_ != 0 {
            *_state_variable_ = 1u8;
            return _coret_;
        }
        EnableHdmaObjects(game);
        EnableAnimtiles(game);
        QueueSamusMovementSfx(game);
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
pub unsafe fn GameState_18_Unpausing(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15u8 {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.game_state = kGameState_8_MainGameplay as u16);
    }
    kCoroutineNone as CoroutineRet
}
#[derive(Clone, Copy)]
enum MapScreen {
    Pause,
    FileSelect,
}

// Render both map screens with the original mask/ROM/RAM access order.
unsafe fn render_map_tilemap(screen: MapScreen, game: &mut Game) {
    let mut v0: u16 = ram_word!(game, area_index);
    if ram_word!(game, area_index) as i32 - 7i32 & 0x8000i32 == 0 {
        v0 = 0u16;
    }
    let r0: *const u16 = RomPtr(
        Load24(
            (game.g_rom.offset(crate::rtl::lorom_offset(0x82964a)) as *mut LongPtr)
                .offset(v0 as isize),
        ),
        game,
    ) as *const u16;
    let r3: *mut u16 = match screen {
        MapScreen::Pause => ram_ptr!(game, Ram4000; 0x4000) as *mut u16,
        MapScreen::FileSelect => ram_ptr!(game, Ram3000; 0x3000) as *mut u16,
    };
    let mut r6: *const u16 = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x829717, v0 as isize),
        game,
    ) as *const u16;
    let mut r9: *const u16 = ram_ptr!(game; 0x7f7) as *const u16;
    if *ram_ptr!(game; 0xd908, ram_word!(game, area_index) as isize) != 0 {
        let mut mapped = u16::swap_bytes(*r6);
        r6 = r6.offset(1);
        let mut explored = u16::swap_bytes(*r9);
        r9 = r9.offset(1);
        for tile in 0..2048isize {
            let mut value = *r0.offset(tile);
            let was_explored = explored & 0x8000 != 0;
            explored = explored.wrapping_mul(2);
            if was_explored {
                value &= 0xfbff;
                mapped = mapped.wrapping_mul(2);
            } else {
                let was_mapped = mapped & 0x8000 != 0;
                mapped = mapped.wrapping_mul(2);
                if !was_mapped {
                    value = 31;
                }
            }
            *r3.offset(tile) = value;
            if tile & 15 == 15 {
                // Keep the original prefetch after the final tile too.
                mapped = u16::swap_bytes(*r6);
                r6 = r6.offset(1);
                explored = u16::swap_bytes(*r9);
                r9 = r9.offset(1);
            }
        }
    } else {
        let mut tile = 0isize;
        for byte in 0..256isize {
            for _ in 0..8 {
                let explored = ram_ptr!(game; 0x7f7, byte);
                let previous = *explored;
                *explored = ((*explored as i32) << 1) as u8;
                if previous & 0x80 == 0 {
                    *r3.offset(tile) = match screen {
                        MapScreen::Pause => 31,
                        MapScreen::FileSelect => 15,
                    };
                } else {
                    *explored = (*explored).wrapping_add(1);
                    *r3.offset(tile) = *r0.offset(tile) & 0xfbff;
                }
                tile += 1;
            }
        }
    }
}

#[derive(Clone, Copy)]
enum MapEdge {
    Left,
    Right,
    Top,
    Bottom,
}

unsafe fn find_map_edge(mut source: *const u8, edge: MapEdge) -> u16 {
    match edge {
        MapEdge::Left | MapEdge::Right => {
            let right = matches!(edge, MapEdge::Right);
            let mut column: u16 = if right { 63 } else { 0 };
            loop {
                let mask = k0x80Shr[(column & 7) as usize];
                for offset in (0..128).step_by(4) {
                    if mask & *source.offset(offset) != 0 {
                        return column;
                    }
                }
                column = if right {
                    column.wrapping_sub(1)
                } else {
                    column.wrapping_add(1)
                };
                if if right {
                    column & 0x8000 != 0
                } else {
                    (column as i16).wrapping_sub(64) >= 0
                } {
                    return if right { 28 } else { 26 };
                }
                if column & 7 == if right { 7 } else { 0 } {
                    source = source.offset(if right { -1 } else { 1 });
                }
                if column == if right { 31 } else { 32 } {
                    source = source.offset(if right { -124 } else { 123 });
                }
            }
        }
        MapEdge::Top | MapEdge::Bottom => {
            let bottom = matches!(edge, MapEdge::Bottom);
            let mut second = source.offset(128);
            let mut row: u16 = if bottom { 31 } else { 0 };
            let mut offset: u16 = 0;
            while *source.offset(offset as isize) == 0 && *second.offset(offset as isize) == 0 {
                offset = offset.wrapping_add(1);
                if (offset as i16).wrapping_sub(4) >= 0 {
                    offset = 0;
                    source = source.offset(if bottom { -4 } else { 4 });
                    second = second.offset(if bottom { -4 } else { 4 });
                    row = if bottom {
                        row.wrapping_sub(1)
                    } else {
                        row.wrapping_add(1)
                    };
                    if if bottom {
                        row == 0
                    } else {
                        (row as i16).wrapping_sub(31) >= 0
                    } {
                        return if bottom { 11 } else { 1 };
                    }
                }
            }
            row
        }
    }
}

pub unsafe fn LoadPauseMenuMapTilemapAndAreaLabel(game: &mut Game) {
    set_ram_word!(game.reg_BG1HOFS = ram_word!(game, reg_BG4HOFS));
    set_ram_word!(game.reg_BG1VOFS = ram_word!(game, reg_BG4VOFS));
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0x30u8);
        byte(VMAIN as u16, 0x80u8);
    }
    LoadPauseMenuMapTilemap(game);
    static unk_8293E9: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x1000);
    SetupDmaTransfer(&raw const unk_8293E9 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0xaau8);
        byte(VMADDH as u16, 0x38u8); byte(VMAIN as u16, 0x80u8);
        byte(DMAP1 as u16, 1u8); byte(BBAD1 as u16, 0x18u8);
        byte(DAS1L as u16, 0x18u8); byte(DAS1H as u16, 0u8);
    }
    let mut v0: u16 = ram_word!(game, area_index);
    if ram_word!(game, area_index) as i32 - 7i32 & 0x8000i32 == 0 {
        v0 = 0u16;
    }
    crate::rtl::register_writes! { game;
        word(A1T1L as u16,
        crate::rtl::rom_table_word!(game, 0x82965f, v0 as isize),); byte(A1B1 as u16, 0x82u8);
        byte(MDMAEN as u16, 2u8);
    }
}
pub unsafe fn LoadPauseMenuMapTilemap(game: &mut Game) {
    render_map_tilemap(MapScreen::Pause, game);
}
pub unsafe fn DrawRoomSelectMap(game: &mut Game) {
    *ram_ptr!(game; 0x5d) = 51u8;
    *ram_ptr!(game; 0x69) = 19u8;
    set_ram_word!(game.reg_BG1VOFS = -40i32 as u16);
    render_map_tilemap(MapScreen::FileSelect, game);
    let v16: u16 = ram_word!(game, vram_write_queue_tail);
    let v17: *mut VramWriteEntry =
        gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v17).size = 4096u16;
    (*v17).src.addr = (ram_ptr!(game, Ram3000; 0x3000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    (*v17).src.bank = 126u8;
    (*v17).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v16).wrapping_add(7u16));
}
pub unsafe fn DrawRoomSelectMapAreaLabel(dst: *mut u16, game: &mut Game) {
    let v2: *const u16 = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x82965f, ram_word!(game, area_index) as isize),
        game,
    ) as *const u16;
    let mut i: i32 = 0i32;
    while i < 12i32 {
        *dst.offset(i as isize) = (*v2.offset(i as isize)) & (0xefffu16);
        i += 1;
    }
}
pub unsafe fn SetupMapScrollingForPauseMenu(a: u16, game: &mut Game) {
    set_ram_word!(
        game.reg_BG1HOFS = (ram_word!(game, map_min_x_scroll) as i32
            + (((ram_word!(game, map_max_x_scroll)).wrapping_sub(ram_word!(game, map_min_x_scroll)))
                as i32
                >> 1i32)
            - 128i32) as u16
    );
    let mut r18: u16 = (8i32
        * (ram_word!(game, room_x_coordinate_on_map) as i32
            + ((ram_word!(game, samus_x_pos) as i32 & 0xff00i32) >> 8i32))
        - ram_word!(game, reg_BG1HOFS) as i32) as u16;
    let mut v1: i16 = (224i16).wrapping_sub((r18) as i16);
    if (224i16).wrapping_sub((r18) as i16) >= 0i16 {
        r18 = (32u16).wrapping_sub(r18);
        if (r18 as i32 & 0x8000i32) == 0i32 {
            update_ram_word!(game.reg_BG1HOFS, wrapping_sub(r18));
        }
    } else {
        r18 = (224u16).wrapping_sub(r18);
        update_ram_word!(game.reg_BG1HOFS, wrapping_sub((v1) as u16));
    }
    r18 = (ram_word!(game, map_min_y_scroll) as i32
        + (((ram_word!(game, map_max_y_scroll)).wrapping_sub(ram_word!(game, map_min_y_scroll)))
            as i32
            >> 1i32)
        + 16i32) as u16;
    set_ram_word!(game.reg_BG1VOFS = -(a as i32 - r18 as i32 & 0xfff8i32) as u16);
    r18 = (8i32
        * (ram_word!(game, room_y_coordinate_on_map) as i32
            + *(ram_ptr!(game, u16; 0xafa) as *mut u8).offset(1isize) as i32
            + 1i32)
        + (a as i32 - r18 as i32 & 0xfff8i32)) as u16;
    let mut v2: i16 = (64i16).wrapping_sub((r18) as i16);
    if v2 >= 0i16 {
        update_ram_word!(game.reg_BG1VOFS, wrapping_sub((v2) as u16));
        if ram_word!(game, reg_BG1VOFS) as i32 + 40i32 & 0x8000i32 != 0 {
            set_ram_word!(game.reg_BG1VOFS = -40i32 as u16);
        }
    }
}
static k0x80Shr: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];

unsafe fn DetermineLeftmostMapColumn(r0: *const u8) -> u16 {
    find_map_edge(r0, MapEdge::Left)
}
unsafe fn DetermineRightmostMapColumn(r0: *const u8) -> u16 {
    find_map_edge(r0, MapEdge::Right)
}
unsafe fn DetermineTopmostMapColumn(r0: *const u8) -> u16 {
    find_map_edge(r0, MapEdge::Top)
}
unsafe fn DetermineBottommostMapColumn(r0: *const u8) -> u16 {
    find_map_edge(r0, MapEdge::Bottom)
}
pub unsafe fn DetermineMapScrollLimits(game: &mut Game) {
    let mut r6: *const u8 = null::<u8>();
    if ram_word!(game, has_area_map) != 0 {
        r6 = RomPtr_82(
            *(RomPtr_82(
                (addr_kPauseMenuMapData as i32 + 2i32 * ram_word!(game, area_index) as i32) as u16,
                game,
            ) as *mut u16),
            game,
        );
    } else {
        r6 = ram_ptr!(game; 0x7f7);
    }
    set_ram_word!(game.map_min_x_scroll = (DetermineLeftmostMapColumn(r6)).wrapping_mul(8u16));
    if ram_word!(game, area_index) == 4u16 {
        update_ram_word!(game.map_min_x_scroll, wrapping_sub(24u16));
    }
    set_ram_word!(
        game.map_max_x_scroll = (DetermineRightmostMapColumn(r6.offset(131))).wrapping_mul(8u16)
    );
    set_ram_word!(game.map_min_y_scroll = (DetermineTopmostMapColumn(r6)).wrapping_mul(8u16));
    set_ram_word!(
        game.map_max_y_scroll = (DetermineBottommostMapColumn(r6.offset(124))).wrapping_mul(8u16)
    );
}
pub unsafe fn SetupPpuForPauseMenu(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(OBSEL as u16, 1, 0x52, false); mirror(BGMODE as u16, 9, 0x55, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, 0x31, 0x58, true); mirror(BG2SC as u16, 0x38, 0x59, true);
        mirror(BG3SC as u16, 0x58, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        mirror(TM as u16, 0x17, 0x69, false); mirror(MOSAIC as u16, 0, 0x57, false);
    }
    let ref mut fresh24 = *ram_ptr!(game; 0x74, 0);
    *fresh24 = (*fresh24) & (0xe0u8);
    let ref mut fresh25 = *ram_ptr!(game; 0x74, 1);
    *fresh25 = (*fresh25) & (0xe0u8);
    let ref mut fresh26 = *ram_ptr!(game; 0x74, 2);
    *fresh26 = (*fresh26) & (0xe0u8);
    *ram_ptr!(game; 0x71) = 0u8;
}
pub unsafe fn ResetPauseMenuAnimations(game: &mut Game) {
    set_ram_words! { game, 0u16;
        pausemenu_reserve_tank_delay_ctr, reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
        reg_BG3VOFS, pausemenu_button_label_mode, pausemenu_lr_animation_frame, UNUSED_word_7E0745,
        samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
        samus_position_indicator_animation_loop_counter,
    }
    set_ram_word!(
        game.pausemenu_lr_animation_timer =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *mut u8 as *mut u16)
    );
    set_ram_word!(game.pausemenu_palette_animation_timer = 1u16);
    set_ram_word!(game.pausemenu_palette_animation_frame = 0u16);
}
unsafe fn ChangePaletteValues(target: *mut u16, new_color: u16, n: size_t) {
    let mut i: size_t = 0 as size_t;
    while i != n {
        *target.offset(i as isize) =
            (*target.offset(i as isize) as i32 & !(0x1c00i32) | new_color as i32) as u16;
        i = i.wrapping_add(1);
    }
}
pub unsafe fn LoadEquipmentScreenEquipmentTilemaps(game: &mut Game) {
    if ram_word!(game, samus_max_reserve_health) != 0 {
        memcpy(
            ram_ptr!(game, c_void; crate::rtl::rom_table_word!(game, 0x82c068, 0) as i32 as isize),
            RomPtr_82(crate::rtl::rom_table_word!(game, 0x82c088, 0), game) as *const c_void,
            14 as size_t,
        );
        memcpy(
            ram_ptr!(game, c_void; crate::rtl::rom_table_word!(game, 0x82c068, 1) as i32 as isize),
            RomPtr_82(crate::rtl::rom_table_word!(game, 0x82c088, 1), game) as *const c_void,
            14 as size_t,
        );
    }
    let mut p: *const u16 = RomPtr_82(addr_kEquipmentTilemapOffs_Weapons as u16, game) as *mut u16;
    if ram_word!(game, hyper_beam_flag) != 0 {
        let mut i: i32 = 0i32;
        while i < 5i32 {
            memcpy(
                ram_ptr!(game, c_void; *p.offset(i as isize) as i32 as isize),
                RomPtr_82(
                    crate::rtl::rom_table_word!(game, 0x82c0a8, i as isize),
                    game,
                ) as *const c_void,
                10 as size_t,
            );
            i += 1;
        }
    } else {
        let mut i_0: i32 = 0i32;
        while i_0 < 6i32 {
            let target: *mut u16 = ram_ptr!(game, u16; *p.offset(i_0 as isize) as i32 as isize);
            if (ram_word!(game, collected_beams) as i32
                & crate::rtl::rom_table_word!(game, 0x82c04c, i_0 as isize) as i32)
                != 0i32
            {
                memcpy(
                    target as *mut c_void,
                    RomPtr_82(
                        crate::rtl::rom_table_word!(game, 0x82c08c, i_0 as isize),
                        game,
                    ) as *const c_void,
                    10 as size_t,
                );
                if (ram_word!(game, equipped_beams) as i32
                    & crate::rtl::rom_table_word!(game, 0x82c04c, i_0 as isize) as i32)
                    == 0i32
                {
                    ChangePaletteValues(target, 3072u16, 5 as size_t);
                }
            } else {
                memcpy(
                    target as *mut c_void,
                    RomPtr_82(addr_kEquipmentScreenTilemap_Blank as u16, game) as *const c_void,
                    10 as size_t,
                );
            }
            i_0 += 1;
        }
    }
    p = RomPtr_82(addr_kEquipmentTilemapOffs_Suits as u16, game) as *mut u16;
    let mut i_1: i32 = 0i32;
    while i_1 < 6i32 {
        let target_0: *mut u16 = ram_ptr!(game, u16; *p.offset(i_1 as isize) as i32 as isize);
        if (ram_word!(game, collected_items) as i32
            & crate::rtl::rom_table_word!(game, 0x82c056, i_1 as isize) as i32)
            != 0i32
        {
            memcpy(
                target_0 as *mut c_void,
                RomPtr_82(
                    crate::rtl::rom_table_word!(game, 0x82c096, i_1 as isize),
                    game,
                ) as *const c_void,
                18 as size_t,
            );
            if (ram_word!(game, equipped_items) as i32
                & crate::rtl::rom_table_word!(game, 0x82c056, i_1 as isize) as i32)
                == 0i32
            {
                ChangePaletteValues(target_0, 3072u16, 9 as size_t);
            }
        } else {
            memcpy(
                target_0 as *mut c_void,
                RomPtr_82(addr_kEquipmentScreenTilemap_Blank as u16, game) as *const c_void,
                18 as size_t,
            );
        }
        i_1 += 1;
    }
    p = RomPtr_82(addr_kEquipmentTilemapOffs_Boots as u16, game) as *mut u16;
    let mut i_2: i32 = 0i32;
    while i_2 < 3i32 {
        let target_1: *mut u16 = ram_ptr!(game, u16; *p.offset(i_2 as isize) as i32 as isize);
        if (ram_word!(game, collected_items) as i32
            & crate::rtl::rom_table_word!(game, 0x82c062, i_2 as isize) as i32)
            != 0i32
        {
            memcpy(
                target_1 as *mut c_void,
                RomPtr_82(
                    crate::rtl::rom_table_word!(game, 0x82c0a2, i_2 as isize),
                    game,
                ) as *const c_void,
                18 as size_t,
            );
            if (ram_word!(game, equipped_items) as i32
                & crate::rtl::rom_table_word!(game, 0x82c062, i_2 as isize) as i32)
                == 0i32
            {
                ChangePaletteValues(target_1, 3072u16, 9 as size_t);
            }
        } else {
            memcpy(
                target_1 as *mut c_void,
                RomPtr_82(addr_kEquipmentScreenTilemap_Blank as u16, game) as *const c_void,
                18 as size_t,
            );
        }
        i_2 += 1;
    }
}
pub unsafe fn ClearSamusBeamTiles(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0x60u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_82A2D4: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ad200, 0x1000);
    SetupDmaTransfer(&raw const unk_82A2D4 as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn ContinueInitGameplayResume(game: &mut Game) {
    let mut v1: u16 = 0u16;
    for _ in (0i16..512i16).step_by(2) {
        set_ram_word!(
            game.palette_buffer[v1] = (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
                [(v1 as i32 + 384i32) as usize]
        );
        v1 = v1.wrapping_add(1);
    }
    SetupPpuForGameplayResume(game);
    CalculateBgScrolls_Unpause(game);
    UpdateBeamTilesAndPalette_Unpause(game);
    ClearPauseMenuData(game);
    CallSomeSamusCode(0xcu16, game);
}
pub unsafe fn SetupPpuForGameplayResume(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(OBSEL as u16, 3, 0x52, false); mirror(BGMODE as u16, 9, 0x55, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, 0x51, 0x58, true); mirror(BG2SC as u16, 0x49, 0x59, true);
        mirror(BG3SC as u16, 0x5a, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
    }
}
pub unsafe fn CalculateBgScrolls_Unpause(game: &mut Game) {
    set_ram_word!(
        game.reg_BG1HOFS =
            (ram_word!(game, bg1_x_offset)).wrapping_add(ram_word!(game, layer1_x_pos))
    );
    set_ram_word!(
        game.reg_BG1VOFS =
            (ram_word!(game, bg1_y_offset)).wrapping_add(ram_word!(game, layer1_y_pos))
    );
    set_ram_word!(
        game.reg_BG2HOFS =
            (ram_word!(game, bg2_x_scroll)).wrapping_add(ram_word!(game, layer2_x_pos))
    );
    set_ram_word!(
        game.reg_BG2VOFS =
            (ram_word!(game, bg2_y_scroll)).wrapping_add(ram_word!(game, layer2_y_pos))
    );
}
pub unsafe fn UpdateBeamTilesAndPalette_Unpause(game: &mut Game) {
    UpdateBeamTilesAndPalette(game);
}
pub unsafe fn ClearPauseMenuData(game: &mut Game) {
    set_ram_words! { game, 0u16;
        menu_index, pausemenu_start_lr_pressed_highlight_timer, pausemenu_lr_animation_timer,
        pausemenu_item_selector_animation_timer, pausemenu_reserve_tank_animation_timer,
        UNUSED_word_7E0731, pausemenu_uparrow_animation_timer, pausemenu_downarrow_animation_timer,
        pausemenu_rightarrow_animation_timer, pausemenu_leftarrow_animation_timer,
        UNUSED_word_7E073D, pausemenu_lr_animation_frame, pausemenu_item_selector_animation_frame,
        pausemenu_reserve_tank_animation_frame, UNUSED_word_7E0745,
        pausemenu_uparrow_animation_frame, pausemenu_downarrow_animation_frame,
        pausemenu_rightarrow_animation_frame, pausemenu_leftarrow_animation_frame,
        pausemenu_shoulder_button_highlight, pausemenu_button_label_mode,
        pausemenu_equipment_category_item, pausemenu_reserve_tank_delay_ctr, UNUSED_word_7E0759,
        UNUSED_word_7E075B, UNUSED_word_7E075D, UNUSED_word_7E075F, UNUSED_word_7E0761,
    }
}
pub fn sub_82A425() {}
pub unsafe fn sub_82A42A(game: &mut Game) {
    let start_spin_jump: bool = if ram_word!(game, samus_pose) as i32
        == kPose_81_FaceR_Screwattack as i32
        || ram_word!(game, samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
    {
        if (ram_word!(game, equipped_items) as i32 & 8i32) != 0i32 {
            return;
        }
        true
    } else {
        ((ram_word!(game, samus_pose) as i32 == kPose_1B_FaceR_SpaceJump as i32
            || ram_word!(game, samus_pose) as i32 == kPose_1C_FaceL_SpaceJump as i32)
            && (ram_word!(game, equipped_items) as i32 & 0x20i32) == 0i32)
    };
    if start_spin_jump {
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_1A_FaceL_SpinJump as u16);
        } else {
            set_ram_word!(game.samus_pose = kPose_19_FaceR_SpinJump as u16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
    };
}
pub unsafe fn sub_82A47B(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 4i32) == 0i32 {
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_41_FaceL_Morphball_Ground as u16);
        } else {
            set_ram_word!(game.samus_pose = kPose_1D_FaceR_Morphball_Ground as u16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
    }
}
pub unsafe fn sub_82A4A9(game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 2i32) != 0i32 {
        if *ram_ptr!(game; 0xa1e) == 4u8 {
            set_ram_word!(game.samus_pose = kPose_7A_FaceL_Springball_Ground as u16);
        } else {
            set_ram_word!(game.samus_pose = kPose_79_FaceR_Springball_Ground as u16);
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
    }
}
pub unsafe fn HandlePauseScreenLR(game: &mut Game) {
    HandlePauseScreenLrInput(game);
    DrawLrHighlight(game);
}
pub unsafe fn HandlePauseScreenLrInput(game: &mut Game) {
    let play_switch_sound: bool = if (ram_word!(game, newly_held_down_timed_held_input) as i32
        & kButton_L as i32)
        != 0i32
    {
        if ram_word!(game, pausemenu_button_label_mode) == 0 {
            return;
        }
        set_ram_words! { game, 5u16; pausemenu_start_lr_pressed_highlight_timer, menu_index }
        set_ram_word!(game.pausemenu_shoulder_button_highlight = 1u16);
        set_ram_word!(game.pausemenu_button_label_mode = 0u16);
        SetPauseScreenButtonLabelPalettes(game);
        true
    } else {
        ((ram_word!(game, newly_held_down_timed_held_input) as i32 & kButton_R as i32) != 0i32
            && ram_word!(game, pausemenu_button_label_mode) != 2u16)
            && ({
                set_ram_word!(game.pausemenu_start_lr_pressed_highlight_timer = 5u16);
                set_ram_words! { game, 2u16;
                    menu_index, pausemenu_shoulder_button_highlight, pausemenu_button_label_mode,
                }
                SetPauseScreenButtonLabelPalettes(game);
                true
            })
    };
    if play_switch_sound {
        QueueSfx1_Max6(0x38u16, game);
    };
}
static kPauseLrButtonPressedHighlight_Spritemap: [u16; 2] = [0x28, 0x29];
static kPauseLrButtonPressedHighlight_Y: [u16; 2] = [0x18, 0xe8];
static kPauseLrButtonPressedHighlight_X: [u16; 2] = [0xd0, 0xd0];
pub unsafe fn HandlePauseMenuLRPressHighlight(game: &mut Game) {
    if ram_word!(game, pausemenu_start_lr_pressed_highlight_timer) != 0 {
        update_ram_word!(
            game.pausemenu_start_lr_pressed_highlight_timer,
            wrapping_sub(1)
        );
        if ram_word!(game, pausemenu_shoulder_button_highlight) != 0 {
            let v0: i32 = ram_word!(game, pausemenu_shoulder_button_highlight) as i32 - 1i32;
            DrawMenuSpritemap(
                kPauseLrButtonPressedHighlight_Spritemap[v0 as usize],
                kPauseLrButtonPressedHighlight_Y[v0 as usize],
                (kPauseLrButtonPressedHighlight_X[v0 as usize]).wrapping_sub(1u16),
                0u16,
                game,
            );
        }
    }
}
pub unsafe fn DrawLrHighlight(game: &mut Game) {
    DrawPauseScreenSpriteAnim(2u16, 0x18u16, 0xd0u16, game);
    DrawPauseScreenSpriteAnim(2u16, 0xe8u16, 0xd0u16, game);
}
pub unsafe fn HandlePauseScreenStart(game: &mut Game) {
    if (ram_word!(game, newly_held_down_timed_held_input) as i32 & kButton_Start as i32) != 0i32 {
        QueueSfx1_Max6(0x38u16, game);
        set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
        let v0: u16 = ram_word!(game, pausemenu_button_label_mode);
        set_ram_word!(game.pausemenu_button_label_mode = 1u16);
        SetPauseScreenButtonLabelPalettes(game);
        set_ram_word!(game.pausemenu_button_label_mode = v0);
        set_ram_word!(game.pausemenu_start_lr_pressed_highlight_timer = 11u16);
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    UpdatePauseMenuLRStartVramTilemap(game);
}
pub unsafe fn HighlightPauseScreenButton(game: &mut Game) {
    if ram_word!(game, pausemenu_start_lr_pressed_highlight_timer) != 0 {
        update_ram_word!(
            game.pausemenu_start_lr_pressed_highlight_timer,
            wrapping_sub(1)
        );
        DrawMenuSpritemap(0x2bu16, 0x90u16, 0xd0u16, 0u16, game);
    }
}
pub fn SetPauseScreenButtonLabelPalettes(game: &mut Game) {
    set_pause_label_palettes(game.pausemenu_button_label_mode(), game);
}

fn set_pause_label_palettes(mode: u16, game: &mut Game) {
    // First tile and width of each two-row label in the pause tilemap.
    const LEFT: (usize, usize) = (805, 5);
    const START: (usize, usize) = (812, 4);
    const RIGHT: (usize, usize) = (822, 5);
    const TILEMAP: usize = 0x3000;

    // Modes retain the original label order, including the unpause mode (1).
    let labels = [
        [(RIGHT, 0x0800), (START, 0x0800), (LEFT, 0x1400)],
        [(START, 0x0800), (LEFT, 0x1400), (RIGHT, 0x1400)],
        [(LEFT, 0x0800), (START, 0x0800), (RIGHT, 0x1400)],
    ][usize::from(mode)];
    for ((first_tile, width), palette) in labels {
        for row in [first_tile, first_tile + 32] {
            for tile in row..row + width {
                let address = TILEMAP + 2 * tile;
                let word = u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]]);
                let word = (word & !0x1c00) | palette;
                game.g_ram[address..address + 2].copy_from_slice(&word.to_le_bytes());
            }
        }
    }
}

pub unsafe fn UpdatePauseMenuLRStartVramTilemap(game: &mut Game) {
    let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 128u16;
    v0 = (v0).wrapping_add(2u16);
    (*gVramWriteEntry(v0 as i32, game)).size = 0x3640u16;
    v0 = (v0).wrapping_add(2u16);
    let fresh23 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh23 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = 126u8;
    (*gVramWriteEntry(v0 as i32, game)).size =
        (((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32) + 800i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
}
pub unsafe fn DrawPauseScreenSpriteAnim(a: u16, input_k: u16, input_j: u16, game: &mut Game) {
    let t: i32 = a as i32 - 1i32;
    let v8: *mut u16 = RomPtr_RAM(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0c4))
            as *mut PauseScreenSpriteAnimationData))
            .arr[t as usize],
        game,
    ) as *mut u16;
    let v5: *mut u16 = RomPtr_RAM(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0b2))
            as *mut PauseScreenSpriteAnimationData))
            .arr[t as usize],
        game,
    ) as *mut u16;
    let v3: *const u8 = RomPtr_82(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0e8))
            as *mut PauseScreenSpriteAnimationData))
            .arr[t as usize],
        game,
    );
    *v5 = (*v5).wrapping_sub(1);
    if (*v5 as i16) <= 0i16 {
        *v8 = (*v8).wrapping_add(1);
        let mut v10: u16 = *v3.offset((3i32 * *v8 as i32) as isize) as u16;
        if v10 == 255u16 {
            *v8 = 0u16;
            v10 = *v3.offset(0) as u16;
        }
        *v5 = v10;
    }
    let r3: u16 = crate::rtl::rom_table_word!(game, 0x82c0fa, 3);
    let r24: i32 = *v3.offset((3i32 * *v8 as i32 + 2i32) as isize) as i32;
    let r26: i32 = 2i32
        * *RomPtr_RAM(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0d6))
                as *mut PauseScreenSpriteAnimationData))
                .arr[t as usize],
            game,
        ) as i32;
    let v11: *const u8 = RomPtr_82(
        (r26 + crate::rtl::rom_table_word!(game, 0x82c1e4, t as isize) as i32) as u16,
        game,
    );
    DrawMenuSpritemap(
        (r24 + *(v11 as *mut u16) as i32) as u16,
        input_k,
        (input_j).wrapping_sub(1u16),
        r3,
        game,
    );
}
pub unsafe fn HandlePauseScreenPaletteAnimation(game: &mut Game) {
    let mut v2: u16 = 0;
    let mut v1: u8 = 0;
    if ram_word!(game, pausemenu_palette_animation_timer) as u8 != 0 {
        *(ram_ptr!(game, u16; 0x73b) as *mut u8).offset(0) =
            ((ram_word!(game, pausemenu_palette_animation_timer)) as u8).wrapping_sub(1u8);
        if ram_word!(game, pausemenu_palette_animation_timer) as u8 == 0 {
            let mut i: i32 = ram_word!(game, pausemenu_palette_animation_frame) as i32 + 1i32;
            loop {
                *(ram_ptr!(game, u16; 0x74f) as *mut u8).offset(0) = i as u8;
                v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *mut u8)
                    .offset((3i32 * i) as isize);
                if v1 != 0xffu8 {
                    break;
                }
                QueueSfx3_Max6(0x2au16, game);
                i = 0i32;
            }
            *(ram_ptr!(game, u16; 0x73b) as *mut u8).offset(0) = v1;
            *(&raw mut v2 as *mut u8).offset(0) = 0u8;
            *(&raw mut v2 as *mut u8).offset(1isize) =
                ram_word!(game, pausemenu_palette_animation_frame) as u8;
            let mut v3: u16 = ((v2 as i32 >> 3i32) + 30i32) as u16;
            let mut j: u16 = 30u16;
            while (j as i32 & 0x8000i32) == 0i32 {
                *(ram_ptr!(game, u16; 0xc000)).offset(((j as i32 >> 1i32) + 176i32) as isize) =
                    crate::rtl::rom_table_word!(game, 0x82a987, (v3 as i32 >> 1i32) as isize);
                v3 = (v3).wrapping_sub(2u16);
                j = (j).wrapping_sub(2u16);
            }
        }
    }
}
pub unsafe fn EquipmentScreenSetupReserveMode(game: &mut Game) {
    let mut current_block: u64;
    let mut v5: i16 = 0;
    let mut v7: i16 = 0;
    set_ram_word!(game.reg_BG4HOFS = ram_word!(game, reg_BG1HOFS));
    set_ram_word!(game.reg_BG4VOFS = ram_word!(game, reg_BG1VOFS));
    set_ram_words! { game, 0u16; reg_BG1HOFS, reg_BG1VOFS }
    if ram_word!(game, samus_max_reserve_health) != 0u16
        && ram_word!(game, reserve_health_mode) != 0u16
    {
        let mut v0: VoidP = addr_kEquipmentScreenTilemap_AUTO as i32 as VoidP;
        if ram_word!(game, reserve_health_mode) != 1u16 {
            v0 = addr_kEquipmentScreenTilemap_MANUAL as i32 as VoidP;
        }
        let table: *mut u16 = RomPtr_82(v0, game) as *mut u16;
        let mut i: i32 = 0i32;
        while i < 4i32 {
            (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[(i + 327i32) as usize] =
                ((*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[(i + 327i32) as usize]
                    as i32
                    & 0xfc00i32
                    | *table.offset(i as isize) as i32) as u16;
            i += 1;
        }
    }
    set_ram_word!(game.pausemenu_item_selector_animation_frame = 0u16);
    set_ram_word!(
        game.pausemenu_item_selector_animation_timer =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *mut u8).offset(0) as u16
    );
    set_ram_word!(game.pausemenu_reserve_tank_animation_frame = 0u16);
    set_ram_word!(
        game.pausemenu_reserve_tank_animation_timer =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c165)) as *mut u8).offset(0) as u16
    );
    if ram_word!(game, samus_max_reserve_health) != 0 {
        set_ram_word!(game.pausemenu_equipment_category_item = 0u16);
    } else {
        if ram_word!(game, hyper_beam_flag) != 0 {
            current_block = 15976848397966268834;
        } else {
            let mut v4: u16 = 0u16;
            loop {
                if ((crate::rtl::rom_table_word!(game, 0x82c04c, (v4 as i32 >> 1i32) as isize)
                    as i32
                    & ram_word!(game, collected_beams) as i32)
                    != 0i32)
                {
                    current_block = 4761528863920922185;
                    break;
                }
                v4 = (v4).wrapping_add(2u16);
                if ((v4) as i16).wrapping_sub(10i16) >= 0i16 {
                    current_block = 15976848397966268834;
                    break;
                }
            }
            if current_block == 15976848397966268834 {
            } else {
                *(&raw mut v5 as *mut u8).offset(0) =
                    ((v4 as i32 >> 1i32) as u16 as i32 >> 8i32) as u8;
                *(&raw mut v5 as *mut u8).offset(1isize) = (v4 as i32 >> 1i32) as u8;
                set_ram_word!(game.pausemenu_equipment_category_item = ((v5) as u16) | (1u16));
                current_block = 9932683223035187777;
            }
        }
        if current_block == 9932683223035187777 {
        } else {
            let mut v6: u16 = 0u16;
            loop {
                if (crate::rtl::rom_table_word!(game, 0x82c056, (v6 as i32 >> 1i32) as isize)
                    as i32
                    & ram_word!(game, collected_items) as i32)
                    != 0i32
                {
                    *(&raw mut v7 as *mut u8).offset(0) =
                        ((v6 as i32 >> 1i32) as u16 as i32 >> 8i32) as u8;
                    *(&raw mut v7 as *mut u8).offset(1isize) = (v6 as i32 >> 1i32) as u8;
                    set_ram_word!(game.pausemenu_equipment_category_item = ((v7) as u16) | (2u16));
                    current_block = 9932683223035187777;
                    break;
                } else {
                    v6 = (v6).wrapping_add(2u16);
                    if (((v6) as i16).wrapping_sub(12i16) >= 0i16) {
                        current_block = 1109700713171191020;
                        break;
                    }
                }
            }
            if current_block == 9932683223035187777 {
            } else {
                let mut v8: i16 = 0i16;
                loop {
                    v8 = (v8).wrapping_add(2i16);
                    if ((v8).wrapping_sub(6i16) >= 0i16) {
                        break;
                    }
                }
            }
        }
    }
    if ram_word!(game, samus_reserve_health) != 0 {
        EquipmentScreenGlowingArrowSolidOn(game);
        WriteSamusWireframeTilemapAndQueue(game);
    }
}
pub unsafe fn EquipmentScreenTransferBG1Tilemap(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0x30u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_82AC3B: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e3800, 0x800);
    SetupDmaTransfer(&raw const unk_82AC3B as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    set_ram_word!(game.reg_BG1VOFS = 0u16);
}
static kEquipmentScreenCategories: [Option<Func_V>; 4] = callback_table! { unsafe Func_V;
    EquipmentScreenCategory_Tanks, EquipmentScreenCategory_Weapons,
    EquipmentScreenCategory_Suit, EquipmentScreenCategory_Boots,
};
pub unsafe fn EquipmentScreenMain(game: &mut Game) {
    kEquipmentScreenCategories[ram_word!(game, pausemenu_equipment_category_item) as u8 as usize]
        .expect("non-null function pointer")(game);
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    WriteSamusWireframeTilemapAndQueue(game);
}
static kEquipmentScreenCategory_TanksFuncs: [Option<Func_V>; 2] = callback_table! { unsafe Func_V;
    EquipmentScreenCategory_Tanks_0, EquipmentScreenCategory_Tanks_1,
};
pub unsafe fn EquipmentScreenCategory_Tanks(game: &mut Game) {
    kEquipmentScreenCategory_TanksFuncs
        [*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize) as usize]
        .expect("non-null function pointer")(game);
    EquipmentScreenHandleDpad(game);
    EquipmentScreenGlowingArrow(game);
}
pub unsafe fn EquipmentScreenHandleDpad(game: &mut Game) {
    let r18: u16 = ram_word!(game, pausemenu_equipment_category_item);
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Right as i32) != 0i32 {
        if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32
            || EquipmentScreenMoveLowerOnSuitsMisc(0u16, game) == 0xffffu16
        {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0u16, r18, game);
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
        if (ram_word!(game, pausemenu_equipment_category_item) as i32 & 0xff00i32) != 0i32 {
            QueueSfx1_Max6(0x37u16, game);
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_sub(256u16));
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
        if (ram_word!(game, pausemenu_equipment_category_item) as i32 & 0xff00i32) == 256i32
            || ram_word!(game, reserve_health_mode) == 1u16
            || {
                update_ram_word!(game.pausemenu_equipment_category_item, wrapping_add(256u16));
                ram_word!(game, samus_reserve_health) == 0
            }
        {
            EquipmentScreenMoveToBeams(0u16, r18, game);
        } else {
            QueueSfx1_Max6(0x37u16, game);
        }
    }
}
static kEquipmentScreenGlowingArrowFuncs: [Option<Func_V>; 2] = callback_table! { unsafe Func_V;
    EquipmentScreenGlowingArrowAnimated, EquipmentScreenGlowingArrowSolidOn,
};
pub unsafe fn EquipmentScreenGlowingArrow(game: &mut Game) {
    if ram_word!(game, pausemenu_equipment_category_item) as u8 != 0 {
        EquipmentScreenGlowingArrowSolidOff(game);
    } else {
        kEquipmentScreenGlowingArrowFuncs
            [*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize) as usize]
            .expect("non-null function pointer")(game);
    };
}
static kEquipmentScreenGlowingArrowPalettes0: [u16; 32] = [
    0x39e, 0x77d, 0xb5c, 0xf5b, 0x133a, 0x171a, 0x1f19, 0x22f8, 0x26d7, 0x2ad6, 0x2eb6, 0x3695,
    0x3a94, 0x3e73, 0x4253, 0x4a52, 0x4a52, 0x4253, 0x3e73, 0x3a94, 0x3695, 0x2eb6, 0x2ad6, 0x26d7,
    0x22f8, 0x1f19, 0x171a, 0x133a, 0xf5b, 0xb5c, 0x77d, 0x39e,
];
static kEquipmentScreenGlowingArrowPalettes1: [u16; 32] = [
    0x156, 0x155, 0x554, 0x954, 0xd53, 0xd52, 0x1152, 0x1551, 0x1970, 0x1d70, 0x1d6f, 0x216e,
    0x256e, 0x296d, 0x296c, 0x318c, 0x318c, 0x296c, 0x296d, 0x256e, 0x216e, 0x1d6f, 0x1d70, 0x1970,
    0x1551, 0x1152, 0xd52, 0xd53, 0x954, 0x554, 0x155, 0x156,
];
pub unsafe fn EquipmentScreenGlowingArrowAnimated(game: &mut Game) {
    if ram_word!(game, reserve_health_mode) == 1u16 {
        let v0: i32 = *ram_ptr!(game; 0x5b5) as i32 & 0x1fi32;
        set_ram_word!(
            game.palette_buffer[102] = kEquipmentScreenGlowingArrowPalettes0[v0 as usize]
        );
        set_ram_word!(
            game.palette_buffer[107] = kEquipmentScreenGlowingArrowPalettes1[v0 as usize]
        );
        EquipmentScreenEnergyArrowGlow_On(game);
    } else {
        set_ram_word!(game.palette_buffer[107] = 926u16);
        set_ram_word!(game.palette_buffer[102] = 342u16);
        EquipmentScreenEnergyArrowGlow_Off(game);
    };
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOn(game: &mut Game) {
    set_ram_word!(game.palette_buffer[107] = 926u16);
    set_ram_word!(game.palette_buffer[102] = 342u16);
    EquipmentScreenEnergyArrowGlow_On(game);
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOff(game: &mut Game) {
    set_ram_word!(game.palette_buffer[107] = 926u16);
    set_ram_word!(game.palette_buffer[102] = 342u16);
    EquipmentScreenEnergyArrowGlow_Off(game);
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_On(game: &mut Game) {
    let mut i: i32 = 0i32;
    while i < 8i32 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
            [(129i32 + i * 32i32) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
            .equipment_screen_bg1_tilemap[(129i32 + i * 32i32) as usize]
            as i32
            & !(0x1c00i32)
            | 0x1800i32) as u16;
        i += 1;
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 2i32 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
            [(385i32 + i_0) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
            .equipment_screen_bg1_tilemap[(385i32 + i_0) as usize]
            as i32
            & !(0x1c00i32)
            | 0x1800i32) as u16;
        i_0 += 1;
    }
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_Off(game: &mut Game) {
    let mut i: i32 = 0i32;
    while i < 8i32 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
            [(129i32 + i * 32i32) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
            .equipment_screen_bg1_tilemap[(129i32 + i * 32i32) as usize]
            as i32
            & !(0x1c00i32)
            | 0x1c00i32) as u16;
        i += 1;
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 2i32 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
            [(385i32 + i_0) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
            .equipment_screen_bg1_tilemap[(385i32 + i_0) as usize]
            as i32
            & !(0x1c00i32)
            | 0x1c00i32) as u16;
        i_0 += 1;
    }
}
static kEquipmentScreenTilemap_MANUAL: [u16; 4] = [0x3d46, 0x3d47, 0x3d48, 0x3d49];
static kEquipmentScreenTilemap_AUTO: [u16; 4] = [0x3d56, 0x3d57, 0x3d58, 0x3d59];
pub unsafe fn EquipmentScreenCategory_Tanks_0(game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_A as i32) != 0i32
        && ram_word!(game, samus_max_reserve_health) != 0u16
    {
        QueueSfx1_Max6(0x37u16, game);
        if ram_word!(game, reserve_health_mode) == 1u16 {
            set_ram_word!(game.reserve_health_mode = 2u16);
            EquipmentScreenHudReserveAutoTilemap_Off(game);
            let mut v1: u16 = 0u16;
            for _ in (0i16..8i16).step_by(2) {
                (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                    [(v1 as i32 + 327i32) as usize] = (kEquipmentScreenTilemap_MANUAL[v1 as usize]
                    as i32
                    | (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                        [(v1 as i32 + 327i32) as usize] as i32
                        & 0xfc00i32) as u16;
                v1 = v1.wrapping_add(1);
            }
        } else {
            set_ram_word!(game.reserve_health_mode = 1u16);
            EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game);
            let mut v3: u16 = 0u16;
            for _ in (0i16..8i16).step_by(2) {
                (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                    [(v3 as i32 + 327i32) as usize] = (kEquipmentScreenTilemap_AUTO[v3 as usize]
                    as i32
                    | (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                        [(v3 as i32 + 327i32) as usize] as i32
                        & 0xfc00i32) as u16;
                v3 = v3.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game: &mut Game) {
    let mut v0: u16 = -26229i32 as u16;
    if ram_word!(game, samus_reserve_health) == 0 {
        v0 = -26217i32 as u16;
    }
    let v1: *const u16 = RomPtr_82(v0, game) as *const u16;
    set_ram_word!(game.hud_tilemap[8] = *v1);
    set_ram_word!(game.hud_tilemap[9] = *v1.offset(1));
    set_ram_word!(game.hud_tilemap[40] = *v1.offset(2));
    set_ram_word!(game.hud_tilemap[41] = *v1.offset(3));
    set_ram_word!(game.hud_tilemap[72] = *v1.offset(4));
    set_ram_word!(game.hud_tilemap[73] = *v1.offset(5));
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_Off(game: &mut Game) {
    set_ram_words! { game, 11279u16;
        hud_tilemap[8], hud_tilemap[9], hud_tilemap[40], hud_tilemap[41], hud_tilemap[72],
        hud_tilemap[73],
    }
}
static kReserveTankEnergyTransferPerFrame: u16 = 1u16;
pub unsafe fn EquipmentScreenCategory_Tanks_1(game: &mut Game) {
    if ram_word!(game, pausemenu_reserve_tank_delay_ctr) == 0 {
        if (ram_word!(game, joypad1_newkeys) as i32 & kButton_A as i32) == 0i32 {
            return;
        }
        set_ram_word!(
            game.pausemenu_reserve_tank_delay_ctr =
                (ram_word!(game, samus_reserve_health) as i32 + 7i32 & 0xfff8i32) as u16
        );
    }
    if (update_ram_word!(
        game.pausemenu_reserve_tank_delay_ctr,
        wrapping_sub(1),
        reread
    ) as i32
        & 7i32)
        == 7i32
    {
        QueueSfx3_Max6(0x2du16, game);
    }
    update_ram_word!(
        game.samus_health,
        wrapping_add(kReserveTankEnergyTransferPerFrame)
    );
    if ((ram_word!(game, samus_health)) as i16)
        .wrapping_sub((ram_word!(game, samus_max_health)) as i16)
        < 0i16
    {
        update_ram_word!(
            game.samus_reserve_health,
            wrapping_sub(kReserveTankEnergyTransferPerFrame)
        );
        if (ram_word!(game, samus_reserve_health) as i16) > 0i16 {
            return;
        }
        update_ram_word!(
            game.samus_health,
            wrapping_add(ram_word!(game, samus_reserve_health))
        );
    } else {
        set_ram_word!(game.samus_health = ram_word!(game, samus_max_health));
    }
    set_ram_words! { game, 0u16; samus_reserve_health, pausemenu_reserve_tank_delay_ctr }
    EquipmentScreenEnergyArrowGlow_Off(game);
    set_ram_word!(game.pausemenu_equipment_category_item = 0u16);
}
pub unsafe fn EquipmentScreenCategory_Weapons(game: &mut Game) {
    EquipmentScreenCategory_Weapons_MoveButtons(game);
    let R36: u16 = ram_word!(game, equipped_beams);
    if ram_word!(game, collected_beams) != 0
        && (ram_word!(game, pausemenu_equipment_category_item) as u8) == 1u8
    {
        EquipmentScreenCategory_ButtonResponse(10u16, game);
        EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_MoveButtons(game: &mut Game) {
    let r18: u16 = ram_word!(game, pausemenu_equipment_category_item);
    if (ram_word!(game, joypad1_newkeys) as i32 & 0x100i32) != 0i32 {
        if (ram_word!(game, joypad1_newkeys) as i32 & 0x800i32) != 0i32 {
            EquipmentScreenMoveLowerOnSuitsMisc(0u16, game);
        } else if EquipmentScreenMoveLowerOnSuitsMisc(4u16, game) != 0 {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0u16, r18, game);
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & 0x400i32) != 0i32 {
        if ram_word!(game, hyper_beam_flag) == 0
            && ram_word!(game, pausemenu_equipment_category_item) != 1025u16
        {
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_add(256u16));
            EquipmentScreenMoveToBeams(
                (2u16)
                    .wrapping_mul((*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize)) as u16),
                r18,
                game,
            );
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & 0x800i32) != 0i32
        && ((ram_word!(game, pausemenu_equipment_category_item) as i32 & 0xff00i32) == 0i32 || {
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_sub(256u16));
            EquipmentScreenMoveToBottomOfBeams(
                (2u16)
                    .wrapping_mul((*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize)) as u16),
                game,
            ) == 0xffffu16
        })
        && EquipmentScreenMoveToReserveTanks(game) == 0
    {
        set_ram_word!(game.pausemenu_equipment_category_item = r18);
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36: u16, game: &mut Game) {
    let t: i32 = ram_word!(game, equipped_beams) as i32 & !(R36 as i32);
    if (t & 4i32) != 0i32 {
        if (R36 as i32 & 4i32) == 0i32 && (ram_word!(game, equipped_beams) as i32 & 8i32) != 0i32 {
            update_ram_word!(game.equipped_beams, |fresh59| (fresh59 as i32 & !(8i32))
                as u16);
            ChangePaletteValues(
                ram_ptr!(game, u16; crate::rtl::rom_table_word!(game, 0x82c06c, 4) as i32 as isize),
                3072u16,
                5 as size_t,
            );
        }
    } else if (t & 8i32) != 0i32
        && (R36 as i32 & 8i32) == 0i32
        && (ram_word!(game, equipped_beams) as i32 & 4i32) != 0i32
    {
        update_ram_word!(game.equipped_beams, |fresh60| (fresh60 as i32 & !(4i32))
            as u16);
        ChangePaletteValues(
            ram_ptr!(game, u16; crate::rtl::rom_table_word!(game, 0x82c06c, 3) as i32 as isize),
            3072u16,
            5 as size_t,
        );
    }
}
pub unsafe fn EquipmentScreenCategory_Suit(game: &mut Game) {
    EquipmentScreenCategory_Suit_MoveResponse(game);
    if (ram_word!(game, pausemenu_equipment_category_item) as u8) == 2u8 {
        EquipmentScreenCategory_ButtonResponse(18u16, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Suit_MoveResponse(game: &mut Game) {
    let r18: u16 = ram_word!(game, pausemenu_equipment_category_item);
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Left as i32) != 0i32 {
        if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) == 0i32 {
            if EquipmentScreenMoveToReserveTanks(game) != 0 {
                return;
            }
            set_ram_word!(game.pausemenu_equipment_category_item = r18);
        }
        EquipmentScreenMoveToBeams(0u16, r18, game);
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
        if (ram_word!(game, pausemenu_equipment_category_item) as i32 & 0xff00i32) != 0i32 {
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_sub(256u16));
            EquipmentScreenMoveToScrewOrHigherOnSuits(
                (2u16)
                    .wrapping_mul((*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize)) as u16),
                r18,
                game,
            );
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32
        && ((ram_word!(game, pausemenu_equipment_category_item) as i32 & 0xff00i32) == 1280i32 || {
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_add(256u16));
            EquipmentScreenMoveLowerOnSuitsMisc(
                (2u16)
                    .wrapping_mul((*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize)) as u16),
                game,
            ) == 0xffffu16
        })
    {
        EquipmentScreenMoveToHighJumpOrLowerInBoots(0u16, r18, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Boots(game: &mut Game) {
    EquipmentScreenCategory_Boots_MoveResponse(game);
    if (ram_word!(game, pausemenu_equipment_category_item) as u8) == 3u8 {
        EquipmentScreenCategory_ButtonResponse(18u16, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveResponse(game: &mut Game) {
    let r18: u16 = ram_word!(game, pausemenu_equipment_category_item);
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Left as i32) != 0i32 {
        if ((ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32
            || EquipmentScreenMoveToBottomOfBeams(8u16, game) == 0xffffu16)
            && EquipmentScreenMoveToReserveTanks(game) == 0
        {
            set_ram_word!(game.pausemenu_equipment_category_item = r18);
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
        if ram_word!(game, pausemenu_equipment_category_item) != 515u16 {
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_add(256u16));
            EquipmentScreenMoveToHighJumpOrLowerInBoots(
                (2u16)
                    .wrapping_mul((*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize)) as u16),
                r18,
                game,
            );
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32
        && ((ram_word!(game, pausemenu_equipment_category_item) as i32 & 0xff00i32) == 0i32 || {
            update_ram_word!(game.pausemenu_equipment_category_item, wrapping_sub(256u16));
            EquipmentScreenCategory_Boots_MoveUpInBoots(
                (2u16)
                    .wrapping_mul((*(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize)) as u16),
                game,
            ) == 0xffffu16
        })
    {
        EquipmentScreenMoveToScrewOrHigherOnSuits(0xau16, r18, game);
    }
}
pub unsafe fn WriteSamusWireframeTilemapAndQueue(game: &mut Game) {
    WriteSamusWireframeTilemap(game);
    let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 1280u16;
    v0 = (v0).wrapping_add(2u16);
    (*gVramWriteEntry(v0 as i32, game)).size = ((ram_ptr!(game, Ram3800; 0x3800) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8)
        as c_long
        + 256 as c_long) as u16;
    v0 = (v0).wrapping_add(2u16);
    let fresh63 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh63 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = 126u8;
    (*gVramWriteEntry(v0 as i32, game)).size = 0x3080u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
}
pub unsafe fn WriteSamusWireframeTilemap(game: &mut Game) {
    let mut i: u16 = 0u16;
    while ram_word!(game, equipped_items) as i32 & 0x101i32
        != crate::rtl::rom_table_word!(game, 0x82b257, (i as i32 >> 1i32) as isize) as i32
    {
        i = (i).wrapping_add(2u16);
    }
    let mut src: *const u16 = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x82b25f, (i as i32 >> 1i32) as isize),
        game,
    ) as *const u16;
    let v1: u16 = 0u16;
    let mut v2: u16 = 472u16;
    for _ in 0i32..17i32 {
        let v3: u16 = v2;
        for _ in 0i32..8i32 {
            let fresh64 = src;
            src = src.offset(1);
            *((&raw mut (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap as *mut u16
                as *mut u8)
                .offset(v2 as i32 as isize) as *mut u16) = *fresh64;
            v2 = (v2).wrapping_add(2u16);
        }
        v2 = (v3).wrapping_add(64u16);
    }
}
pub unsafe fn EquipmentScreenDrawItemSelector(game: &mut Game) {
    if ram_word!(game, samus_max_reserve_health) as i32
        | ((ram_word!(game, collected_items)) | (ram_word!(game, collected_beams))) as i32
        != 0
    {
        let t: *mut u16 = RomPtr_82(
            (crate::rtl::rom_table_word!(
                game,
                0x82c18e,
                ram_word!(game, pausemenu_equipment_category_item) as u8 as isize
            ) as i32
                + 4i32 * *(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize) as i32)
                as u16,
            game,
        ) as *mut u16;
        DrawPauseScreenSpriteAnim(3u16, (*t.offset(0)).wrapping_sub(1u16), *t.offset(1), game);
    }
}
pub unsafe fn EquipmentScreenDisplayReserveTankAmount(game: &mut Game) {
    EquipmentScreenDisplayReserveTankAmount_(game);
}
static kEquipmentScreenReserveTank_X: [u16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
static kEquipmentScreenReserveTank_Y: u16 = 0x60u16;
static kPartialReserveTankSpritemapIds: [u16; 16] = [
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
];
pub unsafe fn EquipmentScreenDisplayReserveTankAmount_(game: &mut Game) {
    let r3: u16 = EquipmentScreenDisplayReserves_PaletteSetup(game);
    let mut r52: i32 = 0i32;
    if ram_word!(game, samus_max_reserve_health) == 0 {
        return;
    }
    let R44: u16 = (ram_word!(game, samus_max_reserve_health) as i32 / 100i32) as u16;
    let mod_value: i32 = ram_word!(game, samus_reserve_health) as i32 % 100i32;
    let R50: u16 = mod_value as u16;
    let r42: i32 = ram_word!(game, samus_reserve_health) as i32 / 100i32;
    let mut R48: u16 = r42 as u16;
    let mut r46: i32 = r42;
    if r42 != 0 {
        let mut v0: u16 = 0u16;
        loop {
            DrawMenuSpritemap(
                0x1bu16,
                kEquipmentScreenReserveTank_X[(v0 as i32 >> 1i32) as usize],
                (kEquipmentScreenReserveTank_Y).wrapping_sub(1u16),
                r3,
                game,
            );
            v0 = (v0).wrapping_add(2u16);
            r46 -= 1;
            if (r46 == 0) {
                break;
            }
        }
        r52 = v0 as i32;
    }
    let RegWord: u16 = mod_value as u16;
    if RegWord != 0 {
        let mut v2: u16 = (2i32 * (RegWord as i32 / 14i32)) as u16;
        if v2 as i32 - 7i32 & 0x8000i32 != 0
            && RegWord as i32 % 14i32 != 0
            && (*ram_ptr!(game; 0x5b5) as i32 & 4i32) == 0i32
        {
            v2 = (v2).wrapping_add(2u16);
        }
        if ram_word!(game, samus_reserve_health) as i32 - 100i32 & 0x8000i32 == 0 {
            v2 = (v2).wrapping_add(16u16);
        }
        DrawMenuSpritemap(
            kPartialReserveTankSpritemapIds[(v2 as i32 >> 1i32) as usize],
            kEquipmentScreenReserveTank_X[(r52 >> 1i32) as usize],
            (kEquipmentScreenReserveTank_Y).wrapping_sub(1u16),
            r3,
            game,
        );
        R48 = R48.wrapping_add(1);
        r52 += 2i32;
    }
    while R48 as i32 - R44 as i32 & 0x8000i32 != 0 {
        DrawMenuSpritemap(
            0x20u16,
            kEquipmentScreenReserveTank_X[(r52 >> 1i32) as usize],
            (kEquipmentScreenReserveTank_Y).wrapping_sub(1u16),
            r3,
            game,
        );
        r52 += 2i32;
        R48 = R48.wrapping_add(1);
    }
    DrawMenuSpritemap(
        0x1fu16,
        kEquipmentScreenReserveTank_X[(r52 >> 1i32) as usize],
        (kEquipmentScreenReserveTank_Y).wrapping_sub(1u16),
        r3,
        game,
    );
    let div_val: i32 = R50 as i32 / 10i32;
    let mod_val: i32 = R50 as i32 % 10i32;
    (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[394i32 as usize] =
        (mod_val + 2052i32) as u16;
    (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[393i32 as usize] =
        (div_val + 2052i32) as u16;
    (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[392i32 as usize] =
        (r42 + 2052i32) as u16;
}
pub unsafe fn EquipmentScreenDisplayReserves_PaletteSetup(game: &mut Game) -> u16 {
    if ram_word!(game, samus_reserve_health) != 0 {
        let v0: bool = (update_ram_word!(
            game.pausemenu_reserve_tank_animation_timer,
            wrapping_sub(1),
            reread
        ) as i32
            & 0x8000i32)
            != 0i32;
        if ram_word!(game, pausemenu_reserve_tank_animation_timer) == 0 || v0 {
            update_ram_word!(game.pausemenu_reserve_tank_animation_frame, wrapping_add(1));
            let mut v1: u16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c165)) as *mut u8)
                .offset(
                    (2i32 * ram_word!(game, pausemenu_reserve_tank_animation_frame) as i32)
                        as isize,
                ) as u16;
            if v1 == 255u16 {
                set_ram_word!(game.pausemenu_reserve_tank_animation_frame = 0u16);
                v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c165)) as *mut u8).offset(0)
                    as u16;
            }
            set_ram_word!(game.pausemenu_reserve_tank_animation_timer = v1);
        }
        crate::rtl::rom_table_word!(game, 0x82c0fa, 3)
    } else {
        1536u16
    }
}
pub unsafe fn EquipmentScreenMoveToReserveTanks(game: &mut Game) -> u16 {
    let result: u16 = ram_word!(game, samus_max_reserve_health);
    if ram_word!(game, samus_max_reserve_health) != 0 {
        set_ram_word!(game.pausemenu_equipment_category_item = 0u16);
        QueueSfx1_Max6(0x37u16, game);
        return 1u16;
    }
    result
}
pub unsafe fn EquipmentScreenMoveToBeams(mut v0: u16, r18: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    let mut current_block_5: u64;
    if ram_word!(game, hyper_beam_flag) != 0 {
        current_block_5 = 16475780731721606008;
    } else {
        loop {
            if ((crate::rtl::rom_table_word!(game, 0x82c04c, (v0 as i32 >> 1i32) as isize) as i32
                & ram_word!(game, collected_beams) as i32)
                != 0i32)
            {
                current_block_5 = 6873731126896040597;
                break;
            }
            v0 = (v0).wrapping_add(2u16);
            if ((v0) as i16).wrapping_sub(10i16) >= 0i16 {
                current_block_5 = 16475780731721606008;
                break;
            }
        }
        if current_block_5 == 16475780731721606008 {
        } else {
            *(&raw mut v1 as *mut u8).offset(0) = ((v0 as i32 >> 1i32) as u16 as i32 >> 8i32) as u8;
            *(&raw mut v1 as *mut u8).offset(1isize) = (v0 as i32 >> 1i32) as u8;
            set_ram_word!(
                game.pausemenu_equipment_category_item = (v1 as i32 & 0xff00i32 | 1i32) as u16
            );
            QueueSfx1_Max6(0x37u16, game);
            current_block_5 = 13513818773234778473;
        }
    }
    if current_block_5 == 16475780731721606008 {
        set_ram_word!(game.pausemenu_equipment_category_item = r18);
    };
}
pub unsafe fn EquipmentScreenMoveToBottomOfBeams(mut k: u16, game: &mut Game) -> u16 {
    if ram_word!(game, hyper_beam_flag) != 0 {
        return -1i32 as u16;
    }
    while (crate::rtl::rom_table_word!(game, 0x82c04c, (k as i32 >> 1i32) as isize) as i32
        & ram_word!(game, collected_beams) as i32)
        == 0i32
    {
        k = (k).wrapping_sub(2u16);
        if (k as i32 & 0x8000i32) != 0i32 {
            return -1i32 as u16;
        }
    }
    QueueSfx1_Max6(0x37u16, game);
    let result: u16 = ((k as i32 >> 1i32) << 8i32 | 1i32) as u16;
    set_ram_word!(game.pausemenu_equipment_category_item = result);
    result
}
pub unsafe fn EquipmentScreenMoveLowerOnSuitsMisc(mut v0: u16, game: &mut Game) -> u16 {
    let mut v1: i16 = 0;
    while (crate::rtl::rom_table_word!(game, 0x82c056, (v0 as i32 >> 1i32) as isize) as i32
        & ram_word!(game, collected_items) as i32)
        == 0i32
    {
        v0 = (v0).wrapping_add(2u16);
        if ((v0) as i16).wrapping_sub(10i16) >= 0i16 {
            return -1i32 as u16;
        }
    }
    QueueSfx1_Max6(0x37u16, game);
    *(&raw mut v1 as *mut u8).offset(0) = ((v0 as i32 >> 1i32) as u16 as i32 >> 8i32) as u8;
    *(&raw mut v1 as *mut u8).offset(1isize) = (v0 as i32 >> 1i32) as u8;
    set_ram_word!(game.pausemenu_equipment_category_item = (v1 as i32 & 0xff00i32 | 2i32) as u16);
    0u16
}
pub unsafe fn EquipmentScreenMoveToScrewOrHigherOnSuits(mut v0: u16, r18: u16, game: &mut Game) {
    while (crate::rtl::rom_table_word!(game, 0x82c056, (v0 as i32 >> 1i32) as isize) as i32
        & ram_word!(game, collected_items) as i32)
        == 0i32
    {
        v0 = (v0).wrapping_sub(2u16);
        if (v0 as i32 & 0x8000i32) != 0i32 {
            set_ram_word!(game.pausemenu_equipment_category_item = r18);
            return;
        }
    }
    QueueSfx1_Max6(0x37u16, game);
    set_ram_word!(
        game.pausemenu_equipment_category_item = ((v0 as i32 >> 1i32) << 8i32 | 2i32) as u16
    );
}
pub unsafe fn EquipmentScreenMoveToHighJumpOrLowerInBoots(mut v0: u16, r18: u16, game: &mut Game) {
    while (crate::rtl::rom_table_word!(game, 0x82c062, (v0 as i32 >> 1i32) as isize) as i32
        & ram_word!(game, collected_items) as i32)
        == 0i32
    {
        v0 = (v0).wrapping_add(2u16);
        if ((v0) as i16).wrapping_sub(6i16) >= 0i16 {
            set_ram_word!(game.pausemenu_equipment_category_item = r18);
            return;
        }
    }
    QueueSfx1_Max6(0x37u16, game);
    set_ram_word!(
        game.pausemenu_equipment_category_item = ((v0 as i32 >> 1i32) << 8i32 | 3i32) as u16
    );
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveUpInBoots(mut k: u16, game: &mut Game) -> u16 {
    while (crate::rtl::rom_table_word!(game, 0x82c062, (k as i32 >> 1i32) as isize) as i32
        & ram_word!(game, collected_items) as i32)
        == 0i32
    {
        k = (k).wrapping_sub(2u16);
        if (k as i32 & 0x8000i32) != 0i32 {
            return -1i32 as u16;
        }
    }
    QueueSfx1_Max6(0x37u16, game);
    let result: u16 = ((k as i32 >> 1i32) << 8i32 | 3i32) as u16;
    set_ram_word!(game.pausemenu_equipment_category_item = result);
    result
}
pub unsafe fn EquipmentScreenCategory_ButtonResponse(r24: u16, game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_A as i32) != 0i32 {
        QueueSfx1_Max6(0x38u16, game);
        let item: i32 = *(ram_ptr!(game, u16; 0x755) as *mut u8).offset(1isize) as i32;
        let category: i32 = ram_word!(game, pausemenu_equipment_category_item) as u8 as i32;
        let target: *mut u8 = ram_ptr!(game; *(RomPtr_82(
                (crate::rtl::rom_table_word!(game, 0x82c02c, category as isize) as i32
                    + item * 2i32) as u16,
                game,
            ) as *mut u16) as i32 as isize,);
        let var: *mut u16 = RomPtr_RAM(
            crate::rtl::rom_table_word!(game, 0x82c03c, category as isize),
            game,
        ) as *mut u16;
        let mask: u16 = *(RomPtr_82(
            (crate::rtl::rom_table_word!(game, 0x82c034, category as isize) as i32 + item * 2i32)
                as u16,
            game,
        ) as *mut u16);
        if (*var as i32 & mask as i32) != 0i32 {
            *var = (*var as i32 & !(mask as i32)) as u16;
            ChangePaletteValues(target as *mut u16, 0xc00u16, (r24 as i32 >> 1i32) as size_t);
        } else {
            *var = (*var) | (mask);
            let src: u16 = *(RomPtr_82(
                (crate::rtl::rom_table_word!(game, 0x82c044, category as isize) as i32
                    + item * 2i32) as u16,
                game,
            ) as *mut u16);
            memcpy(
                target as *mut c_void,
                RomPtr_82(src, game) as *const c_void,
                r24 as size_t,
            );
        }
    }
}
pub unsafe fn DrawPauseMenuDuringFadeIn(game: &mut Game) {
    if ram_word!(game, menu_index) != 0 {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
    } else {
        MapScreenDrawSamusPositionIndicator(game);
        DrawMapIcons(game);
        DisplayMapElevatorDestinations(game);
    };
}
pub unsafe fn UNKNOWN_sub_82B650(game: &mut Game) {
    if ram_word!(game, pausemenu_button_label_mode) != 0 {
        DisplayMapElevatorDestinations(game);
        MapScreenDrawSamusPositionIndicator(game);
        DrawMapIcons(game);
    } else {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
    };
}
static kMap_Criteria_SavePoints: [u16; 16] = [
    0xd8, 0x28, 0x90, 0x38, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe,
    0xfffe, 0xfffe, 0xfffe,
];
pub unsafe fn DrawMapIcons(game: &mut Game) {
    DrawBossMapIcons(9u16, addr_kMapIconDataPointers as u16, game);
    DrawSimpleMapIcons(
        0xbu16,
        (addr_kMapIconDataPointers as i32 + 0x10i32) as u16,
        3584u16,
        game,
    );
    DrawSimpleMapIcons(
        0xau16,
        (addr_kMapIconDataPointers as i32 + 0x20i32) as u16,
        3584u16,
        game,
    );
    DrawSimpleMapIcons(
        0x4eu16,
        (addr_kMapIconDataPointers as i32 + 0x30i32) as u16,
        3584u16,
        game,
    );
    DrawSaveStationMapIcon(8u16, 0xc80bu16, 1024u16, game);
    if ram_word!(game, enable_debug) != 0 {
        DrawSimpleMapIcons(8u16, 0xc82bu16, 1024u16, game);
    }
    if ram_word!(game, area_index) == 0 {
        DrawMenuSpritemap(
            0x63u16,
            (kMap_Criteria_SavePoints[0i32 as usize]).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
            (kMap_Criteria_SavePoints[1i32 as usize]).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
            3584u16,
            game,
        );
    }
}
pub unsafe fn DrawFileSelectMapIcons(game: &mut Game) {
    HandlePauseScreenPaletteAnimation(game);
    DrawBossMapIcons(9u16, 0xc7cbu16, game);
    DrawSimpleMapIcons(0xbu16, 0xc7dbu16, 3584u16, game);
    DrawSimpleMapIcons(0xau16, 0xc7ebu16, 3584u16, game);
    DrawSimpleMapIcons(0x4eu16, 0xc7fbu16, 3584u16, game);
    let a: u16 = UpdateSamusPositionIndicatorAnimation(game);
    let data: *const u16 = (RomPtr_82(
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c7cb))
            as *mut MapIconDataPointers)
            .offset(4))
        .crateria as *mut u8)
            .offset((2i32 * ram_word!(game, area_index) as i32) as isize) as *mut VoidP),
        game,
    ) as *mut u16)
        .offset((ram_word!(game, load_station_index) as i32 * 2i32) as isize);
    let v1: u16 = (*data.offset(0)).wrapping_sub(ram_word!(game, reg_BG1HOFS));
    let v2: u16 = (*data.offset(1)).wrapping_sub(ram_word!(game, reg_BG1VOFS));
    if (ram_word!(game, samus_position_indicator_animation_loop_counter) as i32 & 1i32) == 0i32 {
        DrawMenuSpritemap(0x12u16, v1, v2, 3584u16, game);
    }
    DrawMenuSpritemap(a, v1, v2, 3584u16, game);
    if ram_word!(game, enable_debug) != 0 {
        DrawDebugSaveMapIcons(0xcu16, 0xc80bu16, 1536u16, game);
        DrawDebugElevatorMapIcons(0x17u16, 0xc81bu16, 1536u16, game);
        DrawSimpleMapIcons(0xcu16, 0xc82bu16, 1536u16, game);
    }
    if ram_word!(game, area_index) == 0 {
        DrawMenuSpritemap(
            0x63u16,
            (kMap_Criteria_SavePoints[0i32 as usize]).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
            (kMap_Criteria_SavePoints[1i32 as usize]).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
            3584u16,
            game,
        );
    }
}
pub unsafe fn DrawSaveStationMapIcon(a: u16, k: u16, r3: u16, game: &mut Game) {
    static kShlBit: [u8; 8] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80];
    let R34: u16 = a;
    if ram_word!(game, area_index) == ram_word!(game, sram_area_index) {
        let mut R36: u16 =
            *(ram_ptr!(game, u16; 0xd8f8, (2i32 * ram_word!(game, area_index) as i32) as isize));
        *(&raw mut R36 as *mut u8).offset(0) =
            ((R36) as u8) & (kShlBit[ram_word!(game, load_station_index) as usize]);
        let v2: u16 = *(RomPtr_82(
            (k as i32 + 2i32 * ram_word!(game, area_index) as i32) as u16,
            game,
        ) as *mut u16);
        if v2 != 0 {
            DrawMapIconsOfType(v2, R34, R36, r3, game);
        }
    }
}
pub unsafe fn DrawDebugSaveMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    let R34: u16 = a;
    let R36: u16 =
        *(ram_ptr!(game, u16; 0xd8f8, (2i32 * ram_word!(game, area_index) as i32) as isize));
    let v2: u16 = *(RomPtr_82(
        (k as i32 + 2i32 * ram_word!(game, area_index) as i32) as u16,
        game,
    ) as *mut u16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, R36, r3, game);
    }
}
pub unsafe fn DrawDebugElevatorMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    let R34: u16 = a;
    let R36: u16 =
        *(ram_ptr!(game, u16; 0xd8f8, (2i32 * ram_word!(game, area_index) as i32 + 1i32) as isize));
    let v2: u16 = *(RomPtr_82(
        (k as i32 + 2i32 * ram_word!(game, area_index) as i32) as u16,
        game,
    ) as *mut u16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, R36, r3, game);
    }
}
pub unsafe fn DrawSimpleMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    let R34: u16 = a;
    let v2: u16 = *(RomPtr_82(
        (k as i32 + 2i32 * ram_word!(game, area_index) as i32) as u16,
        game,
    ) as *mut u16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, -1i32 as u16, r3, game);
    }
}
pub unsafe fn DrawMapIconsOfType(mut a: u16, r34: u16, mut r36: u16, r3: u16, game: &mut Game) {
    loop {
        let v2: *const u16 = RomPtr_82(a, game) as *const u16;
        if (*v2 as i32 & 0x8000i32) != 0i32 {
            break;
        }
        let mut v3: i8 = ((r36) as i8) & (1i8);
        r36 = (r36 as i32 >> 1i32) as u16;
        if v3 != 0 {
            let mut v4: i16 =
                CheckIfMapPositionIsExplored(*v2.offset(0), *v2.offset(1), game) as i16;
            if v4 != 0 {
                DrawMenuSpritemap(
                    r34,
                    (*v2.offset(0)).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
                    (*v2.offset(1)).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
                    r3,
                    game,
                );
            }
        }
        a = (a).wrapping_add(4u16);
    }
}
pub unsafe fn CheckIfMapPositionIsExplored(k: u16, j: u16, game: &mut Game) -> u16 {
    static kBits0x80Shr: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
    let mut r18: i32 = ((k) & (0xff00u16)) as i32 >> 1i32;
    r18 += k as u8 as i32 >> 6i32;
    r18 += ((j) & (0xfff8u16)) as i32 >> 1i32;
    let v3: u16 = (k as i32 >> 3i32 & 7i32) as u16;
    ((kBits0x80Shr[v3 as usize] as i32 & *ram_ptr!(game; 0x7f7, r18 as isize) as i32) != 0i32)
        as u16
}
pub unsafe fn DrawBossMapIcons(a: u16, k: u16, game: &mut Game) {
    let mut bits: i32 = *ram_ptr!(game; 0xd828, ram_word!(game, area_index) as isize) as i32;
    let t: i32 = *(RomPtr_82(
        (k as i32 + 2i32 * ram_word!(game, area_index) as i32) as u16,
        game,
    ) as *mut u16) as i32;
    if t == 0i32 {
        return;
    }
    let mut v4: *const u16 = RomPtr_82(t as u16, game) as *const u16;
    let mut advance_boss_bit: bool;
    loop {
        if *v4.offset(0) == 0xffffu16 {
            return;
        }
        advance_boss_bit = !(*v4.offset(0) != 0xfffeu16)
            || ({
                let v5: i32 = bits & 1i32;
                bits >>= 1i32;
                if v5 != 0 {
                    DrawMenuSpritemap(
                        0x62u16,
                        (*v4.offset(0)).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
                        (*v4.offset(1)).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
                        3584u16,
                        game,
                    );
                    DrawMenuSpritemap(
                        a,
                        (*v4.offset(0)).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
                        (*v4.offset(1)).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
                        3072u16,
                        game,
                    );
                    false
                } else {
                    !(ram_word!(game, has_area_map) != 0)
                        || ({
                            DrawMenuSpritemap(
                                a,
                                (*v4.offset(0)).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
                                (*v4.offset(1)).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
                                3584u16,
                                game,
                            );
                            false
                        })
                }
            });
        if advance_boss_bit {
            bits >>= 1i32;
        }
        v4 = v4.offset(2);
    }
}
pub unsafe fn DrawMapScrollArrowAndCheckToScroll(db: u8, k: u16, game: &mut Game) {
    let v1: *const u16 = RomPtrWithBank(db, k, game) as *const u16;
    DrawPauseScreenSpriteAnim(*v1.offset(2), *v1, *v1.offset(1), game);
    let v2: *const u8 = RomPtrWithBank(db, k, game);
    if (ram_word!(game, joypad1_lastkeys) as i32 & *(v2.offset(6) as *mut u16) as i32) != 0i32
        && ram_word!(game, map_scrolling_direction) == 0
    {
        set_ram_word!(game.map_scrolling_direction = *(v2.offset(8) as *mut u16));
    }
}
pub unsafe fn sub_82B932(game: &mut Game) {
    HandleMapScrollArrows(game);
}
pub unsafe fn HandleMapScrollArrows(game: &mut Game) {
    if ram_word!(game, map_min_x_scroll) as i32 - 24i32 - ram_word!(game, reg_BG1HOFS) as i32
        & 0x8000i32
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x82u8, addr_stru_82B9A0 as u16, game);
    }
    if ram_word!(game, map_max_x_scroll) as i32 - 232i32 - ram_word!(game, reg_BG1HOFS) as i32
        & 0x8000i32
        == 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x82u8, addr_stru_82B9AA as u16, game);
    }
    if ram_word!(game, map_min_y_scroll) as i32 - 56i32 - ram_word!(game, reg_BG1VOFS) as i32
        & 0x8000i32
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x82u8, addr_stru_82B9B4 as u16, game);
    }
    if ram_word!(game, map_max_y_scroll) as i32 - 177i32 - ram_word!(game, reg_BG1VOFS) as i32
        & 0x8000i32
        != 0
    {
        if ram_word!(game, map_scrolling_direction)
            == (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82b9be)) as *mut MapScrollArrowData))
                .map_scroll_dir
        {
            set_ram_words! { game, 0u16;
                map_scrolling_gear_switch_timer, map_scrolling_direction, map_scrolling_speed_index,
            }
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(0x82u8, addr_stru_82B9BE as u16, game);
    };
}
pub unsafe fn MapScreenDrawSamusPositionIndicator(game: &mut Game) {
    let a: u16 = UpdateSamusPositionIndicatorAnimation(game);
    DrawMenuSpritemap(
        a,
        (8i32
            * (ram_word!(game, room_x_coordinate_on_map) as i32
                + *(ram_ptr!(game, u16; 0xaf6) as *mut u8).offset(1isize) as i32)
            - ram_word!(game, reg_BG1HOFS) as i32) as u16,
        (8i32
            * (ram_word!(game, room_y_coordinate_on_map) as i32
                + *(ram_ptr!(game, u16; 0xafa) as *mut u8).offset(1isize) as i32
                + 1i32)
            - ram_word!(game, reg_BG1VOFS) as i32) as u16,
        3584u16,
        game,
    );
}
static kMap_SamusPositionIndicator_Delays: [u16; 4] = [8u16, 4u16, 8u16, 4u16];
static kMap_SamusPositionIndicator_SpritemapIds: [u16; 4] = [0x5f, 0x60, 0x61, 0x60];
pub unsafe fn UpdateSamusPositionIndicatorAnimation(game: &mut Game) -> u16 {
    let mut v0: u16 = ram_word!(game, samus_position_indicator_animation_timer);
    if v0 == 0 {
        let mut v1: u16 =
            (ram_word!(game, samus_position_indicator_animation_frame)).wrapping_add(2u16);
        if ((v1) as i16).wrapping_sub(8i16) >= 0i16 {
            update_ram_word!(
                game.samus_position_indicator_animation_loop_counter,
                wrapping_add(1)
            );
            v1 = 0u16;
        }
        set_ram_word!(game.samus_position_indicator_animation_frame = v1);
        v0 = kMap_SamusPositionIndicator_Delays[(v1 as i32 >> 1i32) as usize];
        set_ram_word!(game.samus_position_indicator_animation_timer = v0);
    }
    set_ram_word!(game.samus_position_indicator_animation_timer = (v0).wrapping_sub(1u16));
    kMap_SamusPositionIndicator_SpritemapIds
        [(ram_word!(game, samus_position_indicator_animation_frame) as i32 >> 1i32) as usize]
}
pub unsafe fn DrawBorderAroundSamusData(game: &mut Game) {
    DrawMenuSpritemap(0x48u16, 0x80u16, 0x10u16, 3584u16, game);
}
pub unsafe fn DrawBorderAroundDataCopyMode(game: &mut Game) {
    DrawMenuSpritemap(0x49u16, 0x80u16, 0x10u16, 3584u16, game);
}
pub unsafe fn DrawBorderAroundDataClearMode(game: &mut Game) {
    DrawMenuSpritemap(0x4au16, 0x7cu16, 0x10u16, 3584u16, game);
}
static kDrawMenuSelectionMissile_Enable: [u16; 4] = [8u16, 8u16, 8u16, 8u16];
static kDrawMenuSelectionMissile_SpriteMap: [u16; 4] = [0x37, 0x36, 0x35, 0x34];
pub unsafe fn DrawMenuSelectionMissile(game: &mut Game) {
    if ram_word!(game, cinematic_var5) != 0 {
        if update_ram_word!(game.cinematic_var5, wrapping_sub(1), reread) == 0 {
            set_ram_word!(
                game.eproj_id[0] =
                    (*((ram_ptr!(game, u16; 0x1997)).offset(0) as *mut u8).offset(0) as i32 + 1i32
                        & 3i32) as u16
            );
            set_ram_word!(
                game.cinematic_var5 =
                    kDrawMenuSelectionMissile_Enable[ram_word!(game, eproj_id[0]) as usize]
            );
        }
    }
    DrawMenuSpritemap(
        kDrawMenuSelectionMissile_SpriteMap[ram_word!(game, eproj_id[0]) as usize],
        ram_word!(game, eproj_id[5]),
        ram_word!(game, eproj_id[10]),
        3584u16,
        game,
    );
}
pub unsafe fn DrawFileCopyArrow(game: &mut Game) {
    let mut v0: u16 = 0;
    if ((ram_word!(game, eproj_id[16])) as i16).wrapping_sub((ram_word!(game, eproj_id[17])) as i16)
        >= 0i16
    {
        v0 = 3u16;
        if ram_word!(game, eproj_id[16]) as i32 - ram_word!(game, eproj_id[17]) as i32 - 2i32
            & 0x8000i32
            != 0
        {
            v0 = 4u16;
            if ram_word!(game, eproj_id[16]) != 1u16 {
                v0 = 5u16;
            }
        }
    } else {
        v0 = 0u16;
        if ram_word!(game, eproj_id[17]) as i32 - ram_word!(game, eproj_id[16]) as i32 - 2i32
            & 0x8000i32
            != 0
        {
            v0 = 1u16;
            if ram_word!(game, eproj_id[16]) != 0 {
                v0 = 2u16;
            }
        }
    }
    DrawMenuSpritemap(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82bb0c)) as *mut FileCopyArrowStuff)
            .offset(v0 as isize))
        .spritemap,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82bb0c)) as *mut FileCopyArrowStuff)
            .offset(v0 as isize))
        .xpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82bb0c)) as *mut FileCopyArrowStuff)
            .offset(v0 as isize))
        .ypos,
        512u16,
        game,
    );
}
pub unsafe fn DisplayMapElevatorDestinations(game: &mut Game) {
    if *ram_ptr!(game; 0xd908, ram_word!(game, area_index) as isize) != 0 {
        let mut i: i32 =
            crate::rtl::rom_table_word!(game, 0x82c74d, ram_word!(game, area_index) as isize)
                as i32;
        loop {
            let v1: *const u8 = RomPtr_82(i as u16, game);
            if *(v1 as *mut u16) == 0xffffu16 {
                break;
            }
            DrawMenuSpritemap(
                *(v1.offset(4) as *mut u16),
                (*(v1 as *mut u16)).wrapping_sub(ram_word!(game, reg_BG1HOFS)),
                (*(v1.offset(2) as *mut u16)).wrapping_sub(ram_word!(game, reg_BG1VOFS)),
                0u16,
                game,
            );
            i += 6i32;
        }
    }
}
pub unsafe fn HandleGameOverBabyMetroid(game: &mut Game) {
    if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).instruction_timer != 0 {
        sub_82BB7F(
            (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).instruction_timer,
            game,
        );
    } else {
        sub_82BBDD(game);
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallBabyMetroidPlaySfx(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        8567820 => BabyMetroidPlaySfx0x23; 8567829 => BabyMetroidPlaySfx0x26;
        8567838 => BabyMetroidPlaySfx0x27;
    }
}
pub unsafe fn sub_82BB7F(a: u16, game: &mut Game) {
    let current_instruction: u16 =
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction;
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).instruction_timer = (a).wrapping_sub(1u16);
    if a == 1u16 {
        let v2: *const u8 = RomPtr_82(
            (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction,
            game,
        );
        if *(v2.offset(6) as *mut u16) == 0xffffu16 {
            sub_82BBDD(game);
        } else if ((*(v2.offset(6) as *mut u16)) as i16).wrapping_add(1i16) >= 0i16 {
            (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).instruction_timer =
                *(v2.offset(6) as *mut u16);
            let fresh18 =
                &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction;
            fresh18.write_unaligned((fresh18.read_unaligned()).wrapping_add(6u16));
            DrawBabyMetroid((current_instruction).wrapping_add(6u16), game);
        } else {
            CallBabyMetroidPlaySfx(((*(v2.offset(6) as *mut u16)) as u32) | (0x820000u32), game);
        }
    } else {
        DrawBabyMetroid(
            (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction,
            game,
        );
    };
}
pub unsafe fn sub_82BBDD(game: &mut Game) {
    enemy_instruction! {[(*(ram_ptr!(game, EnemyData; 0xf78)).offset(0))]; instruction = addr_kgameOverBabyMetridInstructionList as u16, timer = 10u16}
    sub_82BB7F(0xau16, game);
}
pub unsafe fn DrawBabyMetroid(k: u16, game: &mut Game) {
    let v1: u16 = *(RomPtr_82(k, game) as *mut u16).offset(2);
    let mut i: i32 = 0i32;
    while i < 32i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 192i32)] =
                *(RomPtr_82((v1 as i32 + i) as u16, game) as *mut u16)
        );
        i += 2i32;
    }
    let v3: *const u8 = RomPtr_82(k, game);
    DrawMenuSpritemap(*(v3.offset(2) as *mut u16), 0x7cu16, 0x50u16, 2048u16, game);
    DrawMenuSpritemap(0x64u16, 0x7cu16, 0x50u16, 2560u16, game);
}
pub unsafe fn FinishProcessingGameOverBabyMetroidAsm(game: &mut Game) {
    let t: u16 = *(RomPtr_82(
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction,
        game,
    ) as *mut u16)
        .offset(4);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).instruction_timer = t;
    let fresh17 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction;
    fresh17.write_unaligned((fresh17.read_unaligned()).wrapping_add(8u16));
    if t == 0xffffu16 {
        sub_82BBDD(game);
    } else {
        DrawBabyMetroid(
            (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).current_instruction,
            game,
        );
    };
}
pub unsafe fn BabyMetroidPlaySfx0x23(game: &mut Game) {
    QueueSfx3_Max6(0x23u16, game);
    FinishProcessingGameOverBabyMetroidAsm(game);
}
pub unsafe fn BabyMetroidPlaySfx0x26(game: &mut Game) {
    QueueSfx3_Max6(0x26u16, game);
    FinishProcessingGameOverBabyMetroidAsm(game);
}
pub unsafe fn BabyMetroidPlaySfx0x27(game: &mut Game) {
    QueueSfx3_Max6(0x27u16, game);
    FinishProcessingGameOverBabyMetroidAsm(game);
}
pub unsafe fn CancelSoundEffects(game: &mut Game) {
    QueueSfx1_Max6(2u16, game);
    QueueSfx2_Max6(0x71u16, game);
    QueueSfx3_Max6(1u16, game);
}
pub unsafe fn QueueSamusMovementSfx(game: &mut Game) {
    if (ram_word!(game, speed_boost_counter) as i32 & 0xff00i32) == 1024i32 {
        QueueSfx3_Max6(0x2bu16, game);
    }
    if ram_word!(game, flare_counter) as i32 - 16i32 & 0x8000i32 == 0 {
        QueueSfx1_Max6(0x41u16, game);
    }
    CallSomeSamusCode(0x14u16, game);
}
pub unsafe fn AdvancePaletteFadeForAllPalettes_0xc(game: &mut Game) -> u8 {
    set_ram_word!(game.palette_change_denom = 12u16);
    AdvancePaletteFadeForAllPalettes(game)
}
pub unsafe fn AdvancePaletteFade_BgPalette6(game: &mut Game) -> u8 {
    set_ram_word!(game.palette_change_denom = 12u16);
    if ram_word!(game, palette_change_num) <= 0xdu16 {
        let mut i: i32 = 192i32;
        while i < 0xe0i32 {
            set_ram_word!(
                game.palette_buffer[(i >> 1i32)] = CalculateNthTransitionColorFromXtoY(
                    ram_word!(game, palette_change_num),
                    ram_word!(game, palette_buffer[(i >> 1i32)]),
                    ram_word!(game, target_palettes[(i >> 1i32)]),
                    game,
                )
            );
            i += 2i32;
        }
        update_ram_word!(game.palette_change_num, wrapping_add(1));
        0u8
    } else {
        set_ram_word!(game.palette_change_num = 0u16);
        1u8
    }
}
pub unsafe fn AdvanceGradualColorChangeOfPalette(mut k: u16, j: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.palette_change_denom = 15u16);
    if (15i16).wrapping_sub((ram_word!(game, palette_change_num)) as i16) >= 0i16 {
        let R34: u16 = j;
        loop {
            set_ram_word!(
                game.palette_buffer[(k as i32 >> 1i32)] = CalculateNthTransitionColorFromXtoY(
                    (ram_word!(game, palette_change_num)).wrapping_add(1u16),
                    ram_word!(game, palette_buffer[(k as i32 >> 1i32)]),
                    ram_word!(game, target_palettes[(k as i32 >> 1i32)]),
                    game,
                )
            );
            k = (k).wrapping_add(2u16);
            if (k >= R34) {
                break;
            }
        }
        update_ram_word!(game.palette_change_num, wrapping_add(1));
        0u8
    } else {
        set_ram_word!(game.palette_change_num = 0u16);
        1u8
    }
}
pub unsafe fn AdvancePaletteFadeForAllPalettes(game: &mut Game) -> u8 {
    if (ram_word!(game, palette_change_denom)).wrapping_add(1u16)
        >= ram_word!(game, palette_change_num)
    {
        let mut i: i32 = 0i32;
        while i < 0x200i32 {
            set_ram_word!(game.g_word_7EC404 = i as u16);
            let v2: i32 = i >> 1i32;
            if ram_word!(game, target_palettes[v2]) != ram_word!(game, palette_buffer[v2]) {
                let v3: u16 = CalculateNthTransitionColorFromXtoY(
                    ram_word!(game, palette_change_num),
                    ram_word!(game, palette_buffer[v2]),
                    ram_word!(game, target_palettes[v2]),
                    game,
                );
                i = ram_word!(game, g_word_7EC404) as i32;
                *(ram_ptr!(game, u16; 0xc000))
                    .offset((ram_word!(game, g_word_7EC404) as i32 >> 1i32) as isize) = v3;
            }
            i += 2i32;
        }
        update_ram_word!(game.palette_change_num, wrapping_add(1));
        0u8
    } else {
        set_ram_word!(game.palette_change_num = 0u16);
        1u8
    }
}
pub unsafe fn CalculateNthTransitionColorFromXtoY(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    (CalculateNthTransitionColorComponentFromXtoY(a, (k) & (0x1fu16), (j) & (0x1fu16), game) as i32
        | (CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as i32 >> 5i32 & 0x1fi32) as u16,
            (j as i32 >> 5i32 & 0x1fi32) as u16,
            game,
        ) as i32)
            << 5i32
        | (CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as i32 >> 10i32 & 0x1fi32) as u16,
            (j as i32 >> 10i32 & 0x1fi32) as u16,
            game,
        ) as i32)
            << 10i32) as u16
}
pub unsafe fn CalculateNthTransitionColorComponentFromXtoY(
    a: u16,
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    if a == 0 {
        return k;
    }
    let v4: u16 = (a).wrapping_sub(1u16);
    if v4 == ram_word!(game, palette_change_denom) {
        return j;
    }
    let RegWord: u16 = SnesDivide(
        ((abs16((j).wrapping_sub(k)) as i32) << 8i32) as u16,
        (ram_word!(game, palette_change_denom) as i32 - (v4 as i32 + 1i32) + 1i32) as u8,
    );
    let r18: u16 = (if j as i32 - k as i32 & 0x8000i32 != 0 {
        -(RegWord as i32)
    } else {
        RegWord as i32
    }) as u16;
    ((r18 as i32 + ((k as i32) << 8i32)) as u16 as i32 >> 8i32) as u16
}
pub unsafe fn sub_82DAF7(a: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.palette_change_denom = 12u16);
    sub_82DB0C(a, game)
}
pub unsafe fn sub_82DB0C(mut a: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, palette_change_denom)).wrapping_add(1u16)
        >= ram_word!(game, palette_change_num)
    {
        set_ram_word!(game.g_word_7EC404 = 0u16);
        while a != 0 {
            if a as i32 & 1i32 == 0 {
                update_ram_word!(game.g_word_7EC404, wrapping_add(32u16));
            } else {
                sub_82DB41(game);
            }
            a = (a as i32 >> 1i32) as u16;
        }
        update_ram_word!(game.palette_change_num, wrapping_add(1));
        0u8
    } else {
        set_ram_word!(game.palette_change_num = 0u16);
        1u8
    }
}
pub unsafe fn sub_82DB41(game: &mut Game) {
    let mut v0: u16 = ram_word!(game, g_word_7EC404);
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        if ram_word!(game, target_palettes[v1]) != ram_word!(game, palette_buffer[v1]) {
            let v2: u16 = CalculateNthTransitionColorFromXtoY(
                ram_word!(game, palette_change_num),
                ram_word!(game, palette_buffer[v1]),
                ram_word!(game, target_palettes[v1]),
                game,
            );
            v0 = ram_word!(game, g_word_7EC404);
            *(ram_ptr!(game, u16; 0xc000))
                .offset((ram_word!(game, g_word_7EC404) as i32 >> 1i32) as isize) = v2;
        }
        v0 = (v0).wrapping_add(2u16);
        set_ram_word!(game.g_word_7EC404 = v0);
        if ((v0 as i32 & 0x1fi32) == 0i32) {
            break;
        }
    }
}
pub unsafe fn HandleSamusOutOfHealthAndGameTile(game: &mut Game) {
    if (ram_word!(game, samus_health) as i16) <= 0i16 {
        if (ram_word!(game, reserve_health_mode) as i32 & 1i32) != 0i32
            && ram_word!(game, samus_reserve_health) != 0u16
        {
            set_ram_word!(game.time_is_frozen_flag = 0x8000u16);
            set_ram_word!(game.game_state = kGameState_27_ReserveTanksAuto as u16);
            CallSomeSamusCode(0x1bu16, game);
        } else {
            if ram_word!(game, game_state) as i32 != kGameState_8_MainGameplay as i32 {
                return;
            }
            set_ram_word!(game.time_is_frozen_flag = 0x8000u16);
            CallSomeSamusCode(0x11u16, game);
            set_ram_word!(game.game_state = kGameState_19_SamusNoHealth as u16);
        }
    }
    if update_ram_word!(game.game_time_frames, wrapping_add(1), reread) as i32 - 60i32 & 0x8000i32
        == 0
    {
        set_ram_word!(game.game_time_frames = 0u16);
        if update_ram_word!(game.game_time_seconds, wrapping_add(1), reread) as i32 - 60i32
            & 0x8000i32
            == 0
        {
            set_ram_word!(game.game_time_seconds = 0u16);
            if update_ram_word!(game.game_time_minutes, wrapping_add(1), reread) as i32 - 60i32
                & 0x8000i32
                == 0
            {
                set_ram_word!(game.game_time_minutes = 0u16);
                update_ram_word!(game.game_time_hours, wrapping_add(1));
            }
        }
    }
    if ram_word!(game, game_time_hours) as i32 - 100i32 & 0x8000i32 == 0 {
        set_ram_words! { game, 59u16; game_time_frames, game_time_seconds, game_time_minutes }
        set_ram_word!(game.game_time_hours = 99u16);
    }
}
pub unsafe fn GameState_27_ReserveTanksAuto(game: &mut Game) -> CoroutineRet {
    if *ram_ptr!(game; 0x612) == 0u8 && RefillHealthFromReserveTanks(game) != 0u8 {
        set_ram_word!(game.time_is_frozen_flag = 0u16);
        set_ram_word!(game.game_state = kGameState_8_MainGameplay as u16);
        CallSomeSamusCode(0x10u16, game);
    }
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    Samus_LowHealthCheck_0(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RefillHealthFromReserveTanks(game: &mut Game) -> u8 {
    let refill_in_progress: bool;
    if ram_word!(game, samus_reserve_health) != 0 {
        if (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32 {
            QueueSfx3_Max3(0x2du16, game);
        }
        refill_in_progress = if ((update_ram_word!(game.samus_health, wrapping_add(1), reread))
            as i16)
            .wrapping_sub((ram_word!(game, samus_max_health)) as i16)
            >= 0i16
        {
            set_ram_word!(game.samus_health = ram_word!(game, samus_max_health));
            false
        } else {
            let v0: bool = (update_ram_word!(game.samus_reserve_health, wrapping_sub(1), reread)
                as i32
                & 0x8000i32)
                != 0i32;
            !(ram_word!(game, samus_reserve_health) == 0)
                && (!(v0)
                    || ({
                        update_ram_word!(
                            game.samus_health,
                            wrapping_add(ram_word!(game, samus_reserve_health))
                        );
                        false
                    }))
        };
        if refill_in_progress {
        } else {
            set_ram_word!(game.samus_reserve_health = 0u16);
            return (ram_word!(game, samus_reserve_health) == 0u16) as u8;
        }
    }
    (ram_word!(game, samus_reserve_health) == 0u16) as u8
}
pub unsafe fn GameState_19_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    let mut i: i32 = 255i32;
    while i >= 0i32 {
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[(i + 384i32) as usize] =
            ram_word!(game, palette_buffer[i]);
        i -= 1;
    }
    let mut j: i32 = 382i32;
    while (j & 0x8000i32) == 0i32 {
        set_ram_word!(game.target_palettes[(j >> 1i32)] = 0u16);
        j -= 2i32;
    }
    let mut k: i32 = 94i32;
    while (k & 0x8000i32) == 0i32 {
        set_ram_word!(game.target_palettes[((k >> 1i32) + 208i32)] = 0u16);
        k -= 2i32;
    }
    let mut m: i32 = 30i32;
    while (m & 0x8000i32) == 0i32 {
        set_ram_word!(
            game.target_palettes[((m >> 1i32) + 192i32)] =
                ram_word!(game, palette_buffer[((m >> 1i32) + 192i32)])
        );
        m -= 2i32;
    }
    set_ram_word!(game.game_options_screen_index = 3u16);
    set_ram_words! { game, 0u16;
        g_word_7E0DE4, g_word_7E0DE6, g_word_7E0DE8, hud_item_index,
        samus_auto_cancel_hud_item_index, samus_invincibility_timer, samus_knockback_timer,
    }
    update_ram_word!(game.game_state, wrapping_add(1));
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_20_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    set_ram_word!(game.palette_change_denom = 6u16);
    if AdvancePaletteFadeForAllPalettes(game) != 0 {
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        set_ram_words! { game, 0u16; fx_layer_blending_config_a, cur_irq_handler }
        *ram_ptr!(game; 0x6e) = 0u8;
        *ram_ptr!(game; 0x71) = 0u8;
        *ram_ptr!(game; 0x69) = 16u8;
        *ram_ptr!(game; 0x6b) = 0u8;
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
        *ram_ptr!(game; 0x55) = 9u8;
        set_ram_words! { game, 0u16;
            game_options_screen_index, screen_fade_delay, screen_fade_counter,
        }
        let mut i: i32 = 254i32;
        while i >= 0i32 {
            set_ram_word!(game.eproj_y_subpos[((i >> 1i32) + 15i32)] = 0u16);
            i -= 2i32;
        }
        set_ram_word!(game.g_word_7E0DE8 = 16u16);
        set_ram_word!(game.game_options_screen_index = 3u16);
        set_ram_words! { game, 0u16; g_word_7E0DE4, g_word_7E0DE6 }
        update_ram_word!(game.game_state, wrapping_add(1));
        set_ram_word!(game.power_bomb_explosion_status = 0u16);
        QueueSfx1_Max15(2u16, game);
        QueueSfx2_Max15(0x71u16, game);
        QueueSfx3_Max15(1u16, game);
        QueueMusic_Delayed8(0u16, game);
        QueueMusic_Delayed8(0xff39u16, game);
        QueueMusic_DelayedY(5u16, 0xeu16, game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_21_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    Samus_DrawWhenNotAnimatingOrDying(game);
    if HasQueuedMusic(game) == 0 {
        StartSamusDeathAnimation(game);
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_22_SamusNoHealth_Dying(game: &mut Game) -> CoroutineRet {
    DrawSamusStartingDeathAnim_(game);
    let v0: bool =
        (update_ram_word!(game.g_word_7E0DE8, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, g_word_7E0DE8) == 0 || v0 {
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_23_SamusNoHealth_Flashing(game: &mut Game) -> CoroutineRet {
    if HandleSamusDeathSequence(game) != 0 {
        update_ram_word!(game.game_state, wrapping_add(1));
    } else {
        Samus_DrawDuringDeathAnim(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion(game: &mut Game) -> CoroutineRet {
    if GameState_24_SamusNoHealth_Explosion_Helper(game) != 0 {
        set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_state, wrapping_add(1));
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_25_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_state, wrapping_add(1));
        set_ram_words! { game, 0u16; menu_index, debug_disable_sounds }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn LoaadDesinationRoomCreBitset(game: &mut Game) {
    let destination_room = (*get_DoorDef(game.door_def_ptr(), game)).room_definition_ptr;
    game.set_previous_cre_bitset(game.cre_bitset());
    let room = get_RoomDefHeader(destination_room, game);
    game.set_cre_bitset(u16::from((*room).cre_bitset_));
}
pub unsafe fn LoadDoorHeader(game: &mut Game) {
    let door = get_DoorDef(game.door_def_ptr(), game);
    game.set_room_ptr((*door).room_definition_ptr);
    // The properties word includes the adjacent orientation byte.
    let properties = u16::from_le_bytes([(*door).door_bitflags, (*door).door_orientation]);
    game.set_elevator_door_properties_orientation(properties);
    game.set_elevator_flags(game.elevator_door_properties_orientation() & 0x0080);
    game.set_door_direction(u16::from((*door).door_orientation));
    game.set_door_destination_x_pos(u16::from((*door).x_pos_in_room) << 8);
    game.set_door_destination_y_pos(u16::from((*door).y_pos_in_room) << 8);
    let mut distance = (*door).samus_distance_from_door as i16;
    if distance < 0 {
        distance = if game.door_direction() & 2 != 0 {
            384
        } else {
            200
        };
    }
    let speed = (i32::from(distance) << 8) as u32;
    game.set_samus_door_transition_subspeed(speed as u16);
    game.set_samus_door_transition_speed((speed >> 16) as u16);
}
pub unsafe fn LoadRoomHeader(game: &mut Game) {
    let room = get_RoomDefHeader(game.room_ptr(), game);
    game.set_room_index(u16::from((*room).semiunique_room_number));
    game.set_area_index(u16::from((*room).area_index_));
    game.set_room_x_coordinate_on_map(u16::from((*room).x_coordinate_on_map));
    game.set_room_y_coordinate_on_map(u16::from((*room).y_coordinate_on_map));
    game.set_room_width_in_scrolls(u16::from((*room).width));
    game.set_room_width_in_blocks(game.room_width_in_scrolls().wrapping_mul(16));
    game.set_room_height_in_scrolls(u16::from((*room).height));
    game.set_room_height_in_blocks(game.room_height_in_scrolls().wrapping_mul(16));
    game.set_up_scroller(u16::from((*room).up_scroller_));
    game.set_down_scroller(u16::from((*room).down_scroller_));
    game.set_door_list_pointer((*room).ptr_to_doorout);
    HandleRoomDefStateSelect(game.room_ptr(), game);
    let product = Mult8x8(
        game.room_width_in_blocks() as u8,
        game.room_height_in_blocks() as u8,
    );
    game.set_room_size_in_blocks(product.wrapping_mul(2));
}
pub unsafe fn LoadStateHeader(game: &mut Game) {
    let room_state = get_RoomDefRoomstate(game.roomdefroomstate_ptr(), game);
    let graphics_set = usize::from((*room_state).graphics_set);
    let tileset_ptr = game.read_rom_word(0x8fe7a7, 2 * graphics_set);
    let tileset = get_TileSet(tileset_ptr, game);
    game.set_tileset_tile_table_pointer((*tileset).tile_table_ptr);
    game.set_tileset_tiles_pointer((*tileset).tiles_ptr);
    game.set_tileset_compr_palette_ptr((*tileset).palette_ptr);
    game.set_room_compr_level_data_ptr((*room_state).compressed_room_map_ptr);
    game.set_room_music_data_index(u16::from((*room_state).music_track));
    game.set_room_music_track_index(u16::from((*room_state).music_control));
    game.set_room_layer3_asm_ptr((*room_state).room_layer3_fx_ptr);
    game.set_room_enemy_population_ptr((*room_state).enemy_population_ptr_);
    game.set_room_enemy_tilesets_ptr((*room_state).enemy_tilesets_ptr);
    let [scroll_x, scroll_y] = (*room_state).vertical_screen_nudge_limit.to_le_bytes();
    game.set_layer2_scroll_x_and_y(scroll_x, scroll_y);
    game.set_room_main_code_ptr((*room_state).main_code_ptr);
}
pub unsafe fn WaitUntilEndOfVblankAndEnableIrq(game: &mut Game) {
    if (*ram_ptr!(game; 0x84) as i32 & 0x30i32) != 0x30i32 {
        EnableIrqInterruptsNow(game);
    }
}
pub fn PointlessFunctionStupidToo() {}
pub unsafe fn SaveMapExploredifElevator(game: &mut Game) {
    if game.elevator_door_properties_orientation() & 0x000f != 0 {
        SetElevatorsAsUsed(game);
    }
    let door = get_DoorDef(game.door_def_ptr(), game);
    if (*door).door_bitflags & 0x40 != 0 {
        SaveExploredMapTilesToSaved(game);
    }
}
pub unsafe fn LoadMapExploredIfElevator(game: &mut Game) {
    let door = get_DoorDef(game.door_def_ptr(), game);
    if (*door).door_bitflags & 0x40 != 0 {
        LoadMirrorOfExploredMapTiles(game);
    }
}
pub unsafe fn EnsureSamusDrawnEachFrame(game: &mut Game) {
    if ram_word!(game, elevator_flags) == 0 {
        Samus_DrawWhenNotAnimatingOrDying(game);
    }
}
pub unsafe fn LoadEnemyGfxToVram(game: &mut Game) {
    let mut ED: *mut EnemyDef = null_mut::<EnemyDef>();
    let mut dst: u16 = 0x7000u16;
    let mut v0: u16 = ram_word!(game, room_enemy_tilesets_ptr);
    if ram_word!(game, room_enemy_tilesets_ptr) != 0 {
        let mut i: i32 = ram_word!(game, room_enemy_tilesets_ptr) as i32;
        loop {
            let enemy_def: u16 = (*get_EnemyTileset(v0, game)).enemy_def;
            if enemy_def == 0xffffu16 {
                break;
            }
            ED = get_EnemyDef_A2(enemy_def, game);
            let mut vram_update_size: u16 = 0;
            let mut vram_update_dst: u16 = 0;
            if ((*ED).tile_data_size as i32 & 0x8000i32) != 0i32 {
                vram_update_size = ((*ED).tile_data_size) & (0x7fffu16);
                vram_update_dst =
                    ((((*get_EnemyTileset(i as u16, game)).vram_dst) & (0xf000u16)) as i32 >> 4i32
                        | 0x6000i32) as u16;
            } else {
                vram_update_size = (*ED).tile_data_size;
                vram_update_dst = dst;
                dst = (dst as i32 + ((*ED).tile_data_size as i32 >> 1i32)) as u16;
            }
            CopyToVramNow(
                vram_update_dst,
                Load24(&raw mut (*ED).tile_data),
                vram_update_size,
                game,
            );
            i += 4i32;
            v0 = i as u16;
        }
    }
}
pub unsafe fn LoadRoomMusic(game: &mut Game) {
    if (ram_word!(game, game_state) as i32) < kGameState_40_TransitionToDemo as i32
        && ram_word!(game, room_music_data_index) != 0u16
        && ram_word!(game, room_music_data_index) != ram_word!(game, music_data_index)
    {
        QueueMusic_Delayed8(0u16, game);
        QueueMusic_Delayed8((ram_word!(game, room_music_data_index)) | (0xff00u16), game);
    }
}
pub unsafe fn UpdateMusicTrackIndex(game: &mut Game) {
    if (ram_word!(game, game_state) as i32) < kGameState_40_TransitionToDemo as i32
        && ram_word!(game, room_music_track_index) != 0u16
    {
        let mut r18: u16 = ((ram_word!(game, room_music_data_index) as i32) << 8i32) as u16;
        r18 = (r18) | (ram_word!(game, room_music_track_index));
        let mut r20: u16 = ((ram_word!(game, music_data_index) as i32) << 8i32) as u16;
        r20 = (r20) | (ram_word!(game, music_track_index));
        if r18 != r20 {
            set_ram_word!(game.music_track_index = ram_word!(game, room_music_track_index));
        }
    }
}
pub unsafe fn LoadNewMusicTrackIfChanged(game: &mut Game) {
    if ram_word!(game, game_state) < 0x28u16 && ram_word!(game, room_music_track_index) != 0u16 {
        let mut r18: u16 = ((ram_word!(game, room_music_data_index) as i32) << 8i32) as u16;
        r18 = (r18) | (ram_word!(game, room_music_track_index));
        let mut r20: u16 = ((ram_word!(game, music_data_index) as i32) << 8i32) as u16;
        r20 = (r20) | (ram_word!(game, music_track_index));
        if r18 != r20 {
            QueueMusic_DelayedY(ram_word!(game, room_music_track_index), 6u16, game);
        }
    }
}
pub fn NullFunc() {}
pub unsafe fn PlayRoomMusicTrackAfterAFrames(a: u16, game: &mut Game) {
    if (ram_word!(game, game_state) as i32) < kGameState_40_TransitionToDemo as i32 {
        QueueMusic_DelayedY(0u16, a, game);
        QueueMusic_Delayed8(ram_word!(game, music_track_index), game);
    }
}
pub unsafe fn LoadColorsForSpritesBeamsAndEnemies(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(
            game.target_palettes[((i >> 1i32) + 208i32)] =
                crate::rtl::rom_table_word!(game, 0x9a8000, ((i >> 1i32) + 208i32) as isize)
        );
        i -= 2i32;
    }
    let mut j: u16 = 30u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        *(ram_ptr!(game, u16; 0xc200)).offset(((j as i32 >> 1i32) + 224i32) as isize) =
            *(ram_ptr!(game, u16; 0xc000)).offset(((j as i32 >> 1i32) + 224i32) as isize);
        j = (j).wrapping_sub(2u16);
    }
    let mut k: u16 = 30u16;
    while (k as i32 & 0x8000i32) == 0i32 {
        *(ram_ptr!(game, u16; 0xc200)).offset(((k as i32 >> 1i32) + 128i32) as isize) =
            crate::rtl::rom_table_word!(game, 0x9afc00, (k as i32 >> 1i32) as isize);
        k = (k).wrapping_sub(2u16);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallDoorTransitionFunction_Async(ea: u32, game: &mut Game) -> CoroutineRet => ea;
        call (game); default Unreachable(game) as CoroutineRet;
    {
        8577405 => DoorTransitionFunction_HandleElevator;
        8577439 => DoorTransitionFunction_Wait48frames;
        8577694 => DoorTransitionFunction_WaitForSoundsToFinish;
        8577755 => DoorTransitionFunction_FadeOutScreen;
        8577783 => DoorTransitionFunction_LoadDoorHeaderEtc;
        8577808 => DoorTransitionFunction_ScrollScreenToAlignment;
        8577875 => DoorTransitionFunction_FixDoorsMovingUp;
        8577902 => DoorTransitionFunction_SetupNewRoom;
        8577934 => DoorTransitionFunction_SetupScrolling;
        8577984 => DoorTransitionFunction_PlaceSamusLoadTiles;
        8578217 => DoorTransitionFunction_LoadMoreThings_Async;
        8578649 => DoorTransition_C_HandleAnimTiles; 8578660 => DoorTransition_WaitForMusicToClear;
        8578722 => DoorTransition_HandleTransition; 8578871 => DoorTransition_FadeInScreenAndFinish;
    }
}
pub unsafe fn GameState_9_HitDoorBlock(game: &mut Game) -> CoroutineRet {
    CallDoorTransitionFunction_Async(u32::from(game.door_transition_function()) | 0x820000, game)
}
pub unsafe fn DoorTransitionFunction_HandleElevator(game: &mut Game) -> CoroutineRet {
    if ram_word!(game, elevator_flags) == 0 {
        update_ram_word!(game.game_state, wrapping_add(1));
        return GameState_10_LoadingNextRoom_Async(game);
    }
    CallSomeSamusCode(0u16, game);
    if (ram_word!(game, elevator_direction) as i32 & 0x8000i32) != 0i32 {
        update_ram_word!(game.game_state, wrapping_add(1));
        return GameState_10_LoadingNextRoom_Async(game);
    }
    set_ram_word!(game.downwards_elevator_delay_timer = 48u16);
    set_ram_word!(game.door_transition_function = ((0x82e19fi32) as u16));
    DoorTransitionFunction_Wait48frames(game)
}
pub unsafe fn DoorTransitionFunction_Wait48frames(game: &mut Game) -> CoroutineRet {
    if (update_ram_word!(game.downwards_elevator_delay_timer, wrapping_sub(1), reread) as i32
        & 0x8000i32)
        != 0i32
    {
        update_ram_word!(game.game_state, wrapping_add(1));
        return GameState_10_LoadingNextRoom_Async(game);
    } else {
        DetermineWhichEnemiesToProcess(game);
        EnemyMain(game);
        DrawSamusEnemiesAndProjectiles(game);
        EnsureSamusDrawnEachFrame(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_10_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    set_ram_words! { game, 1u16;
        door_transition_flag_enemies, door_transition_flag_elevator_zebetites,
    }
    set_ram_words! { game, 0u16; debug_disable_minimap, save_station_lockout_flag }
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    LoaadDesinationRoomCreBitset(game);
    let mut i: i32 = 254i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_words! { game, 0u16; target_palettes[v1], target_palettes[(v1 + 128i32)] }
        i -= 2i32;
    }
    set_ram_word!(game.target_palettes[9] = ram_word!(game, palette_buffer[9]));
    set_ram_word!(game.target_palettes[10] = ram_word!(game, palette_buffer[10]));
    set_ram_word!(game.target_palettes[13] = ram_word!(game, palette_buffer[13]));
    set_ram_word!(game.target_palettes[14] = ram_word!(game, palette_buffer[14]));
    set_ram_word!(game.target_palettes[17] = ram_word!(game, palette_buffer[17]));
    set_ram_word!(game.target_palettes[18] = ram_word!(game, palette_buffer[18]));
    set_ram_word!(game.target_palettes[19] = ram_word!(game, palette_buffer[19]));
    set_ram_word!(game.target_palettes[29] = ram_word!(game, palette_buffer[29]));
    if ((ram_word!(game, previous_cre_bitset) as i32 | ram_word!(game, cre_bitset) as i32) & 1i32)
        == 0i32
    {
        set_ram_word!(game.target_palettes[20] = ram_word!(game, palette_buffer[20]));
        set_ram_word!(game.target_palettes[21] = ram_word!(game, palette_buffer[21]));
        set_ram_word!(game.target_palettes[22] = ram_word!(game, palette_buffer[22]));
        set_ram_word!(game.target_palettes[23] = ram_word!(game, palette_buffer[23]));
        set_ram_word!(game.target_palettes[28] = ram_word!(game, palette_buffer[28]));
        if ram_word!(game, timer_status) != 0 {
            set_ram_word!(game.target_palettes[209] = ram_word!(game, palette_buffer[209]));
            set_ram_word!(game.target_palettes[210] = ram_word!(game, palette_buffer[210]));
            set_ram_word!(game.target_palettes[212] = ram_word!(game, palette_buffer[212]));
            set_ram_word!(game.target_palettes[221] = ram_word!(game, palette_buffer[221]));
            DrawTimer(game);
        }
    }
    ClearSoundsWhenGoingThroughDoor(game);
    QueueSfx2_Max15(0x71u16, game);
    set_ram_word!(game.debug_disable_sounds = -1i32 as u16);
    set_ram_word!(game.door_transition_function = ((0x82e29ei32) as u16));
    update_ram_word!(game.game_state, wrapping_add(1));
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_11_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    let dispatch_result = CallDoorTransitionFunction_Async(
        u32::from(game.door_transition_function()) | 0x820000,
        game,
    );
    if dispatch_result != 0 {
        return dispatch_result;
    }
    if game.timer_status() != 0 {
        DrawTimer(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_WaitForSoundsToFinish(game: &mut Game) -> CoroutineRet {
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    if game.sfx_queue_is_empty(0) && game.sfx_queue_is_empty(1) && game.sfx_queue_is_empty(2) {
        game.set_door_transition_function(0xe2db);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_FadeOutScreen(game: &mut Game) -> CoroutineRet {
    if AdvancePaletteFadeForAllPalettes_0xc(game) != 0 {
        game.set_door_transition_function(0xe2f7);
    } else {
        DetermineWhichEnemiesToProcess(game);
        EnemyMain(game);
        DrawSamusEnemiesAndProjectiles(game);
        EnsureSamusDrawnEachFrame(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_LoadDoorHeaderEtc(game: &mut Game) -> CoroutineRet {
    LoadDoorHeader(game);
    sub_8882AC(game);
    game.set_hdma_objects_enable_flag(game.hdma_objects_enable_flag() & 0x7fff);
    game.set_irqhandler_next_handler(8);
    game.set_door_transition_function(0xe310);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_ScrollScreenToAlignment(game: &mut Game) -> CoroutineRet {
    let align_x = game.door_direction() & 2 != 0;
    let position = if align_x {
        game.layer1_x_pos()
    } else {
        game.layer1_y_pos()
    };
    let aligned = position as u8 == 0;
    if !aligned {
        let position = if position & 0x80 != 0 {
            position.wrapping_add(1)
        } else {
            position.wrapping_sub(1)
        };
        if align_x {
            game.set_layer1_x_pos(position);
        } else {
            game.set_layer1_y_pos(position);
        }
    }
    CalculateLayer2PosAndScrollsWhenScrolling(game);
    if aligned {
        game.set_door_transition_function(0xe353);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_FixDoorsMovingUp(game: &mut Game) -> CoroutineRet {
    if game.door_direction() & 3 == 3 {
        FixDoorsMovingUp(game);
    }
    game.set_door_transition_function(0xe36e);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_SetupNewRoom(game: &mut Game) -> CoroutineRet {
    SaveMapExploredifElevator(game);
    LoadRoomHeader(game);
    LoadStateHeader(game);
    LoadMapExploredIfElevator(game);
    InitializeSpecialEffectsForNewRoom(game);
    LoadLevelScrollAndCre(game);
    game.set_door_transition_function(0xe38e);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_SetupScrolling(game: &mut Game) -> CoroutineRet {
    game.set_reg_bg2hofs(0);
    game.set_reg_bg2vofs(0);
    game.set_scrolling_finished_hook(0);
    if game.door_direction() & 3 == 2 {
        game.set_reg_bg1vofs(game.reg_bg1vofs().wrapping_add(1));
    }
    if game.door_direction() & 3 != 3 {
        game.set_door_transition_frame_counter(0);
    }
    DoorTransitionScrollingSetup(game);
    game.set_door_transition_function(0xe3c0);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_PlaceSamusLoadTiles(game: &mut Game) -> CoroutineRet {
    game.set_samus_x_pos(
        game.layer1_x_pos()
            .wrapping_add(u16::from(game.samus_x_pos() as u8)),
    );
    game.set_samus_prev_x_pos(game.samus_x_pos());
    game.set_samus_y_pos(
        game.layer1_y_pos()
            .wrapping_add(u16::from(game.samus_y_pos() as u8)),
    );
    game.set_samus_prev_y_pos(game.samus_y_pos());
    game.set_door_transition_flag(0);
    let irq_handler = if game.door_direction() & 3 == 2 {
        16
    } else {
        22
    };
    game.set_irqhandler_next_handler(irq_handler);
    WaitUntilEndOfVblankAndEnableIrq(game);
    if game.cre_bitset() & 2 != 0 && game.door_def_ptr() != addr_kDoorDef_947a as u16 {
        DecompressToMem(0xb98000u32, ram_ptr!(game; 0x7000), game);
    }
    DecompressToMem(game.tileset_tiles_pointer(), ram_ptr!(game; 0x2000), game);
    DecompressToMem(
        game.tileset_compr_palette_ptr(),
        ram_ptr!(game; 0xc200),
        game,
    );
    CopyToVramNow(0u16, 0x7e2000u32, 0x2000u16, game);
    CopyToVramNow(0x1000u16, 0x7e4000u32, 0x2000u16, game);
    CopyToVramNow(0x2000u16, 0x7e6000u32, 0x1000u16, game);
    if game.cre_bitset() & 6 != 0 && game.door_def_ptr() != addr_kDoorDef_947a as u16 {
        CopyToVramNow(0x2800u16, 0x7e7000u32, 0x1000u16, game);
        CopyToVramNow(0x3000u16, 0x7e8000u32, 0x2000u16, game);
        CopyToVramNow(0x4000u16, 0x9ab200u32, 0x1000u16, game);
    }
    if game.door_direction() & 3 == 3 {
        game.set_irqhandler_next_handler(16);
    }
    game.set_door_transition_function(0xe4a9);
    kCoroutineNone as CoroutineRet
}
static kUpdateBackgroundCommands: [Func_Y_Y; 8] = [
    UpdateBackgroundCommand_0_Finish,
    UpdateBackgroundCommand_2_TransferToVram,
    UpdateBackgroundCommand_4_Decompression,
    UpdateBackgroundCommand_6_ClearFxTilemap,
    UpdateBackgroundCommand_8_TransferToVramAndSetBg3,
    UpdateBackgroundCommand_A_ClearBg2Tilemap,
    UpdateBackgroundCommand_C_ClearKraidBg2Tilemap,
    UpdateBackgroundCommand_E_DoorDependentTransferToVRAM,
];
pub unsafe fn DoorTransitionFunction_LoadMoreThings_Async(game: &mut Game) -> CoroutineRet {
    let mut bg_data_ptr: u16 = 0;
    's_188: {
        let mut current_block_45: u64;
        match game.coroutine_state_1() {
            0 => {
                LoadEnemyGfxToVram(game);
                LoadRoomMusic(game);
                ClearEprojs(game);
                ClearAnimtiles(game);
                ClearPaletteFXObjects(game);
                ClearPLMs(game);
                CreatePlmsExecuteDoorAsmRoomSetup(game);
                LoadFXHeader(game);
                SpawnDoorClosingPLM(game);
                UpdateBeamTilesAndPalette(game);
                LoadColorsForSpritesBeamsAndEnemies(game);
                LoadEnemies(game);
                InitializeEnemies(game);
                ResetProjectileData(game);
                Samus_LoadSuitTargetPalette(game);
                ClearFxTilemap(game);
                let fx_tilemap_ptr = u16::from_le_bytes([game.g_ram[0x1964], game.g_ram[0x1965]]);
                if fx_tilemap_ptr != 0 {
                    CopyToVramNow(
                        addr_unk_605BE0 as u16,
                        0x8a0000 | u32::from(fx_tilemap_ptr),
                        2112,
                        game,
                    );
                }
                bg_data_ptr = game.read_rom_word(
                    0x8f0000 | u32::from(game.roomdefroomstate_ptr()),
                    core::mem::offset_of!(RoomDefRoomstate, bg_data_ptr),
                );
                if bg_data_ptr & 0x8000 != 0 {
                    loop {
                        let command = game.read_rom_word(0x8f0000 | u32::from(bg_data_ptr), 0);
                        bg_data_ptr = kUpdateBackgroundCommands[usize::from(command >> 1)](
                            bg_data_ptr.wrapping_add(2),
                            game,
                        );
                        if bg_data_ptr == 0 {
                            break;
                        }
                    }
                }
                current_block_45 = 2370887241019905314;
            }
            1 => {
                current_block_45 = 16864618877125483552;
            }
            2 => {
                current_block_45 = 17993885260445251066;
            }
            _ => {
                current_block_45 = 6417057564578538666;
            }
        }
        loop {
            match current_block_45 {
                6417057564578538666 => {
                    break 's_188;
                }
                2370887241019905314 => {
                    let door_transition_flag =
                        u16::from_le_bytes([game.g_ram[0x931], game.g_ram[0x932]]);
                    if door_transition_flag & 0x8000 == 0 {
                        current_block_45 = 16864618877125483552;
                        continue;
                    }
                    game.g_ram[0xc188..0xc18a].copy_from_slice(&15328u16.to_le_bytes());
                    SpawnBG3ScrollHdmaObject(game);
                    game.set_hdma_objects_enable_flag(game.hdma_objects_enable_flag() | 0x8000);
                    current_block_45 = 17993885260445251066;
                }
                16864618877125483552 => {
                    let mut _coret_: u8 = WaitForNMI_Async(game);
                    if _coret_ != 0 {
                        game.set_coroutine_state_1(1);
                        return _coret_;
                    }
                    current_block_45 = 2370887241019905314;
                }
                _ => {
                    let mut _coret__0: u8 = PlmHandler_Async(game);
                    if _coret__0 != 0 {
                        game.set_coroutine_state_1(2);
                        return _coret__0;
                    }
                    if game.door_direction() & 2 == 0 {
                        if game.door_direction() & 3 != 0 {
                            game.set_samus_x_pos(game.samus_x_pos() & 0xfff8);
                        } else {
                            game.set_samus_x_pos(game.samus_x_pos() | 7);
                        }
                    }
                    game.set_door_transition_function(0xe659);
                    current_block_45 = 6417057564578538666;
                }
            }
        }
    }
    game.set_coroutine_state_1(0);
    0 as CoroutineRet
}
pub unsafe fn ClearFxTilemap(game: &mut Game) {
    for i in (0..960).rev() {
        let offset = i * 2;
        game.g_ram[0x4000 + offset..0x4002 + offset].copy_from_slice(&6222u16.to_le_bytes());
        game.g_ram[0x4780 + offset..0x4782 + offset].copy_from_slice(&6222u16.to_le_bytes());
    }
    CopyToVramNow(0x5880, 0x7e4000, 0xf00, game);
}
pub unsafe fn ClearBg2Tilemap(game: &mut Game) {
    for i in (0..1024).rev() {
        let offset = i * 2;
        game.g_ram[0x4000 + offset..0x4002 + offset].copy_from_slice(&824u16.to_le_bytes());
        game.g_ram[0x4800 + offset..0x4802 + offset].copy_from_slice(&824u16.to_le_bytes());
    }
    CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
}
pub unsafe fn ClearKraidBg2Tilemap(game: &mut Game) {
    let mut i: i32 = 1023i32;
    while i >= 0i32 {
        (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[i as usize] = 824u16;
        (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[(i + 1024i32) as usize] = 824u16;
        i -= 1;
    }
    CopyToVramNow(0x4000u16, 0x7e4000u32, 0x1000u16, game);
    CopyToVramNow(0x4800u16, 0x7e4000u32, 0x1000u16, game);
}
pub fn UpdateBackgroundCommand_0_Finish(y: u16, game: &mut Game) -> u16 {
    0u16
}
pub unsafe fn UpdateBackgroundCommand_E_DoorDependentTransferToVRAM(
    j: u16,
    game: &mut Game,
) -> u16 {
    if ram_word!(game, door_def_ptr) == *(RomPtr_8F(j, game) as *mut u16) {
        UpdateBackgroundCommand_2_TransferToVram((j).wrapping_add(2u16), game)
    } else {
        (j).wrapping_add(9u16)
    }
}
pub unsafe fn UpdateBackgroundCommand_2_TransferToVram(j: u16, game: &mut Game) -> u16 {
    let address = 0x8f0000 | u32::from(j);
    let destination = game.read_rom_word(address, 3);
    let source = u32::from(game.read_rom_word(address, 0))
        | (u32::from(game.read_rom_byte(address, 2)) << 16);
    let size = game.read_rom_word(address, 5);
    CopyToVramNow(destination, source, size, game);
    j.wrapping_add(7)
}
pub unsafe fn UpdateBackgroundCommand_4_Decompression(j: u16, game: &mut Game) -> u16 {
    let address = 0x8f0000 | u32::from(j);
    let source = u32::from(game.read_rom_word(address, 0))
        | (u32::from(game.read_rom_byte(address, 2)) << 16);
    let destination = usize::from(game.read_rom_word(address, 3));
    DecompressToMem(source, game.g_ram.as_mut_ptr().add(destination), game);
    j.wrapping_add(5)
}
pub unsafe fn UpdateBackgroundCommand_6_ClearFxTilemap(j: u16, game: &mut Game) -> u16 {
    ClearFxTilemap(game);
    j
}
pub unsafe fn UpdateBackgroundCommand_8_TransferToVramAndSetBg3(
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = UpdateBackgroundCommand_2_TransferToVram(j, game);
    *ram_ptr!(game; 0x5e) = 2u8;
    j
}
pub unsafe fn UpdateBackgroundCommand_A_ClearBg2Tilemap(j: u16, game: &mut Game) -> u16 {
    ClearBg2Tilemap(game);
    j
}
pub unsafe fn UpdateBackgroundCommand_C_ClearKraidBg2Tilemap(j: u16, game: &mut Game) -> u16 {
    ClearKraidBg2Tilemap(game);
    j
}
pub unsafe fn DoorTransition_C_HandleAnimTiles(game: &mut Game) -> CoroutineRet {
    AnimtilesHandler(game);
    game.set_door_transition_function(fnDoorTransition_WaitForMusicToClear as u16);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransition_WaitForMusicToClear(game: &mut Game) -> CoroutineRet {
    if HasQueuedMusic(game) == 0 {
        game.set_door_transition_function(fnDoorTransition_HandleTransition as u16);
        LoadNewMusicTrackIfChanged(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransition_HandleTransition(game: &mut Game) -> CoroutineRet {
    let x = game.samus_x_pos();
    match x & 0x00f0 {
        0x10 => game.set_samus_x_pos((x | 0x000f).wrapping_add(8)),
        0xe0 => game.set_samus_x_pos((x & 0xfff0).wrapping_sub(8)),
        _ => {}
    }
    let y = game.samus_y_pos();
    if y & 0x00f0 == 0x10 {
        game.set_samus_y_pos((y | 0x000f).wrapping_add(8));
    }
    for offset in (0..512).step_by(2).rev() {
        game.g_ram[0x9c00 + offset..0x9c02 + offset].copy_from_slice(&[0, 0]);
        game.g_ram[0x9e00 + offset..0x9e02 + offset].copy_from_slice(&[0, 0]);
    }
    let room_irq = game.room_loading_irq_handler();
    game.set_irqhandler_next_handler(if room_irq == 0 { 4 } else { room_irq });
    PointlessFunctionStupidToo();
    if game.elevator_flags() != 0 {
        let samus_command = if game.elevator_direction() & 0x8000 == 0 {
            7
        } else {
            0
        };
        CallSomeSamusCode(samus_command, game);
    }
    SetLiquidPhysicsType(game);
    game.set_door_transition_function(fnDoorTransition_FadeInScreenAndFinish as u16);
    let inidisp_obsel = u16::from_le_bytes([game.g_ram[0x51], game.g_ram[0x52]]);
    game.g_ram[0x51..0x53].copy_from_slice(&(inidisp_obsel | 0x001f).to_le_bytes());
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransition_FadeInScreenAndFinish(game: &mut Game) -> CoroutineRet {
    AnimtilesHandler(game);
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    EprojRunAll(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    QueueEnemyBG2TilemapTransfers(game);
    if AdvancePaletteFadeForAllPalettes_0xc(game) != 0 {
        game.set_debug_disable_sounds(0);
        PlaySpinJumpSoundIfSpinJumping(game);
        game.set_door_transition_flag_elevator_zebetites(0);
        game.set_door_transition_flag_enemies(0);
        game.set_game_state(kGameState_8_MainGameplay as u16);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn LoadDestinationRoomThings(game: &mut Game) {
    LoaadDesinationRoomCreBitset(game);
    LoadDoorHeader(game);
    LoadRoomHeader(game);
    LoadStateHeader(game);
    LoadCRETilesTilesetTilesAndPalette(game);
}
pub unsafe fn LoadCRETilesTilesetTilesAndPalette(game: &mut Game) {
    set_ram_word!(game.elevator_flags = 0u16);
    crate::rtl::register_writes! { game;
        word(VMAIN as u16, 0x80u16); word(VMADDL as u16, (addr_unk_605000 as i32 >> 1i32) as u16);
    }
    DecompressToVRAM(0xb98000u32, addr_unk_605000 as u16, game);
    WriteRegWord(VMADDL as u16, 0u16, game);
    DecompressToVRAM(Load24(ram_ptr!(game, LongPtr; 0x7c3)), 0u16, game);
    DecompressToMem(
        Load24(ram_ptr!(game, LongPtr; 0x7c6)),
        ram_ptr!(game; 0xc200),
        game,
    );
}
pub unsafe fn LoadLevelDataAndOtherThings(game: &mut Game) {
    let mut i: i32 = 25598i32;
    while i >= 0i32 {
        set_ram_word!(game.level_data[(i >> 1i32)] = 0x8000u16);
        i -= 2i32;
    }
    DecompressToMem(
        Load24(ram_ptr!(game, LongPtr; 0x7bd)),
        ram_ptr!(game, u16; 0x10000) as *mut u8,
        game,
    );
    let size: u16 = ram_word!(game, ram7F_start);
    memcpy(
        ram_ptr!(game, u16; 0x19602) as *mut c_void,
        (ram_ptr!(game, u16; 0x10002) as *mut u8)
            .offset(size as i32 as isize)
            .offset((size as i32 >> 1i32) as isize) as *const c_void,
        size as size_t,
    );
    memcpy(
        ram_ptr!(game, c_void; 0x16402),
        (ram_ptr!(game, u16; 0x10002) as *mut u8).offset(size as i32 as isize) as *const c_void,
        (size as i32 >> 1i32) as size_t,
    );
    if ram_word!(game, area_index) == 6u16 {
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa000),
            game,
        );
    } else {
        DecompressToMem(0xb9a09du32, ram_ptr!(game; 0xa000), game);
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa800),
            game,
        );
    }
    let RD: *mut RoomDefRoomstate =
        get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game);
    let mut rdf_scroll_ptr: i16 = (*RD).rdf_scroll_ptr as i16;
    if rdf_scroll_ptr >= 0i16 {
        let scrollval: u16 = (*RD).rdf_scroll_ptr;
        let r20: u8 = ((ram_word!(game, room_height_in_scrolls)) as u8).wrapping_sub(1u8);
        let mut v8: u8 = 2u8;
        let mut v9: u8 = 0u8;
        let mut v10: i8 = 0i8;
        loop {
            if v10 as i32 == r20 as i32 {
                v8 = ((scrollval) as u8).wrapping_add(1u8);
            }
            let mut v11: i8 = 0i8;
            loop {
                let fresh145 = v9;
                v9 = v9.wrapping_add(1);
                *ram_ptr!(game; 0xcd20, fresh145 as isize) = v8;
                v11 += 1;
                if (v11 as i32 == ram_word!(game, room_width_in_scrolls) as u8 as i32) {
                    break;
                }
            }
            v10 += 1;
            if (v10 as i32 == ram_word!(game, room_height_in_scrolls) as u8 as i32) {
                break;
            }
        }
    } else {
        let mut m: u16 = 0u16;
        while m != 50u16 {
            *(ram_ptr!(game, u16; 0xcd20, m as isize)) =
                *(RomPtr_8F(rdf_scroll_ptr as u16, game) as *mut u16);
            rdf_scroll_ptr = (rdf_scroll_ptr).wrapping_add(2i16);
            m = (m).wrapping_add(2u16);
        }
    }
    if (*RD).room_plm_header_ptr != 0 {
        let mut n: u16 = (*RD).room_plm_header_ptr;
        while (*get_RoomPlmEntry(n, game)).plm_header_ptr_ != 0 {
            SpawnRoomPLM(n, game);
            n = (n).wrapping_add(6u16);
        }
    }
    RunDoorSetupCode(game);
    RunRoomSetupCode(game);
    if ram_word!(game, elevator_flags) != 0 {
        set_ram_word!(game.elevator_status = 2u16);
    }
}
pub unsafe fn SpawnDoorClosingPLM(game: &mut Game) {
    if CheckIfColoredDoorCapSpawned(game) == 0 {
        let v0: *const u16 = RomPtr_8F(
            (2i32 * ram_word!(game, door_direction) as i32 + addr_kDoorClosingPlmIds as i32) as u16,
            game,
        ) as *const u16;
        if *v0 != 0 {
            let rp: *mut RoomPlmEntry = ram_ptr!(game, RoomPlmEntry; 0x12);
            (*rp).plm_header_ptr_ = *v0;
            *(&raw mut (*rp).x_block as *mut u16) = *(&raw mut (*(get_DoorDef
                as unsafe fn(u16, &mut Game) -> *mut DoorDef)(
                ram_word!(game, door_def_ptr),
                game,
            ))
            .x_pos_plm as *mut u16);
            (*rp).plm_room_argument = 0u16;
            SpawnRoomPLM(0x12u16, game);
        }
    }
}
pub unsafe fn CheckIfColoredDoorCapSpawned(game: &mut Game) -> u8 {
    let DD: *mut DoorDef = get_DoorDef(ram_word!(game, door_def_ptr), game);
    let v2: u16 = (2i32
        * (Mult8x8((*DD).y_pos_plm, ram_word!(game, room_width_in_blocks) as u8) as i32
            + (*DD).x_pos_plm as i32)) as u16;
    let mut v3: u16 = 78u16;
    while v2 != ram_word!(game, plm_block_indices[(v3 as i32 >> 1i32)]) {
        v3 = (v3).wrapping_sub(2u16);
        if (v3 as i32 & 0x8000i32) != 0i32 {
            return 0u8;
        }
    }
    let v5: i32 = v3 as i32 >> 1i32;
    if ram_word!(game, plm_header_ptr[v5]) == 0 {
        return 0u8;
    }
    let v6: i16 = ram_word!(game, plm_room_arguments[v5]) as i16;
    if v6 >= 0i16 {
        let idx: i32 = PrepareBitAccess(v6 as u16, game) as i32;
        if (ram_word!(game, bitmask) as i32 & *ram_ptr!(game; 0xd8b0, idx as isize) as i32) != 0i32
        {
            return 0u8;
        }
    }
    set_ram_word!(game.plm_instruction_timer[v5] = 1u16);
    let v8: u16 = ram_word!(game, plm_header_ptr[v5]);
    if v8 == 0 {
        return 0u8;
    }
    set_ram_word!(game.plm_instr_list_ptrs[v5] = (*get_PlmHeader_Size6(v8, game)).instr_list_2_ptr);
    1u8
}
static kLoadLibraryBackgroundFuncs: [Option<Func_Y_Y>; 8] = callback_table! { unsafe Func_Y_Y;
    LoadLibraryBackgroundFunc_0_DONE, LoadLibraryBackgroundFunc_2_TransferToVram,
    LoadLibraryBackgroundFunc_4_Decompress, LoadLibraryBackgroundFunc_6_ClearFxTilemap,
    LoadLibraryBackgroundFunc_8_TransferToVramAndSetBg3,
    LoadLibraryBackgroundFunc_A_ClearBG2Tilemap, LoadLibraryBackgroundFunc_C_ClearKraidLayer2,
    LoadLibraryBackgroundFunc_E_DoorDependentTransferToVram,
};
pub unsafe fn LoadLibraryBackground(game: &mut Game) {
    ClearFXTilemap(game);
    if ram_word!(game, fx_tilemap_ptr) != 0 {
        crate::rtl::register_writes! { game;
            word(A1T1L as u16, ram_word!(game, fx_tilemap_ptr)); word(VMADDL as u16, addr_unk_605BE0 as u16);
            word(DMAP1 as u16, 0x1801u16); word(A1B1 as u16, 0x8au16);
            word(DAS1L as u16, 0x840u16); byte(VMAIN as u16, 0x80u8);
            byte(MDMAEN as u16, 2u8);
        }
    }
    let mut bg_data_ptr: u16 =
        (*get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game)).bg_data_ptr;
    if bg_data_ptr as i32 & 0x8000i32 != 0 {
        loop {
            let v1: u16 = *(RomPtr_8F(bg_data_ptr, game) as *mut u16);
            bg_data_ptr = kLoadLibraryBackgroundFuncs[(v1 as i32 >> 1i32) as usize]
                .expect("non-null function pointer")(
                (bg_data_ptr).wrapping_add(2u16), game
            );
            if (bg_data_ptr == 0) {
                break;
            }
        }
    }
}
pub fn LoadLibraryBackgroundFunc_0_DONE(j: u16, game: &mut Game) -> u16 {
    0u16
}
pub unsafe fn LoadLibraryBackgroundFunc_E_DoorDependentTransferToVram(
    j: u16,
    game: &mut Game,
) -> u16 {
    if ram_word!(game, door_def_ptr) == (*get_LoadBg_E(j, game)).field_0 {
        LoadLibraryBackgroundFunc_2_TransferToVram((j).wrapping_add(2u16), game)
    } else {
        (j).wrapping_add(9u16)
    }
}
pub unsafe fn LoadLibraryBackgroundFunc_2_TransferToVram(j: u16, game: &mut Game) -> u16 {
    let LoadBg_28: *mut u16 = RomPtr_8F(j, game) as *mut u16;
    crate::rtl::register_writes! { game;
        word(VMADDL as u16,
        *((LoadBg_28 as *mut u8).offset(3) as *mut u16),); word(DMAP1 as u16, 0x1801u16);
        word(A1T1L as u16, *LoadBg_28); word(A1B1 as u16, *LoadBg_28.offset(1));
        word(DAS1L as u16,
        *((LoadBg_28 as *mut u8).offset(5) as *mut u16),); byte(VMAIN as u16, 0x80u8);
        byte(MDMAEN as u16, 2u8);
    }
    (j).wrapping_add(7u16)
}
pub unsafe fn LoadLibraryBackgroundFunc_4_Decompress(j: u16, game: &mut Game) -> u16 {
    let p: *const u8 = RomPtr_8F(j, game);
    DecompressToMem(
        Load24(p as *mut LongPtr),
        ram_ptr!(game; *(p.offset(3) as *mut u16) as i32 as isize),
        game,
    );
    (j).wrapping_add(5u16)
}
pub unsafe fn LoadLibraryBackgroundFunc_6_ClearFxTilemap(j: u16, game: &mut Game) -> u16 {
    ClearFXTilemap(game);
    j
}
pub unsafe fn LoadLibraryBackgroundFunc_A_ClearBG2Tilemap(j: u16, game: &mut Game) -> u16 {
    ClearBG2Tilemap(game);
    j
}
pub unsafe fn LoadLibraryBackgroundFunc_C_ClearKraidLayer2(j: u16, game: &mut Game) -> u16 {
    ClearBG2Tilemap(game);
    j
}
pub unsafe fn LoadLibraryBackgroundFunc_8_TransferToVramAndSetBg3(
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = LoadLibraryBackgroundFunc_2_TransferToVram(j, game);
    *ram_ptr!(game; 0x5e) = 2u8;
    j
}
pub unsafe fn LoadLevelScrollAndCre(game: &mut Game) {
    let mut i: i32 = 6398i32;
    while i >= 0i32 {
        set_ram_word!(game.level_data[(i >> 1i32)] = 0x8000u16);
        *(ram_ptr!(game, u16; 0x10002)).offset(((i >> 1i32) + 3200i32 * 1i32) as isize) = 0x8000u16;
        *(ram_ptr!(game, u16; 0x10002)).offset(((i >> 1i32) + 3200i32 * 2i32) as isize) = 0x8000u16;
        *(ram_ptr!(game, u16; 0x10002)).offset(((i >> 1i32) + 3200i32 * 3i32) as isize) = 0x8000u16;
        i -= 2i32;
    }
    DecompressToMem(
        Load24(ram_ptr!(game, LongPtr; 0x7bd)),
        ram_ptr!(game, u16; 0x10000) as *mut u8,
        game,
    );
    let size: u16 = ram_word!(game, ram7F_start);
    memcpy(
        ram_ptr!(game, u16; 0x19602) as *mut c_void,
        (ram_ptr!(game, u16; 0x10002) as *mut u8)
            .offset(size as i32 as isize)
            .offset((size as i32 >> 1i32) as isize) as *const c_void,
        size as size_t,
    );
    memcpy(
        ram_ptr!(game, c_void; 0x16402),
        (ram_ptr!(game, u16; 0x10002) as *mut u8).offset(size as i32 as isize) as *const c_void,
        (size as i32 >> 1i32) as size_t,
    );
    if ram_word!(game, area_index) == 6u16 {
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa000),
            game,
        );
    } else {
        if (ram_word!(game, cre_bitset) as i32 & 2i32) != 0i32 {
            DecompressToMem(0xb9a09du32, ram_ptr!(game; 0xa000), game);
        }
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa800),
            game,
        );
    }
    let RD: *mut RoomDefRoomstate =
        get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game);
    let mut rdf_scroll_ptr: i16 = (*RD).rdf_scroll_ptr as i16;
    if rdf_scroll_ptr >= 0i16 {
        let r18: u16 = (*RD).rdf_scroll_ptr;
        let r20: u8 = ((ram_word!(game, room_height_in_scrolls)) as u8).wrapping_sub(1u8);
        let mut v8: u8 = 2u8;
        let mut v9: u8 = 0u8;
        let mut v10: i8 = 0i8;
        loop {
            if v10 as i32 == r20 as i32 {
                v8 = ((r18) as u8).wrapping_add(1u8);
            }
            let mut v12: i8 = v10;
            let mut v11: i8 = 0i8;
            loop {
                let fresh35 = v9;
                v9 = v9.wrapping_add(1);
                *ram_ptr!(game; 0xcd20, fresh35 as isize) = v8;
                v11 += 1;
                if (v11 as i32 == ram_word!(game, room_width_in_scrolls) as u8 as i32) {
                    break;
                }
            }
            v10 = (v12).wrapping_add(1i8);
            if (v12 as i32 + 1i32 == ram_word!(game, room_height_in_scrolls) as u8 as i32) {
                break;
            }
        }
    } else {
        let mut m: u16 = 0u16;
        while m != 50u16 {
            *(ram_ptr!(game, u16; 0xcd20, m as isize)) =
                *(RomPtr_8F(rdf_scroll_ptr as u16, game) as *mut u16);
            rdf_scroll_ptr = (rdf_scroll_ptr).wrapping_add(2i16);
            m = (m).wrapping_add(2u16);
        }
    };
}
pub unsafe fn CreatePlmsExecuteDoorAsmRoomSetup(game: &mut Game) {
    let mut RoomDefRoomstate: *mut RoomDefRoomstate =
        get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game);
    if (*RoomDefRoomstate).room_plm_header_ptr != 0 {
        let mut i: i32 = (*RoomDefRoomstate).room_plm_header_ptr as i32;
        while (*get_RoomPlmEntry(i as u16, game)).plm_header_ptr_ != 0 {
            SpawnRoomPLM(i as u16, game);
            i += 6i32;
        }
    }
    RunDoorSetupCode(game);
    RunRoomSetupCode(game);
    if ram_word!(game, elevator_flags) != 0 {
        set_ram_word!(game.elevator_status = 2u16);
    }
}
static kGameOptionsMenuFuncs: [Option<Func_V>; 13] = callback_table! { unsafe Func_V;
    GameOptionsMenuFunc_0, GameOptionsMenu_1_LoadingOptionsScreen,
    GameOptionsMenu_2_FadeInOptionsScreen, GameOptionsMenu_3_OptionsScreen,
    GameOptionsMenu_4_StartGame, GameOptionsMenu_5_DissolveOutScreen,
    GameOptionsMenu_6_DissolveInScreen, GameOptionsMenu_7_ControllerSettings,
    GameOptionsMenu_8_SpecialSettings, GameOptionsMenu_9_ScrollControllerSettingsDown,
    GameOptionsMenu_A_ScrollControllerSettingsUp, GameOptionsMenu_B_TransitionBackToFileSelect,
    GameOptionsMenu_C_FadeOutOptionsScreenToStart,
};
pub unsafe fn GameState_2_GameOptionsMenu(game: &mut Game) -> CoroutineRet {
    kGameOptionsMenuFuncs[ram_word!(game, game_options_screen_index) as usize]
        .expect("non-null function pointer")(game);
    OptionsMenuFunc1(game);
    DrawOptionsMenuSpritemaps(game);
    if ram_word!(game, game_options_screen_index) as i32 - 2i32 & 0x8000i32 == 0 {
        OptionsMenu_AddToVramQueue(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameOptionsMenuFunc_0(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_options_screen_index, wrapping_add(1));
    } else if *ram_ptr!(game; 0x51) == 14u8 {
        let v0: u16 = *ram_ptr!(game; 0x6b) as u16;
        if (*ram_ptr!(game; 0x6b) as i32 & 4i32) == 0i32 {
            CreateOptionsMenuObject_(v0, addr_stru_82F4D6 as u16, game);
        }
    }
}
pub unsafe fn GameOptionsMenu_1_LoadingOptionsScreen(game: &mut Game) {
    *ram_ptr!(game; 0x5d) = 0u8;
    *ram_ptr!(game; 0x69) = 19u8;
    *ram_ptr!(game; 0x6b) = 0u8;
    *ram_ptr!(game; 0x6c) = 0u8;
    *ram_ptr!(game; 0x6d) = 0u8;
    *ram_ptr!(game; 0x6e) = 0u8;
    *ram_ptr!(game; 0x71) = 0u8;
    *ram_ptr!(game; 0x6f) = 0u8;
    crate::rtl::register_writes! { game;
        mirror(VMADDL as u16, 0, 0x72, true); byte(VMADDH as u16, 0x58u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_82EC3D: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8edc00, 0x800);
    SetupDmaTransfer(&raw const unk_82EC3D as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    set_ram_words! { game, 0u16;
        reg_BG1HOFS, reg_BG1VOFS, reg_BG2HOFS, reg_BG2VOFS, debug_invincibility,
    }
    let mut i: i32 = 510i32;
    while i >= 0i32 {
        set_ram_word!(
            game.palette_buffer[(i >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ee400, (i >> 1i32) as isize)
        );
        i -= 2i32;
    }
    DecompressToMem(0x978df4u32, ram_ptr!(game; 0x1c000), game);
    DecompressToMem(0x978fcdu32, ram_ptr!(game; 0x1c800), game);
    DecompressToMem(0x9791c4u32, ram_ptr!(game; 0x1d000), game);
    DecompressToMem(0x97938du32, ram_ptr!(game; 0x1d800), game);
    DecompressToMem(0x97953au32, ram_ptr!(game; 0x1e000), game);
    let mut j: u16 = 1023u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[j as usize] =
            ram_word!(game, custom_background[(j as i32 + 5375i32)]);
        j = j.wrapping_sub(1);
    }
    // The ROM keeps the final copied tile in A through the object clear.
    let initial_tile = u16::from_le_bytes([game.g_ram[0x3000], game.g_ram[0x3001]]);
    set_ram_word!(game.menu_option_index = 0u16);
    DeleteAllOptionsMenuObjects_(game);
    CreateOptionsMenuObject_(initial_tile, addr_stru_82F4B8 as u16, game);
    // The first initializer leaves its palette $0E00 in A for this call.
    CreateOptionsMenuObject_(0x0e00, addr_stru_82F4C4 as u16, game);
    update_ram_word!(game.game_options_screen_index, wrapping_add(1));
    OptionsMenuFunc4(game);
}
pub unsafe fn GameOptionsMenu_2_FadeInOptionsScreen(game: &mut Game) {
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15u8 {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        update_ram_word!(game.game_options_screen_index, wrapping_add(1));
    }
}
pub unsafe fn OptionsMenu_AddToVramQueue(game: &mut Game) {
    let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 2048u16;
    v0 = (v0).wrapping_add(2u16);
    (*gVramWriteEntry(v0 as i32, game)).size = 12288u16;
    v0 = (v0).wrapping_add(2u16);
    let fresh118 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh118 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = 126u8;
    (*gVramWriteEntry(v0 as i32, game)).size = 20480u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
}
pub unsafe fn OptionsMenuFunc5(a: u16, mut k: u16, mut j: u16, game: &mut Game) {
    loop {
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
            as *mut u8)
            .offset(k as i32 as isize) as *mut u16) = (a as i32
            | *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
                as *mut u8)
                .offset(k as i32 as isize) as *mut u16) as i32
                & 0xe3ffi32) as u16;
        k = (k).wrapping_add(2u16);
        j = (j).wrapping_sub(2u16);
        if (j == 0) {
            break;
        }
    }
}
static kGameOptionsMenuItemFuncs: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    GameOptionsMenuItemFunc_0, GameOptionsMenuItemFunc_2_ToggleJapanese,
    GameOptionsMenuItemFunc_2_ToggleJapanese, GameOptionsMenuItemFunc_4,
    GameOptionsMenuItemFunc_4,
};
pub unsafe fn GameOptionsMenu_3_OptionsScreen(game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if (update_ram_word!(game.menu_option_index, wrapping_sub(1), reread) as i32 & 0x8000i32)
            != 0i32
        {
            set_ram_word!(game.menu_option_index = 4u16);
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if update_ram_word!(game.menu_option_index, wrapping_add(1), reread) == 5u16 {
            set_ram_word!(game.menu_option_index = 0u16);
        }
    }
    if (ram_word!(game, joypad1_newkeys) as i32 & 0x8000i32) != 0i32 {
        set_ram_word!(game.game_options_screen_index = 11u16);
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_A as i32) != 0i32
        || (ram_word!(game, joypad1_newkeys) as i32 & kButton_Start as i32) != 0i32
    {
        QueueSfx1_Max6(0x38u16, game);
        kGameOptionsMenuItemFuncs[ram_word!(game, menu_option_index) as usize]
            .expect("non-null function pointer")(game);
    }
}
pub unsafe fn GameOptionsMenuItemFunc_0(game: &mut Game) {
    if ram_word!(game, enable_debug) != 0u16
        && (ram_word!(game, joypad1_lastkeys) as i32 & kButton_L as i32) == 0i32
        || ram_word!(game, loading_game_state) as i32 == kGameState_5_FileSelectMap as i32
    {
        set_ram_word!(game.game_options_screen_index = 4u16);
    } else {
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.game_options_screen_index = 12u16);
    };
}
pub unsafe fn GameOptionsMenuItemFunc_2_ToggleJapanese(game: &mut Game) {
    set_ram_word!(game.menu_option_index = 0u16);
    set_ram_word!(game.japanese_text_flag = (ram_word!(game, japanese_text_flag) == 0u16) as u16);
    OptionsMenuFunc4(game);
}
pub unsafe fn OptionsMenuFunc4(game: &mut Game) {
    if ram_word!(game, japanese_text_flag) != 0 {
        OptionsMenuFunc5(0x400u16, 0x288u16, 0x18u16, game);
        OptionsMenuFunc5(0x400u16, 0x2c8u16, 0x18u16, game);
        OptionsMenuFunc5(0u16, 0x348u16, 0x32u16, game);
        OptionsMenuFunc5(0u16, 0x388u16, 0x32u16, game);
    } else {
        OptionsMenuFunc5(0u16, 0x288u16, 0x18u16, game);
        OptionsMenuFunc5(0u16, 0x2c8u16, 0x18u16, game);
        OptionsMenuFunc5(0x400u16, 0x348u16, 0x32u16, game);
        OptionsMenuFunc5(0x400u16, 0x388u16, 0x32u16, game);
    };
}
pub unsafe fn GameOptionsMenuItemFunc_4(game: &mut Game) {
    *ram_ptr!(game; 0x57) = 3u8;
    set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
    set_ram_word!(game.game_options_screen_index = 5u16);
}
pub unsafe fn GameOptionsMenu_B_TransitionBackToFileSelect(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.game_state = kGameState_4_FileSelectMenus as u16);
        set_ram_words! { game, 0u16; menu_index, game_options_screen_index }
    }
}
pub unsafe fn GameOptionsMenu_C_FadeOutOptionsScreenToStart(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        set_ram_word!(game.game_options_screen_index = 4u16);
    }
}
pub unsafe fn GameOptionsMenu_4_StartGame(game: &mut Game) {
    set_ram_word!(game.game_options_screen_index = 0u16);
    if ram_word!(game, enable_debug) != 0u16
        && (ram_word!(game, joypad1_lastkeys) as i32 & 0x20i32) == 0i32
    {
        set_ram_word!(game.game_state = kGameState_5_FileSelectMap as u16);
        if ram_word!(game, loading_game_state) as i32 != kLoadingGameState_5_Main as i32 {
            set_ram_word!(game.loading_game_state = kGameState_5_FileSelectMap as u16);
            SaveToSram(ram_word!(game, selected_save_slot), game);
        }
    } else if ram_word!(game, loading_game_state) != 0 {
        set_ram_word!(game.game_state = ram_word!(game, loading_game_state));
        if ram_word!(game, loading_game_state) as i32 == kGameState_34_CeresGoesBoom as i32 {
            set_ram_word!(game.cinematic_function = ((0x8bc11bi32) as u16));
        }
        set_ram_words! { game, 0u16; menu_option_index, game_options_screen_index }
    } else {
        set_ram_word!(game.game_state = kGameState_30_IntroCinematic as u16);
        set_ram_word!(game.cinematic_function = ((0x8ba395i32) as u16));
        set_ram_words! { game, 0u16; menu_option_index, screen_fade_delay, screen_fade_counter }
    };
}
pub unsafe fn GameOptionsMenu_5_DissolveOutScreen(game: &mut Game) {
    let mut v1: u16 = 0;
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x57) != 0xf3u8 {
        let ref mut fresh70 = *ram_ptr!(game; 0x57);
        *fresh70 = (*fresh70).wrapping_add(16u8);
    }
    if *ram_ptr!(game; 0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter, reg_BG1VOFS }
        update_ram_word!(game.game_options_screen_index, wrapping_add(1));
        if ram_word!(game, menu_option_index) != 0 {
            if (ram_word!(game, menu_option_index) as i32 & 4i32) != 0i32 {
                if ram_word!(game, japanese_text_flag) != 0 {
                    let mut i: i32 = 1023i32;
                    while i >= 0i32 {
                        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[i as usize] =
                            ram_word!(game, custom_background[(i + 9471i32)]);
                        i -= 1;
                    }
                } else {
                    let mut j: u16 = 1023u16;
                    while (j as i32 & 0x8000i32) == 0i32 {
                        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[j as usize] =
                            *(ram_ptr!(game, u16; 0x19602)).offset((j as i32 + 8447i32) as isize);
                        j = j.wrapping_sub(1);
                    }
                }
                set_ram_word!(game.menu_option_index = 0u16);
                OptionsMenuFunc7(game);
                set_ram_word!(game.menu_option_index = 1u16);
                OptionsMenuFunc7(game);
                set_ram_word!(game.menu_option_index = 4u16);
                CreateOptionsMenuObject_(4u16, addr_stru_82F4D0 as u16, game);
            } else {
                if ram_word!(game, japanese_text_flag) != 0 {
                    let mut k: u16 = 1023u16;
                    while (k as i32 & 0x8000i32) == 0i32 {
                        v1 = *(ram_ptr!(game, u16; 0x19602)).offset((k as i32 + 7423i32) as isize);
                        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[k as usize] =
                            v1;
                        k = k.wrapping_sub(1);
                    }
                } else {
                    let mut m: u16 = 1023u16;
                    while (m as i32 & 0x8000i32) == 0i32 {
                        v1 = *(ram_ptr!(game, u16; 0x19602)).offset((m as i32 + 6399i32) as isize);
                        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[m as usize] =
                            v1;
                        m = m.wrapping_sub(1);
                    }
                }
                CreateOptionsMenuObject_(v1, addr_stru_82F4CA as u16, game);
                LoadControllerOptionsFromControllerBindings(game);
                OptionsMenuFunc6(game);
            }
        } else {
            let mut n: u16 = 1023u16;
            while (n as i32 & 0x8000i32) == 0i32 {
                (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[n as usize] =
                    ram_word!(game, custom_background[(n as i32 + 5375i32)]);
                n = n.wrapping_sub(1);
            }
            OptionsMenuFunc4(game);
            let v6: u16 = 0u16;
            CreateOptionsMenuObject_(v6, addr_stru_82F4C4 as u16, game);
        }
    }
}
pub unsafe fn GameOptionsMenu_6_DissolveInScreen(game: &mut Game) {
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x57) != 3u8 {
        let ref mut fresh72 = *ram_ptr!(game; 0x57);
        *fresh72 = (*fresh72).wrapping_sub(16u8);
    }
    if *ram_ptr!(game; 0x51) == 15u8 {
        *ram_ptr!(game; 0x57) = 0u8;
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
        if ram_word!(game, menu_option_index) != 0 {
            if (ram_word!(game, menu_option_index) as i32 & 4i32) != 0i32 {
                set_ram_word!(game.game_options_screen_index = 8u16);
            } else {
                set_ram_word!(game.game_options_screen_index = 7u16);
            }
            set_ram_word!(game.menu_option_index = 0u16);
        } else {
            set_ram_word!(game.game_options_screen_index = 3u16);
        }
    }
}
static kGameOptionsMenuSpecialSettings: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
    GameOptionsMenuSpecialSettings_0, GameOptionsMenuSpecialSettings_0,
    GameOptionsMenuSpecialSettings_2,
};
pub unsafe fn GameOptionsMenu_8_SpecialSettings(game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if (update_ram_word!(game.menu_option_index, wrapping_sub(1), reread) as i32 & 0x8000i32)
            != 0i32
        {
            set_ram_word!(game.menu_option_index = 2u16);
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if update_ram_word!(game.menu_option_index, wrapping_add(1), reread) == 3u16 {
            set_ram_word!(game.menu_option_index = 0u16);
        }
    }
    if (ram_word!(game, joypad1_newkeys) as i32 & 0x8000i32) == 0i32 {
        if ram_word!(game, joypad1_newkeys) as i32
            & (kButton_Start as i32 | kButton_Left as i32 | kButton_Right as i32 | kButton_A as i32)
            != 0i32
        {
            QueueSfx1_Max6(0x38u16, game);
            kGameOptionsMenuSpecialSettings[ram_word!(game, menu_option_index) as usize]
                .expect("non-null function pointer")(game);
        }
    } else {
        QueueSfx1_Max6(0x38u16, game);
        set_ram_word!(game.menu_option_index = 0u16);
        GameOptionsMenuItemFunc_4(game);
    };
}
pub unsafe fn GameOptionsMenuSpecialSettings_0(game: &mut Game) {
    let v0: *mut u8 = RomPtr_RAM(
        crate::rtl::rom_table_word!(game, 0x82f0ae, ram_word!(game, menu_option_index) as isize),
        game,
    );
    if *(v0 as *mut u16) != 0 {
        *(v0 as *mut u16) = 0u16;
    } else {
        *(v0 as *mut u16) = 1u16;
    }
    OptionsMenuFunc7(game);
}
pub unsafe fn GameOptionsMenuSpecialSettings_2(game: &mut Game) {
    set_ram_word!(game.menu_option_index = 0u16);
    GameOptionsMenuItemFunc_4(game);
}
static g_word_82F149: [u16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
static g_word_82F151: [u16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
pub unsafe fn OptionsMenuFunc7(game: &mut Game) {
    let v0: u16 = (4u16).wrapping_mul(ram_word!(game, menu_option_index));
    if *(RomPtr_RAM(
        crate::rtl::rom_table_word!(game, 0x82f0ae, ram_word!(game, menu_option_index) as isize),
        game,
    ) as *mut u16)
        != 0
    {
        OptionsMenuFunc5(
            0u16,
            g_word_82F149[(((4u16).wrapping_mul(ram_word!(game, menu_option_index))) as i32 >> 1i32)
                as usize],
            0xcu16,
            game,
        );
        OptionsMenuFunc5(
            0u16,
            g_word_82F149[((v0 as i32 >> 1i32) + 1i32) as usize],
            0xcu16,
            game,
        );
        OptionsMenuFunc5(
            0x400u16,
            g_word_82F151[(v0 as i32 >> 1i32) as usize],
            0xcu16,
            game,
        );
        OptionsMenuFunc5(
            0x400u16,
            g_word_82F151[((v0 as i32 >> 1i32) + 1i32) as usize],
            0xcu16,
            game,
        );
    } else {
        OptionsMenuFunc5(
            0x400u16,
            g_word_82F149[(((4u16).wrapping_mul(ram_word!(game, menu_option_index))) as i32 >> 1i32)
                as usize],
            0xcu16,
            game,
        );
        OptionsMenuFunc5(
            0x400u16,
            g_word_82F149[((v0 as i32 >> 1i32) + 1i32) as usize],
            0xcu16,
            game,
        );
        OptionsMenuFunc5(
            0u16,
            g_word_82F151[(v0 as i32 >> 1i32) as usize],
            0xcu16,
            game,
        );
        OptionsMenuFunc5(
            0u16,
            g_word_82F151[((v0 as i32 >> 1i32) + 1i32) as usize],
            0xcu16,
            game,
        );
    };
}
static word_82F204: [u16; 16] = [
    0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80, 0x80,
    0x80,
];
pub unsafe fn GameOptionsMenu_7_ControllerSettings(game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        let v0: u16 = update_ram_word!(game.menu_option_index, wrapping_sub(1), reread);
        if (ram_word!(game, menu_option_index) as i32 & 0x8000i32) == 0i32 {
            if v0 != 6u16 {
                return;
            }
            set_ram_word!(game.game_options_screen_index = 10u16);
            return;
        }
        set_ram_word!(game.menu_option_index = 8u16);
        set_ram_word!(game.game_options_screen_index = 9u16);
        return;
    }
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        let v1: u16 = (ram_word!(game, menu_option_index)).wrapping_add(1u16);
        set_ram_word!(game.menu_option_index = v1);
        if v1 == 7u16 {
            set_ram_word!(game.game_options_screen_index = 9u16);
            return;
        }
        if v1 == 9u16 {
            set_ram_word!(game.menu_option_index = 0u16);
            set_ram_word!(game.game_options_screen_index = 10u16);
        }
    } else if ram_word!(game, joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x38u16, game);
        static kOptionsMenuControllerFuncs: [Option<Func_V>; 9] = callback_table! { unsafe Func_V;
            OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0,
            OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0,
            OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_7, OptionsMenuControllerFunc_8,
        };
        kOptionsMenuControllerFuncs[ram_word!(game, menu_option_index) as usize]
            .expect("non-null function pointer")(game);
    } else if ram_word!(game, joypad2_new_keys) != 0u16
        && ram_word!(game, menu_option_index) == 8u16
        && ram_word!(game, debug_invincibility) as i32 - 16i32 & 0x8000i32 != 0
    {
        if word_82F204[ram_word!(game, debug_invincibility) as usize] as i32
            & ram_word!(game, joypad2_new_keys) as i32
            == word_82F204[ram_word!(game, debug_invincibility) as usize] as i32
        {
            update_ram_word!(game.debug_invincibility, wrapping_add(1));
        } else {
            set_ram_word!(game.debug_invincibility = 0u16);
        }
    }
}
pub unsafe fn OptionsMenuControllerFunc_8(game: &mut Game) {
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32 {
        set_ram_word!(game.button_config_shoot_x = 64u16);
        set_ram_word!(game.button_config_jump_a = 128u16);
        set_ram_word!(game.button_config_run_b = 0x8000u16);
        set_ram_word!(game.button_config_itemcancel_y = 0x4000u16);
        set_ram_word!(game.button_config_itemswitch = 0x2000u16);
        set_ram_word!(game.button_config_aim_up_R = 16u16);
        set_ram_word!(game.button_config_aim_down_L = 32u16);
        LoadControllerOptionsFromControllerBindings(game);
        OptionsMenuFunc6(game);
    }
}
pub unsafe fn OptionsMenuControllerFunc_7(game: &mut Game) {
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32
        && OptionsMenuFunc8(game) == 0
    {
        set_ram_word!(game.menu_option_index = 0u16);
        GameOptionsMenuItemFunc_4(game);
    }
}
pub unsafe fn GameOptionsMenu_9_ScrollControllerSettingsDown(game: &mut Game) {
    update_ram_word!(game.reg_BG1VOFS, wrapping_add(2u16));
    if ram_word!(game, reg_BG1VOFS) == 32u16 {
        set_ram_word!(game.game_options_screen_index = 7u16);
    }
}
pub unsafe fn GameOptionsMenu_A_ScrollControllerSettingsUp(game: &mut Game) {
    update_ram_word!(game.reg_BG1VOFS, wrapping_sub(2u16));
    if ram_word!(game, reg_BG1VOFS) == 0 {
        set_ram_word!(game.game_options_screen_index = 7u16);
    }
}
pub unsafe fn sub_82F296(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[(v1 + 13i32)] = 24u16);
    set_ram_word!(game.eproj_x_vel[(v1 + 3i32)] = 56u16);
    set_ram_word!(game.eproj_x_vel[(v1 + 11i32)] = 3584u16);
}
pub unsafe fn OptionsPreInstr_F2A9(v0: u16, game: &mut Game) {
    if ram_word!(game, game_state) as i32 == kGameState_2_GameOptionsMenu as i32 {
        let v2: i32 = ram_word!(game, game_options_screen_index) as i32;
        let v3: u16 = crate::rtl::rom_table_word!(game, 0x82f2ed, v2 as isize);
        if v3 != 0 {
            let v4: *const u16 = RomPtr_82(
                (v3 as i32 + 4i32 * ram_word!(game, menu_option_index) as i32) as u16,
                game,
            ) as *const u16;
            let v5: i32 = v0 as i32 >> 1i32;
            set_ram_word!(game.eproj_y_pos[(v5 + 13i32)] = *v4);
            set_ram_word!(game.eproj_x_vel[(v5 + 3i32)] = *v4.offset(1));
        } else {
            let v6: i32 = v0 as i32 >> 1i32;
            set_ram_word!(game.eproj_y_pos[(v6 + 13i32)] = 384u16);
            set_ram_word!(game.eproj_x_vel[(v6 + 3i32)] = 16u16);
        }
    } else {
        let v1: i32 = v0 as i32 >> 1i32;
        set_ram_word!(game.eproj_E[(v1 + 15i32)] = 1u16);
        set_ram_word!(game.eproj_y_vel[(v1 + 17i32)] = addr_off_82F4B6 as u16);
    };
}
pub unsafe fn sub_82F34B(j: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1i32) + 13i32) as isize) = 124u16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F353(j: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1i32) + 13i32) as isize) = 132u16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F35B(j: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1i32) + 13i32) as isize) = 128u16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F363(j: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1i32) + 13i32) as isize) = 128u16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F369(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[(v1 + 3i32)] = 16u16);
    set_ram_word!(game.eproj_x_vel[(v1 + 11i32)] = 3584u16);
}
pub unsafe fn OptionsPreInstr_F376(k: u16, game: &mut Game) {
    if ram_word!(game, game_state) as i32 != kGameState_2_GameOptionsMenu as i32
        || ram_word!(game, game_options_screen_index) == 6u16 && *ram_ptr!(game; 0x51) == 0x80u8
    {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_E[(v1 + 15i32)] = 1u16);
        set_ram_word!(game.eproj_y_vel[(v1 + 17i32)] = addr_off_82F4B6 as u16);
    }
}
pub unsafe fn OptionsPreInstr_F3A0(k: u16, game: &mut Game) {
    match ram_word!(game, game_options_screen_index) as i32 {
        6 => {
            if *ram_ptr!(game; 0x51) == 0x80u8 {
                let v1: i32 = k as i32 >> 1i32;
                set_ram_word!(game.eproj_E[(v1 + 15i32)] = 1u16);
                set_ram_word!(game.eproj_y_vel[(v1 + 17i32)] = -2890i32 as u16);
            }
        }
        9 => {
            update_ram_word!(
                game.eproj_x_vel[((k as i32 >> 1i32) + 3i32)],
                wrapping_sub(2u16)
            );
        }
        10 => {
            update_ram_word!(
                game.eproj_x_vel[((k as i32 >> 1i32) + 3i32)],
                wrapping_add(2u16)
            );
        }
        _ => {}
    };
}
pub unsafe fn OptionsPreInstr_F3E2(k: u16, game: &mut Game) {
    if ram_word!(game, game_options_screen_index) == 6u16 && *ram_ptr!(game; 0x51) == 0x80u8 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_E[(v1 + 15i32)] = 1u16);
        set_ram_word!(game.eproj_y_vel[(v1 + 17i32)] = -2890i32 as u16);
    }
}
pub unsafe fn sub_82F404(k: u16, game: &mut Game) {
    if ram_word!(game, game_options_screen_index) == 1u16 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_E[(v1 + 15i32)] = 1u16);
        set_ram_word!(game.eproj_y_vel[(v1 + 17i32)] = -2890i32 as u16);
    }
}
pub unsafe fn sub_82F419(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[(v1 + 13i32)] = 216u16);
    set_ram_word!(game.eproj_x_vel[(v1 + 3i32)] = 16u16);
    set_ram_word!(game.eproj_x_vel[(v1 + 11i32)] = 3584u16);
}
pub unsafe fn OptionsPreInstr_F42C(k: u16, game: &mut Game) {
    if ram_word!(game, game_state) != 2u16 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_E[(v1 + 15i32)] = 1u16);
        set_ram_word!(game.eproj_y_vel[(v1 + 17i32)] = addr_off_82F4B6 as u16);
    }
}
pub unsafe fn LoadControllerOptionsFromControllerBindings(game: &mut Game) {
    let mut v0: u16 = 0u16;
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        let mut v2: i16 = *(ram_ptr!(game, u16; crate::rtl::rom_table_word!(
            game,
            0x82f54a,
            v1 as isize
        ) as isize)) as i16;
        let clear_button_mapping: bool = ((v2 as i32 & kButton_X as i32) != 0i32)
            || (if (v2 as i32 & kButton_A as i32) != 0i32 {
                set_ram_word!(game.eproj_F[(v1 + 13i32)] = 1u16);
                false
            } else if v2 < 0i16 {
                set_ram_word!(game.eproj_F[(v1 + 13i32)] = 2u16);
                false
            } else if (v2 as i32 & kButton_Select as i32) != 0i32 {
                set_ram_word!(game.eproj_F[(v1 + 13i32)] = 3u16);
                false
            } else if (v2 as i32 & kButton_Y as i32) != 0i32 {
                set_ram_word!(game.eproj_F[(v1 + 13i32)] = 4u16);
                false
            } else if (v2 as i32 & kButton_L as i32) != 0i32 {
                set_ram_word!(game.eproj_F[(v1 + 13i32)] = 5u16);
                false
            } else {
                ((v2 as i32 & kButton_R as i32) == 0i32)
                    || ({
                        set_ram_word!(game.eproj_F[(v1 + 13i32)] = 6u16);
                        false
                    })
            });
        if clear_button_mapping {
            set_ram_word!(game.eproj_F[(v1 + 13i32)] = 0u16);
        }
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(14i16) >= 0i16) {
            break;
        }
    }
}
static word_82F575: [Buttons; 9] = [
    kButton_X,
    kButton_A,
    kButton_B,
    kButton_Select,
    kButton_Y,
    kButton_L,
    kButton_R,
    kButton_Left,
    kButton_Right,
];
pub unsafe fn OptionsMenuFunc8(game: &mut Game) -> u8 {
    let mut v0: i32 = 0i32;
    loop {
        let mut v2: i32 = v0;
        *(ram_ptr!(game, u16; crate::rtl::rom_table_word!(
            game,
            0x82f54a,
            (v0 >> 1i32) as isize
        ) as isize)) =
            word_82F575[ram_word!(game, eproj_F[((v0 >> 1i32) + 13i32)]) as usize] as u16;
        v0 += 2i32;
        if (v2 >= 12i32) {
            break;
        }
    }
    0u8
}
static g_word_82F6AD: [u16; 6] = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
pub unsafe fn OptionsMenuFunc6(game: &mut Game) {
    let mut v0: u16 = 0u16;
    loop {
        let mut v4: u16 = v0;
        let v1: i32 = v0 as i32 >> 1i32;
        let v2: u16 = crate::rtl::rom_table_word!(game, 0x82f639, v1 as isize);
        let v3: *const u16 = RomPtr_82(
            crate::rtl::rom_table_word!(
                game,
                0x82f647,
                ram_word!(game, eproj_F[(v1 + 13i32)]) as isize
            ),
            game,
        ) as *const u16;
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
            as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) = *v3;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(1) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) = *v3.offset(1);
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(2) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) = *v3.offset(2);
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(32) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) = *v3.offset(3);
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(33) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) = *v3.offset(4);
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(34) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) = *v3.offset(5);
        v0 = (v4).wrapping_add(2u16);
        if (((v4) as i16).wrapping_sub(12i16) >= 0i16) {
            break;
        }
    }
    if ram_word!(game, eproj_instr_list_ptr[0]) != 5u16
        && ram_word!(game, eproj_instr_list_ptr[0]) != 6u16
    {
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000)))
            .menu
            .backup_of_io_registers_in_gameover as *mut u8)
            .offset(46) as *mut u32) = *(&raw const g_word_82F6AD as *const u16 as *mut u32);
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[665i32 as usize] =
            g_word_82F6AD[2i32 as usize];
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.field_536 as *mut u8).offset(56)
            as *mut u32) = *((&raw const g_word_82F6AD as *const u16).offset(3) as *mut u32);
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[697i32 as usize] =
            g_word_82F6AD[5i32 as usize];
    }
    if ram_word!(game, eproj_instr_list_ptr[1]) != 5u16
        && ram_word!(game, eproj_instr_list_ptr[1]) != 6u16
    {
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.field_536 as *mut u8).offset(184)
            as *mut u32) = *(&raw const g_word_82F6AD as *const u16 as *mut u32);
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[761i32 as usize] =
            g_word_82F6AD[2i32 as usize];
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8).offset(46)
            as *mut u32) = *((&raw const g_word_82F6AD as *const u16).offset(3) as *mut u32);
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[793i32 as usize] =
            g_word_82F6AD[5i32 as usize];
    }
}
pub unsafe fn OptionsMenuControllerFunc_0(game: &mut Game) {
    let mut v0: u16 = 12u16;
    while (word_82F575[(v0 as i32 >> 1i32) as usize] & ram_word!(game, joypad1_newkeys) as u32)
        == 0u32
    {
        v0 = (v0).wrapping_sub(2u16);
        if (v0 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    let r18: u16 = (v0 as i32 >> 1i32) as u16;
    let mut v1: u16 = (2i32 * ram_word!(game, menu_option_index) as i32 + 2i32) as u16;
    if 2i32 * ram_word!(game, menu_option_index) as i32 - 12i32 & 0x8000i32 == 0 {
        v1 = 0u16;
    }
    let mut v2: u16 = v1;
    for _ in 0i32..=5i32 {
        if *(ram_ptr!(game, u16; 0x1b23)).offset(((v2 as i32 >> 1i32) + 13i32) as isize) == r18 {
            break;
        }
        v2 = (v2).wrapping_add(2u16);
        if ((v2) as i16).wrapping_sub(14i16) >= 0i16 {
            v2 = 0u16;
        }
    }
    let v4: i32 = ram_word!(game, menu_option_index) as i32;
    let r20: u16 = ram_word!(game, eproj_F[(v4 + 13i32)]);
    set_ram_word!(game.eproj_F[(v4 + 13i32)] = r18);
    *(ram_ptr!(game, u16; 0x1b23)).offset(((v2 as i32 >> 1i32) + 13i32) as isize) = r20;
    OptionsMenuFunc6(game);
}
