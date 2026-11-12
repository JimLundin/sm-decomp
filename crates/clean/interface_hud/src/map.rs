//! Her map, as the mini-map reads it (§1.3): 64 columns by 32 rows of cells, each cell's picture
//! with its flips, its beyond pictures, and the cells it charts; and the sets of its cells. A
//! cell of her map is a `Place`, which may lie outside it (§1.5).

use types::Look;

use crate::grid::Grid;

/// A set of her map's cells, each `true` in it: the explored cells (§1), or the cells a map
/// charts. A cell outside the map is in no set, and joining it changes nothing (§1.5).
pub type MapCellSet = Grid<bool, 64, 32>;

/// Her map's pictures with their flips, 66 columns by 32 rows.
///
/// Its own 64 columns, then in columns 64 and 65 its beyond pictures' columns 0 and 1, as far as
/// the window reaches past the map's last column (spec: LastColumnsShift).
pub type MapPictures = Grid<Look, 66, 32>;

/// Her map, the one of the area she is in (§1.3), as the caller lends it: what it draws and
/// charts. Whether she has collected it is a fact of the frame (`MiniMapInputs`).
///
/// Its default, every picture `000` unflipped and nothing charted, is a stand-in for a caller
/// with nothing imported yet: no rule says what it shows (§1.4).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct HerMap {
    /// Each cell's picture with its flips, then the beyond pictures.
    pub pictures: MapPictures,
    /// The cells it charts.
    pub charted: MapCellSet,
}
