//! §1 What the domain remembers from one frame to the next: where her clip is, and her run's
//! boost. A caller may build each part at any value the spec allows and read it back, and keeps
//! it between frames as `update` left it: nothing else carries over.

use clean_types::BoostStage;

use crate::count::counted_down;
use crate::playhead::Playhead;

/// A number of laps of her run, 1 or more.
///
/// The laps a stage lasts (`BoostStages`), which an adjusted game may set to any number from 1;
/// or the laps a stage still lasts, the lap she is running included, from 1 to the laps the
/// stage lasts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Laps(pub(crate) u8);

impl Laps {
    /// The laps of `count`, if it is 1 or more.
    #[must_use]
    pub const fn new(count: u8) -> Option<Self> {
        if count > 0 { Some(Self(count)) } else { None }
    }

    /// How many laps.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0
    }

    /// Spec: `BoostStages`. The laps left run down by the lap she has run: what is left while
    /// that is above 0; none once the stage has lasted its laps.
    #[must_use]
    pub fn counted_down(self) -> Option<Self> {
        counted_down(self.0).map(Self)
    }
}

/// Her run's boost: its stage and the laps the stage still lasts (`BoostStages`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Boost {
    /// The boost stage, 0 to 4; 4 is boosting.
    pub stage: BoostStage,
    /// The laps left of the stage with the speed booster; none, 0 laps left, without it (§1.5).
    pub laps_left: Option<Laps>,
}

impl Boost {
    /// Spec: `BoostStages`. Her boost once a dash is lost: stage 0, with 0 laps left.
    pub const LOST: Self = Self {
        stage: BoostStage::Stage0,
        laps_left: None,
    };
}

/// The State: everything the domain remembers from one frame to the next (§1).
///
/// Her step, the time left, the boost stage and the laps left. The end, whether her spin has
/// turned far, where the clip of the pose she takes starts, the slowdowns and what her moves did
/// are each worked out within the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct State {
    /// Where her clip is.
    pub playhead: Playhead,
    /// Her run's boost.
    pub boost: Boost,
}
