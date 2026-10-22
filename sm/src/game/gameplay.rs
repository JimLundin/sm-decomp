//! Ordered gameplay phases and the continuation owned by this scheduler.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
use crate::game::scene_drawing::{draw_scene, queue_enemy_tiles};
use crate::game::message_boxes;
use crate::game::prelude::*;
use crate::game::sm_80::{
    CalculateLayer2PosAndScrollsWhenScrolling, DebugScrollPosSaveLoad, HandleHudTilemap,
};
use crate::game::sm_82::HandleSamusOutOfHealthAndGameTile;
use crate::game::sm_84::PlmHandler_Async;
use crate::game::sm_86::EprojRunAll;
use crate::game::sm_87::AnimtilesHandler;
use crate::game::sm_8d::PaletteFxHandler;
use crate::game::sm_8f::RunRoomMainCode;
use crate::game::sm_90::{
    HandleControllerInputForGamePhysics, HandleSamusMovementAndPause, MainScrollingRoutine,
};
use crate::game::sm_a0::{
    DecrementSamusTimers, DetermineWhichEnemiesToProcess,
    EnemyMain, EprojProjCollDet, EprojSamusCollDetect, HandleRoomShaking,
    ProcessEnemyPowerBombInteraction,
    SamusProjectileInteractionHandler,
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

/// Advances through completion or a suspension, returning the callee's exact
/// yield byte. Nested PLM/message continuations remain owned by those modules.
/// Gameplay callbacks still have unsafe memory contracts; this is not a safe
/// facade over them. The scheduler's continuation access itself is checked.
pub(crate) unsafe fn advance(game: &mut Game) -> CoroutineRet {
    crate::verify_probe::hit("coroutine.gameplay");
    let resume = ResumePoint::read(game);
    if resume == ResumePoint::Start {
        DetermineWhichEnemiesToProcess(game);
        PaletteFxHandler(game);
        HandleControllerInputForGamePhysics(game);
        if word!(game.debug_disable_sprite_interact) == 0 {
            SamusProjectileInteractionHandler(game);
        }
        EnemyMain(game);
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
        let yielded = PlmHandler_Async(game);
        if yielded != 0 {
            game.set_coroutine_state_1(ResumePoint::RoomObjects as u8);
            return yielded;
        }
        AnimtilesHandler(game);
        // This is a new read: earlier callbacks may change interaction policy.
        if word!(game.debug_disable_sprite_interact) == 0 {
            EprojSamusCollDetect(game);
            EprojProjCollDet(game);
            ProcessEnemyPowerBombInteraction(game);
        }
        MainScrollingRoutine(game);
        let debug_scrolling_enabled: i32 = 0;
        if debug_scrolling_enabled != 0 {
            DebugScrollPosSaveLoad(game);
        }
        draw_scene(game);
        queue_enemy_tiles(game);
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
