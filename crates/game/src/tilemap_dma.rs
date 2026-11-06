//! The NMI's scrolling DMAs: the tilemap columns and rows the BG graphics update built.
//!
//! Bank `$80` log, handle VRAM write table and scrolling DMAs `$80:8C83`, after the write
//! table: execute horizontal scrolling DMAs (`$80:8CD8`) then vertical (`$80:8DAC`). Each does
//! BG1's upload then BG2's, when the upload's request flag has its low byte set, and clears it.
//! An upload is two DMAs per part on channel 1, one for each half: the first part, then the
//! wrapped part unless its size is 0.

use data::dma::{BG1_COLUMN_BUFFER, BG1_ROW_BUFFER, BG2_BUFFER_OFFSET};

use crate::commit::{DMA_TO_VRAM, DMA_TWO_REGISTERS, VRAM_CHANNEL, write_words};
use crate::state::video::{
    BusAddress, ColumnUpload, DmaChannel, RowUpload, TilemapEntry, TilemapPart, Video, VramStep,
    VramWord,
};

/// The words between a tilemap row's two 8x8 rows: the row upload's bottom half lands
/// `| 20h` after the top.
const ROW_HALF_OFFSET: u16 = 0x0020;
/// Bytes in a word.
const WORD_BYTES: u16 = 2;

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
        channel.control = DMA_TWO_REGISTERS;
        channel.destination = DMA_TO_VRAM;
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

impl Video {
    /// Bank `$80` log, `$80:8CD8` and `$80:8DAC`: the uploads the BG graphics update built go to
    /// VRAM through channel 1, BG1's column, BG2's, BG1's row, then BG2's, and their requests
    /// clear. Channel 1 keeps what the last DMA left.
    pub fn execute_scrolling_dmas(&mut self) {
        let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) else {
            return;
        };
        let vram = &mut self.memory.vram;
        let level = &mut self.batch.level;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::video::UploadRequests;

    /// An entry whose word is `word`.
    fn entry(word: u16) -> TilemapEntry {
        TilemapEntry::from_word(word)
    }

    fn column(first: u16, wrapped: u16) -> ColumnUpload {
        let mut upload = ColumnUpload {
            first: TilemapPart {
                to: VramWord(0x5010),
                bytes: first,
            },
            wrapped: TilemapPart {
                to: VramWord(0x5000),
                bytes: wrapped,
            },
            requests: UploadRequests(1),
            ..ColumnUpload::default()
        };
        for (index, (left, right)) in upload.left.iter_mut().zip(&mut upload.right).enumerate() {
            let index = u16::try_from(index).unwrap_or_default();
            *left = entry(0x100_u16.wrapping_add(index));
            *right = entry(0x200_u16.wrapping_add(index));
        }
        upload
    }

    fn row(first: u16, wrapped: u16) -> RowUpload {
        let mut upload = RowUpload {
            first: TilemapPart {
                to: VramWord(0x5042),
                bytes: first,
            },
            wrapped: TilemapPart {
                to: VramWord(0x5400),
                bytes: wrapped,
            },
            requests: UploadRequests(1),
            ..RowUpload::default()
        };
        for (index, (top, bottom)) in upload.top.iter_mut().zip(&mut upload.bottom).enumerate() {
            let index = u16::try_from(index).unwrap_or_default();
            *top = entry(0x300_u16.wrapping_add(index));
            *bottom = entry(0x400_u16.wrapping_add(index));
        }
        upload
    }

    fn vram(video: &Video, from: u16, count: usize) -> Vec<u16> {
        video
            .memory
            .vram
            .iter()
            .skip(usize::from(from))
            .take(count)
            .copied()
            .collect()
    }

