//! Room-entry selectors held by the Session.

/// The four room asset bindings a room state names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetSlot {
    Level,
    TileTable,
    Tiles,
    Palette,
}
