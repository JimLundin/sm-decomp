//! Animated tiles compose on the shared Session at each legacy entry.
pub(crate) mod execution;
pub(crate) mod native;
use crate::{
    game::{session_access::SessionAccess, session_runtime::Runtime},
    game_state::Game,
};
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
pub(crate) fn upload_pending(game: &mut Game) {
    execution::upload_pending(&mut Runtime::new(game));
}
#[path = "../../verification/animated_entries.rs"]
pub(crate) mod checks;
