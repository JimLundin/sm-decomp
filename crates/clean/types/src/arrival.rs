//! Her arrival at Ceres as a new game begins: the State a caller carries from frame to frame.

use serde::{Deserialize, Serialize};

use crate::brightness::Brightness;

/// The longest a fade's count or rhythm can be, in frames.
const MOST_FADE_FRAMES: u16 = 32_768;

/// A number of frames of the fade in, 0 to 32,768: the fade's count (the frames left before its
/// next step) or its rhythm (the count each step starts again from).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct FadeFrames(u16);

impl FadeFrames {
    /// No frames: the fade steps on its next frame.
    pub const NONE: Self = Self(0);
    /// One frame.
    pub const ONE: Self = Self(1);

    /// These many frames, if within 0 to 32,768.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames <= MOST_FADE_FRAMES {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// The number of frames.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// One frame fewer, or none once the count has run out.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            Some(frames) => Some(Self(frames)),
            None => None,
        }
    }
}

impl TryFrom<u16> for FadeFrames {
    type Error = &'static str;

    fn try_from(frames: u16) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("fade frames must be 0 to 32768")
    }
}

impl From<FadeFrames> for u16 {
    fn from(frames: FadeFrames) -> Self {
        frames.get()
    }
}

/// Where the arrival has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrivalStage {
    /// The game is making the start ready; the screen stays as it was.
    Preparing,
    /// She has arrived and the picture is coming up.
    FadingIn,
    /// The picture is fully up and she plays.
    Playing,
}

/// Everything the arrival remembers from one frame to the next. A caller may build it at any
/// value its types allow, and carry, save and restore it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalState {
    /// Where the arrival has got to.
    pub stage: ArrivalStage,
    /// The screen's brightness.
    pub brightness: Brightness,
    /// The frames left before the fade's next step.
    pub count: FadeFrames,
}
