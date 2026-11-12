//! What the PLM pass draws: a block it changes, as a tile write of the block's look, which
//! the VRAM write queue's encoder (`exact::tilemap`) places on the tilemap while it is on
//! screen.
// open: the draw instructions that draw a column (`$84:901F`) and the ones with more than one
// row or column; slice 0's PLMs draw one row of blocks.

use data::assets::tilesets::Tilesets;
use data::level::Tile;

use crate::bg_graphics::block_look;
use crate::exact::tilemap::TileQueue;
use crate::intent::TileWrite;
use crate::state::room::{BlockIndex, BlockPoint, RoomScoped};
use crate::state::video::Video;

/// A PLM pass's draws: the write queue they fill, and the tilesets blocks look up.
#[derive(Debug)]
pub struct Drawing<'a> {
    queue: TileQueue<'a>,
    tilesets: &'a Tilesets,
}

impl<'a> Drawing<'a> {
    /// A pass's draws, the PLM draw's buffer at its start.
    pub const fn new(video: &'a mut Video, tilesets: &'a Tilesets) -> Self {
        Self {
            queue: TileQueue::new(&mut video.batch),
            tilesets,
        }
    }

    /// Draw the block at `block` in `room` with `tile`'s look (`$84:8DAA` with a draw
    /// instruction of one block).
    pub fn block(&mut self, room: &RoomScoped, block: BlockIndex, tile: Tile) {
        let width = room.width.0;
        let at = BlockPoint {
            x: block.0.checked_rem(width).unwrap_or_default(),
            y: block.0.checked_div(width).unwrap_or_default(),
        };
        let looks = vec![block_look(room, self.tilesets, tile)];
        self.queue
            .write(&room.camera, &TileWrite::Blocks { at, looks });
    }
}
