use crate::game::decompression::decompress_to_ram;
use crate::game::demo_rooms::{self, reset as reset_attract};
use crate::game::file_menu::advance as advance_file_menu;
use crate::game::game_over::advance as advance_game_over;
use crate::game::gameplay::{Entry, advance as AdvanceGameplay};
use crate::game::map_menu::advance as advance_map_menu;
use crate::game::music::{HasQueuedMusic, QueueMusic_Delayed8, QueueMusic_DelayedY};
use crate::game::music::{advance_sound_effects, cancel_effects};
use crate::game::options_settings;
use crate::game::pause_controls::fade_in as pause_fade_in;
pub use crate::game::prelude::*;
use crate::game::save_slots;
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled};

use crate::cpu_infra::DebugGameOverMenu;

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;

use crate::game::animated_tiles;
use crate::game::display_refresh::{GraphicsAsset, setup_transfer};
use crate::game::enemy_projectiles::EnableEprojs;
use crate::game::gameplay_flag_runtime::CheckEventHappened;
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

pub use crate::types::Consts_83;

crate::game::constants::typed_constants! { Consts_83;
    addr_kDoorDef_947a = 38010,
}

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db,
    fnDoorTransition_WaitForMusicToClear = 0x82e664,
    fnDoorTransition_HandleTransition = 0x82e6a2, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
    fnCinematicFunctionNone = 0x8b9a47, fnCinematicFunctionOpening = 0x8b9b68,
    fnCinematicFunction_Intro_Initial = 0x8ba395, fnCinematicFunctionBlackoutFromCeres = 0x8bc11b,
    fnCinematicFunctionEscapeFromCebes = 0x8bd480,
}
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
crate::game::constants::typed_constants! { u32;
    fnDoorTransitionFunction_HandleElevator = 8577405,
    fnPauseHook_Empty = 8946657,
    fnPauseHook_DraygonRoom = 9423094, fnUnpauseHook_DraygonRoom = 9423100,
    fnUnpauseHook_Kraid_IsDead = 10994171, fnUnpauseHook_Kraid_IsAlive = 10994254,
    fnKraid_UnpauseHook_IsSinking = 10994336, fnPauseHook_Kraid = 10994469,
    fnMotherBrainsBody_UnpauseHook = 11110243,
}
pub unsafe fn InitAndLoadGameData_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::verify_probe::hit("coroutine.init");
        let resume_state: *mut u8 = ram_ptr!(game; 0x612);
        let mut stage = *resume_state;
        let mut waiting = stage >= 2;
        if stage == 0 {
            if word!(game.game_state) as i32 == kGameState_40_TransitionToDemo as i32 {
                crate::game::display_refresh::initialize_gameplay(game);
                initialize_player(game);
                demo_rooms::load(game);
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
                crate::game::display_refresh::initialize_gameplay(game);
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
                    demo_rooms::finish_setup(game);
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
pub unsafe fn GameState_33_BlackoutFromCeres(game: &mut Game) -> CoroutineRet {
    unsafe {
        if word!(game.timer_status) != 0 {
            crate::game::escape_timer::draw(game);
        }
        let mut _coret_: u8 = GAMEPLAY(game);
        if _coret_ != 0 {
            return _coret_;
        }
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
            crate::game::display_refresh::reset_gameplay_layers(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.loading_game_state = kLoadingGameState_22_EscapingCeres as u16);
            word!(game.game_state = 34);
            save_slots::save(word!(game.selected_save_slot), game);
            word!(game.cinematic_function = (0x8bc11bi32) as u16);
            word! { game, 0; ceres_status, timer_status }
            QueueMusic_Delayed8(0, game);
            cancel_effects(15, game);
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
            crate::game::display_refresh::reset_gameplay_layers(true, game);
            word! { game, 0;
                screen_fade_delay, screen_fade_counter, ceres_status, timer_status,
            }
            cancel_effects(15, game);
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
        let mut _coret_: u8 = GAMEPLAY(game);
        if _coret_ != 0 {
            return _coret_;
        }
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
            crate::game::display_refresh::reset_gameplay_layers(false, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word! {game; game_state = kGameState_39_EndingAndCredits as u16;
            cinematic_function = ((0x8bd480i32) as u16); timer_status = 0; }
            QueueMusic_Delayed8(0, game);
            cancel_effects(15, game);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_41_TransitionToDemo(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = GAMEPLAY(game);
        if _coret_ != 0 {
            return _coret_;
        }
        crate::game::visual_runtime::update_hdma(game);
        word!(game.game_state += 1);
        *ram_ptr!(game; 0x51) = 15;
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_43_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    unsafe {
        if word!(game.substate) != 1 {
            demo_rooms::advance_set(game);
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
const GAMEPLAY: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::Gameplay, game) };
const FADE_IN: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::FadeIn, game) };
const UNPAUSE: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::Unpause, game) };
const CERES_ELEVATOR: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::CeresElevator, game) };
const TIME_UP: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::TimeUp, game) };

