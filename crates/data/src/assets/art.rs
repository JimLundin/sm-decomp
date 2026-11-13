//! The shapes pictures are made of (docs/architecture/assets.md §2): indexed 8x8 tiles in a
//! sheet, colours in palettes, and cells that place a tile with a palette row and its flips.
//!
//! They hold pixels, colours and choices, not the hardware's encodings of them: the planar
//! tile bytes, the colour words in colour RAM and the tilemap word are `game::exact`'s.

use canonical::Canonical;

/// A tile's side in pixels.
pub const TILE_SIDE: usize = 8;

/// A colour as the console shows it: five bits each of red, green and blue, blue highest
/// (BGR555). The top bit is kept as the content holds it, so the colour round-trips; the
/// console ignores it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Colour(pub u16);

/// The colours a palette row holds.
pub const ROW_COLOURS: usize = 16;

/// A row of 16 colours, which a 4bpp tile's pixels index; colour 0 is transparent.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Palette(pub [Colour; ROW_COLOURS]);

/// Palette rows in order: a cell's palette row picks one.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PaletteSet(pub Vec<Palette>);

/// An 8x8 tile of indexed pixels, row by row from the top, each a colour's place in a palette
/// row: 0 to 15 in a 4bpp sheet.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Tile(pub [[u8; TILE_SIDE]; TILE_SIDE]);

/// Tiles in order, as an indexed image cut into 8x8 tiles; a [`TileRef`] counts from the first.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TileSheet(pub Vec<Tile>);

/// A tile's place among the tiles a background draws from: ten bits.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TileRef(pub u16);

/// Which of a background's eight palette rows a cell draws with.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PaletteRow(pub u8);

/// A tile placed in a layer: which tile, in which palette row, whether it draws above the
/// layer's low-priority cells, and its mirrorings.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    /// The tile.
    pub tile: TileRef,
    /// Its palette row.
    pub palette: PaletteRow,
    /// Whether it draws above the layer's low-priority cells.
    pub priority: bool,
    /// Mirrored left to right.
    pub flip_x: bool,
    /// Mirrored top to bottom.
    pub flip_y: bool,
}

/// A 16x16 block's picture: four cells, as Tiled builds a 16x16 tile of 8x8s.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockLook {
    /// The top-left cell.
    pub top_left: Cell,
    /// The top-right cell.
    pub top_right: Cell,
    /// The bottom-left cell.
    pub bottom_left: Cell,
    /// The bottom-right cell.
    pub bottom_right: Cell,
}

impl BlockLook {
    /// The four cells in reading order: top left, top right, bottom left, bottom right.
    #[must_use]
    pub const fn cells(self) -> [Cell; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
        ]
    }

    /// The look of four cells in reading order.
    #[must_use]
    pub const fn from_cells([top_left, top_right, bottom_left, bottom_right]: [Cell; 4]) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
}
