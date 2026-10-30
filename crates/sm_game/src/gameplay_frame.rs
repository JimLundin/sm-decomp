//! Frame dispatch, ordered gameplay phases and their resumable continuations.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
use crate::actors;
use crate::animated_tiles::execution as animated_tiles;
use crate::constants::game_states::*;
use crate::constants::loading_game_states::*;
use crate::demo_rooms;
use crate::display::registers::{HDMAEN, MDMAEN, VMADDH, VMADDL, VMAIN};
use crate::display::sequence::{register_writes, write_register_and_mirror};
use crate::display_refresh;
use crate::enemy_projectiles;
use crate::gameplay_rules::{finish_frame, finish_health_and_time};
use crate::menus::prelude::CoroutineRet;
use crate::menus::{
    controller_input, file_menu, game_over, map_menu, options_settings, pause_controls,
    reserve_tanks, save_slots,
};
use crate::palette_effects::{execution as palette_effects, fading};
use crate::ram::{RawRam, word};
use crate::room_entry::api::{ArrivalRequest, EntrySignal, StartSite};
use crate::room_entry::content::LoadMode;
use crate::room_scrolls::{advance_camera, debug_position, fill_viewport, update_scrolls};
use crate::runtime_access::RuntimeAccess;
use crate::session_access::SessionAccess;
use crate::word_transfers;
use crate::{escape_timer, hud, message_boxes, samus_death, sprite_drawing};

/// State callbacks the compatibility runtime still implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeState {
    OpeningCinematic,
    CeresGoesBoom,
    EndingAndCredits,
}

/// The room hooks run when the pause screen opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseHook {
    Draygon,
    Kraid,
}

/// The room hooks run when the pause screen closes; they may yield.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnpauseHook {
    Draygon,
    KraidDead,
    KraidAlive,
    KraidSinking,
    MotherBrain,
}

/// What the frame phases need beyond the menu, HUD, scroll and Session hosts:
/// the operations the compatibility runtime still implements.
pub trait Host:
    crate::menus::host::Host
    + crate::hud::HudHost
    + crate::gameplay_rules::Host
    + crate::display_refresh::Host
    + crate::samus_death::Host
    + RuntimeAccess
{
    /// Admit the owned Session for this frame; `fresh` when no pass is resuming.
    fn begin_session_interval(&mut self, fresh: bool);
    /// The gameplay pass resume byte ($7E:0612).
    fn resume_point(&self) -> u8;
    fn set_resume_point(&mut self, point: u8);

    // Frame phase.
    fn resume_boot(&mut self) -> u8;
    /// The original's failure on an unknown frame state byte.
    fn fail_frame_state(&mut self);
    /// The original's warning when a pass leaves continuation bytes set.
    fn report_broken_coroutine(&mut self, state: [u8; 4]);
    fn update_hdma(&mut self);
    fn advance_random(&mut self);
    fn advance_native_state(&mut self, state: NativeState) -> u8;
    fn resume_room_entry(&mut self, signal: EntrySignal) -> u8;
    fn debug_game_over_menu(&mut self);
    /// Emit this frame's queued gameplay sound effects.
    fn advance_sound_effects(&mut self);

    // Room transition phase.
    fn initialize_player(&mut self);

    // Demo phase.
    fn load_title_sequence_graphics(&mut self);
    fn disable_scanline_irq(&mut self);
    fn enable_scanline_irq(&mut self, immediate: bool);

    // Pause phase.
    fn pause_hook(&mut self, hook: PauseHook);
    fn unpause_hook(&mut self, hook: UnpauseHook) -> u8;

    // Death phase.
    fn queue_music_delayed(&mut self, track: u16, delay: u16);
    fn advance_low_health_warning(&mut self);

    // Gameplay pass.
    fn player_projectiles_hit_samus(&mut self);
    fn advance_player(&mut self);
    fn run_room_mechanisms(&mut self) -> u8;
    fn advance_room(&mut self);
}

