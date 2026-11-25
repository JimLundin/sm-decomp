//! `PuffReach`: the box a puff reaches, growing out of its vent the way it faces, and whether
//! her body shares a pixel with it.

use core::num::Wrapping;

use types::{Pair, Px, part_below};

use crate::her::Her;
use crate::puff::{PuffFigure, VentFacing};

/// A box's edges, in px from a vent's position, x rightward and y downward, each edge included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Offsets {
    left: i16,
    top: i16,
    right: i16,
    bottom: i16,
}

/// A box of offsets, its left, top, right and bottom edges.
const fn edges(left: i16, top: i16, right: i16, bottom: i16) -> Offsets {
    Offsets {
        left,
        top,
        right,
        bottom,
    }
}

/// `PuffReach`: the figures that reach: the first five. The last two, thinned away, have no box
/// and reach nothing.
const REACHING_FIGURES: usize = 5;

/// `PuffReach`: one facing's boxes, for figures 1 to 5 in order.
type FacingReach = [Offsets; REACHING_FIGURES];

/// `PuffReach`: a downward puff's boxes.
const DOWN_REACH: FacingReach = [
    edges(-8, 0, 7, 14),
    edges(-8, 0, 7, 23),
    edges(-8, 8, 7, 31),
    edges(-8, 15, 7, 39),
    edges(-8, 23, 6, 38),
];
/// `PuffReach`: a leftward puff's boxes.
const LEFT_REACH: FacingReach = [
    edges(-16, -8, -1, 7),
    edges(-24, -8, -2, 7),
    edges(-32, -7, -9, 7),
    edges(-40, -9, -17, 5),
    edges(-40, -11, -26, 2),
];
/// `PuffReach`: a rightward puff's boxes.
const RIGHT_REACH: FacingReach = [
    edges(0, -8, 15, 7),
    edges(1, -8, 23, 7),
    edges(9, -8, 31, 7),
    edges(18, -9, 38, 5),
    edges(25, -11, 40, 3),
];
/// `PuffReach`: an upward puff's boxes.
const UP_REACH: FacingReach = [
    edges(-8, -16, 7, -1),
    edges(-8, -23, 7, -2),
    edges(-8, -32, 7, -8),
    edges(-8, -40, 7, -16),
    edges(-8, -40, 6, -24),
];

/// `PuffReach`: the boxes of a puff blowing this way.
const fn facing_reach(facing: VentFacing) -> &'static FacingReach {
    match facing {
        VentFacing::Down => &DOWN_REACH,
        VentFacing::Left => &LEFT_REACH,
        VentFacing::Right => &RIGHT_REACH,
        VentFacing::Up => &UP_REACH,
    }
}

/// `PuffReach`: a vent never run reaches the one pixel at its position.
const ONE_PIXEL: Offsets = edges(0, 0, 0, 0);

/// Where a vent reaches on a run: a box of the room, its edges in px, each included and each
/// taken mod 2^16.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reach {
    left: Px,
    top: Px,
    right: Px,
    bottom: Px,
}

impl Reach {
    /// `PuffReach`: the reach of a puff showing `figure`, blowing the way `facing` says, out of
    /// a vent at `vent`: its facing's box for the figure, moved to the vent; none for the sixth
    /// and seventh figures.
    pub(crate) fn of_puff(figure: PuffFigure, facing: VentFacing, vent: Pair<Px>) -> Option<Self> {
        usize::from(figure.get())
            .checked_sub(1)
            .and_then(|row| facing_reach(facing).get(row))
            .map(|offsets| Self::moved(*offsets, vent))
    }

    /// `PuffReach`: the reach of a vent never run: the one pixel at its position.
    pub(crate) fn of_placed(vent: Pair<Px>) -> Self {
        Self::moved(ONE_PIXEL, vent)
    }

    /// A box of offsets moved to a vent's position, each edge mod 2^16.
    fn moved(offsets: Offsets, vent: Pair<Px>) -> Self {
        let at = |position: Px, offset: i16| Wrapping(position.0.wrapping_add_signed(offset));
        Self {
            left: at(vent.x, offsets.left),
            top: at(vent.y, offsets.top),
            right: at(vent.x, offsets.right),
            bottom: at(vent.y, offsets.bottom),
        }
    }

    /// `PuffTouches`: whether her body shares a pixel column and row with this reach, her box
    /// running from `hx − w` to `hx + w − 1` across and `hy − h` to `hy + h − 1` down: each
    /// difference read signed, `L − (hx + w) < 0`, `R − (hx − w) ≥ 0`, `T − (hy + h) < 0` and
    /// `B − (hy − h) ≥ 0`.
    pub(crate) fn touches(self, her: &Her) -> bool {
        let Pair { x, y } = her.position;
        let half = her.body;
        part_below(self.left.0, (x + half.x).0)
            && !part_below(self.right.0, (x - half.x).0)
            && part_below(self.top.0, (y + half.y).0)
            && !part_below(self.bottom.0, (y - half.y).0)
    }
}
