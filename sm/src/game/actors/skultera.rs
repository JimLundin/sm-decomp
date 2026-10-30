//! Skultera runs in `sm_game::actors::skultera`; its sine comes from here.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) use sm_game::actors::skultera::*;

// The named bank $A3 addresses are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::skultera::{function as f, instruction as i};
    const fn at(word: u16) -> u32 {
        0xa3_0000 | word as u32
    }
    assert!(at(f::SWIMMING_LEFT) == 0xa3_9132 && at(f::TURNING_LEFT) == 0xa3_9256);
    assert!(i::LAYER_6 == 10719382 && i::LAYER_2 == 10719392 && i::SET_TURN_FINISHED == 10719402);
};

impl Host for Game {
    fn actor_sine(&mut self, angle: u16, amplitude: u16) -> u16 {
        crate::game::session_runtime::actor_sine(angle, amplitude, self)
    }

    fn unreachable(&mut self) {
        crate::rtl::Unreachable(self);
    }
}