    #[test]
    fn a_column_goes_down_the_tilemap_in_both_halves_and_wraps() {
        let mut video = Video::default();
        video.batch.level.bg1.column = column(8, 56);
        video.execute_scrolling_dmas();
        // First part: 4 words down from 5010h, the right half at 5011h.
        assert_eq!(vram(&video, 0x5010, 1), [0x100]);
        assert_eq!(vram(&video, 0x5030, 1), [0x101]);
        assert_eq!(vram(&video, 0x5011, 1), [0x200]);
        assert_eq!(vram(&video, 0x5071, 1), [0x203]);
        // The wrapped part continues from the buffer's word 4, at the top.
        assert_eq!(vram(&video, 0x5000, 1), [0x104]);
        assert_eq!(vram(&video, 0x5001, 1), [0x204]);
        assert_eq!(vram(&video, 0x5000 + 27 * 32, 1), [0x104 + 27]);
        // Right half of the wrapped part: source = buffer + 0x40 + first.bytes.
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x01,
                destination: 0x18,
                source: 0xC908 + 8 + 56,
                source_bank: 0x7E,
                count: 0,
                ..DmaChannel::default()
            }
        );
        assert_eq!(video.batch.level.bg1.column.requests, UploadRequests(0));
    }

    #[test]
    fn a_wrapped_size_of_zero_leaves_the_channel_at_the_wrapped_source() {
        let mut video = Video::default();
        video.batch.level.bg1.column = column(64, 0);
        video.execute_scrolling_dmas();
        assert_eq!(video.dma[1].source, 0xC8C8 + 64);
        assert_eq!(video.dma[1].count, 0);
        assert_eq!(video.dma[1].source_bank, 0x7E);
        // Nothing was written at the wrapped target.
        assert_eq!(vram(&video, 0x5000, 1), [0]);
    }

    #[test]
    fn a_row_goes_along_the_tilemap_and_its_bottom_half_is_20h_on() {
        let mut video = Video::default();
        video.batch.level.bg1.row = row(8, 60);
        video.execute_scrolling_dmas();
        assert_eq!(vram(&video, 0x5042, 4), [0x300, 0x301, 0x302, 0x303]);
        assert_eq!(vram(&video, 0x5062, 4), [0x400, 0x401, 0x402, 0x403]);
        // The wrapped part: the top from word 4, the bottom likewise, at 5400h and 5420h.
        assert_eq!(vram(&video, 0x5400, 2), [0x304, 0x305]);
        assert_eq!(vram(&video, 0x5420, 2), [0x404, 0x405]);
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x01,
                destination: 0x18,
                source: 0xC98C + 8 + 60,
                source_bank: 0x7E,
                count: 0,
                ..DmaChannel::default()
            }
        );
        assert_eq!(video.batch.level.bg1.row.requests, UploadRequests(0));
    }

    #[test]
    fn bg2_reads_its_own_buffers_after_bg1() {
        let mut video = Video::default();
        video.batch.level.bg1.column = column(64, 0);
        video.batch.level.bg2.column = column(60, 4);
        video.batch.level.bg2.row = row(68, 0);
        video.execute_scrolling_dmas();
        // The row's wrapped size of 0 ends the last DMA: BG2's row buffer + 68.
        assert_eq!(video.dma[1].source, 0xC948 + BG2_BUFFER_OFFSET + 68);
        let mut video = Video::default();
        video.batch.level.bg2.column = column(60, 4);
        video.execute_scrolling_dmas();
        assert_eq!(video.dma[1].source, 0xC908 + BG2_BUFFER_OFFSET + 60 + 4);
        assert_eq!(video.batch.level.bg2.column.requests, UploadRequests(0));
    }

    #[test]
    fn only_the_low_byte_gates_and_the_high_byte_stays() {
        let mut video = Video::default();
        video.batch.level.bg1.column = column(64, 0);
        video.batch.level.bg1.column.requests = UploadRequests(0x0300);
        video.execute_scrolling_dmas();
        assert_eq!(video.dma[1], DmaChannel::default());
        video.batch.level.bg1.column.requests = UploadRequests(0x0305);
        video.execute_scrolling_dmas();
        assert_eq!(
            video.batch.level.bg1.column.requests,
            UploadRequests(0x0300)
        );
        assert_ne!(video.dma[1], DmaChannel::default());
    }
}
