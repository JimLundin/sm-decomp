pub use crate::game::prelude::*;

use crate::cpu_infra::DebugGameOverMenu;

pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;
pub use crate::game::constants::bank_8f::*;

use crate::game::sm_80::{
    memset7E, CalculateLayer2PosAndScrollsWhenScrolling, CheckEventHappened, ClearBG2Tilemap,
    ClearFXTilemap, ClearOamExt, ClearTimerRam, ClearUnusedOam, CopyToVramNow,
    DebugScrollPosSaveLoad, DecompressToMem, DecompressToVRAM, DisableIrqInterrupts,
    DoorTransitionScrollingSetup, DrawTimer, EnableIrqInterruptsNow, EnableNMI, FixDoorsMovingUp,
    GraphicsAsset, HandleFadeIn, HandleFadeOut, HandleHudTilemap, HasQueuedMusic, InitializeHud,
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

use crate::host::{Die, Warning};

pub use crate::types::Func_V_Coroutine;

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct PlmHeader_Size6 {
        pub func_ptr, instr_list_1_ptr, instr_list_2_ptr: VoidP;
    }

    pub struct LoadBg_E {
        pub field_0, field_2: u16;
    }

    pub struct TileSet {
        pub tile_table_ptr, tiles_ptr, palette_ptr: LongPtr;
    }

    pub struct DemoRoomData {
        pub room_ptr_: VoidP;
        pub door_ptr, door_slot, screen_x_pos, screen_y_pos, samus_x_offs, samus_y_offs,
            demo_length: u16;
        pub demo_code_ptr: VoidP;
    }

    pub struct C2RustUnnamed {
        pub unused, lr_highlight, item_selector, unused2, unused3, map_scroll_arrow_up,
            map_scroll_arrow_down, map_scroll_arrow_right, map_scroll_arrow_left: VoidP;
    }

    pub struct FileCopyArrowStuff {
        pub spritemap, xpos, ypos: u16;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union PauseScreenSpriteAnimationData {
    pub c2rust_unnamed: C2RustUnnamed,
    pub arr: [VoidP; 9],
}

pub use crate::types::Consts_83;

crate::game::constants::typed_constants! { Consts_83;
    addr_kDoorDef_947a = 38010,
}

pub const __ASSERT_FUNCTION: [c_char; 28] =
    unsafe { transmute::<[u8; 28], [c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0") };
crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
}
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    crate::native_assertion! {
        '_c2rust_label: addr < 0x2000;
        b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, pointer __ASSERT_FUNCTION.as_ptr()
    };
    ram_ptr!(game; addr as i32 as isize)
}
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
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
    fnDoorTransitionFunction_HandleElevator = 8577405,
}
crate::game::constants::typed_constants! { i32;
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db,
    fnDoorTransition_WaitForMusicToClear = 0x82e664,
    fnDoorTransition_HandleTransition = 0x82e6a2, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
}
crate::game::constants::typed_constants! { u32;
    fnPauseHook_Empty = 8946657,
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
crate::game::callback_dispatch! {
    pub unsafe fn CallDemoRoomDataFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [8554778] DemoRoom_ChargeBeamRoomScroll21 [] {
        *ram_ptr!(game; 0xcd20, 33) = 0;
    };
    [8554789] DemoRoom_SetBG2TilemapBase [] {
        *(ram_ptr!(game, u16; 0x59)) = 74;
    };
    [8554795] DemoRoom_SetKraidFunctionTimer [] {
        (*(ram_ptr!(game, EnemyData; 0xf78))).ai_preinstr = 60;
    };
    [8554802] DemoRoom_SetBrinstarBossBits [] {
        *ram_ptr!(game; 0xd828, 1) = 1;
    };
    @external {
        8554788 => [()];
    }
}
pub unsafe fn InitAndLoadGameData_Async(game: &mut Game) -> CoroutineRet {
    crate::verify_probe::hit("coroutine.init");
    let resume_state: *mut u8 = ram_ptr!(game; 0x612);
    let mut stage = *resume_state;
    let mut waiting = stage >= 2;
    if stage == 0 {
        if word!(game.game_state) as i32 == kGameState_40_TransitionToDemo as i32 {
            InitIoForGameplay(game);
            LoadStdBG3andSpriteTilesClearTilemaps(game);
            LoadInitialPalette(game);
            Samus_Initialize(game);
            LoadDemoRoomData(game);
        } else {
            if word!(game.loading_game_state) as i32 != kLoadingGameState_5_Main as i32 {
                if word!(game.loading_game_state) as i32
                    == kLoadingGameState_1F_StartingAtCeres as i32
                {
                    word! {game; area_index = 6; load_station_index = 0; }
                    ClearTimerRam(game);
                } else if word!(game.loading_game_state) as i32
                    == kLoadingGameState_22_EscapingCeres as i32
                {
                    word! {game; area_index = 0; load_station_index = 18; }
                    LoadMirrorOfExploredMapTiles(game);
                }
            }
            InitIoForGameplay(game);
            LoadStdBG3andSpriteTilesClearTilemaps(game);
            LoadInitialPalette(game);
            Samus_Initialize(game);
            LoadFromLoadStation(game);
        }
    }
    if stage <= 1 {
        let yielded: u8 = StartGameplay_Async(game);
        if yielded != 0 {
            *resume_state = 1;
            return yielded;
        }
        InitializeHud(game);
        word_transfers!(game, ascending, 16; native(0xc380) <- native(0xc180));
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        EnableNMI(game);
        EnableEprojs(game);
        EnablePLMs(game);
        EnablePaletteFx(game);
        EnableHdmaObjects(game);
        EnableAnimtiles(game);
        SetLiquidPhysicsType(game);
        if word!(game.game_state) as i32 == kGameState_40_TransitionToDemo as i32 {
            word!(game.loop_counter_transfer_enemies_to_vram = 6);
            stage = 2;
        } else if word!(game.loading_game_state) as i32 == kLoadingGameState_22_EscapingCeres as i32
        {
            QueueMusic_Delayed8(5, game);
            word!(game.loop_counter_transfer_enemies_to_vram = 15);
            stage = 3;
        } else {
            word!(game.loop_counter_transfer_enemies_to_vram = 6);
            stage = 4;
        }
    }
    if matches!(stage, 2..=4) {
        crate::verify_probe::hit(match stage {
            2 => "coroutine.init.demo",
            3 => "coroutine.init.escape",
            _ => "coroutine.init.normal",
        });
        loop {
            if !waiting {
                TransferEnemyTilesToVramAndInit(game);
            }
            let yielded: u8 = WaitForNMI_Async(game);
            if yielded != 0 {
                *resume_state = stage;
                return yielded;
            }
            word!(game.loop_counter_transfer_enemies_to_vram -= 1);
            if (word!(game.loop_counter_transfer_enemies_to_vram) & 0x8000) != 0 {
                break;
            }
            waiting = false;
        }
        match stage {
            2 => {
                let r18: u16 = (*get_DemoRoomData(
                    (crate::rtl::rom_table_word!(game, 0x82876c, word!(game.demo_set) as isize)
                        as i32
                        + 18i32 * (word!(game.demo_scene) as i32 - 1i32))
                        as u16,
                    game,
                ))
                .demo_code_ptr;
                CallDemoRoomDataFunc(((r18) as u32) | (0x820000u32), game);
                word!(game.game_state += 1);
            }
            3 => {
                word!(game.game_state += 1);
            }
            _ => {
                word!(game.game_state = kGameState_7_MainGameplayFadeIn as u16);
            }
        }
        word_transfers!(game, ascending, 256; native(0xc000) <- le(target_palettes, 0));
        if stage == 4 {
            if word!(game.loading_game_state) as i32 == kLoadingGameState_1F_StartingAtCeres as i32
            {
                word!(game.palette_buffer[223] = 0);
                CallSomeSamusCode(8, game);
            } else {
                CallSomeSamusCode(9, game);
            }
        }
    }
    *resume_state = 0;
    0 as CoroutineRet
}
pub unsafe fn InitIoForGameplay(game: &mut Game) {
    InitCpuForGameplay(game);
    InitPpuForGameplay(game);
}
pub unsafe fn InitCpuForGameplay(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(NMITIMEN as u16, 1, 0x84, false); byte(WRIO as u16, 0);
        byte(WRMPYA as u16, 0); byte(WRMPYB as u16, 0);
        byte(WRDIVL as u16, 0); byte(WRDIVH as u16, 0);
        byte(WRDIVB as u16, 0); byte(HTIMEL as u16, 0);
        byte(HTIMEH as u16, 0); byte(VTIMEL as u16, 0);
        byte(VTIMEH as u16, 0); byte(MDMAEN as u16, 0);
        mirror(HDMAEN as u16, 0, 0x85, false); mirror(MEMSEL as u16, 1, 0x86, false);
    }
}
pub unsafe fn InitPpuForGameplay(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
        byte(OAMADDL as u16, 0);
    }
    *(ram_ptr!(game, u16; 0x53) as *mut u8) = 0;
    WriteReg(OAMADDH as u16, 0x80, game);
    byte!(@pointer ram_ptr!(game, u16; 0x53); 1) = 0x80;
    crate::rtl::register_writes! { game;
        byte(OAMDATA as u16, 0); byte(OAMDATA as u16, 0);
        mirror(BGMODE as u16, 9, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, 0x51, 0x58, true); mirror(BG2SC as u16, 0x49, 0x59, true);
        mirror(BG3SC as u16, 0x5a, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        byte(VMAIN as u16, 0); mirror(W12SEL as u16, 0, 0x60, false);
        mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
        mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
        mirror(WH1 as u16, 0, 0x64, false); mirror(WH2 as u16, 0, 0x65, false);
        mirror(WH3 as u16, 0, 0x66, false); mirror(WBGLOG as u16, 0, 0x67, false);
        mirror(WOBJLOG as u16, 0, 0x68, false); mirror(TM as u16, 0x17, 0x69, false);
        mirror(TMW as u16, 0, 0x6c, false); mirror(TS as u16, 0, 0x6b, false);
        mirror(TSW as u16, 0, 0x6d, false); mirror(CGWSEL as u16, 0, 0x6e, false);
        mirror(CGADSUB as u16, 0, 0x71, false); byte(COLDATA as u16, 0xe0);
        mirror(SETINI as u16, 0, 0x77, false);
    }
    word!(game.oam_next_ptr = 0);
    memset7E(ram_ptr!(game, Ram3000; 0x3000) as *mut u16, 0, 0x7fe);
    memset7E(ram_ptr!(game, Ram4000; 0x4000) as *mut u16, 0x6f, 0x7fe);
    memset7E(
        &raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_tilemap as *mut u16,
        0x2c0f,
        0xfe,
    );
}
pub unsafe fn LoadInitialPalette(game: &mut Game) {
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x9a8000, 0));
}
pub unsafe fn LoadStdBG3andSpriteTilesClearTilemaps(game: &mut Game) {
    GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
    GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x2e00).load(game);
    GraphicsAsset::VramWords(0x7e4000, 0x5000, 0x1000).load(game);
    GraphicsAsset::VramWords(0x7e4000, 0x5800, 0x800).load(game);
}
pub unsafe fn GameState_32_MadeItToCeresElevator(game: &mut Game) -> CoroutineRet {
    if word!(game.timer_status) != 0 {
        DrawTimer(game);
    }
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    let v0: bool = (word!(game.reached_ceres_elevator_fade_timer -= 1, reread) & 0x8000) != 0;
    if word!(game.reached_ceres_elevator_fade_timer) == 0 || v0 {
        word!(game.game_state += 1);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_33_BlackoutFromCeres(game: &mut Game) -> CoroutineRet {
    if word!(game.timer_status) != 0 {
        DrawTimer(game);
    }
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        word! { game, 0; fx_layer_blending_config_a, cur_irq_handler }
        *ram_ptr!(game; 0x6e) = 0;
        *ram_ptr!(game; 0x71) = 0;
        *ram_ptr!(game; 0x69) = 16;
        *ram_ptr!(game; 0x6b) = 0;
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
        *ram_ptr!(game; 0x55) = 9;
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.loading_game_state = kLoadingGameState_22_EscapingCeres as u16);
        word!(game.game_state = 34);
        SaveToSram(word!(game.selected_save_slot), game);
        word!(game.cinematic_function = ((0x8bc11bi32) as u16));
        word! { game, 0; ceres_status, timer_status }
        QueueMusic_Delayed8(0, game);
        QueueSfx1_Max15(2, game);
        QueueSfx2_Max15(0x71, game);
        QueueSfx3_Max15(1, game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_35_TimeUp(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    word!(game.palette_change_denom = 8);
    if AdvancePaletteFadeForAllPalettes(game) != 0 {
        word!(game.game_state = kGameState_36_WhitingOutFromTimeUp as u16);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_36_WhitingOutFromTimeUp(game: &mut Game) -> CoroutineRet {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        word! { game, 0; fx_layer_blending_config_a, cur_irq_handler }
        *ram_ptr!(game; 0x6e) = 0;
        *ram_ptr!(game; 0x71) = 0;
        *ram_ptr!(game; 0x69) = 16;
        *ram_ptr!(game; 0x6b) = 0;
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
        *ram_ptr!(game; 0x55) = 9;
        word! { game, 0;
            screen_fade_delay, screen_fade_counter, ceres_status, timer_status,
        }
        QueueSfx1_Max15(2, game);
        QueueSfx2_Max15(0x71, game);
        QueueSfx3_Max15(1, game);
        if CheckEventHappened(0xe, game) != 0 {
            word! { game, 0; game_options_screen_index, menu_index }
            word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
            word!(game.game_state = kGameState_25_SamusNoHealth as u16);
        } else {
            word!(game.game_state = kGameState_37_CeresGoesBoomWithSamus as u16);
            word!(game.cinematic_function = ((0x8bc11bi32) as u16));
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
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        word!(game.fx_layer_blending_config_a = 0);
        *ram_ptr!(game; 0x6e) = 0;
        *ram_ptr!(game; 0x71) = 0;
        *ram_ptr!(game; 0x69) = 16;
        *ram_ptr!(game; 0x6b) = 0;
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
        *ram_ptr!(game; 0x55) = 9;
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word! {game; game_state = kGameState_39_EndingAndCredits as u16;
        cinematic_function = ((0x8bd480i32) as u16); timer_status = 0; }
        QueueMusic_Delayed8(0, game);
        QueueSfx1_Max15(2, game);
        QueueSfx2_Max15(0x71, game);
        QueueSfx3_Max15(1, game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_41_TransitionToDemo(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HdmaObjectHandler(game);
    word!(game.game_state += 1);
    *ram_ptr!(game; 0x51) = 15;
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_42_PlayingDemo_Async(game: &mut Game) -> CoroutineRet {
    crate::verify_probe::hit("coroutine.demo");
    let resume_state: *mut u8 = ram_ptr!(game; 0x614);
    let resume = *resume_state;
    'done: {
        if resume > 2 {
            break 'done;
        }
        'finish_demo: {
            if resume != 1 {
                let yielded: u8 = GameState_8_MainGameplay(game);
                if yielded != 0 {
                    *resume_state = 2;
                    return yielded;
                }
                if word!(game.joypad1_newkeys) != 0 {
                    word!(game.substate = 1);
                    break 'finish_demo;
                }
                word!(game.demo_timer -= 1);
                if !(word!(game.demo_timer) == 0 || (word!(game.demo_timer) & 0x8000) != 0) {
                    break 'done;
                }
                word! {game; substate = 0; my_counter = 90; }
            }
            do_until! { word!(game.my_counter -= 1, reread) == 0;
                let yielded: u8 = WaitForNMI_Async(game);
                if yielded != 0 {
                    *resume_state = 1;
                    return yielded;
                }
                if word!(game.joypad1_newkeys) != 0 {
                    word!(game.substate = 1);
                    break;
                }
            }
        }
        word!(game.game_state += 1);
        word!(game.debug_disable_sounds = 0);
        *ram_ptr!(game; 0x51) = 0x80;
        word! { game, 1; screen_fade_delay, screen_fade_counter }
    }
    *resume_state = 0;
    0 as CoroutineRet
}
pub unsafe fn GameState_43_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    if word!(game.substate) != 1 {
        CheckForNextDemo(game);
    }
    EnableNMI(game);
    word!(game.game_state += 1);
    word! { game, 0; screen_fade_delay, screen_fade_counter }
    WaitUntilEndOfVblankAndClearHdma(game);
    DisableIrqInterrupts(game);
    word! { game, 0;
        fx_layer_blending_config_a, cur_irq_handler, irqhandler_next_handler,
    }
    DisablePaletteFx(game);
    ClearPaletteFXObjects(game);
    word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
    word_transfers!(game, descending, 270; native(0x996) <- value(0));
    *ram_ptr!(game; 0x6e) = 0;
    *ram_ptr!(game; 0x71) = 0;
    *ram_ptr!(game; 0x69) = 16;
    *ram_ptr!(game; 0x6b) = 0;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 0;
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_44_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    word!(game.game_state = kGameState_1_OpeningCinematic as u16);
    if (word!(game.substate) & 0x8000) != 0 {
        word!(game.game_state = kGameState_40_TransitionToDemo as u16);
    } else if word!(game.substate) != 0 {
        LoadTitleSequenceGraphics(game);
        word! {game; eproj_x_pos[4] = 2; cinematic_function = ((0x8b9a47i32) as u16); }
    } else {
        QueueMusic_Delayed8(0, game);
        word! {game; debug_disable_sounds = 0; cinematic_function = ((0x8b9b68i32) as u16); }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn CheckForNextDemo(game: &mut Game) {
    if *(RomPtr_82(
        (crate::rtl::rom_table_word!(game, 0x82876c, word!(game.demo_set) as isize) as i32
            + 18 * word!(game.demo_scene) as i32) as u16,
        game,
    ) as *mut u16)
        == 0xffff
    {
        word!(game.substate = 0);
        let mut v0: u16 = (word!(game.demo_set)).wrapping_add(1);
        if v0 >= word!(game.num_demo_sets) {
            v0 = 0;
        }
        word! {game; demo_set = v0; demo_scene = 0; }
    } else {
        word!(game.substate = 0x8000);
    };
}
pub unsafe fn LoadDemoRoomData(game: &mut Game) {
    word!(game.door_def_ptr = 0);
    let mut drd = get_DemoRoomData(
        (18 * word!(game.demo_scene) as i32
            + crate::rtl::rom_table_word!(game, 0x82876c, word!(game.demo_set) as isize) as i32)
            as u16,
        game,
    );
    word! { game;
        room_ptr = (*drd).room_ptr_;
        door_def_ptr = (*drd).door_ptr;
        layer1_x_pos = (*drd).screen_x_pos;
        bg1_x_offset = word!(game.layer1_x_pos);
        layer1_y_pos = (*drd).screen_y_pos;
        bg1_y_offset = word!(game.layer1_y_pos);
        samus_y_pos = (word!(game.layer1_y_pos)).wrapping_add((*drd).samus_x_offs);
        samus_prev_y_pos = word!(game.samus_y_pos);
        samus_x_pos = ((*drd).samus_y_offs as i32 + word!(game.layer1_x_pos) as i32 + 128) as u16;
        samus_prev_x_pos = word!(game.samus_x_pos);
        demo_timer = (*drd).demo_length;
    }
    byte!(@pointer ram_ptr!(game, u16; 0x79f); 0) =
        (*get_RoomDefHeader(word!(game.room_ptr), game)).area_index_;
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    word!(game.demo_scene += 1);
    let mut v1: u16 = 0;
    word_transfers!(game, ascending, 4;
        native(0xd830) <- value(-1i32 as u16),
        native(0xd870) <- value(-1i32 as u16),
        le(UNUSED_word_7ED8F0, 0) <- value(-1i32 as u16),
        native(0xd908) <- value(-1i32 as u16),
        native(0xd8f8) <- value(-1i32 as u16),
        native(0xd900) <- value(-1i32 as u16),
        native(0xd8b0) <- value(0),
        native(0xd820) <- value(0),
        native(0xd828) <- value(0),
    );
    v1 = 8;
    word_transfers!(game, ascending, 28;
        native(0xd838) <- value(-1i32 as u16),
        native(0xd878) <- value(-1i32 as u16),
        native(0xd8b8) <- value(0),
    );
    v1 = 64;
    word_transfers!(game, ascending, 768; le(explored_map_tiles_saved, 0) <- value(0));
    word! { game, 0;
        samus_max_reserve_health, samus_reserve_health, reserve_health_mode,
    }
    word!(game.loading_game_state = kLoadingGameState_0_Intro as u16);
    word!(game.debug_disable_minimap = 0);
}
static kGameStateFuncs: [Func_V_Coroutine; 45] = callback_table! { nonnull Func_V_Coroutine;
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
    let begin_frame: bool = if st >= 1 && st <= 3 {
        let mut _coret_: u8 = Vector_RESET_Async(game);
        if _coret_ != 0 {
            return _coret_;
        }
        true
    } else {
        !(st >= 10)
            && ({
                if st != 0 {
                    Die(c"Incorrect coroutine_state_0".as_ptr());
                }
                true
            })
    };
    if begin_frame {
        crate::update_place! { reference fresh149 = *ram_ptr!(game; 0x615) => 0; }
        crate::update_place! { reference fresh150 = *ram_ptr!(game; 0x614) => *fresh149; }
        crate::update_place! { reference fresh151 = *ram_ptr!(game; 0x613) => *fresh150; }
        *ram_ptr!(game; 0x612) = *fresh151;
        ReadJoypadInputs(game);
        HdmaObjectHandler(game);
        NextRandom(game);
        ClearOamExt(game);
        word! { game, 0;
            oam_next_ptr, nmi_copy_samus_halves, nmi_copy_samus_top_half_src,
            nmi_copy_samus_bottom_half_src,
        }
        st = word!(game.game_state) as i32 + 10;
        *ram_ptr!(game; 0x611) = st as u8;
    }
    let mut _coret__0: u8 = kGameStateFuncs[(st - 10i32) as usize](game);
    if _coret__0 != 0 {
        return _coret__0;
    }
    HandleSoundEffects(game);
    ClearUnusedOam(game);
    ShowSpareCpu(game);
    if *ram_ptr!(game; 0x612)
        | *ram_ptr!(game; 0x613)
        | *ram_ptr!(game; 0x614)
        | *ram_ptr!(game; 0x615)
        != 0
    {
        printf(
            c"Coroutine state: %d, %d, %d, %d\n".as_ptr(),
            *ram_ptr!(game; 0x612) as i32,
            *ram_ptr!(game; 0x613) as i32,
            *ram_ptr!(game; 0x614) as i32,
            *ram_ptr!(game; 0x615) as i32,
        );
        Warning(c"Coroutine State is broken!".as_ptr());
    }
    *ram_ptr!(game; 0x611) = 0;
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RunOneFrameOfGame(game: &mut Game) {
    let ret: CoroutineRet = RunOneFrameOfGameInner(game);
    if ret == 0 {
        *ram_ptr!(game; 0x5b4) = 1;
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
static kSfxHandlers: [Func_Y_V; 5] = callback_table! { nonnull Func_Y_V;
    SfxHandlers_0_SendToApu, SfxHandlers_1_WaitForAck, SfxHandlers_2_ClearRequest,
    SfxHandlers_3_WaitForAck, SfxHandlers_4_Reset,
};
pub unsafe fn HandleSoundEffects(game: &mut Game) {
    if ((word!(game.sound_handler_downtime)) as i8).wrapping_sub(1) >= 0 {
        byte!(@pointer ram_ptr!(game, u16; 0x686); 0) =
            ((word!(game.sound_handler_downtime)) as u8).wrapping_sub(1);
        RtlApuWrite(APUI01, 0, game);
        *ram_ptr!(game; 0x64d, 0) = 0;
        RtlApuWrite(APUI02, 0, game);
        *ram_ptr!(game; 0x64d, 1) = 0;
        RtlApuWrite(APUI03, 0, game);
        *ram_ptr!(game; 0x64d, 2) = 0;
    } else {
        for i in 0i32..3 {
            kSfxHandlers[*ram_ptr!(game; 0x649, i as isize) as usize](i as u16, game);
        }
    };
}
pub unsafe fn SfxHandlers_0_SendToApu(j: u16, game: &mut Game) {
    if *ram_ptr!(game; 0x643, j as isize) != *ram_ptr!(game; 0x646, j as isize) {
        let v2: u8 = (*ram_ptr!(game; 0x643, j as isize) as i32 + j as i32 * 16) as u8;
        let v3: u8 = *ram_ptr!(game; 0x656, v2 as isize);
        RtlApuWrite((j as i32 + APUI01 as i32) as SnesRegs, v3, game);
        *ram_ptr!(game; 0x64d, j as isize) = v3;
        *ram_ptr!(game; 0x643, j as isize) = (v2 as i32 + 1 & 0xf) as u8;
        crate::update_place! { reference fresh134 = *ram_ptr!(game; 0x649, j as isize) => (*fresh134).wrapping_add(1); }
    }
}
pub unsafe fn SfxHandlers_1_WaitForAck(j: u16, game: &mut Game) {
    let v1: u8 = *ram_ptr!(game; 0x64d, j as isize);
    crate::update_place! { reference fresh138 = *ram_ptr!(game; 0x649, j as isize) => (*fresh138).wrapping_add(1); }
    *ram_ptr!(game; 0x650, j as isize) = 6;
}
pub unsafe fn SfxHandlers_3_WaitForAck(j: u16, game: &mut Game) {
    let v1: u8 = *ram_ptr!(game; 0x64d, j as isize);
    *ram_ptr!(game; 0x649, j as isize) = 0;
    SfxHandlers_0_SendToApu(j, game);
}
crate::game::countdown_callbacks! {
    pub unsafe fn (j: u16, game: &mut Game);
    before {
        crate::update_place! { reference fresh135 [fresh136] = *ram_ptr!(game; 0x650, j as isize) => (*fresh135).wrapping_sub(1); }
    }
    when fresh136 == 1;
    SfxHandlers_2_ClearRequest {
        *ram_ptr!(game; 0x64d, j as isize) = 0;
        crate::update_place! { reference fresh137 = *ram_ptr!(game; 0x649, j as isize) => (*fresh137).wrapping_add(1); }
    };
    SfxHandlers_4_Reset {
        *ram_ptr!(game; 0x649, j as isize) = 0;
    };
}
pub unsafe fn ResetSoundQueues(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x643)) = 0;
    *(ram_ptr!(game, u16; 0x643, 2)) = 0;
    *(ram_ptr!(game, u16; 0x646, 1)) = 0;
    *(ram_ptr!(game, u16; 0x649)) = 0;
    *ram_ptr!(game; 0x649, 2) = 0;
}
pub unsafe fn ShowSpareCpu(game: &mut Game) {
    word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
}
pub unsafe fn GameState_0_Reset_Async(game: &mut Game) -> CoroutineRet {
    word! { game, 0; UNUSED_word_7E0DF8, UNUSED_word_7E0DFA, UNUSED_word_7E0DFC }
    word! {game; cinematic_function = ((0x8b9b68i32) as u16); demo_set = 0; }
    if word!(game.num_demo_sets) == 4 {
        word!(game.demo_set = 3);
    }
    word!(game.game_state += 1);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_1_OpeningCinematic(game: &mut Game) -> CoroutineRet {
    GameState_1_OpeningCinematic_(game)
}
pub unsafe fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
    kCoroutineNone as CoroutineRet
}
crate::forward_calls! { pub unsafe fn (game: &mut Game) -> CoroutineRet;
    GameState_37_CeresGoesBoomWithSamus => GameState_37_CeresGoesBoomWithSamus_(game);
    GameState_39_EndingAndCredits => GameState_39_EndingAndCredits_(game);
}
pub unsafe fn GameState_7_MainGameplayFadeIn(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    GameState_28_Unused_(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_8_MainGameplay(game: &mut Game) -> CoroutineRet {
    crate::verify_probe::hit("coroutine.gameplay");
    let resume_state: *mut u8 = ram_ptr!(game; 0x612);
    let resume = *resume_state;
    if resume == 0 {
        DetermineWhichEnemiesToProcess(game);
        PaletteFxHandler(game);
        HandleControllerInputForGamePhysics(game);
        if word!(game.debug_disable_sprite_interact) == 0 {
            SamusProjectileInteractionHandler(game);
        }
        EnemyMain(game);
    }
    if resume == 1 || (resume == 0 && word!(game.queued_message_box_index) != 0) {
        let yielded: u8 = DisplayMessageBox_Async(word!(game.queued_message_box_index), game);
        if yielded != 0 {
            *resume_state = 1;
            return yielded;
        }
        word!(game.queued_message_box_index = 0);
    }
    if resume <= 1 {
        HandleSamusMovementAndPause(game);
        EprojRunAll(game);
    }
    if resume <= 2 {
        let yielded: u8 = PlmHandler_Async(game);
        if yielded != 0 {
            *resume_state = 2;
            return yielded;
        }
        AnimtilesHandler(game);
        if word!(game.debug_disable_sprite_interact) == 0 {
            EprojSamusCollDetect(game);
            EprojProjCollDet(game);
            ProcessEnemyPowerBombInteraction(game);
        }
        MainScrollingRoutine(game);
        let debug_scrolling_enabled: i32 = 0;
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
    *resume_state = 0;
    0 as CoroutineRet
}
pub unsafe fn DeleteAllOptionsMenuObjects_(game: &mut Game) {
    word_transfers!(game, descending, 8;
        le(eproj_y_pos, 12) <- value(0),
        le(eproj_y_vel, 24) <- value(0),
    );
}
crate::game::callback_dispatch! {
    pub unsafe fn CallOptionsEntryFunc(ea: u32, j: u16, game: &mut Game) => ea;
    call (j, game); default { Unreachable(game); };
    @functions [pub unsafe fn (j: u16, game: &mut Game)];
    [8581782] sub_82F296 [] {
        let v1: i32 = j as i32 >> 1;
        word! {game; eproj_y_pos[(v1 + 13)] = 24; eproj_x_vel[(v1 + 3)] = 56;
            eproj_x_vel[(v1 + 11)] = 3584; }
    };
    [8581963] sub_82F34B [] {
        *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1) + 13) as isize) = 124;
        sub_82F369(j, game);
    };
    [8581971] sub_82F353 [] {
        *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1) + 13) as isize) = 132;
        sub_82F369(j, game);
    };
    [8581979] sub_82F35B [] {
        *(ram_ptr!(game, u16; 0x1a93)).offset(((j as i32 >> 1) + 13) as isize) = 128;
        sub_82F369(j, game);
    };
    [8582169] sub_82F419 [] {
        let v1: i32 = j as i32 >> 1;
        word! {game; eproj_y_pos[(v1 + 13)] = 216; eproj_x_vel[(v1 + 3)] = 16;
            eproj_x_vel[(v1 + 11)] = 3584; }
    };
    @external {
        8581987 => sub_82F363;
    }
}
pub unsafe fn CreateOptionsMenuObject_(a: u16, j: u16, game: &mut Game) -> u8 {
    word!(game.options_menu_init_param = a);
    let mut v3: u16 = 14;
    while word!(game.cinematicspr_arr7[(v3 >> 1)]) != 0 {
        v3 = (v3).wrapping_sub(2);
        if (v3 & 0x8000) != 0 {
            return 1;
        }
    }
    let v5 = RomPtr_82(j, game);
    let v6: i32 = v3 as i32 >> 1;
    word! {game; optionsmenu_arr1[v6] = *(v5.offset(2) as *mut u16);
    cinematicspr_arr7[v6] = *(v5.offset(4) as *mut u16); cinematicspr_instr_ptr[v6] = 1; }
    word! { game, 0;
        cinematicbg_arr8[v6], optionsmenu_arr5[v6], cinematicspr_arr6[v6], optionsmenu_arr7[v6],
    }
    CallOptionsEntryFunc(((*(v5 as *mut u16)) as u32) | (0x820000u32), v3, game);
    0
}
pub fn OptionsPreInstr_nullsub_57(k: u16) {}
pub unsafe fn OptionsMenuFunc1(game: &mut Game) {
    let mut i: i32 = 14;
    while i >= 0 {
        word!(game.optionsmenu_index = i as u16);
        if word!(game.cinematicspr_arr7[(i >> 1)]) != 0 {
            OptionsMenuFunc2(i as u16, game);
            i = word!(game.optionsmenu_index) as i32;
        }
        i -= 2;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallOptionsPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (v0: u16, game: &mut Game)];
    [8581801] OptionsPreInstr_F2A9 [] {
        if word!(game.game_state) as i32 == kGameState_2_GameOptionsMenu as i32 {
            let v2: i32 = word!(game.game_options_screen_index) as i32;
            let v3: u16 = crate::rtl::rom_table_word!(game, 0x82f2ed, v2 as isize);
            if v3 != 0 {
                let v4 = RomPtr_82(
                    (v3 as i32 + 4 * word!(game.menu_option_index) as i32) as u16,
                    game,
                ) as *const u16;
                let v5: i32 = v0 as i32 >> 1;
                word! {game; eproj_y_pos[(v5 + 13)] = *v4; eproj_x_vel[(v5 + 3)] = *v4.offset(1); }
            } else {
                let v6: i32 = v0 as i32 >> 1;
                word! {game; eproj_y_pos[(v6 + 13)] = 384; eproj_x_vel[(v6 + 3)] = 16; }
            }
        } else {
            let v1: i32 = v0 as i32 >> 1;
            word! {game; eproj_E[(v1 + 15)] = 1; eproj_y_vel[(v1 + 17)] = addr_off_82F4B6 as u16; }
        };
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [8582006] OptionsPreInstr_F376 [] {
        if word!(game.game_state) as i32 != kGameState_2_GameOptionsMenu as i32
            || word!(game.game_options_screen_index) == 6 && *ram_ptr!(game; 0x51) == 0x80
        {
            let v1: i32 = k as i32 >> 1;
            word! {game; eproj_E[(v1 + 15)] = 1; eproj_y_vel[(v1 + 17)] = addr_off_82F4B6 as u16; }
        }
    };
    [8582048] OptionsPreInstr_F3A0 [] {
        match word!(game.game_options_screen_index) as i32 {
            6 => {
                if *ram_ptr!(game; 0x51) == 0x80 {
                    let v1: i32 = k as i32 >> 1;
                    word!(game.eproj_E[(v1 + 15)] = 1);
                    word!(game.eproj_y_vel[(v1 + 17)] = -2890i32 as u16);
                }
            }
            9 => {
                word!(game.eproj_x_vel[((k as i32 >> 1) + 3)] -= 2);
            }
            10 => {
                word!(game.eproj_x_vel[((k as i32 >> 1) + 3)] += 2);
            }
            _ => {}
        };
    };
    [8582114] OptionsPreInstr_F3E2 [] {
        if word!(game.game_options_screen_index) == 6 && *ram_ptr!(game; 0x51) == 0x80 {
            let v1: i32 = k as i32 >> 1;
            word! {game; eproj_E[(v1 + 15)] = 1; eproj_y_vel[(v1 + 17)] = -2890i32 as u16; }
        }
    };
    [8582188] OptionsPreInstr_F42C [] {
        if word!(game.game_state) != 2 {
            let v1: i32 = k as i32 >> 1;
            word! {game; eproj_E[(v1 + 15)] = 1; eproj_y_vel[(v1 + 17)] = addr_off_82F4B6 as u16; }
        }
    };
    @external {
        8555536 => [OptionsPreInstr_nullsub_57(k)];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallOptionsInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 => ea;
    call (k, j, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [8555610] OptionsInstr_Destroy [0] {
        let v2: i32 = k as i32 >> 1;
        word! { game, 0; cinematicbg_arr8[v2], cinematicspr_arr7[v2] }
    };
    [8555620] OptionsInstr_8C64 [0] {
        word!(game.cinematicspr_arr7[(k >> 1)] = (j).wrapping_sub(2));
    };
    [8555630] OptionsInstr_SetPreInstr [(j).wrapping_add(2)] {
        word!(game.optionsmenu_arr1[(k >> 1)] = *(RomPtr_82(j, game) as *mut u16));
    };
    [8555641] OptionsInstr_8C79 [j] {
        word!(game.optionsmenu_arr1[(k >> 1)] = ((0x828c81i32) as u16));
    };
    [8555650] OptionsInstr_Goto [*(RomPtr_82(j, game) as *mut u16)] { };
    [8555657] OptionsInstr_8C89 [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh0 [fresh1] = &raw mut (*(ram_ptr!(game, u16; 0x1b2d)).offset(v2 as isize)) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
        if fresh1 == 1 {
            (j).wrapping_add(2)
        } else {
            OptionsInstr_Goto(k, j, game)
        }
    };
    [8555667] OptionsInstr_8C93 [(j).wrapping_add(2)] {
        word!(game.optionsmenu_arr5[(k >> 1)] = *(RomPtr_82(j, game) as *mut u16));
    };
}
pub unsafe fn OptionsMenuFunc2(k: u16, game: &mut Game) {
    CallOptionsPreInstr(
        ((word!(game.optionsmenu_arr1[(k >> 1)])) as u32) | (0x820000u32),
        k,
        game,
    );
    let v1: u16 = word!(game.optionsmenu_index);
    let v2: i32 = word!(game.optionsmenu_index) as i32 >> 1;
    crate::update_place! { unaligned_word fresh119 [fresh120] = &raw mut (*(ram_ptr!(game, u16; 0x1b1d)).offset(v2 as isize)) => (u16::from_le(fresh119.read_unaligned())).wrapping_sub(1); }
    if fresh120 == 1 {
        let mut v4: u16 = word!(game.cinematicspr_arr7[v2]);
        let mut v5 = null_mut::<u16>();
        loop {
            v5 = RomPtr_82(v4, game) as *mut u16;
            if (*v5 & 0x8000) == 0 {
                break;
            }
            v4 = CallOptionsInstr(
                ((*v5) as u32) | (0x820000u32),
                v1,
                (v4).wrapping_add(2),
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let v7: i32 = v1 as i32 >> 1;
        word! {game; cinematicspr_instr_ptr[v7] = *v5; cinematicbg_arr8[v7] = *v5.offset(1);
        cinematicspr_arr7[v7] = (v4).wrapping_add(4); }
    }
}
pub unsafe fn DrawOptionsMenuSpritemaps(game: &mut Game) {
    for i in (0i32..=14).rev().step_by(2) {
        let v1: i32 = i >> 1;
        if word!(game.cinematicbg_arr8[v1]) != 0 {
            let v2: u16 = word!(game.cinematicbg_arr8[v1]);
            DrawSpritemap(
                0x82,
                v2,
                word!(game.optionsmenu_arr8[v1]),
                word!(game.cinematicbg_arr9[v1]),
                word!(game.optionsmenu_arr10[v1]),
                game,
            );
        }
    }
}
pub unsafe fn GameState_12_Pausing_Darkening_Async(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
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
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    word! {game; pausemenu_leftarrow_animation_frame = 0; pausemenu_palette_animation_timer = 1; }
    word! { game, 0; map_scrolling_direction, map_scrolling_speed_index }
    QueueClearingOfFxTilemap(game);
    word!(game.game_state += 1);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn BackupBG2TilemapForPauseMenu(game: &mut Game) {
    crate::rtl::register_writes! { game;
        word(VMADDL as u16,
        ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8 | 1) as u16,); byte(VMAIN as u16, 0x80);
        byte(DMAP1 as u16, 0x81); byte(BBAD1 as u16, 0x39);
        word(A1T1L as u16, 0xdf5c); byte(A1B1 as u16, 0x7e);
        word(DAS1L as u16, 0x1000); byte(DAS10 as u16, 0);
        word(A2A1L as u16, 0); byte(NTRL1 as u16, 0);
        byte(MDMAEN as u16, 2);
    }
}
pub unsafe fn RestoreBG2TilemapFromPauseScreen(game: &mut Game) {
    GraphicsAsset::VramWords(
        0x7edf5c,
        ((*ram_ptr!(game; 0x59) as u16 & 0xfc) << 8),
        0x1000,
    )
    .load(game);
}
pub unsafe fn BackupSomeGfxStateForPause(game: &mut Game) {
    *ram_ptr!(game; 0x765) = *ram_ptr!(game; 0x58);
    *ram_ptr!(game; 0x766) = *ram_ptr!(game; 0x59);
    *ram_ptr!(game; 0x767) = *ram_ptr!(game; 0x5a);
    *ram_ptr!(game; 0x768) = *ram_ptr!(game; 0x5d);
    *ram_ptr!(game; 0x769) = *ram_ptr!(game; 0x5e);
    *ram_ptr!(game; 0x76a) = *ram_ptr!(game; 0x52);
    *ram_ptr!(game; 0x76b) = word!(game.reg_BG1HOFS) as u8;
    *ram_ptr!(game; 0x76c) = word!(game.reg_BG2HOFS) as u8;
    *ram_ptr!(game; 0x76d) = word!(game.reg_BG3HOFS) as u8;
    *ram_ptr!(game; 0x76e) = word!(game.reg_BG1VOFS) as u8;
    *ram_ptr!(game; 0x76f) = word!(game.reg_BG2VOFS) as u8;
    *ram_ptr!(game; 0x770) = word!(game.reg_BG3VOFS) as u8;
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
    byte!(@pointer ram_ptr!(game, u16; 0xbb); 0) = *ram_ptr!(game; 0x770);
    byte!(@pointer ram_ptr!(game, u16; 0xb7); 0) = *ram_ptr!(game; 0x76f);
    byte!(@pointer ram_ptr!(game, u16; 0xb3); 0) = *ram_ptr!(game; 0x76e);
    byte!(@pointer ram_ptr!(game, u16; 0xb9); 0) = *ram_ptr!(game; 0x76d);
    byte!(@pointer ram_ptr!(game, u16; 0xb5); 0) = *ram_ptr!(game; 0x76c);
    byte!(@pointer ram_ptr!(game, u16; 0xb1); 0) = *ram_ptr!(game; 0x76b);
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
    GraphicsAsset::VramWords(0xb68000, 0, 0x4000).load(game);
    GraphicsAsset::VramWords(0xb6c000, 0x2000, 0x2000).load(game);
    GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
}
pub unsafe fn LoadPauseScreenBaseTilemaps(game: &mut Game) {
    GraphicsAsset::VramWords(0xb6e000, 0x3800, 0x800).load(game);
    GraphicsAsset::WramBytes(0xb6e400, 0x7e3400, 0x400).load(game);
    GraphicsAsset::WramBytes(0xb6e800, 0x7e3800, 0x800).load(game);
    let mut v0: u16 = addr_kDummySamusWireframeTilemap as u16;
    let mut v1: u16 = 236;
    for _ in 0..17 {
        for _ in 0..8 {
            crate::update_place! { previous fresh92 = v1 => v1.wrapping_add(1); }
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[fresh92 as usize] =
                *(RomPtr_82(v0, game) as *mut u16);
            v0 = (v0).wrapping_add(2);
        }
        v1 = (v1).wrapping_add(8);
    }
    WriteSamusWireframeTilemap(game);
    LoadEqupmentScreenReserveHealthTilemap(game);
}
pub unsafe fn LoadEqupmentScreenReserveHealthTilemap(game: &mut Game) {
    if word!(game.samus_max_reserve_health) != 0 {
        let r42: i32 = word!(game.samus_reserve_health) as i32 / 100;
        let rest: i32 = word!(game.samus_reserve_health) as i32 % 100;
        let div10: i32 = rest / 10;
        let mod10: i32 = rest % 10;
        (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[394i32 as usize] =
            (mod10 + 2052) as u16;
        (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[393i32 as usize] =
            (div10 + 2052) as u16;
        (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[392i32 as usize] =
            (r42 + 2052) as u16;
    }
}
pub unsafe fn BackupGameplayPalettesAndLoadForPause(game: &mut Game) {
    word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0xb6f000, 0));
}
pub unsafe fn ContinueInitPauseMenu(game: &mut Game) {
    SetupPpuForPauseMenu(game);
    ResetPauseMenuAnimations(game);
    LoadEquipmentScreenEquipmentTilemaps(game);
    set_pause_label_palettes(0, game);
    UpdatePauseMenuLRStartVramTilemap(game);
    DetermineMapScrollLimits(game);
    SetupMapScrollingForPauseMenu(0x80, game);
}
pub unsafe fn SetupMapScrollingForFileSelectMap(game: &mut Game) {
    ResetPauseMenuAnimations(game);
    DetermineMapScrollLimits(game);
    word!(
        game.reg_BG1HOFS = (word!(game.map_min_x_scroll) as i32
            + (((word!(game.map_max_x_scroll)).wrapping_sub(word!(game.map_min_x_scroll))) as i32
                >> 1)
            - 128i32) as u16
    );
    let mut r18: u16 = (8
        * (word!(game.room_x_coordinate_on_map) as i32
            + ((word!(game.samus_x_pos) as i32 & 0xff00) >> 8))
        - word!(game.reg_BG1HOFS) as i32) as u16;
    let v0: i16 = (224i16).wrapping_sub((r18) as i16);
    if (224i16).wrapping_sub((r18) as i16) >= 0 {
        r18 = (32u16).wrapping_sub(r18);
        if (r18 & 0x8000) == 0 {
            word!(game.reg_BG1HOFS -= r18);
        }
    } else {
        r18 = (224u16).wrapping_sub(r18);
        word!(game.reg_BG1HOFS -= (v0) as u16);
    }
    r18 = (word!(game.map_min_y_scroll) as i32
        + (((word!(game.map_max_y_scroll)).wrapping_sub(word!(game.map_min_y_scroll))) as i32 >> 1)
        + 16) as u16;
    word!(game.reg_BG1VOFS = -(112 - r18 as i32 & 0xfff8) as u16);
    r18 = (8
        * (word!(game.room_y_coordinate_on_map) as i32
            + byte!(@pointer ram_ptr!(game, u16; 0xafa); 1) as i32
            + 1)
        + (112 - r18 as i32 & 0xfff8)) as u16;
    let v1: i16 = (64i16).wrapping_sub((r18) as i16);
    if (64i16).wrapping_sub((r18) as i16) >= 0 {
        r18 = (64u16).wrapping_sub(r18);
        word!(game.reg_BG1VOFS -= (v1) as u16);
        if word!(game.reg_BG1VOFS) as i32 + 40 & 0x8000 != 0 {
            word!(game.reg_BG1VOFS = -40i32 as u16);
        }
    }
}
pub unsafe fn GameState_14_Paused_Async(game: &mut Game) -> CoroutineRet {
    DrawPauseMenuDuringFadeIn(game);
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_15_Paused_Async(game: &mut Game) -> CoroutineRet {
    ReleaseButtonsFilter(3, game);
    MainPauseRoutine(game);
    HandleHudTilemap(game);
    HandlePauseScreenPaletteAnimation(game);
    kCoroutineNone as CoroutineRet
}
static kPauseMenuFuncs: [Func_V; 8] = callback_table! { nonnull Func_V;
    PauseMenu_0_MapScreen, PauseMenu_1_EquipmentScreen, PauseMenu_2,
    PauseMenu_3_MapToEquipment_Load, PauseMenu_4_MapToEquipment_FadeIn, PauseMenu_5,
    PauseMenu_6_EquipmentToMap_Load, PauseMenu_7_EquipmentToMap_FadeIn,
};
pub unsafe fn MainPauseRoutine(game: &mut Game) {
    kPauseMenuFuncs[word!(game.menu_index) as usize](game);
}
pub unsafe fn PauseMenu_0_MapScreen(game: &mut Game) {
    HandlePauseScreenLR(game);
    HandlePauseScreenStart(game);
    HandleMapScrollArrows(game);
    MapScrolling(game);
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    DisplayMapElevatorDestinations(game);
    word!(game.pause_screen_mode = 0);
}
pub unsafe fn PauseMenu_1_EquipmentScreen(game: &mut Game) {
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    EquipmentScreenMain(game);
    HandlePauseScreenLR(game);
    HandlePauseScreenStart(game);
    word!(game.pause_screen_mode = 1);
}
pub unsafe fn PauseMenu_2(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    HandlePauseMenuLRPressHighlight(game);
    word!(game.pause_screen_mode = 0);
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.menu_index += 1);
    }
}
pub unsafe fn PauseMenu_5(game: &mut Game) {
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    HandlePauseMenuLRPressHighlight(game);
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.menu_index += 1);
    }
}
pub unsafe fn PauseMenu_3_MapToEquipment_Load(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    EquipmentScreenSetupReserveMode(game);
    EquipmentScreenTransferBG1Tilemap(game);
    word!(game.pause_screen_mode = 1);
    SetPauseScreenButtonLabelPalettes(game);
    word!(game.pausemenu_lr_animation_frame = 0);
    word!(game.pausemenu_lr_animation_timer = *(rom_ptr!(game, u8; 0x82c10c) as *mut u16));
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    word!(game.menu_index += 1);
}
pub unsafe fn PauseMenu_6_EquipmentToMap_Load(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    LoadPauseMenuMapTilemapAndAreaLabel(game);
    SetPauseScreenButtonLabelPalettes(game);
    word!(game.pausemenu_lr_animation_frame = 0);
    word!(game.pausemenu_lr_animation_timer = *(rom_ptr!(game, u8; 0x82c10c) as *mut u16));
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    word!(game.pause_screen_mode = 0);
    word!(game.menu_index += 1);
}
pub unsafe fn PauseMenu_7_EquipmentToMap_FadeIn(game: &mut Game) {
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    DisplayMapElevatorDestinations(game);
    word!(game.pause_screen_mode = 0);
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        let mut v0: u16 = word!(game.pausemenu_button_label_mode);
        if word!(game.pausemenu_button_label_mode) != 0 {
            v0 = 1;
        }
        word!(game.menu_index = v0);
    }
}
pub unsafe fn PauseMenu_4_MapToEquipment_FadeIn(game: &mut Game) {
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    word!(game.pause_screen_mode = 1);
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        let mut v0: u16 = word!(game.pausemenu_button_label_mode);
        if word!(game.pausemenu_button_label_mode) != 0 {
            v0 = 1;
        }
        word!(game.menu_index = v0);
    }
}
static kMapScrollingFuncs: [Func_V; 5] = callback_table! { nonnull Func_V;
    MapScrolling_0_None, MapScrolling_1_Left, MapScrolling_2_Right, MapScrolling_3_Up,
    MapScrolling_4_Down,
};
pub unsafe fn MapScrolling(game: &mut Game) {
    kMapScrollingFuncs[word!(game.map_scrolling_direction) as usize](game);
}
pub unsafe fn MapScrolling_0_None(game: &mut Game) {
    word!(game.map_scrolling_gear_switch_timer = 4);
}
pub unsafe fn MapScrolling_GetSpeedIndex(game: &mut Game) -> u16 {
    let result: u16 = word!(game.map_scrolling_speed_index);
    if word!(game.map_scrolling_gear_switch_timer) == 0 {
        return (word!(game.map_scrolling_speed_index)).wrapping_add(32);
    }
    result
}
static kMapScrollingSpeedTable: [u16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub unsafe fn MapScrolling_1_Left(game: &mut Game) {
    word!(
        game.reg_BG1HOFS -=
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
    );
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_Common(game: &mut Game) {
    word!(game.map_scrolling_speed_index += 1);
    if (word!(game.map_scrolling_speed_index += 1, reread) & 0xf) == 0 {
        QueueSfx1_Max6(0x36, game);
        word! { game, 0; map_scrolling_direction, map_scrolling_speed_index }
        if word!(game.map_scrolling_gear_switch_timer) != 0 {
            word!(game.map_scrolling_gear_switch_timer -= 1);
        }
    }
}
pub unsafe fn MapScrolling_2_Right(game: &mut Game) {
    word!(
        game.reg_BG1HOFS +=
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
    );
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_3_Up(game: &mut Game) {
    word!(
        game.reg_BG1VOFS -=
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
    );
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_4_Down(game: &mut Game) {
    word!(
        game.reg_BG1VOFS +=
            kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
    );
    MapScrolling_Common(game);
}
pub unsafe fn GameState_16_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    HighlightPauseScreenButton(game);
    DrawPauseMenuDuringFadeout(game);
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DrawPauseMenuDuringFadeout(game: &mut Game) {
    if word!(game.pause_screen_mode) == 1 {
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
            word! { game, 1; screen_fade_delay, screen_fade_counter }
            run_unpause_hook = true;
        }
        1 => run_unpause_hook = true,
        _ => run_unpause_hook = false,
    }
    if run_unpause_hook {
        let mut _coret_: u8 = CallUnpauseHook_Async(Load24(ram_ptr!(game, LongPtr; 0x604)), game);
        if _coret_ != 0 {
            *_state_variable_ = 1;
            return _coret_;
        }
        EnableHdmaObjects(game);
        EnableAnimtiles(game);
        QueueSamusMovementSfx(game);
        word!(game.game_state += 1);
    }
    *_state_variable_ = 0;
    0 as CoroutineRet
}
pub unsafe fn GameState_18_Unpausing(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state = kGameState_8_MainGameplay as u16);
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
    let mut v0: u16 = word!(game.area_index);
    if word!(game.area_index) as i32 - 7i32 & 0x8000 == 0 {
        v0 = 0;
    }
    let r0 = RomPtr(
        Load24((rom_ptr!(game, LongPtr; 0x82964a)).offset(v0 as isize)),
        game,
    ) as *const u16;
    let r3 = match screen {
        MapScreen::Pause => ram_ptr!(game, Ram4000; 0x4000) as *mut u16,
        MapScreen::FileSelect => ram_ptr!(game, Ram3000; 0x3000) as *mut u16,
    };
    let mut r6 = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x829717, v0 as isize),
        game,
    ) as *const u16;
    let mut r9 = ram_ptr!(game; 0x7f7) as *const u16;
    if *ram_ptr!(game; 0xd908, word!(game.area_index) as isize) != 0 {
        let mut mapped = u16::swap_bytes(*r6);
        r6 = r6.offset(1);
        let mut explored = u16::swap_bytes(*r9);
        r9 = r9.offset(1);
        for tile in 0..2048 {
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
        let mut tile = 0;
        for byte in 0..256 {
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
    word! {game; reg_BG1HOFS = word!(game.reg_BG4HOFS); reg_BG1VOFS = word!(game.reg_BG4VOFS); }
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0); byte(VMADDH as u16, 0x30);
        byte(VMAIN as u16, 0x80);
    }
    LoadPauseMenuMapTilemap(game);
    static unk_8293E9: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x1000);
    SetupDmaTransfer(&raw const unk_8293E9 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2); byte(VMADDL as u16, 0xaa);
        byte(VMADDH as u16, 0x38); byte(VMAIN as u16, 0x80);
        byte(DMAP1 as u16, 1); byte(BBAD1 as u16, 0x18);
        byte(DAS1L as u16, 0x18); byte(DAS1H as u16, 0);
    }
    let mut v0: u16 = word!(game.area_index);
    if word!(game.area_index) as i32 - 7i32 & 0x8000 == 0 {
        v0 = 0;
    }
    crate::rtl::register_writes! { game;
        word(A1T1L as u16,
        crate::rtl::rom_table_word!(game, 0x82965f, v0 as isize),); byte(A1B1 as u16, 0x82);
        byte(MDMAEN as u16, 2);
    }
}
pub unsafe fn LoadPauseMenuMapTilemap(game: &mut Game) {
    render_map_tilemap(MapScreen::Pause, game);
}
pub unsafe fn DrawRoomSelectMap(game: &mut Game) {
    *ram_ptr!(game; 0x5d) = 51;
    *ram_ptr!(game; 0x69) = 19;
    word!(game.reg_BG1VOFS = -40i32 as u16);
    render_map_tilemap(MapScreen::FileSelect, game);
    let v16: u16 = word!(game.vram_write_queue_tail);
    let v17 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v17).size = 4096;
    (*v17).src.addr = (ram_ptr!(game, Ram3000; 0x3000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    (*v17).src.bank = 126;
    (*v17).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v16).wrapping_add(7));
}
pub unsafe fn DrawRoomSelectMapAreaLabel(dst: *mut u16, game: &mut Game) {
    let v2 = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x82965f, word!(game.area_index) as isize),
        game,
    ) as *const u16;
    for i in 0i32..12 {
        *dst.offset(i as isize) = (*v2.offset(i as isize)) & (0xefffu16);
    }
}
pub unsafe fn SetupMapScrollingForPauseMenu(a: u16, game: &mut Game) {
    word!(
        game.reg_BG1HOFS = (word!(game.map_min_x_scroll) as i32
            + (((word!(game.map_max_x_scroll)).wrapping_sub(word!(game.map_min_x_scroll))) as i32
                >> 1)
            - 128i32) as u16
    );
    let mut r18: u16 = (8
        * (word!(game.room_x_coordinate_on_map) as i32
            + ((word!(game.samus_x_pos) as i32 & 0xff00) >> 8))
        - word!(game.reg_BG1HOFS) as i32) as u16;
    let mut v1: i16 = (224i16).wrapping_sub((r18) as i16);
    if (224i16).wrapping_sub((r18) as i16) >= 0 {
        r18 = (32u16).wrapping_sub(r18);
        if (r18 & 0x8000) == 0 {
            word!(game.reg_BG1HOFS -= r18);
        }
    } else {
        r18 = (224u16).wrapping_sub(r18);
        word!(game.reg_BG1HOFS -= (v1) as u16);
    }
    r18 = (word!(game.map_min_y_scroll) as i32
        + (((word!(game.map_max_y_scroll)).wrapping_sub(word!(game.map_min_y_scroll))) as i32 >> 1)
        + 16) as u16;
    word!(game.reg_BG1VOFS = -(a as i32 - r18 as i32 & 0xfff8) as u16);
    r18 = (8
        * (word!(game.room_y_coordinate_on_map) as i32
            + byte!(@pointer ram_ptr!(game, u16; 0xafa); 1) as i32
            + 1)
        + (a as i32 - r18 as i32 & 0xfff8)) as u16;
    let mut v2: i16 = (64i16).wrapping_sub((r18) as i16);
    if v2 >= 0 {
        word!(game.reg_BG1VOFS -= (v2) as u16);
        if word!(game.reg_BG1VOFS) as i32 + 40 & 0x8000 != 0 {
            word!(game.reg_BG1VOFS = -40i32 as u16);
        }
    }
}
static k0x80Shr: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];

crate::forward_calls! { unsafe fn (r0: *const u8) -> u16;
    DetermineLeftmostMapColumn => find_map_edge(r0, MapEdge::Left);
    DetermineRightmostMapColumn => find_map_edge(r0, MapEdge::Right);
    DetermineTopmostMapColumn => find_map_edge(r0, MapEdge::Top);
    DetermineBottommostMapColumn => find_map_edge(r0, MapEdge::Bottom);
}
pub unsafe fn DetermineMapScrollLimits(game: &mut Game) {
    let mut r6: *const u8 = if word!(game.has_area_map) != 0 {
        RomPtr_82(
            *(RomPtr_82(
                (addr_kPauseMenuMapData as i32 + 2 * word!(game.area_index) as i32) as u16,
                game,
            ) as *mut u16),
            game,
        )
    } else {
        ram_ptr!(game; 0x7f7)
    };
    word!(game.map_min_x_scroll = (DetermineLeftmostMapColumn(r6)).wrapping_mul(8));
    if word!(game.area_index) == 4 {
        word!(game.map_min_x_scroll -= 24);
    }
    word!(game.map_max_x_scroll = (DetermineRightmostMapColumn(r6.offset(131))).wrapping_mul(8));
    word!(game.map_min_y_scroll = (DetermineTopmostMapColumn(r6)).wrapping_mul(8));
    word!(game.map_max_y_scroll = (DetermineBottommostMapColumn(r6.offset(124))).wrapping_mul(8));
}
pub unsafe fn SetupPpuForPauseMenu(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(OBSEL as u16, 1, 0x52, false); mirror(BGMODE as u16, 9, 0x55, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, 0x31, 0x58, true); mirror(BG2SC as u16, 0x38, 0x59, true);
        mirror(BG3SC as u16, 0x58, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        mirror(TM as u16, 0x17, 0x69, false); mirror(MOSAIC as u16, 0, 0x57, false);
    }
    crate::update_place! { reference fresh24 = *ram_ptr!(game; 0x74, 0) => (*fresh24) & (0xe0u8); }
    crate::update_place! { reference fresh25 = *ram_ptr!(game; 0x74, 1) => (*fresh25) & (0xe0u8); }
    crate::update_place! { reference fresh26 = *ram_ptr!(game; 0x74, 2) => (*fresh26) & (0xe0u8); }
    *ram_ptr!(game; 0x71) = 0;
}
pub unsafe fn ResetPauseMenuAnimations(game: &mut Game) {
    word! { game, 0;
        pausemenu_reserve_tank_delay_ctr, reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
        reg_BG3VOFS, pausemenu_button_label_mode, pausemenu_lr_animation_frame, UNUSED_word_7E0745,
        samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
        samus_position_indicator_animation_loop_counter,
    }
    word! {game; pausemenu_lr_animation_timer = *(rom_ptr!(game, u8; 0x82c10c) as *mut u16);
    pausemenu_palette_animation_timer = 1; pausemenu_palette_animation_frame = 0; }
}
unsafe fn ChangePaletteValues(target: *mut u16, new_color: u16, n: size_t) {
    let mut i: size_t = 0 as size_t;
    while i != n {
        *target.offset(i as isize) = (*target.offset(i as isize) & !(0x1c00) | new_color);
        i = i.wrapping_add(1);
    }
}

macro_rules! draw_equipment_icons {
    (
        $icon_count:literal, $game:ident, $destinations:ident, $collected_field:ident,
        $mask_table:literal, $tilemap_table:literal, $byte_count:literal,
        $equipped_field:ident, $word_count:literal $(,)?
    ) => {
        let mut i_0: i32 = 0;
        while i_0 < $icon_count {
            let target: *mut u16 = ram_ptr!(
                $game, u16; *$destinations.offset(i_0 as isize) as i32 as isize
            );
            if (word!($game.$collected_field)
                & crate::rtl::rom_table_word!($game, $mask_table, i_0 as isize))
                != 0
            {
                memcpy(
                    target as *mut c_void,
                    RomPtr_82(
                        crate::rtl::rom_table_word!(
                            $game,
                            $tilemap_table,
                            i_0 as isize
                        ),
                        $game,
                    ) as *const c_void,
                    $byte_count as size_t,
                );
                if (word!($game.$equipped_field)
                    & crate::rtl::rom_table_word!($game, $mask_table, i_0 as isize))
                    == 0
                {
                    ChangePaletteValues(target, 3072, $word_count as size_t);
                }
            } else {
                memcpy(
                    target as *mut c_void,
                    RomPtr_82(addr_kEquipmentScreenTilemap_Blank as u16, $game)
                        as *const c_void,
                    $byte_count as size_t,
                );
            }
            i_0 += 1;
        }
    };
}

pub unsafe fn LoadEquipmentScreenEquipmentTilemaps(game: &mut Game) {
    if word!(game.samus_max_reserve_health) != 0 {
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
    if word!(game.hyper_beam_flag) != 0 {
        for i in 0i32..5 {
            memcpy(
                ram_ptr!(game, c_void; *p.offset(i as isize) as i32 as isize),
                RomPtr_82(
                    crate::rtl::rom_table_word!(game, 0x82c0a8, i as isize),
                    game,
                ) as *const c_void,
                10 as size_t,
            );
        }
    } else {
        draw_equipment_icons!(
            6,
            game,
            p,
            collected_beams,
            0x82c04c,
            0x82c08c,
            10,
            equipped_beams,
            5
        );
    }
    p = RomPtr_82(addr_kEquipmentTilemapOffs_Suits as u16, game) as *mut u16;
    draw_equipment_icons!(
        6,
        game,
        p,
        collected_items,
        0x82c056,
        0x82c096,
        18,
        equipped_items,
        9
    );
    p = RomPtr_82(addr_kEquipmentTilemapOffs_Boots as u16, game) as *mut u16;
    draw_equipment_icons!(
        3,
        game,
        p,
        collected_items,
        0x82c062,
        0x82c0a2,
        18,
        equipped_items,
        9
    );
}
pub unsafe fn ClearSamusBeamTiles(game: &mut Game) {
    GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1000).load(game);
}
pub unsafe fn ContinueInitGameplayResume(game: &mut Game) {
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- pause_map(384));
    SetupPpuForGameplayResume(game);
    CalculateBgScrolls_Unpause(game);
    UpdateBeamTilesAndPalette_Unpause(game);
    ClearPauseMenuData(game);
    CallSomeSamusCode(0xc, game);
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
    word!(game.reg_BG1HOFS = (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos)));
    word!(game.reg_BG1VOFS = (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos)));
    word!(game.reg_BG2HOFS = (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos)));
    word!(game.reg_BG2VOFS = (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos)));
}
pub unsafe fn UpdateBeamTilesAndPalette_Unpause(game: &mut Game) {
    UpdateBeamTilesAndPalette(game);
}
pub unsafe fn ClearPauseMenuData(game: &mut Game) {
    word! { game, 0;
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
pub unsafe fn HandlePauseScreenLR(game: &mut Game) {
    HandlePauseScreenLrInput(game);
    DrawLrHighlight(game);
}
pub unsafe fn HandlePauseScreenLrInput(game: &mut Game) {
    let play_switch_sound: bool = if (word!(game.newly_held_down_timed_held_input) as i32
        & kButton_L as i32)
        != 0
    {
        if word!(game.pausemenu_button_label_mode) == 0 {
            return;
        }
        word! { game, 5; pausemenu_start_lr_pressed_highlight_timer, menu_index }
        word! {game; pausemenu_shoulder_button_highlight = 1; pausemenu_button_label_mode = 0; }
        SetPauseScreenButtonLabelPalettes(game);
        true
    } else {
        ((word!(game.newly_held_down_timed_held_input) as i32 & kButton_R as i32) != 0
            && word!(game.pausemenu_button_label_mode) != 2)
            && ({
                word!(game.pausemenu_start_lr_pressed_highlight_timer = 5);
                word! { game, 2;
                    menu_index, pausemenu_shoulder_button_highlight, pausemenu_button_label_mode,
                }
                SetPauseScreenButtonLabelPalettes(game);
                true
            })
    };
    if play_switch_sound {
        QueueSfx1_Max6(0x38, game);
    };
}
static kPauseLrButtonPressedHighlight_Spritemap: [u16; 2] = [0x28, 0x29];
static kPauseLrButtonPressedHighlight_Y: [u16; 2] = [0x18, 0xe8];
static kPauseLrButtonPressedHighlight_X: [u16; 2] = [0xd0, 0xd0];
pub unsafe fn HandlePauseMenuLRPressHighlight(game: &mut Game) {
    if word!(game.pausemenu_start_lr_pressed_highlight_timer) != 0 {
        word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
        if word!(game.pausemenu_shoulder_button_highlight) != 0 {
            let v0: i32 = word!(game.pausemenu_shoulder_button_highlight) as i32 - 1i32;
            DrawMenuSpritemap(
                kPauseLrButtonPressedHighlight_Spritemap[v0 as usize],
                kPauseLrButtonPressedHighlight_Y[v0 as usize],
                (kPauseLrButtonPressedHighlight_X[v0 as usize]).wrapping_sub(1),
                0,
                game,
            );
        }
    }
}
pub unsafe fn DrawLrHighlight(game: &mut Game) {
    DrawPauseScreenSpriteAnim(2, 0x18, 0xd0, game);
    DrawPauseScreenSpriteAnim(2, 0xe8, 0xd0, game);
}
pub unsafe fn HandlePauseScreenStart(game: &mut Game) {
    if (word!(game.newly_held_down_timed_held_input) as i32 & kButton_Start as i32) != 0 {
        QueueSfx1_Max6(0x38, game);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        let v0: u16 = word!(game.pausemenu_button_label_mode);
        word!(game.pausemenu_button_label_mode = 1);
        SetPauseScreenButtonLabelPalettes(game);
        word!(game.pausemenu_button_label_mode = v0);
        word!(game.pausemenu_start_lr_pressed_highlight_timer = 11);
        word!(game.game_state += 1);
    }
    UpdatePauseMenuLRStartVramTilemap(game);
}
pub unsafe fn HighlightPauseScreenButton(game: &mut Game) {
    if word!(game.pausemenu_start_lr_pressed_highlight_timer) != 0 {
        word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
        DrawMenuSpritemap(0x2b, 0x90, 0xd0, 0, game);
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
    let mut v0: u16 = word!(game.vram_write_queue_tail);
    (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 128;
    v0 = (v0).wrapping_add(2);
    (*gVramWriteEntry(v0 as i32, game)).size = 0x3640;
    v0 = (v0).wrapping_add(2);
    crate::update_place! { previous fresh23 = v0 => v0.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh23 as i32,
        game,
    ))
    .size as *mut u8) = 126;
    (*gVramWriteEntry(v0 as i32, game)).size =
        (((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) + 800) as u16;
    word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
}
pub unsafe fn DrawPauseScreenSpriteAnim(a: u16, input_k: u16, input_j: u16, game: &mut Game) {
    let t: i32 = a as i32 - 1i32;
    let v8 = RomPtr_RAM(
        (*(rom_ptr!(game, PauseScreenSpriteAnimationData; 0x82c0c4))).arr[t as usize],
        game,
    ) as *mut u16;
    let v5 = RomPtr_RAM(
        (*(rom_ptr!(game, PauseScreenSpriteAnimationData; 0x82c0b2))).arr[t as usize],
        game,
    ) as *mut u16;
    let v3 = RomPtr_82(
        (*(rom_ptr!(game, PauseScreenSpriteAnimationData; 0x82c0e8))).arr[t as usize],
        game,
    );
    *v5 = (*v5).wrapping_sub(1);
    if (*v5 as i16) <= 0 {
        *v8 = (*v8).wrapping_add(1);
        let mut v10: u16 = *v3.offset((3 * *v8 as i32) as isize) as u16;
        if v10 == 255 {
            *v8 = 0;
            v10 = *v3 as u16;
        }
        *v5 = v10;
    }
    let r3: u16 = crate::rtl::rom_table_word!(game, 0x82c0fa, 3);
    let r24: i32 = *v3.offset((3 * *v8 as i32 + 2) as isize) as i32;
    let r26: i32 = 2 * *RomPtr_RAM(
        (*(rom_ptr!(game, PauseScreenSpriteAnimationData; 0x82c0d6))).arr[t as usize],
        game,
    ) as i32;
    let v11 = RomPtr_82(
        (r26 + crate::rtl::rom_table_word!(game, 0x82c1e4, t as isize) as i32) as u16,
        game,
    );
    DrawMenuSpritemap(
        (r24 + *(v11 as *mut u16) as i32) as u16,
        input_k,
        (input_j).wrapping_sub(1),
        r3,
        game,
    );
}
pub unsafe fn HandlePauseScreenPaletteAnimation(game: &mut Game) {
    let mut v2: u16 = 0;
    let mut v1: u8 = 0;
    if word!(game.pausemenu_palette_animation_timer) as u8 != 0 {
        byte!(@pointer ram_ptr!(game, u16; 0x73b); 0) =
            ((word!(game.pausemenu_palette_animation_timer)) as u8).wrapping_sub(1);
        if word!(game.pausemenu_palette_animation_timer) as u8 == 0 {
            let mut i: i32 = word!(game.pausemenu_palette_animation_frame) as i32 + 1;
            loop {
                byte!(@pointer ram_ptr!(game, u16; 0x74f); 0) = i as u8;
                v1 = byte!(@pointer rom_ptr!(game; 0x82c10c); (3 * i) as isize);
                if v1 != 0xff {
                    break;
                }
                QueueSfx3_Max6(0x2a, game);
                i = 0;
            }
            byte!(@pointer ram_ptr!(game, u16; 0x73b); 0) = v1;
            byte!(v2; 0) = 0;
            byte!(v2; 1) = word!(game.pausemenu_palette_animation_frame) as u8;
            let mut v3: u16 = ((v2 as i32 >> 3) + 30) as u16;
            for j in (0u16..=30).rev().step_by(2) {
                *(ram_ptr!(game, u16; 0xc000)).offset(((j as i32 >> 1) + 176) as isize) =
                    crate::rtl::rom_table_word!(game, 0x82a987, (v3 >> 1) as isize);
                v3 = (v3).wrapping_sub(2);
            }
        }
    }
}
pub unsafe fn EquipmentScreenSetupReserveMode(game: &mut Game) {
    let mut current_block: u64;
    let mut v5: i16 = 0;
    let mut v7: i16 = 0;
    word! {game; reg_BG4HOFS = word!(game.reg_BG1HOFS); reg_BG4VOFS = word!(game.reg_BG1VOFS); }
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    if word!(game.samus_max_reserve_health) != 0 && word!(game.reserve_health_mode) != 0 {
        let mut v0: VoidP = addr_kEquipmentScreenTilemap_AUTO as i32 as VoidP;
        if word!(game.reserve_health_mode) != 1 {
            v0 = addr_kEquipmentScreenTilemap_MANUAL as i32 as VoidP;
        }
        let table = RomPtr_82(v0, game) as *mut u16;
        for i in 0i32..4 {
            (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[(i + 327) as usize] =
                ((*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[(i + 327) as usize]
                    & 0xfc00
                    | *table.offset(i as isize));
        }
    }
    word!(game.pausemenu_item_selector_animation_frame = 0);
    word!(game.pausemenu_item_selector_animation_timer = *(rom_ptr!(game, u8; 0x82c10c)) as u16);
    word!(game.pausemenu_reserve_tank_animation_frame = 0);
    word!(game.pausemenu_reserve_tank_animation_timer = *(rom_ptr!(game, u8; 0x82c165)) as u16);
    if word!(game.samus_max_reserve_health) != 0 {
        word!(game.pausemenu_equipment_category_item = 0);
    } else {
        if word!(game.hyper_beam_flag) != 0 {
            current_block = 15976848397966268834;
        } else {
            let mut v4: u16 = 0;
            loop {
                if ((crate::rtl::rom_table_word!(game, 0x82c04c, (v4 >> 1) as isize)
                    & word!(game.collected_beams))
                    != 0)
                {
                    current_block = 4761528863920922185;
                    break;
                }
                v4 = (v4).wrapping_add(2);
                if ((v4) as i16).wrapping_sub(10) >= 0 {
                    current_block = 15976848397966268834;
                    break;
                }
            }
            if current_block == 15976848397966268834 {
            } else {
                byte!(v5; 0) = ((v4 >> 1) >> 8) as u8;
                byte!(v5; 1) = (v4 >> 1) as u8;
                word!(game.pausemenu_equipment_category_item = ((v5) as u16) | (1u16));
                current_block = 9932683223035187777;
            }
        }
        if current_block == 9932683223035187777 {
        } else {
            let mut v6: u16 = 0;
            loop {
                if (crate::rtl::rom_table_word!(game, 0x82c056, (v6 >> 1) as isize)
                    & word!(game.collected_items))
                    != 0
                {
                    byte!(v7; 0) = ((v6 >> 1) >> 8) as u8;
                    byte!(v7; 1) = (v6 >> 1) as u8;
                    word!(game.pausemenu_equipment_category_item = ((v7) as u16) | (2u16));
                    current_block = 9932683223035187777;
                    break;
                } else {
                    v6 = (v6).wrapping_add(2);
                    if (((v6) as i16).wrapping_sub(12) >= 0) {
                        current_block = 1109700713171191020;
                        break;
                    }
                }
            }
            if current_block == 9932683223035187777 {
            } else {
                let mut v8: i16 = 0;
                do_until! { ((v8).wrapping_sub(6) >= 0);
                    v8 = (v8).wrapping_add(2);
                }
            }
        }
    }
    if word!(game.samus_reserve_health) != 0 {
        EquipmentScreenGlowingArrowSolidOn(game);
        WriteSamusWireframeTilemapAndQueue(game);
    }
}
pub unsafe fn EquipmentScreenTransferBG1Tilemap(game: &mut Game) {
    GraphicsAsset::VramWords(0x7e3800, 0x3000, 0x800).load(game);
    word!(game.reg_BG1VOFS = 0);
}
static kEquipmentScreenCategories: [Func_V; 4] = callback_table! { nonnull Func_V;
    EquipmentScreenCategory_Tanks, EquipmentScreenCategory_Weapons,
    EquipmentScreenCategory_Suit, EquipmentScreenCategory_Boots,
};
pub unsafe fn EquipmentScreenMain(game: &mut Game) {
    kEquipmentScreenCategories[word!(game.pausemenu_equipment_category_item) as u8 as usize](game);
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    WriteSamusWireframeTilemapAndQueue(game);
}
static kEquipmentScreenCategory_TanksFuncs: [Func_V; 2] = callback_table! { nonnull Func_V;
    EquipmentScreenCategory_Tanks_0, EquipmentScreenCategory_Tanks_1,
};
pub unsafe fn EquipmentScreenCategory_Tanks(game: &mut Game) {
    kEquipmentScreenCategory_TanksFuncs
        [byte!(@pointer ram_ptr!(game, u16; 0x755); 1isize) as usize](game);
    EquipmentScreenHandleDpad(game);
    EquipmentScreenGlowingArrow(game);
}
pub unsafe fn EquipmentScreenHandleDpad(game: &mut Game) {
    let r18: u16 = word!(game.pausemenu_equipment_category_item);
    if (word!(game.joypad1_newkeys) as i32 & kButton_Right as i32) != 0 {
        if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0
            || EquipmentScreenMoveLowerOnSuitsMisc(0, game) == 0xffff
        {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0, r18, game);
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
        if (word!(game.pausemenu_equipment_category_item) & 0xff00) != 0 {
            QueueSfx1_Max6(0x37, game);
            word!(game.pausemenu_equipment_category_item -= 256);
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
        if (word!(game.pausemenu_equipment_category_item) & 0xff00) == 256
            || word!(game.reserve_health_mode) == 1
            || {
                word!(game.pausemenu_equipment_category_item += 256);
                word!(game.samus_reserve_health) == 0
            }
        {
            EquipmentScreenMoveToBeams(0, r18, game);
        } else {
            QueueSfx1_Max6(0x37, game);
        }
    }
}
static kEquipmentScreenGlowingArrowFuncs: [Func_V; 2] = callback_table! { nonnull Func_V;
    EquipmentScreenGlowingArrowAnimated, EquipmentScreenGlowingArrowSolidOn,
};
pub unsafe fn EquipmentScreenGlowingArrow(game: &mut Game) {
    if word!(game.pausemenu_equipment_category_item) as u8 != 0 {
        EquipmentScreenGlowingArrowSolidOff(game);
    } else {
        kEquipmentScreenGlowingArrowFuncs
            [byte!(@pointer ram_ptr!(game, u16; 0x755); 1isize) as usize](game);
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
    if word!(game.reserve_health_mode) == 1 {
        let v0: i32 = *ram_ptr!(game; 0x5b5) as i32 & 0x1f;
        word!(game.palette_buffer[102] = kEquipmentScreenGlowingArrowPalettes0[v0 as usize]);
        word!(game.palette_buffer[107] = kEquipmentScreenGlowingArrowPalettes1[v0 as usize]);
        EquipmentScreenEnergyArrowGlow_On(game);
    } else {
        word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
        EquipmentScreenEnergyArrowGlow_Off(game);
    };
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOn(game: &mut Game) {
    word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
    EquipmentScreenEnergyArrowGlow_On(game);
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOff(game: &mut Game) {
    word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
    EquipmentScreenEnergyArrowGlow_Off(game);
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_On(game: &mut Game) {
    for i in 0i32..8 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
            [(129 + i * 32) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
            .equipment_screen_bg1_tilemap[(129 + i * 32) as usize]
            & !(0x1c00)
            | 0x1800);
    }
    for i_0 in 0i32..2 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap[(385 + i_0) as usize] =
            ((*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
                [(385 + i_0) as usize]
                & !(0x1c00)
                | 0x1800);
    }
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_Off(game: &mut Game) {
    for i in 0i32..8 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
            [(129 + i * 32) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
            .equipment_screen_bg1_tilemap[(129 + i * 32) as usize]
            & !(0x1c00)
            | 0x1c00);
    }
    for i_0 in 0i32..2 {
        (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap[(385 + i_0) as usize] =
            ((*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
                [(385 + i_0) as usize]
                & !(0x1c00)
                | 0x1c00);
    }
}
static kEquipmentScreenTilemap_MANUAL: [u16; 4] = [0x3d46, 0x3d47, 0x3d48, 0x3d49];
static kEquipmentScreenTilemap_AUTO: [u16; 4] = [0x3d56, 0x3d57, 0x3d58, 0x3d59];
pub unsafe fn EquipmentScreenCategory_Tanks_0(game: &mut Game) {
    if (word!(game.joypad1_newkeys) as i32 & kButton_A as i32) != 0
        && word!(game.samus_max_reserve_health) != 0
    {
        QueueSfx1_Max6(0x37, game);
        if word!(game.reserve_health_mode) == 1 {
            word!(game.reserve_health_mode = 2);
            EquipmentScreenHudReserveAutoTilemap_Off(game);
            let mut v1: u16 = 0;
            for _ in (0i16..8i16).step_by(2) {
                (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                    [(v1 as i32 + 327) as usize] = (kEquipmentScreenTilemap_MANUAL[v1 as usize]
                    | (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                        [(v1 as i32 + 327) as usize]
                        & 0xfc00);
                v1 = v1.wrapping_add(1);
            }
        } else {
            word!(game.reserve_health_mode = 1);
            EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game);
            let mut v3: u16 = 0;
            for _ in (0i16..8i16).step_by(2) {
                (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                    [(v3 as i32 + 327) as usize] = (kEquipmentScreenTilemap_AUTO[v3 as usize]
                    | (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap
                        [(v3 as i32 + 327) as usize]
                        & 0xfc00);
                v3 = v3.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game: &mut Game) {
    let mut v0: u16 = -26229i32 as u16;
    if word!(game.samus_reserve_health) == 0 {
        v0 = -26217i32 as u16;
    }
    let v1 = RomPtr_82(v0, game) as *const u16;
    word! {game; hud_tilemap[8] = *v1; hud_tilemap[9] = *v1.offset(1);
    hud_tilemap[40] = *v1.offset(2); hud_tilemap[41] = *v1.offset(3);
    hud_tilemap[72] = *v1.offset(4); hud_tilemap[73] = *v1.offset(5); }
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_Off(game: &mut Game) {
    word! { game, 11279;
        hud_tilemap[8], hud_tilemap[9], hud_tilemap[40], hud_tilemap[41], hud_tilemap[72],
        hud_tilemap[73],
    }
}
static kReserveTankEnergyTransferPerFrame: u16 = 1;
pub unsafe fn EquipmentScreenCategory_Tanks_1(game: &mut Game) {
    if word!(game.pausemenu_reserve_tank_delay_ctr) == 0 {
        if (word!(game.joypad1_newkeys) as i32 & kButton_A as i32) == 0 {
            return;
        }
        word!(
            game.pausemenu_reserve_tank_delay_ctr =
                (word!(game.samus_reserve_health) as i32 + 7 & 0xfff8) as u16
        );
    }
    if (word!(game.pausemenu_reserve_tank_delay_ctr -= 1, reread) & 7) == 7 {
        QueueSfx3_Max6(0x2d, game);
    }
    word!(game.samus_health += kReserveTankEnergyTransferPerFrame);
    if ((word!(game.samus_health)) as i16).wrapping_sub((word!(game.samus_max_health)) as i16) < 0 {
        word!(game.samus_reserve_health -= kReserveTankEnergyTransferPerFrame);
        if (word!(game.samus_reserve_health) as i16) > 0 {
            return;
        }
        word!(game.samus_health += word!(game.samus_reserve_health));
    } else {
        word!(game.samus_health = word!(game.samus_max_health));
    }
    word! { game, 0; samus_reserve_health, pausemenu_reserve_tank_delay_ctr }
    EquipmentScreenEnergyArrowGlow_Off(game);
    word!(game.pausemenu_equipment_category_item = 0);
}
pub unsafe fn EquipmentScreenCategory_Weapons(game: &mut Game) {
    EquipmentScreenCategory_Weapons_MoveButtons(game);
    let R36: u16 = word!(game.equipped_beams);
    if word!(game.collected_beams) != 0
        && (word!(game.pausemenu_equipment_category_item) as u8) == 1
    {
        EquipmentScreenCategory_ButtonResponse(10, game);
        EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_MoveButtons(game: &mut Game) {
    let r18: u16 = word!(game.pausemenu_equipment_category_item);
    if (word!(game.joypad1_newkeys) & 0x100) != 0 {
        if (word!(game.joypad1_newkeys) & 0x800) != 0 {
            EquipmentScreenMoveLowerOnSuitsMisc(0, game);
        } else if EquipmentScreenMoveLowerOnSuitsMisc(4, game) != 0 {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0, r18, game);
        }
    } else if (word!(game.joypad1_newkeys) & 0x400) != 0 {
        if word!(game.hyper_beam_flag) == 0 && word!(game.pausemenu_equipment_category_item) != 1025
        {
            word!(game.pausemenu_equipment_category_item += 256);
            EquipmentScreenMoveToBeams(
                (2u16).wrapping_mul((byte!(@pointer ram_ptr!(game, u16; 0x755); 1)) as u16),
                r18,
                game,
            );
        }
    } else if (word!(game.joypad1_newkeys) & 0x800) != 0
        && ((word!(game.pausemenu_equipment_category_item) & 0xff00) == 0 || {
            word!(game.pausemenu_equipment_category_item -= 256);
            EquipmentScreenMoveToBottomOfBeams(
                (2u16).wrapping_mul((byte!(@pointer ram_ptr!(game, u16; 0x755); 1)) as u16),
                game,
            ) == 0xffff
        })
        && EquipmentScreenMoveToReserveTanks(game) == 0
    {
        word!(game.pausemenu_equipment_category_item = r18);
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36: u16, game: &mut Game) {
    let t: i32 = word!(game.equipped_beams) as i32 & !(R36 as i32);
    if (t & 4) != 0 {
        if (R36 & 4) == 0 && (word!(game.equipped_beams) & 8) != 0 {
            word!(game.equipped_beams &= !(8));
            ChangePaletteValues(
                ram_ptr!(game, u16; crate::rtl::rom_table_word!(game, 0x82c06c, 4) as i32 as isize),
                3072,
                5 as size_t,
            );
        }
    } else if (t & 8) != 0 && (R36 & 8) == 0 && (word!(game.equipped_beams) & 4) != 0 {
        word!(game.equipped_beams &= !(4));
        ChangePaletteValues(
            ram_ptr!(game, u16; crate::rtl::rom_table_word!(game, 0x82c06c, 3) as i32 as isize),
            3072,
            5 as size_t,
        );
    }
}
pub unsafe fn EquipmentScreenCategory_Suit(game: &mut Game) {
    EquipmentScreenCategory_Suit_MoveResponse(game);
    if (word!(game.pausemenu_equipment_category_item) as u8) == 2 {
        EquipmentScreenCategory_ButtonResponse(18, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Suit_MoveResponse(game: &mut Game) {
    let r18: u16 = word!(game.pausemenu_equipment_category_item);
    if (word!(game.joypad1_newkeys) as i32 & kButton_Left as i32) != 0 {
        if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) == 0 {
            if EquipmentScreenMoveToReserveTanks(game) != 0 {
                return;
            }
            word!(game.pausemenu_equipment_category_item = r18);
        }
        EquipmentScreenMoveToBeams(0, r18, game);
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
        if (word!(game.pausemenu_equipment_category_item) & 0xff00) != 0 {
            word!(game.pausemenu_equipment_category_item -= 256);
            EquipmentScreenMoveToScrewOrHigherOnSuits(
                (2u16).wrapping_mul((byte!(@pointer ram_ptr!(game, u16; 0x755); 1)) as u16),
                r18,
                game,
            );
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0
        && ((word!(game.pausemenu_equipment_category_item) & 0xff00) == 1280 || {
            word!(game.pausemenu_equipment_category_item += 256);
            EquipmentScreenMoveLowerOnSuitsMisc(
                (2u16).wrapping_mul((byte!(@pointer ram_ptr!(game, u16; 0x755); 1)) as u16),
                game,
            ) == 0xffff
        })
    {
        EquipmentScreenMoveToHighJumpOrLowerInBoots(0, r18, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Boots(game: &mut Game) {
    EquipmentScreenCategory_Boots_MoveResponse(game);
    if (word!(game.pausemenu_equipment_category_item) as u8) == 3 {
        EquipmentScreenCategory_ButtonResponse(18, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveResponse(game: &mut Game) {
    let r18: u16 = word!(game.pausemenu_equipment_category_item);
    if (word!(game.joypad1_newkeys) as i32 & kButton_Left as i32) != 0 {
        if ((word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0
            || EquipmentScreenMoveToBottomOfBeams(8, game) == 0xffff)
            && EquipmentScreenMoveToReserveTanks(game) == 0
        {
            word!(game.pausemenu_equipment_category_item = r18);
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
        if word!(game.pausemenu_equipment_category_item) != 515 {
            word!(game.pausemenu_equipment_category_item += 256);
            EquipmentScreenMoveToHighJumpOrLowerInBoots(
                (2u16).wrapping_mul((byte!(@pointer ram_ptr!(game, u16; 0x755); 1)) as u16),
                r18,
                game,
            );
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0
        && ((word!(game.pausemenu_equipment_category_item) & 0xff00) == 0 || {
            word!(game.pausemenu_equipment_category_item -= 256);
            EquipmentScreenCategory_Boots_MoveUpInBoots(
                (2u16).wrapping_mul((byte!(@pointer ram_ptr!(game, u16; 0x755); 1)) as u16),
                game,
            ) == 0xffff
        })
    {
        EquipmentScreenMoveToScrewOrHigherOnSuits(0xa, r18, game);
    }
}

macro_rules! queue_ram_tilemap_write {
    (
        $game:ident, $byte_count:literal, $source_offset:literal, $bank_write_offset:ident,
        $destination:literal $(,)?
    ) => {
        let mut v0: u16 = word!($game.vram_write_queue_tail);
        (*gVramWriteEntry(word!($game.vram_write_queue_tail) as i32, $game)).size =
            $byte_count;
        v0 = (v0).wrapping_add(2);
        (*gVramWriteEntry(v0 as i32, $game)).size =
            ((ram_ptr!($game, Ram3800; 0x3800) as *mut u8)
                .offset_from(&raw mut $game.g_ram as *mut u8) as c_long
                + $source_offset as c_long) as u16;
        v0 = (v0).wrapping_add(2);
        crate::update_place! { previous $bank_write_offset = v0 => v0.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry
            as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            $bank_write_offset as i32,
            $game,
        ))
        .size as *mut u8)
             = 126;
        (*gVramWriteEntry(v0 as i32, $game)).size = $destination;
        word!($game.vram_write_queue_tail = (v0).wrapping_add(2));
    };
}
pub(crate) use queue_ram_tilemap_write;

pub unsafe fn WriteSamusWireframeTilemapAndQueue(game: &mut Game) {
    WriteSamusWireframeTilemap(game);
    queue_ram_tilemap_write!(game, 1280, 256, fresh63, 0x3080);
}
pub unsafe fn WriteSamusWireframeTilemap(game: &mut Game) {
    let mut i: u16 = 0;
    while word!(game.equipped_items) & 0x101
        != crate::rtl::rom_table_word!(game, 0x82b257, (i >> 1) as isize)
    {
        i = (i).wrapping_add(2);
    }
    let mut src = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x82b25f, (i >> 1) as isize),
        game,
    ) as *const u16;
    let v1: u16 = 0;
    let mut v2: u16 = 472;
    for _ in 0..17 {
        let v3: u16 = v2;
        for _ in 0..8 {
            crate::update_place! { previous fresh64 = src => src.offset(1); }
            *((&raw mut (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap as *mut u16
                as *mut u8)
                .offset(v2 as i32 as isize) as *mut u16) = *fresh64;
            v2 = (v2).wrapping_add(2);
        }
        v2 = (v3).wrapping_add(64);
    }
}
pub unsafe fn EquipmentScreenDrawItemSelector(game: &mut Game) {
    if word!(game.samus_max_reserve_health)
        | ((word!(game.collected_items)) | (word!(game.collected_beams)))
        != 0
    {
        let t = RomPtr_82(
            (crate::rtl::rom_table_word!(
                game,
                0x82c18e,
                word!(game.pausemenu_equipment_category_item) as u8 as isize
            ) as i32
                + 4 * byte!(@pointer ram_ptr!(game, u16; 0x755); 1) as i32) as u16,
            game,
        ) as *mut u16;
        DrawPauseScreenSpriteAnim(3, (*t).wrapping_sub(1), *t.offset(1), game);
    }
}
pub unsafe fn EquipmentScreenDisplayReserveTankAmount(game: &mut Game) {
    EquipmentScreenDisplayReserveTankAmount_(game);
}
static kEquipmentScreenReserveTank_X: [u16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
static kEquipmentScreenReserveTank_Y: u16 = 0x60;
static kPartialReserveTankSpritemapIds: [u16; 16] = [
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
];
pub unsafe fn EquipmentScreenDisplayReserveTankAmount_(game: &mut Game) {
    let r3: u16 = EquipmentScreenDisplayReserves_PaletteSetup(game);
    let mut r52: i32 = 0;
    if word!(game.samus_max_reserve_health) == 0 {
        return;
    }
    let R44: u16 = (word!(game.samus_max_reserve_health) as i32 / 100) as u16;
    let mod_value: i32 = word!(game.samus_reserve_health) as i32 % 100;
    let R50: u16 = mod_value as u16;
    let r42: i32 = word!(game.samus_reserve_health) as i32 / 100;
    let mut R48: u16 = r42 as u16;
    let mut r46: i32 = r42;
    if r42 != 0 {
        let mut v0: u16 = 0;
        do_until! { (r46 == 0);
            DrawMenuSpritemap(
                0x1b,
                kEquipmentScreenReserveTank_X[(v0 >> 1) as usize],
                (kEquipmentScreenReserveTank_Y).wrapping_sub(1),
                r3,
                game,
            );
            v0 = (v0).wrapping_add(2);
            r46 -= 1;
        }
        r52 = v0 as i32;
    }
    let RegWord: u16 = mod_value as u16;
    if RegWord != 0 {
        let mut v2: u16 = (2 * (RegWord as i32 / 14)) as u16;
        if v2 as i32 - 7i32 & 0x8000 != 0
            && RegWord as i32 % 14 != 0
            && (*ram_ptr!(game; 0x5b5) & 4) == 0
        {
            v2 = (v2).wrapping_add(2);
        }
        if word!(game.samus_reserve_health) as i32 - 100i32 & 0x8000 == 0 {
            v2 = (v2).wrapping_add(16);
        }
        DrawMenuSpritemap(
            kPartialReserveTankSpritemapIds[(v2 >> 1) as usize],
            kEquipmentScreenReserveTank_X[(r52 >> 1) as usize],
            (kEquipmentScreenReserveTank_Y).wrapping_sub(1),
            r3,
            game,
        );
        R48 = R48.wrapping_add(1);
        r52 += 2;
    }
    while R48 as i32 - R44 as i32 & 0x8000 != 0 {
        DrawMenuSpritemap(
            0x20,
            kEquipmentScreenReserveTank_X[(r52 >> 1) as usize],
            (kEquipmentScreenReserveTank_Y).wrapping_sub(1),
            r3,
            game,
        );
        r52 += 2;
        R48 = R48.wrapping_add(1);
    }
    DrawMenuSpritemap(
        0x1f,
        kEquipmentScreenReserveTank_X[(r52 >> 1) as usize],
        (kEquipmentScreenReserveTank_Y).wrapping_sub(1),
        r3,
        game,
    );
    let div_val: i32 = R50 as i32 / 10;
    let mod_val: i32 = R50 as i32 % 10;
    (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[394i32 as usize] =
        (mod_val + 2052) as u16;
    (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[393i32 as usize] =
        (div_val + 2052) as u16;
    (*(ram_ptr!(game, Ram3800; 0x3800))).cinematic_bg_tilemap[392i32 as usize] =
        (r42 + 2052) as u16;
}
pub unsafe fn EquipmentScreenDisplayReserves_PaletteSetup(game: &mut Game) -> u16 {
    if word!(game.samus_reserve_health) != 0 {
        let v0: bool =
            (word!(game.pausemenu_reserve_tank_animation_timer -= 1, reread) & 0x8000) != 0;
        if word!(game.pausemenu_reserve_tank_animation_timer) == 0 || v0 {
            word!(game.pausemenu_reserve_tank_animation_frame += 1);
            let mut v1: u16 = byte!(@pointer rom_ptr!(game; 0x82c165); (2 * word!(game.pausemenu_reserve_tank_animation_frame) as i32) as isize)
                as u16;
            if v1 == 255 {
                word!(game.pausemenu_reserve_tank_animation_frame = 0);
                v1 = byte!(@pointer rom_ptr!(game; 0x82c165); 0) as u16;
            }
            word!(game.pausemenu_reserve_tank_animation_timer = v1);
        }
        crate::rtl::rom_table_word!(game, 0x82c0fa, 3)
    } else {
        1536
    }
}
pub unsafe fn EquipmentScreenMoveToReserveTanks(game: &mut Game) -> u16 {
    let result: u16 = word!(game.samus_max_reserve_health);
    if word!(game.samus_max_reserve_health) != 0 {
        word!(game.pausemenu_equipment_category_item = 0);
        QueueSfx1_Max6(0x37, game);
        return 1;
    }
    result
}
// Scan the original word-offset tables, including their signed wraparound gates.
// A missing item either preserves the selection or restores the caller's saved word.
unsafe fn move_equipment_selection(
    category: u16,
    mut offset: u16,
    upward: bool,
    fallback: Option<u16>,
    game: &mut Game,
) -> u16 {
    let (table, limit) = match category {
        1 => (0xc04c, 10),
        2 => (0xc056, 10),
        3 => (0xc062, 6),
        _ => unreachable!(),
    };
    let mut found = category != 1 || word!(game.hyper_beam_flag) == 0;
    while found {
        let mask = crate::rtl::read_rom_table_word(
            RomPtr_82(table, game).cast::<u16>(),
            (offset >> 1) as isize,
        );
        let collected = if category == 1 {
            word!(game.collected_beams)
        } else {
            word!(game.collected_items)
        };
        if mask & collected != 0 {
            break;
        }
        crate::verify_probe::hit("equipment-search-skip");
        offset = offset.wrapping_add(if upward { 0xfffe } else { 2 });
        found = if upward {
            offset & 0x8000 == 0
        } else {
            (offset as i16).wrapping_sub(limit) < 0
        };
    }
    if !found {
        crate::verify_probe::hit("equipment-search-empty");
        if let Some(previous) = fallback {
            word!(game.pausemenu_equipment_category_item = previous);
        }
        return 0xffff;
    }
    let selection = ((offset >> 1) << 8) | category;
    // Downward beam selection stores before queuing sound; the other paths store after it.
    if category == 1 && !upward {
        word!(game.pausemenu_equipment_category_item = selection);
        QueueSfx1_Max6(0x37, game);
    } else {
        QueueSfx1_Max6(0x37, game);
        word!(game.pausemenu_equipment_category_item = selection);
    }
    if category == 2 && !upward {
        0
    } else {
        selection
    }
}
pub unsafe fn EquipmentScreenMoveToBeams(v0: u16, r18: u16, game: &mut Game) {
    crate::verify_probe::hit("equipment-search-beams-down");
    move_equipment_selection(1, v0, false, Some(r18), game);
}
pub unsafe fn EquipmentScreenMoveToBottomOfBeams(k: u16, game: &mut Game) -> u16 {
    crate::verify_probe::hit("equipment-search-beams-up");
    move_equipment_selection(1, k, true, None, game)
}
pub unsafe fn EquipmentScreenMoveLowerOnSuitsMisc(v0: u16, game: &mut Game) -> u16 {
    crate::verify_probe::hit("equipment-search-suits-down");
    move_equipment_selection(2, v0, false, None, game)
}
pub unsafe fn EquipmentScreenMoveToScrewOrHigherOnSuits(v0: u16, r18: u16, game: &mut Game) {
    crate::verify_probe::hit("equipment-search-suits-up");
    move_equipment_selection(2, v0, true, Some(r18), game);
}
pub unsafe fn EquipmentScreenMoveToHighJumpOrLowerInBoots(v0: u16, r18: u16, game: &mut Game) {
    crate::verify_probe::hit("equipment-search-boots-down");
    move_equipment_selection(3, v0, false, Some(r18), game);
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveUpInBoots(k: u16, game: &mut Game) -> u16 {
    crate::verify_probe::hit("equipment-search-boots-up");
    move_equipment_selection(3, k, true, None, game)
}
pub unsafe fn EquipmentScreenCategory_ButtonResponse(r24: u16, game: &mut Game) {
    if (word!(game.joypad1_newkeys) as i32 & kButton_A as i32) != 0 {
        QueueSfx1_Max6(0x38, game);
        let item: i32 = byte!(@pointer ram_ptr!(game, u16; 0x755); 1) as i32;
        let category: i32 = word!(game.pausemenu_equipment_category_item) as u8 as i32;
        let target: *mut u8 = ram_ptr!(game; *(RomPtr_82(
                (crate::rtl::rom_table_word!(game, 0x82c02c, category as isize) as i32
                    + item * 2) as u16,
                game,
            ) as *mut u16) as i32 as isize,);
        let var = RomPtr_RAM(
            crate::rtl::rom_table_word!(game, 0x82c03c, category as isize),
            game,
        ) as *mut u16;
        let mask: u16 = *(RomPtr_82(
            (crate::rtl::rom_table_word!(game, 0x82c034, category as isize) as i32 + item * 2)
                as u16,
            game,
        ) as *mut u16);
        if (*var & mask) != 0 {
            *var = (*var & !(mask));
            ChangePaletteValues(target as *mut u16, 0xc00, (r24 as i32 >> 1) as size_t);
        } else {
            *var = (*var) | (mask);
            let src: u16 = *(RomPtr_82(
                (crate::rtl::rom_table_word!(game, 0x82c044, category as isize) as i32 + item * 2)
                    as u16,
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
    if word!(game.menu_index) != 0 {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
    } else {
        MapScreenDrawSamusPositionIndicator(game);
        DrawMapIcons(game);
        DisplayMapElevatorDestinations(game);
    };
}
static kMap_Criteria_SavePoints: [u16; 16] = [
    0xd8, 0x28, 0x90, 0x38, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe,
    0xfffe, 0xfffe, 0xfffe,
];
pub unsafe fn DrawMapIcons(game: &mut Game) {
    DrawBossMapIcons(9, addr_kMapIconDataPointers as u16, game);
    DrawSimpleMapIcons(
        0xb,
        (addr_kMapIconDataPointers as i32 + 0x10) as u16,
        3584,
        game,
    );
    DrawSimpleMapIcons(
        0xa,
        (addr_kMapIconDataPointers as i32 + 0x20) as u16,
        3584,
        game,
    );
    DrawSimpleMapIcons(
        0x4e,
        (addr_kMapIconDataPointers as i32 + 0x30) as u16,
        3584,
        game,
    );
    DrawSaveStationMapIcon(8, 0xc80b, 1024, game);
    if word!(game.enable_debug) != 0 {
        DrawSimpleMapIcons(8, 0xc82b, 1024, game);
    }
    if word!(game.area_index) == 0 {
        DrawMenuSpritemap(
            0x63,
            (kMap_Criteria_SavePoints[0i32 as usize]).wrapping_sub(word!(game.reg_BG1HOFS)),
            (kMap_Criteria_SavePoints[1i32 as usize]).wrapping_sub(word!(game.reg_BG1VOFS)),
            3584,
            game,
        );
    }
}
pub unsafe fn DrawFileSelectMapIcons(game: &mut Game) {
    HandlePauseScreenPaletteAnimation(game);
    DrawBossMapIcons(9, 0xc7cb, game);
    DrawSimpleMapIcons(0xb, 0xc7db, 3584, game);
    DrawSimpleMapIcons(0xa, 0xc7eb, 3584, game);
    DrawSimpleMapIcons(0x4e, 0xc7fb, 3584, game);
    let a: u16 = UpdateSamusPositionIndicatorAnimation(game);
    let data: *const u16 = (RomPtr_82(
        *((&raw mut (*(rom_ptr!(game, MapIconDataPointers; 0x82c7cb)).offset(4)).crateria
            as *mut u8)
            .offset((2i32 * word!(game.area_index) as i32) as isize) as *mut VoidP),
        game,
    ) as *mut u16)
        .offset((word!(game.load_station_index) as i32 * 2) as isize);
    let v1: u16 = (*data).wrapping_sub(word!(game.reg_BG1HOFS));
    let v2: u16 = (*data.offset(1)).wrapping_sub(word!(game.reg_BG1VOFS));
    if (word!(game.samus_position_indicator_animation_loop_counter) & 1) == 0 {
        DrawMenuSpritemap(0x12, v1, v2, 3584, game);
    }
    DrawMenuSpritemap(a, v1, v2, 3584, game);
    if word!(game.enable_debug) != 0 {
        DrawDebugSaveMapIcons(0xc, 0xc80b, 1536, game);
        DrawDebugElevatorMapIcons(0x17, 0xc81b, 1536, game);
        DrawSimpleMapIcons(0xc, 0xc82b, 1536, game);
    }
    if word!(game.area_index) == 0 {
        DrawMenuSpritemap(
            0x63,
            (kMap_Criteria_SavePoints[0i32 as usize]).wrapping_sub(word!(game.reg_BG1HOFS)),
            (kMap_Criteria_SavePoints[1i32 as usize]).wrapping_sub(word!(game.reg_BG1VOFS)),
            3584,
            game,
        );
    }
}
pub unsafe fn DrawSaveStationMapIcon(a: u16, k: u16, r3: u16, game: &mut Game) {
    static kShlBit: [u8; 8] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80];
    let R34: u16 = a;
    if word!(game.area_index) == word!(game.sram_area_index) {
        let mut R36: u16 =
            *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
        byte!(R36; 0) = ((R36) as u8) & (kShlBit[word!(game.load_station_index) as usize]);
        let v2: u16 =
            *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
        if v2 != 0 {
            DrawMapIconsOfType(v2, R34, R36, r3, game);
        }
    }
}
pub unsafe fn DrawDebugSaveMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    let R34: u16 = a;
    let R36: u16 = *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
    let v2: u16 =
        *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, R36, r3, game);
    }
}
pub unsafe fn DrawDebugElevatorMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    let R34: u16 = a;
    let R36: u16 = *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32 + 1) as isize));
    let v2: u16 =
        *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, R36, r3, game);
    }
}
pub unsafe fn DrawSimpleMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    let R34: u16 = a;
    let v2: u16 =
        *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, -1i32 as u16, r3, game);
    }
}
pub unsafe fn DrawMapIconsOfType(mut a: u16, r34: u16, mut r36: u16, r3: u16, game: &mut Game) {
    loop {
        let v2 = RomPtr_82(a, game) as *const u16;
        if (*v2 & 0x8000) != 0 {
            break;
        }
        let mut v3: i8 = ((r36) as i8) & (1i8);
        r36 = (r36 >> 1);
        if v3 != 0 {
            let mut v4: i16 = CheckIfMapPositionIsExplored(*v2, *v2.offset(1), game) as i16;
            if v4 != 0 {
                DrawMenuSpritemap(
                    r34,
                    (*v2).wrapping_sub(word!(game.reg_BG1HOFS)),
                    (*v2.offset(1)).wrapping_sub(word!(game.reg_BG1VOFS)),
                    r3,
                    game,
                );
            }
        }
        a = (a).wrapping_add(4);
    }
}
pub unsafe fn CheckIfMapPositionIsExplored(k: u16, j: u16, game: &mut Game) -> u16 {
    static kBits0x80Shr: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
    let mut r18: i32 = ((k) & (0xff00u16)) as i32 >> 1;
    r18 += k as u8 as i32 >> 6;
    r18 += ((j) & (0xfff8u16)) as i32 >> 1;
    let v3: u16 = (k as i32 >> 3 & 7) as u16;
    ((kBits0x80Shr[v3 as usize] & *ram_ptr!(game; 0x7f7, r18 as isize)) != 0) as u16
}
pub unsafe fn DrawBossMapIcons(a: u16, k: u16, game: &mut Game) {
    let mut bits: i32 = *ram_ptr!(game; 0xd828, word!(game.area_index) as isize) as i32;
    let t: i32 = *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game)
        as *mut u16) as i32;
    if t == 0 {
        return;
    }
    let mut v4 = RomPtr_82(t as u16, game) as *const u16;
    loop {
        if *v4 == 0xffff {
            return;
        }
        let mut advance_boss_bit: bool = !(*v4 != 0xfffe)
            || ({
                let v5: i32 = bits & 1;
                bits >>= 1;
                if v5 != 0 {
                    DrawMenuSpritemap(
                        0x62,
                        (*v4).wrapping_sub(word!(game.reg_BG1HOFS)),
                        (*v4.offset(1)).wrapping_sub(word!(game.reg_BG1VOFS)),
                        3584,
                        game,
                    );
                    DrawMenuSpritemap(
                        a,
                        (*v4).wrapping_sub(word!(game.reg_BG1HOFS)),
                        (*v4.offset(1)).wrapping_sub(word!(game.reg_BG1VOFS)),
                        3072,
                        game,
                    );
                    false
                } else {
                    !(word!(game.has_area_map) != 0)
                        || ({
                            DrawMenuSpritemap(
                                a,
                                (*v4).wrapping_sub(word!(game.reg_BG1HOFS)),
                                (*v4.offset(1)).wrapping_sub(word!(game.reg_BG1VOFS)),
                                3584,
                                game,
                            );
                            false
                        })
                }
            });
        if advance_boss_bit {
            bits >>= 1;
        }
        v4 = v4.offset(2);
    }
}
pub unsafe fn DrawMapScrollArrowAndCheckToScroll(db: u8, k: u16, game: &mut Game) {
    let v1 = RomPtrWithBank(db, k, game) as *const u16;
    DrawPauseScreenSpriteAnim(*v1.offset(2), *v1, *v1.offset(1), game);
    let v2 = RomPtrWithBank(db, k, game);
    if (word!(game.joypad1_lastkeys) & *(v2.offset(6) as *mut u16)) != 0
        && word!(game.map_scrolling_direction) == 0
    {
        word!(game.map_scrolling_direction = *(v2.offset(8) as *mut u16));
    }
}
pub unsafe fn HandleMapScrollArrows(game: &mut Game) {
    if word!(game.map_min_x_scroll) as i32 - 24i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 != 0 {
        DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9A0 as u16, game);
    }
    if word!(game.map_max_x_scroll) as i32 - 232i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 == 0 {
        DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9AA as u16, game);
    }
    if word!(game.map_min_y_scroll) as i32 - 56i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0 {
        DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9B4 as u16, game);
    }
    if word!(game.map_max_y_scroll) as i32 - 177i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0 {
        if word!(game.map_scrolling_direction)
            == (*(rom_ptr!(game, MapScrollArrowData; 0x82b9be))).map_scroll_dir
        {
            word! { game, 0;
                map_scrolling_gear_switch_timer, map_scrolling_direction, map_scrolling_speed_index,
            }
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9BE as u16, game);
    };
}
pub unsafe fn MapScreenDrawSamusPositionIndicator(game: &mut Game) {
    let a: u16 = UpdateSamusPositionIndicatorAnimation(game);
    DrawMenuSpritemap(
        a,
        (8 * (word!(game.room_x_coordinate_on_map) as i32
            + byte!(@pointer ram_ptr!(game, u16; 0xaf6); 1) as i32)
            - word!(game.reg_BG1HOFS) as i32) as u16,
        (8 * (word!(game.room_y_coordinate_on_map) as i32
            + byte!(@pointer ram_ptr!(game, u16; 0xafa); 1) as i32
            + 1)
            - word!(game.reg_BG1VOFS) as i32) as u16,
        3584,
        game,
    );
}
static kMap_SamusPositionIndicator_Delays: [u16; 4] = [8, 4, 8, 4];
static kMap_SamusPositionIndicator_SpritemapIds: [u16; 4] = [0x5f, 0x60, 0x61, 0x60];
pub unsafe fn UpdateSamusPositionIndicatorAnimation(game: &mut Game) -> u16 {
    let mut v0: u16 = word!(game.samus_position_indicator_animation_timer);
    if v0 == 0 {
        let mut v1: u16 = (word!(game.samus_position_indicator_animation_frame)).wrapping_add(2);
        if ((v1) as i16).wrapping_sub(8) >= 0 {
            word!(game.samus_position_indicator_animation_loop_counter += 1);
            v1 = 0;
        }
        word!(game.samus_position_indicator_animation_frame = v1);
        v0 = kMap_SamusPositionIndicator_Delays[(v1 >> 1) as usize];
        word!(game.samus_position_indicator_animation_timer = v0);
    }
    word!(game.samus_position_indicator_animation_timer = (v0).wrapping_sub(1));
    kMap_SamusPositionIndicator_SpritemapIds
        [(word!(game.samus_position_indicator_animation_frame) >> 1) as usize]
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    DrawBorderAroundSamusData => DrawMenuSpritemap(0x48, 0x80, 0x10, 3584, game);
    DrawBorderAroundDataCopyMode => DrawMenuSpritemap(0x49, 0x80, 0x10, 3584, game);
    DrawBorderAroundDataClearMode => DrawMenuSpritemap(0x4a, 0x7c, 0x10, 3584, game);
}
static kDrawMenuSelectionMissile_Enable: [u16; 4] = [8, 8, 8, 8];
static kDrawMenuSelectionMissile_SpriteMap: [u16; 4] = [0x37, 0x36, 0x35, 0x34];
pub unsafe fn DrawMenuSelectionMissile(game: &mut Game) {
    if word!(game.cinematic_var5) != 0 {
        if word!(game.cinematic_var5 -= 1, reread) == 0 {
            word! { game;
                eproj_id[0] = (*((ram_ptr!(game, u16; 0x1997)) as *mut u8) as i32 + 1 & 3) as u16;
                cinematic_var5 = kDrawMenuSelectionMissile_Enable[word!(game.eproj_id[0]) as usize];
            }
        }
    }
    DrawMenuSpritemap(
        kDrawMenuSelectionMissile_SpriteMap[word!(game.eproj_id[0]) as usize],
        word!(game.eproj_id[5]),
        word!(game.eproj_id[10]),
        3584,
        game,
    );
}
pub unsafe fn DrawFileCopyArrow(game: &mut Game) {
    let mut v0: u16 = 0;
    if ((word!(game.eproj_id[16])) as i16).wrapping_sub((word!(game.eproj_id[17])) as i16) >= 0 {
        v0 = 3;
        if word!(game.eproj_id[16]) as i32 - word!(game.eproj_id[17]) as i32 - 2i32 & 0x8000 != 0 {
            v0 = 4;
            if word!(game.eproj_id[16]) != 1 {
                v0 = 5;
            }
        }
    } else {
        v0 = 0;
        if word!(game.eproj_id[17]) as i32 - word!(game.eproj_id[16]) as i32 - 2i32 & 0x8000 != 0 {
            v0 = 1;
            if word!(game.eproj_id[16]) != 0 {
                v0 = 2;
            }
        }
    }
    DrawMenuSpritemap(
        (*(rom_ptr!(game, FileCopyArrowStuff; 0x82bb0c)).offset(v0 as isize)).spritemap,
        (*(rom_ptr!(game, FileCopyArrowStuff; 0x82bb0c)).offset(v0 as isize)).xpos,
        (*(rom_ptr!(game, FileCopyArrowStuff; 0x82bb0c)).offset(v0 as isize)).ypos,
        512,
        game,
    );
}
pub unsafe fn DisplayMapElevatorDestinations(game: &mut Game) {
    if *ram_ptr!(game; 0xd908, word!(game.area_index) as isize) != 0 {
        let mut i: i32 =
            crate::rtl::rom_table_word!(game, 0x82c74d, word!(game.area_index) as isize) as i32;
        loop {
            let v1 = RomPtr_82(i as u16, game);
            if *(v1 as *mut u16) == 0xffff {
                break;
            }
            DrawMenuSpritemap(
                *(v1.offset(4) as *mut u16),
                (*(v1 as *mut u16)).wrapping_sub(word!(game.reg_BG1HOFS)),
                (*(v1.offset(2) as *mut u16)).wrapping_sub(word!(game.reg_BG1VOFS)),
                0,
                game,
            );
            i += 6;
        }
    }
}
pub unsafe fn HandleGameOverBabyMetroid(game: &mut Game) {
    if (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer != 0 {
        sub_82BB7F(
            (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer,
            game,
        );
    } else {
        sub_82BBDD(game);
    };
}
crate::game::callback_dispatch! {
    pub unsafe fn CallBabyMetroidPlaySfx(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [8567820] BabyMetroidPlaySfx0x23 [] {
        QueueSfx3_Max6(0x23, game);
        FinishProcessingGameOverBabyMetroidAsm(game);
    };
    [8567829] BabyMetroidPlaySfx0x26 [] {
        QueueSfx3_Max6(0x26, game);
        FinishProcessingGameOverBabyMetroidAsm(game);
    };
    [8567838] BabyMetroidPlaySfx0x27 [] {
        QueueSfx3_Max6(0x27, game);
        FinishProcessingGameOverBabyMetroidAsm(game);
    };
}
pub unsafe fn sub_82BB7F(a: u16, game: &mut Game) {
    let current_instruction: u16 = (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction;
    (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer = (a).wrapping_sub(1);
    if a == 1 {
        let v2 = RomPtr_82(
            (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction,
            game,
        );
        if *(v2.offset(6) as *mut u16) == 0xffff {
            sub_82BBDD(game);
        } else if ((*(v2.offset(6) as *mut u16)) as i16).wrapping_add(1) >= 0 {
            (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer = *(v2.offset(6) as *mut u16);
            let fresh18 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction;
            fresh18.write_unaligned((fresh18.read_unaligned()).wrapping_add(6));
            DrawBabyMetroid((current_instruction).wrapping_add(6), game);
        } else {
            CallBabyMetroidPlaySfx(((*(v2.offset(6) as *mut u16)) as u32) | (0x820000u32), game);
        }
    } else {
        DrawBabyMetroid(
            (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction,
            game,
        );
    };
}
pub unsafe fn sub_82BBDD(game: &mut Game) {
    enemy_instruction! {[(*(ram_ptr!(game, EnemyData; 0xf78)))]; instruction = addr_kgameOverBabyMetridInstructionList as u16, timer = 10}
    sub_82BB7F(0xa, game);
}
pub unsafe fn DrawBabyMetroid(k: u16, game: &mut Game) {
    let v1: u16 = *(RomPtr_82(k, game) as *mut u16).offset(2);
    for i in (0i32..32).step_by(2) {
        word!(
            game.palette_buffer[((i >> 1) + 192)] =
                *(RomPtr_82((v1 as i32 + i) as u16, game) as *mut u16)
        );
    }
    let v3 = RomPtr_82(k, game);
    DrawMenuSpritemap(*(v3.offset(2) as *mut u16), 0x7c, 0x50, 2048, game);
    DrawMenuSpritemap(0x64, 0x7c, 0x50, 2560, game);
}
pub unsafe fn FinishProcessingGameOverBabyMetroidAsm(game: &mut Game) {
    let t: u16 = *(RomPtr_82(
        (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction,
        game,
    ) as *mut u16)
        .offset(4);
    (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer = t;
    let fresh17 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction;
    fresh17.write_unaligned((fresh17.read_unaligned()).wrapping_add(8));
    if t == 0xffff {
        sub_82BBDD(game);
    } else {
        DrawBabyMetroid(
            (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction,
            game,
        );
    };
}
pub unsafe fn CancelSoundEffects(game: &mut Game) {
    QueueSfx1_Max6(2, game);
    QueueSfx2_Max6(0x71, game);
    QueueSfx3_Max6(1, game);
}
pub unsafe fn QueueSamusMovementSfx(game: &mut Game) {
    if (word!(game.speed_boost_counter) & 0xff00) == 1024 {
        QueueSfx3_Max6(0x2b, game);
    }
    if word!(game.flare_counter) as i32 - 16i32 & 0x8000 == 0 {
        QueueSfx1_Max6(0x41, game);
    }
    CallSomeSamusCode(0x14, game);
}
pub unsafe fn AdvancePaletteFadeForAllPalettes_0xc(game: &mut Game) -> u8 {
    word!(game.palette_change_denom = 12);
    AdvancePaletteFadeForAllPalettes(game)
}
pub unsafe fn AdvancePaletteFade_BgPalette6(game: &mut Game) -> u8 {
    word!(game.palette_change_denom = 12);
    if word!(game.palette_change_num) <= 0xd {
        for i in (192i32..224).step_by(2) {
            word!(
                game.palette_buffer[(i >> 1)] = CalculateNthTransitionColorFromXtoY(
                    word!(game.palette_change_num),
                    word!(game.palette_buffer[(i >> 1)]),
                    word!(game.target_palettes[(i >> 1)]),
                    game,
                )
            );
        }
        word!(game.palette_change_num += 1);
        0
    } else {
        word!(game.palette_change_num = 0);
        1
    }
}
pub unsafe fn AdvanceGradualColorChangeOfPalette(mut k: u16, j: u16, game: &mut Game) -> u8 {
    word!(game.palette_change_denom = 15);
    if (15i16).wrapping_sub((word!(game.palette_change_num)) as i16) >= 0 {
        let R34: u16 = j;
        do_until! { (k >= R34);
            word!(
                game.palette_buffer[(k >> 1)] = CalculateNthTransitionColorFromXtoY(
                    (word!(game.palette_change_num)).wrapping_add(1),
                    word!(game.palette_buffer[(k >> 1)]),
                    word!(game.target_palettes[(k >> 1)]),
                    game,
                )
            );
            k = (k).wrapping_add(2);
        }
        word!(game.palette_change_num += 1);
        0
    } else {
        word!(game.palette_change_num = 0);
        1
    }
}
pub unsafe fn AdvancePaletteFadeForAllPalettes(game: &mut Game) -> u8 {
    if (word!(game.palette_change_denom)).wrapping_add(1) >= word!(game.palette_change_num) {
        let mut i: i32 = 0;
        while i < 0x200 {
            word!(game.g_word_7EC404 = i as u16);
            let v2: i32 = i >> 1;
            if word!(game.target_palettes[v2]) != word!(game.palette_buffer[v2]) {
                let v3: u16 = CalculateNthTransitionColorFromXtoY(
                    word!(game.palette_change_num),
                    word!(game.palette_buffer[v2]),
                    word!(game.target_palettes[v2]),
                    game,
                );
                i = word!(game.g_word_7EC404) as i32;
                *(ram_ptr!(game, u16; 0xc000)).offset((word!(game.g_word_7EC404) >> 1) as isize) =
                    v3;
            }
            i += 2;
        }
        word!(game.palette_change_num += 1);
        0
    } else {
        word!(game.palette_change_num = 0);
        1
    }
}
pub unsafe fn CalculateNthTransitionColorFromXtoY(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    (CalculateNthTransitionColorComponentFromXtoY(a, (k) & (0x1fu16), (j) & (0x1fu16), game) as i32
        | (CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as i32 >> 5 & 0x1f) as u16,
            (j as i32 >> 5 & 0x1f) as u16,
            game,
        ) as i32)
            << 5
        | (CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as i32 >> 10 & 0x1f) as u16,
            (j as i32 >> 10 & 0x1f) as u16,
            game,
        ) as i32)
            << 10) as u16
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
    let v4: u16 = (a).wrapping_sub(1);
    if v4 == word!(game.palette_change_denom) {
        return j;
    }
    let RegWord: u16 = SnesDivide(
        ((abs16((j).wrapping_sub(k)) as i32) << 8) as u16,
        (word!(game.palette_change_denom) as i32 - (v4 as i32 + 1) + 1) as u8,
    );
    let r18: u16 = (if j as i32 - k as i32 & 0x8000 != 0 {
        -(RegWord as i32)
    } else {
        RegWord as i32
    }) as u16;
    ((r18 as i32 + ((k as i32) << 8)) as u16 >> 8)
}
pub unsafe fn sub_82DAF7(a: u16, game: &mut Game) -> u8 {
    word!(game.palette_change_denom = 12);
    sub_82DB0C(a, game)
}
pub unsafe fn sub_82DB0C(mut a: u16, game: &mut Game) -> u8 {
    if (word!(game.palette_change_denom)).wrapping_add(1) >= word!(game.palette_change_num) {
        word!(game.g_word_7EC404 = 0);
        while a != 0 {
            if a & 1 == 0 {
                word!(game.g_word_7EC404 += 32);
            } else {
                sub_82DB41(game);
            }
            a = (a >> 1);
        }
        word!(game.palette_change_num += 1);
        0
    } else {
        word!(game.palette_change_num = 0);
        1
    }
}
pub unsafe fn sub_82DB41(game: &mut Game) {
    let mut v0: u16 = word!(game.g_word_7EC404);
    do_until! { ((v0 & 0x1f) == 0);
        let v1: i32 = v0 as i32 >> 1;
        if word!(game.target_palettes[v1]) != word!(game.palette_buffer[v1]) {
            let v2: u16 = CalculateNthTransitionColorFromXtoY(
                word!(game.palette_change_num),
                word!(game.palette_buffer[v1]),
                word!(game.target_palettes[v1]),
                game,
            );
            v0 = word!(game.g_word_7EC404);
            *(ram_ptr!(game, u16; 0xc000)).offset((word!(game.g_word_7EC404) >> 1) as isize) = v2;
        }
        v0 = (v0).wrapping_add(2);
        word!(game.g_word_7EC404 = v0);
    }
}
pub unsafe fn HandleSamusOutOfHealthAndGameTile(game: &mut Game) {
    if (word!(game.samus_health) as i16) <= 0 {
        if (word!(game.reserve_health_mode) & 1) != 0 && word!(game.samus_reserve_health) != 0 {
            word!(game.time_is_frozen_flag = 0x8000);
            word!(game.game_state = kGameState_27_ReserveTanksAuto as u16);
            CallSomeSamusCode(0x1b, game);
        } else {
            if word!(game.game_state) as i32 != kGameState_8_MainGameplay as i32 {
                return;
            }
            word!(game.time_is_frozen_flag = 0x8000);
            CallSomeSamusCode(0x11, game);
            word!(game.game_state = kGameState_19_SamusNoHealth as u16);
        }
    }
    if word!(game.game_time_frames += 1, reread) as i32 - 60i32 & 0x8000 == 0 {
        word!(game.game_time_frames = 0);
        if word!(game.game_time_seconds += 1, reread) as i32 - 60i32 & 0x8000 == 0 {
            word!(game.game_time_seconds = 0);
            if word!(game.game_time_minutes += 1, reread) as i32 - 60i32 & 0x8000 == 0 {
                word!(game.game_time_minutes = 0);
                word!(game.game_time_hours += 1);
            }
        }
    }
    if word!(game.game_time_hours) as i32 - 100i32 & 0x8000 == 0 {
        word! { game, 59; game_time_frames, game_time_seconds, game_time_minutes }
        word!(game.game_time_hours = 99);
    }
}
pub unsafe fn GameState_27_ReserveTanksAuto(game: &mut Game) -> CoroutineRet {
    if *ram_ptr!(game; 0x612) == 0 && RefillHealthFromReserveTanks(game) != 0 {
        word! {game; time_is_frozen_flag = 0; game_state = kGameState_8_MainGameplay as u16; }
        CallSomeSamusCode(0x10, game);
    }
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    Samus_LowHealthCheck_0(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RefillHealthFromReserveTanks(game: &mut Game) -> u8 {
    if word!(game.samus_reserve_health) != 0 {
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            QueueSfx3_Max3(0x2d, game);
        }
        let refill_in_progress: bool = if ((word!(game.samus_health += 1, reread)) as i16)
            .wrapping_sub((word!(game.samus_max_health)) as i16)
            >= 0
        {
            word!(game.samus_health = word!(game.samus_max_health));
            false
        } else {
            let v0: bool = (word!(game.samus_reserve_health -= 1, reread) & 0x8000) != 0;
            !(word!(game.samus_reserve_health) == 0)
                && (!(v0)
                    || ({
                        word!(game.samus_health += word!(game.samus_reserve_health));
                        false
                    }))
        };
        if refill_in_progress {
        } else {
            word!(game.samus_reserve_health = 0);
            return (word!(game.samus_reserve_health) == 0) as u8;
        }
    }
    (word!(game.samus_reserve_health) == 0) as u8
}
pub unsafe fn GameState_19_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    word_transfers!(game, descending, 256; pause_map(639) <- le(palette_buffer, 255));
    word_transfers!(game, descending, 192; le(target_palettes, 191) <- value(0));
    word_transfers!(game, descending, 48; le(target_palettes, 255) <- value(0));
    word_transfers!(game, descending, 16; le(target_palettes, 207) <- le(palette_buffer, 207));
    word!(game.game_options_screen_index = 3);
    word! { game, 0;
        g_word_7E0DE4, g_word_7E0DE6, g_word_7E0DE8, hud_item_index,
        samus_auto_cancel_hud_item_index, samus_invincibility_timer, samus_knockback_timer,
    }
    word!(game.game_state += 1);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_20_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    let mut _coret_: u8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    word!(game.palette_change_denom = 6);
    if AdvancePaletteFadeForAllPalettes(game) != 0 {
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        word! { game, 0; fx_layer_blending_config_a, cur_irq_handler }
        *ram_ptr!(game; 0x6e) = 0;
        *ram_ptr!(game; 0x71) = 0;
        *ram_ptr!(game; 0x69) = 16;
        *ram_ptr!(game; 0x6b) = 0;
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
        *ram_ptr!(game; 0x55) = 9;
        word! { game, 0;
            game_options_screen_index, screen_fade_delay, screen_fade_counter,
        }
        word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
        word! {game; g_word_7E0DE8 = 16; game_options_screen_index = 3; }
        word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
        word!(game.game_state += 1);
        word!(game.power_bomb_explosion_status = 0);
        QueueSfx1_Max15(2, game);
        QueueSfx2_Max15(0x71, game);
        QueueSfx3_Max15(1, game);
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff39, game);
        QueueMusic_DelayedY(5, 0xe, game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_21_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    Samus_DrawWhenNotAnimatingOrDying(game);
    if HasQueuedMusic(game) == 0 {
        StartSamusDeathAnimation(game);
        word!(game.game_state += 1);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_22_SamusNoHealth_Dying(game: &mut Game) -> CoroutineRet {
    DrawSamusStartingDeathAnim_(game);
    let v0: bool = (word!(game.g_word_7E0DE8 -= 1, reread) & 0x8000) != 0;
    if word!(game.g_word_7E0DE8) == 0 || v0 {
        word!(game.game_state += 1);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_23_SamusNoHealth_Flashing(game: &mut Game) -> CoroutineRet {
    if HandleSamusDeathSequence(game) != 0 {
        word!(game.game_state += 1);
    } else {
        Samus_DrawDuringDeathAnim(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion(game: &mut Game) -> CoroutineRet {
    if GameState_24_SamusNoHealth_Explosion_Helper(game) != 0 {
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_25_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
        word! { game, 0; menu_index, debug_disable_sounds }
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
    if (*ram_ptr!(game; 0x84) & 0x30) != 0x30 {
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
    if word!(game.elevator_flags) == 0 {
        Samus_DrawWhenNotAnimatingOrDying(game);
    }
}
pub unsafe fn LoadEnemyGfxToVram(game: &mut Game) {
    let mut dst: u16 = 0x7000;
    let mut v0: u16 = word!(game.room_enemy_tilesets_ptr);
    if word!(game.room_enemy_tilesets_ptr) != 0 {
        let mut i: i32 = word!(game.room_enemy_tilesets_ptr) as i32;
        loop {
            let enemy_def: u16 = (*get_EnemyTileset(v0, game)).enemy_def;
            if enemy_def == 0xffff {
                break;
            }
            let mut ED: *mut EnemyDef = get_EnemyDef_A2(enemy_def, game);
            let mut vram_update_size: u16 = 0;
            let mut vram_update_dst: u16 = 0;
            if ((*ED).tile_data_size & 0x8000) != 0 {
                vram_update_size = ((*ED).tile_data_size) & (0x7fffu16);
                vram_update_dst =
                    ((((*get_EnemyTileset(i as u16, game)).vram_dst) & (0xf000u16)) as i32 >> 4
                        | 0x6000) as u16;
            } else {
                vram_update_size = (*ED).tile_data_size;
                vram_update_dst = dst;
                dst = (dst as i32 + ((*ED).tile_data_size as i32 >> 1)) as u16;
            }
            CopyToVramNow(
                vram_update_dst,
                Load24(&raw mut (*ED).tile_data),
                vram_update_size,
                game,
            );
            i += 4;
            v0 = i as u16;
        }
    }
}
pub unsafe fn LoadRoomMusic(game: &mut Game) {
    if (word!(game.game_state) as i32) < kGameState_40_TransitionToDemo as i32
        && word!(game.room_music_data_index) != 0
        && word!(game.room_music_data_index) != word!(game.music_data_index)
    {
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8((word!(game.room_music_data_index)) | (0xff00u16), game);
    }
}
pub unsafe fn UpdateMusicTrackIndex(game: &mut Game) {
    if (word!(game.game_state) as i32) < kGameState_40_TransitionToDemo as i32
        && word!(game.room_music_track_index) != 0
    {
        let mut r18: u16 = ((word!(game.room_music_data_index) as i32) << 8) as u16;
        r18 = (r18) | (word!(game.room_music_track_index));
        let mut r20: u16 = ((word!(game.music_data_index) as i32) << 8) as u16;
        r20 = (r20) | (word!(game.music_track_index));
        if r18 != r20 {
            word!(game.music_track_index = word!(game.room_music_track_index));
        }
    }
}
pub unsafe fn LoadNewMusicTrackIfChanged(game: &mut Game) {
    if word!(game.game_state) < 0x28 && word!(game.room_music_track_index) != 0 {
        let mut r18: u16 = ((word!(game.room_music_data_index) as i32) << 8) as u16;
        r18 = (r18) | (word!(game.room_music_track_index));
        let mut r20: u16 = ((word!(game.music_data_index) as i32) << 8) as u16;
        r20 = (r20) | (word!(game.music_track_index));
        if r18 != r20 {
            QueueMusic_DelayedY(word!(game.room_music_track_index), 6, game);
        }
    }
}
pub fn NullFunc() {}
pub unsafe fn PlayRoomMusicTrackAfterAFrames(a: u16, game: &mut Game) {
    if (word!(game.game_state) as i32) < kGameState_40_TransitionToDemo as i32 {
        QueueMusic_DelayedY(0, a, game);
        QueueMusic_Delayed8(word!(game.music_track_index), game);
    }
}
pub unsafe fn LoadColorsForSpritesBeamsAndEnemies(game: &mut Game) {
    word_transfers!(game, descending, 16; le(target_palettes, 223) <- rom(0x9a8000, 223));
    word_transfers!(game, descending, 16; native(0xc3de) <- native(0xc1de));
    word_transfers!(game, descending, 16; native(0xc31e) <- rom(0x9afc00, 15));
}
crate::game::callback_dispatch! {
    pub unsafe fn CallDoorTransitionFunction_Async(ea: u32, game: &mut Game) -> CoroutineRet => ea;
    call (game); default Unreachable(game) as CoroutineRet;
    @functions [pub unsafe fn (game: &mut Game) -> CoroutineRet];
    [8577405] DoorTransitionFunction_HandleElevator [] {
        if word!(game.elevator_flags) == 0 {
            word!(game.game_state += 1);
            return GameState_10_LoadingNextRoom_Async(game);
        }
        CallSomeSamusCode(0, game);
        if (word!(game.elevator_direction) & 0x8000) != 0 {
            word!(game.game_state += 1);
            return GameState_10_LoadingNextRoom_Async(game);
        }
        word!(game.downwards_elevator_delay_timer = 48);
        word!(game.door_transition_function = ((0x82e19fi32) as u16));
        DoorTransitionFunction_Wait48frames(game)
    };
    [8577439] DoorTransitionFunction_Wait48frames [] {
        if (word!(game.downwards_elevator_delay_timer -= 1, reread) & 0x8000) != 0 {
            word!(game.game_state += 1);
            return GameState_10_LoadingNextRoom_Async(game);
        } else {
            DetermineWhichEnemiesToProcess(game);
            EnemyMain(game);
            DrawSamusEnemiesAndProjectiles(game);
            EnsureSamusDrawnEachFrame(game);
        }
        kCoroutineNone as CoroutineRet
    };
    [8577694] DoorTransitionFunction_WaitForSoundsToFinish [] {
        DetermineWhichEnemiesToProcess(game);
        EnemyMain(game);
        DrawSamusEnemiesAndProjectiles(game);
        EnsureSamusDrawnEachFrame(game);
        if game.sfx_queue_is_empty(0) && game.sfx_queue_is_empty(1) && game.sfx_queue_is_empty(2) {
            game.set_door_transition_function(0xe2db);
        }
        kCoroutineNone as CoroutineRet
    };
    [8577755] DoorTransitionFunction_FadeOutScreen [] {
        if AdvancePaletteFadeForAllPalettes_0xc(game) != 0 {
            game.set_door_transition_function(0xe2f7);
        } else {
            DetermineWhichEnemiesToProcess(game);
            EnemyMain(game);
            DrawSamusEnemiesAndProjectiles(game);
            EnsureSamusDrawnEachFrame(game);
        }
        kCoroutineNone as CoroutineRet
    };
    [8577783] DoorTransitionFunction_LoadDoorHeaderEtc [] {
        LoadDoorHeader(game);
        sub_8882AC(game);
        game.set_hdma_objects_enable_flag(game.hdma_objects_enable_flag() & 0x7fff);
        game.set_irqhandler_next_handler(8);
        game.set_door_transition_function(0xe310);
        kCoroutineNone as CoroutineRet
    };
    [8577808] DoorTransitionFunction_ScrollScreenToAlignment [] {
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
    };
    [8577875] DoorTransitionFunction_FixDoorsMovingUp [] {
        if game.door_direction() & 3 == 3 {
            FixDoorsMovingUp(game);
        }
        game.set_door_transition_function(0xe36e);
        kCoroutineNone as CoroutineRet
    };
    [8577902] DoorTransitionFunction_SetupNewRoom [] {
        SaveMapExploredifElevator(game);
        LoadRoomHeader(game);
        LoadStateHeader(game);
        LoadMapExploredIfElevator(game);
        InitializeSpecialEffectsForNewRoom(game);
        LoadLevelScrollAndCre(game);
        game.set_door_transition_function(0xe38e);
        kCoroutineNone as CoroutineRet
    };
    [8577934] DoorTransitionFunction_SetupScrolling [] {
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
    };
    [8577984] DoorTransitionFunction_PlaceSamusLoadTiles [] {
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
            DecompressToMem(0xb98000, ram_ptr!(game; 0x7000), game);
        }
        DecompressToMem(game.tileset_tiles_pointer(), ram_ptr!(game; 0x2000), game);
        DecompressToMem(
            game.tileset_compr_palette_ptr(),
            ram_ptr!(game; 0xc200),
            game,
        );
        CopyToVramNow(0, 0x7e2000, 0x2000, game);
        CopyToVramNow(0x1000, 0x7e4000, 0x2000, game);
        CopyToVramNow(0x2000, 0x7e6000, 0x1000, game);
        if game.cre_bitset() & 6 != 0 && game.door_def_ptr() != addr_kDoorDef_947a as u16 {
            CopyToVramNow(0x2800, 0x7e7000, 0x1000, game);
            CopyToVramNow(0x3000, 0x7e8000, 0x2000, game);
            CopyToVramNow(0x4000, 0x9ab200, 0x1000, game);
        }
        if game.door_direction() & 3 == 3 {
            game.set_irqhandler_next_handler(16);
        }
        game.set_door_transition_function(0xe4a9);
        kCoroutineNone as CoroutineRet
    };
    [8578217] DoorTransitionFunction_LoadMoreThings_Async [] {
        crate::verify_probe::hit("coroutine.door");
        let resume = game.coroutine_state_1();
        if resume == 0 {
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
            let mut bg_data_ptr: u16 = game.read_rom_word(
                0x8f0000 | u32::from(game.roomdefroomstate_ptr()),
                core::mem::offset_of!(RoomDefRoomstate, bg_data_ptr),
            );
            if bg_data_ptr & 0x8000 != 0 {
                do_until! { bg_data_ptr == 0;
    let command = game.read_rom_word(0x8f0000 | u32::from(bg_data_ptr), 0);
    bg_data_ptr = kUpdateBackgroundCommands[usize::from(command >> 1)](
        bg_data_ptr.wrapping_add(2),
        game,
    );
}
            }
        }
        if resume <= 1 {
            let mut waiting = resume == 1;
            loop {
                if waiting {
                    let yielded: u8 = WaitForNMI_Async(game);
                    if yielded != 0 {
                        game.set_coroutine_state_1(1);
                        return yielded;
                    }
                }
                let door_transition_flag =
                    u16::from_le_bytes([game.g_ram[0x931], game.g_ram[0x932]]);
                if door_transition_flag & 0x8000 != 0 {
                    break;
                }
                waiting = true;
            }
            game.g_ram[0xc188..0xc18a].copy_from_slice(&15328u16.to_le_bytes());
            SpawnBG3ScrollHdmaObject(game);
            game.set_hdma_objects_enable_flag(game.hdma_objects_enable_flag() | 0x8000);
        }
        if resume <= 2 {
            let yielded: u8 = PlmHandler_Async(game);
            if yielded != 0 {
                game.set_coroutine_state_1(2);
                return yielded;
            }
            if game.door_direction() & 2 == 0 {
                if game.door_direction() & 3 != 0 {
                    game.set_samus_x_pos(game.samus_x_pos() & 0xfff8);
                } else {
                    game.set_samus_x_pos(game.samus_x_pos() | 7);
                }
            }
            game.set_door_transition_function(0xe659);
        }
        game.set_coroutine_state_1(0);
        0 as CoroutineRet
    };
    [8578649] DoorTransition_C_HandleAnimTiles [] {
        AnimtilesHandler(game);
        game.set_door_transition_function(fnDoorTransition_WaitForMusicToClear as u16);
        kCoroutineNone as CoroutineRet
    };
    [8578660] DoorTransition_WaitForMusicToClear [] {
        if HasQueuedMusic(game) == 0 {
            game.set_door_transition_function(fnDoorTransition_HandleTransition as u16);
            LoadNewMusicTrackIfChanged(game);
        }
        kCoroutineNone as CoroutineRet
    };
    [8578722] DoorTransition_HandleTransition [] {
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
        word_transfers!(game, descending, 256;
            little(0x9dfe) <- value(0), little(0x9ffe) <- value(0),
        );
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
    };
    [8578871] DoorTransition_FadeInScreenAndFinish [] {
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
    };
}
pub unsafe fn GameState_9_HitDoorBlock(game: &mut Game) -> CoroutineRet {
    CallDoorTransitionFunction_Async(u32::from(game.door_transition_function()) | 0x820000, game)
}
pub unsafe fn GameState_10_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    word! { game, 1;
        door_transition_flag_enemies, door_transition_flag_elevator_zebetites,
    }
    word! { game, 0; debug_disable_minimap, save_station_lockout_flag }
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    LoaadDesinationRoomCreBitset(game);
    word_transfers!(game, descending, 128;
        le(target_palettes, 127) <- value(0),
        le(target_palettes, 255) <- value(0),
    );
    word!(game.target_palettes[9] = word!(game.palette_buffer[9]));
    word!(game.target_palettes[10] = word!(game.palette_buffer[10]));
    word!(game.target_palettes[13] = word!(game.palette_buffer[13]));
    word!(game.target_palettes[14] = word!(game.palette_buffer[14]));
    word!(game.target_palettes[17] = word!(game.palette_buffer[17]));
    word!(game.target_palettes[18] = word!(game.palette_buffer[18]));
    word!(game.target_palettes[19] = word!(game.palette_buffer[19]));
    word!(game.target_palettes[29] = word!(game.palette_buffer[29]));
    if ((word!(game.previous_cre_bitset) | word!(game.cre_bitset)) & 1) == 0 {
        word!(game.target_palettes[20] = word!(game.palette_buffer[20]));
        word!(game.target_palettes[21] = word!(game.palette_buffer[21]));
        word!(game.target_palettes[22] = word!(game.palette_buffer[22]));
        word!(game.target_palettes[23] = word!(game.palette_buffer[23]));
        word!(game.target_palettes[28] = word!(game.palette_buffer[28]));
        if word!(game.timer_status) != 0 {
            word!(game.target_palettes[209] = word!(game.palette_buffer[209]));
            word!(game.target_palettes[210] = word!(game.palette_buffer[210]));
            word!(game.target_palettes[212] = word!(game.palette_buffer[212]));
            word!(game.target_palettes[221] = word!(game.palette_buffer[221]));
            DrawTimer(game);
        }
    }
    ClearSoundsWhenGoingThroughDoor(game);
    QueueSfx2_Max15(0x71, game);
    word!(game.debug_disable_sounds = -1i32 as u16);
    word!(game.door_transition_function = ((0x82e29ei32) as u16));
    word!(game.game_state += 1);
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
pub unsafe fn ClearFxTilemap(game: &mut Game) {
    word_transfers!(game, descending, 960;
        little(0x477e) <- value(6222), little(0x4efe) <- value(6222),
    );
    CopyToVramNow(0x5880, 0x7e4000, 0xf00, game);
}
pub unsafe fn ClearBg2Tilemap(game: &mut Game) {
    word_transfers!(game, descending, 1024;
        little(0x47fe) <- value(824), little(0x4ffe) <- value(824),
    );
    CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
}
pub unsafe fn ClearKraidBg2Tilemap(game: &mut Game) {
    word_transfers!(game, descending, 1024; xray(1023) <- value(824), xray(2047) <- value(824));
    CopyToVramNow(0x4000, 0x7e4000, 0x1000, game);
    CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
}
pub fn UpdateBackgroundCommand_0_Finish(y: u16, game: &mut Game) -> u16 {
    0
}
pub unsafe fn UpdateBackgroundCommand_E_DoorDependentTransferToVRAM(
    j: u16,
    game: &mut Game,
) -> u16 {
    if word!(game.door_def_ptr) == *(RomPtr_8F(j, game) as *mut u16) {
        UpdateBackgroundCommand_2_TransferToVram((j).wrapping_add(2), game)
    } else {
        (j).wrapping_add(9)
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
    *ram_ptr!(game; 0x5e) = 2;
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
pub unsafe fn LoadDestinationRoomThings(game: &mut Game) {
    LoaadDesinationRoomCreBitset(game);
    LoadDoorHeader(game);
    LoadRoomHeader(game);
    LoadStateHeader(game);
    LoadCRETilesTilesetTilesAndPalette(game);
}
pub unsafe fn LoadCRETilesTilesetTilesAndPalette(game: &mut Game) {
    word!(game.elevator_flags = 0);
    crate::rtl::register_writes! { game;
        word(VMAIN as u16, 0x80); word(VMADDL as u16, (addr_unk_605000 as i32 >> 1) as u16);
    }
    DecompressToVRAM(0xb98000, addr_unk_605000 as u16, game);
    WriteRegWord(VMADDL as u16, 0, game);
    DecompressToVRAM(Load24(ram_ptr!(game, LongPtr; 0x7c3)), 0, game);
    DecompressToMem(
        Load24(ram_ptr!(game, LongPtr; 0x7c6)),
        ram_ptr!(game; 0xc200),
        game,
    );
}

macro_rules! decompress_room_level_and_split_arrays {
    ($game:ident) => {
        DecompressToMem(
            Load24(ram_ptr!($game, LongPtr; 0x7bd)),
            ram_ptr!($game, u16; 0x10000) as *mut u8,
            $game,
        );
        let size: u16 = word!($game.ram7F_start);
        memcpy(
            ram_ptr!($game, u16; 0x19602) as *mut c_void,
            (ram_ptr!($game, u16; 0x10002) as *mut u8)
                .offset(size as i32 as isize)
                .offset((size >> 1) as isize) as *const c_void,
            size as size_t,
        );
        memcpy(
            ram_ptr!($game, c_void; 0x16402),
            (ram_ptr!($game, u16; 0x10002) as *mut u8).offset(size as i32 as isize)
                as *const c_void,
            (size as i32 >> 1) as size_t,
        );
    };
}

pub unsafe fn LoadLevelDataAndOtherThings(game: &mut Game) {
    word_transfers!(game, descending, 12800; le(level_data, 12799) <- value(0x8000));
    decompress_room_level_and_split_arrays!(game);
    if word!(game.area_index) == 6 {
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa000),
            game,
        );
    } else {
        DecompressToMem(0xb9a09d, ram_ptr!(game; 0xa000), game);
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa800),
            game,
        );
    }
    let RD = get_RoomDefRoomstate(word!(game.roomdefroomstate_ptr), game);
    let mut rdf_scroll_ptr: i16 = (*RD).rdf_scroll_ptr as i16;
    if rdf_scroll_ptr >= 0 {
        let scrollval: u16 = (*RD).rdf_scroll_ptr;
        let r20: u8 = ((word!(game.room_height_in_scrolls)) as u8).wrapping_sub(1);
        let mut v8: u8 = 2;
        let mut v9: u8 = 0;
        let mut v10: i8 = 0;
        do_until! { (v10 as i32 == word!(game.room_height_in_scrolls) as u8 as i32);
            if v10 as i32 == r20 as i32 {
                v8 = ((scrollval) as u8).wrapping_add(1);
            }
            let mut v11: i8 = 0;
            do_until! { (v11 as i32 == word!(game.room_width_in_scrolls) as u8 as i32);
            crate::update_place! { previous fresh145 = v9 => v9.wrapping_add(1); }
            *ram_ptr!(game; 0xcd20, fresh145 as isize) = v8;
            v11 += 1;
        }
            v10 += 1;
        }
    } else {
        for m in (0u16..50).step_by(2) {
            *(ram_ptr!(game, u16; 0xcd20, m as isize)) =
                *(RomPtr_8F(rdf_scroll_ptr as u16, game) as *mut u16);
            rdf_scroll_ptr = (rdf_scroll_ptr).wrapping_add(2);
        }
    }
    if (*RD).room_plm_header_ptr != 0 {
        let mut n: u16 = (*RD).room_plm_header_ptr;
        while (*get_RoomPlmEntry(n, game)).plm_header_ptr_ != 0 {
            SpawnRoomPLM(n, game);
            n = (n).wrapping_add(6);
        }
    }
    RunDoorSetupCode(game);
    RunRoomSetupCode(game);
    if word!(game.elevator_flags) != 0 {
        word!(game.elevator_status = 2);
    }
}
pub unsafe fn SpawnDoorClosingPLM(game: &mut Game) {
    if CheckIfColoredDoorCapSpawned(game) == 0 {
        let v0 = RomPtr_8F(
            (2 * word!(game.door_direction) as i32 + addr_kDoorClosingPlmIds as i32) as u16,
            game,
        ) as *const u16;
        if *v0 != 0 {
            let rp: *mut RoomPlmEntry = ram_ptr!(game, RoomPlmEntry; 0x12);
            (*rp).plm_header_ptr_ = *v0;
            *(&raw mut (*rp).x_block as *mut u16) = *(&raw mut (*(get_DoorDef
                as unsafe fn(u16, &mut Game) -> *mut DoorDef)(
                word!(game.door_def_ptr), game
            ))
            .x_pos_plm as *mut u16);
            (*rp).plm_room_argument = 0;
            SpawnRoomPLM(0x12, game);
        }
    }
}
pub unsafe fn CheckIfColoredDoorCapSpawned(game: &mut Game) -> u8 {
    let DD = get_DoorDef(word!(game.door_def_ptr), game);
    let v2: u16 = (2
        * (Mult8x8((*DD).y_pos_plm, word!(game.room_width_in_blocks) as u8) as i32
            + (*DD).x_pos_plm as i32)) as u16;
    let mut v3: u16 = 78;
    while v2 != word!(game.plm_block_indices[(v3 >> 1)]) {
        v3 = (v3).wrapping_sub(2);
        if (v3 & 0x8000) != 0 {
            return 0;
        }
    }
    let v5: i32 = v3 as i32 >> 1;
    if word!(game.plm_header_ptr[v5]) == 0 {
        return 0;
    }
    let v6: i16 = word!(game.plm_room_arguments[v5]) as i16;
    if v6 >= 0 {
        let idx: i32 = PrepareBitAccess(v6 as u16, game) as i32;
        if (word!(game.bitmask) & *ram_ptr!(game; 0xd8b0, idx as isize) as u16) != 0 {
            return 0;
        }
    }
    word!(game.plm_instruction_timer[v5] = 1);
    let v8: u16 = word!(game.plm_header_ptr[v5]);
    if v8 == 0 {
        return 0;
    }
    word!(game.plm_instr_list_ptrs[v5] = (*get_PlmHeader_Size6(v8, game)).instr_list_2_ptr);
    1
}
static kLoadLibraryBackgroundFuncs: [Func_Y_Y; 8] = callback_table! { nonnull Func_Y_Y;
    LoadLibraryBackgroundFunc_0_DONE, LoadLibraryBackgroundFunc_2_TransferToVram,
    LoadLibraryBackgroundFunc_4_Decompress, LoadLibraryBackgroundFunc_6_ClearFxTilemap,
    LoadLibraryBackgroundFunc_8_TransferToVramAndSetBg3,
    LoadLibraryBackgroundFunc_A_ClearBG2Tilemap, LoadLibraryBackgroundFunc_C_ClearKraidLayer2,
    LoadLibraryBackgroundFunc_E_DoorDependentTransferToVram,
};
pub unsafe fn LoadLibraryBackground(game: &mut Game) {
    ClearFXTilemap(game);
    if word!(game.fx_tilemap_ptr) != 0 {
        crate::rtl::register_writes! { game;
            word(A1T1L as u16, word!(game.fx_tilemap_ptr)); word(VMADDL as u16, addr_unk_605BE0 as u16);
            word(DMAP1 as u16, 0x1801); word(A1B1 as u16, 0x8a);
            word(DAS1L as u16, 0x840); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
    let mut bg_data_ptr: u16 =
        (*get_RoomDefRoomstate(word!(game.roomdefroomstate_ptr), game)).bg_data_ptr;
    if bg_data_ptr & 0x8000 != 0 {
        do_until! { (bg_data_ptr == 0);
            let v1: u16 = *(RomPtr_8F(bg_data_ptr, game) as *mut u16);
            bg_data_ptr = kLoadLibraryBackgroundFuncs[(v1 >> 1) as usize]
                (
                (bg_data_ptr).wrapping_add(2), game
            );
        }
    }
}
pub fn LoadLibraryBackgroundFunc_0_DONE(j: u16, game: &mut Game) -> u16 {
    0
}
pub unsafe fn LoadLibraryBackgroundFunc_E_DoorDependentTransferToVram(
    j: u16,
    game: &mut Game,
) -> u16 {
    if word!(game.door_def_ptr) == (*get_LoadBg_E(j, game)).field_0 {
        LoadLibraryBackgroundFunc_2_TransferToVram((j).wrapping_add(2), game)
    } else {
        (j).wrapping_add(9)
    }
}
pub unsafe fn LoadLibraryBackgroundFunc_2_TransferToVram(j: u16, game: &mut Game) -> u16 {
    let LoadBg_28 = RomPtr_8F(j, game) as *mut u16;
    crate::rtl::register_writes! { game;
        word(VMADDL as u16,
        *((LoadBg_28 as *mut u8).offset(3) as *mut u16),); word(DMAP1 as u16, 0x1801);
        word(A1T1L as u16, *LoadBg_28); word(A1B1 as u16, *LoadBg_28.offset(1));
        word(DAS1L as u16,
        *((LoadBg_28 as *mut u8).offset(5) as *mut u16),); byte(VMAIN as u16, 0x80);
        byte(MDMAEN as u16, 2);
    }
    (j).wrapping_add(7)
}
pub unsafe fn LoadLibraryBackgroundFunc_4_Decompress(j: u16, game: &mut Game) -> u16 {
    let p = RomPtr_8F(j, game);
    DecompressToMem(
        Load24(p as *mut LongPtr),
        ram_ptr!(game; *(p.offset(3) as *mut u16) as i32 as isize),
        game,
    );
    (j).wrapping_add(5)
}
pub unsafe fn LoadLibraryBackgroundFunc_6_ClearFxTilemap(j: u16, game: &mut Game) -> u16 {
    ClearFXTilemap(game);
    j
}
pub unsafe fn LoadLibraryBackgroundFunc_A_ClearBG2Tilemap(j: u16, game: &mut Game) -> u16 {
    ClearBG2Tilemap(game);
    j
}
pub use self::LoadLibraryBackgroundFunc_A_ClearBG2Tilemap as LoadLibraryBackgroundFunc_C_ClearKraidLayer2;
pub unsafe fn LoadLibraryBackgroundFunc_8_TransferToVramAndSetBg3(
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = LoadLibraryBackgroundFunc_2_TransferToVram(j, game);
    *ram_ptr!(game; 0x5e) = 2;
    j
}
pub unsafe fn LoadLevelScrollAndCre(game: &mut Game) {
    word_transfers!(game, descending, 3200;
        le(level_data, 3199) <- value(0x8000),
        native(0x13200) <- value(0x8000),
        native(0x14b00) <- value(0x8000),
        native(0x16400) <- value(0x8000),
    );
    decompress_room_level_and_split_arrays!(game);
    if word!(game.area_index) == 6 {
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa000),
            game,
        );
    } else {
        if (word!(game.cre_bitset) & 2) != 0 {
            DecompressToMem(0xb9a09d, ram_ptr!(game; 0xa000), game);
        }
        DecompressToMem(
            Load24(ram_ptr!(game, LongPtr; 0x7c0)),
            ram_ptr!(game; 0xa800),
            game,
        );
    }
    let RD = get_RoomDefRoomstate(word!(game.roomdefroomstate_ptr), game);
    let mut rdf_scroll_ptr: i16 = (*RD).rdf_scroll_ptr as i16;
    if rdf_scroll_ptr >= 0 {
        let r18: u16 = (*RD).rdf_scroll_ptr;
        let r20: u8 = ((word!(game.room_height_in_scrolls)) as u8).wrapping_sub(1);
        let mut v8: u8 = 2;
        let mut v9: u8 = 0;
        let mut v10: i8 = 0;
        do_until! { (v12 as i32 + 1 == word!(game.room_height_in_scrolls) as u8 as i32);
            if v10 as i32 == r20 as i32 {
                v8 = ((r18) as u8).wrapping_add(1);
            }
            let mut v12: i8 = v10;
            let mut v11: i8 = 0;
            do_until! { (v11 as i32 == word!(game.room_width_in_scrolls) as u8 as i32);
            crate::update_place! { previous fresh35 = v9 => v9.wrapping_add(1); }
            *ram_ptr!(game; 0xcd20, fresh35 as isize) = v8;
            v11 += 1;
        }
            v10 = (v12).wrapping_add(1);
        }
    } else {
        for m in (0u16..50).step_by(2) {
            *(ram_ptr!(game, u16; 0xcd20, m as isize)) =
                *(RomPtr_8F(rdf_scroll_ptr as u16, game) as *mut u16);
            rdf_scroll_ptr = (rdf_scroll_ptr).wrapping_add(2);
        }
    };
}
pub unsafe fn CreatePlmsExecuteDoorAsmRoomSetup(game: &mut Game) {
    let mut RoomDefRoomstate = get_RoomDefRoomstate(word!(game.roomdefroomstate_ptr), game);
    if (*RoomDefRoomstate).room_plm_header_ptr != 0 {
        let mut i: i32 = (*RoomDefRoomstate).room_plm_header_ptr as i32;
        while (*get_RoomPlmEntry(i as u16, game)).plm_header_ptr_ != 0 {
            SpawnRoomPLM(i as u16, game);
            i += 6;
        }
    }
    RunDoorSetupCode(game);
    RunRoomSetupCode(game);
    if word!(game.elevator_flags) != 0 {
        word!(game.elevator_status = 2);
    }
}
static kGameOptionsMenuFuncs: [Func_V; 13] = callback_table! { nonnull Func_V;
    GameOptionsMenuFunc_0, GameOptionsMenu_1_LoadingOptionsScreen,
    GameOptionsMenu_2_FadeInOptionsScreen, GameOptionsMenu_3_OptionsScreen,
    GameOptionsMenu_4_StartGame, GameOptionsMenu_5_DissolveOutScreen,
    GameOptionsMenu_6_DissolveInScreen, GameOptionsMenu_7_ControllerSettings,
    GameOptionsMenu_8_SpecialSettings, GameOptionsMenu_9_ScrollControllerSettingsDown,
    GameOptionsMenu_A_ScrollControllerSettingsUp, GameOptionsMenu_B_TransitionBackToFileSelect,
    GameOptionsMenu_C_FadeOutOptionsScreenToStart,
};
pub unsafe fn GameState_2_GameOptionsMenu(game: &mut Game) -> CoroutineRet {
    kGameOptionsMenuFuncs[word!(game.game_options_screen_index) as usize](game);
    OptionsMenuFunc1(game);
    DrawOptionsMenuSpritemaps(game);
    if word!(game.game_options_screen_index) as i32 - 2i32 & 0x8000 == 0 {
        OptionsMenu_AddToVramQueue(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameOptionsMenuFunc_0(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_options_screen_index += 1);
    } else if *ram_ptr!(game; 0x51) == 14 {
        let v0: u16 = *ram_ptr!(game; 0x6b) as u16;
        if (*ram_ptr!(game; 0x6b) & 4) == 0 {
            CreateOptionsMenuObject_(v0, addr_stru_82F4D6 as u16, game);
        }
    }
}
pub unsafe fn GameOptionsMenu_1_LoadingOptionsScreen(game: &mut Game) {
    *ram_ptr!(game; 0x5d) = 0;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 0;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 0;
    *ram_ptr!(game; 0x6e) = 0;
    *ram_ptr!(game; 0x71) = 0;
    *ram_ptr!(game; 0x6f) = 0;
    crate::rtl::register_writes! { game;
        mirror(VMADDL as u16, 0, 0x72, true); byte(VMADDH as u16, 0x58);
        byte(VMAIN as u16, 0x80);
    }
    static unk_82EC3D: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8edc00, 0x800);
    SetupDmaTransfer(&raw const unk_82EC3D as *const c_void, game);
    WriteReg(MDMAEN as u16, 2, game);
    word! { game, 0;
        reg_BG1HOFS, reg_BG1VOFS, reg_BG2HOFS, reg_BG2VOFS, debug_invincibility,
    }
    word_transfers!(game, descending, 256; le(palette_buffer, 255) <- rom(0x8ee400, 255));
    DecompressToMem(0x978df4, ram_ptr!(game; 0x1c000), game);
    DecompressToMem(0x978fcd, ram_ptr!(game; 0x1c800), game);
    DecompressToMem(0x9791c4, ram_ptr!(game; 0x1d000), game);
    DecompressToMem(0x97938d, ram_ptr!(game; 0x1d800), game);
    DecompressToMem(0x97953a, ram_ptr!(game; 0x1e000), game);
    word_transfers!(game, descending, 1024; pause_map(1023) <- le(custom_background, 6398));
    // The ROM keeps the final copied tile in A through the object clear.
    let initial_tile = u16::from_le_bytes([game.g_ram[0x3000], game.g_ram[0x3001]]);
    word!(game.menu_option_index = 0);
    DeleteAllOptionsMenuObjects_(game);
    CreateOptionsMenuObject_(initial_tile, addr_stru_82F4B8 as u16, game);
    // The first initializer leaves its palette $0E00 in A for this call.
    CreateOptionsMenuObject_(0x0e00, addr_stru_82F4C4 as u16, game);
    word!(game.game_options_screen_index += 1);
    OptionsMenuFunc4(game);
}
pub unsafe fn GameOptionsMenu_2_FadeInOptionsScreen(game: &mut Game) {
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_options_screen_index += 1);
    }
}
pub unsafe fn OptionsMenu_AddToVramQueue(game: &mut Game) {
    let mut v0: u16 = word!(game.vram_write_queue_tail);
    (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 2048;
    v0 = (v0).wrapping_add(2);
    (*gVramWriteEntry(v0 as i32, game)).size = 12288;
    v0 = (v0).wrapping_add(2);
    crate::update_place! { previous fresh118 = v0 => v0.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh118 as i32,
        game,
    ))
    .size as *mut u8) = 126;
    (*gVramWriteEntry(v0 as i32, game)).size = 20480;
    word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
}
pub unsafe fn OptionsMenuFunc5(a: u16, mut k: u16, mut j: u16, game: &mut Game) {
    do_until! { (j == 0);
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
            as *mut u8)
            .offset(k as i32 as isize) as *mut u16) = (a
            | *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
                as *mut u8)
                .offset(k as i32 as isize) as *mut u16)
                & 0xe3ff);
        k = (k).wrapping_add(2);
        j = (j).wrapping_sub(2);
    }
}
static kGameOptionsMenuItemFuncs: [Func_V; 5] = callback_table! { nonnull Func_V;
    GameOptionsMenuItemFunc_0, GameOptionsMenuItemFunc_2_ToggleJapanese,
    GameOptionsMenuItemFunc_2_ToggleJapanese, GameOptionsMenuItemFunc_4,
    GameOptionsMenuItemFunc_4,
};
pub unsafe fn GameOptionsMenu_3_OptionsScreen(game: &mut Game) {
    if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if (word!(game.menu_option_index -= 1, reread) & 0x8000) != 0 {
            word!(game.menu_option_index = 4);
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.menu_option_index += 1, reread) == 5 {
            word!(game.menu_option_index = 0);
        }
    }
    if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
        word!(game.game_options_screen_index = 11);
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_A as i32) != 0
        || (word!(game.joypad1_newkeys) as i32 & kButton_Start as i32) != 0
    {
        QueueSfx1_Max6(0x38, game);
        kGameOptionsMenuItemFuncs[word!(game.menu_option_index) as usize](game);
    }
}
pub unsafe fn GameOptionsMenuItemFunc_0(game: &mut Game) {
    if word!(game.enable_debug) != 0
        && (word!(game.joypad1_lastkeys) as i32 & kButton_L as i32) == 0
        || word!(game.loading_game_state) as i32 == kGameState_5_FileSelectMap as i32
    {
        word!(game.game_options_screen_index = 4);
    } else {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_options_screen_index = 12);
    };
}
pub unsafe fn GameOptionsMenuItemFunc_2_ToggleJapanese(game: &mut Game) {
    word!(game.menu_option_index = 0);
    word!(game.japanese_text_flag = (word!(game.japanese_text_flag) == 0) as u16);
    OptionsMenuFunc4(game);
}
pub unsafe fn OptionsMenuFunc4(game: &mut Game) {
    if word!(game.japanese_text_flag) != 0 {
        OptionsMenuFunc5(0x400, 0x288, 0x18, game);
        OptionsMenuFunc5(0x400, 0x2c8, 0x18, game);
        OptionsMenuFunc5(0, 0x348, 0x32, game);
        OptionsMenuFunc5(0, 0x388, 0x32, game);
    } else {
        OptionsMenuFunc5(0, 0x288, 0x18, game);
        OptionsMenuFunc5(0, 0x2c8, 0x18, game);
        OptionsMenuFunc5(0x400, 0x348, 0x32, game);
        OptionsMenuFunc5(0x400, 0x388, 0x32, game);
    };
}
pub unsafe fn GameOptionsMenuItemFunc_4(game: &mut Game) {
    *ram_ptr!(game; 0x57) = 3;
    word! { game, 0; screen_fade_delay, screen_fade_counter }
    word!(game.game_options_screen_index = 5);
}
pub unsafe fn GameOptionsMenu_B_TransitionBackToFileSelect(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state = kGameState_4_FileSelectMenus as u16);
        word! { game, 0; menu_index, game_options_screen_index }
    }
}
pub unsafe fn GameOptionsMenu_C_FadeOutOptionsScreenToStart(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_options_screen_index = 4);
    }
}
pub unsafe fn GameOptionsMenu_4_StartGame(game: &mut Game) {
    word!(game.game_options_screen_index = 0);
    if word!(game.enable_debug) != 0 && (word!(game.joypad1_lastkeys) & 0x20) == 0 {
        word!(game.game_state = kGameState_5_FileSelectMap as u16);
        if word!(game.loading_game_state) as i32 != kLoadingGameState_5_Main as i32 {
            word!(game.loading_game_state = kGameState_5_FileSelectMap as u16);
            SaveToSram(word!(game.selected_save_slot), game);
        }
    } else if word!(game.loading_game_state) != 0 {
        word!(game.game_state = word!(game.loading_game_state));
        if word!(game.loading_game_state) as i32 == kGameState_34_CeresGoesBoom as i32 {
            word!(game.cinematic_function = ((0x8bc11bi32) as u16));
        }
        word! { game, 0; menu_option_index, game_options_screen_index }
    } else {
        word!(game.game_state = kGameState_30_IntroCinematic as u16);
        word!(game.cinematic_function = ((0x8ba395i32) as u16));
        word! { game, 0; menu_option_index, screen_fade_delay, screen_fade_counter }
    };
}
pub unsafe fn GameOptionsMenu_5_DissolveOutScreen(game: &mut Game) {
    let mut v1: u16 = 0;
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x57) != 0xf3 {
        crate::update_place! { reference fresh70 = *ram_ptr!(game; 0x57) => (*fresh70).wrapping_add(16); }
    }
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter, reg_BG1VOFS }
        word!(game.game_options_screen_index += 1);
        if word!(game.menu_option_index) != 0 {
            if (word!(game.menu_option_index) & 4) != 0 {
                if word!(game.japanese_text_flag) != 0 {
                    word_transfers!(game, descending, 1024;
                        pause_map(1023) <- le(custom_background, 10494),
                    );
                } else {
                    word_transfers!(game, descending, 1024; pause_map(1023) <- native(0x1dffe));
                }
                word!(game.menu_option_index = 0);
                OptionsMenuFunc7(game);
                word!(game.menu_option_index = 1);
                OptionsMenuFunc7(game);
                word!(game.menu_option_index = 4);
                CreateOptionsMenuObject_(4, addr_stru_82F4D0 as u16, game);
            } else {
                if word!(game.japanese_text_flag) != 0 {
                    for k in (0u16..=1023).rev() {
                        v1 = *(ram_ptr!(game, u16; 0x19602)).offset((k as i32 + 7423) as isize);
                        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[k as usize] =
                            v1;
                    }
                } else {
                    for m in (0u16..=1023).rev() {
                        v1 = *(ram_ptr!(game, u16; 0x19602)).offset((m as i32 + 6399) as isize);
                        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[m as usize] =
                            v1;
                    }
                }
                CreateOptionsMenuObject_(v1, addr_stru_82F4CA as u16, game);
                LoadControllerOptionsFromControllerBindings(game);
                OptionsMenuFunc6(game);
            }
        } else {
            word_transfers!(game, descending, 1024; pause_map(1023) <- le(custom_background, 6398));
            OptionsMenuFunc4(game);
            let v6: u16 = 0;
            CreateOptionsMenuObject_(v6, addr_stru_82F4C4 as u16, game);
        }
    }
}
pub unsafe fn GameOptionsMenu_6_DissolveInScreen(game: &mut Game) {
    HandleFadeIn(game);
    if *ram_ptr!(game; 0x57) != 3 {
        crate::update_place! { reference fresh72 = *ram_ptr!(game; 0x57) => (*fresh72).wrapping_sub(16); }
    }
    if *ram_ptr!(game; 0x51) == 15 {
        *ram_ptr!(game; 0x57) = 0;
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        if word!(game.menu_option_index) != 0 {
            if (word!(game.menu_option_index) & 4) != 0 {
                word!(game.game_options_screen_index = 8);
            } else {
                word!(game.game_options_screen_index = 7);
            }
            word!(game.menu_option_index = 0);
        } else {
            word!(game.game_options_screen_index = 3);
        }
    }
}
static kGameOptionsMenuSpecialSettings: [Func_V; 3] = callback_table! { nonnull Func_V;
    GameOptionsMenuSpecialSettings_0, GameOptionsMenuSpecialSettings_0,
    GameOptionsMenuSpecialSettings_2,
};
pub unsafe fn GameOptionsMenu_8_SpecialSettings(game: &mut Game) {
    if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if (word!(game.menu_option_index -= 1, reread) & 0x8000) != 0 {
            word!(game.menu_option_index = 2);
        }
    } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.menu_option_index += 1, reread) == 3 {
            word!(game.menu_option_index = 0);
        }
    }
    if (word!(game.joypad1_newkeys) & 0x8000) == 0 {
        if word!(game.joypad1_newkeys) as i32
            & (kButton_Start as i32 | kButton_Left as i32 | kButton_Right as i32 | kButton_A as i32)
            != 0
        {
            QueueSfx1_Max6(0x38, game);
            kGameOptionsMenuSpecialSettings[word!(game.menu_option_index) as usize](game);
        }
    } else {
        QueueSfx1_Max6(0x38, game);
        word!(game.menu_option_index = 0);
        GameOptionsMenuItemFunc_4(game);
    };
}
pub unsafe fn GameOptionsMenuSpecialSettings_0(game: &mut Game) {
    let v0 = RomPtr_RAM(
        crate::rtl::rom_table_word!(game, 0x82f0ae, word!(game.menu_option_index) as isize),
        game,
    );
    if *(v0 as *mut u16) != 0 {
        *(v0 as *mut u16) = 0;
    } else {
        *(v0 as *mut u16) = 1;
    }
    OptionsMenuFunc7(game);
}
pub unsafe fn GameOptionsMenuSpecialSettings_2(game: &mut Game) {
    word!(game.menu_option_index = 0);
    GameOptionsMenuItemFunc_4(game);
}
static g_word_82F149: [u16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
static g_word_82F151: [u16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
pub unsafe fn OptionsMenuFunc7(game: &mut Game) {
    let v0: u16 = (4u16).wrapping_mul(word!(game.menu_option_index));
    if *(RomPtr_RAM(
        crate::rtl::rom_table_word!(game, 0x82f0ae, word!(game.menu_option_index) as isize),
        game,
    ) as *mut u16)
        != 0
    {
        OptionsMenuFunc5(
            0,
            g_word_82F149[(((4u16).wrapping_mul(word!(game.menu_option_index))) >> 1) as usize],
            0xc,
            game,
        );
        OptionsMenuFunc5(0, g_word_82F149[((v0 as i32 >> 1) + 1) as usize], 0xc, game);
        OptionsMenuFunc5(0x400, g_word_82F151[(v0 >> 1) as usize], 0xc, game);
        OptionsMenuFunc5(
            0x400,
            g_word_82F151[((v0 as i32 >> 1) + 1) as usize],
            0xc,
            game,
        );
    } else {
        OptionsMenuFunc5(
            0x400,
            g_word_82F149[(((4u16).wrapping_mul(word!(game.menu_option_index))) >> 1) as usize],
            0xc,
            game,
        );
        OptionsMenuFunc5(
            0x400,
            g_word_82F149[((v0 as i32 >> 1) + 1) as usize],
            0xc,
            game,
        );
        OptionsMenuFunc5(0, g_word_82F151[(v0 >> 1) as usize], 0xc, game);
        OptionsMenuFunc5(0, g_word_82F151[((v0 as i32 >> 1) + 1) as usize], 0xc, game);
    };
}
static word_82F204: [u16; 16] = [
    0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80, 0x80,
    0x80,
];
pub unsafe fn GameOptionsMenu_7_ControllerSettings(game: &mut Game) {
    if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        let v0: u16 = word!(game.menu_option_index -= 1, reread);
        if (word!(game.menu_option_index) & 0x8000) == 0 {
            if v0 != 6 {
                return;
            }
            word!(game.game_options_screen_index = 10);
            return;
        }
        word! {game; menu_option_index = 8; game_options_screen_index = 9; }
        return;
    }
    if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        let v1: u16 = (word!(game.menu_option_index)).wrapping_add(1);
        word!(game.menu_option_index = v1);
        if v1 == 7 {
            word!(game.game_options_screen_index = 9);
            return;
        }
        if v1 == 9 {
            word! {game; menu_option_index = 0; game_options_screen_index = 10; }
        }
    } else if word!(game.joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x38, game);
        static kOptionsMenuControllerFuncs: [Func_V; 9] = callback_table! { nonnull Func_V;
            OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0,
            OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_0,
            OptionsMenuControllerFunc_0, OptionsMenuControllerFunc_7, OptionsMenuControllerFunc_8,
        };
        kOptionsMenuControllerFuncs[word!(game.menu_option_index) as usize](game);
    } else if word!(game.joypad2_new_keys) != 0
        && word!(game.menu_option_index) == 8
        && word!(game.debug_invincibility) as i32 - 16i32 & 0x8000 != 0
    {
        if word_82F204[word!(game.debug_invincibility) as usize] & word!(game.joypad2_new_keys)
            == word_82F204[word!(game.debug_invincibility) as usize]
        {
            word!(game.debug_invincibility += 1);
        } else {
            word!(game.debug_invincibility = 0);
        }
    }
}
pub unsafe fn OptionsMenuControllerFunc_8(game: &mut Game) {
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        word! {game; button_config_shoot_x = 64; button_config_jump_a = 128;
        button_config_run_b = 0x8000; button_config_itemcancel_y = 0x4000;
        button_config_itemswitch = 0x2000; button_config_aim_up_R = 16;
        button_config_aim_down_L = 32; }
        LoadControllerOptionsFromControllerBindings(game);
        OptionsMenuFunc6(game);
    }
}
pub unsafe fn OptionsMenuControllerFunc_7(game: &mut Game) {
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0
        && OptionsMenuFunc8(game) == 0
    {
        word!(game.menu_option_index = 0);
        GameOptionsMenuItemFunc_4(game);
    }
}
pub unsafe fn GameOptionsMenu_9_ScrollControllerSettingsDown(game: &mut Game) {
    word!(game.reg_BG1VOFS += 2);
    if word!(game.reg_BG1VOFS) == 32 {
        word!(game.game_options_screen_index = 7);
    }
}
pub unsafe fn GameOptionsMenu_A_ScrollControllerSettingsUp(game: &mut Game) {
    word!(game.reg_BG1VOFS -= 2);
    if word!(game.reg_BG1VOFS) == 0 {
        word!(game.game_options_screen_index = 7);
    }
}
pub use self::sub_82F35B as sub_82F363;
pub unsafe fn sub_82F369(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! {game; eproj_x_vel[(v1 + 3)] = 16; eproj_x_vel[(v1 + 11)] = 3584; }
}
pub unsafe fn LoadControllerOptionsFromControllerBindings(game: &mut Game) {
    for v0 in (0u16..14).step_by(2) {
        let v1: i32 = v0 as i32 >> 1;
        let mut v2: i16 = *(ram_ptr!(game, u16; crate::rtl::rom_table_word!(
            game,
            0x82f54a,
            v1 as isize
        ) as isize)) as i16;
        let clear_button_mapping: bool = ((v2 as i32 & kButton_X as i32) != 0)
            || (if (v2 as i32 & kButton_A as i32) != 0 {
                word!(game.eproj_F[(v1 + 13)] = 1);
                false
            } else if v2 < 0 {
                word!(game.eproj_F[(v1 + 13)] = 2);
                false
            } else if (v2 as i32 & kButton_Select as i32) != 0 {
                word!(game.eproj_F[(v1 + 13)] = 3);
                false
            } else if (v2 as i32 & kButton_Y as i32) != 0 {
                word!(game.eproj_F[(v1 + 13)] = 4);
                false
            } else if (v2 as i32 & kButton_L as i32) != 0 {
                word!(game.eproj_F[(v1 + 13)] = 5);
                false
            } else {
                ((v2 as i32 & kButton_R as i32) == 0)
                    || ({
                        word!(game.eproj_F[(v1 + 13)] = 6);
                        false
                    })
            });
        if clear_button_mapping {
            word!(game.eproj_F[(v1 + 13)] = 0);
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
    let mut v0: i32 = 0;
    do_until! { (v2 >= 12);
        let mut v2: i32 = v0;
        *(ram_ptr!(game, u16; crate::rtl::rom_table_word!(
            game,
            0x82f54a,
            (v0 >> 1) as isize
        ) as isize)) = word_82F575[word!(game.eproj_F[((v0 >> 1) + 13)]) as usize] as u16;
        v0 += 2;
    }
    0
}
static g_word_82F6AD: [u16; 6] = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
pub unsafe fn OptionsMenuFunc6(game: &mut Game) {
    for v0 in (0u16..14).step_by(2) {
        let v1: i32 = v0 as i32 >> 1;
        let v2: u16 = crate::rtl::rom_table_word!(game, 0x82f639, v1 as isize);
        let v3 = RomPtr_82(
            crate::rtl::rom_table_word!(game, 0x82f647, word!(game.eproj_F[(v1 + 13)]) as isize),
            game,
        ) as *const u16;
        word_transfers!(game, ascending, 3;
            native(0x3000 + v2 as isize) <- native_ptr(v3),
        );
        word_transfers!(game, ascending, 3;
            native(0x3040 + v2 as isize) <- native_ptr(v3.offset(3)),
        );
    }
    if word!(game.eproj_instr_list_ptr[0]) != 5 && word!(game.eproj_instr_list_ptr[0]) != 6 {
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
    if word!(game.eproj_instr_list_ptr[1]) != 5 && word!(game.eproj_instr_list_ptr[1]) != 6 {
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
    let mut v0: u16 = 12;
    while (word_82F575[(v0 >> 1) as usize] & word!(game.joypad1_newkeys) as u32) == 0 {
        v0 = (v0).wrapping_sub(2);
        if (v0 & 0x8000) != 0 {
            return;
        }
    }
    let r18: u16 = (v0 >> 1);
    let mut v1: u16 = (2 * word!(game.menu_option_index) as i32 + 2) as u16;
    if 2 * word!(game.menu_option_index) as i32 - 12i32 & 0x8000 == 0 {
        v1 = 0;
    }
    let mut v2: u16 = v1;
    for _ in 0..=5 {
        if *(ram_ptr!(game, u16; 0x1b23)).offset(((v2 as i32 >> 1) + 13) as isize) == r18 {
            break;
        }
        v2 = (v2).wrapping_add(2);
        if ((v2) as i16).wrapping_sub(14) >= 0 {
            v2 = 0;
        }
    }
    let v4: i32 = word!(game.menu_option_index) as i32;
    let r20: u16 = word!(game.eproj_F[(v4 + 13)]);
    word!(game.eproj_F[(v4 + 13)] = r18);
    *(ram_ptr!(game, u16; 0x1b23)).offset(((v2 as i32 >> 1) + 13) as isize) = r20;
    OptionsMenuFunc6(game);
}
