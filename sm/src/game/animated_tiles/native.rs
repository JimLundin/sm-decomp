//! Qualified legacy diagnostics.
use crate::game_state::Game;
pub(crate) fn unreachable(game: &mut Game) -> u16 {
    crate::rtl::report_unreachable(game) as u16
}
