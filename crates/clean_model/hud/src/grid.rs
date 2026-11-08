//! The HUD's tile grid (Notation, §1.4): three rows of 32 tiles, each a picture in one of the
//! HUD's colour sets, perhaps flipped. Every tile keeps what it shows until a rule draws it.

/// The grid's rows.
const ROWS: usize = 3;
/// The grid's columns.
const COLUMNS: usize = 32;
/// The HUD's colour sets (§1.4: eight palettes).
const COLOUR_SETS: u8 = 8;

/// A picture of the HUD's graphics, by the number the importer gives it (§1.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Picture(u16);

impl Picture {
    /// The picture numbered `number`.
    #[must_use]
    pub const fn new(number: u16) -> Self {
        Self(number)
    }

    /// Its number.
    #[must_use]
    pub const fn number(self) -> u16 {
        self.0
    }
}

/// One of the HUD's eight colour sets, 0 to 7 (§1.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColourSet(u8);

impl ColourSet {
    /// Colour set `number`, or `None` past the eight.
    #[must_use]
    pub const fn new(number: u8) -> Option<Self> {
        if number < COLOUR_SETS {
            Some(Self(number))
        } else {
            None
        }
    }

    /// A colour set the spec names; the tables of §1.4 give only numbers below eight.
    pub(crate) const fn named(number: u8) -> Self {
        Self(number)
    }

    /// Its number, 0 to 7.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.0
    }
}

/// How a tile's picture is flipped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flip {
    /// As drawn.
    None,
    /// Flipped left to right.
    Horizontal,
    /// Flipped top to bottom.
    Vertical,
    /// Flipped both ways.
    Both,
}

impl Flip {
    /// Whether it is flipped left to right.
    #[must_use]
    pub const fn horizontal(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    /// Whether it is flipped top to bottom.
    #[must_use]
    pub const fn vertical(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
}

/// What one of the HUD's tiles shows: a picture in a colour set, perhaps flipped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    /// The picture.
    pub picture: Picture,
    /// Its colour set.
    pub colours: ColourSet,
    /// Its flips.
    pub flip: Flip,
}

impl Tile {
    /// A picture in a colour set, flipped as `flip` says.
    #[must_use]
    pub const fn new(picture: Picture, colours: ColourSet, flip: Flip) -> Self {
        Self {
            picture,
            colours,
            flip,
        }
    }

    /// The same picture and flips in other colours.
    #[must_use]
    pub const fn in_colours(self, colours: ColourSet) -> Self {
        Self { colours, ..self }
    }
}

/// A tile's place in the grid: row 0 to 2 from the top, column 0 to 31 from the left.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GridPos {
    row: u8,
    column: u8,
}

impl GridPos {
    /// The tile at `row` and `column`, or `None` outside the grid's 3 rows and 32 columns.
    #[must_use]
    pub fn new(row: u8, column: u8) -> Option<Self> {
        (usize::from(row) < ROWS && usize::from(column) < COLUMNS).then_some(Self { row, column })
    }

    /// A place the spec's layout gives (§1.2), which lies in the grid.
    pub(crate) const fn at(row: u8, column: u8) -> Self {
        Self { row, column }
    }

    /// Its row.
    #[must_use]
    pub const fn row(self) -> u8 {
        self.row
    }

    /// Its column.
    #[must_use]
    pub const fn column(self) -> u8 {
        self.column
    }

    /// The place `rows` down and `columns` right of this one, if it lies in the grid.
    pub(crate) fn offset(self, rows: i32, columns: i32) -> Option<Self> {
        let row = i32::from(self.row).checked_add(rows)?;
        let column = i32::from(self.column).checked_add(columns)?;
        Self::new(u8::try_from(row).ok()?, u8::try_from(column).ok()?)
    }
}

/// The HUD's tiles: three rows of 32, the status bar and the mini-map's window.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HudGrid {
    rows: [[Tile; COLUMNS]; ROWS],
}

impl HudGrid {
    /// Every tile showing `tile`.
    #[must_use]
    pub const fn filled(tile: Tile) -> Self {
        Self {
            rows: [[tile; COLUMNS]; ROWS],
        }
    }

    /// What the tile at `pos` shows.
    #[must_use]
    pub fn tile(&self, pos: GridPos) -> Tile {
        // A `GridPos` always lies in the grid, so the fallback is never read.
        self.rows
            .get(usize::from(pos.row))
            .and_then(|row| row.get(usize::from(pos.column)))
            .copied()
            .unwrap_or(Tile::new(Picture(0), ColourSet(0), Flip::None))
    }

    /// Draws `tile` at `pos`.
    pub fn set(&mut self, pos: GridPos, tile: Tile) {
        if let Some(cell) = self
            .rows
            .get_mut(usize::from(pos.row))
            .and_then(|row| row.get_mut(usize::from(pos.column)))
        {
            *cell = tile;
        }
    }
}
