//! VRAM write-queue entries copied from cartridge tables through the native image.
#![forbid(unsafe_code)]
use crate::game::constants::bank_a6::{ADDR_BYTE_A6C15D, ADDR_STRU_A6C3B8};
use crate::game::session_runtime::native_record as read;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::RomRecord;
use crate::types::EnemyData;

const QUEUE: usize = 0xd0;
const ENTRY: u16 = 7;

/// Copy the seven bytes to queue byte `tail` in the original store order:
/// size, the bank word at 3, the source address, then the destination.
fn copy(entry: &RomRecord, tail: usize, game: &mut Game) {
    for field in [0, 3, 2, 5] {
        let bytes = entry.word(field).to_ne_bytes();
        game.ram().write_byte(QUEUE + tail + field, bytes[0]);
        game.ram().write_byte(QUEUE + tail + field + 1, bytes[1]);
    }
}

fn push(entry: &RomRecord, game: &mut Game) {
    let tail = word!(game.VRAM_WRITE_QUEUE_TAIL);
    copy(entry, usize::from(tail), game);
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(ENTRY));
}

/// Ceres Ridley's single getaway tile entry.
/// Was Ridley_Func_59.
pub(crate) fn queue_getaway_tiles(game: &mut Game) {
    let entry = read(0xa6, ADDR_BYTE_A6C15D as u16, game);
    push(&entry, game);
}

/// Every entry of the table at $A6:C3B8 up to its zero size, then one tail store.
/// Was Ridley_Func_62.
pub(crate) fn queue_escape_tile_table(game: &mut Game) {
    let (mut cursor, mut tail) = (
        ADDR_STRU_A6C3B8 as u16,
        usize::from(word!(game.VRAM_WRITE_QUEUE_TAIL)),
    );
    loop {
        let entry = read(0xa6, cursor, game);
        if entry.word(0) == 0 {
            break;
        }
        copy(&entry, tail, game);
        cursor = cursor.wrapping_add(ENTRY);
        tail += usize::from(ENTRY);
    }
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail as u16);
}

/// One escape-timer entry per call; 1 once the table at enemy 0's cursor is exhausted.
/// Was ProcessEscapeTimerTileTransfers.
pub(crate) fn queue_escape_timer_tiles(game: &mut Game) -> u8 {
    let cursor = slot!(game, EnemyData[0].ai_var_e);
    let entry = read(0xa6, cursor, game);
    if entry.word(0) != 0 {
        push(&entry, game);
        slot!(game, EnemyData[0].ai_var_e = cursor.wrapping_add(ENTRY));
        if read(0xa6, cursor.wrapping_add(ENTRY), game).word(0) != 0 {
            return 0;
        }
    }
    1
}

/// Queue the Mother Brain sprite-tile entry at `at` in `bank` unless its size
/// is 0; the size. The body's cursor through the list is the fight's.
pub(crate) fn queue_sprite_tile_entry(bank: u8, at: u16, game: &mut Game) -> u16 {
    let entry = read(bank, at, game);
    let size = entry.word(0);
    if size != 0 {
        push(&entry, game);
    }
    size
}
