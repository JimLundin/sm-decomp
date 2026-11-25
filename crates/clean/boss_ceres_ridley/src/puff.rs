//! A vent's puff: the way it blows, and its seven figures, from a small puff at the vent to a
//! thin one far from it, each held for a few runs.

/// `VentPuffs`: a puff shows seven figures.
const FIGURES: u8 = 7;
/// `VentPuffs`: each figure's hold, the runs it shows for: three.
const FIGURE_HOLD: u8 = 3;

/// The way a vent's puff blows. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VentFacing {
    /// Downward, out of a ceiling.
    Down,
    /// Leftward, out of a wall.
    Left,
    /// Rightward, out of a wall.
    Right,
    /// Upward, out of a floor.
    Up,
}

/// Which of the puff's seven figures shows: 1, a small puff at the vent, to 7, thinned away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PuffFigure(u8);

impl PuffFigure {
    /// The puff's first figure.
    pub const FIRST: Self = Self(1);

    /// The figure of this number, or none outside 1 to 7.
    #[must_use]
    pub const fn new(figure: u8) -> Option<Self> {
        if figure >= 1 && figure <= FIGURES {
            Some(Self(figure))
        } else {
            None
        }
    }

    /// Its number, 1 to 7.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The figure after it; none after the seventh.
    const fn next(self) -> Option<Self> {
        Self::new(self.0.saturating_add(1))
    }
}

/// A puff out of its vent: the figure it shows and that figure's remaining hold, the runs it is
/// still shown for, 1 to 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Puff {
    figure: PuffFigure,
    remaining_hold: u8,
}

impl Puff {
    /// `VentWaits`, `VentRests`: a puff as it comes, its first figure with its whole hold.
    pub(crate) const COMES: Self = Self {
        figure: PuffFigure::FIRST,
        remaining_hold: FIGURE_HOLD,
    };

    /// The puff showing `figure` with `remaining_hold` runs of it to go; none outside 1 to 3.
    #[must_use]
    pub const fn new(figure: PuffFigure, remaining_hold: u8) -> Option<Self> {
        if remaining_hold >= 1 && remaining_hold <= FIGURE_HOLD {
            Some(Self {
                figure,
                remaining_hold,
            })
        } else {
            None
        }
    }

    /// The figure it shows.
    #[must_use]
    pub const fn figure(self) -> PuffFigure {
        self.figure
    }

    /// Its figure's remaining hold: the runs the figure is still shown for, 1 to 3.
    #[must_use]
    pub const fn remaining_hold(self) -> u8 {
        self.remaining_hold
    }

    /// `VentPuffs`: the puff after one run: its figure's remaining hold one run less while more
    /// than one run of it remains; then the next figure, with its whole hold; none once the
    /// seventh's hold is over, as the puff is gone.
    pub(crate) const fn ran(self) -> Option<Self> {
        let remaining_hold = self.remaining_hold.saturating_sub(1);
        if remaining_hold >= 1 {
            return Some(Self {
                remaining_hold,
                ..self
            });
        }
        match self.figure.next() {
            Some(figure) => Some(Self {
                figure,
                remaining_hold: FIGURE_HOLD,
            }),
            None => None,
        }
    }
}
