//! A door's phase: where it stands in its cycle, with what each phase carries (its wait, and how
//! far an opening or a closing has gone), and the tables of which phases are shown and solid.

use core::num::NonZeroU16;

/// The frames before a door's next step (`Waiting`), at least 1: no run ends with a wait of 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wait(NonZeroU16);

impl Wait {
    /// A wait of this many frames, or none for 0, which no door ever holds.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        match NonZeroU16::new(frames) {
            Some(frames) => Some(Self(frames)),
            None => None,
        }
    }

    /// How many frames it is.
    #[must_use]
    pub const fn frames(self) -> u16 {
        self.0.get()
    }

    /// `Waiting`: the wait a run leaves, one less, while it is over 1; a wait of 1 has run out.
    const fn counted_down(self) -> Option<Self> {
        Self::new(self.frames().saturating_sub(1))
    }
}

/// How far an opening or a closing has gone: which of its four figures it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quarter {
    /// The first figure.
    First,
    /// The second figure.
    Second,
    /// The third figure.
    Third,
    /// The fourth and last figure.
    Fourth,
}

impl Quarter {
    /// `Opening`, `Closing`: the figure after this one, or none after the fourth, when the
    /// opening or closing is done.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::First => Some(Self::Second),
            Self::Second => Some(Self::Third),
            Self::Third => Some(Self::Fourth),
            Self::Fourth => None,
        }
    }
}

/// A door's phase, with what it carries. Listed alphabetically, as the spec lists them; the
/// order means nothing.
///
/// `Placed` and `Wall` carry no wait: each steps at every run it gets, which the spec writes as
/// a wait of 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Shown and solid, its closed figure; it looks for her each time its wait runs out.
    Closed {
        /// The frames before it next looks.
        wait: Wait,
    },
    /// Shown and solid, sliding shut, without looking for her.
    Closing {
        /// Which of the closing's figures it shows.
        quarter: Quarter,
        /// The frames before its next figure.
        wait: Wait,
    },
    /// Hidden and passable; it looks for her each time its wait runs out.
    Open {
        /// The frames before it next looks.
        wait: Wait,
    },
    /// Shown and solid, sliding open, without looking for her.
    Opening {
        /// Which of the opening's figures it shows.
        quarter: Quarter,
        /// The frames before its next figure.
        wait: Wait,
    },
    /// As the room placed it: shown and solid, with no figure; its first run starts it.
    Placed,
    /// Hidden and passable; it looks for her once, when its wait runs out, to choose its first
    /// phase.
    Starting {
        /// The frames before it looks.
        wait: Wait,
    },
    /// The shaft's door once Ridley has fled: hidden and solid, for good.
    Wall,
}

impl Phase {
    /// `Waiting`: the phase a run leaves when its wait is over 1, the same phase with one frame
    /// less to wait; none when the wait runs out, or the phase has none, and the door steps.
    #[must_use]
    pub fn counted_down(mut self) -> Option<Self> {
        let wait = self.wait_mut()?;
        *wait = wait.counted_down()?;
        Some(self)
    }

    /// The wait the phase carries, if it has one.
    const fn wait_mut(&mut self) -> Option<&mut Wait> {
        match self {
            Self::Closed { wait }
            | Self::Closing { wait, .. }
            | Self::Open { wait }
            | Self::Opening { wait, .. }
            | Self::Starting { wait } => Some(wait),
            Self::Placed | Self::Wall => None,
        }
    }

    /// `Shown`: whether the framework draws the door's figure. The phases that show it are the
    /// solid ones but the wall.
    #[must_use]
    pub const fn shown(self) -> bool {
        match self {
            Self::Closed { .. } | Self::Closing { .. } | Self::Opening { .. } | Self::Placed => {
                true
            }
            Self::Open { .. } | Self::Starting { .. } | Self::Wall => false,
        }
    }

    /// `Solid`: whether the door blocks her, her touch and her shots, as a solid enemy does. The
    /// open and starting doors let her through as air.
    #[must_use]
    pub const fn solid(self) -> bool {
        match self {
            Self::Closed { .. }
            | Self::Closing { .. }
            | Self::Opening { .. }
            | Self::Placed
            | Self::Wall => true,
            Self::Open { .. } | Self::Starting { .. } => false,
        }
    }
}
