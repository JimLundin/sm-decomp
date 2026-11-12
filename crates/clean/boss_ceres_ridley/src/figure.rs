//! The figures a door holds: which one each phase shows, for each way the door faces.

use types::Side;

use crate::phase::{Phase, Quarter};

/// How far open a figure shows the door, from closed to open. Which sprites each is belongs to
/// the doors' content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FigureLook {
    /// Shut.
    Closed,
    /// The first figure past closed.
    Ajar1,
    /// The second.
    Ajar2,
    /// The third.
    Ajar3,
    /// Wide open.
    Open,
}

/// One of the doors' ten figures: a look, drawn for a door facing left or right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Figure {
    /// How far open it shows the door.
    pub look: FigureLook,
    /// The side it is drawn for.
    pub facing: Side,
}

/// A phase's figure for a door facing each way.
pub struct Figures {
    /// For a door facing left.
    left: Option<Figure>,
    /// For a door facing right.
    right: Option<Figure>,
}

impl Figures {
    /// `Placed`: no figure, whichever way the door faces.
    const NONE: Self = Self {
        left: None,
        right: None,
    };

    /// The look, facing the door's way: what most phases hold.
    const fn facing_its_way(look: FigureLook) -> Self {
        Self {
            left: Some(Figure {
                look,
                facing: Side::Left,
            }),
            right: Some(Figure {
                look,
                facing: Side::Right,
            }),
        }
    }

    /// The look facing one way, whichever way the door faces.
    const fn always(look: FigureLook, facing: Side) -> Self {
        let figure = Some(Figure { look, facing });
        Self {
            left: figure,
            right: figure,
        }
    }

    /// The figure for a door facing this way.
    pub const fn for_door(self, facing: Side) -> Option<Figure> {
        match facing {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }
}

/// An open door holds its open figure when it faces right, and its closed figure when it faces
/// left, each facing its way. It is hidden, so neither is drawn.
const OPEN_DOOR: Figures = Figures {
    left: Some(Figure {
        look: FigureLook::Closed,
        facing: Side::Left,
    }),
    right: Some(Figure {
        look: FigureLook::Open,
        facing: Side::Right,
    }),
};

/// The figures each phase holds, the spec's table of figures. An opening shows `Ajar1`, `Ajar2`,
/// `Ajar3` and `Open` and a closing shows them back, so each figure of a closing is the
/// opening's from the other end. A starting door holds the closed figure facing right and the
/// wall the closed figure facing left, whichever way the door faces. A hidden door holds its
/// figure too, though it is never drawn.
#[must_use]
pub const fn figures(phase: Phase) -> Figures {
    match phase {
        Phase::Placed => Figures::NONE,
        Phase::Starting { .. } => Figures::always(FigureLook::Closed, Side::Right),
        Phase::Closed { .. } => Figures::facing_its_way(FigureLook::Closed),
        Phase::Opening {
            quarter: Quarter::First,
            ..
        }
        | Phase::Closing {
            quarter: Quarter::Fourth,
            ..
        } => Figures::facing_its_way(FigureLook::Ajar1),
        Phase::Opening {
            quarter: Quarter::Second,
            ..
        }
        | Phase::Closing {
            quarter: Quarter::Third,
            ..
        } => Figures::facing_its_way(FigureLook::Ajar2),
        Phase::Opening {
            quarter: Quarter::Third,
            ..
        }
        | Phase::Closing {
            quarter: Quarter::Second,
            ..
        } => Figures::facing_its_way(FigureLook::Ajar3),
        Phase::Opening {
            quarter: Quarter::Fourth,
            ..
        }
        | Phase::Closing {
            quarter: Quarter::First,
            ..
        } => Figures::facing_its_way(FigureLook::Open),
        Phase::Open { .. } => OPEN_DOOR,
        Phase::Wall => Figures::always(FigureLook::Closed, Side::Left),
    }
}
