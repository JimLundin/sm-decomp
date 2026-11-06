//! The PLM draw (bank $84 log, `$84:8DAA`): a block row a PLM draws is copied into the BG1
//! tilemap through the VRAM write table, when it is on screen.
//!
//! The draw first clips the row to the screen: rows of blocks above the screen's top or below
//! its fifteenth row are not drawn, nor are blocks left of the one before the screen or right
//! of the 17th past it. Each block's four tilemap words come from the tile table, and the
//! rows land in the BG1 tilemap's screen the scroll offset says holds that column, the top
//! 8x8 halves in one queued upload and the bottom halves in the next. A row that runs off the
//! end of its tilemap screen continues at the start of the other one, which takes two uploads
//! for each half.
// open: the draw instructions that draw a column (`$84:901F`) and the ones with more than one
// row or column; slice 0's PLMs draw one row of blocks.

use data::dma::PLM_DRAW_TILEMAP;
use data::level::Tile;
use data::tileset::TileTables;

use crate::bg_graphics::{BG1_TILEMAP, tile_words};
use crate::state::room::{BlockIndex, RoomScoped};
use crate::state::video::{BusAddress, Video, VramSource, VramStep, VramTransfer, VramWord};

/// Where BG1's second tilemap screen starts: one screen's words on from the first.
const SECOND_SCREEN: u16 = 0x0400;
/// The words in a tilemap row, which the bottom half of a block lies below the top.
const ROW_WORDS: u16 = 0x0020;
/// The words a block takes along one 8x8 row of the tilemap.
const BLOCK_ROW_WORDS: u16 = 2;
/// The words in a block row of the tilemap: two 8x8 rows.
const BLOCK_ROW_STRIDE: u16 = 0x0040;
/// How many blocks across a tilemap screen is: its block columns repeat after this.
const SCREEN_COLUMNS: u16 = 0x10;
/// The block columns a tilemap holds, over both its screens.
const COLUMN_MASK: u16 = 0x1F;
/// The block rows a tilemap screen holds.
const ROW_MASK: u16 = 0x0F;
/// The rows of blocks the screen shows, less one.
const SCREEN_ROWS: u16 = 0x0F;
/// The columns of blocks from the one left of the screen to the one right of it.
const SCREEN_SPAN: u16 = 0x11;
/// Whether BG1's X scroll offset has put the other tilemap screen first.
const OTHER_SCREEN_FIRST: u16 = 0x0100;
/// The bytes of the draw tilemap buffer a block takes: its four words, in the two halves'
/// rows.
const BLOCK_BUFFER_BYTES: u16 = 8;
/// The buffer's bytes; a draw that would pass them draws nothing.
const BUFFER_BYTES: u16 = 0x0200;
/// The bytes of a VRAM write table entry.
const TABLE_ENTRY_BYTES: u16 = 7;
/// The write table's size at which a draw gives up: one entry pair's room.
const TABLE_FULL: u16 = 0x01E0;
/// The write table's size past which a draw that needs four entries gives up.
const TABLE_FULL_FOR_SPLIT: u16 = 0x00E4;

/// The game's video the PLM draw queues into, and the tile tables it draws with.
#[derive(Debug)]
pub struct Drawing<'a> {
    /// The queue of VRAM uploads the next NMI flushes.
    video: &'a mut Video,
    /// Where the block numbers a PLM draws find their four tiles.
    tables: &'a TileTables,
    /// Where the next block lands in the draw tilemap buffer, in bytes (`$1C25`): the pass
    /// starts it at 0 and every drawn block takes 8.
    index: u16,
}

impl<'a> Drawing<'a> {
    /// A pass's draws, the buffer at its start.
    pub const fn new(video: &'a mut Video, tables: &'a TileTables) -> Self {
        Self {
            video,
            tables,
            index: 0,
        }
    }

    /// The write table's size in bytes (`$0330`): every upload takes one entry.
    fn table_size(&self) -> u16 {
        u16::try_from(self.video.batch.vram.len())
            .unwrap_or(u16::MAX)
            .wrapping_mul(TABLE_ENTRY_BYTES)
    }

    /// Queue an upload of `words`, read from `source` in the draw tilemap buffer, to the
    /// tilemap row at `to`.
    fn queue(&mut self, to: VramWord, source: BusAddress, words: &[u16]) {
        self.video.batch.vram.push(VramTransfer {
            to,
            step: VramStep::Row,
            source: VramSource::Words {
                from: source,
                words: words.to_vec(),
            },
        });
    }

    /// Draw the block at `block` in `room`, if it is on screen (`$84:8DAA` with a draw
    /// instruction of one block).
    pub fn block(&mut self, room: &RoomScoped, block: BlockIndex, tile: Tile) {
        self.row(room, block, &[tile]);
    }

