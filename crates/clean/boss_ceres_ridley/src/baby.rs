//! The baby Ridley holds as she enters his room: it fidgets through its figures, ten runs
//! each, and at the end of each fidget, one time in two, it cries and glows.

use core::num::Wrapping;

use enum_map::{Array, Enum};
use types::{Pair, Px};

use crate::shown::{FigureLength, ShownFigure};

/// `BabyHeld`: the baby is held 16 px left of him.
const HOLD_LEFT: Px = Wrapping(16);
/// `BabyHeld`: and 22 px down.
const HOLD_DOWN: Px = Wrapping(22);
/// `BabyFidgets`: each figure of the fidget shows for 10 runs.
const FIDGET_RUNS: u8 = 10;
/// `BabyCries`: the random number's lowest bit, set when it is odd.
const ODD_BIT: u16 = 1;

/// Which of the baby's three figures is drawn. Which sprites each is belongs to the baby's
/// content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BabyLook {
    /// The first figure.
    First,
    /// The second.
    Second,
    /// The third.
    Third,
}

/// One of the baby's colour sets, which the screen shows in the baby's palette at once (the
/// renderer). Listed alphabetically; the order means nothing. The colours of each are content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BabyColours {
    /// The glow's first set.
    Glow1,
    /// Its second.
    Glow2,
    /// Its third.
    Glow3,
    /// The baby's colours at rest, back once the glow is over.
    Rest,
}

/// The baby's cry, asked of audio with a queue limit of 6 (`BabyCries`). What the queue does
/// with a request it can't hold is audio's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BabyCry;

impl BabyCry {
    /// `BabyCries`: the queue limit the cry is asked for with.
    pub const QUEUE_LIMIT: u8 = 6;
}

/// One of the fidget's eight figures, `Fidget(0)` to `Fidget(7)`, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum FidgetFigure {
    /// `Fidget(0)`.
    Zero,
    /// `Fidget(1)`.
    One,
    /// `Fidget(2)`.
    Two,
    /// `Fidget(3)`.
    Three,
    /// `Fidget(4)`.
    Four,
    /// `Fidget(5)`.
    Five,
    /// `Fidget(6)`.
    Six,
    /// `Fidget(7)`.
    Seven,
}

impl FidgetFigure {
    /// `BabyFidgets`: its look: `First`, `Second`, `Third`, `Second`, twice over.
    const fn look(self) -> BabyLook {
        match self {
            Self::Zero | Self::Four => BabyLook::First,
            Self::One | Self::Three | Self::Five | Self::Seven => BabyLook::Second,
            Self::Two | Self::Six => BabyLook::Third,
        }
    }
}

/// One of the glow's twelve figures, `Glow(0)` to `Glow(11)`, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum GlowFigure {
    /// `Glow(0)`.
    Zero,
    /// `Glow(1)`.
    One,
    /// `Glow(2)`.
    Two,
    /// `Glow(3)`.
    Three,
    /// `Glow(4)`.
    Four,
    /// `Glow(5)`.
    Five,
    /// `Glow(6)`.
    Six,
    /// `Glow(7)`.
    Seven,
    /// `Glow(8)`.
    Eight,
    /// `Glow(9)`.
    Nine,
    /// `Glow(10)`.
    Ten,
    /// `Glow(11)`.
    Eleven,
}

/// A figure of the glow, as the spec gives it.
struct GlowRow {
    /// Which of the baby's figures is drawn.
    look: BabyLook,
    /// How many runs it shows.
    runs: u8,
    /// The colour set the screen takes as it starts.
    colours: BabyColours,
}

impl GlowFigure {
    /// `BabyCries`: the glow's figures, lasting 6, 5, 4, 3, 2, 3, 4, 5, 6, 7, 8 and 9 runs; their
    /// looks `First`, `Second`, `Third`, `Second` and their colour sets `Glow1`, `Glow2`,
    /// `Glow3`, `Glow2`, each three times over. Its first nine figures are the same either side
    /// of the fifth, `Glow(4)`.
    const fn row(self) -> GlowRow {
        match self {
            Self::Zero | Self::Eight => GlowRow {
                look: BabyLook::First,
                runs: 6,
                colours: BabyColours::Glow1,
            },
            Self::One | Self::Seven => GlowRow {
                look: BabyLook::Second,
                runs: 5,
                colours: BabyColours::Glow2,
            },
            Self::Two | Self::Six => GlowRow {
                look: BabyLook::Third,
                runs: 4,
                colours: BabyColours::Glow3,
            },
            Self::Three | Self::Five => GlowRow {
                look: BabyLook::Second,
                runs: 3,
                colours: BabyColours::Glow2,
            },
            Self::Four => GlowRow {
                look: BabyLook::First,
                runs: 2,
                colours: BabyColours::Glow1,
            },
            Self::Nine => GlowRow {
                look: BabyLook::Second,
                runs: 7,
                colours: BabyColours::Glow2,
            },
            Self::Ten => GlowRow {
                look: BabyLook::Third,
                runs: 8,
                colours: BabyColours::Glow3,
            },
            Self::Eleven => GlowRow {
                look: BabyLook::Second,
                runs: 9,
                colours: BabyColours::Glow2,
            },
        }
    }
}

/// The figure after this one in its routine's order, or none after the last.
fn following<F: Enum>(figure: F) -> Option<F> {
    figure
        .into_usize()
        .checked_add(1)
        .filter(|&next| next < <F::Array<()> as Array>::LENGTH)
        .map(F::from_usize)
}

/// A figure of the baby's routine: one of the fidget's or one of the glow's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BabyFigure {
    /// `Fidget(i)`.
    Fidget(FidgetFigure),
    /// `Glow(j)`.
    Glow(GlowFigure),
}

