//! The lines of blocks coming into view, as the original streams them into the tilemaps.
//!
//! A background's tilemap is two 32x32 screens side by side: a ring of 32 block columns and 16
//! block rows around the camera. BG1's starts at [`BG1_TILEMAP`]; BG2's lies the room's tilemap
//! gap before it. A line lands where its background's block falls on the ring:
//!
//! - A column goes down one screen. The part of it past the screen's last block row wraps to the
//!   same column at the screen's top.
//! - A row goes across from its block to the end of that screen, and its rest wraps to the start
//!   of the same row on the other screen.
//!
//! Each line is built into its layer's column or row buffer (`video.batch.level`), State the
//! original keeps in work RAM, and counted as a request. The NMI makes each requested upload
//! (`commit_exposures`): two DMAs per part on channel 1, one for each half, the first part and
//! then the wrapped part unless its size is 0. A lag frame makes no upload, so the buffers and
//! their requests carry to the next NMI.

use data::dma::{BG1_COLUMN_BUFFER, BG1_ROW_BUFFER, BG2_BUFFER_OFFSET};

use super::BG1_TILEMAP;
use crate::commit::{VRAM_CHANNEL, write_words};
use crate::intent::{BlockLine, BlockLook, LayerExposure, LevelLayer};
use crate::state::room::BlockPoint;
use crate::state::video::{
    BBusRegister, BusAddress, ColumnUpload, DmaChannel, DmaControl, LevelUploads, RowUpload,
    TilemapEntry, TilemapPart, Video, VramStep, VramWord,
};

/// A 32x32 tilemap screen's words.
const SCREEN_WORDS: u16 = 0x0400;
/// A block row's words in a tilemap screen: two rows of 32.
const BLOCK_ROW_WORDS: u16 = 0x0040;
/// The block columns or rows of a tilemap screen.
const SCREEN_BLOCKS: u16 = 0x10;
/// The bytes a block adds to a line's part: two words.
const BLOCK_BYTES: u16 = 4;
/// The words between a tilemap row's two 8x8 rows: a row's bottom half lands this far after
/// its top.
const ROW_HALF_OFFSET: u16 = 0x0020;
/// Bytes in a word.
const WORD_BYTES: u16 = 2;

/// Where a background's tilemap screens start and where in them a block lands.
#[derive(Clone, Copy, Debug)]
struct Tilemap {
    /// The background's first screen.
    base: VramWord,
}

impl Tilemap {
    /// `layer`'s background's tilemap: BG2's lies `gap` words before BG1's.
    const fn of(layer: LevelLayer, gap: VramWord) -> Self {
        let base = match layer {
            LevelLayer::One => BG1_TILEMAP,
            LevelLayer::Two => VramWord(BG1_TILEMAP.0.wrapping_sub(gap.0)),
        };
        Self { base }
    }

    /// The screen block column `column` is in, and the other one.
    const fn screens(self, column: u16) -> (VramWord, VramWord) {
        let left = self.base;
        let right = VramWord(self.base.0.wrapping_add(SCREEN_WORDS));
        if column & SCREEN_BLOCKS == 0 {
            (left, right)
        } else {
            (right, left)
        }
    }

    /// The word `row` block rows down and `column` block columns across a screen, each
    /// counted around the screen.
    const fn at(screen: VramWord, row: u16, column: u16) -> VramWord {
        let row = row % SCREEN_BLOCKS;
        let column = column % SCREEN_BLOCKS;
        VramWord(
            screen
                .0
                .wrapping_add(row.wrapping_mul(BLOCK_ROW_WORDS))
                .wrapping_add(column.wrapping_mul(2)),
        )
    }
}