#[derive(Clone, Copy)]
enum StateEntry {
    Advance(Entry),
    Native(NativeState),
    DemoReset,
    FileMenu,
    MapMenu,
    PauseFadeIn,
    GameOver,
    BeginDeath,
    DeathPose,
    FlashDeath,
    ExplodeSuit,
    Door(EntrySignal),
    Demo,
    Options,
    Null,
    Unused,
    Debug,
}
use StateEntry::{
    Advance, BeginDeath, DeathPose, Debug, Demo, DemoReset, Door, ExplodeSuit, FileMenu,
    FlashDeath, GameOver, MapMenu, Native, Null, Options, PauseFadeIn, Unused,
};

static GAME_STATE_ENTRIES: [StateEntry; 45] = [
    DemoReset,
    Native(NativeState::OpeningCinematic),
    Options,
    Null,
    FileMenu,
    MapMenu,
    Advance(Entry::Initialize),
    Advance(Entry::FadeIn),
    Advance(Entry::Gameplay),
    Door(EntrySignal::DoorState9),
    Door(EntrySignal::DoorState10),
    Door(EntrySignal::DoorState11),
    Advance(Entry::PauseFadeOut),
    Advance(Entry::PauseSetup),
    PauseFadeIn,
    Advance(Entry::Paused),
    Advance(Entry::UnpauseFadeOut),
    Advance(Entry::UnpauseSetup),
    Advance(Entry::Unpause),
    Advance(Entry::PrepareDeath),
    Advance(Entry::DeathBlackout),
    BeginDeath,
    DeathPose,
    FlashDeath,
    ExplodeSuit,
    Advance(Entry::GameOverFadeOut),
    GameOver,
    Advance(Entry::ReserveTanks),
    Unused,
    Debug,
    Native(NativeState::CeresGoesBoom),
    Advance(Entry::Initialize),
    Advance(Entry::CeresElevator),
    Advance(Entry::CeresBlackout),
    Native(NativeState::CeresGoesBoom),
    Advance(Entry::TimeUp),
    Advance(Entry::TimeUpExit),
    Native(NativeState::CeresGoesBoom),
    Advance(Entry::ZebesEscape),
    Native(NativeState::EndingAndCredits),
    Advance(Entry::Initialize),
    Advance(Entry::DemoStart),
    Demo,
    Advance(Entry::DemoStop),
    Advance(Entry::DemoNext),
];

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum ResumePoint {
    Start = 0,
    EnemyMessage = 1,
    RoomObjects = 2,
    // Existing bytes 3..=255 skip this pass and reset to Start on completion.
    Skip,
}

