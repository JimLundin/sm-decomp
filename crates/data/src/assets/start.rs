//! The new game's start (docs/architecture/assets.md): the room a new game starts in and her
//! place across it.

use crate::assets::art::{Colour, ROW_COLOURS};
use crate::room::RoomKey;

/// Where a new game starts: at the Ceres station, before any file is saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewGame {
    /// The room she starts in.
    pub room: RoomKey,
    /// The view's place in the room as it loads, in pixels.
    pub view: (u16, u16),
    /// Her place in the room as it loads, in pixels.
    pub her: (u16, u16),
    /// The colours the fade in heads for in her row as it starts, which are not her suit's.
    pub her_row: [Colour; ROW_COLOURS],
}

/// One cell of the opening status bar, including its authored colour set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudCell {
    /// Picture and flips.
    pub look: types::Look,
    /// Palette index, 0 through 7.
    pub colours: u8,
}

/// The map looks in a new file's opening window, top row first.
pub type MapPreview = [[Option<types::Look>; 5]; 3];

/// The HUD art supplied by the new game's room setup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hud {
    /// Four rows of 32 cells, with the separately published upper row first.
    pub frame: [[HudCell; 32]; 4],
    /// The map pictures that the prepared room's first HUD draw shows.
    pub preview: MapPreview,
}