    /// Draw the row of `tiles` that starts at `block`.
    fn row(&mut self, room: &RoomScoped, block: BlockIndex, tiles: &[Tile]) {
        let Some(clip) = Clip::of(room, block, tiles.len()) else {
            return;
        };
        let size = self.table_size();
        if size >= TABLE_FULL {
            return;
        }
        // The buffer must hold the row: its free bytes, in blocks, against the length.
        let room_left = BUFFER_BYTES.wrapping_sub(self.index) >> 3;
        if is_negative(room_left.wrapping_sub(clip.length)) {
            return;
        }
        let to = destination(room, clip.first, clip.row);
        let split = (to.0 & COLUMN_MASK).wrapping_add(clip.length.wrapping_mul(BLOCK_ROW_WORDS))
            > ROW_WORDS;
        if split && size >= TABLE_FULL_FOR_SPLIT {
            return;
        }
        let visible = tiles
            .iter()
            .skip(usize::from(clip.skipped))
            .take(usize::from(clip.length));
        let (top, bottom): (Vec<_>, Vec<_>) = visible
            .map(|&tile| {
                let [top_left, top_right, bottom_left, bottom_right] =
                    tile_words(room, self.tables, tile);
                ([top_left, top_right], [bottom_left, bottom_right])
            })
            .unzip();
        let (top, bottom): (Vec<u16>, Vec<u16>) = (top.concat(), bottom.concat());
        let top_source = PLM_DRAW_TILEMAP.advanced(self.index);
        let row_bytes = clip.length.wrapping_mul(BLOCK_ROW_WORDS).wrapping_mul(2);
        let bottom_source = top_source.advanced(row_bytes);
        let below = |at: VramWord| VramWord(at.0.wrapping_add(ROW_WORDS));
        if split {
            // Up to the end of the tilemap row on this screen, the rest from the start of the
            // same row on the other.
            let first_words = usize::from(ROW_WORDS.wrapping_sub(to.0 & COLUMN_MASK));
            let other = VramWord((to.0 & !COLUMN_MASK) ^ SECOND_SCREEN);
            let (top_first, top_rest) = top.split_at(first_words.min(top.len()));
            let (bottom_first, bottom_rest) = bottom.split_at(first_words.min(bottom.len()));
            let first_bytes = u16::try_from(top_first.len().wrapping_mul(2)).unwrap_or_default();
            self.queue(to, top_source, top_first);
            self.queue(other, top_source.advanced(first_bytes), top_rest);
            self.queue(below(to), bottom_source, bottom_first);
            self.queue(
                below(other),
                bottom_source.advanced(first_bytes),
                bottom_rest,
            );
        } else {
            self.queue(to, top_source, &top);
            self.queue(below(to), bottom_source, &bottom);
        }
        self.index = self
            .index
            .wrapping_add(clip.length.wrapping_mul(BLOCK_BUFFER_BYTES));
    }
}

/// Whether a 16-bit difference is negative as the original's flags see it.
const fn is_negative(difference: u16) -> bool {
    difference.cast_signed() < 0
}

/// The part of a row of blocks the screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Clip {
    /// The block row.
    row: u16,
    /// The first block column drawn.
    first: u16,
    /// The blocks the row's start lies before the first drawn.
    skipped: u16,
    /// The blocks drawn.
    length: u16,
}

impl Clip {
    /// What of a row of `length` blocks starting at `block` shows (`$84:8DC5` to `$84:8E44`),
    /// or `None` if none does.
    fn of(room: &RoomScoped, block: BlockIndex, length: usize) -> Option<Self> {
        let width = room.width.0;
        let x = block.0.checked_rem(width).unwrap_or_default();
        let y = block.0.checked_div(width).unwrap_or_default();
        let screen_y = room.camera.y.pixels >> 4;
        // Below the screen's last row, then above its first.
        if is_negative(screen_y.wrapping_add(SCREEN_ROWS).wrapping_sub(y))
            || is_negative(y.wrapping_sub(screen_y))
        {
            return None;
        }
        let mut length = u16::try_from(length).ok().filter(|&length| length != 0)?;
        let mut first = x;
        let mut skipped = 0;
        // The block before the screen's left edge, which a draw starts from.
        let left = (room.camera.x.pixels.wrapping_add(0x0F) >> 4).wrapping_sub(1);
        let before = left.wrapping_sub(x);
        if !is_negative(before) && before != 0 {
            skipped = before;
            // Entirely left of the screen.
            if !is_positive(x.wrapping_add(length).wrapping_sub(left)) {
                return None;
            }
            length = length.wrapping_sub(skipped);
            first = left;
        }
        let right = left.wrapping_add(SCREEN_SPAN);
        if is_negative(right.wrapping_sub(x)) {
            return None;
        }
        let truncated = first
            .wrapping_add(length)
            .wrapping_sub(1)
            .wrapping_sub(right);
        if !is_negative(truncated) {
            length = length.wrapping_sub(truncated);
            if length == 0 {
                return None;
            }
        }
        Some(Self {
            row: y,
            first,
            skipped,
            length,
        })
    }
}

/// Whether a 16-bit difference is greater than zero.
const fn is_positive(difference: u16) -> bool {
    difference != 0 && !is_negative(difference)
}

/// The tilemap word a block row's first block lands on (`$84:8E5F` to `$84:8EA1`): its column
/// in the screen that column is in, which BG1's scroll offset may have swapped.
const fn destination(room: &RoomScoped, column: u16, row: u16) -> VramWord {
    let row_start = (row & ROW_MASK).wrapping_mul(BLOCK_ROW_STRIDE);
    let word = (column & COLUMN_MASK).wrapping_mul(BLOCK_ROW_WORDS);
    let swapped = room.camera.bg1_offset.x & OTHER_SCREEN_FIRST != 0;
    let in_first_screen = column & COLUMN_MASK < SCREEN_COLUMNS;
    let first_screen = BG1_TILEMAP.0;
    let second_screen = first_screen.wrapping_add(SECOND_SCREEN);
    // The second screen's columns start at its 16th block column, which `word` counts from the
    // first screen's start.
    let base = if in_first_screen {
        first_screen
    } else {
        second_screen.wrapping_sub(ROW_WORDS)
    };
    let at = word.wrapping_add(base).wrapping_add(row_start);
    VramWord(match (in_first_screen, swapped) {
        (true, true) => at.wrapping_add(SECOND_SCREEN),
        (false, true) => at.wrapping_sub(SECOND_SCREEN),
        (_, false) => at,
    })
}

#[cfg(test)]
mod tests;