impl ResumePoint {
    fn read<H: Host>(game: &H) -> Self {
        match game.resume_point() {
            0 => Self::Start,
            1 => Self::EnemyMessage,
            2 => Self::RoomObjects,
            _ => Self::Skip,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Entry {
    Frame,
    Initialize,
    Gameplay,
    FadeIn,
    Unpause,
    DemoStart,
    DemoStop,
    DemoNext,
    PauseSetup,
    Paused,
    UnpauseSetup,
    PauseFadeOut,
    UnpauseFadeOut,
    GameOverFadeOut,
    CeresElevator,
    CeresBlackout,
    ZebesEscape,
    TimeUp,
    TimeUpExit,
    ReserveTanks,
    PrepareDeath,
    DeathBlackout,
}

/// Advances through completion or a suspension, returning the callee's exact
/// yield byte. Nested PLM/message continuations remain owned by those modules.
/// Entry stays fixed even when callbacks change the live game state.
pub fn advance<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    match entry {
        Entry::Frame => frame(game),
        Entry::Initialize => initialize(game),
        Entry::DemoStop => stop_demo(game),
        Entry::DemoNext => next_demo(game),
        Entry::PauseSetup => setup_pause(game),
        Entry::Paused => advance_pause(game),
        Entry::UnpauseSetup => setup_unpause(game),
        Entry::UnpauseFadeOut | Entry::GameOverFadeOut => fade_out(entry, game),
        Entry::TimeUpExit => exit_time_up(game),
        Entry::Gameplay
        | Entry::FadeIn
        | Entry::Unpause
        | Entry::DemoStart
        | Entry::PauseFadeOut
        | Entry::CeresElevator
        | Entry::CeresBlackout
        | Entry::ZebesEscape
        | Entry::TimeUp
        | Entry::ReserveTanks
        | Entry::PrepareDeath
        | Entry::DeathBlackout => pass(entry, game),
    }
}

/// Frame phase: boot continuation, frame start, state dispatch and frame end.
fn frame<H: Host>(game: &mut H) -> CoroutineRet {
    game.begin_session_interval(game.resume_point() == 0);
    let returned = 'frame: {
        let mut st: i32 = game.ram().read_byte(0x611) as i32;
        let begin_frame: bool = if (1..=3).contains(&st) {
            let returned = game.resume_boot();
            if returned != 0 {
                break 'frame returned;
            }
            true
        } else {
            st < 10 && {
                if st != 0 {
                    game.fail_frame_state();
                }
                true
            }
        };
        if begin_frame {
            for offset in (0x612..0x616).rev() {
                game.ram().write_byte(offset, 0);
            }
            controller_input::sample(game);
            game.update_hdma();
            game.advance_random();
            sprite_drawing::clear_high_table(game);
            word! { game, 0;
                OAM_NEXT_PTR, NMI_COPY_SAMUS_HALVES, NMI_COPY_SAMUS_TOP_HALF_SRC,
                NMI_COPY_SAMUS_BOTTOM_HALF_SRC,
            }
            st = word!(game.GAME_STATE) as i32 + 10;
            game.ram().write_byte(0x611, st as u8);
        }
        let returned = match GAME_STATE_ENTRIES[usize::from((st - 10i32) as u16)] {
            Advance(entry) => advance(entry, game),
            Native(state) => game.advance_native_state(state),
            DemoReset => demo_rooms::reset(game),
            FileMenu => file_menu::advance(game),
            MapMenu => map_menu::advance(game),
            PauseFadeIn => pause_controls::fade_in(game),
            GameOver => game_over::advance(game),
            BeginDeath => samus_death::begin_death(game),
            DeathPose => samus_death::advance_death_pose(game),
            FlashDeath => samus_death::flash_death(game),
            ExplodeSuit => samus_death::explode_suit(game),
            Door(signal) => game.resume_room_entry(signal),
            Demo => demo_rooms::play(game, |game| advance(Entry::Gameplay, game)),
            Null => 0,
            Unused => {
                game.rtl_unreachable();
                0
            }
            Debug => {
                game.debug_game_over_menu();
                0
            }
            Options => options_settings::advance(game, upload_options_tiles, |game| {
                game.enable_display_nmi(true)
            }),
        };
        if returned != 0 {
            break 'frame returned;
        }
        game.advance_sound_effects();
        sprite_drawing::finish_frame(game);
        word!(game.JOYPAD1_INPUT_SAMUSFILTER = word!(game.JOYPAD1_LASTKEYS));
        let state = [0x612, 0x613, 0x614, 0x615].map(|at| game.ram().read_byte(at));
        if state.iter().fold(0, |bits, byte| bits | byte) != 0 {
            game.report_broken_coroutine(state);
        }
        game.ram().write_byte(0x611, 0);
        0
    };
    if returned == 0 {
        game.ram().write_byte(0x5b4, 1);
    }
    display_refresh::run(game);
    returned
}

/// Room transition phase: initial room load and actor tile staging.
fn initialize<H: Host>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("coroutine.init");
    let mut stage = game.ram().read_byte(0x612);
    let mut waiting = stage >= 2;
    if stage == 0 {
        let demo = u32::from(word!(game.GAME_STATE)) == K_GAME_STATE_40_TRANSITION_TO_DEMO;
        if !demo {
            let loading = u32::from(word!(game.LOADING_GAME_STATE));
            if loading == K_LOADING_GAME_STATE_1F_STARTING_AT_CERES {
                game.runtime().session_mut().room.area = 6;
                word!(game.LOAD_STATION_INDEX = 0);
                escape_timer::clear(game);
            } else if loading == K_LOADING_GAME_STATE_22_ESCAPING_CERES {
                game.runtime().session_mut().room.area = 0;
                word!(game.LOAD_STATION_INDEX = 18);
                save_slots::load_area_map(game);
            }
        }
        display_refresh::initialize_gameplay(game);
        game.initialize_player();
        if demo {
            demo_rooms::load(game);
        } else {
            crate::room_entry::selection::begin(
                &mut game.runtime(),
                ArrivalRequest::Start(StartSite::CurrentLoadStation),
            );
        }
    }
    if stage <= 1 {
        let yielded = game.resume_room_entry(EntrySignal::Initial);
        if yielded != 0 {
            game.ram().write_byte(0x612, 1);
            return yielded;
        }
        hud::initialize(game);
        word_transfers!(game, ascending, 16; native(0xc380) <- native(0xc180));
        word! { game, 1; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
        game.enable_display_nmi(true);
        enemy_projectiles::lifecycle::set_enabled(&mut game.runtime(), true);
        set_mechanisms_enabled(game, true);
        palette_effects::set_enabled(game.runtime().session_mut(), true);
        game.set_hdma_enabled(true);
        animated_tiles::set_enabled(game.runtime().session_mut(), true);
        crate::player_movement::speed::set_liquid_physics(&mut game.runtime());
        stage = if u32::from(word!(game.GAME_STATE)) == K_GAME_STATE_40_TRANSITION_TO_DEMO {
            2
        } else if u32::from(word!(game.LOADING_GAME_STATE))
            == K_LOADING_GAME_STATE_22_ESCAPING_CERES
        {
            game.queue_music_delayed8(5);
            3
        } else {
            4
        };
        word!(game.LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM = if stage == 3 { 15 } else { 6 });
    }
    if matches!(stage, 2..=4) {
        crate::probe::hit(match stage {
            2 => "coroutine.init.demo",
            3 => "coroutine.init.escape",
            _ => "coroutine.init.normal",
        });
        loop {
            if !waiting {
                actors::staging::transfer_actor_tiles_and_initialize(&mut game.runtime());
            }
            let yielded = game.display_wait(true);
            if yielded != 0 {
                game.ram().write_byte(0x612, stage);
                return yielded;
            }
            word!(game.LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM -= 1);
            if (word!(game.LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM) & 0x8000) != 0 {
                break;
            }
            waiting = false;
        }
        if stage == 2 {
            demo_rooms::finish_setup(game);
        }
        if stage == 4 {
            word!(game.GAME_STATE = K_GAME_STATE_7_MAIN_GAMEPLAY_FADE_IN as u16);
        } else {
            word!(game.GAME_STATE += 1);
        }
        word_transfers!(game, ascending, 256; native(0xc000) <- le(TARGET_PALETTES, 0));
        if stage == 4 {
            if u32::from(word!(game.LOADING_GAME_STATE))
                == K_LOADING_GAME_STATE_1F_STARTING_AT_CERES
            {
                word!(game.PALETTE_BUFFER[223] = 0);
                game.player_command(8);
            } else {
                game.player_command(9);
            }
        }
    }
    game.ram().write_byte(0x612, 0);
    0
}

/// Demo phase: leave a finished demo.
fn stop_demo<H: Host>(game: &mut H) -> CoroutineRet {
    if word!(game.SUBSTATE) != 1 {
        demo_rooms::advance_set(game);
    }
    game.enable_display_nmi(true);
    word!(game.GAME_STATE += 1);
    word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
    game.clear_hdma();
    game.disable_scanline_irq();
    word! { game, 0;
        FX_LAYER_BLENDING_CONFIG_A, CUR_IRQ_HANDLER, IRQHANDLER_NEXT_HANDLER,
    }
    palette_effects::set_enabled(game.runtime().session_mut(), false);
    palette_effects::clear(game.runtime().session_mut());
    word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
    word_transfers!(game, descending, 270; native(0x996) <- value(0));
    display_refresh::reset_layer_mirrors(game);
    0
}

/// Demo phase: choose the next demo or title step.
fn next_demo<H: Host>(game: &mut H) -> CoroutineRet {
    word!(game.GAME_STATE = K_GAME_STATE_1_OPENING_CINEMATIC as u16);
    if (word!(game.SUBSTATE) & 0x8000) != 0 {
        word!(game.GAME_STATE = K_GAME_STATE_40_TRANSITION_TO_DEMO as u16);
    } else if word!(game.SUBSTATE) != 0 {
        game.load_title_sequence_graphics();
        word!(game.EPROJ_X_POS[4] = 2);
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            (0x8b9a47i32) as u16,
        );
    } else {
        game.queue_music_delayed8(0);
        word!(game.DEBUG_DISABLE_SOUNDS = 0);
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            (0x8b9b68i32) as u16,
        );
    }
    0
}

