//! Ordered gameplay phases and the continuation owned by this scheduler.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
use crate::game::actors;
use crate::game::animated_tiles;
use crate::game::enemy_projectiles::EprojRunAll;
use crate::game::message_boxes;
use crate::game::palette_effects;
use crate::game::player_projectiles::prepare;
use crate::game::prelude::*;
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
    Gameplay,
    FadeIn,
    Unpause,
    CeresElevator,
    TimeUp,
}

/// Advances through completion or a suspension, returning the callee's exact
/// yield byte. Nested PLM/message continuations remain owned by those modules.
/// Gameplay callbacks still have unsafe memory contracts; this is not a safe
/// facade over them. The scheduler's continuation access itself is checked.
/// Entry stays fixed even when callbacks change the live game state.
pub(crate) unsafe fn advance(entry: Entry, game: &mut Game) -> CoroutineRet {
    unsafe {
        if matches!(entry, Entry::CeresElevator) && word!(game.timer_status) != 0 {
            crate::game::escape_timer::draw(game);
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
            Entry::Gameplay => {}
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
