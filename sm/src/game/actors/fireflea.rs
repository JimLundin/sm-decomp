//! The fireflea runs in `sm_game::actors::fireflea`; its trigonometry, death
//! and room darkening come from here.
#![forbid(unsafe_code)]
use crate::game::actors::{api::DeathKind, combat};
use crate::game::session_runtime::{Runtime, actor_cosine, actor_sine};
use crate::game_state::Game;
pub(crate) use sm_game::actors::fireflea::*;
use sm_game::room_visuals::darken_fireflea_room;

impl Host for Game {
    fn actor_cosine(&mut self, angle: u16, amplitude: u16) -> u16 {
        actor_cosine(angle, amplitude, self)
    }

    fn actor_sine(&mut self, angle: u16, amplitude: u16) -> u16 {
        actor_sine(angle, amplitude, self)
    }

    fn die_normally(&mut self) {
        combat::die(&mut Runtime::new(self), DeathKind::Normal { animation: 0 });
    }

    fn darken_room(&mut self) {
        darken_fireflea_room(self);
    }
}
