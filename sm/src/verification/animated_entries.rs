//! Comparison-only entry points, accounted separately from runtime.
#![forbid(unsafe_code)]
use super::*;
pub(crate) fn command(address: u32, index: u16, cursor: u16, game: &mut Game) -> u16 {
    execution::command(&mut Runtime::new(game), address, index, cursor)
}
pub(crate) fn process_one(game: &mut Game) {
    execution::process(&mut Runtime::new(game));
}
pub(crate) fn guard(game: &mut Game) -> bool {
    crate::game::visual_runtime::check_animation_guard(game)
}
