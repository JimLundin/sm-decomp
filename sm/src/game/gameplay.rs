//! Frame dispatch, ordered gameplay phases and their resumable continuations.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
use crate::game::actors;
use crate::game::animated_tiles;
use crate::game::demo_rooms;
use crate::game::display_refresh::{self, GraphicsAsset};
use crate::game::enemy_projectiles::EprojRunAll;
use crate::game::file_menu::advance as advance_file_menu;
use crate::game::game_over::advance as advance_game_over;
use crate::game::map_menu::advance as advance_map_menu;
use crate::game::message_boxes;
use crate::game::options_settings;
use crate::game::palette_effects;
use crate::game::pause_controls;
use crate::game::pause_controls::fade_in as pause_fade_in;
use crate::game::player_projectiles::prepare;
use crate::game::prelude::*;
use crate::game::room_entry::{self, ArrivalRequest, EntrySignal, StartSite};
use crate::game::room_mechanisms::run;
use crate::game::room_scrolls::advance_camera;
use crate::game::room_scrolls::{debug_position, update_scrolls};
use crate::game::room_state::advance_room;
use crate::game::room_state::{pause_draygon, unpause_draygon};
use crate::game::samus_animation::{advance_death_pose, begin_death, explode_suit, flash_death};
use crate::game::session_runtime::{Runtime, player_command};
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_,
};
use crate::game::sm_a7::{
    Kraid_UnpauseHook_IsSinking, PauseHook_Kraid, UnpauseHook_Kraid_IsAlive,
    UnpauseHook_Kraid_IsDead,
};
use crate::game::sm_a9::MotherBrainsBody_UnpauseHook;
use crate::ram_map::{checked_word, words as ram};
use crate::types::Func_V_Coroutine;

#[derive(Clone, Copy)]
enum FrameEntry {
    Advance(Entry),
    Callback(Func_V_Coroutine),
    Door(EntrySignal),
    Demo,
    Options,
    Null,
    Unused,
    Debug,
}
use FrameEntry::{Advance, Callback, Debug, Demo, Door, Null, Options, Unused};