/// Pause phase: enter the pause screen.
fn setup_pause<H: Host>(game: &mut H) -> CoroutineRet {
    game.set_hdma_enabled(false);
    write_register_and_mirror(game, HDMAEN as u16, 0, 0x85, true);
    animated_tiles::set_enabled(game.runtime().session_mut(), false);
    display_refresh::capture_pause_background(game);
    match pause_controls::hook_address(true, game) {
        8946657 => {}
        9423094 => game.pause_hook(PauseHook::Draygon),
        10994469 => game.pause_hook(PauseHook::Kraid),
        _ => {
            game.rtl_unreachable();
        }
    }
    cancel_sound_effects(game, 6);
    pause_controls::save_graphics(game);
    display_refresh::load_pause_assets(game);
    display_refresh::upload_pause_map(game);
    word_transfers!(game, ascending, 256; pause_map(384) <- le(PALETTE_BUFFER, 0));
    word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(0xb6f000, 0));
    display_refresh::pause_graphics(true, game);
    pause_controls::initialize(game);
    0
}

/// Pause phase: one pause menu frame.
fn advance_pause<H: Host>(game: &mut H) -> CoroutineRet {
    controller_input::filter(3, game);
    // Preserve the eight-entry bounds failure before any tab work.
    let tab = [0, 1, 2, 3, 4, 5, 6, 7][usize::from(crate::menu!(game.MenuIndex))];
    match tab {
        0 => pause_controls::update_map(game),
        1 => pause_controls::update_equipment(game),
        2 | 5 => pause_controls::fade_out_tab(game, tab == 5, |game| {
            game.enable_display_nmi(true);
        }),
        3 | 6 => pause_controls::load_tab(game, tab == 3, |game| {
            if tab == 3 {
                game.load_vram_words(0x7e3800, 0x3000, 0x800);
            } else {
                display_refresh::upload_pause_map(game);
            }
        }),
        4 => pause_controls::fade_tab::<true, _>(game),
        _ => pause_controls::fade_tab::<false, _>(game),
    }
    hud::advance(game);
    map_menu::animate_palette(game);
    0
}

