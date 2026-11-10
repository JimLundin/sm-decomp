//! Her map, as the mini-map reads it (§1.3): 64 columns by 32 rows of cells, each cell's picture
//! with its flips, its beyond pictures, and the cells it charts; and the sets of its cells. A
//! cell of her map is a `Point`, which may lie outside it (§1.5).

use crate::grid::Grid;
use crate::tile::Picture;

/// A set of her map's cells, each `true` in it: the explored cells (§1), or the cells a map
/// charts. A cell outside the map is in no set, and joining it changes nothing (§1.5).
pub type MapCellSet = Grid<bool, 64, 32>;

/// Her map's pictures: its 64 columns, then in columns 64 and 65 its beyond pictures' columns 0
/// and 1, as far as the window reaches past the map's last column (spec: LastColumnsShift); 32
/// rows.
pub type MapPictures = Grid<Picture, 66, 32>;

/// Her map, the one of the area she is in (§1.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HerMap {
    /// Each cell's picture with its flips, then the beyond pictures.
    pub pictures: MapPictures,
    /// The cells it charts.
    pub charted: MapCellSet,
    /// Whether she has collected it.
    pub collected: bool,
}
