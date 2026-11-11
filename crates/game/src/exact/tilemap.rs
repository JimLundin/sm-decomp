//! The VRAM write queue's tilemap uploads: each tile write as the original's queued transfers.
//!
//! A write lands where the original's tilemaps hold it, in the order the original queued it:
//!
//! - The overlay's rows (the HUD's) go up as one upload read from the HUD's tilemap buffer,
//!   which holds the overlay from its second row.
//! - A row of the room's blocks lands in BG1's tilemap only while it is on screen: two 32x32
//!   screens side by side that hold the 32 block columns around the camera, the screen BG1's
//!   scroll offset puts first swapped. Each block's top cells go up in one upload and its
//!   bottom cells in the next, read from the PLM draw's tilemap buffer, which a pass fills
//!   from its start. A row that runs off the end of its tilemap screen continues at the start
//!   of the other one, which takes two uploads for each half.
//! - The queue holds a limited number of uploads: a block row that would pass it, or pass the
//!   PLM draw's buffer, is dropped whole.
//!
//! The lines of blocks the camera brings into view stream in apart from the queue, through the
//! layers' column and row buffers ([`expose`], [`commit_exposures`]).

use data::dma::{BusAddress, HUD_TILEMAP_BUFFER, PLM_DRAW_TILEMAP};

use crate::intent::{BlockLook, TileWrite};
use crate::state::room::{BlockPoint, Camera};
use crate::state::video::{NmiBatch, TilemapEntry, VramSource, VramStep, VramTransfer, VramWord};

mod exposure;

pub use exposure::{commit_exposures, expose};

/// BG1's tilemap in VRAM, two 32x32 screens side by side.
pub const BG1_TILEMAP: VramWord = VramWord(0x5000);
/// The overlay's tilemap in VRAM: BG3's above the playfield, 32 cells across.
const OVERLAY_TILEMAP: VramWord = VramWord(0x5800);
/// The overlay's row the HUD's tilemap buffer starts with.
const HUD_BUFFER_ROW: u16 = 1;

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
/// The bytes of the PLM draw's buffer a block takes: its four words, in the two halves' rows.
const BLOCK_BUFFER_BYTES: u16 = 8;
/// The PLM draw buffer's bytes; a row that would pass them is dropped.
const BUFFER_BYTES: u16 = 0x0200;
/// The bytes of a VRAM write queue entry.
const QUEUE_ENTRY_BYTES: u16 = 7;
/// The queue's size at which a block row is dropped: one entry pair's room.
const QUEUE_FULL: u16 = 0x01E0;
/// The queue's size past which a block row that needs four entries is dropped.
const QUEUE_FULL_FOR_SPLIT: u16 = 0x00E4;
/// Bytes in a word.
const WORD_BYTES: u16 = 2;

/// The VRAM write queue as a pass's tile writes fill it.
#[derive(Debug)]
pub struct TileQueue<'a> {
    /// The batch whose queue the next NMI makes.
    batch: &'a mut NmiBatch,
    /// Where the next block lands in the PLM draw's buffer, in bytes: a pass starts it at 0
    /// and every drawn block takes 8.
    buffer: u16,
}

impl<'a> TileQueue<'a> {
    /// A pass's writes, the PLM draw's buffer at its start.
    pub const fn new(batch: &'a mut NmiBatch) -> Self {
        Self { batch, buffer: 0 }
    }

    /// Queue `write` where the original queued it; a block row is placed from `camera`.
    pub fn write(&mut self, camera: &Camera, write: &TileWrite) {
        match write {
            TileWrite::Overlay { row, cells } => self.overlay(*row, cells),
            TileWrite::Blocks { at, looks } => self.blocks(camera, *at, looks),
        }
    }

    /// The queue's size in bytes: every upload takes one entry.
    fn size(&self) -> u16 {
        u16::try_from(self.batch.vram.len())
            .unwrap_or(u16::MAX)
            .wrapping_mul(QUEUE_ENTRY_BYTES)
    }

    /// Queue an upload of `words`, read from `source`, along the tilemap row at `to`.
    fn queue(&mut self, to: VramWord, source: BusAddress, words: impl Iterator<Item = u16>) {
        self.batch.vram.push(VramTransfer {
            to,
            step: VramStep::Row,
            source: VramSource::Words {
                from: source,
                words: words.collect(),
            },
        });
    }

