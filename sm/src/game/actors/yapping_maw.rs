//! Yapping Maw runs in `sm_game::actors::yapping_maw`; its Samus control and
//! visibility come from here.
#![forbid(unsafe_code)]
use crate::game::session_runtime::{Runtime, write_multiplicand};
use crate::game_state::Game;
pub(crate) use sm_game::actors::yapping_maw::*;

// The named bank $A8 addresses are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::yapping_maw::instruction as i;
    assert!(i::OFFSET_SAMUS_UP_RIGHT == 11051207 && i::OFFSET_SAMUS_UP_LEFT == 11051225);
    assert!(i::OFFSET_SAMUS_DOWN_RIGHT == 11051243 && i::OFFSET_SAMUS_DOWN_LEFT == 11051261);
    assert!(i::OFFSET_SAMUS_UP == 11051279 && i::OFFSET_SAMUS_DOWN == 11051297);
    assert!(i::QUEUE_SOUND_IF_ON_SCREEN == 11051315);
};

/// Main AI - Yapping Maw ($A8:A211) on the scheduler's lease:
/// its function, then its body segments; the multiplicand writes and an
/// unknown function's failure run on the game.
pub(crate) fn run_main_ai(runtime: &mut Runtime<'_>) {
    let known = main_ai(runtime, |operand, runtime: &mut Runtime<'_>| {
        runtime.foreign(|game| write_multiplicand(operand, game));
    });
    if !known {
        crate::rtl::unreachable();
    }
    update_segments(runtime);
}

impl Host for Game {
    fn update_maw_visibility(&mut self, actor: i32) {
        Runtime::new(self).update_maw_visibility(actor);
    }
}
