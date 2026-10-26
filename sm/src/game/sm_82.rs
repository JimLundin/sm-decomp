use crate::game::controller_input::remap_binding;
use crate::game::decompression::decompress_to_ram;
use crate::game::gameplay::advance as AdvanceGameplay;
use crate::game::gameplay_flag_runtime;
use crate::game::music::{CancelSoundEffects, advance_sound_effects};
use crate::game::music::{HasQueuedMusic, QueueMusic_Delayed8, QueueMusic_DelayedY};
pub use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::save_slots;
use crate::game::sprite_drawing::Clip;
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled};

use crate::cpu_infra::DebugGameOverMenu;

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;

use crate::game::animated_tiles;
use crate::game::display_refresh::{GraphicsAsset, setup_transfer};
use crate::game::enemy_projectiles::EnableEprojs;
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::music::{QueueSfx1_Max6, QueueSfx1_Max15, QueueSfx2_Max15, QueueSfx3_Max15};
use crate::game::palette_effects::{self, fading};
use crate::game::room_entry::{self, ArrivalRequest, EntrySignal, StartSite};
use crate::game::room_mechanisms::set_enabled;
use crate::game::room_state::{pause_draygon, unpause_draygon};
use crate::game::samus_animation::{
    Samus_DrawDuringDeathAnim, Samus_DrawStartingDeathAnim, Samus_DrawWhenNotAnimatingOrDying,
};
use crate::game::save_slots::load_area_map;
use crate::game::session_runtime::initialize_player;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_, LoadTitleSequenceGraphics,
};
use crate::game::sm_9b::{
    GameState_24_SamusNoHealth_Explosion_Helper, HandleSamusDeathSequence, StartSamusDeathAnimation,
};
use crate::game::sm_a7::{
    Kraid_UnpauseHook_IsSinking, PauseHook_Kraid, UnpauseHook_Kraid_IsAlive,
    UnpauseHook_Kraid_IsDead,
};
use crate::game::sm_a9::MotherBrainsBody_UnpauseHook;
use crate::game::sprite_drawing::{clear_high_table, finish_frame};
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
                initialize_player(game);
                LoadDemoRoomData(game);
            } else {
                if word!(game.loading_game_state) as i32 != kLoadingGameState_5_Main as i32 {
                    if word!(game.loading_game_state) as i32
                        == kLoadingGameState_1F_StartingAtCeres as i32
                    {
                        word! {game; area_index = 6; load_station_index = 0; }
                        crate::game::escape_timer::clear(game);
                    } else if word!(game.loading_game_state) as i32
                        == kLoadingGameState_22_EscapingCeres as i32
                    {
                        word! {game; area_index = 0; load_station_index = 18; }
                        load_area_map(game);
                    }
                }
                InitIoForGameplay(game);
                LoadStdBG3andSpriteTilesClearTilemaps(game);
                LoadInitialPalette(game);
                initialize_player(game);
                room_entry::begin(
                    &mut Runtime::new(game),
                    ArrivalRequest::Start(StartSite::CurrentLoadStation),
                );
            }
        }
        if stage <= 1 {
            let yielded: u8 =
                room_entry::protocol::resume(&mut Runtime::new(game), EntrySignal::Initial)
                    .returned_byte();
            if yielded != 0 {
                *resume_state = 1;
                return yielded;
            }
            crate::game::hud::initialize(game);
            word_transfers!(game, ascending, 16; native(0xc380) <- native(0xc180));
            word! { game, 1; screen_fade_delay, screen_fade_counter }
            crate::game::display_refresh::enable(true, game);
            EnableEprojs(game);
            set_enabled(true, game);
            palette_effects::set_enabled(true, game);
            set_hdma_enabled(true, game);
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
                let yielded: u8 = crate::game::display_refresh::wait(true, game);
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
        word_transfers!(game, ascending, 1023; native(0x3000) <- value(0));
        word_transfers!(game, ascending, 1023; native(0x4000) <- value(0x6f));
        word_transfers!(game, ascending, 127; native(0x4000) <- value(0x2c0f));
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
            crate::game::escape_timer::draw(game);
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
            crate::game::escape_timer::draw(game);
        }
        let mut _coret_: u8 = AdvanceGameplay(game);
        if _coret_ != 0 {
            return _coret_;
        }
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
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
            save_slots::save(word!(game.selected_save_slot), game);
            word!(game.cinematic_function = (0x8bc11bi32) as u16);
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
        if fading::all(8, game) != 0 {
            word!(game.game_state = kGameState_36_WhitingOutFromTimeUp as u16);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_36_WhitingOutFromTimeUp(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
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
                word!(game.cinematic_function = (0x8bc11bi32) as u16);
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
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
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
        crate::game::visual_runtime::update_hdma(game);
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
                    let yielded: u8 = crate::game::display_refresh::wait(true, game);
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
        crate::game::display_refresh::enable(true, game);
        word!(game.game_state += 1);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        clear_hdma(game);
        crate::game::scanline_irq::disable(game);
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
            let mut _coret_: u8 = crate::game::boot::resume(game);
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
            crate::game::controller_input::sample(game);
            crate::game::visual_runtime::update_hdma(game);
            game.next_random();
            clear_high_table(game);
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
        advance_sound_effects(game, |channel, value, game| {
            crate::native_audio::write_sound_port(channel, value, game)
        });
        finish_frame(game);
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
        crate::game::display_refresh::run(game);
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
        crate::game::game_over::advance(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_4_FileSelectMenus(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::file_menu::advance(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_5_FileSelectMap(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::map_menu::advance(game);
        kCoroutineNone as CoroutineRet
    }
}
pub fn ShowSpareCpu(game: &mut Game) {
    {
        word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
    }
}
pub fn GameState_0_Reset_Async(game: &mut Game) -> CoroutineRet {
    {
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
pub fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
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
        crate::game::display_refresh::fade(true, game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::rtl::Unreachable(game);
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
        word!(game.optionsmenu_arr1[(k >> 1)] = (0x828c81i32) as u16);
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
    {
        for i in (0i32..=14).rev().step_by(2) {
            let v1: i32 = i >> 1;
            if word!(game.cinematicbg_arr8[v1]) != 0 {
                let v2: u16 = word!(game.cinematicbg_arr8[v1]);
                sprite_output::paletted(
                    0x82,
                    v2,
                    (
                        word!(game.optionsmenu_arr8[v1]),
                        word!(game.cinematicbg_arr9[v1]),
                    ),
                    word!(game.optionsmenu_arr10[v1]),
                    Clip::Bounded,
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
        crate::game::display_refresh::fade(false, game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
            crate::game::display_refresh::enable(true, game);
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
        8946657 => [()]; 9423094 => pause_draygon; 10994469 => PauseHook_Kraid;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallUnpauseHook_Async(ea: u32, game: &mut Game) -> CoroutineRet => ea;
        call (game); default {
            Unreachable(game);
            kCoroutineNone as CoroutineRet
        };
    {
        8946657 => [kCoroutineNone as CoroutineRet]; 9423100 => unpause_draygon;
        10994171 => UnpauseHook_Kraid_IsDead; 10994254 => UnpauseHook_Kraid_IsAlive;
        10994336 => Kraid_UnpauseHook_IsSinking; 11110243 => MotherBrainsBody_UnpauseHook;
    }
}
pub unsafe fn GameState_13_Pausing_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        set_hdma_enabled(false, game);
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
        crate::game::display_refresh::queue_fx_clear(game);
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
        GraphicsAsset::VramWords(0x7edf5c, (*ram_ptr!(game; 0x59) as u16 & 0xfc) << 8, 0x1000)
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
        crate::game::menu_tilemap::prepare_equipment_frame(game);
    }
}
pub unsafe fn BackupGameplayPalettesAndLoadForPause(game: &mut Game) {
    word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0xb6f000, 0));
}
pub unsafe fn ContinueInitPauseMenu(game: &mut Game) {
    unsafe {
        SetupPpuForPauseMenu(game);
        crate::game::map_menu::reset_animations(game);
        crate::game::menu_tilemap::equipment_icons(game);
        set_pause_label_palettes(0, game);
        UpdatePauseMenuLRStartVramTilemap(game);
        crate::game::map_menu::scroll_limits(game);
        crate::game::map_menu::center_map(0x80, game);
    }
}
pub unsafe fn GameState_14_Paused_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        DrawPauseMenuDuringFadeIn(game);
        crate::game::display_refresh::fade(true, game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_15_Paused_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::controller_input::filter(3, game);
        MainPauseRoutine(game);
        crate::game::hud::advance(game);
        crate::game::map_menu::animate_palette(game);
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
        crate::game::map_menu::scroll_arrows(crate::game::map_menu::MapScreen::Pause, game);
        crate::game::map_menu::scroll(game);
        crate::game::map_menu::draw_player_marker(game);
        crate::game::map_menu::draw_icons(crate::game::map_menu::IconMode::Pause, game);
        crate::game::map_menu::display_elevator_destinations(game);
        word!(game.pause_screen_mode = 0);
    }
}
pub unsafe fn PauseMenu_1_EquipmentScreen(game: &mut Game) {
    unsafe {
        word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
        crate::game::equipment_menu::update(game);
        HandlePauseScreenLR(game);
        HandlePauseScreenStart(game);
        word!(game.pause_screen_mode = 1);
    }
}
pub unsafe fn PauseMenu_2(game: &mut Game) {
    unsafe {
        crate::game::map_menu::display_elevator_destinations(game);
        crate::game::map_menu::draw_player_marker(game);
        crate::game::map_menu::draw_icons(crate::game::map_menu::IconMode::Pause, game);
        HandlePauseMenuLRPressHighlight(game);
        word!(game.pause_screen_mode = 0);
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.menu_index += 1);
        }
    }
}
pub unsafe fn PauseMenu_5(game: &mut Game) {
    unsafe {
        crate::game::map_menu::draw_equipment_selector(game);
        crate::game::menu_tilemap::reserve_tanks(game);
        HandlePauseMenuLRPressHighlight(game);
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.menu_index += 1);
        }
    }
}
pub unsafe fn PauseMenu_3_MapToEquipment_Load(game: &mut Game) {
    unsafe {
        crate::game::map_menu::display_elevator_destinations(game);
        crate::game::reserve_tanks::prepare_menu(game);
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
        crate::game::map_menu::display_elevator_destinations(game);
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
        crate::game::map_menu::draw_player_marker(game);
        crate::game::map_menu::draw_icons(crate::game::map_menu::IconMode::Pause, game);
        crate::game::map_menu::display_elevator_destinations(game);
        word!(game.pause_screen_mode = 0);
        crate::game::display_refresh::fade(true, game);
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
        crate::game::map_menu::draw_equipment_selector(game);
        crate::game::menu_tilemap::reserve_tanks(game);
        word!(game.pause_screen_mode = 1);
        crate::game::display_refresh::fade(true, game);
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
pub unsafe fn GameState_16_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        HighlightPauseScreenButton(game);
        DrawPauseMenuDuringFadeout(game);
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn DrawPauseMenuDuringFadeout(game: &mut Game) {
    unsafe {
        if word!(game.pause_screen_mode) == 1 {
            crate::game::map_menu::draw_equipment_selector(game);
            crate::game::menu_tilemap::reserve_tanks(game);
            HandlePauseMenuLRPressHighlight(game);
        } else {
            crate::game::map_menu::display_elevator_destinations(game);
            crate::game::map_menu::draw_icons(crate::game::map_menu::IconMode::Pause, game);
            crate::game::map_menu::draw_player_marker(game);
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
                WriteRegWord(MDMAEN as u16, 0, game);
                crate::game::display_refresh::enable(false, game);
                crate::game::scanline_irq::disable(game);
                crate::game::room_state::load_tiles_and_palette(game);
                crate::game::room_entry::background::load(
                    &mut crate::game::session_runtime::Runtime::new(game),
                    crate::game::room_entry::content::LoadMode::Initial,
                );
                crate::game::room_scrolls::fill_viewport(game, |game| {
                    crate::game::display_refresh::flush_tilemaps(game)
                });
                crate::game::room_mechanisms::reload_graphics(game);
                crate::game::display_refresh::enable(true, game);
                crate::game::scanline_irq::enable(false, game);
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
            set_hdma_enabled(true, game);
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
        crate::game::display_refresh::fade(true, game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state = kGameState_8_MainGameplay as u16);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn LoadPauseMenuMapTilemapAndAreaLabel(game: &mut Game) {
    unsafe {
        word! {game; reg_BG1HOFS = word!(game.reg_BG4HOFS); reg_BG1VOFS = word!(game.reg_BG4VOFS); }
        crate::rtl::register_writes! { game;
            byte(VMADDL as u16, 0); byte(VMADDH as u16, 0x30);
            byte(VMAIN as u16, 0x80);
        }
        crate::game::map_menu::render_map(crate::game::map_menu::MapScreen::Pause, game);
        setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x7e4000, 0x1000, game);
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
        crate::game::visual_runtime::update_beam_tiles_and_palette(game);
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
pub fn CalculateBgScrolls_Unpause(game: &mut Game) {
    {
        word!(game.reg_BG1HOFS = (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos)));
        word!(game.reg_BG1VOFS = (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos)));
        word!(game.reg_BG2HOFS = (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos)));
        word!(game.reg_BG2VOFS = (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos)));
    }
}
pub fn ClearPauseMenuData(game: &mut Game) {
    {
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
        crate::game::map_menu::animate_pause_sprite(2, 0x18, 0xd0, game);
        crate::game::map_menu::animate_pause_sprite(2, 0xe8, 0xd0, game);
    }
}
pub unsafe fn HandlePauseScreenLrInput(game: &mut Game) {
    {
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
    {
        if word!(game.pausemenu_start_lr_pressed_highlight_timer) != 0 {
            word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
            if word!(game.pausemenu_shoulder_button_highlight) != 0 {
                let v0: i32 = word!(game.pausemenu_shoulder_button_highlight) as i32 - 1i32;
                sprite_output::menu(
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
    {
        if word!(game.pausemenu_start_lr_pressed_highlight_timer) != 0 {
            word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
            sprite_output::menu(0x2b, 0x90, 0xd0, 0, game);
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
pub unsafe fn EquipmentScreenTransferBG1Tilemap(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(0x7e3800, 0x3000, 0x800).load(game);
        word!(game.reg_BG1VOFS = 0);
    }
}
pub fn DrawPauseMenuDuringFadeIn(game: &mut Game) {
    {
        if word!(game.menu_index) != 0 {
            crate::game::map_menu::draw_equipment_selector(game);
            crate::game::menu_tilemap::reserve_tanks(game);
        } else {
            crate::game::map_menu::draw_player_marker(game);
            crate::game::map_menu::draw_icons(crate::game::map_menu::IconMode::Pause, game);
            crate::game::map_menu::display_elevator_destinations(game);
        };
    }
}
crate::forward_calls! { pub fn (game: &mut Game);
    DrawBorderAroundSamusData => sprite_output::menu(0x48, 0x80, 0x10, 3584, game);
    DrawBorderAroundDataCopyMode => sprite_output::menu(0x49, 0x80, 0x10, 3584, game);
    DrawBorderAroundDataClearMode => sprite_output::menu(0x4a, 0x7c, 0x10, 3584, game);
}
pub fn HandleSamusOutOfHealthAndGameTile(game: &mut Game) {
    {
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
        if *ram_ptr!(game; 0x612) == 0 && crate::game::reserve_tanks::refill_auto(game) != 0 {
            word! {game; time_is_frozen_flag = 0; game_state = kGameState_8_MainGameplay as u16; }
            CallSomeSamusCode(0x10, game);
        }
        let mut _coret_: u8 = AdvanceGameplay(game);
        if _coret_ != 0 {
            return _coret_;
        }
        crate::game::session_runtime::player_update_runtime::low_health(game);
        kCoroutineNone as CoroutineRet
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
        if fading::all(6, game) != 0 {
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
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
    {
        Samus_DrawStartingDeathAnim(game);
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
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
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
            crate::game::scanline_irq::enable(true, game);
        }
    }
}
pub unsafe fn EnsureSamusDrawnEachFrame(game: &mut Game) {
    {
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
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
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
        setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x8edc00, 0x800, game);
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
        crate::game::display_refresh::fade(true, game);
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
        do_until! { j == 0 ;
            *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
                as *mut u8)
                .offset(k as i32 as isize) as *mut u16) = a
                | *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
                    as *mut u8)
                    .offset(k as i32 as isize) as *mut u16)
                    & 0xe3ff;
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
pub fn GameOptionsMenuItemFunc_0(game: &mut Game) {
    {
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
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state = kGameState_4_FileSelectMenus as u16);
            word! { game, 0; menu_index, game_options_screen_index }
        }
    }
}
pub unsafe fn GameOptionsMenu_C_FadeOutOptionsScreenToStart(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_options_screen_index = 4);
        }
    }
}
pub unsafe fn GameOptionsMenu_4_StartGame(game: &mut Game) {
    {
        word!(game.game_options_screen_index = 0);
        if word!(game.enable_debug) != 0 && (word!(game.joypad1_lastkeys) & 0x20) == 0 {
            word!(game.game_state = kGameState_5_FileSelectMap as u16);
            if word!(game.loading_game_state) as i32 != kLoadingGameState_5_Main as i32 {
                word!(game.loading_game_state = kGameState_5_FileSelectMap as u16);
                save_slots::save(word!(game.selected_save_slot), game);
            }
        } else if word!(game.loading_game_state) != 0 {
            word!(game.game_state = word!(game.loading_game_state));
            if word!(game.loading_game_state) as i32 == kGameState_34_CeresGoesBoom as i32 {
                word!(game.cinematic_function = (0x8bc11bi32) as u16);
            }
            word! { game, 0; menu_option_index, game_options_screen_index }
        } else {
            word!(game.game_state = kGameState_30_IntroCinematic as u16);
            word!(game.cinematic_function = (0x8ba395i32) as u16);
            word! { game, 0; menu_option_index, screen_fade_delay, screen_fade_counter }
        };
    }
}
pub unsafe fn GameOptionsMenu_5_DissolveOutScreen(game: &mut Game) {
    unsafe {
        let mut v1: u16 = 0;
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x57) != 0xf3 {
            crate::update_place! { reference fresh70 = *ram_ptr!(game; 0x57) => (*fresh70).wrapping_add(16); }
        }
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
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
                    crate::game::controller_input::load_binding_choices(game);
                    crate::game::menu_tilemap::controller_bindings(game);
                }
            } else {
                word_transfers!(game, descending, 1024; pause_map(1023) <- le(custom_background, 6398));
                OptionsMenuFunc4(game);
                // The ROM passes the final language-highlight tile left in A (82:EFD7).
                let tile = u16::from_le_bytes([game.g_ram[0x33b8], game.g_ram[0x33b9]]);
                CreateOptionsMenuObject_(tile, addr_stru_82F4C4 as u16, game);
            }
        }
    }
}
pub unsafe fn GameOptionsMenu_6_DissolveInScreen(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::fade(true, game);
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
                remap_binding, remap_binding, remap_binding,
                remap_binding, remap_binding, remap_binding,
                remap_binding, OptionsMenuControllerFunc_7, OptionsMenuControllerFunc_8,
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
pub fn OptionsMenuControllerFunc_8(game: &mut Game) {
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        crate::game::controller_input::reset_binding_choices(game);
        crate::game::menu_tilemap::controller_bindings(game);
    }
}
pub unsafe fn OptionsMenuControllerFunc_7(game: &mut Game) {
    unsafe {
        if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0
            && crate::game::controller_input::store_binding_choices(game) == 0
        {
            word!(game.menu_option_index = 0);
            GameOptionsMenuItemFunc_4(game);
        }
    }
}
pub fn GameOptionsMenu_9_ScrollControllerSettingsDown(game: &mut Game) {
    {
        word!(game.reg_BG1VOFS += 2);
        if word!(game.reg_BG1VOFS) == 32 {
            word!(game.game_options_screen_index = 7);
        }
    }
}
pub fn GameOptionsMenu_A_ScrollControllerSettingsUp(game: &mut Game) {
    {
        word!(game.reg_BG1VOFS -= 2);
        if word!(game.reg_BG1VOFS) == 0 {
            word!(game.game_options_screen_index = 7);
        }
    }
}
pub use self::sub_82F35B as sub_82F363;
pub fn sub_82F369(j: u16, game: &mut Game) {
    {
        let v1: i32 = j as i32 >> 1;
        word! {game; eproj_x_vel[(v1 + 3)] = 16; eproj_x_vel[(v1 + 11)] = 3584; }
    }
}