/// Pause phase: restore the room after the pause screen.
fn setup_unpause<H: Host>(game: &mut H) -> CoroutineRet {
    let resume = game.ram().read_byte(0x612);
    if resume == 0 {
        game.load_vram_words(0x9ad200, 0x6000, 0x1000);
        word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- pause_map(384));
        display_refresh::pause_graphics(false, game);
        pause_controls::resume(game);
        game.write_register_word(MDMAEN as u16, 0);
        game.enable_display_nmi(false);
        game.disable_scanline_irq();
        crate::room_entry::installation::load_tiles_and_palette(&mut game.runtime());
        crate::room_entry::background::load(&mut game.runtime(), LoadMode::Initial);
        fill_viewport(&mut game.runtime());
        crate::room_mechanisms::graphics::reload_graphics(&mut game.runtime());
        game.enable_display_nmi(true);
        game.enable_scanline_irq(false);
        pause_controls::restore_graphics(game);
        game.load_vram_words(
            0x7edf5c,
            u16::from(game.ram().read_byte(0x59) & 0xfc) << 8,
            0x1000,
        );
        word! { game, 1; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
    }
    if resume <= 1 {
        let yielded = match pause_controls::hook_address(false, game) {
            8946657 => 0,
            9423100 => game.unpause_hook(UnpauseHook::Draygon),
            10994171 => game.unpause_hook(UnpauseHook::KraidDead),
            10994254 => game.unpause_hook(UnpauseHook::KraidAlive),
            10994336 => game.unpause_hook(UnpauseHook::KraidSinking),
            11110243 => game.unpause_hook(UnpauseHook::MotherBrain),
            _ => {
                game.rtl_unreachable();
                0
            }
        };
        if yielded != 0 {
            game.ram().write_byte(0x612, 1);
            return yielded;
        }
        game.set_hdma_enabled(true);
        animated_tiles::set_enabled(game.runtime().session_mut(), true);
        game.restore_movement_sound();
        word!(game.GAME_STATE += 1);
    }
    game.ram().write_byte(0x612, 0);
    0
}

