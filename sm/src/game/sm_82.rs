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
use crate::game::visual_runtime::clear_hdma;

use crate::cpu_infra::DebugGameOverMenu;

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;

use crate::game::display_refresh::setup_transfer;
use crate::game::palette_effects::fading;
use crate::game::samus_animation::{
    Samus_DrawDuringDeathAnim, Samus_DrawStartingDeathAnim, Samus_DrawWhenNotAnimatingOrDying,
};
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_,
};
use crate::game::sm_9b::{
    GameState_24_SamusNoHealth_Explosion_Helper, HandleSamusDeathSequence, StartSamusDeathAnimation,
};
use crate::game::sprite_drawing::{clear_high_table, finish_frame};

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
const INITIALIZE: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::Initialize, game) };
const GAMEPLAY: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::Gameplay, game) };
const FADE_IN: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::FadeIn, game) };
const UNPAUSE: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::Unpause, game) };
const CERES_ELEVATOR: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::CeresElevator, game) };
const CERES_BLACKOUT: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::CeresBlackout, game) };
const ZEBES_ESCAPE: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::ZebesEscape, game) };
const DEMO_START: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::DemoStart, game) };
const DEMO_STOP: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::DemoStop, game) };
const DEMO_NEXT: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::DemoNext, game) };
const PAUSE_SETUP: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::PauseSetup, game) };
const PAUSED: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::Paused, game) };
const UNPAUSE_SETUP: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::UnpauseSetup, game) };
const PAUSE_FADE_OUT: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::PauseFadeOut, game) };
const UNPAUSE_FADE_OUT: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::UnpauseFadeOut, game) };
const GAME_OVER_FADE_OUT: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::GameOverFadeOut, game) };
const TIME_UP_EXIT: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::TimeUpExit, game) };
const TIME_UP: Func_V_Coroutine = |game| unsafe { AdvanceGameplay(Entry::TimeUp, game) };

const RESERVE_TANKS: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::ReserveTanks, game) };
const PREPARE_DEATH: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::PrepareDeath, game) };

const DEMO: Func_V_Coroutine = |game| demo_rooms::play(game, |game| unsafe { GAMEPLAY(game) });

static kGameStateFuncs: [Func_V_Coroutine; 45] = callback_table! { nonnull Func_V_Coroutine;
    reset_attract, GameState_1_OpeningCinematic_, GameState_2_GameOptionsMenu,
    GameState_3_Null, advance_file_menu, advance_map_menu,
    INITIALIZE, FADE_IN, GAMEPLAY,
    GameState_9_HitDoorBlock, GameState_10_LoadingNextRoom_Async,
    GameState_11_LoadingNextRoom_Async, PAUSE_FADE_OUT,
    PAUSE_SETUP, pause_fade_in, PAUSED,
    UNPAUSE_FADE_OUT, UNPAUSE_SETUP, UNPAUSE,
    PREPARE_DEATH, GameState_20_SamusNoHealth_BlackOut, GameState_21_SamusNoHealth,
    GameState_22_SamusNoHealth_Dying, GameState_23_SamusNoHealth_Flashing,
    GameState_24_SamusNoHealth_Explosion, GAME_OVER_FADE_OUT,
    advance_game_over, RESERVE_TANKS, GameState_28_Unused,
    GameState_29_DebugGameOverMenu, GameState_37_CeresGoesBoomWithSamus_,
    INITIALIZE, CERES_ELEVATOR,
    CERES_BLACKOUT, GameState_37_CeresGoesBoomWithSamus_, TIME_UP,
    TIME_UP_EXIT, GameState_37_CeresGoesBoomWithSamus_,
    ZEBES_ESCAPE, GameState_39_EndingAndCredits_,
    INITIALIZE, DEMO_START, DEMO,
    DEMO_STOP, DEMO_NEXT,
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

pub unsafe fn GameState_2_GameOptionsMenu(game: &mut Game) -> CoroutineRet {
    unsafe {
        // Keep the original thirteen-entry bounds failure ahead of all menu effects.
        let screen = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
            [usize::from(word!(game.game_options_screen_index))];
        match screen {
            1 => options_settings::initialize(game, |game| {
                crate::rtl::register_writes! { game;
                    mirror(VMADDL as u16, 0, 0x72, true); byte(VMADDH as u16, 0x58);
                    byte(VMAIN as u16, 0x80);
                }
                setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x8edc00, 0x800, game);
                WriteReg(MDMAEN as u16, 2, game);
            }),
            2 => options_settings::fade_in::<false>(game),
            3 => options_settings::main_menu(game),
            4 => GameOptionsMenu_4_StartGame(game),
            6 => options_settings::fade_in::<true>(game),
            7 => options_settings::controller(game),
            8 => options_settings::special(game),
            9 => options_settings::scroll::<true>(game),
            10 => options_settings::scroll::<false>(game),
            _ => options_settings::fade_out(game, screen, |game| {
                crate::game::display_refresh::enable(true, game);
            }),
        }
        crate::game::options_objects::advance_and_draw(game);
        if word!(game.game_options_screen_index) as i32 - 2i32 & 0x8000 == 0 {
            crate::game::menu_tilemap::queue_options(game);
        }
        kCoroutineNone as CoroutineRet
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
