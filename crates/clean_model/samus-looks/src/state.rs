//! §1 What the domain remembers from one frame to the next: where her clip is, and her run's
//! boost. A caller may build each part at any value the spec allows and read it back.

use clean_types::BoostStage;

use crate::clip::{Clip, Frames, Step};

/// Where her clip is: her step, and the time it has left.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Playhead {
    /// Her step: the picture she shows (part 2).
    pub step: Step,
    /// The time left: how long her step still shows, 0 to 255 frames.
    pub left: Frames,
}

impl Playhead {
    /// `ClipPlays`: the time run down by 1, while it stays above 0 on the same step; none when it
    /// runs out and the clip moves on.
    #[must_use]
    pub fn counted_down(self) -> Option<Self> {
        let left = self.left.0.checked_sub(1).filter(|&left| left > 0)?;
        Some(Self {
            left: Frames(left),
            ..self
        })
    }

    /// `ClipEnds`, the frame's first half (§6): the end her clip reaches this frame, from her
    /// step, the time left and her clip at the frame's start, before samus-motion picks her
    /// pose. §1.5 makes it exact: no move of the frame changes it.
    #[must_use]
    pub fn end_ahead(self, clip: &Clip) -> Option<clean_types::ClipEnd> {
        if self.counted_down().is_some() {
            return None;
        }
        clip.after(self.step)?.1.end()
    }
}

/// How many laps of her run the boost stage still lasts: 1 or 2 with the speed booster, 0
/// without it (§1.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Laps(pub u8);

/// Her run's boost: its stage and the laps the stage still lasts (`BoostStages`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Boost {
    /// The boost stage, 0 to 4; 4 is boosting.
    pub stage: BoostStage,
    /// The laps left.
    pub laps: Laps,
}

/// The State: everything the domain remembers from one frame to the next.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct State {
    /// Where her clip is.
    pub playhead: Playhead,
    /// Her run's boost.
    pub boost: Boost,
}
