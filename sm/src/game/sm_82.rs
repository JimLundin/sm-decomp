use crate::game::decompression::decompress_to_ram;
use crate::game::gameplay::advance as AdvanceGameplay;
use crate::game::gameplay_flag_runtime;
use crate::game::music::{
    HasQueuedMusic, LoadNewMusicTrackIfChanged, LoadRoomMusic, QueueMusic_Delayed8,
    QueueMusic_DelayedY,
};
pub use crate::game::prelude::*;

use crate::cpu_infra::DebugGameOverMenu;

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;

use crate::game::animated_tiles;
use crate::game::enemy_projectiles::{ClearEprojs, EnableEprojs, EprojRunAll};
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::message_boxes;
use crate::game::palette_effects;
use crate::game::room_mechanisms::{clear, close_door, run_door as run, set_enabled};
use crate::game::samus_animation::{Samus_DrawDuringDeathAnim, Samus_DrawWhenNotAnimatingOrDying};
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_, LoadTitleSequenceGraphics,
};
use crate::game::sm_8f::{PauseHook_DraygonRoom, UnpauseHook_DraygonRoom};
use crate::game::sm_9b::{
    DrawSamusStartingDeathAnim_, GameState_24_SamusNoHealth_Explosion_Helper,
    HandleSamusDeathSequence, StartSamusDeathAnimation,
};
use crate::game::sm_80::{
    CalculateLayer2PosAndScrollsWhenScrolling, ClearOamExt, ClearTimerRam, ClearUnusedOam,
    CopyToVramNow, DecompressToVRAM, DisableIrqInterrupts, DoorTransitionScrollingSetup, DrawTimer,
    EnableIrqInterruptsNow, EnableNMI, FixDoorsMovingUp, GraphicsAsset, HandleFadeIn,
    HandleFadeOut, HandleHudTilemap, InitializeHud, LoadFromLoadStation,
    LoadMirrorOfExploredMapTiles, NextRandom, QueueClearingOfFxTilemap, QueueSfx1_Max6,
    QueueSfx1_Max15, QueueSfx2_Max6, QueueSfx2_Max15, QueueSfx3_Max3, QueueSfx3_Max6,
    QueueSfx3_Max15, ReadJoypadInputs, ReleaseButtonsFilter, ResumeGameplay, SetupDmaTransfer,
    StartGameplay_Async, Vector_NMI, Vector_RESET_Async, WaitForNMI_Async, memset7E,
};
use crate::game::sm_81::{
    DrawMenuSpritemap, DrawSpritemap, FileSelectMap, FileSelectMenu, GameOverMenu, SaveToSram,
};
use crate::game::sm_88::{
    DisableHdmaObjects, EnableHdmaObjects, HdmaObjectHandler, InitializeSpecialEffectsForNewRoom,
    SpawnBG3ScrollHdmaObject, WaitUntilEndOfVblankAndClearHdma, sub_8882AC,
};
use crate::game::sm_89::LoadFXHeader;
use crate::game::sm_90::{CallSomeSamusCode, Samus_LowHealthCheck_0, UpdateBeamTilesAndPalette};
use crate::game::sm_91::{GameState_28_Unused_, Samus_Initialize, Samus_LoadSuitTargetPalette};
use crate::game::sm_a7::{
    Kraid_UnpauseHook_IsSinking, PauseHook_Kraid, UnpauseHook_Kraid_IsAlive,
    UnpauseHook_Kraid_IsDead,
};
use crate::game::sm_a9::MotherBrainsBody_UnpauseHook;
use crate::game::{actors, session_runtime::Runtime};

use crate::host::{Die, Warning};

