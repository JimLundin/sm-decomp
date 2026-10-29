//! Kraid's sink table at $A7:C5E7: six-byte rows of (y, VRAM row, callback),
//! ended by a negative y.
#![forbid(unsafe_code)]
use crate::game::enemy_layouts::KraidLayout as Kraid;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::RomRecord;

const TABLE: u16 = 0xc5e7;
const ROW: usize = 6;
const QUEUE: usize = 0xd0;

pub(crate) fn row(index: usize, game: &mut Game) -> RomRecord {
    let table = native_record(0xa7, TABLE, game);
    RomRecord {
        base: table.base + ROW * index,
        image: table.image,
    }
}

fn store_word(at: usize, value: u16, game: &mut Game) {
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}

/// Queue the 64-byte tile row from $7E:2FC0 at queue byte `tail` to the row's
/// VRAM word on the current BG1 page, then set the tail after it.
pub(crate) fn queue_tiles(tail: u16, vram_row: u16, game: &mut Game) {
    let at = QUEUE + usize::from(tail);
    store_word(at, 64, game);
    store_word(at + 2, 0x2fc0, game);
    game.ram().write_byte(at + 4, 0x7e);
    let page = (i32::from(game.ram().read_byte(0x59)) & 0xfc) << 8;
    store_word(at + 5, (i32::from(vram_row) + page) as u16, game);
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

/// The row matching Kraid's y: queue its tiles unless bit 15 of its VRAM word
/// is set, and return its callback in bank $A7. None once the table ends.
pub(crate) fn sink_row(game: &mut Game) -> Option<u32> {
    for index in 0.. {
        let entry = row(index, game);
        let y = entry.word(0) as i16;
        if y < 0 {
            return None;
        }
        if i32::from(y) == i32::from(slot!(game, Kraid[0].base.y_pos)) {
            if entry.word(2) & 0x8000 == 0 {
                let tail = word!(game.vram_write_queue_tail);
                queue_tiles(tail, entry.word(2), game);
            }
            return Some(u32::from(entry.word(4)) | 0xa70000);
        }
    }
    None
}
