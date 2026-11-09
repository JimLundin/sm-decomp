//! The room's cells as this domain sees them: each cell's kind, mark and look (spec §1).

use clean_types::{Cell, Look, MetAs, Pair};

use crate::reactions::ReactionName;

/// A grid of the room, one value per cell or per screen, read by `(x, y)` from its top-left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid<T> {
    /// The rows, top to bottom, each left to right.
    pub rows: Vec<Vec<T>>,
}

impl<T> Grid<T> {
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
    /// What she meets when she is inside it.
    pub inside: Inside,
}

/// A cell's kind, as far as it concerns this domain (spec §1). Listed alphabetically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    /// Nothing stops her or her shots.
    Air,
    /// A reacting block, solid by its body; its mark says how it reacts.
    Block,
    /// A reacting cell with an air body: she and her shots pass it.
    ReactingAir,
    /// A plain solid block.
    Solid,
    /// A cell that stands for another along its row; its mark is how far.
    StandInAcross,
    /// A cell that stands for another along its column; its mark is how far.
    StandInDown,
}

/// The two reacting kinds: what a reaction remembers its cell was, and grows it back as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reacting {
    /// `Block`: solid to her.
    Block,
    /// `ReactingAir`: air to her.
    Air,
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
    /// The reacting kind this is, if it is one: only a reacting cell reacts to anything.
    #[must_use]
    pub const fn reacting(self) -> Option<Reacting> {
        match self {
            Self::Block => Some(Reacting::Block),
            Self::ReactingAir => Some(Reacting::Air),
            Self::Air | Self::Solid | Self::StandInAcross | Self::StandInDown => None,
        }
    }

    /// `AnswersToHerMotion`: how she meets a cell of this kind by its body. A stand-in here stands
    /// for no cell (her motion resolves the others), and one that names nothing is air.
    #[must_use]
    pub const fn body(self) -> Answer {
        match self {
            Self::Block | Self::Solid => Answer::Solid,
            Self::Air | Self::ReactingAir | Self::StandInAcross | Self::StandInDown => Answer::Air,
        }
    }
}

impl Reacting {
    /// The cell kind it is.
    #[must_use]
    pub const fn kind(self) -> CellKind {
        match self {
            Self::Block => CellKind::Block,
            Self::Air => CellKind::ReactingAir,
        }
    }
}

/// A cell's mark: which reaction a reacting cell has, or how far a stand-in reaches (spec §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mark {
    /// No mark: a cell of a kind that doesn't read one.
    Unmarked,
    /// An imported reaction, by its name.
    Reaction(ReactionName),
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
    /// Reacting air whose mark holds a place when she is inside it: each of her points inside it
    /// holds one until the run.
    HoldsPlace,
}

/// The cell `by` cells from `cell`, if it is in the room's range of cells.
#[must_use]
pub fn offset(cell: Cell, by: Pair<i16>) -> Option<Cell> {
    Some(Pair {
        x: cell.x.checked_add_signed(by.x)?,
        y: cell.y.checked_add_signed(by.y)?,
    })
}
