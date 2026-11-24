//! The pad and the cover, one piece of the shaft as she arrives: the elevator's pad that holds
//! her as a new game starts, lowers her down the shaft to its floor and lets her go; and the
//! cover that hides the shaft's floor below it until she stands on it.
//!
//! The caller holds a [`PadAndCover`] between frames and steps it on the frame the arrival
//! places them ([`PadAndCover::arrival`]) and on each frame they run ([`PadAndCover::run`]). On
//! a frame they don't run the caller doesn't step them, and nothing of them changes.

use core::num::Wrapping;

use types::{Pair, Px};

use crate::look::AlternatingLook;
use crate::pad_wait::PadWait;
use crate::rules::Rules;

/// `PadPlaced`, `PadCarries`: the pad stands 28 px below her.
const PAD_BELOW_HER: Px = Wrapping(28);
/// `PadPlaced`: the cover stands 97 px down the room.
const COVER_Y: Px = Wrapping(97);
/// `PadCarries`: the pad lowers her one pixel a run.
const ONE_PIXEL: Px = Wrapping(1);

/// `PadFlickers`: the look a run that leaves the pad present gives it: `First` after none, and
/// the other look after either.
const fn flickered(look: Option<AlternatingLook>) -> AlternatingLook {
    match look {
        Some(look) => look.other(),
        None => AlternatingLook::First,
    }
}

/// The elevator's pad while it is present: she is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pad {
    /// The runs before it starts lowering her; 0 once it has.
    pub wait: PadWait,
    /// Where it is drawn, in whole pixels. Its x never changes.
    pub position: Pair<Px>,
    /// The pad's look, which of its two figures it shows; none before its first run.
    pub look: Option<AlternatingLook>,
}

/// A figure of the pad as the framework draws it: its look at its position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadFigure {
    /// The pad's look, which of its two figures.
    pub look: AlternatingLook,
    /// Where it is drawn.
    pub position: Pair<Px>,
}

/// What one run of the pad answers.
struct PadRun {
    /// The pad as the run leaves it; none once it has let her go.
    pad: Option<Pad>,
    /// Her height as the run leaves it.
    her_y: Px,
}

impl Pad {
    /// `PadPlaced`: the arrival puts the pad 28 px below her, with the whole wait and no look.
    fn place(her: Pair<Px>, wait: PadWait) -> Self {
        Self {
            wait,
            position: Pair {
                x: her.x,
                y: her.y + PAD_BELOW_HER,
            },
            look: None,
        }
    }

    /// One run of the pad, with her height after her own frame and the shaft's floor.
    ///
    /// `PadWaits`: while its wait is over 1 the run only counts it off. Otherwise the run lowers
    /// her (`PadCarries`): above the floor she goes down a pixel and the pad is drawn 28 px
    /// below where she stood; anywhere else she is set on the floor and the pad is gone.
    /// `PadFlickers`: a run that leaves the pad present turns its look.
    fn run(self, her_y: Px, floor: Px) -> PadRun {
        let look = Some(flickered(self.look));
        if let Some(wait) = self.wait.counted_down() {
            return PadRun {
                pad: Some(Self { wait, look, ..self }),
                her_y,
            };
        }
        if !above_floor(her_y, floor) {
            return PadRun {
                pad: None,
                her_y: floor,
            };
        }
        PadRun {
            pad: Some(Self {
                wait: PadWait::OVER,
                position: Pair {
                    x: self.position.x,
                    y: her_y + PAD_BELOW_HER,
                },
                look,
            }),
            her_y: her_y + ONE_PIXEL,
        }
    }

    /// The figure the framework draws, its look at its position; none before its first run.
    fn figure(self) -> Option<PadFigure> {
        self.look.map(|look| PadFigure {
            look,
            position: self.position,
        })
    }
}