static GAME_STATE_ENTRIES: [FrameEntry; 45] = callback_table! { FrameEntry;
    {Callback(demo_rooms::reset)}, {Callback(GameState_1_OpeningCinematic_)}, {Options},
    {Null}, {Callback(advance_file_menu)}, {Callback(advance_map_menu)},
    {Advance(Entry::Initialize)}, {Advance(Entry::FadeIn)}, {Advance(Entry::Gameplay)},
    {Door(EntrySignal::DoorState9)}, {Door(EntrySignal::DoorState10)},
    {Door(EntrySignal::DoorState11)}, {Advance(Entry::PauseFadeOut)},
    {Advance(Entry::PauseSetup)}, {Callback(pause_fade_in)}, {Advance(Entry::Paused)},
    {Advance(Entry::UnpauseFadeOut)}, {Advance(Entry::UnpauseSetup)}, {Advance(Entry::Unpause)},
    {Advance(Entry::PrepareDeath)}, {Advance(Entry::DeathBlackout)}, {Callback(begin_death)},
    {Callback(advance_death_pose)}, {Callback(flash_death)},
    {Callback(explode_suit)}, {Advance(Entry::GameOverFadeOut)},
    {Callback(advance_game_over)}, {Advance(Entry::ReserveTanks)}, {Unused},
    {Debug}, {Callback(GameState_37_CeresGoesBoomWithSamus_)},
    {Advance(Entry::Initialize)}, {Advance(Entry::CeresElevator)},
    {Advance(Entry::CeresBlackout)}, {Callback(GameState_37_CeresGoesBoomWithSamus_)}, {Advance(Entry::TimeUp)},
    {Advance(Entry::TimeUpExit)}, {Callback(GameState_37_CeresGoesBoomWithSamus_)},
    {Advance(Entry::ZebesEscape)}, {Callback(GameState_39_EndingAndCredits_)},
    {Advance(Entry::Initialize)}, {Advance(Entry::DemoStart)}, {Demo},
    {Advance(Entry::DemoStop)}, {Advance(Entry::DemoNext)},
};
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
    fn read(game: &Game) -> Self {
        match game.coroutine_state_1() {
            0 => Self::Start,
            1 => Self::EnemyMessage,
            2 => Self::RoomObjects,
            _ => Self::Skip,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Entry {
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
/// Gameplay callbacks still have unsafe memory contracts; this is not a safe
/// facade over them. The scheduler's continuation access itself is checked.
/// Entry stays fixed even when callbacks change the live game state.
pub(crate) unsafe fn advance(entry: Entry, game: &mut Game) -> CoroutineRet {
    unsafe {
        // Initial room loading retains the same unsafe backend contract on every
        // resume path, including actor staging after the display callbacks are skipped.
        if matches!(entry, Entry::Initialize) {
            crate::verify_probe::hit("coroutine.init");
            let mut stage = game.g_ram[0x612];
            let mut waiting = stage >= 2;
            if stage == 0 {
                let demo = u32::from(word!(game.game_state)) == kGameState_40_TransitionToDemo;
                if !demo {
                    match u32::from(word!(game.loading_game_state)) {
                        kLoadingGameState_1F_StartingAtCeres => {
                            word! {game; area_index = 6; load_station_index = 0; }
                            crate::game::escape_timer::clear(game);
                        }
                        kLoadingGameState_22_EscapingCeres => {
                            word! {game; area_index = 0; load_station_index = 18; }
                            crate::game::save_slots::load_area_map(game);
                        }
                        _ => {}
                    }
                }
                crate::game::display_refresh::initialize_gameplay(game);
                crate::game::session_runtime::initialize_player(game);
                if demo {
                    demo_rooms::load(game);
                } else {
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
                    game.g_ram[0x612] = 1;
                    return yielded;
                }
                crate::game::hud::initialize(game);
                word_transfers!(game, ascending, 16; native(0xc380) <- native(0xc180));
                word! { game, 1; screen_fade_delay, screen_fade_counter }
                crate::game::display_refresh::enable(true, game);
                crate::game::enemy_projectiles::EnableEprojs(game);
                crate::game::room_mechanisms::set_enabled(true, game);
                palette_effects::set_enabled(true, game);
                crate::game::visual_runtime::set_hdma_enabled(true, game);
                animated_tiles::set_enabled(true, game);
                crate::game::player_movement::speed::set_liquid_physics(&mut Runtime::new(game));
                stage = if u32::from(word!(game.game_state)) == kGameState_40_TransitionToDemo {
                    2
                } else if u32::from(word!(game.loading_game_state))
                    == kLoadingGameState_22_EscapingCeres
                {
                    crate::game::music::QueueMusic_Delayed8(5, game);
                    3
                } else {
                    4
                };
                word!(game.loop_counter_transfer_enemies_to_vram = if stage == 3 { 15 } else { 6 });
            }
            if matches!(stage, 2..=4) {
                crate::verify_probe::hit(match stage {
                    2 => "coroutine.init.demo",
                    3 => "coroutine.init.escape",
                    _ => "coroutine.init.normal",
                });
                loop {
                    if !waiting {
                        actors::staging::transfer_actor_tiles_and_initialize(&mut Runtime::new(
                            game,
                        ));
                    }
                    let yielded: u8 = crate::game::display_refresh::wait(true, game);
                    if yielded != 0 {
                        game.g_ram[0x612] = stage;
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
                    if u32::from(word!(game.loading_game_state))
                        == kLoadingGameState_1F_StartingAtCeres
                    {
                        word!(game.palette_buffer[223] = 0);
                        player_command(8, game);
                    } else {
                        player_command(9, game);
                    }
                }
            }
            game.g_ram[0x612] = 0;
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::Frame) {
            let returned = 'frame: {
                let mut st: i32 = game.g_ram[0x611] as i32;
                let begin_frame: bool = if st >= 1 && st <= 3 {
                    let mut _coret_: u8 = crate::game::boot::resume(game);
                    if _coret_ != 0 {
                        break 'frame _coret_;
                    }
                    true
                } else {
                    !(st >= 10)
                        && ({
                            if st != 0 {
                                crate::host::Die(c"Incorrect coroutine_state_0".as_ptr());
                            }
                            true
                        })
                };
                if begin_frame {
                    for offset in (0x612..0x616).rev() {
                        game.g_ram[offset] = 0;
                    }
                    crate::game::controller_input::sample(game);
                    crate::game::visual_runtime::update_hdma(game);
                    game.next_random();
                    crate::game::sprite_drawing::clear_high_table(game);
                    word! { game, 0;
                        oam_next_ptr, nmi_copy_samus_halves, nmi_copy_samus_top_half_src,
                        nmi_copy_samus_bottom_half_src,
                    }
                    st = word!(game.game_state) as i32 + 10;
                    game.g_ram[0x611] = st as u8;
                }
                let mut _coret__0: u8 = match GAME_STATE_ENTRIES[usize::from((st - 10i32) as u16)] {
                    Advance(entry) => advance(entry, game),
                    Callback(callback) => callback(game),
                    Door(signal) => {
                        let poll = room_entry::protocol::resume(&mut Runtime::new(game), signal);
                        poll.returned_byte()
                    }
                    Demo => demo_rooms::play(game, |game| advance(Entry::Gameplay, game)),
                    Null => 0,
                    Unused => {
                        crate::rtl::Unreachable(game);
                        0
                    }
                    Debug => {
                        crate::cpu_infra::DebugGameOverMenu();
                        0
                    }
                    Options => options_settings::advance(
                        game,
                        |game| {
                            crate::rtl::register_writes! { game;
                                mirror(VMADDL as u16, 0, 0x72, true); byte(VMADDH as u16, 0x58);
                                byte(VMAIN as u16, 0x80);
                            }
                            display_refresh::setup_transfer(
                                u16::from_ne_bytes([1, 0x18]),
                                0x8edc00,
                                0x800,
                                game,
                            );
                            WriteReg(MDMAEN as u16, 2, game);
                        },
                        |game| display_refresh::enable(true, game),
                    ),
                };
                if _coret__0 != 0 {
                    break 'frame _coret__0;
                }
                crate::game::music::advance_sound_effects(game, |channel, value, game| {
                    crate::native_audio::write_sound_port(channel, value, game)
                });
                crate::game::sprite_drawing::finish_frame(game);
                word!(game.joypad1_input_samusfilter = word!(game.joypad1_lastkeys));
                if game.g_ram[0x612] | game.g_ram[0x613] | game.g_ram[0x614] | game.g_ram[0x615]
                    != 0
                {
                    printf(
                        c"Coroutine state: %d, %d, %d, %d\n".as_ptr(),
                        game.g_ram[0x612] as i32,
                        game.g_ram[0x613] as i32,
                        game.g_ram[0x614] as i32,
                        game.g_ram[0x615] as i32,
                    );
                    crate::host::Warning(c"Coroutine State is broken!".as_ptr());
                }
                game.g_ram[0x611] = 0;
                0
            };
            if returned == 0 {
                game.g_ram[0x5b4] = 1;
            }
            display_refresh::run(game);
            return returned;
        }
        if matches!(entry, Entry::DemoStop) {
            if word!(game.substate) != 1 {
                demo_rooms::advance_set(game);
            }
            crate::game::display_refresh::enable(true, game);
            word!(game.game_state += 1);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            crate::game::visual_runtime::clear_hdma(game);
            crate::game::scanline_irq::disable(game);
            word! { game, 0;
                fx_layer_blending_config_a, cur_irq_handler, irqhandler_next_handler,
            }
            palette_effects::set_enabled(false, game);
            palette_effects::clear(game);
            word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
            word_transfers!(game, descending, 270; native(0x996) <- value(0));
            display_refresh::reset_layer_mirrors(game);
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::DemoNext) {
            word!(game.game_state = kGameState_1_OpeningCinematic as u16);
            if (word!(game.substate) & 0x8000) != 0 {
                word!(game.game_state = kGameState_40_TransitionToDemo as u16);
            } else if word!(game.substate) != 0 {
                crate::game::sm_8b::LoadTitleSequenceGraphics(game);
                word! {game; eproj_x_pos[4] = 2; cinematic_function = ((0x8b9a47i32) as u16); }
            } else {
                crate::game::music::QueueMusic_Delayed8(0, game);
                word! {game; debug_disable_sounds = 0; cinematic_function = ((0x8b9b68i32) as u16); }
            }
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::PauseSetup) {
            crate::game::visual_runtime::set_hdma_enabled(false, game);
            crate::rtl::write_register_and_mirror(HDMAEN as u16, 0, 0x85, true, game);
            animated_tiles::set_enabled(false, game);
            display_refresh::capture_pause_background(game);
            match pause_controls::hook_address(true, game) {
                8946657 => {}
                9423094 => pause_draygon(game),
                10994469 => PauseHook_Kraid(game),
                _ => {
                    Unreachable(game);
                }
            }
            crate::game::music::cancel_effects(6, game);
            pause_controls::save_graphics(game);
            display_refresh::load_pause_assets(game);
            display_refresh::upload_pause_map(game);
            word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
            word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0xb6f000, 0));
            display_refresh::pause_graphics(true, game);
            pause_controls::initialize(game);
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::Paused) {
            crate::game::controller_input::filter(3, game);
            // Preserve the eight-entry bounds failure before any tab work.
            let tab = [0, 1, 2, 3, 4, 5, 6, 7][usize::from(word!(game.menu_index))];
            match tab {
                0 => pause_controls::update_map(game),
                1 => pause_controls::update_equipment(game),
                2 | 5 => pause_controls::fade_out_tab(game, tab == 5, |game| {
                    display_refresh::enable(true, game);
                }),
                3 | 6 => pause_controls::load_tab(game, tab == 3, |game| {
                    if tab == 3 {
                        GraphicsAsset::VramWords(0x7e3800, 0x3000, 0x800).load(game);
                    } else {
                        display_refresh::upload_pause_map(game);
                    }
                }),
                4 => pause_controls::fade_tab::<true>(game),
                _ => pause_controls::fade_tab::<false>(game),
            }
            crate::game::hud::advance(game);
            crate::game::map_menu::animate_palette(game);
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::UnpauseSetup) {
            let resume = game.g_ram[0x612];
            if resume == 0 {
                GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1000).load(game);
                word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- pause_map(384));
                display_refresh::pause_graphics(false, game);
                pause_controls::resume(game);
                WriteRegWord(MDMAEN as u16, 0, game);
                display_refresh::enable(false, game);
                crate::game::scanline_irq::disable(game);
                crate::game::room_state::load_tiles_and_palette(game);
                crate::game::room_entry::background::load(
                    &mut Runtime::new(game),
                    crate::game::room_entry::content::LoadMode::Initial,
                );
                crate::game::room_scrolls::fill_viewport(game, |game| {
                    display_refresh::flush_tilemaps(game)
                });
                crate::game::room_mechanisms::reload_graphics(game);
                display_refresh::enable(true, game);
                crate::game::scanline_irq::enable(false, game);
                pause_controls::restore_graphics(game);
                GraphicsAsset::VramWords(0x7edf5c, u16::from(game.g_ram[0x59] & 0xfc) << 8, 0x1000)
                    .load(game);
                word! { game, 1; screen_fade_delay, screen_fade_counter }
            }
            if resume <= 1 {
                let yielded = match pause_controls::hook_address(false, game) {
                    8946657 => 0,
                    9423100 => unpause_draygon(game),
                    10994171 => UnpauseHook_Kraid_IsDead(game),
                    10994254 => UnpauseHook_Kraid_IsAlive(game),
                    10994336 => Kraid_UnpauseHook_IsSinking(game),
                    11110243 => MotherBrainsBody_UnpauseHook(game),
                    _ => {
                        Unreachable(game);
                        0
                    }
                };
                if yielded != 0 {
                    game.g_ram[0x612] = 1;
                    return yielded;
                }
                crate::game::visual_runtime::set_hdma_enabled(true, game);
                animated_tiles::set_enabled(true, game);
                crate::game::player_projectiles::restore_sound(game);
                word!(game.game_state += 1);
            }
            game.g_ram[0x612] = 0;
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::UnpauseFadeOut | Entry::GameOverFadeOut) {
            if matches!(entry, Entry::UnpauseFadeOut) {
                crate::game::pause_controls::draw_unpause(game);
            }
            crate::game::display_refresh::fade(false, game);
            if game.g_ram[0x51] == 0x80 {
                crate::game::display_refresh::enable(true, game);
                word! { game, 0; screen_fade_delay, screen_fade_counter }
                word!(game.game_state += 1);
                if matches!(entry, Entry::GameOverFadeOut) {
                    word! { game, 0; menu_index, debug_disable_sounds }
                }
            }
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::TimeUpExit) {
            crate::game::display_refresh::fade(false, game);
            if game.g_ram[0x51] == 0x80 {
                crate::game::display_refresh::enable(true, game);
                crate::game::display_refresh::finish_gameplay(true, game);
                word! { game, 0;
                    screen_fade_delay, screen_fade_counter, ceres_status, timer_status,
                }
                crate::game::music::cancel_effects(15, game);
                if crate::game::gameplay_flag_runtime::CheckEventHappened(0xe, game) != 0 {
                    word! { game, 0; game_options_screen_index, menu_index }
                    word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
                    word!(game.game_state = kGameState_25_SamusNoHealth as u16);
                } else {
                    word!(game.game_state = kGameState_37_CeresGoesBoomWithSamus as u16);
                    word!(game.cinematic_function = (0x8bc11bi32) as u16);
                }
            }
            return 0 as CoroutineRet;
        }
        if matches!(entry, Entry::CeresElevator | Entry::CeresBlackout)
            && word!(game.timer_status) != 0
        {
            crate::game::escape_timer::draw(game);
        }
        if matches!(entry, Entry::ReserveTanks)
            && game.g_ram[0x612] == 0
            && crate::game::reserve_tanks::refill_auto(game) != 0
        {
            word! {game; time_is_frozen_flag = 0; game_state = kGameState_8_MainGameplay as u16; }
            player_command(0x10, game);
        }
        crate::verify_probe::hit("coroutine.gameplay");
        let resume = ResumePoint::read(game);
        if resume == ResumePoint::Start {
            actors::scheduling::select(&mut Runtime::new(game));
            palette_effects::advance(game);
            prepare(game);
            if word!(game.debug_disable_sprite_interact) == 0 {
                crate::game::projectile_contact::player_projectiles_hit_samus(game);
            }
            actors::scheduling::advance_selected(&mut Runtime::new(game));
        }
        // Read the queue after enemy callbacks, even when they changed it this pass.
        if resume == ResumePoint::EnemyMessage
            || (resume == ResumePoint::Start && word!(game.queued_message_box_index) != 0)
        {
            let yielded = message_boxes::display(word!(game.queued_message_box_index), game);
            if yielded != 0 {
                game.set_coroutine_state_1(ResumePoint::EnemyMessage as u8);
                return yielded;
            }
            word!(game.queued_message_box_index = 0);
        }
        if matches!(resume, ResumePoint::Start | ResumePoint::EnemyMessage) {
            crate::game::session_runtime::player_update_runtime::advance_frame(game);
            EprojRunAll(game);
        }
        if resume != ResumePoint::Skip {
            let yielded = run(game);
            if yielded != 0 {
                game.set_coroutine_state_1(ResumePoint::RoomObjects as u8);
                return yielded;
            }
            animated_tiles::advance(game);
            // This is a new read: earlier callbacks may change interaction policy.
            if word!(game.debug_disable_sprite_interact) == 0 {
                crate::game::enemy_projectiles::enemy_projectiles_hit_samus(game);
                crate::game::enemy_projectiles::shoot_enemy_projectiles(game);
                actors::contacts::apply_power_bomb_to_enemies(&mut Runtime::new(game));
            }
            advance_camera(game);
            let debug_scrolling_enabled: i32 = 0;
            if debug_scrolling_enabled != 0 {
                debug_position(game);
            }
            actors::drawing::draw_scene(&mut Runtime::new(game));
            actors::staging::queue_actor_tiles(&mut Runtime::new(game));
            crate::game::hud::advance(game);
            update_scrolls(game);
            advance_room(game);
            finish_health_and_time(game);
            finish_frame(game);
        }
        game.set_coroutine_state_1(ResumePoint::Start as u8);
        match entry {
            Entry::FadeIn | Entry::Unpause => {
                crate::game::display_refresh::fade(true, game);
                if game.g_ram[0x51] == 15 {
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
                if palette_effects::fading::all(if death { 6 } else { 8 }, game) != 0 {
                    if death {
                        crate::game::display_refresh::finish_gameplay(true, game);
                        word! { game, 0;
                            game_options_screen_index, screen_fade_delay, screen_fade_counter,
                        }
                        word_transfers!(game, descending, 128; le(eproj_y_subpos, 142) <- value(0));
                        word! {game; g_word_7E0DE8 = 16; game_options_screen_index = 3; }
                        word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
                        word!(game.game_state += 1);
                        word!(game.power_bomb_explosion_status = 0);
                        crate::game::music::cancel_effects(15, game);
                        crate::game::music::QueueMusic_Delayed8(0, game);
                        crate::game::music::QueueMusic_Delayed8(0xff39, game);
                        crate::game::music::QueueMusic_DelayedY(5, 0xe, game);
                    } else {
                        word!(game.game_state = kGameState_36_WhitingOutFromTimeUp as u16);
                        word! { game, 0; screen_fade_delay, screen_fade_counter }
                    }
                }
            }
            Entry::ReserveTanks => {
                crate::game::session_runtime::player_update_runtime::low_health(game);
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
                crate::game::display_refresh::fade(false, game);
                if game.g_ram[0x51] == 0x80 {
                    let ceres = matches!(entry, Entry::CeresBlackout);
                    crate::game::display_refresh::enable(true, game);
                    crate::game::display_refresh::finish_gameplay(ceres, game);
                    word! { game, 0; screen_fade_delay, screen_fade_counter }
                    if ceres {
                        word!(game.loading_game_state = kLoadingGameState_22_EscapingCeres as u16);
                        word!(game.game_state = 34);
                        crate::game::save_slots::save(word!(game.selected_save_slot), game);
                        word!(game.cinematic_function = (0x8bc11bi32) as u16);
                        word! { game, 0; ceres_status, timer_status }
                    } else {
                        word! {game; game_state = kGameState_39_EndingAndCredits as u16;
                        cinematic_function = ((0x8bd480i32) as u16); timer_status = 0; }
                    }
                    crate::game::music::QueueMusic_Delayed8(0, game);
                    crate::game::music::cancel_effects(15, game);
                }
            }
            Entry::PauseFadeOut => {
                crate::game::display_refresh::fade(false, game);
                if game.g_ram[0x51] & 0xf == 0 {
                    crate::game::display_refresh::enable(true, game);
                    word! { game, 0; screen_fade_delay, screen_fade_counter }
                    word!(game.game_state += 1);
                }
            }
            Entry::DemoStart => {
                crate::game::visual_runtime::update_hdma(game);
                word!(game.game_state += 1);
                game.g_ram[0x51] = 15;
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
        0 as CoroutineRet
    }
}

fn finish_health_and_time(game: &mut Game) {
    if (word!(game.samus_health) as i16) <= 0 {
        if (word!(game.reserve_health_mode) & 1) != 0 && word!(game.samus_reserve_health) != 0 {
            word!(game.time_is_frozen_flag = 0x8000);
            word!(game.game_state = kGameState_27_ReserveTanksAuto as u16);
            player_command(0x1b, game);
        } else if word!(game.game_state) as i32 != kGameState_8_MainGameplay as i32 {
            return;
        } else {
            word!(game.time_is_frozen_flag = 0x8000);
            player_command(0x11, game);
            word!(game.game_state = kGameState_19_SamusNoHealth as u16);
        }
    }
    // Clock words are frames, seconds, minutes, then hours in ascending RAM order.
    'carry: {
        for timer in (ram::game_time_frames..ram::game_time_hours).step_by(2) {
            checked_word!(game, timer; += 1);
            if checked_word!(game, timer;) as i32 - 60i32 & 0x8000 != 0 {
                break 'carry;
            }
            checked_word!(game, timer; = 0);
        }
        word!(game.game_time_hours += 1);
    }
    if word!(game.game_time_hours) as i32 - 100i32 & 0x8000 == 0 {
        word! { game, 59; game_time_frames, game_time_seconds, game_time_minutes }
        word!(game.game_time_hours = 99);
    }
}

static kRoomShakes: [u16; 144] = [
    1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0,
    3, 3, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 3, 0, 3, 0,
    0, 3, 0, 3, 3, 3, 3, 3, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2,
    3, 0, 3, 0, 0, 3, 0, 3, 3, 3, 3, 3, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2,
    0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 3,
];

fn finish_frame(game: &mut Game) {
    if word!(game.earthquake_timer) != 0
        && word!(game.time_is_frozen_flag) == 0
        && (word!(game.earthquake_type) as i16).wrapping_sub(36) < 0
    {
        let index = usize::from(word!(game.earthquake_type)) * 4;
        let direction = if word!(game.earthquake_timer) & 2 != 0 {
            u16::MAX
        } else {
            1
        };
        for (offset, register) in [
            ram::reg_BG1HOFS,
            ram::reg_BG1VOFS,
            ram::reg_BG2HOFS,
            ram::reg_BG2VOFS,
        ]
        .into_iter()
        .enumerate()
        {
            checked_word!(game, register; += kRoomShakes[index + offset].wrapping_mul(direction));
        }
        word!(game.earthquake_timer -= 1);
        if (word!(game.earthquake_type) as i16).wrapping_sub(18) >= 0 {
            actors::scheduling::shake_active_enemies(&mut Runtime::new(game));
        }
    }
    word!(game.frame_counter_every_frame += 1);
    for timer in [
        ram::samus_invincibility_timer,
        ram::samus_knockback_timer,
        ram::projectile_invincibility_timer,
    ] {
        let remaining = checked_word!(game, timer;);
        if remaining != 0 {
            checked_word!(game, timer; = remaining - 1);
        }
    }
    word!(game.interactive_enemy_indexes[0] = u16::MAX);
    word!(game.active_enemy_indexes[0] = u16::MAX);
}
