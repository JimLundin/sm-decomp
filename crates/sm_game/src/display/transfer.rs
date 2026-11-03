//! Display uploads and readbacks queued in RAM, decoded one record at a time.
//!
//! Each queue holds packed records that the original drained into DMA
//! channel 1. Records are decoded lazily: a readback's DMA can overwrite the records
//! after it, so the next record is read only after the previous transfer ran.
//! Within one record no register write reaches RAM before the transfer, so a
//! record's fields are read together before its first register write.

pub use super::transfer_types::{
    Mode7Target, Mode7Upload, ScrollingLayout, Source, TilemapUpload, VramReadback,
};

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

/// The current packed Mode 7 queue, preserving linear byte offsets and aliases.
pub trait Mode7Queue {
    fn mode7_queue_tail(&self) -> u16;
    fn set_mode7_queue_tail(&mut self, tail: u16);
    fn set_mode7_queue_byte(&mut self, offset: usize, value: u8);
}

/// Append bank `bank`'s Mode 7 record list at `cursor` to the queue, each
/// record's terminating sentinel stored before its bytes.
pub fn enqueue_mode7_rom<H>(bank: u8, cursor: u16, game: &mut H)
where
    H: Mode7Queue + crate::decompression::DecompressionHost + ?Sized,
{
    let address = (u32::from(bank) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        game.report_invalid_rom_address(address);
    }
    let address = address | 0x8000;
    let mut source = 0;
    let mut destination = usize::from(game.mode7_queue_tail());
    loop {
        let tag = game.read_rom_byte(address, source);
        let length = if tag & 0x80 != 0 {
            9
        } else if tag & 0x40 != 0 {
            7
        } else {
            break;
        };
        // The sentinel precedes the copy. Both cursors advance linearly, without
        // the per-record u16 wrap used by the direct cinematic producers.
        game.set_mode7_queue_byte(destination + length, 0);
        for index in 0..length {
            let value = game.read_rom_byte(address, source + index);
            game.set_mode7_queue_byte(destination + index, value);
        }
        source += length;
        destination += length;
    }
    game.set_mode7_queue_tail(
        u16::try_from(destination & usize::from(u16::MAX)).expect("mode7 queue tail"),
    );
}

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

/// A tilemap queue record's size in bytes, as the cartridge tables hold them.
pub const TILEMAP_RECORD: u16 = 7;

/// Copy the cartridge's queue record `entry` to queue byte `tail` in the original
/// store order: the size, the bank word at 3, the source address, then the destination.
pub fn copy_tilemap_record(
    host: &mut (impl TilemapQueue + ?Sized),
    entry: &sm_data::cartridge::RomRecord,
    tail: usize,
) {
    for field in [0, 3, 2, 5] {
        let [first, second] = entry.word(field).to_ne_bytes();
        host.set_tilemap_queue_byte(TILEMAP_QUEUE + tail + field, first);
        host.set_tilemap_queue_byte(TILEMAP_QUEUE + tail + field + 1, second);
    }
}

/// Append the cartridge's queue record `entry` at the tail and advance the tail past it.
pub fn push_tilemap_record(
    host: &mut (impl TilemapQueue + ?Sized),
    entry: &sm_data::cartridge::RomRecord,
) {
    let tail = host.tilemap_queue_tail();
    copy_tilemap_record(host, entry, usize::from(tail));
    host.set_tilemap_queue_tail(tail.wrapping_add(TILEMAP_RECORD));
}

pub fn drain_tilemaps(host: &mut (impl TransferHost + ?Sized)) {
    struct Reader<'a, H: ?Sized>(&'a mut H);
    impl<H: TransferHost + ?Sized> super::queued_tilemaps::TilemapDrain for Reader<'_, H> {
        fn queue_byte(&self, offset: usize) -> u8 {
            self.0.byte(TILEMAP_QUEUE + offset)
        }
        fn queue_word(&self, offset: usize) -> u16 {
            self.0.word(TILEMAP_QUEUE + offset)
        }
        fn upload_tilemap(&mut self, upload: TilemapUpload) {
            self.0.upload_tilemap(upload);
        }
    }
    super::queued_tilemaps::drain(&mut Reader(host));
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
