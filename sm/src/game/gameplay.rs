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
use crate::game::prelude::*;
use crate::game::room_mechanisms::run;
use crate::game::session_runtime::Runtime;
use crate::game::sm_8f::RunRoomMainCode;
use crate::game::sm_80::{
    CalculateLayer2PosAndScrollsWhenScrolling, DebugScrollPosSaveLoad, HandleHudTilemap,
};
use crate::game::sm_82::HandleSamusOutOfHealthAndGameTile;
use crate::game::sm_90::{
    HandleControllerInputForGamePhysics, HandleSamusMovementAndPause, MainScrollingRoutine,
};
use crate::game::sm_a0::{DecrementSamusTimers, HandleRoomShaking};

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
            HandleControllerInputForGamePhysics(game);
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
            HandleSamusMovementAndPause(game);
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
            MainScrollingRoutine(game);
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
            HandleRoomShaking(game);
            DecrementSamusTimers(game);
        }
        game.set_coroutine_state_1(ResumePoint::Start as u8);
        0 as CoroutineRet
    }
}
