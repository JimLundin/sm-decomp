//! A timed figure and how many runs it has been shown: the baby's figures and the roar's
//! figures each show for a fixed number of runs, then give way to the next.

/// A figure that shows for a fixed number of runs, its length.
pub trait FigureLength: Copy {
    /// How many runs it shows.
    fn length(self) -> u8;
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

    /// `BabyFidgets`, `Roars`: the figure shown one run more, while it has shown fewer than its
    /// length; none once it has shown them all, when it gives way to the next.
    pub(crate) fn one_run_more(self) -> Option<Self> {
        self.runs
            .checked_add(1)
            .and_then(|runs| Self::new(self.figure, runs))
    }
}
