//! How long Ridley waits in the dark before he wakes: the wait as a tuning value, and the count
//! he keeps as he waits.

/// `RidleyWaits`: the wait, `w`: Ridley wakes on the `w + 1`-th run after the passage into his
/// room is over. A rule from 1 to 32,768 frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WakeWait(u16);

impl WakeWait {
    /// The longest wait the rule allows, in frames.
    pub const LONGEST: u16 = 32_768;

    /// A wait of this many frames, or none for 0 or past the longest.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames != 0 && frames <= Self::LONGEST {
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

    /// `RidleyWaits`: his wait as he starts waiting, `w − 1`: the run that starts it counts its
    /// first frame.
    #[must_use]
    pub const fn first_count(self) -> RidleyWait {
        RidleyWait(self.0.saturating_sub(1))
    }
}

/// His wait while he is `Waiting`: the runs left before the one he wakes on, 0 to `w − 1`, so
/// at most 32,767.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RidleyWait(u16);

impl RidleyWait {
    /// The longest wait he can hold, one less than the longest rule.
    pub const LONGEST: u16 = 32_767;

    /// A wait of this many runs, or none past the longest.
    #[must_use]
    pub const fn new(runs: u16) -> Option<Self> {
        if runs <= Self::LONGEST {
            Some(Self(runs))
        } else {
            None
        }
    }

    /// How many runs it is.
    #[must_use]
    pub const fn runs(self) -> u16 {
        self.0
    }

    /// `RidleyWaits`: the wait a run leaves, one less; none at 0, when he wakes.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            Some(runs) => Some(Self(runs)),
            None => None,
        }
    }
}
