//! Palette effects using the shared gameplay Session and runtime.
pub(crate) use sm_game::palette_effects::execution;
pub(crate) mod fading;
pub(crate) mod torizo;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    execution::set_enabled(Runtime::new(game).session_mut(), enabled);
}
pub(crate) fn clear(game: &mut Game) {
    execution::clear(Runtime::new(game).session_mut());
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
        crate::game::visual_runtime::check_palette_guard(game)
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
