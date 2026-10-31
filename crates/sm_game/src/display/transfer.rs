//! Display uploads and readbacks queued in RAM, decoded one record at a time.
//!
//! Each queue holds packed records that the original drained into DMA
//! channel 1. Records are decoded lazily: a readback's DMA can overwrite the records
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

/// The tilemap queue's bytes and its tail ($7E:0330), the byte offset of its next record.
pub trait TilemapQueue {
    fn tilemap_queue_tail(&self) -> u16;
    fn set_tilemap_queue_tail(&mut self, tail: u16);
    fn set_tilemap_queue_byte(&mut self, at: usize, value: u8);
}

/// Append `upload` at the tilemap queue's tail and advance the tail past it. The
/// destination word carries `column`, as the queue's records do.
pub fn queue_tilemap(host: &mut (impl TilemapQueue + ?Sized), upload: TilemapUpload) {
    let tail = host.tilemap_queue_tail();
    let [size_low, size_high] = upload.bytes.to_le_bytes();
    let [source_low, source_high] = upload.source.address.to_le_bytes();
    let [destination_low, destination_high] = upload.vram_address.to_le_bytes();
    let record = [
        size_low,
        size_high,
        source_low,
        source_high,
        upload.source.bank,
        destination_low,
        destination_high,
    ];
    for (offset, byte) in record.into_iter().enumerate() {
        host.set_tilemap_queue_byte(TILEMAP_QUEUE + usize::from(tail) + offset, byte);
    }
    host.set_tilemap_queue_tail(tail.wrapping_add(7));
}

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

/// Program DMA channel 1's mode, 24-bit source and byte count, in the
/// original register order.
pub fn setup_transfer<H: crate::display::sequence::Registers + ?Sized>(
    mode: u16,
    source: u32,
    bytes: u16,
    game: &mut H,
) {
    use crate::display::registers::{A1B1, A1T1L, DAS1L, DMAP1};
    game.write_register_word(DMAP1 as u16, mode);
    game.write_register_word(A1T1L as u16, source as u16);
    game.write_register(A1B1 as u16, (source >> 16) as u8);
    game.write_register_word(DAS1L as u16, bytes);
}

/// Immediate DMA starts: register writes, and a start ($420B) that may read
/// the room's prepared assets instead of the loaded image.
pub trait RoomDma: crate::display::sequence::Registers {
    fn start_room_dma(
        &mut self,
        mask: u8,
        assets: Option<&crate::room_entry::prepared_assets::PreparedRoomAssets>,
    );
}

/// Copy `size` bytes from `source` to VRAM `destination` now, under forced
/// blank, then show the screen at full brightness. Was `CopyToVramNow`.
pub fn copy_to_vram_now<H: RoomDma + ?Sized>(
    destination: u16,
    source: u32,
    size: u16,
    assets: Option<&crate::room_entry::prepared_assets::PreparedRoomAssets>,
    game: &mut H,
) {
    use crate::display::control::{Brightness, DisplayRegister as _};
    use crate::display::registers::{INIDISP, VMADDL, VMAIN};
    let blank = Brightness::BLANK_DARK;
    game.write_register(blank.register(), blank.value());
    game.write_register_word(VMADDL as u16, destination);
    setup_transfer(0x1801, source, size, game);
    game.write_register(VMAIN as u16, 0x80);
    game.start_room_dma(2, assets);
    game.write_register(INIDISP as u16, Brightness::FULL.byte());
}
