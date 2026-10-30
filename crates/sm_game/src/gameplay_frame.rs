//! Frame dispatch, ordered gameplay phases and their resumable continuations.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
#![forbid(unsafe_code)]
use crate::actors;
use crate::animated_tiles::execution as animated_tiles;
use crate::constants::game_states::*;
use crate::constants::loading_game_states::*;
use crate::demo_rooms;
use crate::display::registers::{HDMAEN, MDMAEN};
use crate::display::sequence::write_register_and_mirror;
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
use crate::{escape_timer, hud, message_boxes, sprite_drawing};

/// State callbacks the compatibility runtime still implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeState {
    OpeningCinematic,
    CeresGoesBoom,
    EndingAndCredits,
    BeginDeath,
    DeathPose,
    FlashDeath,
    ExplodeSuit,
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
    + crate::room_scrolls::Host
    + crate::gameplay_rules::Host
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
    /// Upload the options screen tiles.
    fn upload_options_tiles(&mut self);
    /// Emit this frame's queued gameplay sound effects.
    fn advance_sound_effects(&mut self);
    fn refresh_display(&mut self);

    // Room transition phase.
    fn initialize_gameplay_display(&mut self);
    fn initialize_player(&mut self);
    fn flush_tilemaps(&mut self);

    // Demo phase.
    fn load_title_sequence_graphics(&mut self);
    fn disable_scanline_irq(&mut self);
    fn enable_scanline_irq(&mut self, immediate: bool);
    fn reset_layer_mirrors(&mut self);

    // Pause phase.
    fn capture_pause_background(&mut self);
    fn pause_hook(&mut self, hook: PauseHook);
    fn unpause_hook(&mut self, hook: UnpauseHook) -> u8;
    fn load_pause_assets(&mut self);
    fn upload_pause_map(&mut self);
    fn pause_graphics(&mut self, paused: bool);

    // Death phase.
    fn finish_gameplay_display(&mut self, clear_current_irq: bool);
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
    Door(EntrySignal),
    Demo,
    Options,
    Null,
    Unused,
    Debug,
}
use StateEntry::{
    Advance, Debug, Demo, DemoReset, Door, FileMenu, GameOver, MapMenu, Native, Null, Options,
    PauseFadeIn, Unused,
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
    Native(NativeState::BeginDeath),
    Native(NativeState::DeathPose),
    Native(NativeState::FlashDeath),
    Native(NativeState::ExplodeSuit),
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
                oam_next_ptr, nmi_copy_samus_halves, nmi_copy_samus_top_half_src,
                nmi_copy_samus_bottom_half_src,
            }
            st = word!(game.game_state) as i32 + 10;
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
            Options => options_settings::advance(game, H::upload_options_tiles, |game| {
                game.enable_display_nmi(true)
            }),
        };
        if returned != 0 {
            break 'frame returned;
        }
        game.advance_sound_effects();
        sprite_drawing::finish_frame(game);
        word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
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
    game.refresh_display();
    returned
}

