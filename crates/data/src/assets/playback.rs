//! The intro's three recorded pad timelines.

use core::num::NonZeroU16;

pub use types::playback::{PadAction, PadSample};

/// One timed hold in the recorded pad.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadHold {
    /// Room actor opportunities in this hold.
    pub turns: NonZeroU16,
    /// Held buttons and the fresh presses on the first opportunity.
    pub sample: PadSample,
}

/// The holds as they play in each scene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePad {
    /// The fight on Zebes.
    pub battle: Vec<PadHold>,
    /// The approach to the baby.
    pub approach: Vec<PadHold>,
    /// The watch by the baby.
    pub watch: Vec<PadHold>,
}
