use super::{DirectBuffer, read_now, write_now};
use crate::intent::OverlayRows;
use crate::state::video::{BBusRegister, DmaChannel, DmaControl, TilemapEntry, VRAM_WORDS, Video};

/// DMA 1 as a write of `bytes` from the buffer at `offset` in bank `$7E` leaves it.
fn written(offset: u16, bytes: u16) -> DmaChannel {
    DmaChannel {
        control: DmaControl::TO_TWO_REGISTERS,
        destination: BBusRegister::VramData,
        source: offset.wrapping_add(bytes),
        source_bank: 0x7E,
        ..DmaChannel::default()
    }
}

/// Invented rows: `count` cells whose words count up from `first`.
fn rows(row: u16, first: u16, count: u16) -> OverlayRows {
    OverlayRows {
        row,
        cells: (first..first.wrapping_add(count))
            .map(TilemapEntry::from_word)
            .collect(),
    }
}

#[test]
fn a_write_lands_on_its_rows_at_once_and_no_further() {
    let mut video = Video::default();
    video.memory.vram = vec![0x1111; VRAM_WORDS];
    write_now(&mut video, &rows(14, 1, 3), DirectBuffer::Message);
    assert_eq!(video.memory.vram[0x59BF..0x59C4], [0x1111, 1, 2, 3, 0x1111]);
    assert_eq!(video.batch.vram.len(), 0);
    assert_eq!(video.dma[1], written(0x3200, 6));
}

#[test]
fn each_write_leaves_dma_1_past_its_buffer_with_the_indirect_fields_cleared() {
    let mut video = Video::default();
    for (buffer, offset) in [
        (DirectBuffer::Blank, 0x3800),
        (DirectBuffer::Message, 0x3200),
        (DirectBuffer::Backup, 0x4100),
    ] {
        video.dma[1].indirect_bank = 0x7E;
        video.dma[1].table = 0xFFFF;
        video.dma[1].line_counter = 0xFF;
        write_now(&mut video, &rows(4, 0, 0x380), buffer);
        assert_eq!(video.dma[1], written(offset, 0x700));
    }
}

#[test]
fn the_backup_reads_vram_through_dma_1() {
    let mut video = Video::default();
    video.memory.vram[0x5880] = 0xA5C3;
    video.memory.vram[0x5BFF] = 0x7FFF;
    video.dma[1].table = 0xFFFF;
    video.dma[1].line_counter = 0xFF;
    let backup = read_now(&mut video, 4, 28);
    assert_eq!(backup.len(), 0x380);
    assert_eq!((backup[0], backup[0x37F]), (0xA5C3, 0x7FFF));
    assert_eq!(
        video.dma[1],
        DmaChannel {
            control: DmaControl::FROM_TWO_REGISTERS,
            destination: BBusRegister::VramDataRead,
            source: 0x4800,
            source_bank: 0x7E,
            ..DmaChannel::default()
        }
    );
}

#[test]
fn every_word_read_writes_back_unchanged() {
    let mut video = Video::default();
    let stale: Vec<u16> = (0..0x380_u16)
        .map(|index| index.wrapping_mul(0x9E37))
        .collect();
    video.memory.vram[0x5880..0x5C00].copy_from_slice(&stale);
    let backup = read_now(&mut video, 4, 28);
    video.memory.vram[0x5880..0x5C00].fill(0x000E);
    let restore = OverlayRows {
        row: 4,
        cells: backup.into_iter().map(TilemapEntry::from_word).collect(),
    };
    write_now(&mut video, &restore, DirectBuffer::Backup);
    assert_eq!(video.memory.vram[0x5880..0x5C00], stale);
}
