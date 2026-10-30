//! Spore Spawn runs in `sm_game::actors::spore_spawn`; its stem and boss
//! bits come from here.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) use sm_game::actors::spore_spawn::*;

impl Host for Game {
    fn update_spore_stem(&mut self) {
        crate::game::enemy_projectiles::update_spore_stem(self);
    }
    fn boss_bits(&mut self, area: u16) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(area, self)
    }
}

// The named bank $A5 functions and instructions are the ones the species
// tables dispatch on.
const _: () = {
    use sm_game::actors::spore_spawn::{function as f, instruction as i};
    assert!(i::INCREASE_MAX_X_RADIUS == 10872671 && i::CLEAR_DAMAGED_FLAG == 10872689);
    assert!(i::SET_MAX_X_RADIUS_AND_ANGLE_DELTA == 10872877 && i::HARDEN == 10872956);
    assert!(i::SET_FUNCTION == 10873018 && 0xa5_0000 | f::NOTHING as u32 == 10873626);
    assert!(0xa5_0000 | f::DESCEND as u32 == 10873627 && 0xa5_0000 | f::ORBIT as u32 == 10873682);
    assert!(0xa5_0000 | f::BEGIN_RETURN as u32 == 10873755);
    assert!(0xa5_0000 | f::FINISH_RETURN as u32 == 10873838);
};
