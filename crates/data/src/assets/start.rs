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
