//! The level as the backgrounds draw it: the look of each block of a layer's level data.
//!
//! A line coming into view (`intent::LayerExposure`) reads its blocks from the original's level
//! buffer, as the column builder (`$80:A9DE`) and the row builder (`$80:AB78`) do: 16 blocks
//! down or 17 along, from the index the original's multiply gives the line's first block.
//! Each block draws its tile table entry, or the one an item's GFX load rewrote, with its flips.

use data::level::Tile;
use data::tileset::{BlockTiles, TileTables};

use crate::intent::{BlockLine, BlockLook, LayerExposure, LevelLayer};
use crate::state::room::{BlockPoint, RoomScoped};
use crate::state::video::TilemapEntry;

/// The blocks the original's level buffer holds: decompressing a room's level data first
/// fills that many words with `$8000` (bank `$82` log, `$82:E7D3`).
const LEVEL_BUFFER_BLOCKS: usize = 0x3200;
/// The level buffer's bank wraps a block index at 64 KiB of words.
const BANK_BLOCKS: u16 = 0x7FFF;
/// The fill word's bytes, low then high.
const FILL: [u8; 2] = [0x00, 0x80];

/// What the backgrounds draw from: the room's level data and the tile tables.
#[derive(Clone, Copy, Debug)]
struct Level<'a> {
    room: &'a RoomScoped,
    tables: &'a TileTables,
}

impl Level<'_> {
    /// The level data word at block `index` of `layer`, as the original's level buffer holds
    /// it, or `None` where the buffer holds what an earlier room left (docs/LEGACY.md).
    fn word(self, layer: LevelLayer, index: u16) -> Option<u16> {
        let index = usize::from(index & BANK_BLOCKS);
        match layer {
            LevelLayer::One => {
                if let Some(block) = self.room.blocks.get(index) {
                    return Some(block.word());
                }
                if index >= LEVEL_BUFFER_BLOCKS {
                    return None;
                }
                let past = index.checked_sub(self.room.blocks.len())?.wrapping_mul(2);
                let byte =
                    |at: usize, fill: u8| self.room.level_tail.get(at).copied().unwrap_or(fill);
                let [low, high] = FILL;
                Some(u16::from_le_bytes([
                    byte(past, low),
                    byte(past.wrapping_add(1), high),
                ]))
            }
            LevelLayer::Two => self
                .room
                .layer2
                .as_ref()
                .and_then(|blocks| blocks.get(index))
                .map(|block| block.word()),
        }
    }

    /// The four tilemap words the block at `index` draws, with its flips: the entry an item's
    /// GFX load rewrote in the original's tile table, else the tileset's.
    ///
    /// A block word past the level data, or a tile number past what the tile tables fill,
    /// reads stale RAM in the original; the engine draws tile 0's words or zero words there
    /// (docs/LEGACY.md).
    fn drawn(self, layer: LevelLayer, index: u16) -> BlockLook {
        self.drawn_tile(Tile::from_word(self.word(layer, index).unwrap_or_default()))
    }

    /// The four tilemap words `tile` draws, with its flips.
    fn drawn_tile(self, tile: Tile) -> BlockLook {
        let number = tile.number.0;
        let tiles = crate::plm::item_gfx::block(&self.room.item_gfx, number)
            .or_else(|| self.tables.block(self.room.area, self.room.tileset, number))
            .unwrap_or_default();
        BlockLook::of(tiles, tile)
    }

    /// The level block index the original's multiply gives `block`: the row's low byte times
    /// the width's low byte, plus the column.
    fn index(self, block: BlockPoint) -> u16 {
        let [row, _] = block.y.to_le_bytes();
        let [width, _] = self.room.width.0.to_le_bytes();
        u16::from(row)
            .wrapping_mul(u16::from(width))
            .wrapping_add(block.x)
    }
}

/// The look `tile` has in `room`, as the level draws it.
pub(crate) fn block_look(room: &RoomScoped, tables: &TileTables, tile: Tile) -> BlockLook {
    Level { room, tables }.drawn_tile(tile)
}

/// A block's look is its tile table entry with its flips.
impl BlockLook {
    /// A block flipped across mirrors its quarters and flips each word the same way.
    fn of(tiles: BlockTiles, tile: Tile) -> Self {
        let entry = |word: u16| {
            let entry = TilemapEntry::from_word(word);
            TilemapEntry {
                flip_x: entry.flip_x != tile.x_flip,
                flip_y: entry.flip_y != tile.y_flip,
                ..entry
            }
        };
        let [top_left, top_right, bottom_left, bottom_right] = [
            tiles.top_left,
            tiles.top_right,
            tiles.bottom_left,
            tiles.bottom_right,
        ]
        .map(entry);
        let unflipped = Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        };
        let across = if tile.x_flip {
            unflipped.mirrored_across()
        } else {
            unflipped
        };
        if tile.y_flip {
            across.mirrored_down()
        } else {
            across
        }
    }

    /// Left and right swapped.
    const fn mirrored_across(self) -> Self {
        Self {
            top_left: self.top_right,
            top_right: self.top_left,
            bottom_left: self.bottom_right,
            bottom_right: self.bottom_left,
        }
    }

    /// Top and bottom swapped.
    const fn mirrored_down(self) -> Self {
        Self {
            top_left: self.bottom_left,
            top_right: self.bottom_right,
            bottom_left: self.top_left,
            bottom_right: self.top_right,
        }
    }
}

/// The looks of the blocks `exposure` brings into view, in its line's order.
pub fn exposed_looks<'a>(
    room: &'a RoomScoped,
    tables: &'a TileTables,
    exposure: &LayerExposure,
) -> impl Iterator<Item = BlockLook> + 'a {
    let level = Level { room, tables };
    let start = level.index(exposure.level);
    let stride = match exposure.line {
        BlockLine::Column => room.width.0,
        BlockLine::Row => 1,
    };
    let layer = exposure.layer;
    (0..exposure.line.blocks())
        .map(move |step| level.drawn(layer, start.wrapping_add(step.wrapping_mul(stride))))
}

#[cfg(test)]
mod tests;