/// Build `exposure`'s line, its blocks `looks`, into its layer's column or row buffer, placed on
/// its background's tilemap; BG2's lies `gap` words before BG1's.
pub fn expose(
    uploads: &mut LevelUploads,
    exposure: &LayerExposure,
    gap: VramWord,
    looks: impl IntoIterator<Item = BlockLook>,
) {
    let tilemap = Tilemap::of(exposure.layer, gap);
    let layer = match exposure.layer {
        LevelLayer::One => &mut uploads.bg1,
        LevelLayer::Two => &mut uploads.bg2,
    };
    match exposure.line {
        BlockLine::Column => column(&mut layer.column, tilemap, exposure.background, looks),
        BlockLine::Row => row(&mut layer.row, tilemap, exposure.background, looks),
    }
}

/// A column landing at the background's block `at`: down its screen, wrapping to the top.
fn column(
    upload: &mut ColumnUpload,
    tilemap: Tilemap,
    at: BlockPoint,
    looks: impl IntoIterator<Item = BlockLook>,
) {
    // The blocks from the screen's top to the column's start are the ones past its last row.
    let wrapped = (at.y % SCREEN_BLOCKS).wrapping_mul(BLOCK_BYTES);
    let (screen, _) = tilemap.screens(at.x);
    upload.first = TilemapPart {
        to: Tilemap::at(screen, at.y, at.x),
        bytes: BlockLine::Column
            .blocks()
            .wrapping_mul(BLOCK_BYTES)
            .wrapping_sub(wrapped),
    };
    upload.wrapped = TilemapPart {
        to: Tilemap::at(screen, 0, at.x),
        bytes: wrapped,
    };
    let halves = upload
        .left
        .as_chunks_mut::<2>()
        .0
        .iter_mut()
        .zip(upload.right.as_chunks_mut::<2>().0.iter_mut());
    for (look, (left, right)) in looks.into_iter().zip(halves) {
        *left = [look.top_left, look.bottom_left];
        *right = [look.top_right, look.bottom_right];
    }
    upload.requests = upload.requests.requested();
}

/// A row landing at the background's block `at`: across to its screen's end, the rest from the
/// other screen's start.
fn row(
    upload: &mut RowUpload,
    tilemap: Tilemap,
    at: BlockPoint,
    looks: impl IntoIterator<Item = BlockLook>,
) {
    let column = at.x % SCREEN_BLOCKS;
    let (screen, other) = tilemap.screens(at.x);
    upload.first = TilemapPart {
        to: Tilemap::at(screen, at.y, at.x),
        bytes: SCREEN_BLOCKS.wrapping_sub(column).wrapping_mul(BLOCK_BYTES),
    };
    upload.wrapped = TilemapPart {
        to: Tilemap::at(other, at.y, 0),
        bytes: column.wrapping_add(1).wrapping_mul(BLOCK_BYTES),
    };
    let halves = upload
        .top
        .as_chunks_mut::<2>()
        .0
        .iter_mut()
        .zip(upload.bottom.as_chunks_mut::<2>().0.iter_mut());
    for (look, (top, bottom)) in looks.into_iter().zip(halves) {
        *top = [look.top_left, look.top_right];
        *bottom = [look.bottom_left, look.bottom_right];
    }
    upload.requests = upload.requests.requested();
}

/// Which way an upload runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// A block column: its right half goes one word after the left, down the tilemap.
    Column,
    /// A block row: its bottom half goes `ROW_HALF_OFFSET` after the top, along the tilemap.
    Row,
}

impl Shape {
    /// How VRAM's address moves for this shape.
    const fn step(self) -> VramStep {
        match self {
            Self::Column => VramStep::Column,
            Self::Row => VramStep::Row,
        }
    }

    /// Where the second half of a part lands, given where the first does.
    const fn second_half(self, to: VramWord) -> VramWord {
        match self {
            Self::Column => VramWord(to.0.wrapping_add(1)),
            Self::Row => VramWord(to.0 | ROW_HALF_OFFSET),
        }
    }

    /// BG1's first half's buffer.
    const fn buffer(self) -> BusAddress {
        match self {
            Self::Column => BG1_COLUMN_BUFFER,
            Self::Row => BG1_ROW_BUFFER,
        }
    }
}