pub use crate::types::Func_V_Coroutine;

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
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
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db,
    fnDoorTransition_WaitForMusicToClear = 0x82e664,
    fnDoorTransition_HandleTransition = 0x82e6a2, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
    fnCinematicFunctionNone = 0x8b9a47, fnCinematicFunctionOpening = 0x8b9b68,
    fnCinematicFunction_Intro_Initial = 0x8ba395, fnCinematicFunctionBlackoutFromCeres = 0x8bc11b,
    fnCinematicFunctionEscapeFromCebes = 0x8bd480,
}
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: addr < 0x2000;
            b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, pointer __ASSERT_FUNCTION.as_ptr()
        };
        ram_ptr!(game; addr as i32 as isize)
    }
}
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_RoomDefHeader(a: u16, game: &mut Game) -> *mut RoomDefHeader {
    unsafe { RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader }
}
#[inline]
unsafe fn get_DemoRoomData(a: u16, game: &mut Game) -> *mut DemoRoomData {
    unsafe { RomPtr((0x820000u32) | ((a) as u32), game) as *mut DemoRoomData }
}
crate::game::constants::typed_constants! { u32;
    fnDoorTransitionFunction_HandleElevator = 8577405,
    fnPauseHook_Empty = 8946657,
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
        gameplay_flag_runtime::set_boss_byte_at(1, 1, game);
    };
    @external {
        8554788 => [()];
    }
}
pub unsafe fn InitAndLoadGameData_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
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
            set_enabled(true, game);
            palette_effects::set_enabled(true, game);
            EnableHdmaObjects(game);
            animated_tiles::set_enabled(true, game);
            crate::game::player_movement::speed::set_liquid_physics(
                &mut crate::game::session_runtime::Runtime::new(game),
            );
            if word!(game.game_state) as i32 == kGameState_40_TransitionToDemo as i32 {
                word!(game.loop_counter_transfer_enemies_to_vram = 6);
                stage = 2;
            } else if word!(game.loading_game_state) as i32
                == kLoadingGameState_22_EscapingCeres as i32
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
                    actors::staging::transfer_actor_tiles_and_initialize(&mut Runtime::new(game));
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
                if word!(game.loading_game_state) as i32
                    == kLoadingGameState_1F_StartingAtCeres as i32
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
}
pub unsafe fn InitIoForGameplay(game: &mut Game) {
    unsafe {
        InitCpuForGameplay(game);
        InitPpuForGameplay(game);
    }
}
pub unsafe fn InitCpuForGameplay(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn InitPpuForGameplay(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn LoadInitialPalette(game: &mut Game) {
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x9a8000, 0));
}
pub unsafe fn LoadStdBG3andSpriteTilesClearTilemaps(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x2e00).load(game);
        GraphicsAsset::VramWords(0x7e4000, 0x5000, 0x1000).load(game);
        GraphicsAsset::VramWords(0x7e4000, 0x5800, 0x800).load(game);
    }
}
pub unsafe fn GameState_32_MadeItToCeresElevator(game: &mut Game) -> CoroutineRet {
    unsafe {
        if word!(game.timer_status) != 0 {
            DrawTimer(game);
        }
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_33_BlackoutFromCeres(game: &mut Game) -> CoroutineRet {
    unsafe {
        if word!(game.timer_status) != 0 {
            DrawTimer(game);
        }
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_35_TimeUp(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_36_WhitingOutFromTimeUp(game: &mut Game) -> CoroutineRet {
    unsafe {
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
}
pub unsafe fn GameState_38_SamusEscapesFromZebes(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_41_TransitionToDemo(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
        if _coret_ != 0 {
            return _coret_;
        }
        HdmaObjectHandler(game);
        word!(game.game_state += 1);
        *ram_ptr!(game; 0x51) = 15;
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_42_PlayingDemo_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::verify_probe::hit("coroutine.demo");
        let resume_state: *mut u8 = ram_ptr!(game; 0x614);
        let resume = *resume_state;
        'done: {
            if resume > 2 {
                break 'done;
            }
            'finish_demo: {
                if resume != 1 {
                    let yielded: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_43_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    unsafe {
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
        palette_effects::set_enabled(false, game);
        palette_effects::clear(game);
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
}
pub unsafe fn GameState_44_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    unsafe {
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
}
pub unsafe fn CheckForNextDemo(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn LoadDemoRoomData(game: &mut Game) {
    unsafe {
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
}
static kGameStateFuncs: [Func_V_Coroutine; 45] = callback_table! { nonnull Func_V_Coroutine;
    GameState_0_Reset_Async, GameState_1_OpeningCinematic, GameState_2_GameOptionsMenu,
    GameState_3_Null, GameState_4_FileSelectMenus, GameState_5_FileSelectMap,
    InitAndLoadGameData_Async, GameState_7_MainGameplayFadeIn, AdvanceGameplay,
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
    unsafe {
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
}
pub unsafe fn RunOneFrameOfGame(game: &mut Game) {
    unsafe {
        let ret: CoroutineRet = RunOneFrameOfGameInner(game);
        if ret == 0 {
            *ram_ptr!(game; 0x5b4) = 1;
        }
        Vector_NMI(game);
    }
}
pub unsafe fn GameState_29_DebugGameOverMenu(game: &mut Game) -> CoroutineRet {
    unsafe {
        DebugGameOverMenu();
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_26_GameOverMenu(game: &mut Game) -> CoroutineRet {
    unsafe {
        GameOverMenu(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_4_FileSelectMenus(game: &mut Game) -> CoroutineRet {
    unsafe {
        FileSelectMenu(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_5_FileSelectMap(game: &mut Game) -> CoroutineRet {
    unsafe {
        FileSelectMap(game);
        kCoroutineNone as CoroutineRet
    }
}
static kSfxHandlers: [Func_Y_V; 5] = callback_table! { nonnull Func_Y_V;
    SfxHandlers_0_SendToApu, SfxHandlers_1_WaitForAck, SfxHandlers_2_ClearRequest,
    SfxHandlers_3_WaitForAck, SfxHandlers_4_Reset,
};
pub unsafe fn HandleSoundEffects(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn SfxHandlers_0_SendToApu(j: u16, game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0x643, j as isize) != *ram_ptr!(game; 0x646, j as isize) {
            let v2: u8 = (*ram_ptr!(game; 0x643, j as isize) as i32 + j as i32 * 16) as u8;
            let v3: u8 = *ram_ptr!(game; 0x656, v2 as isize);
            RtlApuWrite((j as i32 + APUI01 as i32) as SnesRegs, v3, game);
            *ram_ptr!(game; 0x64d, j as isize) = v3;
            *ram_ptr!(game; 0x643, j as isize) = (v2 as i32 + 1 & 0xf) as u8;
            crate::update_place! { reference fresh134 = *ram_ptr!(game; 0x649, j as isize) => (*fresh134).wrapping_add(1); }
        }
    }
}
pub unsafe fn SfxHandlers_1_WaitForAck(j: u16, game: &mut Game) {
    unsafe {
        let v1: u8 = *ram_ptr!(game; 0x64d, j as isize);
        crate::update_place! { reference fresh138 = *ram_ptr!(game; 0x649, j as isize) => (*fresh138).wrapping_add(1); }
        *ram_ptr!(game; 0x650, j as isize) = 6;
    }
}
pub unsafe fn SfxHandlers_3_WaitForAck(j: u16, game: &mut Game) {
    unsafe {
        let v1: u8 = *ram_ptr!(game; 0x64d, j as isize);
        *ram_ptr!(game; 0x649, j as isize) = 0;
        SfxHandlers_0_SendToApu(j, game);
    }
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
    unsafe {
        *(ram_ptr!(game, u16; 0x643)) = 0;
        *(ram_ptr!(game, u16; 0x643, 2)) = 0;
        *(ram_ptr!(game, u16; 0x646, 1)) = 0;
        *(ram_ptr!(game, u16; 0x649)) = 0;
        *ram_ptr!(game; 0x649, 2) = 0;
    }
}
pub unsafe fn ShowSpareCpu(game: &mut Game) {
    unsafe {
        word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
    }
}
pub unsafe fn GameState_0_Reset_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        word! { game, 0; UNUSED_word_7E0DF8, UNUSED_word_7E0DFA, UNUSED_word_7E0DFC }
        word! {game; cinematic_function = ((0x8b9b68i32) as u16); demo_set = 0; }
        if word!(game.num_demo_sets) == 4 {
            word!(game.demo_set = 3);
        }
        word!(game.game_state += 1);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_1_OpeningCinematic(game: &mut Game) -> CoroutineRet {
    unsafe { GameState_1_OpeningCinematic_(game) }
}
pub unsafe fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
    kCoroutineNone as CoroutineRet
}
crate::forward_calls! { pub unsafe fn (game: &mut Game) -> CoroutineRet;
    GameState_37_CeresGoesBoomWithSamus => GameState_37_CeresGoesBoomWithSamus_(game);
    GameState_39_EndingAndCredits => GameState_39_EndingAndCredits_(game);
}
pub unsafe fn GameState_7_MainGameplayFadeIn(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    unsafe {
        GameState_28_Unused_(game);
        kCoroutineNone as CoroutineRet
    }
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
    unsafe {
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
}
pub fn OptionsPreInstr_nullsub_57(k: u16) {}
pub unsafe fn OptionsMenuFunc1(game: &mut Game) {
    unsafe {
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
    unsafe {
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
}
pub unsafe fn DrawOptionsMenuSpritemaps(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameState_12_Pausing_Darkening_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
    unsafe {
        DisableHdmaObjects(game);
        crate::rtl::write_register_and_mirror(HDMAEN as u16, 0, 0x85, true, game);
        animated_tiles::set_enabled(false, game);
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
}
pub unsafe fn BackupBG2TilemapForPauseMenu(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn RestoreBG2TilemapFromPauseScreen(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(
            0x7edf5c,
            ((*ram_ptr!(game; 0x59) as u16 & 0xfc) << 8),
            0x1000,
        )
        .load(game);
    }
}
pub unsafe fn BackupSomeGfxStateForPause(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn RestoreSomeGfxStateForPause(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn LoadPauseMenuTilesAndClearBG2(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(0xb68000, 0, 0x4000).load(game);
        GraphicsAsset::VramWords(0xb6c000, 0x2000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
    }
}
pub unsafe fn LoadPauseScreenBaseTilemaps(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn LoadEqupmentScreenReserveHealthTilemap(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn BackupGameplayPalettesAndLoadForPause(game: &mut Game) {
    word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0xb6f000, 0));
}
pub unsafe fn ContinueInitPauseMenu(game: &mut Game) {
    unsafe {
        SetupPpuForPauseMenu(game);
        ResetPauseMenuAnimations(game);
        LoadEquipmentScreenEquipmentTilemaps(game);
        set_pause_label_palettes(0, game);
        UpdatePauseMenuLRStartVramTilemap(game);
        DetermineMapScrollLimits(game);
        SetupMapScrollingForPauseMenu(0x80, game);
    }
}
pub unsafe fn SetupMapScrollingForFileSelectMap(game: &mut Game) {
    unsafe {
        ResetPauseMenuAnimations(game);
        DetermineMapScrollLimits(game);
        word!(
            game.reg_BG1HOFS = (word!(game.map_min_x_scroll) as i32
                + (((word!(game.map_max_x_scroll)).wrapping_sub(word!(game.map_min_x_scroll)))
                    as i32
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
            + (((word!(game.map_max_y_scroll)).wrapping_sub(word!(game.map_min_y_scroll))) as i32
                >> 1)
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
}
pub unsafe fn GameState_14_Paused_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        DrawPauseMenuDuringFadeIn(game);
        HandleFadeIn(game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_15_Paused_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        ReleaseButtonsFilter(3, game);
        MainPauseRoutine(game);
        HandleHudTilemap(game);
        HandlePauseScreenPaletteAnimation(game);
        kCoroutineNone as CoroutineRet
    }
}
static kPauseMenuFuncs: [Func_V; 8] = callback_table! { nonnull Func_V;
    PauseMenu_0_MapScreen, PauseMenu_1_EquipmentScreen, PauseMenu_2,
    PauseMenu_3_MapToEquipment_Load, PauseMenu_4_MapToEquipment_FadeIn, PauseMenu_5,
    PauseMenu_6_EquipmentToMap_Load, PauseMenu_7_EquipmentToMap_FadeIn,
};
pub unsafe fn MainPauseRoutine(game: &mut Game) {
    unsafe {
        kPauseMenuFuncs[word!(game.menu_index) as usize](game);
    }
}
pub unsafe fn PauseMenu_0_MapScreen(game: &mut Game) {
    unsafe {
        HandlePauseScreenLR(game);
        HandlePauseScreenStart(game);
        HandleMapScrollArrows(game);
        MapScrolling(game);
        MapScreenDrawSamusPositionIndicator(game);
        DrawMapIcons(game);
        DisplayMapElevatorDestinations(game);
        word!(game.pause_screen_mode = 0);
    }
}
pub unsafe fn PauseMenu_1_EquipmentScreen(game: &mut Game) {
    unsafe {
        word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
        EquipmentScreenMain(game);
        HandlePauseScreenLR(game);
        HandlePauseScreenStart(game);
        word!(game.pause_screen_mode = 1);
    }
}
pub unsafe fn PauseMenu_2(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn PauseMenu_5(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn PauseMenu_3_MapToEquipment_Load(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn PauseMenu_6_EquipmentToMap_Load(game: &mut Game) {
    unsafe {
        DisplayMapElevatorDestinations(game);
        LoadPauseMenuMapTilemapAndAreaLabel(game);
        SetPauseScreenButtonLabelPalettes(game);
        word!(game.pausemenu_lr_animation_frame = 0);
        word!(game.pausemenu_lr_animation_timer = *(rom_ptr!(game, u8; 0x82c10c) as *mut u16));
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.pause_screen_mode = 0);
        word!(game.menu_index += 1);
    }
}
pub unsafe fn PauseMenu_7_EquipmentToMap_FadeIn(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn PauseMenu_4_MapToEquipment_FadeIn(game: &mut Game) {
    unsafe {
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
}
static kMapScrollingFuncs: [Func_V; 5] = callback_table! { nonnull Func_V;
    MapScrolling_0_None, MapScrolling_1_Left, MapScrolling_2_Right, MapScrolling_3_Up,
    MapScrolling_4_Down,
};
pub unsafe fn MapScrolling(game: &mut Game) {
    unsafe {
        kMapScrollingFuncs[word!(game.map_scrolling_direction) as usize](game);
    }
}
pub unsafe fn MapScrolling_0_None(game: &mut Game) {
    unsafe {
        word!(game.map_scrolling_gear_switch_timer = 4);
    }
}
pub unsafe fn MapScrolling_GetSpeedIndex(game: &mut Game) -> u16 {
    unsafe {
        let result: u16 = word!(game.map_scrolling_speed_index);
        if word!(game.map_scrolling_gear_switch_timer) == 0 {
            return (word!(game.map_scrolling_speed_index)).wrapping_add(32);
        }
        result
    }
}
static kMapScrollingSpeedTable: [u16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub unsafe fn MapScrolling_1_Left(game: &mut Game) {
    unsafe {
        word!(
            game.reg_BG1HOFS -=
                kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
        );
        MapScrolling_Common(game);
    }
}
pub unsafe fn MapScrolling_Common(game: &mut Game) {
    unsafe {
        word!(game.map_scrolling_speed_index += 1);
        if (word!(game.map_scrolling_speed_index += 1, reread) & 0xf) == 0 {
            QueueSfx1_Max6(0x36, game);
            word! { game, 0; map_scrolling_direction, map_scrolling_speed_index }
            if word!(game.map_scrolling_gear_switch_timer) != 0 {
                word!(game.map_scrolling_gear_switch_timer -= 1);
            }
        }
    }
}
pub unsafe fn MapScrolling_2_Right(game: &mut Game) {
    unsafe {
        word!(
            game.reg_BG1HOFS +=
                kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
        );
        MapScrolling_Common(game);
    }
}
pub unsafe fn MapScrolling_3_Up(game: &mut Game) {
    unsafe {
        word!(
            game.reg_BG1VOFS -=
                kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
        );
        MapScrolling_Common(game);
    }
}
pub unsafe fn MapScrolling_4_Down(game: &mut Game) {
    unsafe {
        word!(
            game.reg_BG1VOFS +=
                kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) >> 1) as usize]
        );
        MapScrolling_Common(game);
    }
}
pub unsafe fn GameState_16_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
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
}
pub unsafe fn DrawPauseMenuDuringFadeout(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameState_17_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
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
            let mut _coret_: u8 =
                CallUnpauseHook_Async(Load24(ram_ptr!(game, LongPtr; 0x604)), game);
            if _coret_ != 0 {
                *_state_variable_ = 1;
                return _coret_;
            }
            EnableHdmaObjects(game);
            animated_tiles::set_enabled(true, game);
            crate::game::player_projectiles::restore_sound(game);
            word!(game.game_state += 1);
        }
        *_state_variable_ = 0;
        0 as CoroutineRet
    }
}
pub unsafe fn GameState_18_Unpausing(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
#[derive(Clone, Copy)]
enum MapScreen {
    Pause,
    FileSelect,
}

// Render both map screens with the original mask/ROM/RAM access order.
unsafe fn render_map_tilemap(screen: MapScreen, game: &mut Game) {
    unsafe {
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
                    #[cfg(sm_verify_dll)]
                    crate::game::session_runtime::player_legacy_edges::map_write(
                        game,
                        byte as usize,
                        tile as usize,
                        0,
                    );
                    if previous & 0x80 == 0 {
                        *r3.offset(tile) = match screen {
                            MapScreen::Pause => 31,
                            MapScreen::FileSelect => 15,
                        };
                    } else {
                        *explored = (*explored).wrapping_add(1);
                        #[cfg(sm_verify_dll)]
                        crate::game::session_runtime::player_legacy_edges::map_write(
                            game,
                            byte as usize,
                            tile as usize,
                            1,
                        );
                        *r3.offset(tile) = *r0.offset(tile) & 0xfbff;
                    }
                    #[cfg(sm_verify_dll)]
                    crate::game::session_runtime::player_legacy_edges::map_write(
                        game,
                        byte as usize,
                        tile as usize,
                        2,
                    );
                    tile += 1;
                }
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
    unsafe {
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
}

pub unsafe fn LoadPauseMenuMapTilemapAndAreaLabel(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn LoadPauseMenuMapTilemap(game: &mut Game) {
    unsafe {
        render_map_tilemap(MapScreen::Pause, game);
    }
}
pub unsafe fn DrawRoomSelectMap(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x5d) = 51;
        *ram_ptr!(game; 0x69) = 19;
        word!(game.reg_BG1VOFS = -40i32 as u16);
        render_map_tilemap(MapScreen::FileSelect, game);
        let v16: u16 = word!(game.vram_write_queue_tail);
        let v17 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v17).size = 4096;
        (*v17).src.addr = (ram_ptr!(game, Ram3000; 0x3000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            as VoidP;
        (*v17).src.bank = 126;
        (*v17).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
        word!(game.vram_write_queue_tail = (v16).wrapping_add(7));
    }
}
pub unsafe fn DrawRoomSelectMapAreaLabel(dst: *mut u16, game: &mut Game) {
    unsafe {
        let v2 = RomPtr_82(
            crate::rtl::rom_table_word!(game, 0x82965f, word!(game.area_index) as isize),
            game,
        ) as *const u16;
        for i in 0i32..12 {
            *dst.offset(i as isize) = (*v2.offset(i as isize)) & (0xefffu16);
        }
    }
}
pub unsafe fn SetupMapScrollingForPauseMenu(a: u16, game: &mut Game) {
    unsafe {
        word!(
            game.reg_BG1HOFS = (word!(game.map_min_x_scroll) as i32
                + (((word!(game.map_max_x_scroll)).wrapping_sub(word!(game.map_min_x_scroll)))
                    as i32
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
            + (((word!(game.map_max_y_scroll)).wrapping_sub(word!(game.map_min_y_scroll))) as i32
                >> 1)
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
}
static k0x80Shr: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];

crate::forward_calls! { unsafe fn (r0: *const u8) -> u16;
    DetermineLeftmostMapColumn => find_map_edge(r0, MapEdge::Left);
    DetermineRightmostMapColumn => find_map_edge(r0, MapEdge::Right);
    DetermineTopmostMapColumn => find_map_edge(r0, MapEdge::Top);
    DetermineBottommostMapColumn => find_map_edge(r0, MapEdge::Bottom);
}
pub unsafe fn DetermineMapScrollLimits(game: &mut Game) {
    unsafe {
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
        word!(
            game.map_max_x_scroll = (DetermineRightmostMapColumn(r6.offset(131))).wrapping_mul(8)
        );
        word!(game.map_min_y_scroll = (DetermineTopmostMapColumn(r6)).wrapping_mul(8));
        word!(
            game.map_max_y_scroll = (DetermineBottommostMapColumn(r6.offset(124))).wrapping_mul(8)
        );
    }
}
pub unsafe fn SetupPpuForPauseMenu(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn ResetPauseMenuAnimations(game: &mut Game) {
    unsafe {
        word! { game, 0;
            pausemenu_reserve_tank_delay_ctr, reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
            reg_BG3VOFS, pausemenu_button_label_mode, pausemenu_lr_animation_frame, UNUSED_word_7E0745,
            samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
            samus_position_indicator_animation_loop_counter,
        }
        word! {game; pausemenu_lr_animation_timer = *(rom_ptr!(game, u8; 0x82c10c) as *mut u16);
        pausemenu_palette_animation_timer = 1; pausemenu_palette_animation_frame = 0; }
    }
}
unsafe fn ChangePaletteValues(target: *mut u16, new_color: u16, n: size_t) {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i != n {
            *target.offset(i as isize) = (*target.offset(i as isize) & !(0x1c00) | new_color);
            i = i.wrapping_add(1);
        }
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
    unsafe {
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
        let mut p: *const u16 =
            RomPtr_82(addr_kEquipmentTilemapOffs_Weapons as u16, game) as *mut u16;
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
}
pub unsafe fn ClearSamusBeamTiles(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1000).load(game);
    }
}
pub unsafe fn ContinueInitGameplayResume(game: &mut Game) {
    unsafe {
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- pause_map(384));
        SetupPpuForGameplayResume(game);
        CalculateBgScrolls_Unpause(game);
        UpdateBeamTilesAndPalette_Unpause(game);
        ClearPauseMenuData(game);
        CallSomeSamusCode(0xc, game);
    }
}
pub unsafe fn SetupPpuForGameplayResume(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            mirror(OBSEL as u16, 3, 0x52, false); mirror(BGMODE as u16, 9, 0x55, false);
            mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
            mirror(BG1SC as u16, 0x51, 0x58, true); mirror(BG2SC as u16, 0x49, 0x59, true);
            mirror(BG3SC as u16, 0x5a, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        }
    }
}
pub unsafe fn CalculateBgScrolls_Unpause(game: &mut Game) {
    unsafe {
        word!(game.reg_BG1HOFS = (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos)));
        word!(game.reg_BG1VOFS = (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos)));
        word!(game.reg_BG2HOFS = (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos)));
        word!(game.reg_BG2VOFS = (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos)));
    }
}
pub unsafe fn UpdateBeamTilesAndPalette_Unpause(game: &mut Game) {
    unsafe {
        UpdateBeamTilesAndPalette(game);
    }
}
pub unsafe fn ClearPauseMenuData(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HandlePauseScreenLR(game: &mut Game) {
    unsafe {
        HandlePauseScreenLrInput(game);
        DrawLrHighlight(game);
    }
}
pub unsafe fn HandlePauseScreenLrInput(game: &mut Game) {
    unsafe {
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
}
static kPauseLrButtonPressedHighlight_Spritemap: [u16; 2] = [0x28, 0x29];
static kPauseLrButtonPressedHighlight_Y: [u16; 2] = [0x18, 0xe8];
static kPauseLrButtonPressedHighlight_X: [u16; 2] = [0xd0, 0xd0];
pub unsafe fn HandlePauseMenuLRPressHighlight(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn DrawLrHighlight(game: &mut Game) {
    unsafe {
        DrawPauseScreenSpriteAnim(2, 0x18, 0xd0, game);
        DrawPauseScreenSpriteAnim(2, 0xe8, 0xd0, game);
    }
}
pub unsafe fn HandlePauseScreenStart(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HighlightPauseScreenButton(game: &mut Game) {
    unsafe {
        if word!(game.pausemenu_start_lr_pressed_highlight_timer) != 0 {
            word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
            DrawMenuSpritemap(0x2b, 0x90, 0xd0, 0, game);
        }
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
    unsafe {
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
}
pub unsafe fn DrawPauseScreenSpriteAnim(a: u16, input_k: u16, input_j: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HandlePauseScreenPaletteAnimation(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn EquipmentScreenSetupReserveMode(game: &mut Game) {
    unsafe {
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
        word!(
            game.pausemenu_item_selector_animation_timer = *(rom_ptr!(game, u8; 0x82c10c)) as u16
        );
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
                let item_found = loop {
                    if (crate::rtl::rom_table_word!(game, 0x82c056, (v6 >> 1) as isize)
                        & word!(game.collected_items))
                        != 0
                    {
                        byte!(v7; 0) = ((v6 >> 1) >> 8) as u8;
                        byte!(v7; 1) = (v6 >> 1) as u8;
                        word!(game.pausemenu_equipment_category_item = ((v7) as u16) | (2u16));
                        break true;
                    } else {
                        v6 = (v6).wrapping_add(2);
                        if (((v6) as i16).wrapping_sub(12) >= 0) {
                            break false;
                        }
                    }
                };
                if !item_found {
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
}
pub unsafe fn EquipmentScreenTransferBG1Tilemap(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(0x7e3800, 0x3000, 0x800).load(game);
        word!(game.reg_BG1VOFS = 0);
    }
}
static kEquipmentScreenCategories: [Func_V; 4] = callback_table! { nonnull Func_V;
    EquipmentScreenCategory_Tanks, EquipmentScreenCategory_Weapons,
    EquipmentScreenCategory_Suit, EquipmentScreenCategory_Boots,
};
pub unsafe fn EquipmentScreenMain(game: &mut Game) {
    unsafe {
        kEquipmentScreenCategories[word!(game.pausemenu_equipment_category_item) as u8 as usize](
            game,
        );
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
        WriteSamusWireframeTilemapAndQueue(game);
    }
}
static kEquipmentScreenCategory_TanksFuncs: [Func_V; 2] = callback_table! { nonnull Func_V;
    EquipmentScreenCategory_Tanks_0, EquipmentScreenCategory_Tanks_1,
};
pub unsafe fn EquipmentScreenCategory_Tanks(game: &mut Game) {
    unsafe {
        kEquipmentScreenCategory_TanksFuncs
            [byte!(@pointer ram_ptr!(game, u16; 0x755); 1isize) as usize](game);
        EquipmentScreenHandleDpad(game);
        EquipmentScreenGlowingArrow(game);
    }
}
pub unsafe fn EquipmentScreenHandleDpad(game: &mut Game) {
    unsafe {
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
}
static kEquipmentScreenGlowingArrowFuncs: [Func_V; 2] = callback_table! { nonnull Func_V;
    EquipmentScreenGlowingArrowAnimated, EquipmentScreenGlowingArrowSolidOn,
};
pub unsafe fn EquipmentScreenGlowingArrow(game: &mut Game) {
    unsafe {
        if word!(game.pausemenu_equipment_category_item) as u8 != 0 {
            EquipmentScreenGlowingArrowSolidOff(game);
        } else {
            kEquipmentScreenGlowingArrowFuncs
                [byte!(@pointer ram_ptr!(game, u16; 0x755); 1isize) as usize](game);
        };
    }
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
    unsafe {
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
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOn(game: &mut Game) {
    unsafe {
        word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
        EquipmentScreenEnergyArrowGlow_On(game);
    }
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOff(game: &mut Game) {
    unsafe {
        word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
        EquipmentScreenEnergyArrowGlow_Off(game);
    }
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_On(game: &mut Game) {
    unsafe {
        for i in 0i32..8 {
            (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
                [(129 + i * 32) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
                .equipment_screen_bg1_tilemap[(129 + i * 32) as usize]
                & !(0x1c00)
                | 0x1800);
        }
        for i_0 in 0i32..2 {
            (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
                [(385 + i_0) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
                .equipment_screen_bg1_tilemap[(385 + i_0) as usize]
                & !(0x1c00)
                | 0x1800);
        }
    }
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_Off(game: &mut Game) {
    unsafe {
        for i in 0i32..8 {
            (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
                [(129 + i * 32) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
                .equipment_screen_bg1_tilemap[(129 + i * 32) as usize]
                & !(0x1c00)
                | 0x1c00);
        }
        for i_0 in 0i32..2 {
            (*(ram_ptr!(game, Ram3800; 0x3800))).equipment_screen_bg1_tilemap
                [(385 + i_0) as usize] = ((*(ram_ptr!(game, Ram3800; 0x3800)))
                .equipment_screen_bg1_tilemap[(385 + i_0) as usize]
                & !(0x1c00)
                | 0x1c00);
        }
    }
}
static kEquipmentScreenTilemap_MANUAL: [u16; 4] = [0x3d46, 0x3d47, 0x3d48, 0x3d49];
static kEquipmentScreenTilemap_AUTO: [u16; 4] = [0x3d56, 0x3d57, 0x3d58, 0x3d59];
pub unsafe fn EquipmentScreenCategory_Tanks_0(game: &mut Game) {
    unsafe {
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
                        [(v1 as i32 + 327) as usize] = (kEquipmentScreenTilemap_MANUAL
                        [v1 as usize]
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
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game: &mut Game) {
    unsafe {
        let mut v0: u16 = -26229i32 as u16;
        if word!(game.samus_reserve_health) == 0 {
            v0 = -26217i32 as u16;
        }
        let v1 = RomPtr_82(v0, game) as *const u16;
        word! {game; hud_tilemap[8] = *v1; hud_tilemap[9] = *v1.offset(1);
        hud_tilemap[40] = *v1.offset(2); hud_tilemap[41] = *v1.offset(3);
        hud_tilemap[72] = *v1.offset(4); hud_tilemap[73] = *v1.offset(5); }
    }
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_Off(game: &mut Game) {
    unsafe {
        word! { game, 11279;
            hud_tilemap[8], hud_tilemap[9], hud_tilemap[40], hud_tilemap[41], hud_tilemap[72],
            hud_tilemap[73],
        }
    }
}
static kReserveTankEnergyTransferPerFrame: u16 = 1;
pub unsafe fn EquipmentScreenCategory_Tanks_1(game: &mut Game) {
    unsafe {
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
        if ((word!(game.samus_health)) as i16).wrapping_sub((word!(game.samus_max_health)) as i16)
            < 0
        {
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
}
pub unsafe fn EquipmentScreenCategory_Weapons(game: &mut Game) {
    unsafe {
        EquipmentScreenCategory_Weapons_MoveButtons(game);
        let R36: u16 = word!(game.equipped_beams);
        if word!(game.collected_beams) != 0
            && (word!(game.pausemenu_equipment_category_item) as u8) == 1
        {
            EquipmentScreenCategory_ButtonResponse(10, game);
            EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36, game);
        }
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_MoveButtons(game: &mut Game) {
    unsafe {
        let r18: u16 = word!(game.pausemenu_equipment_category_item);
        if (word!(game.joypad1_newkeys) & 0x100) != 0 {
            if (word!(game.joypad1_newkeys) & 0x800) != 0 {
                EquipmentScreenMoveLowerOnSuitsMisc(0, game);
            } else if EquipmentScreenMoveLowerOnSuitsMisc(4, game) != 0 {
                EquipmentScreenMoveToHighJumpOrLowerInBoots(0, r18, game);
            }
        } else if (word!(game.joypad1_newkeys) & 0x400) != 0 {
            if word!(game.hyper_beam_flag) == 0
                && word!(game.pausemenu_equipment_category_item) != 1025
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
}
pub unsafe fn EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn EquipmentScreenCategory_Suit(game: &mut Game) {
    unsafe {
        EquipmentScreenCategory_Suit_MoveResponse(game);
        if (word!(game.pausemenu_equipment_category_item) as u8) == 2 {
            EquipmentScreenCategory_ButtonResponse(18, game);
        }
    }
}
pub unsafe fn EquipmentScreenCategory_Suit_MoveResponse(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn EquipmentScreenCategory_Boots(game: &mut Game) {
    unsafe {
        EquipmentScreenCategory_Boots_MoveResponse(game);
        if (word!(game.pausemenu_equipment_category_item) as u8) == 3 {
            EquipmentScreenCategory_ButtonResponse(18, game);
        }
    }
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveResponse(game: &mut Game) {
    unsafe {
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
    unsafe {
        WriteSamusWireframeTilemap(game);
        queue_ram_tilemap_write!(game, 1280, 256, fresh63, 0x3080);
    }
}
pub unsafe fn WriteSamusWireframeTilemap(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn EquipmentScreenDrawItemSelector(game: &mut Game) {
    unsafe {
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
                    + 4 * byte!(@pointer ram_ptr!(game, u16; 0x755); 1) as i32)
                    as u16,
                game,
            ) as *mut u16;
            DrawPauseScreenSpriteAnim(3, (*t).wrapping_sub(1), *t.offset(1), game);
        }
    }
}
pub unsafe fn EquipmentScreenDisplayReserveTankAmount(game: &mut Game) {
    unsafe {
        EquipmentScreenDisplayReserveTankAmount_(game);
    }
}
static kEquipmentScreenReserveTank_X: [u16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
static kEquipmentScreenReserveTank_Y: u16 = 0x60;
static kPartialReserveTankSpritemapIds: [u16; 16] = [
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
];
pub unsafe fn EquipmentScreenDisplayReserveTankAmount_(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn EquipmentScreenDisplayReserves_PaletteSetup(game: &mut Game) -> u16 {
    unsafe {
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
}
pub unsafe fn EquipmentScreenMoveToReserveTanks(game: &mut Game) -> u16 {
    unsafe {
        let result: u16 = word!(game.samus_max_reserve_health);
        if word!(game.samus_max_reserve_health) != 0 {
            word!(game.pausemenu_equipment_category_item = 0);
            QueueSfx1_Max6(0x37, game);
            return 1;
        }
        result
    }
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
    unsafe {
        let (table, limit) = match category {
            1 => (0xc04c, 10),
            2 => (0xc056, 10),
            3 => (0xc062, 6),
            _ => unreachable!(),
        };
        let mut found = category != 1 || word!(game.hyper_beam_flag) == 0;
        while found {
            let mask =
                crate::rtl::rom_table_word!(game, 0x820000 | table as u32, (offset >> 1) as isize,);
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
}
pub unsafe fn EquipmentScreenMoveToBeams(v0: u16, r18: u16, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("equipment-search-beams-down");
        move_equipment_selection(1, v0, false, Some(r18), game);
    }
}
pub unsafe fn EquipmentScreenMoveToBottomOfBeams(k: u16, game: &mut Game) -> u16 {
    unsafe {
        crate::verify_probe::hit("equipment-search-beams-up");
        move_equipment_selection(1, k, true, None, game)
    }
}
pub unsafe fn EquipmentScreenMoveLowerOnSuitsMisc(v0: u16, game: &mut Game) -> u16 {
    unsafe {
        crate::verify_probe::hit("equipment-search-suits-down");
        move_equipment_selection(2, v0, false, None, game)
    }
}
pub unsafe fn EquipmentScreenMoveToScrewOrHigherOnSuits(v0: u16, r18: u16, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("equipment-search-suits-up");
        move_equipment_selection(2, v0, true, Some(r18), game);
    }
}
pub unsafe fn EquipmentScreenMoveToHighJumpOrLowerInBoots(v0: u16, r18: u16, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("equipment-search-boots-down");
        move_equipment_selection(3, v0, false, Some(r18), game);
    }
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveUpInBoots(k: u16, game: &mut Game) -> u16 {
    unsafe {
        crate::verify_probe::hit("equipment-search-boots-up");
        move_equipment_selection(3, k, true, None, game)
    }
}
pub unsafe fn EquipmentScreenCategory_ButtonResponse(r24: u16, game: &mut Game) {
    unsafe {
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
                    (crate::rtl::rom_table_word!(game, 0x82c044, category as isize) as i32
                        + item * 2) as u16,
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
}
pub unsafe fn DrawPauseMenuDuringFadeIn(game: &mut Game) {
    unsafe {
        if word!(game.menu_index) != 0 {
            EquipmentScreenDrawItemSelector(game);
            EquipmentScreenDisplayReserveTankAmount(game);
        } else {
            MapScreenDrawSamusPositionIndicator(game);
            DrawMapIcons(game);
            DisplayMapElevatorDestinations(game);
        };
    }
}
static kMap_Criteria_SavePoints: [u16; 16] = [
    0xd8, 0x28, 0x90, 0x38, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe,
    0xfffe, 0xfffe, 0xfffe,
];
pub unsafe fn DrawMapIcons(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn DrawFileSelectMapIcons(game: &mut Game) {
    unsafe {
        HandlePauseScreenPaletteAnimation(game);
        DrawBossMapIcons(9, 0xc7cb, game);
        DrawSimpleMapIcons(0xb, 0xc7db, 3584, game);
        DrawSimpleMapIcons(0xa, 0xc7eb, 3584, game);
        DrawSimpleMapIcons(0x4e, 0xc7fb, 3584, game);
        let a: u16 = UpdateSamusPositionIndicatorAnimation(game);
        let data: *const u16 = (RomPtr_82(
            *((&raw mut (*(rom_ptr!(game, MapIconDataPointers; 0x82c7cb)).offset(4)).crateria
                as *mut u8)
                .offset((2i32 * word!(game.area_index) as i32) as isize)
                as *mut VoidP),
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
}
pub unsafe fn DrawSaveStationMapIcon(a: u16, k: u16, r3: u16, game: &mut Game) {
    unsafe {
        static kShlBit: [u8; 8] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80];
        let R34: u16 = a;
        if word!(game.area_index) == word!(game.sram_area_index) {
            let mut R36: u16 =
                *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
            byte!(R36; 0) = ((R36) as u8) & (kShlBit[word!(game.load_station_index) as usize]);
            let v2: u16 = *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game)
                as *mut u16);
            if v2 != 0 {
                DrawMapIconsOfType(v2, R34, R36, r3, game);
            }
        }
    }
}
pub unsafe fn DrawDebugSaveMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    unsafe {
        let R34: u16 = a;
        let R36: u16 = *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
        let v2: u16 =
            *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
        if v2 != 0 {
            DrawMapIconsOfType(v2, R34, R36, r3, game);
        }
    }
}
pub unsafe fn DrawDebugElevatorMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    unsafe {
        let R34: u16 = a;
        let R36: u16 =
            *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32 + 1) as isize));
        let v2: u16 =
            *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
        if v2 != 0 {
            DrawMapIconsOfType(v2, R34, R36, r3, game);
        }
    }
}
pub unsafe fn DrawSimpleMapIcons(a: u16, k: u16, r3: u16, game: &mut Game) {
    unsafe {
        let R34: u16 = a;
        let v2: u16 =
            *(RomPtr_82((k as i32 + 2 * word!(game.area_index) as i32) as u16, game) as *mut u16);
        if v2 != 0 {
            DrawMapIconsOfType(v2, R34, -1i32 as u16, r3, game);
        }
    }
}
pub unsafe fn DrawMapIconsOfType(mut a: u16, r34: u16, mut r36: u16, r3: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn CheckIfMapPositionIsExplored(k: u16, j: u16, game: &mut Game) -> u16 {
    unsafe {
        static kBits0x80Shr: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
        let mut r18: i32 = ((k) & (0xff00u16)) as i32 >> 1;
        r18 += k as u8 as i32 >> 6;
        r18 += ((j) & (0xfff8u16)) as i32 >> 1;
        let v3: u16 = (k as i32 >> 3 & 7) as u16;
        ((kBits0x80Shr[v3 as usize] & *ram_ptr!(game; 0x7f7, r18 as isize)) != 0) as u16
    }
}
pub unsafe fn DrawBossMapIcons(a: u16, k: u16, game: &mut Game) {
    unsafe {
        let mut bits: i32 =
            gameplay_flag_runtime::boss_byte_at(word!(game.area_index), game) as i32;
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
}
pub unsafe fn DrawMapScrollArrowAndCheckToScroll(db: u8, k: u16, game: &mut Game) {
    unsafe {
        let v1 = RomPtrWithBank(db, k, game) as *const u16;
        DrawPauseScreenSpriteAnim(*v1.offset(2), *v1, *v1.offset(1), game);
        let v2 = RomPtrWithBank(db, k, game);
        if (word!(game.joypad1_lastkeys) & *(v2.offset(6) as *mut u16)) != 0
            && word!(game.map_scrolling_direction) == 0
        {
            word!(game.map_scrolling_direction = *(v2.offset(8) as *mut u16));
        }
    }
}
pub unsafe fn HandleMapScrollArrows(game: &mut Game) {
    unsafe {
        if word!(game.map_min_x_scroll) as i32 - 24i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000
            != 0
        {
            DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9A0 as u16, game);
        }
        if word!(game.map_max_x_scroll) as i32 - 232i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000
            == 0
        {
            DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9AA as u16, game);
        }
        if word!(game.map_min_y_scroll) as i32 - 56i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000
            != 0
        {
            DrawMapScrollArrowAndCheckToScroll(0x82, addr_stru_82B9B4 as u16, game);
        }
        if word!(game.map_max_y_scroll) as i32 - 177i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000
            != 0
        {
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
}
pub unsafe fn MapScreenDrawSamusPositionIndicator(game: &mut Game) {
    unsafe {
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
}
static kMap_SamusPositionIndicator_Delays: [u16; 4] = [8, 4, 8, 4];
static kMap_SamusPositionIndicator_SpritemapIds: [u16; 4] = [0x5f, 0x60, 0x61, 0x60];
pub unsafe fn UpdateSamusPositionIndicatorAnimation(game: &mut Game) -> u16 {
    unsafe {
        let mut v0: u16 = word!(game.samus_position_indicator_animation_timer);
        if v0 == 0 {
            let mut v1: u16 =
                (word!(game.samus_position_indicator_animation_frame)).wrapping_add(2);
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
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    DrawBorderAroundSamusData => DrawMenuSpritemap(0x48, 0x80, 0x10, 3584, game);
    DrawBorderAroundDataCopyMode => DrawMenuSpritemap(0x49, 0x80, 0x10, 3584, game);
    DrawBorderAroundDataClearMode => DrawMenuSpritemap(0x4a, 0x7c, 0x10, 3584, game);
}
static kDrawMenuSelectionMissile_Enable: [u16; 4] = [8, 8, 8, 8];
static kDrawMenuSelectionMissile_SpriteMap: [u16; 4] = [0x37, 0x36, 0x35, 0x34];
pub unsafe fn DrawMenuSelectionMissile(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn DrawFileCopyArrow(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        if ((word!(game.eproj_id[16])) as i16).wrapping_sub((word!(game.eproj_id[17])) as i16) >= 0
        {
            v0 = 3;
            if word!(game.eproj_id[16]) as i32 - word!(game.eproj_id[17]) as i32 - 2i32 & 0x8000
                != 0
            {
                v0 = 4;
                if word!(game.eproj_id[16]) != 1 {
                    v0 = 5;
                }
            }
        } else {
            v0 = 0;
            if word!(game.eproj_id[17]) as i32 - word!(game.eproj_id[16]) as i32 - 2i32 & 0x8000
                != 0
            {
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
}
pub unsafe fn DisplayMapElevatorDestinations(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HandleGameOverBabyMetroid(game: &mut Game) {
    unsafe {
        if (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer != 0 {
            sub_82BB7F(
                (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer,
                game,
            );
        } else {
            sub_82BBDD(game);
        };
    }
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
    unsafe {
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
                (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer =
                    *(v2.offset(6) as *mut u16);
                let fresh18 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction;
                fresh18.write_unaligned((fresh18.read_unaligned()).wrapping_add(6));
                DrawBabyMetroid((current_instruction).wrapping_add(6), game);
            } else {
                CallBabyMetroidPlaySfx(
                    ((*(v2.offset(6) as *mut u16)) as u32) | (0x820000u32),
                    game,
                );
            }
        } else {
            DrawBabyMetroid(
                (*(ram_ptr!(game, EnemyData; 0xf78))).current_instruction,
                game,
            );
        };
    }
}
pub unsafe fn sub_82BBDD(game: &mut Game) {
    unsafe {
        enemy_instruction! {[(*(ram_ptr!(game, EnemyData; 0xf78)))]; instruction = addr_kgameOverBabyMetridInstructionList as u16, timer = 10}
        sub_82BB7F(0xa, game);
    }
}
pub unsafe fn DrawBabyMetroid(k: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn FinishProcessingGameOverBabyMetroidAsm(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn CancelSoundEffects(game: &mut Game) {
    QueueSfx1_Max6(2, game);
    QueueSfx2_Max6(0x71, game);
    QueueSfx3_Max6(1, game);
}
pub unsafe fn AdvancePaletteFadeForAllPalettes_0xc(game: &mut Game) -> u8 {
    unsafe {
        word!(game.palette_change_denom = 12);
        AdvancePaletteFadeForAllPalettes(game)
    }
}
pub unsafe fn AdvancePaletteFade_BgPalette6(game: &mut Game) -> u8 {
    unsafe {
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
}
pub unsafe fn AdvanceGradualColorChangeOfPalette(mut k: u16, j: u16, game: &mut Game) -> u8 {
    unsafe {
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
}
pub unsafe fn AdvancePaletteFadeForAllPalettes(game: &mut Game) -> u8 {
    unsafe {
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
                    *(ram_ptr!(game, u16; 0xc000))
                        .offset((word!(game.g_word_7EC404) >> 1) as isize) = v3;
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
}
pub unsafe fn CalculateNthTransitionColorFromXtoY(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    unsafe {
        (CalculateNthTransitionColorComponentFromXtoY(a, (k) & (0x1fu16), (j) & (0x1fu16), game)
            as i32
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
}
pub unsafe fn CalculateNthTransitionColorComponentFromXtoY(
    a: u16,
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    unsafe {
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
}
pub unsafe fn sub_82DAF7(a: u16, game: &mut Game) -> u8 {
    unsafe {
        word!(game.palette_change_denom = 12);
        sub_82DB0C(a, game)
    }
}
pub unsafe fn sub_82DB0C(mut a: u16, game: &mut Game) -> u8 {
    unsafe {
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
}
pub unsafe fn sub_82DB41(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HandleSamusOutOfHealthAndGameTile(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameState_27_ReserveTanksAuto(game: &mut Game) -> CoroutineRet {
    unsafe {
        if *ram_ptr!(game; 0x612) == 0 && RefillHealthFromReserveTanks(game) != 0 {
            word! {game; time_is_frozen_flag = 0; game_state = kGameState_8_MainGameplay as u16; }
            CallSomeSamusCode(0x10, game);
        }
        let mut _coret_: u8 = AdvanceGameplay(game);
        if _coret_ != 0 {
            return _coret_;
        }
        Samus_LowHealthCheck_0(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn RefillHealthFromReserveTanks(game: &mut Game) -> u8 {
    unsafe {
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
}
pub unsafe fn GameState_19_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_20_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = AdvanceGameplay(game);
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
}
pub unsafe fn GameState_21_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    unsafe {
        Samus_DrawWhenNotAnimatingOrDying(game);
        if HasQueuedMusic(game) == 0 {
            StartSamusDeathAnimation(game);
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_22_SamusNoHealth_Dying(game: &mut Game) -> CoroutineRet {
    unsafe {
        DrawSamusStartingDeathAnim_(game);
        let v0: bool = (word!(game.g_word_7E0DE8 -= 1, reread) & 0x8000) != 0;
        if word!(game.g_word_7E0DE8) == 0 || v0 {
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_23_SamusNoHealth_Flashing(game: &mut Game) -> CoroutineRet {
    unsafe {
        if HandleSamusDeathSequence(game) != 0 {
            word!(game.game_state += 1);
        } else {
            Samus_DrawDuringDeathAnim(game);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion(game: &mut Game) -> CoroutineRet {
    unsafe {
        if GameState_24_SamusNoHealth_Explosion_Helper(game) != 0 {
            word! { game, 1; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_25_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    unsafe {
        HandleFadeOut(game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            EnableNMI(game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
            word! { game, 0; menu_index, debug_disable_sounds }
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn WaitUntilEndOfVblankAndEnableIrq(game: &mut Game) {
    unsafe {
        if (*ram_ptr!(game; 0x84) & 0x30) != 0x30 {
            EnableIrqInterruptsNow(game);
        }
    }
}
pub unsafe fn EnsureSamusDrawnEachFrame(game: &mut Game) {
    unsafe {
        if word!(game.elevator_flags) == 0 {
            Samus_DrawWhenNotAnimatingOrDying(game);
        }
    }
}
pub unsafe fn CallDoorTransitionFunction_Async(ea: u32, game: &mut Game) -> CoroutineRet {
    let poll = crate::game::room_entry::protocol::dispatch(
        &mut crate::game::session_runtime::Runtime::new(game),
        ea,
    );
    poll.returned_byte()
}
pub unsafe fn GameState_9_HitDoorBlock(game: &mut Game) -> CoroutineRet {
    let poll = crate::game::room_entry::protocol::resume(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::room_entry::EntrySignal::DoorState9,
    );
    poll.returned_byte()
}
pub unsafe fn GameState_10_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    let poll = crate::game::room_entry::protocol::resume(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::room_entry::EntrySignal::DoorState10,
    );
    poll.returned_byte()
}
pub unsafe fn GameState_11_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    let poll = crate::game::room_entry::protocol::resume(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::room_entry::EntrySignal::DoorState11,
    );
    poll.returned_byte()
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
    unsafe {
        kGameOptionsMenuFuncs[word!(game.game_options_screen_index) as usize](game);
        OptionsMenuFunc1(game);
        DrawOptionsMenuSpritemaps(game);
        if word!(game.game_options_screen_index) as i32 - 2i32 & 0x8000 == 0 {
            OptionsMenu_AddToVramQueue(game);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameOptionsMenuFunc_0(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameOptionsMenu_1_LoadingOptionsScreen(game: &mut Game) {
    unsafe {
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
        decompress_to_ram(0x978df4, 0x1c000, game);
        decompress_to_ram(0x978fcd, 0x1c800, game);
        decompress_to_ram(0x9791c4, 0x1d000, game);
        decompress_to_ram(0x97938d, 0x1d800, game);
        decompress_to_ram(0x97953a, 0x1e000, game);
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
}
pub unsafe fn GameOptionsMenu_2_FadeInOptionsScreen(game: &mut Game) {
    unsafe {
        HandleFadeIn(game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_options_screen_index += 1);
        }
    }
}
pub unsafe fn OptionsMenu_AddToVramQueue(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn OptionsMenuFunc5(a: u16, mut k: u16, mut j: u16, game: &mut Game) {
    unsafe {
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
}
static kGameOptionsMenuItemFuncs: [Func_V; 5] = callback_table! { nonnull Func_V;
    GameOptionsMenuItemFunc_0, GameOptionsMenuItemFunc_2_ToggleJapanese,
    GameOptionsMenuItemFunc_2_ToggleJapanese, GameOptionsMenuItemFunc_4,
    GameOptionsMenuItemFunc_4,
};
pub unsafe fn GameOptionsMenu_3_OptionsScreen(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameOptionsMenuItemFunc_0(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameOptionsMenuItemFunc_2_ToggleJapanese(game: &mut Game) {
    unsafe {
        word!(game.menu_option_index = 0);
        word!(game.japanese_text_flag = (word!(game.japanese_text_flag) == 0) as u16);
        OptionsMenuFunc4(game);
    }
}
pub unsafe fn OptionsMenuFunc4(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameOptionsMenuItemFunc_4(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x57) = 3;
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_options_screen_index = 5);
    }
}
pub unsafe fn GameOptionsMenu_B_TransitionBackToFileSelect(game: &mut Game) {
    unsafe {
        HandleFadeOut(game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            EnableNMI(game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state = kGameState_4_FileSelectMenus as u16);
            word! { game, 0; menu_index, game_options_screen_index }
        }
    }
}
pub unsafe fn GameOptionsMenu_C_FadeOutOptionsScreenToStart(game: &mut Game) {
    unsafe {
        HandleFadeOut(game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            EnableNMI(game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_options_screen_index = 4);
        }
    }
}
pub unsafe fn GameOptionsMenu_4_StartGame(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameOptionsMenu_5_DissolveOutScreen(game: &mut Game) {
    unsafe {
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
                            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
                                [k as usize] = v1;
                        }
                    } else {
                        for m in (0u16..=1023).rev() {
                            v1 = *(ram_ptr!(game, u16; 0x19602)).offset((m as i32 + 6399) as isize);
                            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
                                [m as usize] = v1;
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
}
pub unsafe fn GameOptionsMenu_6_DissolveInScreen(game: &mut Game) {
    unsafe {
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
}
static kGameOptionsMenuSpecialSettings: [Func_V; 3] = callback_table! { nonnull Func_V;
    GameOptionsMenuSpecialSettings_0, GameOptionsMenuSpecialSettings_0,
    GameOptionsMenuSpecialSettings_2,
};
pub unsafe fn GameOptionsMenu_8_SpecialSettings(game: &mut Game) {
    unsafe {
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
                & (kButton_Start as i32
                    | kButton_Left as i32
                    | kButton_Right as i32
                    | kButton_A as i32)
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
}
pub unsafe fn GameOptionsMenuSpecialSettings_0(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn GameOptionsMenuSpecialSettings_2(game: &mut Game) {
    unsafe {
        word!(game.menu_option_index = 0);
        GameOptionsMenuItemFunc_4(game);
    }
}
static g_word_82F149: [u16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
static g_word_82F151: [u16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
pub unsafe fn OptionsMenuFunc7(game: &mut Game) {
    unsafe {
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
}
static word_82F204: [u16; 16] = [
    0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80, 0x80,
    0x80,
];
pub unsafe fn GameOptionsMenu_7_ControllerSettings(game: &mut Game) {
    unsafe {
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
}
pub unsafe fn OptionsMenuControllerFunc_8(game: &mut Game) {
    unsafe {
        if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
            word! {game; button_config_shoot_x = 64; button_config_jump_a = 128;
            button_config_run_b = 0x8000; button_config_itemcancel_y = 0x4000;
            button_config_itemswitch = 0x2000; button_config_aim_up_R = 16;
            button_config_aim_down_L = 32; }
            LoadControllerOptionsFromControllerBindings(game);
            OptionsMenuFunc6(game);
        }
    }
}
pub unsafe fn OptionsMenuControllerFunc_7(game: &mut Game) {
    unsafe {
        if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0
            && OptionsMenuFunc8(game) == 0
        {
            word!(game.menu_option_index = 0);
            GameOptionsMenuItemFunc_4(game);
        }
    }
}
pub unsafe fn GameOptionsMenu_9_ScrollControllerSettingsDown(game: &mut Game) {
    unsafe {
        word!(game.reg_BG1VOFS += 2);
        if word!(game.reg_BG1VOFS) == 32 {
            word!(game.game_options_screen_index = 7);
        }
    }
}
pub unsafe fn GameOptionsMenu_A_ScrollControllerSettingsUp(game: &mut Game) {
    unsafe {
        word!(game.reg_BG1VOFS -= 2);
        if word!(game.reg_BG1VOFS) == 0 {
            word!(game.game_options_screen_index = 7);
        }
    }
}
pub use self::sub_82F35B as sub_82F363;
pub unsafe fn sub_82F369(j: u16, game: &mut Game) {
    unsafe {
        let v1: i32 = j as i32 >> 1;
        word! {game; eproj_x_vel[(v1 + 3)] = 16; eproj_x_vel[(v1 + 11)] = 3584; }
    }
}
pub unsafe fn LoadControllerOptionsFromControllerBindings(game: &mut Game) {
    unsafe {
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
    unsafe {
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
}
static g_word_82F6AD: [u16; 6] = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
pub unsafe fn OptionsMenuFunc6(game: &mut Game) {
    unsafe {
        for v0 in (0u16..14).step_by(2) {
            let v1: i32 = v0 as i32 >> 1;
            let v2: u16 = crate::rtl::rom_table_word!(game, 0x82f639, v1 as isize);
            let address = 0x820000
                | u32::from(crate::rtl::rom_table_word!(
                    game,
                    0x82f647,
                    word!(game.eproj_F[(v1 + 13)]) as isize
                ));
            if address & 0x8000 == 0 {
                // Keep this diagnostic on the existing C stream and its buffer.
                printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
                game.g_fail = true;
            }
            let source = crate::word_transfers::rom_source(address, game);
            word_transfers!(game, ascending, 3;
                native(0x3000 + v2 as isize) <- captured(source, 0),
            );
            word_transfers!(game, ascending, 3;
                native(0x3040 + v2 as isize) <- captured(source, 3),
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
            *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
                .offset(46) as *mut u32) =
                *((&raw const g_word_82F6AD as *const u16).offset(3) as *mut u32);
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[793i32 as usize] =
                g_word_82F6AD[5i32 as usize];
        }
    }
}
pub unsafe fn OptionsMenuControllerFunc_0(game: &mut Game) {
    unsafe {
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
}

pub unsafe fn ClearSoundsWhenGoingThroughDoor(game: &mut Game) {
    unsafe {
        CallSomeSamusCode(0x1d, game);
    }
}
pub unsafe fn PlaySpinJumpSoundIfSpinJumping(game: &mut Game) {
    unsafe {
        CallSomeSamusCode(0x1c, game);
    }
}
