//! Ordered gameplay phases and the continuation owned by this scheduler.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
use crate::game::actors;
use crate::game::animated_tiles;
use crate::game::demo_rooms;
use crate::game::enemy_projectiles::EprojRunAll;
use crate::game::message_boxes;
use crate::game::palette_effects;
use crate::game::player_projectiles::prepare;
use crate::game::prelude::*;
use crate::game::room_entry::{self, ArrivalRequest, EntrySignal, StartSite};
use crate::game::room_mechanisms::run;
use crate::game::room_scrolls::advance_camera;
use crate::game::room_scrolls::{debug_position, update_scrolls};
use crate::game::room_state::advance_room;
use crate::game::session_runtime::{Runtime, player_command};
use crate::ram_map::{checked_word, words as ram};

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
    Initialize,
    Gameplay,
    FadeIn,
    Unpause,
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
            Entry::TimeUp => {
                if palette_effects::fading::all(8, game) != 0 {
                    word!(game.game_state = kGameState_36_WhitingOutFromTimeUp as u16);
                    word! { game, 0; screen_fade_delay, screen_fade_counter }
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
            Entry::TimeUpExit
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