    /// The overlay's rows from `row`, as one upload.
    fn overlay(&mut self, row: u16, cells: &[TilemapEntry]) {
        let to = VramWord(OVERLAY_TILEMAP.0.wrapping_add(row.wrapping_mul(ROW_WORDS)));
        let source = HUD_TILEMAP_BUFFER.advanced(
            row.wrapping_sub(HUD_BUFFER_ROW)
                .wrapping_mul(ROW_WORDS)
                .wrapping_mul(WORD_BYTES),
        );
        self.queue(to, source, cells.iter().map(|cell| cell.word()));
    }

    /// The row of blocks `looks` from the block at `at`, if any of it is on screen.
    fn blocks(&mut self, camera: &Camera, at: BlockPoint, looks: &[BlockLook]) {
        let Some(clip) = Clip::of(camera, at, looks.len()) else {
            return;
        };
        let size = self.size();
        if size >= QUEUE_FULL {
            return;
        }
        // The buffer must hold the row: its free bytes, in blocks, against the length.
        let room_left = BUFFER_BYTES.wrapping_sub(self.buffer) >> 3;
        if is_negative(room_left.wrapping_sub(clip.length)) {
            return;
        }
        let to = destination(camera, clip.first, clip.row);
        let split = (to.0 & COLUMN_MASK).wrapping_add(clip.length.wrapping_mul(BLOCK_ROW_WORDS))
            > ROW_WORDS;
        if split && size >= QUEUE_FULL_FOR_SPLIT {
            return;
        }
        let visible: Vec<BlockLook> = looks
            .iter()
            .skip(usize::from(clip.skipped))
            .take(usize::from(clip.length))
            .copied()
            .collect();
        let top: Vec<u16> = visible
            .iter()
            .flat_map(|look| [look.top_left, look.top_right])
            .map(TilemapEntry::word)
            .collect();
        let bottom: Vec<u16> = visible
            .iter()
            .flat_map(|look| [look.bottom_left, look.bottom_right])
            .map(TilemapEntry::word)
            .collect();
        let top_source = PLM_DRAW_TILEMAP.advanced(self.buffer);
        let row_bytes = clip
            .length
            .wrapping_mul(BLOCK_ROW_WORDS)
            .wrapping_mul(WORD_BYTES);
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
            self.queue(to, top_source, top_first.iter().copied());
            self.queue(
                other,
                top_source.advanced(first_bytes),
                top_rest.iter().copied(),
            );
            self.queue(below(to), bottom_source, bottom_first.iter().copied());
            self.queue(
                below(other),
                bottom_source.advanced(first_bytes),
                bottom_rest.iter().copied(),
            );
        } else {
            self.queue(to, top_source, top.into_iter());
            self.queue(below(to), bottom_source, bottom.into_iter());
        }
        self.buffer = self
            .buffer
            .wrapping_add(clip.length.wrapping_mul(BLOCK_BUFFER_BYTES));
    }
}

/// Whether a 16-bit difference is negative as the original's flags see it.
const fn is_negative(difference: u16) -> bool {
    difference.cast_signed() < 0
}

/// Whether a 16-bit difference is greater than zero.
const fn is_positive(difference: u16) -> bool {
    difference != 0 && !is_negative(difference)
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
    /// What of a row of `length` blocks starting at `at` shows, or `None` if none does: rows
    /// above the screen's top or below its fifteenth row show nothing, nor do blocks left of
    /// the one before the screen or right of the 17th past it.
    fn of(camera: &Camera, at: BlockPoint, length: usize) -> Option<Self> {
        let BlockPoint { x, y } = at;
        let screen_y = camera.y.pixels >> 4;
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
        let left = (camera.x.pixels.wrapping_add(0x0F) >> 4).wrapping_sub(1);
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

/// The tilemap word a block row's first block lands on: its column in the screen that column
/// is in, which BG1's scroll offset may have swapped.
const fn destination(camera: &Camera, column: u16, row: u16) -> VramWord {
    let row_start = (row & ROW_MASK).wrapping_mul(BLOCK_ROW_STRIDE);
    let word = (column & COLUMN_MASK).wrapping_mul(BLOCK_ROW_WORDS);
    let swapped = camera.bg1_offset.x & OTHER_SCREEN_FIRST != 0;
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
