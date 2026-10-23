//! Complete palette Session core with one explicit runtime compatibility edge.
mod content;
mod diagnostics;
mod execution;
mod runtime;
mod state;
use crate::game_state::Game;
use execution::Host;
use runtime::Runtime;

pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    Runtime::new(game).session_mut().set_enabled(enabled);
}
pub(crate) fn clear(game: &mut Game) {
    Runtime::new(game).session_mut().clear();
}
pub(crate) fn spawn(definition: u16, game: &mut Game) {
    execution::spawn(&mut Runtime::new(game), definition);
}
pub(crate) fn advance(game: &mut Game) {
    execution::advance(&mut Runtime::new(game));
}

/// Numeric identities remain private to the existing actual-code comparison carrier.
pub(crate) mod checks {
    pub(crate) fn guard(game: &mut crate::game_state::Game) -> bool {
        super::runtime::check_guard(game)
    }
    use super::*;
    pub(crate) fn setup(ea: u32, _k: u16, slot: u16, game: &mut Game) {
        execution::setup(&mut Runtime::new(game), ea, slot);
    }
    pub(crate) fn pre(ea: u32, k: u16, game: &mut Game) {
        execution::pre(&mut Runtime::new(game), ea, k);
    }
    pub(crate) fn process_one(k: u16, game: &mut Game) {
        execution::process(&mut Runtime::new(game), k);
    }
    pub(crate) fn command(ea: u32, k: u16, j: u16, game: &mut Game) -> crate::types::PairU16 {
        let (k, j) = execution::command(&mut Runtime::new(game), ea, k, j);
        crate::types::PairU16 { k, j }
    }
}
