//! Mode 7 VRAM and palette transfers, packed in the original RAM queue.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rtl::{WriteReg, WriteRegWord};
use crate::snes::registers::*;

const QUEUE: usize = 0x2d0;

#[forbid(unsafe_code)]
pub(super) fn write_word(tail: u16, field: usize, value: u16, game: &mut Game) {
    let at = QUEUE + usize::from(tail) + field;
    let bytes = value.to_ne_bytes();
    game.g_ram[at] = bytes[0];
    game.g_ram[at + 1] = bytes[1];
}

#[forbid(unsafe_code)]
pub(super) fn advance_ending_tile_upload(game: &mut Game) {
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
        game.g_ram[0x55] = 1;
        word! {game; cinematic_var4 = 0; cinematic_function = 0xdb9d; }
    }
}

#[forbid(unsafe_code)]
fn read_word(game: &Game, at: usize) -> u16 {
    u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]])
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
        game.g_ram[destination + length] = 0;
        for index in 0..length {
            game.g_ram[destination + index] = game.read_rom_byte(address, source + index);
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
    let mut entry = QUEUE;
    loop {
        let tag = game.g_ram[entry];
        if tag & 0xc0 == 0 {
            break;
        }
        // Keep the tag snapshot, but reread every field at its register call.
        unsafe {
            WriteReg(DMAP1 as u16, tag & 0x1f, game);
            WriteRegWord(A1T1L as u16, read_word(game, entry + 1), game);
            WriteReg(A1B1 as u16, game.g_ram[entry + 3], game);
            WriteRegWord(DAS1L as u16, read_word(game, entry + 4), game);
            if tag & 0x80 != 0 {
                WriteReg(
                    BBAD1 as u16,
                    if tag & 0x40 != 0 { 0x19 } else { 0x18 },
                    game,
                );
                WriteRegWord(VMADDL as u16, read_word(game, entry + 6), game);
                WriteReg(VMAIN as u16, game.g_ram[entry + 8], game);
            } else {
                WriteReg(BBAD1 as u16, 0x22, game);
                WriteReg(CGADD as u16, game.g_ram[entry + 6], game);
            }
            WriteReg(MDMAEN as u16, 2, game);
        }
        entry += if tag & 0x80 != 0 { 9 } else { 7 };
    }
    game.g_ram[QUEUE..QUEUE + 2].fill(0);
    word!(game.mode7_vram_write_queue_tail = 0);
}
