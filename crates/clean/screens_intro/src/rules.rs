//! The intro's rules: what an adjusted game may change.

use types::intro::FadeFrames;

/// The intro's adjustable rules; `Rules::REFERENCE` is the game's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// `PictureDarkens`: the fade's rhythm `d`; each level holds `d + 1` frames.
    pub rhythm: FadeFrames,
}

impl Rules {
    /// The game's rules: the rhythm 1 frame.
    pub const REFERENCE: Self = Self {
        rhythm: FadeFrames::REFERENCE_RHYTHM,
    };
}
