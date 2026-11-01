//! Skultera runs in `sm_game::actors::skultera` on the scheduler's lease.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
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

impl Host for Runtime<'_> {
    fn unreachable(&mut self) {
        self.foreign(|game| crate::rtl::report_unreachable(game));
    }
}
