//! What the arrival reads: whether the start is ready, and the start it arrives at.

use types::{Colour, Palette, Px};

/// The number of colours in her suit.
pub const SUIT_COLOURS: usize = 16;

/// Her suit's sixteen colours as it stands (samus-appearance's content), in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SuitColours(pub [Colour; SUIT_COLOURS]);

/// The new game's start, as the game has made it ready.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start {
    /// Her place across the start room, in pixels.
    pub x: Px,
    /// Her suit's colours.
    pub suit: SuitColours,
    /// The screen's goal colours as the start's loading leaves them: her block, 192 to 207,
    /// holds the start's own goal colours, the rest the room's.
    pub goal: Palette,
}

/// Whether the game has made the start ready this frame. How long that takes is the start
/// loading's, so the caller says so on the one frame it is done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preparation<'start> {
    /// The game is still making the start ready.
    Underway,
    /// The start is ready this frame.
    Ready(&'start Start),
}