/// Pause and death phases: fade to black before unpausing or game over.
fn fade_out<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    if matches!(entry, Entry::UnpauseFadeOut) {
        pause_controls::draw_unpause(game);
    }
    game.fade(false);
    if game.ram().read_byte(0x51) == 0x80 {
        game.enable_display_nmi(true);
        word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
        word!(game.GAME_STATE += 1);
        if matches!(entry, Entry::GameOverFadeOut) {
            {
                crate::menu!(game.MenuIndex = 0);
                word!(game.DEBUG_DISABLE_SOUNDS = 0);
            }
        }
    }
    0
}

/// Death phase: leave the time-up whiteout.
fn exit_time_up<H: Host>(game: &mut H) -> CoroutineRet {
    game.fade(false);
    if game.ram().read_byte(0x51) == 0x80 {
        game.enable_display_nmi(true);
        finish_gameplay(true, game);
        word! { game, 0;
            SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER, CERES_STATUS, TIMER_STATUS,
        }
        cancel_sound_effects(game, 15);
        if crate::progress::event_happened(&mut game.runtime(), 0xe) {
            {
                word!(game.GAME_OPTIONS_SCREEN_INDEX = 0);
                crate::menu!(game.MenuIndex = 0);
            }
            word_transfers!(game, descending, 128; le(EPROJ_Y_SUBPOS, 142) <- value(0));
            word!(game.GAME_STATE = K_GAME_STATE_25_SAMUS_NO_HEALTH as u16);
        } else {
            word!(game.GAME_STATE = K_GAME_STATE_37_CERES_GOES_BOOM_WITH_SAMUS as u16);
            crate::flow::set(
                game,
                crate::flow::Word::CinematicFunction,
                (0x8bc11bi32) as u16,
            );
        }
    }
    0
}

/// States that run the gameplay pass: their pre-pass work, the pass and,
/// once it completes, their per-state finish.
fn pass<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    if matches!(entry, Entry::CeresElevator | Entry::CeresBlackout) && word!(game.TIMER_STATUS) != 0
    {
        escape_timer::draw(game);
    }
    if matches!(entry, Entry::ReserveTanks)
        && game.ram().read_byte(0x612) == 0
        && reserve_tanks::refill_auto(game) != 0
    {
        word! {game; TIME_IS_FROZEN_FLAG = 0; GAME_STATE = K_GAME_STATE_8_MAIN_GAMEPLAY as u16; }
        game.player_command(0x10);
    }
    let yielded = advance_gameplay(game);
    if yielded != 0 {
        return yielded;
    }
    finish_pass(entry, game);
    0
}

/// Frame phase: the resumable gameplay pass shared by playing states.
fn advance_gameplay<H: Host>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("coroutine.gameplay");
    let resume = ResumePoint::read(game);
    if resume == ResumePoint::Start {
        actors::scheduling::select(&mut game.runtime());
        palette_effects::advance(&mut game.runtime());
        prepare_player_projectiles(game);
        if word!(game.DEBUG_DISABLE_SPRITE_INTERACT) == 0 {
            game.player_projectiles_hit_samus();
        }
        actors::scheduling::advance_selected(&mut game.runtime());
    }
    // Read the queue after enemy callbacks, even when they changed it this pass.
    if resume == ResumePoint::EnemyMessage
        || (resume == ResumePoint::Start && word!(game.QUEUED_MESSAGE_BOX_INDEX) != 0)
    {
        let yielded = message_boxes::display(word!(game.QUEUED_MESSAGE_BOX_INDEX), game);
        if yielded != 0 {
            game.set_resume_point(ResumePoint::EnemyMessage as u8);
            return yielded;
        }
        word!(game.QUEUED_MESSAGE_BOX_INDEX = 0);
    }
    if matches!(resume, ResumePoint::Start | ResumePoint::EnemyMessage) {
        game.advance_player();
        enemy_projectiles::lifecycle::advance(&mut game.runtime());
    }
    if resume != ResumePoint::Skip {
        let yielded = game.run_room_mechanisms();
        if yielded != 0 {
            game.set_resume_point(ResumePoint::RoomObjects as u8);
            return yielded;
        }
        animated_tiles::advance(&mut game.runtime());
        // This is a new read: earlier callbacks may change interaction policy.
        if word!(game.DEBUG_DISABLE_SPRITE_INTERACT) == 0 {
            enemy_projectiles::contact::enemy_projectiles_hit_samus(&mut game.runtime());
            enemy_projectiles::contact::shoot_enemy_projectiles(&mut game.runtime());
            actors::contacts::apply_power_bomb_to_enemies(&mut game.runtime());
        }
        advance_camera(&mut game.runtime());
        let debug_scrolling_enabled: i32 = 0;
        if debug_scrolling_enabled != 0 {
            debug_position(&mut game.runtime());
        }
        actors::drawing::draw_scene(&mut game.runtime());
        {
            // The HUD reads Samus's owned values on the actor-tile lease.
            let mut lease = game.runtime();
            actors::staging::queue_actor_tiles(&mut lease);
            hud::advance(&mut lease);
        }
        update_scrolls(&mut game.runtime());
        game.advance_room();
        finish_health_and_time(game);
        finish_frame(game);
    }
    game.set_resume_point(ResumePoint::Start as u8);
    0
}