/// Room transition phase: initial room load and actor tile staging.
fn initialize<H: Host>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("coroutine.init");
    let mut stage = game.ram().read_byte(0x612);
    let mut waiting = stage >= 2;
    if stage == 0 {
        let demo = u32::from(word!(game.game_state)) == kGameState_40_TransitionToDemo;
        if !demo {
            match u32::from(word!(game.loading_game_state)) {
                kLoadingGameState_1F_StartingAtCeres => {
                    word! {game; area_index = 6; load_station_index = 0; }
                    escape_timer::clear(game);
                }
                kLoadingGameState_22_EscapingCeres => {
                    word! {game; area_index = 0; load_station_index = 18; }
                    save_slots::load_area_map(game);
                }
                _ => {}
            }
        }
        game.initialize_gameplay_display();
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
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        game.enable_display_nmi(true);
        enemy_projectiles::lifecycle::set_enabled(&mut game.runtime(), true);
        set_mechanisms_enabled(game, true);
        palette_effects::set_enabled(game.runtime().session_mut(), true);
        game.set_hdma_enabled(true);
        animated_tiles::set_enabled(game.runtime().session_mut(), true);
        crate::player_movement::speed::set_liquid_physics(&mut game.runtime());
        stage = if u32::from(word!(game.game_state)) == kGameState_40_TransitionToDemo {
            2
        } else if u32::from(word!(game.loading_game_state)) == kLoadingGameState_22_EscapingCeres {
            game.queue_music_delayed8(5);
            3
        } else {
            4
        };
        word!(game.loop_counter_transfer_enemies_to_vram = if stage == 3 { 15 } else { 6 });
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
            word!(game.loop_counter_transfer_enemies_to_vram -= 1);
            if (word!(game.loop_counter_transfer_enemies_to_vram) & 0x8000) != 0 {
                break;
            }
            waiting = false;
        }
        if stage == 2 {
            demo_rooms::finish_setup(game);
        }
        if stage == 4 {
            word!(game.game_state = kGameState_7_MainGameplayFadeIn as u16);
        } else {
            word!(game.game_state += 1);
        }
        word_transfers!(game, ascending, 256; native(0xc000) <- le(target_palettes, 0));
        if stage == 4 {
            if u32::from(word!(game.loading_game_state)) == kLoadingGameState_1F_StartingAtCeres {
                word!(game.palette_buffer[223] = 0);
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
    if word!(game.substate) != 1 {
        demo_rooms::advance_set(game);
    }
    game.enable_display_nmi(true);
    word!(game.game_state += 1);
    word! { game, 0; screen_fade_delay, screen_fade_counter }
    game.clear_hdma();
    game.disable_scanline_irq();
    word! { game, 0;
        fx_layer_blending_config_a, cur_irq_handler, irqhandler_next_handler,
    }
    palette_effects::set_enabled(game.runtime().session_mut(), false);
    palette_effects::clear(game.runtime().session_mut());
    word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
    word_transfers!(game, descending, 270; native(0x996) <- value(0));
    game.reset_layer_mirrors();
    0
}

/// Demo phase: choose the next demo or title step.
fn next_demo<H: Host>(game: &mut H) -> CoroutineRet {
    word!(game.game_state = kGameState_1_OpeningCinematic as u16);
    if (word!(game.substate) & 0x8000) != 0 {
        word!(game.game_state = kGameState_40_TransitionToDemo as u16);
    } else if word!(game.substate) != 0 {
        game.load_title_sequence_graphics();
        word! {game; eproj_x_pos[4] = 2; cinematic_function = ((0x8b9a47i32) as u16); }
    } else {
        game.queue_music_delayed8(0);
        word! {game; debug_disable_sounds = 0; cinematic_function = ((0x8b9b68i32) as u16); }
    }
    0
}

/// Pause phase: enter the pause screen.
fn setup_pause<H: Host>(game: &mut H) -> CoroutineRet {
    game.set_hdma_enabled(false);
    write_register_and_mirror(game, HDMAEN as u16, 0, 0x85, true);
    animated_tiles::set_enabled(game.runtime().session_mut(), false);
    game.capture_pause_background();
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
    game.load_pause_assets();
    game.upload_pause_map();
    word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0xb6f000, 0));
    game.pause_graphics(true);
    pause_controls::initialize(game);
    0
}

/// Pause phase: one pause menu frame.
fn advance_pause<H: Host>(game: &mut H) -> CoroutineRet {
    controller_input::filter(3, game);
    // Preserve the eight-entry bounds failure before any tab work.
    let tab = [0, 1, 2, 3, 4, 5, 6, 7][usize::from(word!(game.menu_index))];
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
                game.upload_pause_map();
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
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- pause_map(384));
        game.pause_graphics(false);
        pause_controls::resume(game);
        game.write_register_word(MDMAEN as u16, 0);
        game.enable_display_nmi(false);
        game.disable_scanline_irq();
        crate::room_entry::installation::load_tiles_and_palette(&mut game.runtime());
        crate::room_entry::background::load(&mut game.runtime(), LoadMode::Initial);
        fill_viewport(game, |game| game.flush_tilemaps());
        crate::room_mechanisms::graphics::reload_graphics(&mut game.runtime());
        game.enable_display_nmi(true);
        game.enable_scanline_irq(false);
        pause_controls::restore_graphics(game);
        game.load_vram_words(
            0x7edf5c,
            u16::from(game.ram().read_byte(0x59) & 0xfc) << 8,
            0x1000,
        );
        word! { game, 1; screen_fade_delay, screen_fade_counter }
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
        word!(game.game_state += 1);
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
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
        if matches!(entry, Entry::GameOverFadeOut) {
            word! { game, 0; menu_index, debug_disable_sounds }
        }
    }
    0
}

/// Death phase: leave the time-up whiteout.
fn exit_time_up<H: Host>(game: &mut H) -> CoroutineRet {
    game.fade(false);
    if game.ram().read_byte(0x51) == 0x80 {
        game.enable_display_nmi(true);
        game.finish_gameplay_display(true);
        word! { game, 0;
            screen_fade_delay, screen_fade_counter, ceres_status, timer_status,
        }
        cancel_sound_effects(game, 15);
        if crate::progress::event_happened(&mut game.runtime(), 0xe) {
            word! { game, 0; game_options_screen_index, menu_index }
            word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
            word!(game.game_state = kGameState_25_SamusNoHealth as u16);
        } else {
            word!(game.game_state = kGameState_37_CeresGoesBoomWithSamus as u16);
            word!(game.cinematic_function = (0x8bc11bi32) as u16);
        }
    }
    0
}

