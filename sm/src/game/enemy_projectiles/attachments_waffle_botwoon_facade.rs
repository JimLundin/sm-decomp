//! Compatibility entry points for complete Waffle/Botwoon interactions.
#![forbid(unsafe_code)]
use super::attachments_waffle_botwoon as behavior;
use crate::{game::session_runtime::Runtime, game_state::Game};
pub(crate) fn advance_waffle_orbit(game: &mut Game) {
    behavior::advance_waffle_orbit(&mut Runtime::new(game));
}
pub(crate) fn record_botwoon_head(game: &mut Game) {
    behavior::record_botwoon_head(&mut Runtime::new(game));
}
pub(crate) fn follow_botwoon_history(game: &mut Game) {
    behavior::follow_botwoon_history(&mut Runtime::new(game));
}
pub(crate) fn advance_botwoon_history(game: &mut Game) {
    behavior::advance_botwoon_history(&mut Runtime::new(game));
}
pub(crate) fn orient_botwoon_body(game: &mut Game) {
    behavior::orient_botwoon_body(&mut Runtime::new(game));
}
