use crate::game::demo_rooms::{self, reset as reset_attract};
use crate::game::file_menu::advance as advance_file_menu;
use crate::game::game_over::advance as advance_game_over;
use crate::game::gameplay::{Entry, advance as AdvanceGameplay};
use crate::game::map_menu::advance as advance_map_menu;
use crate::game::options_settings;
use crate::game::pause_controls::fade_in as pause_fade_in;
pub use crate::game::prelude::*;
use crate::game::save_slots;

use crate::cpu_infra::DebugGameOverMenu;

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;

use crate::game::display_refresh::setup_transfer;
use crate::game::samus_animation::{advance_death_pose, begin_death, explode_suit, flash_death};
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_,
};

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

const DEATH_BLACKOUT: Func_V_Coroutine =
    |game| unsafe { AdvanceGameplay(Entry::DeathBlackout, game) };

const DEMO: Func_V_Coroutine = |game| demo_rooms::play(game, |game| unsafe { GAMEPLAY(game) });

static kGameStateFuncs: [Func_V_Coroutine; 45] = callback_table! { nonnull Func_V_Coroutine;
    reset_attract, GameState_1_OpeningCinematic_, GameState_2_GameOptionsMenu,
    GameState_3_Null, advance_file_menu, advance_map_menu,
    INITIALIZE, FADE_IN, GAMEPLAY,
    GameState_9_HitDoorBlock, GameState_10_LoadingNextRoom_Async,
    GameState_11_LoadingNextRoom_Async, PAUSE_FADE_OUT,
    PAUSE_SETUP, pause_fade_in, PAUSED,
    UNPAUSE_FADE_OUT, UNPAUSE_SETUP, UNPAUSE,
    PREPARE_DEATH, DEATH_BLACKOUT, begin_death,
    advance_death_pose, flash_death,
    explode_suit, GAME_OVER_FADE_OUT,
    advance_game_over, RESERVE_TANKS, GameState_28_Unused,
    GameState_29_DebugGameOverMenu, GameState_37_CeresGoesBoomWithSamus_,
    INITIALIZE, CERES_ELEVATOR,
    CERES_BLACKOUT, GameState_37_CeresGoesBoomWithSamus_, TIME_UP,
    TIME_UP_EXIT, GameState_37_CeresGoesBoomWithSamus_,
    ZEBES_ESCAPE, GameState_39_EndingAndCredits_,
    INITIALIZE, DEMO_START, DEMO,
    DEMO_STOP, DEMO_NEXT,
};
pub unsafe fn RunOneFrameOfGame(game: &mut Game) {
    unsafe {
        let ret: CoroutineRet = AdvanceGameplay(Entry::Frame(&kGameStateFuncs), game);
        if ret == 0 {
            game.g_ram[0x5b4] = 1;
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
pub fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
    kCoroutineNone as CoroutineRet
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    crate::rtl::Unreachable(game);
    kCoroutineNone as CoroutineRet
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