/// States that run the gameplay pass: their pre-pass work, the pass and,
/// once it completes, their per-state finish.
fn pass<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    if matches!(entry, Entry::CeresElevator | Entry::CeresBlackout) && word!(game.timer_status) != 0
    {
        escape_timer::draw(game);
    }
    if matches!(entry, Entry::ReserveTanks)
        && game.ram().read_byte(0x612) == 0
        && reserve_tanks::refill_auto(game) != 0
    {
        word! {game; time_is_frozen_flag = 0; game_state = kGameState_8_MainGameplay as u16; }
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
        crate::player_projectiles::feedback::grapple_flare(&mut game.runtime());
        if word!(game.debug_disable_sprite_interact) == 0 {
            game.player_projectiles_hit_samus();
        }
        actors::scheduling::advance_selected(&mut game.runtime());
    }
    // Read the queue after enemy callbacks, even when they changed it this pass.
    if resume == ResumePoint::EnemyMessage
        || (resume == ResumePoint::Start && word!(game.queued_message_box_index) != 0)
    {
        let yielded = message_boxes::display(word!(game.queued_message_box_index), game);
        if yielded != 0 {
            game.set_resume_point(ResumePoint::EnemyMessage as u8);
            return yielded;
        }
        word!(game.queued_message_box_index = 0);
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
        if word!(game.debug_disable_sprite_interact) == 0 {
            enemy_projectiles::contact::enemy_projectiles_hit_samus(&mut game.runtime());
            enemy_projectiles::contact::shoot_enemy_projectiles(&mut game.runtime());
            actors::contacts::apply_power_bomb_to_enemies(&mut game.runtime());
        }
        advance_camera(game);
        let debug_scrolling_enabled: i32 = 0;
        if debug_scrolling_enabled != 0 {
            debug_position(game);
        }
        actors::drawing::draw_scene(&mut game.runtime());
        actors::staging::queue_actor_tiles(&mut game.runtime());
        hud::advance(game);
        update_scrolls(game);
        game.advance_room();
        finish_health_and_time(game);
        finish_frame(game);
    }
    game.set_resume_point(ResumePoint::Start as u8);
    0
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
                word! { game, 0; screen_fade_delay, screen_fade_counter }
                if matches!(entry, Entry::FadeIn) {
                    word!(game.game_state += 1);
                } else {
                    word!(game.game_state = kGameState_8_MainGameplay as u16);
                }
            }
        }
        Entry::CeresElevator => {
            let negative =
                (word!(game.reached_ceres_elevator_fade_timer -= 1, reread) & 0x8000) != 0;
            if word!(game.reached_ceres_elevator_fade_timer) == 0 || negative {
                word!(game.game_state += 1);
                word! { game, 0; screen_fade_delay, screen_fade_counter }
            }
        }
        Entry::TimeUp | Entry::DeathBlackout => {
            let death = matches!(entry, Entry::DeathBlackout);
            if fading::all(if death { 6 } else { 8 }, game) != 0 {
                if death {
                    game.finish_gameplay_display(true);
                    word! { game, 0;
                        game_options_screen_index, screen_fade_delay, screen_fade_counter,
                    }
                    word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
                    word! {game; g_word_7E0DE8 = 16; game_options_screen_index = 3; }
                    word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
                    word!(game.game_state += 1);
                    word!(game.power_bomb_explosion_status = 0);
                    cancel_sound_effects(game, 15);
                    game.queue_music_delayed8(0);
                    game.queue_music_delayed8(0xff39);
                    game.queue_music_delayed(5, 0xe);
                } else {
                    word!(game.game_state = kGameState_36_WhitingOutFromTimeUp as u16);
                    word! { game, 0; screen_fade_delay, screen_fade_counter }
                }
            }
        }
        Entry::ReserveTanks => {
            game.advance_low_health_warning();
        }
        Entry::PrepareDeath => {
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
        }
        Entry::CeresBlackout | Entry::ZebesEscape => {
            game.fade(false);
            if game.ram().read_byte(0x51) == 0x80 {
                let ceres = matches!(entry, Entry::CeresBlackout);
                game.enable_display_nmi(true);
                game.finish_gameplay_display(ceres);
                word! { game, 0; screen_fade_delay, screen_fade_counter }
                if ceres {
                    word!(game.loading_game_state = kLoadingGameState_22_EscapingCeres as u16);
                    word!(game.game_state = 34);
                    save_slots::save(word!(game.selected_save_slot), game);
                    word!(game.cinematic_function = (0x8bc11bi32) as u16);
                    word! { game, 0; ceres_status, timer_status }
                } else {
                    word! {game; game_state = kGameState_39_EndingAndCredits as u16;
                    cinematic_function = ((0x8bd480i32) as u16); timer_status = 0; }
                }
                game.queue_music_delayed8(0);
                cancel_sound_effects(game, 15);
            }
        }
        Entry::PauseFadeOut => {
            game.fade(false);
            if game.ram().read_byte(0x51) & 0xf == 0 {
                game.enable_display_nmi(true);
                word! { game, 0; screen_fade_delay, screen_fade_counter }
                word!(game.game_state += 1);
            }
        }
        Entry::DemoStart => {
            game.update_hdma();
            word!(game.game_state += 1);
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
