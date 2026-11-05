//! Draygon runs in `sm_game::actors::draygon_*`; its sprites, VRAM queue
//! writes and runtime hooks come from here.
#![forbid(unsafe_code)]
use crate::types::EnemyDef;
use core::mem::offset_of;
use sm_game::actors::draygon_grab::{ENEMY_DEF_DAMAGE, ENEMY_DEF_HURT_AI_TIME};

const _: () = assert!(
    offset_of!(EnemyDef, hurt_ai_time) == ENEMY_DEF_HURT_AI_TIME
        && offset_of!(EnemyDef, damage) == ENEMY_DEF_DAMAGE
);

// The named bank $A5 functions are the ones the species table dispatches on.
const _: () = {
    use sm_game::actors::draygon_host::function as f;
    const fn at(word: u16) -> u32 {
        0xa5_0000 | word as u32
    }
    assert!(at(f::FIGHT_INTRO_INITIAL_DELAY) == 10848027 && at(f::FIGHT_INTRO_DANCE) == 10848139);
    assert!(at(f::SWOOP_RIGHT_SETUP) == 10848244 && at(f::SWOOP_RIGHT_DESCENDING) == 10848433);
    assert!(at(f::SWOOP_RIGHT_APEX) == 10848546 && at(f::SWOOP_RIGHT_ASCENDING) == 10848593);
    assert!(at(f::SWOOP_LEFT_SETUP) == 10848691 && at(f::SWOOP_LEFT_DESCENDING) == 10848768);
    assert!(at(f::SWOOP_LEFT_APEX) == 10848848 && at(f::SWOOP_LEFT_ASCENDING) == 10848912);
    assert!(at(f::GOOP_RIGHT_SETUP) == 10849034 && at(f::GOOP_LEFT_SETUP) == 10849422);
    assert!(at(f::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE) == 10849106);
    assert!(at(f::GOOP_LEFT_MOVE_UNTIL_IN_RANGE) == 10849492);
    assert!(at(f::GOOP_RIGHT_FIRING) == 10849198 && at(f::GOOP_LEFT_FIRING) == 10849584);
    assert!(at(f::GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN) == 10849331);
    assert!(at(f::GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN) == 10849714);
    assert!(at(f::CHASE_SAMUS) == 10849817 && at(f::REPELLED_BY_GRAPPLE) == 10850064);
    assert!(
        at(f::GRABBED_MOVING_TO_TARGET) == 10850078 && at(f::GRABBED_RISING_SPIRAL) == 10850262
    );
    assert!(at(f::GRABBED_TAIL_WHIP) == 10850516);
    assert!(at(f::GRABBED_FINAL_TAIL_WHIPS_START) == 10850565);
    assert!(at(f::GRABBED_FINAL_TAIL_WHIPS_ONGOING) == 10850596);
    assert!(
        at(f::FLAIL_TAIL_AND_FLY_STRAIGHT_UP) == 10850600 && at(f::FLY_STRAIGHT_UP) == 10850644
    );
    assert!(
        at(f::DEATH_DRIFT_TO_DEATH_SPOT) == 10850693 && at(f::DEATH_WAIT_FOR_EVIRS) == 10850964
    );
    assert!(at(f::DEATH_BURIED_BY_EVIRS) == 10850987 && at(f::NONE) == 10846283);
    assert!(at(f::EYE_FACING_LEFT) == 10863757 && at(f::EYE_FACING_RIGHT) == 10863891);
};
