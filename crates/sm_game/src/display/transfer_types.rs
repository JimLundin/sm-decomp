//! Pure display transfer records and scrolling layout.
#![forbid(unsafe_code)]
/// A CPU-bus source: a 24-bit address split as the DMA registers hold it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub address: u16,
    pub bank: u8,
}

/// Where a Mode 7 queue record writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode7Target {
    /// VRAM at `address` with the VMAIN `increment` byte. `high_byte` selects
    /// the VMDATAH port (Mode 7 pixels) instead of VMDATAL (the tilemap).
    Vram {
        high_byte: bool,
        address: u16,
        increment: u8,
    },
    /// CGRAM from colour `index`.
    Cgram { index: u8 },
}

/// One Mode 7 queue record: `bytes` from `source` through DMA mode `mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode7Upload {
    pub mode: u8,
    pub source: Source,
    pub bytes: u16,
    pub target: Mode7Target,
}

/// One VRAM readback record: `bytes` from VRAM `vram_address` into `destination`,
/// using DMA `control` and B-bus `port` (normally $81 and RDVRAML).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VramReadback {
    pub vram_address: u16,
    pub control: u8,
    pub port: u8,
    pub destination: Source,
    pub bytes: u16,
}

/// One tilemap queue record: `bytes` from `source` to VRAM `vram_address`.
/// `column` selects the 32-word VMAIN increment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TilemapUpload {
    pub bytes: u16,
    pub source: Source,
    pub column: bool,
    pub vram_address: u16,
}

/// How a scrolling tilemap buffer's two halves lie in VRAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollingLayout {
    Column,
    Row,
}

impl ScrollingLayout {
    /// The second half's VRAM address for a first half at `address`.
    pub fn second_half_destination(self, address: u16) -> u16 {
        match self {
            Self::Column => address.wrapping_add(1),
            Self::Row => address | 0x20,
        }
    }
}
