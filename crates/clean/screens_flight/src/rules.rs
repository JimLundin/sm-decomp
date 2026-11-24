//! The flight's adjustable rules.

use types::flight::FadeFrames;

/// The numbers of the flight a caller may adjust. A `Rules` other than `REFERENCE` is an
/// adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `PictureDarkens`. The fade's rhythm `d`: each level holds `d + 1` frames. The fade's
    /// count starts at it as the label ends (`LabelIsTyped`) and again at each darkening.
    pub rhythm: FadeFrames,
}

impl Rules {
    /// The game as it is: each level of the fade holds 2 frames.
    pub const REFERENCE: Self = Self {
        rhythm: FadeFrames::ONE,
    };
}
