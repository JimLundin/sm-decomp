//! Where the original's NMI DMAs read its work RAM buffers from.
//!
//! A DMA leaves its A-bus address in the channel's registers, past the bytes it read, and the
//! next frame's state holds those registers. The engine keeps its buffers as values, so these
//! are the one place their addresses in the original's memory are named.

use canonical::Canonical;

use crate::pose::tiles::GraphicsRef;

/// An A-bus address: a bank and the offset in it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BusAddress {
    /// The bank.
    pub bank: u8,
    /// The offset in it.
    pub offset: u16,
}

impl BusAddress {
    /// The address `bytes` on, within the bank as a DMA's A-bus address stays.
    #[must_use]
    pub const fn advanced(self, bytes: u16) -> Self {
        Self {
            bank: self.bank,
            offset: self.offset.wrapping_add(bytes),
        }
    }
}

impl From<GraphicsRef> for BusAddress {
    fn from(GraphicsRef { bank, offset }: GraphicsRef) -> Self {
        Self { bank, offset }
    }
}

/// The OAM buffer, which `$80:933A` DMAs to OAM through channel 0.
pub const OAM_BUFFER: BusAddress = BusAddress {
    bank: 0x00,
    offset: 0x0370,
};

/// The palette buffer, which `$80:933A` DMAs to CGRAM through channel 1.
pub const PALETTE_BUFFER: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0xC000,
};

/// The HUD tilemap, rows 1 to 3, which `$80:9CAD` queues for the VRAM write table.
pub const HUD_TILEMAP_BUFFER: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0xC608,
};

/// BG1's block column buffer, the left 8x8 column then the right, which `$80:8CD8` DMAs from
/// (bank `$80` log, the column builder `$80:A9DE`).
pub const BG1_COLUMN_BUFFER: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0xC8C8,
};

/// BG1's block row buffer, the top 8x8 row then the bottom, which `$80:8DAC` DMAs from (the
/// row builder `$80:AB78`).
pub const BG1_ROW_BUFFER: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0xC948,
};

/// The PLM draw's tilemap buffer: the top 8x8 row of the blocks a draw instruction draws, then
/// the bottom row, which the draw (`$84:8DAA`) queues for the VRAM write table.
pub const PLM_DRAW_TILEMAP: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0xC6C8,
};

/// The message box's backup of the BG3 tilemap it covers, which `$85:81B5` DMAs from VRAM and
/// `$85:861F` back.
pub const MESSAGE_BOX_BACKUP: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0x4100,
};

/// The message box's blank tilemap, which `$85:81F3` fills and DMAs to VRAM.
pub const MESSAGE_BOX_BLANK: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0x3800,
};

/// The message box's BG3 Y scroll HDMA table, which `$85:8363` points HDMA channel 6 at.
pub const MESSAGE_BOX_SCROLL_TABLE: BusAddress = BusAddress {
    bank: 0x7E,
    offset: 0x3380,
};

/// How far BG2's column and row buffers lie after BG1's.
pub const BG2_BUFFER_OFFSET: u16 = 0x0108;

#[cfg(test)]
mod tests {
    use super::{BusAddress, PALETTE_BUFFER};

    #[test]
    fn the_address_wraps_within_its_bank() {
        assert_eq!(
            PALETTE_BUFFER.advanced(0x0200),
            BusAddress {
                bank: 0x7E,
                offset: 0xC200
            }
        );
        assert_eq!(
            BusAddress {
                bank: 0x9E,
                offset: 0xFF00
            }
            .advanced(0x0200),
            BusAddress {
                bank: 0x9E,
                offset: 0x0100
            }
        );
    }
}
