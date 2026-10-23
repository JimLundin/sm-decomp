//! Animated graphics scripts and acknowledged graphics uploads.
mod compatibility;
mod execution;
mod native;
mod state;
#[path = "../ownership_transport.rs"]
mod transport;
use crate::game_state::Game;
use state::Upload;
type Owner<'g> = transport::Port<'g, state::AnimatedTiles>;
pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    let mut owner = Owner::acquire(game);
    owner.state.flags = if enabled {
        owner.state.flags | 0x8000
    } else {
        owner.state.flags & !0x8000
    };
}
pub(crate) fn clear(game: &mut Game) {
    let mut owner = Owner::acquire(game);
    for slot in (0..6).rev() {
        owner.state.slots[slot].id = 0;
    }
}
pub(crate) fn spawn(definition: u16, game: &mut Game) {
    execution::spawn(Owner::acquire(game), definition);
}
pub(crate) fn advance(game: &mut Game) {
    execution::advance(Owner::acquire(game));
}
pub(crate) fn upload_pending(game: &mut Game) {
    let mut owner = Owner::acquire(game);
    if owner.state.flags & 0x8000 == 0 {
        return;
    }
    for slot in (0..6).rev() {
        let Some(upload) = owner.state.pending_upload(slot) else {
            continue;
        };
        crate::verify_probe::hit("animated_tiles.upload.started");
        // No owned state survives the backend call. A failed transfer is not acknowledged.
        (owner, ()) = owner.foreign(|game| native::upload(upload, game));
        owner.state.acknowledge_upload(slot);
        crate::verify_probe::hit("animated_tiles.upload.acknowledged");
    }
}
// Comparison-only numeric entry points.
pub(crate) mod checks;
