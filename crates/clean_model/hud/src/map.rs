//! Her map, as the mini-map reads it (§1.3): 64 columns by 32 rows of cells, each cell's picture
//! with its flips, its beyond pictures, and the cells it charts; and the sets of its cells.

use crate::grid::{Grid, Pos};
use crate::pictures::{ColourSet, Flip, Picture, Tile};

/// A cell of her map: column 0 to 63 across, row 0 to 31 down (Notation).
pub type MapCell = Pos<64, 32>;

/// A set of her map's cells, each `true` in it: the explored cells (§1), or the cells a map
/// charts.
pub type MapCellSet = Grid<bool, 64, 32>;

/// A place among her map's pictures: its 64 columns, then the 2 columns of its beyond pictures,
/// as far as the window reaches past its last column (spec: `LastColumnsShift`); 32 rows.
pub type PicturePos = Pos<66, 32>;

/// The map's blank picture, `01F` (§1.4).
const MAP_BLANK: Picture = Picture::new(0x01F);

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

    /// The picture as a HUD tile in `colours`, its flips kept.
    #[must_use]
    pub const fn in_colours(self, colours: ColourSet) -> Tile {
        Tile::new(self.picture, colours, self.flip)
    }
}

impl Default for MapPicture {
    /// The map's blank, which a cell shows until the map gives it a picture.
    fn default() -> Self {
        Self::BLANK
    }
}

/// Her map, the one of the area she is in (§1.3).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct HerMap {
    /// Each cell's picture with its flips; past column 63 the beyond pictures, which continue
    /// the map's pictures (spec: `LastColumnsShift`).
    pub pictures: Grid<MapPicture, 66, 32>,
    /// The cells it charts.
    pub charted: MapCellSet,
}
