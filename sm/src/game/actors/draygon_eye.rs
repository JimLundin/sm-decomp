//! Draygon's eye runs in `sm_game::actors::draygon_eye`; the actor lease for
//! Game is the compat Runtime.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::actors::draygon_eye::*;
use sm_game::actors::host::{ActorLease, LeaseActors};

impl ActorLease for Runtime<'_> {
    type Game = Game;

    fn word(&self, at: usize) -> u16 {
        Runtime::word(self, at)
    }

    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        Runtime::foreign(self, action)
    }
}

impl LeaseActors for Game {
    type Lease<'a> = Runtime<'a>;

    fn lease(&mut self) -> Runtime<'_> {
        Runtime::new(self)
    }
}
