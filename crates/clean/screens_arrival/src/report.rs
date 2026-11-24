//! What a frame of the arrival reports to the other parts of the game.

use types::brightness::Brightness;
use types::{Facing, Pair, Palette, Px};

/// Spec: `ArrivesAtTheTop`. She arrives: reported on the arrival's frame only.
///
/// She stands still at `place`, with no speed, facing the way `facing` says (samus-movement,
/// samus-appearance). The pad is placed with her at the same `place` (boss-ceres-ridley's
/// `PadPlaced`), and the mini-map is shown (interface-hud).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrival {
    /// Her place, in pixels: the start's x, at the top of the room.
    pub place: Pair<Px>,
    /// The way she faces: the screen.
    pub facing: Facing,
    /// The screen's colours as they now stand. Their goal is left as it was.
    pub colours: Palette,
}

/// A frame's outputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Her arrival, on the frame she arrives; none on every other frame.
    pub arrival: Option<Arrival>,
    /// The screen's brightness, for the renderer.
    pub brightness: Brightness,
    /// Whether she plays: from the next frame the game plays as in any room.
    pub play_begins: bool,
}
