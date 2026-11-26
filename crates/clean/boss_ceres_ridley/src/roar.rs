//! Ridley's looks, and his roar: he opens his jaws, holds them wide, shuts them and sits.

use crate::shown::{FigureLength, FigureSequence, NextFigure};

/// The figure he is drawn with, facing left.
///
/// These are labels, not an order: `RoarFigure` orders the looks of his roar, and
/// `TakeOffFigure` orders the looks of his take-off.
/// Which sprites make up a look belongs to his content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RidleyLook {
    /// At rest.
    Rest,
    /// His jaws open.
    Open,
    /// His jaws wide, roaring.
    Wide,
    /// His take-off's first crouch.
    TakeOff1,
    /// His take-off's second crouch.
    TakeOff2,
}

/// One of the roar's first four figures, each shown a fixed number of runs, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimedRoarFigure {
    /// Figure 1: at rest, before he opens his jaws.
    First,
    /// Figure 2: his jaws open, the roar heard as it starts.
    Second,
    /// Figure 3: his jaws wide.
    Third,
    /// Figure 4: his jaws open again, shutting.
    Fourth,
}

/// A figure of the roar, as the spec gives it.
struct RoarRow {
    /// Which of his figures is drawn.
    look: RidleyLook,
    /// How many runs it shows.
    runs: u8,
    /// Whether he is roaring while it shows.
    roaring: bool,
}

impl TimedRoarFigure {
    /// `Roars`: the roar's figures with their lengths in runs: 1 `Rest` 6, 2 `Open` 8, 3 `Wide`
    /// 96, 4 `Open` 8, the second and the fourth alike; he is roaring from the second to the
    /// fourth.
    const fn row(self) -> RoarRow {
        match self {
            Self::First => RoarRow {
                look: RidleyLook::Rest,
                runs: 6,
                roaring: false,
            },
            Self::Second | Self::Fourth => RoarRow {
                look: RidleyLook::Open,
                runs: 8,
                roaring: true,
            },
            Self::Third => RoarRow {
                look: RidleyLook::Wide,
                runs: 96,
                roaring: true,
            },
        }
    }
}

impl FigureLength for TimedRoarFigure {
    fn length(self) -> u8 {
        self.row().runs
    }
}

impl NextFigure for TimedRoarFigure {
    /// `Roars`: the figure that follows this one once it has shown its length; the fifth after
    /// the fourth.
    fn next(self) -> Option<Self> {
        match self {
            Self::First => Some(Self::Second),
            Self::Second => Some(Self::Third),
            Self::Third => Some(Self::Fourth),
            Self::Fourth => None,
        }
    }
}

/// The roar's figure, 1 to 5, with how many runs it has shown; the fifth, at rest, is held for
/// the rest of the roar and has no length and no runs.
pub type RoarFigure = FigureSequence<TimedRoarFigure>;

/// `Roars`: the second figure at its first run, the run the roar is heard.
const ROAR_HEARD: RoarFigure = RoarFigure::starting(TimedRoarFigure::Second);

impl RoarFigure {
    /// `ReadyToRoar`: the roar as it starts: its first figure, its first run.
    pub const START: Self = Self::starting(TimedRoarFigure::First);

    /// `Roars`: his look while it shows: `Rest`, `Open`, `Wide`, `Open`, then `Rest`.
    #[must_use]
    pub const fn look(self) -> RidleyLook {
        match self {
            Self::Timed(shown) => shown.figure().row().look,
            Self::Fifth => RidleyLook::Rest,
        }
    }

    /// Whether he is roaring while it shows: figures 2, 3 and 4.
    #[must_use]
    pub const fn roaring(self) -> bool {
        match self {
            Self::Timed(shown) => shown.figure().row().roaring,
            Self::Fifth => false,
        }
    }

    /// `Roars`: whether this is the run the roar is heard: the second figure's first.
    #[must_use]
    pub fn heard(self) -> bool {
        self == ROAR_HEARD
    }
}
