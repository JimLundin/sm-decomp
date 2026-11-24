//! The pad's wait: how long the elevator's pad holds her still before it lowers her. It is both
//! a tuning value and the count the pad keeps as it runs.

/// The runs the pad holds her still before it lowers her (`PadWaits`): 0 to 32,768. The pad's
/// own wait counts down to 0 and stays there once it lowers her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadWait(u16);

impl PadWait {
    /// `PadWaits`: the longest wait the rule allows, in frames.
    pub const LONGEST: u16 = 32_768;
    /// The wait of a pad that is lowering her.
    pub const OVER: Self = Self(0);

    /// A wait of this many frames, or none past the longest.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames <= Self::LONGEST {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// How many frames it is.
    #[must_use]
    pub const fn frames(self) -> u16 {
        self.0
    }

    /// `PadWaits`: the wait a run leaves while it is over 1, one less; none when it is 1 or 0,
    /// and the pad lowers her in this run.
    pub(crate) fn counted_down(self) -> Option<Self> {
        self.0.checked_sub(1).filter(|&left| left != 0).map(Self)
    }
}
