//! The Torizo corpse's tables in bank $A9: the VRAM transfers that queue its
//! rotting tiles and the offsets that copy its sand heap one line at a time.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $A9:D549 and $A9:D583: the rotting transfers queued on even and odd frames,
/// seven rows each then a zero size that ends the table.
pub const EVEN_TRANSFER_ADDRESS: u32 = 0xa9_d549;
pub const ODD_TRANSFER_ADDRESS: u32 = 0xa9_d583;
pub const TRANSFER_ROWS: usize = 7;
pub const TRANSFER_BYTES: usize = TRANSFER_ROWS * 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileTransfer {
    pub size: u16,
    /// Source bank * 100h.
    pub source_bank: u16,
    pub source_address: u16,
    pub vram_address: u16,
}

pub type TileTransfers = Rows<TileTransfer, TRANSFER_ROWS>;

pub fn import_tile_transfers(bytes: &[u8]) -> Result<TileTransfers, LengthError> {
    Rows::import(
        bytes,
        |[size, source_bank, source_address, vram_address]| TileTransfer {
            size,
            source_bank,
            source_address,
            vram_address,
        },
    )
}

/// $A9:D67C destination and $A9:D69C source offsets for the sixteen lines of
/// sand heap tile data.
pub const SAND_HEAP_LINE_ADDRESS: u32 = 0xa9_d67c;
pub const SAND_HEAP_SOURCE_ADDRESS: u32 = 0xa9_d69c;
pub const SAND_HEAP_LINE_ROWS: usize = 16;
pub const SAND_HEAP_LINE_BYTES: usize = SAND_HEAP_LINE_ROWS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandHeapLine {
    pub destination: u16,
    pub source: u16,
}

pub type SandHeapLines = Rows<SandHeapLine, SAND_HEAP_LINE_ROWS>;

pub fn import_sand_heap_lines(bytes: &[u8]) -> Result<SandHeapLines, LengthError> {
    Rows::import_columns(bytes, |[destination, source]| SandHeapLine {
        destination,
        source,
    })
}
