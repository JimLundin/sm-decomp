//! Mode 7 VRAM and palette transfers, packed in the original RAM queue.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

const QUEUE: usize = sm_game::display::transfer::MODE7_QUEUE;

pub(super) use sm_game::display::transfer::enqueue_mode7_rom as enqueue_rom;

pub(super) unsafe fn process(game: &mut Game) {
    if word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL) == 0 {
        return;
    }
    sm_game::display::transfer::drain_mode7(&mut unsafe {
        crate::game::display_transfers::RawQueue::new(game)
    });
    game.ram().fill(QUEUE..QUEUE + 2, 0);
    word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL = 0);
}

#[forbid(unsafe_code)]
impl sm_game::display::transfer::Mode7Queue for Game {
    fn mode7_queue_tail(&self) -> u16 {
        word!(self.MODE7_VRAM_WRITE_QUEUE_TAIL)
    }
    fn set_mode7_queue_tail(&mut self, tail: u16) {
        word!(self.MODE7_VRAM_WRITE_QUEUE_TAIL = tail);
    }
    fn set_mode7_queue_byte(&mut self, offset: usize, value: u8) {
        self.ram().write_byte(QUEUE + offset, value);
    }
}
