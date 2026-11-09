//! Her map, as the mini-map reads it (§1.3): its cells, each cell's picture with its flips, the
//! cells it charts, whether she has collected it, and its beyond pictures.

use crate::grid::{Flip, Picture};

/// A cell of her map: column 0 to 63, row 0 to 31, from the top left (Notation).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MapCell {
    column: u8,
    row: u8,
}

impl MapCell {
    /// Her map's columns (Notation).
    pub const COLUMNS: usize = 64;
    /// Her map's rows (Notation).
    pub const ROWS: usize = 32;

    /// The cell at `column` and `row`, or `None` off her map.
    #[must_use]
    pub fn new(column: u8, row: u8) -> Option<Self> {
        (usize::from(column) < Self::COLUMNS && usize::from(row) < Self::ROWS)
            .then_some(Self { column, row })
    }

    /// Its column.
    #[must_use]
    pub const fn column(self) -> u8 {
        self.column
    }

    /// Its row.
    #[must_use]
    pub const fn row(self) -> u8 {
        self.row
    }
}

/// A set of her map's cells: the explored cells (§1), or the cells a map charts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MapCellSet {
    /// One word per row; bit `c` is the cell in column `c`.
    rows: [u64; MapCell::ROWS],
}

impl MapCellSet {
    /// No cells.
    pub const EMPTY: Self = Self {
        rows: [0; MapCell::ROWS],
    };

    /// The bit of `cell` within its row's word.
    const fn bit(cell: MapCell) -> u64 {
        1 << cell.column
    }

    /// Adds `cell` to the set.
    pub fn insert(&mut self, cell: MapCell) {
        if let Some(row) = self.rows.get_mut(usize::from(cell.row)) {
            *row |= Self::bit(cell);
        }
    }

    /// Whether `cell` is in the set.
    #[must_use]
    pub fn contains(&self, cell: MapCell) -> bool {
        self.rows
            .get(usize::from(cell.row))
            .is_some_and(|row| row & Self::bit(cell) != 0)
    }

    /// The cells in the set, row by row from the top, each row left to right.
    pub fn iter(&self) -> impl Iterator<Item = MapCell> + '_ {
        (0..=u8::MAX)
            .flat_map(|row| (0..=u8::MAX).filter_map(move |column| MapCell::new(column, row)))
            .filter(|&cell| self.contains(cell))
    }
}

/// The map's blank picture, `01F` (§1.4).
const MAP_BLANK: Picture = Picture::new(0x01F);

/// The beyond pictures' columns: as many as the window reaches past the map's last column, 2
/// (spec: `LastColumnsShift`, the window's 2 columns to each side of hers).
const BEYOND_COLUMNS: usize = 2;

/// A picture of her map, with its flips; the mini-map chooses its colours.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MapPicture {
    /// The picture.
    pub picture: Picture,
    /// Its flips.
    pub flip: Flip,
}

impl MapPicture {
    /// The map's blank, `01F` unflipped (§1.4).
    pub const BLANK: Self = Self {
        picture: MAP_BLANK,
        flip: Flip::None,
    };
}

/// A beyond picture's place: column 0 or 1 past the map's last column, row 0 to 31 (spec:
/// `LastColumnsShift`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BeyondCell {
    column: u8,
    row: u8,
}

impl BeyondCell {
    /// The beyond picture `column` past the map's last column (from 0) in `row`, or `None` past
    /// the two columns the window reaches or the map's 32 rows.
    #[must_use]
    pub fn new(column: u8, row: u8) -> Option<Self> {
        (usize::from(column) < BEYOND_COLUMNS && usize::from(row) < MapCell::ROWS)
            .then_some(Self { column, row })
    }
}

/// Her map, the one of the area she is in: 64 columns by 32 rows of cells (§1.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AreaMap {
    pictures: [[MapPicture; MapCell::COLUMNS]; MapCell::ROWS],
    charted: MapCellSet,
    collected: bool,
    beyond: [[MapPicture; BEYOND_COLUMNS]; MapCell::ROWS],
}

impl Default for AreaMap {
    fn default() -> Self {
        Self::new()
    }
}

impl AreaMap {
    /// A map she hasn't collected that charts nothing, every picture the map's blank.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pictures: [[MapPicture::BLANK; MapCell::COLUMNS]; MapCell::ROWS],
            charted: MapCellSet::EMPTY,
            collected: false,
            beyond: [[MapPicture::BLANK; BEYOND_COLUMNS]; MapCell::ROWS],
        }
    }

    /// Sets the picture of `cell`.
    pub fn set_picture(&mut self, cell: MapCell, picture: MapPicture) {
        if let Some(slot) = self
            .pictures
            .get_mut(usize::from(cell.row()))
            .and_then(|row| row.get_mut(usize::from(cell.column())))
        {
            *slot = picture;
        }
    }

    /// Sets a beyond picture.
    pub fn set_beyond(&mut self, cell: BeyondCell, picture: MapPicture) {
        if let Some(slot) = self
            .beyond
            .get_mut(usize::from(cell.row))
            .and_then(|row| row.get_mut(usize::from(cell.column)))
        {
            *slot = picture;
        }
    }

    /// Makes the map chart `cell`.
    pub fn chart(&mut self, cell: MapCell) {
        self.charted.insert(cell);
    }

    /// Sets whether she has collected the map.
    pub const fn set_collected(&mut self, collected: bool) {
        self.collected = collected;
    }

    /// The picture of `cell`.
    #[must_use]
    pub fn picture(&self, cell: MapCell) -> MapPicture {
        self.pictures
            .get(usize::from(cell.row()))
            .and_then(|row| row.get(usize::from(cell.column())))
            .copied()
            .unwrap_or(MapPicture::BLANK)
    }

    /// A beyond picture.
    #[must_use]
    pub fn beyond(&self, cell: BeyondCell) -> MapPicture {
        self.beyond
            .get(usize::from(cell.row))
            .and_then(|row| row.get(usize::from(cell.column)))
            .copied()
            .unwrap_or(MapPicture::BLANK)
    }

    /// Whether the map charts `cell`.
    #[must_use]
    pub fn charts(&self, cell: MapCell) -> bool {
        self.charted.contains(cell)
    }

    /// Whether she has collected the map.
    #[must_use]
    pub const fn collected(&self) -> bool {
        self.collected
    }
}
