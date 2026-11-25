//! His take-off: he crouches, hops up and to the left twice, then springs into the air.

use core::num::Wrapping;

use types::{Pair, Px};

use crate::flight::Speed;
use crate::roar::RidleyLook;
use crate::shown::{FigureLength, FigureSequence, NextFigure};

/// `TakesOff`: the lift-off speed, 352/256 px a run upward.
pub const LIFT_OFF: Pair<Speed> = Pair {
    x: Wrapping(0),
    y: Wrapping(-352),
};

/// One of the take-off's first four figures, each shown a fixed number of runs, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimedTakeOffFigure {
    /// Figure 1: at rest.
    First,
    /// Figure 2: his first crouch, his first hop as it starts.
    Second,
    /// Figure 3: his second crouch, his second hop as it starts.
    Third,
    /// Figure 4: his first crouch again, the lift-off as it starts.
    Fourth,
}

/// What happens as a figure of the take-off starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TakeOffStart {
    /// He hops by this many whole px, x rightward and y downward, each mod 2^16, his fractions
    /// kept; the room's bounds don't hold him until his next move.
    Hop(Pair<i16>),
    /// He springs into the air: `Rising`, at the lift-off speed.
    LiftOff,
}

/// A figure of the take-off, as the spec gives it.
struct TakeOffRow {
    /// Which of his figures is drawn.
    look: RidleyLook,
    /// How many runs it shows.
    runs: u8,
    /// What happens as it starts, if anything.
    start: Option<TakeOffStart>,
}

impl TimedTakeOffFigure {
    /// `TakesOff`: the take-off's figures with their lengths in runs: 1 `Rest` 3, 2 `TakeOff1`
    /// 4, 3 `TakeOff2` 5, 4 `TakeOff1` 17; the hops at the start of figure 2, (+1, −12) px, and
    /// of figure 3, (−4, −8) px; the lift-off at the start of figure 4.
    const fn row(self) -> TakeOffRow {
        match self {
            Self::First => TakeOffRow {
                look: RidleyLook::Rest,
                runs: 3,
                start: None,
            },
            Self::Second => TakeOffRow {
                look: RidleyLook::TakeOff1,
                runs: 4,
                start: Some(TakeOffStart::Hop(Pair { x: 1, y: -12 })),
            },
            Self::Third => TakeOffRow {
                look: RidleyLook::TakeOff2,
                runs: 5,
                start: Some(TakeOffStart::Hop(Pair { x: -4, y: -8 })),
            },
            Self::Fourth => TakeOffRow {
                look: RidleyLook::TakeOff1,
                runs: 17,
                start: Some(TakeOffStart::LiftOff),
            },
        }
    }
}

impl FigureLength for TimedTakeOffFigure {
    fn length(self) -> u8 {
        self.row().runs
    }
}

impl NextFigure for TimedTakeOffFigure {
    /// `TakesOff`: the figure that follows this one once it has shown its length; the fifth after
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

/// The take-off's figure, 1 to 5, with how many runs it has shown; the fifth, at rest, is held
/// from then on and has no length and no runs.
pub type TakeOffFigure = FigureSequence<TimedTakeOffFigure>;

impl TakeOffFigure {
    /// `TakesOff`: the take-off as it starts: its first figure, its first run.
    pub const START: Self = Self::starting(TimedTakeOffFigure::First);

    /// `TakesOff`: his look while it shows: `Rest`, `TakeOff1`, `TakeOff2`, `TakeOff1`, then
    /// `Rest`.
    #[must_use]
    pub const fn look(self) -> RidleyLook {
        match self {
            Self::Timed(shown) => shown.figure().row().look,
            Self::Fifth => RidleyLook::Rest,
        }
    }

    /// `TakesOff`: what happens on the run this figure starts: a hop or the lift-off at its
    /// first run, if its figure has one; nothing on any other run.
    pub(crate) const fn starts(self) -> Option<TakeOffStart> {
        match self {
            Self::Timed(shown) if shown.runs() == 1 => shown.figure().row().start,
            Self::Timed(_) | Self::Fifth => None,
        }
    }
}

/// `TakesOff`: his whole px after a hop, each axis mod 2^16.
pub const fn hopped(position: Pair<Px>, hop: Pair<i16>) -> Pair<Px> {
    Pair {
        x: Wrapping(position.x.0.wrapping_add_signed(hop.x)),
        y: Wrapping(position.y.0.wrapping_add_signed(hop.y)),
    }
}
