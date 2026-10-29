//! Packed VRAM readback requests shared by X-ray backgrounds and Kraid pause.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rtl::WriteReg;
use crate::snes::registers::*;

#[forbid(unsafe_code)]
fn read_word(game: &Game, offset: usize) -> u16 {
    u16::from_ne_bytes([
        game.ram().read_byte(offset),
        game.ram().read_byte(offset + 1),
    ])
}

#[forbid(unsafe_code)]
fn write_word(game: &mut Game, offset: usize, value: u16) {
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(offset, bytes[0]);
    game.ram().write_byte(offset + 1, bytes[1]);
}

#[forbid(unsafe_code)]
pub(super) fn enqueue(
    register: usize,
    vram_offset: u16,
    destination: u16,
    size: u16,
    retain_mode: bool,
    game: &mut Game,
) {
    let tail = word!(game.vram_read_queue_tail);
    let entry = 0x340 + usize::from(tail);
    let target =
        ((u16::from(game.ram().read_byte(register)) & 0xfc) << 8).wrapping_add(vram_offset);
    write_word(game, entry, target);
    if !retain_mode {
        write_word(game, entry + 2, 129);
    }
    // These stores overlap. Stage five of X-ray retains the preceding mode byte.
    write_word(game, entry + 3, 57);
    write_word(game, entry + 4, destination);
    write_word(game, entry + 6, 126);
    write_word(game, entry + 7, size);
    word!(game.vram_read_queue_tail = tail.wrapping_add(9));
}

pub(super) unsafe fn process(game: &mut Game) {
    let tail = word!(game.vram_read_queue_tail) as u8;
    if tail != 0 {
        // Only the low sentinel byte is cleared; the high byte remains live.
        game.ram().write_byte(0x340 + usize::from(tail), 0);
        unsafe {
            WriteReg(VMAIN as u16, 0x80, game);
        }
        // DMA can overwrite later records, so each is decoded after the last ran.
        sm_game::display::transfer::drain_readbacks(&mut unsafe {
            crate::game::display_transfers::RawQueue::new(game)
        });
        game.ram().write_byte(0x360, 0);
    }
}
