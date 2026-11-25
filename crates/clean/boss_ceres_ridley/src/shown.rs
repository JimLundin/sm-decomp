//! A timed figure and how many runs it has been shown: the baby's figures, the roar's and the
//! take-off's each show for a fixed number of runs, then give way to the next. The roar and the
//! take-off each run through four timed figures, then hold a fifth.

/// A figure that shows for a fixed number of runs, its length.
pub trait FigureLength: Copy {
    /// How many runs it shows.
    fn length(self) -> u8;
}

/// One of four timed figures shown in order, before a fifth that is held.
pub trait NextFigure: FigureLength {
    /// The timed figure after this one; none after the fourth, when the fifth is held.
    fn next(self) -> Option<Self>;
}

/// Where a run of four timed figures and a held fifth has got to: one of the four, with how many
/// runs it has shown, or the fifth, which has no length and no runs (`Roars`, `TakesOff`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FigureSequence<F> {
    /// One of the first four, with its runs.
    Timed(ShownFigure<F>),
    /// The fifth, held from then on.
    Fifth,
}

impl<F: NextFigure> FigureSequence<F> {
    /// This timed figure at its first run.
    pub(crate) const fn starting(figure: F) -> Self {
        Self::Timed(ShownFigure::first_run(figure))
    }

    /// `Roars`, `TakesOff`: the figure a run moves on to: a figure shown fewer runs than its
    /// length shows one run more, then gives way to the next at its first run; after the fourth
    /// the fifth, which is held.
    #[must_use]
    pub fn ran(self) -> Self {
        match self {
            Self::Timed(shown) => shown.one_run_more().map_or_else(
                || shown.figure().next().map_or(Self::Fifth, Self::starting),
                Self::Timed,
            ),
            Self::Fifth => Self::Fifth,
        }
    }
}

/// A figure and how many runs it has shown so far, 1 to its length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShownFigure<F> {
    /// The figure.
    figure: F,
    /// The runs it has shown.
    runs: u8,
}

impl<F: FigureLength> ShownFigure<F> {
    /// The figure shown this many runs, or none for 0 or past its length.
    #[must_use]
    pub fn new(figure: F, runs: u8) -> Option<Self> {
        (runs != 0 && runs <= figure.length()).then_some(Self { figure, runs })
    }

    /// The figure at its first run.
    pub(crate) const fn first_run(figure: F) -> Self {
        Self { figure, runs: 1 }
    }

    /// The figure.
    #[must_use]
    pub const fn figure(self) -> F {
        self.figure
    }

    /// The runs it has shown.
    #[must_use]
    pub const fn runs(self) -> u8 {
        self.runs
    }

    /// `BabyFidgets`, `Roars`, `TakesOff`: the figure shown one run more, while it has shown fewer than its
    /// length; none once it has shown them all, when it gives way to the next.
    pub(crate) fn one_run_more(self) -> Option<Self> {
        self.runs
            .checked_add(1)
            .and_then(|runs| Self::new(self.figure, runs))
    }
}
