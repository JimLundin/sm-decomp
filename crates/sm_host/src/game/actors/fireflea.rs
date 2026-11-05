//! The fireflea runs in `sm_game::actors::fireflea` on the scheduler's lease;
//! its death and its room darkening run on that lease.
#![forbid(unsafe_code)]
use crate::game::actors::{api::DeathKind, combat};
use crate::game::session_runtime::Runtime;
pub(crate) use sm_game::actors::fireflea::*;

impl Host for Runtime<'_> {
    fn die_normally(&mut self) {
        combat::die(self, DeathKind::Normal { animation: 0 });
    }

    fn darken_room(&mut self) {
        self.owned_player_call(sm_game::fx_lighting::darken_current);
    }
}
