//! The imported sprite clips and normal boss colours in the Zebes flashback.

use crate::spritemap::Spritemap;
pub use types::{Colour, ROW_COLOURS, RowColours};

/// The encounter's four clips, with their images in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scene {
    /// Mother Brain's breathing images.
    pub brain: [Spritemap; 3],
    /// The rings' images.
    pub ring: [Spritemap; 3],
    /// Small burst images.
    pub small_burst: [Spritemap; 6],
    /// Large burst images.
    pub large_burst: [Spritemap; 6],
    /// The palette Mother Brain's flash restores.
    pub normal_boss: RowColours,
}
