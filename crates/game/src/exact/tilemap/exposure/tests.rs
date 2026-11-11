use super::*;
use crate::state::video::{LayerUploads, UploadRequests};

/// Where a column buffer's right half starts after its left (16 entries of 4 bytes).
const RIGHT_HALF: u16 = 0x40;

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
    commit_exposures(&mut video);
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
            control: DmaControl::TO_TWO_REGISTERS,
            destination: BBusRegister::VramData,
            source: BG1_COLUMN_BUFFER.offset + RIGHT_HALF + 8 + 56,
            source_bank: BG1_COLUMN_BUFFER.bank,
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
    commit_exposures(&mut video);
    assert_eq!(video.dma[1].source, BG1_COLUMN_BUFFER.offset + 64);
    assert_eq!(video.dma[1].count, 0);
    assert_eq!(video.dma[1].source_bank, BG1_COLUMN_BUFFER.bank);
    // Nothing was written at the wrapped target.
    assert_eq!(vram(&video, 0x5000, 1), [0]);
}

#[test]
fn a_row_goes_along_the_tilemap_and_its_bottom_half_is_20h_on() {
    let mut video = Video::default();
    video.batch.level.bg1.row = row(8, 60);
    commit_exposures(&mut video);
    assert_eq!(vram(&video, 0x5042, 4), [0x300, 0x301, 0x302, 0x303]);
    assert_eq!(vram(&video, 0x5062, 4), [0x400, 0x401, 0x402, 0x403]);
    // The wrapped part: the top from word 4, the bottom likewise, at 5400h and 5420h.
    assert_eq!(vram(&video, 0x5400, 2), [0x304, 0x305]);
    assert_eq!(vram(&video, 0x5420, 2), [0x404, 0x405]);
    assert_eq!(
        video.dma[1],
        DmaChannel {
            control: DmaControl::TO_TWO_REGISTERS,
            destination: BBusRegister::VramData,
            source: BG1_ROW_BUFFER.offset + RIGHT_HALF + 4 + 8 + 60,
            source_bank: BG1_COLUMN_BUFFER.bank,
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
    commit_exposures(&mut video);
    // The row's wrapped size of 0 ends the last DMA: BG2's row buffer + 68.
    assert_eq!(
        video.dma[1].source,
        BG1_ROW_BUFFER.offset + BG2_BUFFER_OFFSET + 68
    );
    let mut video = Video::default();
    video.batch.level.bg2.column = column(60, 4);
    commit_exposures(&mut video);
    assert_eq!(
        video.dma[1].source,
        BG1_COLUMN_BUFFER.offset + RIGHT_HALF + BG2_BUFFER_OFFSET + 60 + 4
    );
    assert_eq!(video.batch.level.bg2.column.requests, UploadRequests(0));
}

#[test]
fn only_the_low_byte_gates_and_the_high_byte_stays() {
    let mut video = Video::default();
    video.batch.level.bg1.column = column(64, 0);
    video.batch.level.bg1.column.requests = UploadRequests(0x0300);
    commit_exposures(&mut video);
    assert_eq!(video.dma[1], DmaChannel::default());
    video.batch.level.bg1.column.requests = UploadRequests(0x0305);
    commit_exposures(&mut video);
    assert_eq!(
        video.batch.level.bg1.column.requests,
        UploadRequests(0x0300)
    );
    assert_ne!(video.dma[1], DmaChannel::default());
}

/// An invented look whose four cells' words are `first` to `first + 3`, top left first.
fn look(first: u16) -> BlockLook {
    let cell = |offset: u16| entry(first.wrapping_add(offset));
    BlockLook {
        top_left: cell(0),
        top_right: cell(1),
        bottom_left: cell(2),
        bottom_right: cell(3),
    }
}

/// The uploads `line` of `layer` makes landing at the background's block (`x`, `y`), each
/// block's look from four times its place in the line, BG2's tilemap `gap` before BG1's.
fn exposed(layer: LevelLayer, line: BlockLine, x: u16, y: u16, gap: u16) -> LevelUploads {
    let mut uploads = LevelUploads::default();
    let exposure = LayerExposure {
        layer,
        line,
        level: BlockPoint::default(),
        background: BlockPoint { x, y },
    };
    let looks = (0..line.blocks()).map(|step| look(step.wrapping_mul(4)));
    expose(&mut uploads, &exposure, VramWord(gap), looks);
    uploads
}

#[test]
fn a_column_on_the_second_screen_goes_down_it_whole() {
    let uploads = exposed(LevelLayer::One, BlockLine::Column, 17, 0, 0);
    let column = uploads.bg1.column;
    assert_eq!(
        (column.first, column.wrapped),
        (
            TilemapPart {
                to: VramWord(0x5402),
                bytes: 0x40
            },
            TilemapPart {
                to: VramWord(0x5402),
                bytes: 0
            }
        )
    );
    // Each block's left cells go down the left half, its right cells down the right.
    assert_eq!(column.left[..3], [entry(0), entry(2), entry(4)]);
    assert_eq!(column.right[..2], [entry(1), entry(3)]);
    assert_eq!(column.requests, UploadRequests(1));
    assert_eq!(uploads.bg1.row, RowUpload::default());
    assert_eq!(uploads.bg2, LayerUploads::default());
}

#[test]
fn a_column_below_the_screens_top_wraps_its_last_blocks_to_the_top() {
    let column = exposed(LevelLayer::One, BlockLine::Column, 4, 3, 0)
        .bg1
        .column;
    assert_eq!(
        (column.first, column.wrapped),
        (
            TilemapPart {
                to: VramWord(0x50C8),
                bytes: 0x34
            },
            TilemapPart {
                to: VramWord(0x5008),
                bytes: 0x0C
            }
        )
    );
}

#[test]
fn a_row_runs_to_its_screens_end_and_wraps_to_the_other_screens_start() {
    // Block row 16 is the screen's first; five blocks across, 11 fit before its right edge
    // and the other 6 of the 17 go on the other screen.
    let row = exposed(LevelLayer::One, BlockLine::Row, 5, 16, 0).bg1.row;
    assert_eq!(
        (row.first, row.wrapped),
        (
            TilemapPart {
                to: VramWord(0x500A),
                bytes: 44
            },
            TilemapPart {
                to: VramWord(0x5400),
                bytes: 24
            }
        )
    );
    assert_eq!(row.top[..3], [entry(0), entry(1), entry(4)]);
    assert_eq!(row.bottom[..2], [entry(2), entry(3)]);
    assert_eq!(row.requests, UploadRequests(1));
}

#[test]
fn a_row_on_the_second_screen_wraps_to_the_first() {
    let row = exposed(LevelLayer::One, BlockLine::Row, 0x13, 2, 0).bg1.row;
    assert_eq!(row.first.to, VramWord(0x5400 + 2 * 0x40 + 3 * 2));
    assert_eq!(row.wrapped.to, VramWord(0x5000 + 2 * 0x40));
}

#[test]
fn bg2s_lines_land_the_tilemap_gap_before_bg1s() {
    let uploads = exposed(LevelLayer::Two, BlockLine::Column, 17, 1, 0x0800);
    assert_eq!(uploads.bg2.column.first.to, VramWord(0x4C00 + 0x40 + 2));
    assert_eq!(uploads.bg2.column.requests, UploadRequests(1));
    assert_eq!(uploads.bg1, LayerUploads::default());
}

#[test]
fn a_second_line_before_the_nmi_counts_another_request() {
    let mut uploads = exposed(LevelLayer::One, BlockLine::Row, 0, 0, 0);
    let exposure = LayerExposure {
        layer: LevelLayer::One,
        line: BlockLine::Row,
        level: BlockPoint::default(),
        background: BlockPoint { x: 1, y: 0 },
    };
    expose(&mut uploads, &exposure, VramWord(0), [look(0x100)]);
    assert_eq!(uploads.bg1.row.requests, UploadRequests(2));
    // The later line overwrites what it builds and keeps the rest of the buffer.
    assert_eq!(
        uploads.bg1.row.top[..3],
        [entry(0x100), entry(0x101), entry(4)]
    );
}
