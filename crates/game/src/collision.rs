//! The room Samus meets and what her moves start in it.
//!
//! The level's blocks, the solid enemies, and the blocks whose reactions her moves reach,
//! which spawn their PLMs after the moves. The moves themselves are the samus-motion model's
//! (`samus_world`); the reactions of the blocks she is inside are `inside`'s.

use data::level::slope::Slopes;
use data::level::{BlockKind, Bts};

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

/// The special air BTS that triggers a scroll PLM: its collision reactions spawn the scroll
/// block touch PLM (`$94:9139` + `$46` * 2 = `$84:B6FF`), and so does its inside reaction at
/// Samus's centre (`$94:9956`).
const SCROLL_TRIGGER: Bts = Bts(0x46);

/// A block whose collision reaction spawns a PLM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    /// A scroll trigger: the scroll block touch PLM (`$84:B6FF`).
    Scroll(BlockIndex),
    /// An item's block: the item collision detection PLM (`$84:EED3`).
    Item(BlockIndex),
}

/// The blocks a frame's moves reached whose reactions spawn a PLM, in the order their scans
/// reached them.
///
/// The collision reactions spawn the PLM during the scan (`$94:906F`, `$94:909D` for special
/// air, `$94:90CB`, `$94:9102` for special blocks, through `$84:84E7`), but the setups
/// (`$84:B393`, `$84:EEAB`) change only PLM slots, which no move reads, and the item's lets
/// her through (carry clear), so the touches apply after the moves in the same order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Touches(Vec<Touch>);

impl Touches {
    /// The touches, in order.
    #[must_use]
    pub fn touched(&self) -> &[Touch] {
        &self.0
    }

    /// Spawn each touch's PLM in order.
    ///
    /// # Errors
    ///
    /// [`TouchFault`] where a touch finds no PLM of its kind on its block.
    pub fn apply(self, room: &mut RoomScoped) -> Result<(), TouchFault> {
        self.0.into_iter().try_for_each(|touch| match touch {
            Touch::Scroll(block) => plm::touch_scroll_block(room, block),
            Touch::Item(block) => plm::touch_item_block(room, block),
        })
    }
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
