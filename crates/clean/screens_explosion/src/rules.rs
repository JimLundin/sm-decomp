//! The scene's adjustable values: the reference game's, or an adjusted game's.

use core::num;

use types::explosion::HoldFrames;

/// Spec: `SpaceHolds`. The reference hold, 192 frames.
const REFERENCE_HOLD_FRAMES: u16 = 192;

/// The scene's adjustable values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `SpaceHolds`. How many frames space holds after the ship disappears.
    pub hold_frames: HoldFrames,
}

impl Rules {
    /// The reference game's values.
    pub const REFERENCE: Self = Self {
        hold_frames: HoldFrames(match num::NonZeroU16::new(REFERENCE_HOLD_FRAMES) {
            Some(frames) => frames,
            None => num::NonZeroU16::MIN,
        }),
    };
}
