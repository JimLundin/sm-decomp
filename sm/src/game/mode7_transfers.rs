//! Mode 7 VRAM and palette transfers, packed in the original RAM queue.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

const QUEUE: usize = sm_game::display::transfer::MODE7_QUEUE;

#[forbid(unsafe_code)]
pub(super) fn write_word(tail: u16, field: usize, value: u16, game: &mut Game) {
    let at = QUEUE + usize::from(tail) + field;
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}

#[forbid(unsafe_code)]
pub(crate) fn advance_ending_tile_upload(game: &mut Game) {
    use crate::ram_map::word;
    let page = word!(game.cinematic_var4);
    let tail = word!(game.mode7_vram_write_queue_tail);
    let high_plane = word!(game.cinematic_var4) as i32 - 8 & 0x8000 != 0;
    write_word(tail, 0, if high_plane { 192 } else { 128 }, game);
    // Both original tables have 16 pages; bounds failure follows the first store.
    assert!(page < 16);
    let source = page * 0x800;
    write_word(tail, 1, source, game);
    write_word(tail, 3, 127, game);
    write_word(tail, 4, 2048, game);
    write_word(tail, 6, source & 0x3fff, game);
    write_word(tail, 8, if high_plane { 128 } else { 0 }, game);
    word!(game.mode7_vram_write_queue_tail = tail.wrapping_add(9));
    if word!(game.cinematic_var4 += 1, reread) as i32 - 16 & 0x8000 == 0 {
        game.ram().write_byte(0x55, 1);
        word! {game; cinematic_var4 = 0; cinematic_function = 0xdb9d; }
    }
}

#[forbid(unsafe_code)]
fn read_word(game: &Game, at: usize) -> u16 {
    u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}

#[forbid(unsafe_code)]
pub(super) fn enqueue_rom(bank: u8, cursor: u16, game: &mut Game) {
    let address = (u32::from(bank) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        crate::rtl::report_invalid_rom_address(address, game);
    }
    let address = address | 0x8000;
    let mut source = 0;
    let mut destination = QUEUE + usize::from(word!(game.mode7_vram_write_queue_tail));
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
        game.ram().write_byte(destination + length, 0);
        for index in 0..length {
            let value = game.read_rom_byte(address, source + index);
            game.ram().write_byte(destination + index, value);
        }
        source += length;
        destination += length;
    }
    word!(game.mode7_vram_write_queue_tail = (destination - QUEUE) as u16);
}

pub(super) unsafe fn process(game: &mut Game) {
    if word!(game.mode7_vram_write_queue_tail) == 0 {
        return;
    }
    sm_game::display::transfer::drain_mode7(&mut unsafe {
        crate::game::display_transfers::RawQueue::new(game)
    });
    game.ram().fill(QUEUE..QUEUE + 2, 0);
    word!(game.mode7_vram_write_queue_tail = 0);
}
