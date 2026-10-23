//! Owned animated-tile execution with an explicit runtime compatibility edge.
mod content;
mod execution;
mod native;
mod runtime;
mod state;
use crate::game_state::Game;
use execution::Host;
use runtime::Runtime;
use state::Upload;
pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    Runtime::new(game).tiles_mut().set_enabled(enabled);
}
pub(crate) fn clear(game: &mut Game) {
    Runtime::new(game).tiles_mut().clear();
}
pub(crate) fn spawn(definition: u16, game: &mut Game) {
    execution::spawn(&mut Runtime::new(game), definition);
}
pub(crate) fn advance(game: &mut Game) {
    execution::advance(&mut Runtime::new(game));
}
pub(crate) fn upload_pending(game: &mut Game) {
    execution::upload_pending(&mut Runtime::new(game));
}
#[path = "../../verification/animated_entries.rs"]
pub(crate) mod checks;
