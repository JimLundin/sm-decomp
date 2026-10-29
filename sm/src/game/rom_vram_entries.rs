//! VRAM write-queue entries copied from cartridge tables through the native image.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game::constants::bank_a6::{addr_byte_A6C15D, addr_stru_A6C3B8};
use crate::game::session_runtime::native_rom_diagnostic;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::{RomRecord, lorom_offset};
use crate::types::{Enemy_MotherBrain, EnemyData};

const QUEUE: usize = 0xd0;
const ENTRY: u16 = 7;

/// The entry at `bank:cursor`; a cursor below $8000 reports the invalid-window
/// diagnostic once, as each original pointer fetch did, and still reads.
fn read(bank: u8, cursor: u16, game: &mut Game) -> RomRecord {
    let address = (u32::from(bank) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        native_rom_diagnostic(address, game);
    }
    RomRecord {
        image: game.capture_native_rom(),
        base: lorom_offset(address) as usize,
    }
}

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
pub(crate) fn Ridley_Func_59(game: &mut Game) {
    let entry = read(0xa6, addr_byte_A6C15D as u16, game);
    push(&entry, game);
}

/// Every entry of the table at $A6:C3B8 up to its zero size, then one tail store.
pub(crate) fn Ridley_Func_62(game: &mut Game) {
    let (mut cursor, mut tail) = (
        addr_stru_A6C3B8 as u16,
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
pub(crate) fn ProcessEscapeTimerTileTransfers(game: &mut Game) -> u8 {
    let cursor = slot!(game, EnemyData[0].ai_var_E);
    let entry = read(0xa6, cursor, game);
    if entry.word(0) != 0 {
        push(&entry, game);
        slot!(game, EnemyData[0].ai_var_E = cursor.wrapping_add(ENTRY));
        if read(0xa6, cursor.wrapping_add(ENTRY), game).word(0) != 0 {
            return 0;
        }
    }
    1
}

/// One Mother Brain sprite-tile entry per call from `bank`, starting at `start`;
/// 1 and a cleared cursor once the table is exhausted.
pub(crate) fn ProcessSpriteTilesTransfers(bank: u8, start: u16, game: &mut Game) -> u8 {
    let mut cursor = slot!(game, Enemy_MotherBrain[0].var_22);
    if cursor == 0 {
        cursor = start;
    }
    let entry = read(bank, cursor, game);
    let mut size = entry.word(0);
    if size != 0 {
        push(&entry, game);
        slot!(
            game,
            Enemy_MotherBrain[0].var_22 = cursor.wrapping_add(ENTRY)
        );
        size = read(bank, slot!(game, Enemy_MotherBrain[0].var_22), game).word(0);
        if size != 0 {
            return 0;
        }
    }
    slot!(game, Enemy_MotherBrain[0].var_22 = size);
    1
}
