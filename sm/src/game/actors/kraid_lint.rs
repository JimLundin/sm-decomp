//! Kraid's lint contact runs in `sm_game::actors::kraid_lint`.
#![forbid(unsafe_code)]
use crate::game::actors::{self, api::DeathHandling};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::actors::kraid_lint::*;

impl Host for Game {
    fn contact_reaction_finalizing_death(&mut self) {
        actors::combat::contact_reaction(&mut Runtime::new(self), DeathHandling::Finalize);
    }
}
