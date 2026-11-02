//! VRAM write-queue entries copied from cartridge tables through the native image.
#![forbid(unsafe_code)]
use crate::constants::bank_a6::{ADDR_BYTE_A6C15D, ADDR_STRU_A6C3B8};
use crate::display::transfer::TilemapQueue;
use sm_data::cartridge::RomRecord;
fn read<H: Host>(bank: u8, address: u16, game: &mut H) -> RomRecord {
    game.native_actor_record(bank, address)
}
pub trait Host: crate::actors::boss_fields_owned::State + TilemapQueue {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord;
}

const QUEUE: usize = 0xd0;
const ENTRY: u16 = 7;

/// Copy the seven bytes to queue byte `tail` in the original store order:
/// size, the bank word at 3, the source address, then the destination.
fn copy<H: Host>(entry: &RomRecord, tail: usize, game: &mut H) {
    for field in [0, 3, 2, 5] {
        let bytes = entry.word(field).to_ne_bytes();
        game.set_tilemap_queue_byte(QUEUE + tail + field, bytes[0]);
        game.set_tilemap_queue_byte(QUEUE + tail + field + 1, bytes[1]);
    }
}

fn push<H: Host>(entry: &RomRecord, game: &mut H) {
    let tail = game.tilemap_queue_tail();
    copy(entry, usize::from(tail), game);
    game.set_tilemap_queue_tail(tail.wrapping_add(ENTRY));
}

/// Ceres Ridley's single getaway tile entry.
/// Was Ridley_Func_59.
pub fn queue_getaway_tiles<H: Host>(game: &mut H) {
    let entry = read(0xa6, ADDR_BYTE_A6C15D as u16, game);
    push(&entry, game);
}

/// Every entry of the table at $A6:C3B8 up to its zero size, then one tail store.
/// Was Ridley_Func_62.
pub fn queue_escape_tile_table<H: Host>(game: &mut H) {
    let (mut cursor, mut tail) = (
        ADDR_STRU_A6C3B8 as u16,
        usize::from(game.tilemap_queue_tail()),
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
    game.set_tilemap_queue_tail(tail as u16);
}

/// One escape-timer entry per call; 1 once the table at enemy 0's cursor is exhausted.
/// Was ProcessEscapeTimerTileTransfers.
pub fn queue_escape_timer_tiles<H: Host>(game: &mut H) -> u8 {
    let cursor = game.session().actors.slots[0].variables[4];
    let entry = read(0xa6, cursor, game);
    if entry.word(0) != 0 {
        push(&entry, game);
        game.session_mut().actors.slots[0].variables[4] = cursor.wrapping_add(ENTRY);
        if read(0xa6, cursor.wrapping_add(ENTRY), game).word(0) != 0 {
            return 0;
        }
    }
    1
}

/// One Mother Brain sprite-tile entry per call from `bank`, starting at `start`;
/// 1 and a cleared cursor once the table is exhausted.
/// Was ProcessSpriteTilesTransfers.
pub fn queue_mother_brain_sprite_tiles<H: Host>(bank: u8, start: u16, game: &mut H) -> u8 {
    // Fixed Mother Brain body 0 cursor physically aliases Botwoon head 0 at $8004.
    let mut cursor = game.session().actors.extended.botwoon_head[0].pre_death_counter;
    if cursor == 0 {
        cursor = start;
    }
    let entry = read(bank, cursor, game);
    let mut size = entry.word(0);
    if size != 0 {
        push(&entry, game);
        game.session_mut().actors.extended.botwoon_head[0].pre_death_counter =
            cursor.wrapping_add(ENTRY);
        size = read(
            bank,
            game.session().actors.extended.botwoon_head[0].pre_death_counter,
            game,
        )
        .word(0);
        if size != 0 {
            return 0;
        }
    }
    game.session_mut().actors.extended.botwoon_head[0].pre_death_counter = size;
    1
}

impl Host for crate::actors::owned_execution::PreparedActorState<'_> {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord {
        crate::actors::owned_execution::PreparedActorState::native_actor_record(self, bank, address)
    }
}
