//! The values an adjusted game may change.

use types::arrival::FadeFrames;

/// The arrival's adjustable values. `Rules::REFERENCE` is the game as it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `PictureBrightens`. The fade's rhythm, `d`: the count each step of the fade starts
    /// again from, so each level holds `d + 1` frames. The arrival starts the count here too.
    pub rhythm: FadeFrames,
}

impl Rules {
    /// The game as it is: a rhythm of 1 frame, so each level holds 2 frames.
    pub const REFERENCE: Self = Self {
        rhythm: FadeFrames::ONE,
    };
}
