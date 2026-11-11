//! The overlay rows the message box moves itself: straight between VRAM and work RAM through
//! DMA 1 while the NMI lags, around the NMI's batch.
//!
//! Each move leaves DMA 1's registers as its own DMA does: its buffer's address on the A bus,
//! run on past what it moved, the count run down, and the indirect bank, the table address and
//! the line counter cleared, because the box writes zero to them before each DMA. The next
//! NMI's uploads start from those registers, so they are observed.

use data::dma::{BusAddress, MESSAGE_BOX_BACKUP, MESSAGE_BOX_BLANK, MESSAGE_BOX_ROWS};

use super::{ROW_WORDS, WORD_BYTES, overlay_row};
use crate::commit::{VRAM_CHANNEL, write_words};
use crate::intent::OverlayRows;
use crate::state::video::{BBusRegister, DmaChannel, DmaControl, Video};

/// The work RAM buffer a direct move goes through, which DMA 1 still points into after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectBuffer {
    /// The box's blank rows.
    Blank,
    /// The box's rows: its borders around the message.
    Message,
    /// The backup of the rows the box covers.
    Backup,
}

impl DirectBuffer {
    /// Where the buffer is on the A bus.
    const fn address(self) -> BusAddress {
        match self {
            Self::Blank => MESSAGE_BOX_BLANK,
            Self::Message => MESSAGE_BOX_ROWS,
            Self::Backup => MESSAGE_BOX_BACKUP,
        }
    }
}

/// Write `rows` to the overlay at once, through DMA 1 from `buffer`.
pub fn write_now(video: &mut Video, rows: &OverlayRows, buffer: DirectBuffer) {
    write_words(
        &mut video.memory.vram,
        overlay_row(rows.row),
        1,
        rows.cells.iter().map(|cell| cell.word()),
    );
    if let Some(channel) = video.dma.get_mut(VRAM_CHANNEL) {
        channel.control = DmaControl::TO_TWO_REGISTERS;
        channel.destination = BBusRegister::VramData;
        channel.moved(buffer, rows.cells.len());
    }
}

/// Read `count` of the overlay's rows from `row` at once, through DMA 1 into the backup: the
/// words VRAM holds there, including cells no write has drawn since the screen last showed them.
pub fn read_now(video: &mut Video, row: u16, count: u16) -> Vec<u16> {
    let from = usize::from(overlay_row(row).0);
    let cells = usize::from(count.wrapping_mul(ROW_WORDS));
    let read = video
        .memory
        .vram
        .get(from..from.saturating_add(cells))
        .map_or_else(Vec::new, <[u16]>::to_vec);
    if let Some(channel) = video.dma.get_mut(VRAM_CHANNEL) {
        channel.control = DmaControl::FROM_TWO_REGISTERS;
        channel.destination = BBusRegister::VramDataRead;
        channel.moved(DirectBuffer::Backup, cells);
    }
    read
}

impl DmaChannel {
    /// One of the box's DMAs of `cells` through `buffer`, with the indirect fields cleared.
    fn moved(&mut self, buffer: DirectBuffer, cells: usize) {
        self.transferred(
            buffer.address(),
            cells.wrapping_mul(usize::from(WORD_BYTES)),
        );
        self.indirect_bank = 0;
        self.table = 0;
        self.line_counter = 0;
    }
}

#[cfg(test)]
mod tests;