/// Prepare Samus's projectiles through the frame handler's entry: normal,
/// demo, projectiles only or inactive.
fn prepare_player_projectiles<H: Host>(game: &mut H) {
    use crate::session::PrepareEntry;
    let at = crate::ram::words::FRAME_HANDLER_ALFA;
    let handler = u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
    let entry = match 0x900000 | u32::from(handler) {
        0x90e695 => PrepareEntry::Normal,
        0x90e6c9 => PrepareEntry::Demo,
        0x90e713 => PrepareEntry::ProjectilesOnly,
        0x90e8cd => PrepareEntry::Inactive,
        _ => {
            crate::player_projectiles::host::Host::unknown(&mut game.runtime());
            return;
        }
    };
    crate::player_projectiles::preparation::prepare(&mut game.runtime(), entry);
}

/// Upload the options screen tiles.
fn upload_options_tiles<H: Host>(game: &mut H) {
    register_writes! { game;
        mirror(VMADDL as u16, 0, 0x72, true); byte(VMADDH as u16, 0x58);
        byte(VMAIN as u16, 0x80);
    }
    crate::display::transfer::setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x8edc00, 0x800, game);
    game.write_register(MDMAEN as u16, 2);
}

/// Clear HDMA and the scanline IRQ, then reset the gameplay layers.
fn finish_gameplay<H: Host>(clear_current_irq: bool, game: &mut H) {
    game.clear_hdma();
    game.disable_scanline_irq();
    display_refresh::reset_gameplay_layers(clear_current_irq, game);
}

/// Cancel the three sound channels, keeping at most `max_queued` queued.
fn cancel_sound_effects<H: Host>(game: &mut H, max_queued: u8) {
    for (channel, sound) in [2, 0x71, 1].into_iter().enumerate() {
        game.queue_sound(channel, sound, max_queued);
    }
}

fn set_mechanisms_enabled<H: Host>(game: &mut H, enabled: bool) {
    let mut h = game.runtime();
    let flags = h.session().mechanisms.flags;
    h.session_mut().mechanisms.flags = if enabled {
        flags | 0x8000
    } else {
        flags & !0x8000
    };
}