const RESERVE_TANKS: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::ReserveTanks, game) };
const PREPARE_DEATH: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::PrepareDeath, game) };

const DEMO: Func_V_Coroutine = |game| demo_rooms::play(game, |game| unsafe { GAMEPLAY(game) });

static kGameStateFuncs: [Func_V_Coroutine; 45] = callback_table! { nonnull Func_V_Coroutine;
    reset_attract, GameState_1_OpeningCinematic_, GameState_2_GameOptionsMenu,
    GameState_3_Null, advance_file_menu, advance_map_menu,
    InitAndLoadGameData_Async, FADE_IN, GAMEPLAY,
    GameState_9_HitDoorBlock, GameState_10_LoadingNextRoom_Async,
    GameState_11_LoadingNextRoom_Async, GameState_12_Pausing_Darkening_Async,
    GameState_13_Pausing_Async, pause_fade_in, GameState_15_Paused_Async,
    GameState_16_Unpausing_Async, GameState_17_Unpausing_Async, UNPAUSE,
    PREPARE_DEATH, GameState_20_SamusNoHealth_BlackOut, GameState_21_SamusNoHealth,
    GameState_22_SamusNoHealth_Dying, GameState_23_SamusNoHealth_Flashing,
    GameState_24_SamusNoHealth_Explosion, GameState_25_SamusNoHealth_BlackOut,
    advance_game_over, RESERVE_TANKS, GameState_28_Unused,
    GameState_29_DebugGameOverMenu, GameState_37_CeresGoesBoomWithSamus_,
    InitAndLoadGameData_Async, CERES_ELEVATOR,
    GameState_33_BlackoutFromCeres, GameState_37_CeresGoesBoomWithSamus_, TIME_UP,
    GameState_36_WhitingOutFromTimeUp, GameState_37_CeresGoesBoomWithSamus_,
    GameState_38_SamusEscapesFromZebes, GameState_39_EndingAndCredits_,
    InitAndLoadGameData_Async, GameState_41_TransitionToDemo, DEMO,
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
pub fn ShowSpareCpu(game: &mut Game) {
    {
        word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
    }
}
pub fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::rtl::Unreachable(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_12_Pausing_Darkening_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = GAMEPLAY(game);
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
        crate::game::display_refresh::capture_pause_background(game);
        CallPauseHook(Load24(ram_ptr!(game, LongPtr; 0x601)), game);
        cancel_effects(6, game);
        crate::game::pause_controls::save_graphics(game);
        crate::game::display_refresh::load_pause_assets(game);
        crate::game::display_refresh::upload_pause_map(game);
        word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0xb6f000, 0));
        crate::game::display_refresh::pause_graphics(true, game);
        crate::game::pause_controls::initialize(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_15_Paused_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::controller_input::filter(3, game);
        // Preserve the eight-entry bounds failure before any tab work.
        let tab = [0, 1, 2, 3, 4, 5, 6, 7][usize::from(word!(game.menu_index))];
        match tab {
            0 => crate::game::pause_controls::update_map(game),
            1 => crate::game::pause_controls::update_equipment(game),
            2 | 5 => crate::game::pause_controls::fade_out_tab(game, tab == 5, |game| {
                crate::game::display_refresh::enable(true, game);
            }),
            3 | 6 => crate::game::pause_controls::load_tab(game, tab == 3, |game| {
                if tab == 3 {
                    GraphicsAsset::VramWords(0x7e3800, 0x3000, 0x800).load(game);
                } else {
                    crate::game::display_refresh::upload_pause_map(game);
                }
            }),
            4 => crate::game::pause_controls::fade_tab::<true>(game),
            _ => crate::game::pause_controls::fade_tab::<false>(game),
        }
        crate::game::hud::advance(game);
        crate::game::map_menu::animate_palette(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_16_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::pause_controls::draw_unpause(game);
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state += 1);
        }
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn GameState_17_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x612);
        let run_unpause_hook: bool;
        match *_state_variable_ as i32 {
            0 => {
                GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1000).load(game);
                word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- pause_map(384));
                crate::game::display_refresh::pause_graphics(false, game);
                crate::game::pause_controls::resume(game);
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
                crate::game::pause_controls::restore_graphics(game);
                GraphicsAsset::VramWords(0x7edf5c, u16::from(game.g_ram[0x59] & 0xfc) << 8, 0x1000)
                    .load(game);
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
pub unsafe fn GameState_20_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _coret_: u8 = GAMEPLAY(game);
        if _coret_ != 0 {
            return _coret_;
        }
        if fading::all(6, game) != 0 {
            clear_hdma(game);
            crate::game::scanline_irq::disable(game);
            crate::game::display_refresh::reset_gameplay_layers(true, game);
            word! { game, 0;
                game_options_screen_index, screen_fade_delay, screen_fade_counter,
            }
            word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
            word! {game; g_word_7E0DE8 = 16; game_options_screen_index = 3; }
            word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
            word!(game.game_state += 1);
            word!(game.power_bomb_explosion_status = 0);
            cancel_effects(15, game);
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

static kGameOptionsMenuFuncs: [Func_V; 13] = [
    GameOptionsMenuFunc_0,
    GameOptionsMenu_1_LoadingOptionsScreen,
    options_settings::fade_in::<false>,
    options_settings::main_menu,
    GameOptionsMenu_4_StartGame,
    GameOptionsMenu_5_DissolveOutScreen,
    options_settings::fade_in::<true>,
    options_settings::controller,
    options_settings::special,
    options_settings::scroll::<true>,
    options_settings::scroll::<false>,
    GameOptionsMenu_B_TransitionBackToFileSelect,
    GameOptionsMenu_C_FadeOutOptionsScreenToStart,
];
pub unsafe fn GameState_2_GameOptionsMenu(game: &mut Game) -> CoroutineRet {
    unsafe {
        kGameOptionsMenuFuncs[word!(game.game_options_screen_index) as usize](game);
        crate::game::options_objects::advance_and_draw(game);
        if word!(game.game_options_screen_index) as i32 - 2i32 & 0x8000 == 0 {
            crate::game::menu_tilemap::queue_options(game);
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
                crate::game::options_objects::spawn(v0, addr_stru_82F4D6 as u16, game);
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
        options_settings::copy_screen(0x1c7fe, false, game);
        // The ROM keeps the final copied tile in A through the object clear.
        let initial_tile = u16::from_le_bytes([game.g_ram[0x3000], game.g_ram[0x3001]]);
        word!(game.menu_option_index = 0);
        crate::game::options_objects::clear(game);
        crate::game::options_objects::spawn(initial_tile, addr_stru_82F4B8 as u16, game);
        // The first initializer leaves its palette $0E00 in A for this call.
        crate::game::options_objects::spawn(0x0e00, addr_stru_82F4C4 as u16, game);
        word!(game.game_options_screen_index += 1);
        options_settings::highlight_language(game);
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
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x57) != 0xf3 {
            crate::update_place! { reference fresh70 = *ram_ptr!(game; 0x57) => (*fresh70).wrapping_add(16); }
        }
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter, reg_BG1VOFS }
            word!(game.game_options_screen_index += 1);
            options_settings::replace_screen(game);
        }
    }
}
