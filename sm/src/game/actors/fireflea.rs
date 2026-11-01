//! The fireflea runs in `sm_game::actors::fireflea` on the scheduler's lease;
//! its death runs on that lease and its room darkening on the game.
#![forbid(unsafe_code)]
use crate::game::actors::{api::DeathKind, combat};
use crate::game::session_runtime::Runtime;
pub(crate) use sm_game::actors::fireflea::*;
use sm_game::room_visuals::darken_fireflea_room;

impl Host for Runtime<'_> {
    fn die_normally(&mut self) {
        combat::die(self, DeathKind::Normal { animation: 0 });
    }

    fn darken_room(&mut self) {
        self.foreign(darken_fireflea_room);
    }
}