/// Per-state work after a completed gameplay pass.
fn finish_pass<H: Host>(entry: Entry, game: &mut H) {
    match entry {
        Entry::FadeIn | Entry::Unpause => {
            game.fade(true);
            if game.ram().read_byte(0x51) == 15 {
                word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
                if matches!(entry, Entry::FadeIn) {
                    word!(game.GAME_STATE += 1);
                } else {
                    word!(game.GAME_STATE = K_GAME_STATE_8_MAIN_GAMEPLAY as u16);
                }
            }
        }
        Entry::CeresElevator => {
            let negative =
                (word!(game.REACHED_CERES_ELEVATOR_FADE_TIMER -= 1, reread) & 0x8000) != 0;
            if word!(game.REACHED_CERES_ELEVATOR_FADE_TIMER) == 0 || negative {
                word!(game.GAME_STATE += 1);
                word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
            }
        }
        Entry::TimeUp | Entry::DeathBlackout => {
            let death = matches!(entry, Entry::DeathBlackout);
            if fading::all(if death { 6 } else { 8 }, game) != 0 {
                if death {
                    finish_gameplay(true, game);
                    word! { game, 0;
                        GAME_OPTIONS_SCREEN_INDEX, SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER,
                    }
                    word_transfers!(game, descending, 128; le(EPROJ_Y_SUBPOS, 142) <- value(0));
                    word! {game; DEATH_FLASH_TIMER = 16; GAME_OPTIONS_SCREEN_INDEX = 3; }
                    word! { game, 0; DEATH_INDEX, DEATH_COUNTER }
                    word!(game.GAME_STATE += 1);
                    word!(game.POWER_BOMB_EXPLOSION_STATUS = 0);
                    cancel_sound_effects(game, 15);
                    game.queue_music_delayed8(0);
                    game.queue_music_delayed8(0xff39);
                    game.queue_music_delayed(5, 0xe);
                } else {
                    word!(game.GAME_STATE = K_GAME_STATE_36_WHITING_OUT_FROM_TIME_UP as u16);
                    word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
                }
            }
        }
        Entry::ReserveTanks => {
            game.advance_low_health_warning();
        }
        Entry::PrepareDeath => {
            word_transfers!(game, descending, 256; pause_map(639) <- le(PALETTE_BUFFER, 255));
            word_transfers!(game, descending, 192; le(TARGET_PALETTES, 191) <- value(0));
            word_transfers!(game, descending, 48; le(TARGET_PALETTES, 255) <- value(0));
            word_transfers!(game, descending, 16; le(TARGET_PALETTES, 207) <- le(PALETTE_BUFFER, 207));
            word!(game.GAME_OPTIONS_SCREEN_INDEX = 3);
            word! { game, 0;
                DEATH_INDEX, DEATH_COUNTER, DEATH_FLASH_TIMER, HUD_ITEM_INDEX,
                SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX, SAMUS_INVINCIBILITY_TIMER, SAMUS_KNOCKBACK_TIMER,
            }
            word!(game.GAME_STATE += 1);
        }
        Entry::CeresBlackout | Entry::ZebesEscape => {
            game.fade(false);
            if game.ram().read_byte(0x51) == 0x80 {
                let ceres = matches!(entry, Entry::CeresBlackout);
                game.enable_display_nmi(true);
                finish_gameplay(ceres, game);
                word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
                if ceres {
                    word!(game.LOADING_GAME_STATE = K_LOADING_GAME_STATE_22_ESCAPING_CERES as u16);
                    word!(game.GAME_STATE = 34);
                    save_slots::save(word!(game.SELECTED_SAVE_SLOT), game);
                    crate::flow::set(
                        game,
                        crate::flow::Word::CinematicFunction,
                        (0x8bc11bi32) as u16,
                    );
                    word! { game, 0; CERES_STATUS, TIMER_STATUS }
                } else {
                    word!(game.GAME_STATE = K_GAME_STATE_39_ENDING_AND_CREDITS as u16);
                    crate::flow::set(
                        game,
                        crate::flow::Word::CinematicFunction,
                        (0x8bd480i32) as u16,
                    );
                    word!(game.TIMER_STATUS = 0);
                }
                game.queue_music_delayed8(0);
                cancel_sound_effects(game, 15);
            }
        }
        Entry::PauseFadeOut => {
            game.fade(false);
            if game.ram().read_byte(0x51) & 0xf == 0 {
                game.enable_display_nmi(true);
                word! { game, 0; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
                word!(game.GAME_STATE += 1);
            }
        }
        Entry::DemoStart => {
            game.update_hdma();
            word!(game.GAME_STATE += 1);
            game.ram().write_byte(0x51, 15);
        }
        Entry::Frame
        | Entry::DemoStop
        | Entry::DemoNext
        | Entry::PauseSetup
        | Entry::Paused
        | Entry::UnpauseSetup
        | Entry::TimeUpExit
        | Entry::UnpauseFadeOut
        | Entry::GameOverFadeOut
        | Entry::Initialize
        | Entry::Gameplay => {}
    }
}
