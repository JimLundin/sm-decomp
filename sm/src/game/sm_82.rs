use crate::game::demo_rooms::{self, reset as reset_attract};
use crate::game::file_menu::advance as advance_file_menu;
use crate::game::game_over::advance as advance_game_over;
use crate::game::gameplay::{Entry, advance as AdvanceGameplay};
use crate::game::map_menu::advance as advance_map_menu;
use crate::game::options_settings;
use crate::game::pause_controls::fade_in as pause_fade_in;
pub use crate::game::prelude::*;

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
#[derive(Clone, Copy)]
enum FrameEntry {
    Advance(Entry),
    Callback(Func_V_Coroutine),
}
use FrameEntry::{Advance, Callback};

const DEMO: Func_V_Coroutine = |game| {
    demo_rooms::play(game, |game| unsafe {
        AdvanceGameplay(Entry::Gameplay, game)
    })
};

static GAME_STATE_ENTRIES: [FrameEntry; 45] = callback_table! { FrameEntry;
    {Callback(reset_attract)}, {Callback(GameState_1_OpeningCinematic_)}, {Callback(GameState_2_GameOptionsMenu)},
    {Callback(GameState_3_Null)}, {Callback(advance_file_menu)}, {Callback(advance_map_menu)},
    {Advance(Entry::Initialize)}, {Advance(Entry::FadeIn)}, {Advance(Entry::Gameplay)},
    {Callback(GameState_9_HitDoorBlock)}, {Callback(GameState_10_LoadingNextRoom_Async)},
    {Callback(GameState_11_LoadingNextRoom_Async)}, {Advance(Entry::PauseFadeOut)},
    {Advance(Entry::PauseSetup)}, {Callback(pause_fade_in)}, {Advance(Entry::Paused)},
    {Advance(Entry::UnpauseFadeOut)}, {Advance(Entry::UnpauseSetup)}, {Advance(Entry::Unpause)},
    {Advance(Entry::PrepareDeath)}, {Advance(Entry::DeathBlackout)}, {Callback(begin_death)},
    {Callback(advance_death_pose)}, {Callback(flash_death)},
    {Callback(explode_suit)}, {Advance(Entry::GameOverFadeOut)},
    {Callback(advance_game_over)}, {Advance(Entry::ReserveTanks)}, {Callback(GameState_28_Unused)},
    {Callback(GameState_29_DebugGameOverMenu)}, {Callback(GameState_37_CeresGoesBoomWithSamus_)},
    {Advance(Entry::Initialize)}, {Advance(Entry::CeresElevator)},
    {Advance(Entry::CeresBlackout)}, {Callback(GameState_37_CeresGoesBoomWithSamus_)}, {Advance(Entry::TimeUp)},
    {Advance(Entry::TimeUpExit)}, {Callback(GameState_37_CeresGoesBoomWithSamus_)},
    {Advance(Entry::ZebesEscape)}, {Callback(GameState_39_EndingAndCredits_)},
    {Advance(Entry::Initialize)}, {Advance(Entry::DemoStart)}, {Callback(DEMO)},
    {Advance(Entry::DemoStop)}, {Advance(Entry::DemoNext)},
};
unsafe fn dispatch_game_state(index: u16, game: &mut Game) -> u16 {
    unsafe {
        let returned: CoroutineRet = match GAME_STATE_ENTRIES[usize::from(index)] {
            Advance(entry) => AdvanceGameplay(entry, game),
            Callback(callback) => callback(game),
        };
        u16::from(returned)
    }
}

pub unsafe fn RunOneFrameOfGame(game: &mut Game) {
    unsafe {
        let ret: CoroutineRet = AdvanceGameplay(Entry::Frame(dispatch_game_state), game);
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
            4 => options_settings::finish_start(game),
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
