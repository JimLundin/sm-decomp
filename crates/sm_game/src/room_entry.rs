//! Room-entry selectors held by the Session.

/// The four room asset bindings a room state names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetSlot {
    Level,
    TileTable,
    Tiles,
    Palette,
}

pub mod api;
pub mod background;
pub mod content;
pub mod content_background;
pub mod content_state;
pub mod effects;
pub mod installation;
pub mod prepared_assets;
pub mod prepared_tables;
pub mod protocol;
pub mod selection;
pub mod state;
pub mod world;
