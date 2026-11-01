//! Room-entry selectors held by the Session.

/// The four room asset bindings a room state names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetSlot {
    Level,
    TileTable,
    Tiles,
    Palette,
}

/// Clear the door exit the next room entry follows (`DOOR_DEF_PTR`, a world
/// key) to key 0, as the cinematics do before they take over the screen: on
/// the retained Session's lease while one is current, so it stays current,
/// and through the host's projected bytes otherwise, as the named word store
/// did.
pub fn clear_exit<H>(game: &mut H)
where
    H: crate::ram::CompatibilityRam + crate::runtime_access::RuntimeAccess + ?Sized,
{
    if game.session_retained() {
        use crate::session_access::SessionAccess as _;
        let mut lease = game.runtime();
        let session = lease.session_mut();
        session.room.entry.exit = session.world.exit_identity(0);
    } else {
        crate::projection::set_word(game, crate::ram::words::DOOR_DEF_PTR, 0);
    }
}

pub mod api;
pub mod background;
pub mod content;
pub mod content_background;
pub mod content_state;
pub mod effects;
pub mod installation;
pub mod pack;
pub mod prepared_assets;
pub mod prepared_tables;
pub mod protocol;
pub mod selection;
pub mod state;
pub mod world;
