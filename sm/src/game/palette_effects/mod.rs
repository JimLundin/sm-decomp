//! Palette effects using the shared gameplay Session and runtime.
pub(crate) use sm_game::palette_effects::execution;
pub(crate) mod torizo;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) use execution::leased::{advance, clear, set_enabled, spawn};

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
        // Only bank $8D holds palette commands; any other bank decodes as none.
        let handler = if ea >> 16 == 0x8d { ea as u16 } else { 0 };
        let (k, j) = execution::command(&mut Runtime::new(game), handler, k, j);
        crate::types::PairU16 { k, j }
    }
}
