//! The flight's adjustable values. A `Rules` other than the reference is an adjusted game.

use types::zebes::HoldFrames;

/// The values an adjusted flight may change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `PlanetHolds`. How long the planet and stars stay still once the ship disappears.
    pub hold_frames: HoldFrames,
}

impl Rules {
    /// The reference game: a 64-frame hold.
    pub const REFERENCE: Self = Self {
        hold_frames: HoldFrames::REFERENCE,
    };
}
