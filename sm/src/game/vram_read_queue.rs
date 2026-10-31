//! Packed VRAM readback requests shared by X-ray backgrounds and Kraid pause.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rtl::write_reg;
use crate::snes::registers::*;

pub(super) unsafe fn process(game: &mut Game) {
    let tail = word!(game.VRAM_READ_QUEUE_TAIL) as u8;
    if tail != 0 {
        // Only the low sentinel byte is cleared; the high byte remains live.
        game.ram().write_byte(0x340 + usize::from(tail), 0);
        unsafe {
            write_reg(VMAIN as u16, 0x80, game);
        }
        // DMA can overwrite later records, so each is decoded after the last ran.
        sm_game::display::transfer::drain_readbacks(&mut unsafe {
            crate::game::display_transfers::RawQueue::new(game)
        });
        game.ram().write_byte(0x360, 0);
    }
}
