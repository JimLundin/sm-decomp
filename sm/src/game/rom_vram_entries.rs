//! VRAM write-queue entries copied from cartridge tables through the native image.
#![forbid(unsafe_code)]
use crate::game::constants::bank_a6::{ADDR_BYTE_A6C15D, ADDR_STRU_A6C3B8};
use crate::game::session_runtime::native_record as read;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::RomRecord;
use crate::types::EnemyData;
use sm_game::actors::mother_brain::MotherBrainBody;

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
    let tail = word!(game.vram_write_queue_tail);
    copy(entry, usize::from(tail), game);
    word!(game.vram_write_queue_tail = tail.wrapping_add(ENTRY));
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
        usize::from(word!(game.vram_write_queue_tail)),
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
    word!(game.vram_write_queue_tail = tail as u16);
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

/// One Mother Brain sprite-tile entry per call from `bank`, starting at `start`;
/// 1 and a cleared cursor once the table is exhausted.
/// Was ProcessSpriteTilesTransfers.
pub(crate) fn queue_mother_brain_sprite_tiles(bank: u8, start: u16, game: &mut Game) -> u8 {
    let mut cursor = slot!(game, MotherBrainBody[0].sprite_tile_transfer);
    if cursor == 0 {
        cursor = start;
    }
    let entry = read(bank, cursor, game);
    let mut size = entry.word(0);
    if size != 0 {
        push(&entry, game);
        slot!(
            game,
            MotherBrainBody[0].sprite_tile_transfer = cursor.wrapping_add(ENTRY)
        );
        size = read(
            bank,
            slot!(game, MotherBrainBody[0].sprite_tile_transfer),
            game,
        )
        .word(0);
        if size != 0 {
            return 0;
        }
    }
    slot!(game, MotherBrainBody[0].sprite_tile_transfer = size);
    1
}