impl BabyFigure {
    /// Which of the baby's figures it draws.
    #[must_use]
    pub const fn look(self) -> BabyLook {
        match self {
            Self::Fidget(figure) => figure.look(),
            Self::Glow(figure) => figure.row().look,
        }
    }

    /// The turn the routine takes once this figure has shown its length.
    ///
    /// `BabyFidgets`: a fidget's figure gives way to the next. `BabyCries`: after the last,
    /// one time in two, when the random number is even, the baby cries and starts to glow;
    /// otherwise it fidgets again, quiet. A glow's figure gives way to the next, with its
    /// colour set; after the last the baby's colours are back at rest and it fidgets again.
    fn after(self, random: u16) -> BabyTurn {
        match self {
            Self::Fidget(figure) => following(figure).map_or_else(
                || fidget_over(random),
                |next| BabyTurn::quiet(Self::Fidget(next)),
            ),
            Self::Glow(figure) => following(figure).map_or(
                BabyTurn {
                    routine: BabyRoutine::starting(Self::Fidget(FidgetFigure::Zero)),
                    cry: None,
                    colours: Some(BabyColours::Rest),
                },
                BabyTurn::glowing,
            ),
        }
    }
}

impl FigureLength for BabyFigure {
    /// `BabyFidgets`, `BabyCries`: a fidget's figure shows 10 runs; a glow's, its row's.
    fn length(self) -> u8 {
        match self {
            Self::Fidget(_) => FIDGET_RUNS,
            Self::Glow(figure) => figure.row().runs,
        }
    }
}

/// `BabyCries`: the end of a fidget: with an odd random number the fidget starts again, quiet;
/// with an even one the baby cries and starts to glow.
const fn fidget_over(random: u16) -> BabyTurn {
    if random & ODD_BIT == ODD_BIT {
        BabyTurn::quiet(BabyFigure::Fidget(FidgetFigure::Zero))
    } else {
        BabyTurn {
            cry: Some(BabyCry),
            ..BabyTurn::glowing(GlowFigure::Zero)
        }
    }
}

/// The baby's routine: where it is in its fidget or its glow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BabyRoutine {
    /// Before its first run.
    New,
    /// Showing one of its figures.
    Showing(ShownFigure<BabyFigure>),
}

impl BabyRoutine {
    /// The routine at its figure's first run.
    const fn starting(figure: BabyFigure) -> Self {
        Self::Showing(ShownFigure::first_run(figure))
    }

    /// Which of its figures the baby draws; none while it is new.
    #[must_use]
    pub const fn look(self) -> Option<BabyLook> {
        match self {
            Self::New => None,
            Self::Showing(shown) => Some(shown.figure().look()),
        }
    }

    /// One run of the routine. `BabyFidgets`: its first run starts the fidget's first figure;
    /// a figure shown fewer runs than its length shows one run more; then the routine turns.
    fn run(self, random: u16) -> BabyTurn {
        match self {
            Self::New => BabyTurn::quiet(BabyFigure::Fidget(FidgetFigure::Zero)),
            Self::Showing(shown) => shown.one_run_more().map_or_else(
                || shown.figure().after(random),
                |shown| BabyTurn {
                    routine: Self::Showing(shown),
                    cry: None,
                    colours: None,
                },
            ),
        }
    }
}

/// What one run of the routine leaves: the routine, and the cry and the colours it hands on.
struct BabyTurn {
    /// The routine as the run leaves it.
    routine: BabyRoutine,
    /// The cry, on the run the glow starts.
    cry: Option<BabyCry>,
    /// The colour set the screen takes, on the runs a glow's figure starts or the glow ends.
    colours: Option<BabyColours>,
}

impl BabyTurn {
    /// A turn to this figure's first run that hands on nothing.
    const fn quiet(figure: BabyFigure) -> Self {
        Self {
            routine: BabyRoutine::starting(figure),
            cry: None,
            colours: None,
        }
    }

    /// A turn to this glow figure's first run, with its colour set.
    const fn glowing(figure: GlowFigure) -> Self {
        Self {
            routine: BabyRoutine::starting(BabyFigure::Glow(figure)),
            cry: None,
            colours: Some(figure.row().colours),
        }
    }
}

/// The baby: the state a caller holds, within Ridley's, from one frame to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Baby {
    /// Where it is in its fidget or its glow.
    pub routine: BabyRoutine,
    /// Where it is drawn, in whole pixels; none before its first run.
    pub position: Option<Pair<Px>>,
}

/// What one run of the baby leaves and hands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BabyRun {
    /// The baby as the run leaves it.
    pub baby: Baby,
    /// The baby's cry, if it starts to cry this run.
    pub cry: Option<BabyCry>,
    /// The colour set the screen takes in the baby's palette, if any.
    pub colours: Option<BabyColours>,
}

impl Baby {
    /// `RidleyPlaced`: the baby as Ridley is placed, before its first run.
    pub const NEW: Self = Self {
        routine: BabyRoutine::New,
        position: None,
    };

    /// One run of the baby, held by Ridley at `ridley`, with the frame's random number.
    /// `BabyHeld`: every run puts it in his hold, 16 px left of him and 22 px down.
    #[must_use]
    pub fn run(self, ridley: Pair<Px>, random: u16) -> BabyRun {
        let turn = self.routine.run(random);
        BabyRun {
            baby: Self {
                routine: turn.routine,
                position: Some(Pair {
                    x: ridley.x - HOLD_LEFT,
                    y: ridley.y + HOLD_DOWN,
                }),
            },
            cry: turn.cry,
            colours: turn.colours,
        }
    }
}
