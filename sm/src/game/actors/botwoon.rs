//! Botwoon runs in `sm_game::actors::botwoon`; its boss flag store comes from
//! here, and its death signals run in the Runtime lease.
#![forbid(unsafe_code)]
use crate::game::gameplay_flag_runtime::set_boss_bits_for_current_area;
use crate::game_state::Game;
pub(crate) use sm_game::actors::botwoon::*;

// The named bank $B3 functions are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::botwoon::{function as f, head_function as h, movement_function as m};
    const fn at(word: u16) -> u32 {
        0xb3_0000 | word as u32
    }
    assert!(at(f::INITIAL) == 11769976 && at(f::GO_THROUGH_HOLE) == 11770013);
    assert!(at(f::MOVING_AROUND) == 11770276 && at(f::SPITTING) == 11770340);
    assert!(at(f::DEATH_PRE_DEATH_DELAY) == 11770438 && at(f::DEATH_FALLING_TO_GROUND) == 11770462);
    assert!(at(f::DEATH_WAIT_FOR_BODY_TO_FALL) == 11770570);
    assert!(at(f::DEATH_CRUMBLING_WALL) == 11770617);
    assert!(at(m::MOVE_TOWARD_TARGET_HOLE) == 11770807 && at(m::MOVE_BY_MOVEMENT_DATA) == 11788940);
    assert!(at(m::START_MOVING_BY_MOVEMENT_DATA) == 11788880);
    assert!(at(h::MOVING_AROUND) == 11771328 && at(h::SPITTING_SET_ANGLE_AND_SHOW) == 11771517);
    assert!(at(h::SPITTING_SPAWN_5) == 11771616 && at(h::SPITTING_SPAWN_3) == 11771700);
    assert!(at(h::SPITTING_COOLDOWN) == 11771770);
};

impl Effects for Game {
    fn set_boss_bit(&mut self, mask: u16) {
        set_boss_bits_for_current_area(mask, self);
    }
}
