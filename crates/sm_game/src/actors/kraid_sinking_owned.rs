//! Kraid's exact sinking-table walk and ordered row upload.
#![forbid(unsafe_code)]
use crate::display::transfer::{TILEMAP_QUEUE, TilemapQueue};
use sm_data::cartridge::RomRecord;

pub trait Host: TilemapQueue {
    fn native_sinking_table(&mut self) -> RomRecord;
    fn kraid_y(&self) -> u16;
    fn bg2_tilemap_base(&self) -> u8;
}
pub fn sinking_row(index: usize, host: &mut impl Host) -> RomRecord {
    let table = host.native_sinking_table();
    RomRecord {
        base: table.base + 6 * index,
        image: table.image,
    }
}
pub fn queue_row_clear(tail: u16, offset: u16, bg2_base: u8, queue: &mut impl TilemapQueue) {
    let destination = offset.wrapping_add(u16::from(bg2_base & 0xfc) << 8);
    let [destination_low, destination_high] = destination.to_le_bytes();
    let entry = [64, 0, 0xc0, 0x2f, 0x7e, destination_low, destination_high];
    let at = TILEMAP_QUEUE + usize::from(tail);
    for (offset, byte) in entry.into_iter().enumerate() {
        queue.set_tilemap_queue_byte(at + offset, byte);
    }
    queue.set_tilemap_queue_tail(tail.wrapping_add(7));
}
pub fn handle_sinking(host: &mut impl Host) -> Option<u32> {
    for index in 0.. {
        let row = sinking_row(index, host);
        let y = i16::from_ne_bytes(row.word(0).to_ne_bytes());
        if y < 0 {
            return None;
        }
        if i32::from(y) == i32::from(host.kraid_y()) {
            if row.word(2) & 0x8000 == 0 {
                let tail = host.tilemap_queue_tail();
                let base = host.bg2_tilemap_base();
                queue_row_clear(tail, row.word(2), base, host);
            }
            return Some(0xa70000 | u32::from(row.word(4)));
        }
    }
    None
}
