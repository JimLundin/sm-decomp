//! Display uploads and readbacks queued in RAM, decoded one record at a time.
//!
//! Each queue holds packed records that the original drained into DMA channel
//! 1. Records are decoded lazily: a readback's DMA can overwrite the records
//! after it, so the next record is read only after the previous transfer ran.
//! Within one record no register write reaches RAM before the transfer, so a
//! record's fields are read together before its first register write.

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

/// RAM reads for a queue and the sink that performs each decoded transfer.
pub trait TransferHost {
    fn byte(&self, at: usize) -> u8;
    /// A native-order word at `at`, as the original's unaligned loads read it.
    fn word(&self, at: usize) -> u16;
    fn upload_mode7(&mut self, upload: Mode7Upload);
    fn read_vram(&mut self, readback: VramReadback);
    fn upload_tilemap(&mut self, upload: TilemapUpload);
}

/// The Mode 7 queue at $7E:02D0: tagged 9-byte VRAM and 7-byte CGRAM records,
/// ended by a tag without bits 6 or 7.
pub const MODE7_QUEUE: usize = 0x2d0;

pub fn drain_mode7(host: &mut (impl TransferHost + ?Sized)) {
    let mut entry = MODE7_QUEUE;
    loop {
        let tag = host.byte(entry);
        if tag & 0xc0 == 0 {
            break;
        }
        let vram = tag & 0x80 != 0;
        let target = if vram {
            Mode7Target::Vram {
                high_byte: tag & 0x40 != 0,
                address: host.word(entry + 6),
                increment: host.byte(entry + 8),
            }
        } else {
            Mode7Target::Cgram {
                index: host.byte(entry + 6),
            }
        };
        let upload = Mode7Upload {
            mode: tag & 0x1f,
            source: Source {
                address: host.word(entry + 1),
                bank: host.byte(entry + 3),
            },
            bytes: host.word(entry + 4),
            target,
        };
        host.upload_mode7(upload);
        entry += if vram { 9 } else { 7 };
    }
}

/// The VRAM readback queue at $7E:0340: 9-byte records ended by a zero word.
pub const READBACK_QUEUE: usize = 0x340;

pub fn drain_readbacks(host: &mut (impl TransferHost + ?Sized)) {
    let mut entry = READBACK_QUEUE;
    loop {
        let vram_address = host.word(entry);
        if vram_address == 0 {
            break;
        }
        let readback = VramReadback {
            vram_address,
            control: host.byte(entry + 2),
            port: host.byte(entry + 3),
            destination: Source {
                address: host.word(entry + 4),
                bank: host.byte(entry + 6),
            },
            bytes: host.word(entry + 7),
        };
        host.read_vram(readback);
        entry += 9;
    }
}

/// The tilemap queue at $7E:00D0: 7-byte records ended by a zero size. The
/// cursor advances linearly, without bank wrap.
pub const TILEMAP_QUEUE: usize = 0xd0;

pub fn drain_tilemaps(host: &mut (impl TransferHost + ?Sized)) {
    let mut entry = TILEMAP_QUEUE;
    loop {
        let bytes = host.word(entry);
        if bytes == 0 {
            break;
        }
        let destination = host.word(entry + 5);
        let upload = TilemapUpload {
            bytes,
            source: Source {
                address: host.word(entry + 2),
                bank: host.byte(entry + 4),
            },
            column: destination & 0x8000 != 0,
            vram_address: destination,
        };
        host.upload_tilemap(upload);
        entry += 7;
    }
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
