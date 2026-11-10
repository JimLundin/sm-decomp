//! The room's cells as this domain sees them: each cell's kind, mark and look (spec §1), and how
//! a step writes a cell.

use std::sync::Arc;

use clean_types::{Cell, Look, MetAs, Pair, Picture};
use enum_map::Enum;

use crate::reactions::{Becomes, PatternCell, Reaction};

/// A grid of the room, one value per cell or per screen, read by `(x, y)` from its top-left;
/// every row is as long as the others.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid<T> {
    /// The rows, top to bottom, each left to right.
    rows: Vec<Vec<T>>,
}

impl<T> Grid<T> {
    /// The grid of these rows, top to bottom, each left to right, if every row is as long as the
    /// others.
    #[must_use]
    pub fn new(rows: Vec<Vec<T>>) -> Option<Self> {
        let width = rows.first().map(Vec::len);
        rows.iter()
            .all(|row| Some(row.len()) == width)
            .then_some(Self { rows })
    }

    /// The rows, top to bottom, each left to right.
    #[must_use]
    pub fn rows(&self) -> &[Vec<T>] {
        &self.rows
    }

    /// The value at `at`, if the grid reaches it.
    #[must_use]
    pub fn get(&self, at: Pair<u16>) -> Option<&T> {
        self.rows.get(usize::from(at.y))?.get(usize::from(at.x))
    }

    /// The value at `at` to change, if the grid reaches it.
    pub fn get_mut(&mut self, at: Pair<u16>) -> Option<&mut T> {
        self.rows
            .get_mut(usize::from(at.y))?
            .get_mut(usize::from(at.x))
    }
}

/// The room's cells.
pub type Tilemap = Grid<Tile>;

/// One cell of the room: what she and her shots meet, what reacts, and the picture it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    /// Its kind.
    pub kind: CellKind,
    /// Its mark, which no step changes.
    pub mark: Mark,
    /// The picture it shows.
    pub look: Look,
    /// The part of its mark that says what she meets when she is inside it.
    pub inside: Inside,
}

/// A cell's kind, as far as it concerns this domain (spec §1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    /// `Air`: nothing stops her or her shots.
    Air,
    /// `Solid`: a plain solid block.
    Solid,
    /// A reacting cell, `Block` or `ReactingAir` by its body; its mark says how it reacts.
    Reacting(ReactingBody),
    /// `StandInAcross` or `StandInDown`: a cell that stands for another along its row or its
    /// column; its mark is how far.
    StandIn(StandIn),
}

/// A reacting cell's body: whether she and her shots pass it. What a reaction grows back as is
/// one of these. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ReactingBody {
    /// `ReactingAir`: she and her shots pass it.
    Air,
    /// `Block`: solid to her by its body.
    Block,
}

/// The way a stand-in stands for another cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandIn {
    /// Along its row.
    Across,
    /// Along its column.
    Down,
}

/// How a touch of a cell meets her motion (spec §1.4, `AnswersToHerMotion`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// She passes it.
    Air,
    /// It stops her.
    Solid,
    /// A cell marked `Other`: its part answers it.
    Other,
}

impl CellKind {
    /// `AnswersToHerMotion`: how she meets a cell of this kind by its body. Her motion follows a
    /// stand-in to the cell it stands for, so one met here stands for none, and is air.
    #[must_use]
    pub const fn body(self) -> Answer {
        match self {
            Self::Solid | Self::Reacting(ReactingBody::Block) => Answer::Solid,
            Self::Air | Self::Reacting(ReactingBody::Air) | Self::StandIn(_) => Answer::Air,
        }
    }
}

impl ReactingBody {
    /// The plain cell of this body, which reacts to nothing: `Solid` for a block, `Air` for
    /// reacting air.
    #[must_use]
    pub const fn plain(self) -> CellKind {
        match self {
            Self::Block => CellKind::Solid,
            Self::Air => CellKind::Air,
        }
    }
}

/// A cell's mark: which reaction a reacting cell has, or how far a stand-in reaches (spec §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mark {
    /// No mark: a cell of a kind that doesn't read one.
    Unmarked,
    /// One of the room's reactions, which the mark names.
    Reaction(Arc<Reaction>),
    /// A scroll cell (`ScrollCells`).
    Scroll,
    /// A cell that reacts to nothing but holds a place, met as this family's blocks
    /// (`BlocksMarkedNothing`).
    Nothing(MetAs),
    /// A cell that reacts to nothing and takes no place (`BlocksMarkedNothing`).
    Inert,
    /// A cell of another part, which answers it; with the family a blast meets it as, if any
    /// (the room gives only `Crumble`, for a station's cells).
    Other(Option<MetAs>),
    /// A stand-in's reach: how many cells along its row or column the cell it stands for is.
    Reach(i16),
}

/// What she meets when she is inside a cell (`BlocksMarkedNothing`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inside {
    /// Nothing but what its mark says.
    Passes,
    /// Its mark holds a place when she is inside it: while it is reacting air, each of her points
    /// inside it holds one until the run.
    HoldsPlace,
}

impl Tile {
    /// `PatternCells`: a show's pattern cell writes this cell: it takes the pattern's kind and
    /// look, and keeps its mark. `Itself` is its reacting kind as a block.
    pub const fn show(&mut self, cell: &PatternCell) {
        self.kind = match cell.becomes {
            Becomes::Air => CellKind::Air,
            Becomes::Itself => CellKind::Reacting(ReactingBody::Block),
            Becomes::StandInAcross => CellKind::StandIn(StandIn::Across),
            Becomes::StandInDown => CellKind::StandIn(StandIn::Down),
        };
        self.look = cell.look;
    }

    /// `Regrowing` (a quirk): the cell grows back as the reacting kind its reaction remembers,
    /// with its reaction's whole look, not the look it had before its start, so a hidden block
    /// shows itself once it has grown back.
    pub const fn regrow(&mut self, body: ReactingBody, whole_look: Option<Picture>) {
        self.kind = CellKind::Reacting(body);
        if let Some(picture) = whole_look {
            self.look = Look::unflipped(picture);
        }
    }
}

/// The cell `by` cells from `cell`, if it is in the room's range of cells.
#[must_use]
pub fn offset(cell: Cell, by: Pair<i16>) -> Option<Cell> {
    Some(Pair {
        x: cell.x.checked_add_signed(by.x)?,
        y: cell.y.checked_add_signed(by.y)?,
    })
}
