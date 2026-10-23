//! Compatibility entry points for complete Spore/Maw attachment interactions.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) fn update_spore_stem(game: &mut Game) {
    super::with_execution(game, super::attachments_spore_maw::update_spore_stem);
}
pub(crate) fn update_maw_segment(part: u8, game: &mut Game) {
    super::with_execution(game, |ctx| {
        super::attachments_spore_maw::update_maw_segment(part, ctx)
    });
}
pub(crate) fn update_maw_palette(handle: u16, game: &mut Game) {
    super::with_execution(game, |ctx| {
        super::attachments_spore_maw::update_maw_palette(handle, ctx)
    });
}
