//! Kraid sinking through the floor after his death: the sinking Kraid table
//! at $A7:C5E7 and the row clears it queues (bank $A7:C537-$A7:C6FF).
use crate::actors::boss_lease::BossState;
use crate::actors::layouts::KraidLayout as Kraid;
use crate::actors::session_slots::session_slot;
use crate::cartridge::{NativeDiagnostics, native_record};
use crate::display::transfer::{TILEMAP_QUEUE, TilemapQueue};
use sm_data::cartridge::RomRecord;

const BANK: u8 = 0xa7;
/// $A7:C5E7: six-byte rows of Kraid's y, a BG2 tilemap word and a platform
/// crumble routine, ended by a negative y.
const SINKING_TABLE: u16 = 0xc5e7;
const ROW_BYTES: usize = 6;
const ROW_Y: usize = 0;
const ROW_TILEMAP: usize = 2;
const ROW_ROUTINE: usize = 4;
/// A row whose tilemap word has bit 15 set clears nothing.
const NO_CLEAR: u16 = 0x8000;

/// Each row clear copies 64 bytes of blank tilemap from $7E:2FC0.
const CLEAR_BYTES: u16 = 64;
const BLANK_TILEMAP: u16 = 0x2fc0;
const BLANK_TILEMAP_BANK: u8 = 0x7e;
const QUEUE_ENTRY_BYTES: u16 = 7;
/// $7E:0059, the BG2 tilemap base and size register (`reg_BG2SC`).
const BG2_TILEMAP_BASE: usize = 0x59;
const BASE_BITS: i32 = 0xfc;

/// Row `index` of the sinking Kraid table.
pub fn sinking_row(index: usize, game: &mut (impl NativeDiagnostics + ?Sized)) -> RomRecord {
    let table = native_record(BANK, SINKING_TABLE, game);
    RomRecord {
        base: table.base + ROW_BYTES * index,
        image: table.image,
    }
}

/// Queue the blank tilemap row at queue byte `tail` to `tilemap_offset` past
/// the BG2 tilemap base `bg2_base` (BG2SC), clearing a row of Kraid, and set
/// the tail after it.
pub fn queue_row_clear(
    tail: u16,
    tilemap_offset: u16,
    bg2_base: u8,
    queue: &mut (impl TilemapQueue + ?Sized),
) {
    let base = (i32::from(bg2_base) & BASE_BITS) << 8;
    let destination = (i32::from(tilemap_offset) + base) as u16;
    let [size_low, size_high] = CLEAR_BYTES.to_le_bytes();
    let [source_low, source_high] = BLANK_TILEMAP.to_le_bytes();
    let [destination_low, destination_high] = destination.to_le_bytes();
    let entry = [
        size_low,
        size_high,
        source_low,
        source_high,
        BLANK_TILEMAP_BANK,
        destination_low,
        destination_high,
    ];
    let at = TILEMAP_QUEUE + usize::from(tail);
    for (offset, byte) in entry.into_iter().enumerate() {
        queue.set_tilemap_queue_byte(at + offset, byte);
    }
    queue.set_tilemap_queue_tail(tail.wrapping_add(QUEUE_ENTRY_BYTES));
}

/// Handle Kraid sinking ($A7:C59F): find the row for Kraid's y, queue its
/// clear unless it has none, and return its crumble routine in bank $A7.
/// None once the table ends. The table walk of `Kraid_HandleSinking`.
pub fn handle_sinking(game: &mut (impl BossState + ?Sized)) -> Option<u32> {
    for index in 0.. {
        let row = sinking_row(index, game);
        let y = row.word(ROW_Y) as i16;
        if y < 0 {
            return None;
        }
        if i32::from(y) == i32::from(session_slot!(game, Kraid[0].base.y_pos)) {
            if row.word(ROW_TILEMAP) & NO_CLEAR == 0 {
                let tail = game.tilemap_queue_tail();
                let bg2_base = game.slot_byte(BG2_TILEMAP_BASE);
                queue_row_clear(tail, row.word(ROW_TILEMAP), bg2_base, game);
            }
            return Some(u32::from(BANK) << 16 | u32::from(row.word(ROW_ROUTINE)));
        }
    }
    None
}