/// `PadCarries`: whether her height is above the floor, by the 16-bit difference: `(y − f) mod
/// 2^16` is 32,768 or more, its bit 15 set. So a height past the floor (lower on the screen) by
/// 32,768 or more counts as above it, and is lowered as any other.
const fn above_floor(her_y: Px, floor: Px) -> bool {
    her_y.0.wrapping_sub(floor.0).cast_signed() < 0
}

/// The cover over the shaft's floor below the pad, while it is present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cover {
    /// Where it is drawn, in whole pixels.
    pub position: Pair<Px>,
}

impl Cover {
    /// `PadPlaced`: the arrival puts the cover under her, 97 px down the room.
    const fn place(her: Pair<Px>) -> Self {
        Self {
            position: Pair {
                x: her.x,
                y: COVER_Y,
            },
        }
    }

    /// `CoverHides`: a run of the cover with her height on the floor ends it; at any other
    /// height it stays where it is.
    fn run(self, her_y: Px, floor: Px) -> Option<Self> {
        (her_y != floor).then_some(self)
    }
}

/// The pad and the cover: the state a caller holds from one frame to the next. Each is present
/// or gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadAndCover {
    /// The elevator's pad; none once it has let her go.
    pub pad: Option<Pad>,
    /// The cover over the shaft's floor; none once she has stood on the floor.
    pub cover: Option<Cover>,
}

/// The figures a step of the pad and the cover reports for the framework to draw, each none
/// while it is gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadAndCoverFigures {
    /// The pad's look at its position; none before its first run.
    pub pad: Option<PadFigure>,
    /// Where the cover is drawn; none on the frame it is placed, until its first run.
    pub cover: Option<Pair<Px>>,
}

impl PadAndCoverFigures {
    /// `PadPlaced`: the frame the arrival places them draws neither.
    const NONE: Self = Self {
        pad: None,
        cover: None,
    };
}

/// What one step of the pad and the cover answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadAndCoverStep {
    /// The pad and the cover as the step leaves them.
    pub pad_and_cover: PadAndCover,
    /// Her height as the pad leaves it, which samus-movement takes as her position's y, and
    /// the view and the HUD see that frame.
    pub her_y: Px,
    /// What the framework draws of them this frame.
    pub figures: PadAndCoverFigures,
}

impl PadAndCover {
    /// `PadPlaced`: the arrival, with her at `her`, puts the pad under her and the cover below
    /// it, and she is held. Neither runs on this frame, so it draws neither: their first run
    /// is the next frame's.
    #[must_use]
    pub fn arrival(her: Pair<Px>, rules: &Rules) -> PadAndCoverStep {
        PadAndCoverStep {
            pad_and_cover: Self {
                pad: Some(Pad::place(her, rules.pad_wait)),
                cover: Some(Cover::place(her)),
            },
            her_y: her.y,
            figures: PadAndCoverFigures::NONE,
        }
    }

    /// One run of the pad and then the cover, after her own frame, with her height then. The
    /// pad runs first, so the cover sees her where the pad leaves her (`CoverHides`).
    #[must_use]
    pub fn run(self, her_y: Px, rules: &Rules) -> PadAndCoverStep {
        let carried = self.pad.map(|pad| pad.run(her_y, rules.floor));
        let her_y = carried.as_ref().map_or(her_y, |run| run.her_y);
        let pad_and_cover = Self {
            pad: carried.and_then(|run| run.pad),
            cover: self.cover.and_then(|cover| cover.run(her_y, rules.floor)),
        };
        PadAndCoverStep {
            pad_and_cover,
            her_y,
            figures: PadAndCoverFigures {
                pad: pad_and_cover.pad.and_then(Pad::figure),
                cover: pad_and_cover.cover.map(|cover| cover.position),
            },
        }
    }

    /// Whether she is held: while the pad is present, nothing of her moves or changes but what
    /// the pad does to her height.
    #[must_use]
    pub const fn holds_her(&self) -> bool {
        self.pad.is_some()
    }
}
