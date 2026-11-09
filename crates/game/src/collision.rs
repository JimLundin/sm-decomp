//! The room Samus meets and what her moves start in it.
//!
//! The level's blocks, the solid enemies, and the cells her moves reach that react, which the
//! blocks model and the items meet after the moves. The moves themselves are the samus-motion
//! model's (`samus_world`); the cells she is inside are `inside`'s.

use data::level::BlockKind;
use data::level::slope::Slopes;

use crate::blocks_world::{self, BlocksFault, Reads};
use crate::plm::{self, TouchFault};
use crate::state::room::{Block, BlockColumns, BlockIndex, RoomScoped};

pub mod inside;
pub mod solid_enemy;

/// What a move went through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Room<'a> {
    /// The level's blocks, row-major.
    pub blocks: &'a [Block],
    /// Blocks a row.
    pub width: BlockColumns,
    /// The slope shapes.
    pub slopes: Option<&'a Slopes>,
}

/// A reacting cell her moves reached.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Touch {
    /// A cell of the blocks', and the way she reached it.
    Blocks {
        /// The cell, a stand-in followed to the cell it stands for.
        cell: clean_types::Cell,
        /// The way she reached it.
        way: clean_types::Way,
    },
    /// An item's block: the item collision detection PLM (`$84:EED3`).
    Item(BlockIndex),
}

impl core::fmt::Debug for Touch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Blocks { cell, way } => {
                let way = match way {
                    clean_types::Way::Move(heading) => match heading {
                        clean_types::Heading::Down => "down",
                        clean_types::Heading::Left => "left",
                        clean_types::Heading::Right => "right",
                        clean_types::Heading::Up => "up",
                    },
                    clean_types::Way::Look => "look",
                };
                write!(f, "Blocks({cell:?}, {way})")
            }
            Self::Item(block) => write!(f, "Item({block:?})"),
        }
    }
}

/// The reacting cells a frame's moves reached, in the order their scans reached them.
///
/// The collision reactions spawn their PLMs during the scan (`$94:906F`, `$94:909D` for special
/// air, `$94:90CB`, `$94:9102` for special blocks, through `$84:84E7`). Her moves meet each cell
/// by its body (`samus_world`), so the touches apply after the moves in the same order; the
/// touch's answer to her motion is a hole until her moves ask the blocks (`blocks_world`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Touches(Vec<Touch>);

impl Touches {
    /// The touches, in order.
    #[must_use]
    pub fn touched(&self) -> &[Touch] {
        &self.0
    }

    /// Each touch in order: the blocks' to the blocks model, the items' to their PLMs.
    ///
    /// # Errors
    ///
    /// [`Fault`] where an item's touch finds no item on its block, or a cell is one the room's
    /// loading would refuse.
    pub fn apply(self, room: &mut RoomScoped, reads: Reads<'_>) -> Result<(), Fault> {
        self.0.into_iter().try_for_each(|touch| match touch {
            Touch::Blocks { cell, way } => {
                blocks_world::touch(room, reads, clean_types::Touch { cell, way })
                    .map_err(Fault::Blocks)
            }
            Touch::Item(block) => plm::touch_item_block(room, block).map_err(Fault::Item),
        })
    }
}

/// Why a touch can't go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// An item's touch.
    Item(TouchFault),
    /// The blocks'.
    Blocks(BlocksFault),
}

impl Extend<Touch> for Touches {
    fn extend<I: IntoIterator<Item = Touch>>(&mut self, touches: I) {
        self.0.extend(touches);
    }
}

impl FromIterator<Touch> for Touches {
    fn from_iter<I: IntoIterator<Item = Touch>>(touches: I) -> Self {
        Self(touches.into_iter().collect())
    }
}

/// Which way a vertical scan crosses Samus's span (`$94:9775`: by the frame counter's parity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScanOrder {
    /// Even frames.
    #[default]
    LeftToRight,
    /// Odd frames.
    RightToLeft,
}

impl ScanOrder {
    /// The order on a frame whose requested-NMI count is `frame`.
    #[must_use]
    pub const fn of(frame: u16) -> Self {
        if frame & 1 == 0 {
            Self::LeftToRight
        } else {
            Self::RightToLeft
        }
    }
}

/// Where the frame's moves happen: the room and the enemies she can't pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Surroundings<'a> {
    /// The room's blocks.
    pub room: Room<'a>,
    /// The interactive enemies, which solid enemy collision checks before the blocks.
    pub enemies: solid_enemy::SolidEnemies<'a>,
}

/// The first block of a row: the multiplier takes the row's and the width's low bytes.
fn row_start(row: u16, width: BlockColumns) -> u16 {
    let [row, _] = row.to_le_bytes();
    let [width, _] = width.0.to_le_bytes();
    u16::from(row).wrapping_mul(u16::from(width))
}

impl Room<'_> {
    /// The block at `index`; past the level, air.
    // open: the original reads whatever work RAM follows the level there.
    fn kind(&self, index: u16) -> BlockKind {
        self.blocks
            .get(usize::from(index))
            .map_or_else(BlockKind::default, |block| block.kind)
    }
}
