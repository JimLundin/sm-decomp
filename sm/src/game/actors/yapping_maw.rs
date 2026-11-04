//! Yapping Maw runs in `sm_game::actors::yapping_maw`; its main AI and its
//! multiplicand writes run from here.
#![forbid(unsafe_code)]
<<<<<<< HEAD
use crate::game::session_runtime::{Runtime, write_multiplicand};
=======
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
>>>>>>> 7deecc6fc (Hardware ports: the multipliers, device-only display commands, HDMA channel registers and the slope bus read leave foreign through a device interval [exact])
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
/// its function, then its body segments; the multiplicand writes go to the
/// multiplier port, and an unknown function's failure runs on the game.
pub(crate) fn run_main_ai(runtime: &mut Runtime<'_>) {
    let known = main_ai(runtime, |operand, runtime: &mut Runtime<'_>| {
        sm_game::hardware::Multiplier::cpu_multiplicand(runtime, operand);
    });
    if !known {
        crate::rtl::unreachable();
    }
    update_segments(runtime);
}
