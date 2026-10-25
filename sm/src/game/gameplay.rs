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
use crate::game::session_runtime::Runtime;
use crate::game::sm_8f::RunRoomMainCode;
use crate::game::sm_80::{
    CalculateLayer2PosAndScrollsWhenScrolling, DebugScrollPosSaveLoad, HandleHudTilemap,
};
use crate::game::sm_82::HandleSamusOutOfHealthAndGameTile;
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

/// Advances through completion or a suspension, returning the callee's exact
/// yield byte. Nested PLM/message continuations remain owned by those modules.
/// Gameplay callbacks still have unsafe memory contracts; this is not a safe
/// facade over them. The scheduler's continuation access itself is checked.
pub(crate) unsafe fn advance(game: &mut Game) -> CoroutineRet {
    unsafe {
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
                DebugScrollPosSaveLoad(game);
            }
            actors::drawing::draw_scene(&mut Runtime::new(game));
            actors::staging::queue_actor_tiles(&mut Runtime::new(game));
            HandleHudTilemap(game);
            CalculateLayer2PosAndScrollsWhenScrolling(game);
            RunRoomMainCode(game);
            HandleSamusOutOfHealthAndGameTile(game);
            finish_frame(game);
        }
        game.set_coroutine_state_1(ResumePoint::Start as u8);
        0 as CoroutineRet
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