/// One upload as the DMAs see it: its two halves, where its buffer is and its two parts.
struct Upload<'a> {
    shape: Shape,
    halves: [&'a [TilemapEntry]; 2],
    /// The first half's buffer, which the wrapped part's sources lie on from.
    buffer: BusAddress,
    first: TilemapPart,
    wrapped: TilemapPart,
}

impl<'a> Upload<'a> {
    /// A BG's column upload; BG2's buffers lie `layer_offset` bytes after BG1's.
    const fn column(upload: &'a ColumnUpload, layer_offset: u16) -> Self {
        Self {
            shape: Shape::Column,
            halves: [&upload.left, &upload.right],
            buffer: Shape::Column.buffer().advanced(layer_offset),
            first: upload.first,
            wrapped: upload.wrapped,
        }
    }

    /// A BG's row upload; BG2's buffers lie `layer_offset` bytes after BG1's.
    const fn row(upload: &'a RowUpload, layer_offset: u16) -> Self {
        Self {
            shape: Shape::Row,
            halves: [&upload.top, &upload.bottom],
            buffer: Shape::Row.buffer().advanced(layer_offset),
            first: upload.first,
            wrapped: upload.wrapped,
        }
    }

    /// The DMAs: the first part's two halves, then the wrapped part's. The channel's source is
    /// written to the wrapped part's first half before its size is tested, so a wrapped size of
    /// 0 leaves it holding that source.
    fn execute(&self, vram: &mut [u16], channel: &mut DmaChannel) {
        channel.control = DmaControl::TO_TWO_REGISTERS;
        channel.destination = BBusRegister::VramData;
        self.part(vram, channel, 0, self.first);
        channel.source = self.buffer.advanced(self.first.bytes).offset;
        if self.wrapped.bytes != 0 {
            self.part(vram, channel, self.first.bytes, self.wrapped);
        }
    }

    /// The bytes in one half's buffer.
    fn half_bytes(&self) -> u16 {
        let [half, _] = self.halves;
        u16::try_from(half.len()).map_or(0, |words| words.wrapping_mul(WORD_BYTES))
    }

    /// One part's two halves, from `skip` bytes into each half's buffer.
    fn part(&self, vram: &mut [u16], channel: &mut DmaChannel, skip: u16, part: TilemapPart) {
        let words = |bytes: u16| usize::from(bytes / WORD_BYTES);
        for (index, half) in self.halves.iter().enumerate() {
            let to = if index == 0 {
                part.to
            } else {
                self.shape.second_half(part.to)
            };
            let entries = half
                .iter()
                .skip(words(skip))
                .take(words(part.bytes))
                .map(|entry| entry.word());
            write_words(vram, to, self.shape.step().words(), entries);
            let half_start = if index == 0 {
                self.buffer
            } else {
                self.buffer.advanced(self.half_bytes())
            };
            channel.transferred(half_start.advanced(skip), usize::from(part.bytes));
        }
    }
}

/// The NMI's scrolling DMAs (bank `$80` log, `$80:8CD8` then `$80:8DAC`): each requested line
/// goes to VRAM, and its request clears.
///
/// They run through channel 1, BG1's column, BG2's, BG1's row, then BG2's. A request counts
/// only by its low byte. Channel 1 keeps what the last DMA left.
pub fn commit_exposures(video: &mut Video) {
    let Some(channel) = video.dma.get_mut(VRAM_CHANNEL) else {
        return;
    };
    let vram = &mut video.memory.vram;
    let level = &mut video.batch.level;
    let layers = [0, BG2_BUFFER_OFFSET];
    for (uploads, offset) in [&mut level.bg1, &mut level.bg2].into_iter().zip(layers) {
        if uploads.column.requests.pending() {
            uploads.column.requests = uploads.column.requests.served();
            Upload::column(&uploads.column, offset).execute(vram, channel);
        }
    }
    for (uploads, offset) in [&mut level.bg1, &mut level.bg2].into_iter().zip(layers) {
        if uploads.row.requests.pending() {
            uploads.row.requests = uploads.row.requests.served();
            Upload::row(&uploads.row, offset).execute(vram, channel);
        }
    }
}

#[cfg(test)]
mod tests;
